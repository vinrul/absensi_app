use axum::Json;
use axum::Router;
use axum::extract::{Path, Query, State};
use axum::routing::{get, post};
use chrono::NaiveDate;
use serde::Deserialize;
use uuid::Uuid;

use crate::auth::extractor::AuthUser;
use crate::domain::enums::{BerkasOwner, JenisOutput, StatusHasil, StatusOutput};
use crate::error::{AppError, AppResult};
use crate::repo::hasil;
use crate::repo::lampiran;
use crate::repo::notifikasi;
use crate::repo::output::{self, ListFilter, NewOutput, OutputRow, SkorOutput};
use crate::repo::personel;
use crate::repo::{ListResponse, pagination};
use crate::service::akses;
use crate::service::scoring;
use crate::state::AppState;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/", get(list_output).post(create_output))
        .route("/{id}", get(detail_output))
        .route("/{id}/ajukan", post(ajukan_output))
        .route("/{id}/verifikasi", post(verifikasi_output))
}

#[derive(Debug, Deserialize)]
pub struct ListQuery {
    pub page: Option<i64>,
    pub per_page: Option<i64>,
    pub satker_id: Option<Uuid>,
    pub personel_id: Option<Uuid>,
    pub status: Option<StatusOutput>,
}

pub async fn list_output(
    State(state): State<AppState>,
    user: AuthUser,
    Query(query): Query<ListQuery>,
) -> AppResult<Json<ListResponse<OutputRow>>> {
    let mut scope = crate::routes::personel::scope_for(&state, &user).await?;

    if let Some(satker_id) = query.satker_id {
        if !scope.contains(&satker_id) {
            return Err(AppError::Forbidden(
                "Anda tidak berhak melihat output satker tersebut".into(),
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
    let (rows, total) = output::list(
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

pub async fn detail_output(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<Uuid>,
) -> AppResult<Json<serde_json::Value>> {
    let row = output::find(&state.pool, id)
        .await?
        .ok_or_else(|| AppError::NotFound("Output tidak ditemukan".into()))?;

    let boleh = akses::boleh_melihat_personel(
        user.role_scopes(),
        user.satker_akses(),
        user.id(),
        row.personel_id,
        row.satker_id,
    );
    if !boleh {
        return Err(AppError::Forbidden(
            "Anda tidak berhak melihat output ini".into(),
        ));
    }

    let lampiran_rows = lampiran::list(&state.pool, BerkasOwner::Output, id).await?;

    Ok(Json(serde_json::json!({
        "data": row,
        "lampiran": lampiran_rows,
    })))
}

#[derive(Debug, Deserialize)]
pub struct CreateOutputRequest {
    pub hasil_kerja_id: Uuid,
    pub jenis_output: JenisOutput,
    pub judul: String,
    pub uraian: Option<String>,
    pub tanggal_hasil: NaiveDate,
    pub url_eksternal: Option<String>,
}

pub async fn create_output(
    State(state): State<AppState>,
    user: AuthUser,
    Json(req): Json<CreateOutputRequest>,
) -> AppResult<Json<OutputRow>> {
    let personel = personel::active_personel(&state.pool, user.id()).await?;

    let hasil_row = hasil::find(&state.pool, req.hasil_kerja_id)
        .await?
        .ok_or_else(|| AppError::NotFound("Hasil kerja tidak ditemukan".into()))?;

    if hasil_row.personel_id != personel.id {
        return Err(AppError::Forbidden(
            "Output hanya untuk hasil kerja milik Anda".into(),
        ));
    }
    if hasil_row.status != StatusHasil::Terverifikasi {
        return Err(AppError::Unprocessable(
            "Hasil kerja harus terverifikasi atasan sebelum output dibuat".into(),
        ));
    }

    if req.judul.trim().is_empty() {
        return Err(AppError::BadRequest("Judul output wajib diisi".into()));
    }
    if req.jenis_output == JenisOutput::LinkSistem {
        let url = req.url_eksternal.as_deref().unwrap_or("").trim();
        if url.is_empty() {
            return Err(AppError::Unprocessable(
                "Jenis Link/Sistem wajib menyertakan URL".into(),
            ));
        }
        if !url.starts_with("http://") && !url.starts_with("https://") {
            return Err(AppError::Unprocessable(
                "URL harus diawali http(s)://".into(),
            ));
        }
    }

    let id = Uuid::now_v7();
    output::create(
        &state.pool,
        &NewOutput {
            id,
            hasil_kerja_id: hasil_row.id,
            rencana_id: hasil_row.rencana_id,
            personel_id: personel.id,
            satker_id: personel.satker_id,
            jenis_output: req.jenis_output,
            judul: req.judul.trim().to_string(),
            uraian: req.uraian,
            tanggal_hasil: req.tanggal_hasil,
            url_eksternal: req.url_eksternal,
            status: StatusOutput::MenungguVerifikasi,
        },
    )
    .await?;

    let row = output::find(&state.pool, id)
        .await?
        .ok_or_else(|| AppError::Internal("output gagal dibaca".into()))?;
    Ok(Json(row))
}

pub async fn ajukan_output(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<Uuid>,
) -> AppResult<Json<OutputRow>> {
    let row = output::find(&state.pool, id)
        .await?
        .ok_or_else(|| AppError::NotFound("Output tidak ditemukan".into()))?;

    if row.personel_id != user.id() {
        return Err(AppError::Forbidden(
            "Hanya pemilik output yang dapat mengajukan".into(),
        ));
    }

    let jumlah_lampiran = lampiran::count(&state.pool, BerkasOwner::Output, id).await?;
    if row.jenis_output.wajib_berkas() && jumlah_lampiran == 0 {
        return Err(AppError::Unprocessable(
            "Unggah berkas bukti sebelum mengajukan output".into(),
        ));
    }

    output::set_status(&state.pool, id, StatusOutput::DalamVerifikasi).await?;

    let row = output::find(&state.pool, id)
        .await?
        .ok_or_else(|| AppError::NotFound("Output tidak ditemukan".into()))?;
    Ok(Json(row))
}

#[derive(Debug, Deserialize)]
pub struct VerifikasiOutputRequest {
    pub skor_kesesuaian: Option<f64>,
    pub skor_keabsahan: Option<f64>,
    pub catatan: Option<String>,
    pub setuju: Option<bool>,
}

pub async fn verifikasi_output(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<Uuid>,
    Json(req): Json<VerifikasiOutputRequest>,
) -> AppResult<Json<OutputRow>> {
    if !(user.is_admin_any() || user.is_atasan()) {
        return Err(AppError::Forbidden(
            "Hanya atasan atau admin yang dapat memverifikasi output".into(),
        ));
    }

    let row = output::find(&state.pool, id)
        .await?
        .ok_or_else(|| AppError::NotFound("Output tidak ditemukan".into()))?;

    let rencana_row = crate::repo::rencana::find(&state.pool, row.rencana_id)
        .await?
        .ok_or_else(|| AppError::NotFound("Rencana kerja tidak ditemukan".into()))?;
    user.require_satker(rencana_row.satker_id)?;

    let claims = user.claims();
    if rencana_row.atasan_id != Some(claims.sub) && !user.is_admin_any() {
        return Err(AppError::Forbidden(
            "Hanya atasan langsung yang dapat memverifikasi".into(),
        ));
    }

    let setuju = req.setuju.unwrap_or(true);

    let jumlah_lampiran = lampiran::count(&state.pool, BerkasOwner::Output, id).await?;
    let kelengkapan = match row.jenis_output {
        JenisOutput::LinkSistem => {
            if row.url_eksternal.as_deref().unwrap_or("").trim().is_empty() {
                0.0
            } else {
                100.0
            }
        }
        _ => {
            if jumlah_lampiran > 0 {
                100.0
            } else {
                0.0
            }
        }
    };

    let (komponen, total) = if setuju {
        let kesesuaian = req.skor_kesesuaian.unwrap_or(80.0);
        let keabsahan = req.skor_keabsahan.unwrap_or(85.0);
        let (kelengkapan, kesesuaian, keabsahan, total) =
            scoring::skor_output(kelengkapan, kesesuaian, keabsahan);
        (
            SkorOutput {
                kelengkapan,
                kesesuaian,
                keabsahan,
                total,
            },
            total,
        )
    } else {
        let catatan = req.catatan.as_deref().unwrap_or("").trim();
        if catatan.is_empty() {
            return Err(AppError::Unprocessable(
                "Catatan wajib diisi saat meminta revisi".into(),
            ));
        }
        (
            SkorOutput {
                kelengkapan,
                kesesuaian: 0.0,
                keabsahan: 0.0,
                total: 0.0,
            },
            0.0,
        )
    };

    let status = if setuju {
        StatusOutput::Disetujui
    } else {
        StatusOutput::PerluRevisi
    };

    output::verifikasi(
        &state.pool,
        id,
        claims.sub,
        &komponen,
        status,
        req.catatan.as_deref(),
    )
    .await?;

    if setuju {
        notifikasi::kirim(
            &state.pool,
            row.personel_id,
            "output_disetujui",
            "Output disetujui",
            &format!("Nilai output: {total:.2}"),
            Some("output"),
            Some(id),
        )
        .await?;
    }

    let row = output::find(&state.pool, id)
        .await?
        .ok_or_else(|| AppError::NotFound("Output tidak ditemukan".into()))?;
    Ok(Json(row))
}
