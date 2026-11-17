use axum::Json;
use axum::Router;
use axum::extract::{Path, Query, State};
use axum::routing::{get, post};
use chrono::NaiveDate;
use serde::Deserialize;
use uuid::Uuid;

use crate::auth::extractor::AuthUser;
use crate::domain::enums::{StatusHasil, StatusRencana};
use crate::error::{AppError, AppResult};
use crate::repo::hasil::{self, HasilRow, ListFilter, NewHasil, SkorTersimpan};
use crate::repo::notifikasi;
use crate::repo::periode;
use crate::repo::personel;
use crate::repo::rencana;
use crate::repo::{ListResponse, pagination};
use crate::service::akses;
use crate::service::scoring;
use crate::state::AppState;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/", get(list_hasil).post(create_hasil))
        .route("/{id}", get(detail_hasil))
        .route("/{id}/verifikasi", post(verifikasi_hasil))
}

#[derive(Debug, Deserialize)]
pub struct ListQuery {
    pub page: Option<i64>,
    pub per_page: Option<i64>,
    pub satker_id: Option<Uuid>,
    pub personel_id: Option<Uuid>,
    pub status: Option<StatusHasil>,
}

pub async fn list_hasil(
    State(state): State<AppState>,
    user: AuthUser,
    Query(query): Query<ListQuery>,
) -> AppResult<Json<ListResponse<HasilRow>>> {
    let mut scope = crate::routes::personel::scope_for(&state, &user).await?;

    if let Some(satker_id) = query.satker_id {
        if !scope.contains(&satker_id) {
            return Err(AppError::Forbidden(
                "Anda tidak berhak melihat hasil kerja satker tersebut".into(),
            ));
        }
        scope = vec![satker_id];
    }

    let claims = user.claims();
    let personel_id = if claims.is_admin() || claims.is_atasan() || claims.is_pimpinan() {
        query.personel_id
    } else {
        Some(claims.sub)
    };

    let (page, per_page, offset) = pagination(query.page, query.per_page);
    let (rows, total) = hasil::list(
        &state.pool,
        &ListFilter {
            satker_ids: scope,
            personel_id,
            status: query.status,
            limit: per_page,
            offset,
        },
    )
    .await?;

    Ok(Json(ListResponse::new(rows, page, per_page, total)))
}

pub async fn detail_hasil(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<Uuid>,
) -> AppResult<Json<HasilRow>> {
    let row = hasil::find(&state.pool, id)
        .await?
        .ok_or_else(|| AppError::NotFound("Hasil kerja tidak ditemukan".into()))?;

    let boleh = akses::boleh_melihat_personel(
        user.role_scopes(),
        user.satker_akses(),
        user.id(),
        row.personel_id,
        row.satker_id,
    );
    if !boleh {
        return Err(AppError::Forbidden(
            "Anda tidak berhak melihat hasil kerja ini".into(),
        ));
    }

    Ok(Json(row))
}

#[derive(Debug, Deserialize)]
pub struct CreateHasilRequest {
    pub rencana_id: Uuid,
    pub uraian_pelaksanaan: String,
    pub progres_persen: f64,
    pub jumlah_realisasi: Option<f64>,
    pub kendala: Option<String>,
    pub tanggal_realisasi: NaiveDate,
}

pub async fn create_hasil(
    State(state): State<AppState>,
    user: AuthUser,
    Json(req): Json<CreateHasilRequest>,
) -> AppResult<Json<HasilRow>> {
    let personel = personel::active_personel(&state.pool, user.id()).await?;

    let rencana = rencana::find(&state.pool, req.rencana_id)
        .await?
        .ok_or_else(|| AppError::NotFound("Rencana kerja tidak ditemukan".into()))?;

    if rencana.personel_id != personel.id {
        return Err(AppError::Forbidden(
            "Hasil kerja hanya untuk rencana kerja milik Anda".into(),
        ));
    }
    if rencana.status != StatusRencana::Disetujui {
        return Err(AppError::Unprocessable(
            "Rencana kerja harus disetujui atasan sebelum diisi hasilnya".into(),
        ));
    }

    periode::pastikan_terbuka(&state.pool, rencana.periode_id).await?;

    if req.uraian_pelaksanaan.trim().is_empty() {
        return Err(AppError::BadRequest(
            "Uraian pelaksanaan wajib diisi".into(),
        ));
    }
    if !(0.0..=100.0).contains(&req.progres_persen) {
        return Err(AppError::Unprocessable(
            "Progres harus antara 0 dan 100 persen".into(),
        ));
    }

    let id = Uuid::now_v7();
    hasil::create(
        &state.pool,
        &NewHasil {
            id,
            rencana_id: rencana.id,
            personel_id: personel.id,
            satker_id: personel.satker_id,
            uraian_pelaksanaan: req.uraian_pelaksanaan,
            progres_persen: req.progres_persen,
            jumlah_realisasi: req.jumlah_realisasi,
            kendala: req.kendala,
            tanggal_realisasi: req.tanggal_realisasi,
        },
    )
    .await?;

    if let Some(atasan_id) = rencana.atasan_id {
        notifikasi::kirim(
            &state.pool,
            atasan_id,
            "hasil_diajukan",
            "Hasil kerja menunggu verifikasi",
            &format!("{} mengirim hasil kerja", personel.nama),
            Some("hasil"),
            Some(id),
        )
        .await?;
    }

    let row = hasil::find(&state.pool, id)
        .await?
        .ok_or_else(|| AppError::Internal("hasil kerja gagal dibaca".into()))?;
    Ok(Json(row))
}

#[derive(Debug, Deserialize)]
pub struct VerifikasiRequest {
    pub skor_kualitas: Option<f64>,
    pub catatan: Option<String>,
    pub setuju: Option<bool>,
}

pub async fn verifikasi_hasil(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<Uuid>,
    Json(req): Json<VerifikasiRequest>,
) -> AppResult<Json<HasilRow>> {
    if !(user.is_admin_any() || user.is_atasan()) {
        return Err(AppError::Forbidden(
            "Hanya atasan atau admin yang dapat memverifikasi hasil kerja".into(),
        ));
    }

    let hasil_row = hasil::find(&state.pool, id)
        .await?
        .ok_or_else(|| AppError::NotFound("Hasil kerja tidak ditemukan".into()))?;

    let rencana_row = rencana::find(&state.pool, hasil_row.rencana_id)
        .await?
        .ok_or_else(|| AppError::NotFound("Rencana kerja tidak ditemukan".into()))?;
    user.require_satker(rencana_row.satker_id)?;

    let claims = user.claims();
    let boleh = rencana_row.atasan_id == Some(claims.sub) || user.is_admin_any();
    if !boleh {
        return Err(AppError::Forbidden(
            "Hanya atasan langsung yang dapat memverifikasi".into(),
        ));
    }

    let setuju = req.setuju.unwrap_or(true);

    if !setuju {
        let catatan = req.catatan.as_deref().unwrap_or("").trim();
        if catatan.is_empty() {
            return Err(AppError::Unprocessable(
                "Catatan wajib diisi saat meminta revisi".into(),
            ));
        }
        hasil::minta_revisi(&state.pool, id, claims.sub, catatan).await?;
    } else {
        let kualitas = req.skor_kualitas.unwrap_or(80.0);
        if !(0.0..=100.0).contains(&kualitas) {
            return Err(AppError::Unprocessable(
                "Skor kualitas harus antara 0 dan 100".into(),
            ));
        }

        let kuantitas =
            scoring::komponen_kuantitas(hasil_row.jumlah_realisasi, rencana_row.target_kuantitas);
        let waktu = scoring::komponen_waktu(hasil_row.tanggal_realisasi, rencana_row.target_waktu);
        let (k_kuantitas, k_kualitas, k_waktu, total) =
            scoring::skor_hasil(kuantitas, kualitas, waktu);

        hasil::verifikasi(
            &state.pool,
            id,
            claims.sub,
            &SkorTersimpan {
                kuantitas: k_kuantitas,
                kualitas: k_kualitas,
                waktu: k_waktu,
                total,
            },
            req.catatan.as_deref(),
        )
        .await?;

        notifikasi::kirim(
            &state.pool,
            hasil_row.personel_id,
            "hasil_terverifikasi",
            "Hasil kerja terverifikasi",
            &format!("Nilai hasil kerja: {total:.2}"),
            Some("hasil"),
            Some(id),
        )
        .await?;
    }

    let row = hasil::find(&state.pool, id)
        .await?
        .ok_or_else(|| AppError::NotFound("Hasil kerja tidak ditemukan".into()))?;
    Ok(Json(row))
}
