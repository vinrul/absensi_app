use axum::Json;
use axum::Router;
use axum::extract::{Path, Query, State};
use axum::routing::{get, post};
use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::auth::extractor::AuthUser;
use crate::domain::enums::{BerkasOwner, StatusHasil, StatusRencana};
use crate::error::{AppError, AppResult};
use crate::repo::hasil::{self, HasilRow, ListFilter, NewHasil, SkorTersimpan, UpdateHasil};
use crate::repo::lampiran::{self, LampiranRow};
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
        .route("/verifikasi", get(list_verifikasi_hasil))
        .route("/{id}", get(detail_hasil).patch(update_hasil))
        .route("/{id}/ajukan", post(ajukan_hasil))
        .route("/{id}/verifikasi", post(verifikasi_hasil))
}

#[derive(Debug, Serialize)]
pub struct HasilDetail {
    #[serde(flatten)]
    pub hasil: HasilRow,
    pub lampiran: Vec<LampiranRow>,
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
            atasan_id: None,
            hanya_diajukan: false,
            limit: per_page,
            offset,
        },
    )
    .await?;

    Ok(Json(ListResponse::new(rows, page, per_page, total)))
}

pub async fn list_verifikasi_hasil(
    State(state): State<AppState>,
    user: AuthUser,
    Query(query): Query<ListQuery>,
) -> AppResult<Json<ListResponse<HasilRow>>> {
    if !(user.is_admin_any() || user.is_atasan() || user.is_pimpinan_any()) {
        return Err(AppError::Forbidden(
            "Hanya atasan, pimpinan, atau admin yang dapat melihat verifikasi".into(),
        ));
    }

    let scope = crate::routes::personel::scope_for(&state, &user).await?;
    let (page, per_page, offset) = pagination(query.page, query.per_page);

    let (rows, total) = hasil::list(
        &state.pool,
        &ListFilter {
            satker_ids: scope,
            personel_id: None,
            status: None,
            atasan_id: if user.is_admin_any() {
                None
            } else {
                Some(user.id())
            },
            hanya_diajukan: true,
            limit: per_page,
            offset,
        },
    )
    .await?;

    Ok(Json(ListResponse::new(rows, page, per_page, total)))
}

async fn detail_internal(state: &AppState, id: Uuid) -> AppResult<HasilDetail> {
    let hasil = hasil::find(&state.pool, id)
        .await?
        .ok_or_else(|| AppError::NotFound("Hasil kerja tidak ditemukan".into()))?;
    let lampiran_rows = lampiran::list(&state.pool, BerkasOwner::Hasil, id).await?;

    Ok(HasilDetail {
        hasil,
        lampiran: lampiran_rows,
    })
}

pub async fn detail_hasil(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<Uuid>,
) -> AppResult<Json<HasilDetail>> {
    let detail = detail_internal(&state, id).await?;
    let row = &detail.hasil;

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

    Ok(Json(detail))
}

#[derive(Debug, Deserialize)]
pub struct CreateHasilRequest {
    pub rencana_id: Uuid,
    pub uraian_pelaksanaan: String,
    pub progres_persen: f64,
    pub jumlah_realisasi: Option<f64>,
    pub kendala: Option<String>,
    pub tanggal_realisasi: NaiveDate,
    pub ajukan: Option<bool>,
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

    let ajukan = req.ajukan.unwrap_or(true);
    if ajukan && rencana.atasan_id.is_none() {
        return Err(AppError::Unprocessable(
            "Atasan langsung belum ditetapkan untuk akun Anda".into(),
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
            status: if ajukan {
                StatusHasil::Diajukan
            } else {
                StatusHasil::Draft
            },
        },
    )
    .await?;

    if ajukan && let Some(atasan_id) = rencana.atasan_id {
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
pub struct UpdateHasilRequest {
    pub uraian_pelaksanaan: String,
    pub progres_persen: f64,
    pub jumlah_realisasi: Option<f64>,
    pub kendala: Option<String>,
    pub tanggal_realisasi: NaiveDate,
}

pub async fn update_hasil(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateHasilRequest>,
) -> AppResult<Json<HasilDetail>> {
    let existing = hasil::find(&state.pool, id)
        .await?
        .ok_or_else(|| AppError::NotFound("Hasil kerja tidak ditemukan".into()))?;

    if existing.personel_id != user.id() {
        return Err(AppError::Forbidden(
            "Hanya pemilik hasil kerja yang dapat mengubah".into(),
        ));
    }
    if !matches!(
        existing.status,
        StatusHasil::Draft | StatusHasil::PerluRevisi
    ) {
        return Err(AppError::Unprocessable(
            "Hasil kerja yang sudah diajukan tidak dapat diubah".into(),
        ));
    }

    let rencana = rencana::find(&state.pool, existing.rencana_id)
        .await?
        .ok_or_else(|| AppError::NotFound("Rencana kerja tidak ditemukan".into()))?;
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

    hasil::update(
        &state.pool,
        id,
        &UpdateHasil {
            uraian_pelaksanaan: req.uraian_pelaksanaan,
            progres_persen: req.progres_persen,
            jumlah_realisasi: req.jumlah_realisasi,
            kendala: req.kendala,
            tanggal_realisasi: req.tanggal_realisasi,
        },
    )
    .await?;

    Ok(Json(detail_internal(&state, id).await?))
}

pub async fn ajukan_hasil(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<Uuid>,
) -> AppResult<Json<HasilDetail>> {
    let existing = hasil::find(&state.pool, id)
        .await?
        .ok_or_else(|| AppError::NotFound("Hasil kerja tidak ditemukan".into()))?;

    if existing.personel_id != user.id() {
        return Err(AppError::Forbidden(
            "Hanya pemilik hasil kerja yang dapat mengajukan".into(),
        ));
    }
    if !matches!(
        existing.status,
        StatusHasil::Draft | StatusHasil::PerluRevisi
    ) {
        return Err(AppError::Unprocessable(
            "Hasil kerja ini sudah diajukan atau sudah diverifikasi".into(),
        ));
    }

    let rencana = rencana::find(&state.pool, existing.rencana_id)
        .await?
        .ok_or_else(|| AppError::NotFound("Rencana kerja tidak ditemukan".into()))?;
    let atasan_id = rencana.atasan_id.ok_or_else(|| {
        AppError::Unprocessable("Atasan langsung belum ditetapkan untuk akun Anda".into())
    })?;
    periode::pastikan_terbuka(&state.pool, rencana.periode_id).await?;

    hasil::ajukan(&state.pool, id).await?;

    let personel = personel::active_personel(&state.pool, user.id()).await?;
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

    Ok(Json(detail_internal(&state, id).await?))
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
    if !(user.is_admin_any() || user.is_atasan() || user.is_pimpinan_any()) {
        return Err(AppError::Forbidden(
            "Hanya atasan, pimpinan, atau admin yang dapat memverifikasi hasil kerja".into(),
        ));
    }

    let hasil_row = hasil::find(&state.pool, id)
        .await?
        .ok_or_else(|| AppError::NotFound("Hasil kerja tidak ditemukan".into()))?;

    if hasil_row.status != StatusHasil::Diajukan {
        return Err(AppError::Unprocessable(
            "Hasil kerja ini tidak sedang menunggu verifikasi".into(),
        ));
    }

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
