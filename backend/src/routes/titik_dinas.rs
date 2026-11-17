use axum::Json;
use axum::Router;
use axum::extract::{Path, Query, State};
use axum::routing::{get, patch};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::auth::extractor::AuthUser;
use crate::domain::enums::TitikDinasJenis;
use crate::error::{AppError, AppResult};
use crate::repo::titik_dinas::{self, TitikDinasRow};
use crate::routes::personel::scope_for;
use crate::state::AppState;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/", get(list_titik).post(create_titik))
        .route("/{id}", patch(update_titik).delete(delete_titik))
}

#[derive(Debug, Serialize)]
pub struct TitikDinasDto {
    pub id: Uuid,
    pub satker_id: Uuid,
    pub nama: String,
    pub jenis: TitikDinasJenis,
    pub latitude: f64,
    pub longitude: f64,
    pub radius_m: i32,
    pub is_active: bool,
}

impl From<TitikDinasRow> for TitikDinasDto {
    fn from(row: TitikDinasRow) -> Self {
        Self {
            id: row.id,
            satker_id: row.satker_id,
            nama: row.nama,
            jenis: row.jenis,
            latitude: row.latitude,
            longitude: row.longitude,
            radius_m: row.radius_m,
            is_active: row.is_active,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct ListQuery {
    pub satker_id: Option<Uuid>,
}

pub async fn list_titik(
    State(state): State<AppState>,
    user: AuthUser,
    Query(query): Query<ListQuery>,
) -> AppResult<Json<Vec<TitikDinasDto>>> {
    let mut scope = scope_for(&state, &user).await?;

    if let Some(satker_id) = query.satker_id {
        if !scope.contains(&satker_id) {
            return Err(AppError::Forbidden(
                "Anda tidak berhak melihat titik dinas satker tersebut".into(),
            ));
        }
        scope = vec![satker_id];
    }

    let rows = titik_dinas::list(&state.pool, &scope).await?;
    Ok(Json(rows.into_iter().map(TitikDinasDto::from).collect()))
}

#[derive(Debug, Deserialize)]
pub struct CreateTitikRequest {
    pub satker_id: Uuid,
    pub nama: String,
    pub jenis: Option<TitikDinasJenis>,
    pub latitude: f64,
    pub longitude: f64,
    pub radius_m: Option<i32>,
}

pub async fn create_titik(
    State(state): State<AppState>,
    user: AuthUser,
    Json(req): Json<CreateTitikRequest>,
) -> AppResult<Json<TitikDinasDto>> {
    if !(user.is_admin_any() || user.is_atasan()) {
        return Err(AppError::Forbidden(
            "Hanya atasan atau admin yang dapat menambah titik dinas".into(),
        ));
    }
    user.require_satker(req.satker_id)?;

    let nama = req.nama.trim();
    if nama.is_empty() {
        return Err(AppError::BadRequest("Nama titik dinas wajib diisi".into()));
    }
    if !(-90.0..=90.0).contains(&req.latitude) || !(-180.0..=180.0).contains(&req.longitude) {
        return Err(AppError::Unprocessable("Koordinat tidak valid".into()));
    }

    let radius = req.radius_m.unwrap_or(100);
    if !(10..=5000).contains(&radius) {
        return Err(AppError::Unprocessable(
            "Radius harus antara 10 dan 5000 meter".into(),
        ));
    }

    let id = Uuid::now_v7();
    titik_dinas::create(
        &state.pool,
        id,
        req.satker_id,
        nama,
        req.jenis.unwrap_or(TitikDinasJenis::Kantor),
        req.latitude,
        req.longitude,
        radius,
    )
    .await?;

    let row = titik_dinas::find(&state.pool, id)
        .await?
        .ok_or_else(|| AppError::Internal("titik dinas gagal dibuat".into()))?;
    Ok(Json(row.into()))
}

#[derive(Debug, Deserialize)]
pub struct UpdateTitikRequest {
    pub nama: Option<String>,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    pub radius_m: Option<i32>,
    pub is_active: Option<bool>,
}

pub async fn update_titik(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateTitikRequest>,
) -> AppResult<Json<TitikDinasDto>> {
    if !(user.is_admin_any() || user.is_atasan()) {
        return Err(AppError::Forbidden(
            "Hanya atasan atau admin yang dapat mengubah titik dinas".into(),
        ));
    }

    let existing = titik_dinas::find(&state.pool, id)
        .await?
        .ok_or_else(|| AppError::NotFound("Titik dinas tidak ditemukan".into()))?;
    user.require_satker(existing.satker_id)?;

    titik_dinas::update(
        &state.pool,
        id,
        req.nama.as_deref(),
        req.latitude,
        req.longitude,
        req.radius_m,
        req.is_active,
    )
    .await?;

    let row = titik_dinas::find(&state.pool, id)
        .await?
        .ok_or_else(|| AppError::NotFound("Titik dinas tidak ditemukan".into()))?;
    Ok(Json(row.into()))
}

pub async fn delete_titik(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<Uuid>,
) -> AppResult<Json<serde_json::Value>> {
    if !(user.is_admin_any() || user.is_atasan()) {
        return Err(AppError::Forbidden(
            "Hanya atasan atau admin yang dapat menghapus titik dinas".into(),
        ));
    }

    let existing = titik_dinas::find(&state.pool, id)
        .await?
        .ok_or_else(|| AppError::NotFound("Titik dinas tidak ditemukan".into()))?;
    user.require_satker(existing.satker_id)?;

    titik_dinas::soft_delete(&state.pool, id).await?;
    Ok(Json(
        serde_json::json!({ "message": "Titik dinas dihapus" }),
    ))
}
