use axum::Json;
use axum::Router;
use axum::extract::{Path, Query, State};
use axum::routing::{get, patch};
use chrono::{Datelike, NaiveDate};
use serde::Deserialize;
use uuid::Uuid;

use crate::auth::extractor::AuthUser;
use crate::domain::enums::{JenisLibur, SatkerLevel};
use crate::error::{AppError, AppResult};
use crate::repo::libur::{self, LiburBaru, LiburRow};
use crate::repo::satker;
use crate::service::{akses, waktu};
use crate::state::AppState;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/", get(list_libur).post(create_libur))
        .route("/{id}", patch(update_libur).delete(delete_libur))
}

#[derive(Debug, Deserialize)]
pub struct ListQuery {
    pub tahun: Option<i32>,
}

pub async fn list_libur(
    State(state): State<AppState>,
    user: AuthUser,
    Query(query): Query<ListQuery>,
) -> AppResult<Json<Vec<LiburRow>>> {
    let tahun = query.tahun.unwrap_or_else(|| waktu::hari_ini_wita().year());
    if !(2000..=2100).contains(&tahun) {
        return Err(AppError::Unprocessable("Tahun tidak valid".into()));
    }
    let dari = NaiveDate::from_ymd_opt(tahun, 1, 1)
        .ok_or_else(|| AppError::Unprocessable("Tahun tidak valid".into()))?;
    let sampai = NaiveDate::from_ymd_opt(tahun, 12, 31)
        .ok_or_else(|| AppError::Unprocessable("Tahun tidak valid".into()))?;
    Ok(Json(
        libur::list(&state.pool, dari, sampai, user.satker_akses()).await?,
    ))
}

#[derive(Debug, Deserialize)]
pub struct CreateLiburRequest {
    pub tanggal: NaiveDate,
    pub nama: String,
    pub jenis: JenisLibur,
    /// NULL = libur nasional; terisi = libur khusus satker.
    pub satker_id: Option<Uuid>,
}

pub async fn create_libur(
    State(state): State<AppState>,
    user: AuthUser,
    Json(req): Json<CreateLiburRequest>,
) -> AppResult<Json<LiburRow>> {
    if !user.is_admin_any() {
        return Err(AppError::Forbidden(
            "Hanya admin yang dapat menambah hari libur".into(),
        ));
    }
    let nama = req.nama.trim();
    if nama.is_empty() {
        return Err(AppError::BadRequest("Nama hari libur wajib diisi".into()));
    }
    if nama.chars().count() > 150 {
        return Err(AppError::Unprocessable(
            "Nama hari libur maksimal 150 karakter".into(),
        ));
    }
    match req.satker_id {
        None => {
            user.require_admin_global()?;
            if req.jenis == JenisLibur::Khusus {
                return Err(AppError::Unprocessable(
                    "Libur khusus harus memiliki satker".into(),
                ));
            }
        }
        Some(satker_id) => {
            let target = satker::find(&state.pool, satker_id)
                .await?
                .ok_or_else(|| AppError::NotFound("Satker tidak ditemukan".into()))?;
            if target.level != SatkerLevel::Satker {
                return Err(AppError::Unprocessable(
                    "Libur khusus hanya untuk level satker".into(),
                ));
            }
            if req.jenis == JenisLibur::Nasional {
                return Err(AppError::Unprocessable(
                    "Libur nasional harus berlaku untuk semua satker".into(),
                ));
            }
            user.require_satker(satker_id)?;
            if !akses::boleh_kelola_personel_id(&state.pool, user.role_scopes(), satker_id).await? {
                return Err(AppError::Forbidden(
                    "Satker berada di luar cakupan admin".into(),
                ));
            }
        }
    }
    if libur::exists(&state.pool, req.tanggal, req.satker_id, None).await? {
        return Err(AppError::Conflict(
            "Tanggal tersebut sudah memiliki hari libur pada cakupan yang sama".into(),
        ));
    }

    let id = Uuid::now_v7();
    libur::create(
        &state.pool,
        &LiburBaru {
            id,
            tanggal: req.tanggal,
            nama,
            jenis: req.jenis,
            satker_id: req.satker_id,
            created_by: user.id(),
        },
    )
    .await?;
    let row = libur::find(&state.pool, id)
        .await?
        .ok_or_else(|| AppError::Internal("hari libur gagal dibaca".into()))?;
    Ok(Json(row))
}

#[derive(Debug, Deserialize)]
pub struct UpdateLiburRequest {
    pub tanggal: Option<NaiveDate>,
    pub nama: Option<String>,
    pub jenis: Option<JenisLibur>,
    pub is_active: Option<bool>,
}

async fn pastikan_boleh_ubah(
    state: &AppState,
    user: &AuthUser,
    existing: &LiburRow,
) -> AppResult<()> {
    if !user.is_admin_any() {
        return Err(AppError::Forbidden(
            "Hanya admin yang dapat mengubah hari libur".into(),
        ));
    }
    match existing.satker_id {
        None => user.require_admin_global(),
        Some(satker_id) => {
            user.require_satker(satker_id)?;
            if akses::boleh_kelola_personel_id(&state.pool, user.role_scopes(), satker_id).await? {
                Ok(())
            } else {
                Err(AppError::Forbidden(
                    "Satker berada di luar cakupan admin".into(),
                ))
            }
        }
    }
}

pub async fn update_libur(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateLiburRequest>,
) -> AppResult<Json<LiburRow>> {
    let existing = libur::find(&state.pool, id)
        .await?
        .ok_or_else(|| AppError::NotFound("Hari libur tidak ditemukan".into()))?;
    pastikan_boleh_ubah(&state, &user, &existing).await?;

    if existing.satker_id.is_none() && req.jenis == Some(JenisLibur::Khusus) {
        return Err(AppError::Unprocessable(
            "Libur khusus harus memiliki satker".into(),
        ));
    }
    if existing.satker_id.is_some() && req.jenis == Some(JenisLibur::Nasional) {
        return Err(AppError::Unprocessable(
            "Libur nasional harus berlaku untuk semua satker".into(),
        ));
    }
    let nama = req.nama.as_deref().map(str::trim).filter(|v| !v.is_empty());
    if let Some(nama) = nama
        && nama.chars().count() > 150
    {
        return Err(AppError::Unprocessable(
            "Nama hari libur maksimal 150 karakter".into(),
        ));
    }
    let tanggal = req.tanggal.unwrap_or(existing.tanggal);
    if req.tanggal.is_some()
        && libur::exists(&state.pool, tanggal, existing.satker_id, Some(existing.id)).await?
    {
        return Err(AppError::Conflict(
            "Tanggal tersebut sudah memiliki hari libur pada cakupan yang sama".into(),
        ));
    }

    libur::update(&state.pool, id, req.tanggal, nama, req.jenis, req.is_active).await?;
    let row = libur::find(&state.pool, id)
        .await?
        .ok_or_else(|| AppError::Internal("hari libur gagal dibaca".into()))?;
    Ok(Json(row))
}

pub async fn delete_libur(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<Uuid>,
) -> AppResult<Json<serde_json::Value>> {
    let existing = libur::find(&state.pool, id)
        .await?
        .ok_or_else(|| AppError::NotFound("Hari libur tidak ditemukan".into()))?;
    pastikan_boleh_ubah(&state, &user, &existing).await?;
    libur::delete(&state.pool, id).await?;
    Ok(Json(serde_json::json!({ "message": "Hari libur dihapus" })))
}
