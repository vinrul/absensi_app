use axum::Json;
use axum::Router;
use axum::extract::{Path, Query, State};
use axum::routing::{get, post};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::auth::extractor::AuthUser;
use crate::domain::enums::{KategoriKinerja, KeputusanPenilaian};
use crate::error::{AppError, AppResult};
use crate::repo::penilaian::{self, ListFilter, PenilaianRow};
use crate::repo::periode;
use crate::repo::{ListResponse, pagination};
use crate::service::akses;
use crate::service::penilaian as service_penilaian;
use crate::state::AppState;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/", get(list_penilaian))
        .route("/{periode_id}/hitung", post(hitung))
        .route("/personel/{personel_id}", get(detail_penilaian))
        .route("/{id}/keputusan", post(keputusan))
}

#[derive(Debug, Deserialize)]
pub struct ListQuery {
    pub page: Option<i64>,
    pub per_page: Option<i64>,
    pub periode_id: Option<Uuid>,
    pub satker_id: Option<Uuid>,
    pub unit: Option<String>,
    pub kategori: Option<KategoriKinerja>,
}

pub async fn list_penilaian(
    State(state): State<AppState>,
    user: AuthUser,
    Query(query): Query<ListQuery>,
) -> AppResult<Json<ListResponse<PenilaianRow>>> {
    let periode_row = periode::find_or_current(&state.pool, query.periode_id).await?;

    let mut scope = crate::routes::personel::scope_for(&state, &user).await?;
    if let Some(satker_id) = query.satker_id {
        if !scope.contains(&satker_id) {
            return Err(AppError::Forbidden(
                "Anda tidak berhak melihat penilaian satker tersebut".into(),
            ));
        }
        scope = vec![satker_id];
    }

    let (page, per_page, offset) = pagination(query.page, query.per_page);
    let claims = user.claims();
    let personel_id = if claims.is_admin() || claims.is_atasan() || claims.is_pimpinan() {
        None
    } else {
        Some(claims.sub)
    };

    let (rows, total) = penilaian::list(
        &state.pool,
        &ListFilter {
            periode_id: periode_row.id,
            satker_ids: scope,
            unit: query.unit.filter(|v| !v.is_empty()),
            kategori: query.kategori,
            personel_id,
            limit: per_page,
            offset,
        },
    )
    .await?;

    Ok(Json(ListResponse::new(rows, page, per_page, total)))
}

#[derive(Debug, Serialize)]
pub struct HitungResponse {
    pub periode_id: Uuid,
    pub jumlah_personel: usize,
    pub rata_rata_nilai: f64,
    pub pesan: String,
}

pub async fn hitung(
    State(state): State<AppState>,
    user: AuthUser,
    Path(periode_id): Path<Uuid>,
) -> AppResult<Json<HitungResponse>> {
    if !(user.is_admin_any() || user.is_atasan() || user.is_pimpinan_any()) {
        return Err(AppError::Forbidden(
            "Hitung penilaian hanya untuk atasan/pimpinan/admin".into(),
        ));
    }

    let scope = crate::routes::personel::scope_for(&state, &user).await?;

    let hasil =
        service_penilaian::hitung_periode(&state, periode_id, &scope, None, user.id()).await?;

    Ok(Json(HitungResponse {
        periode_id: hasil.periode_id,
        jumlah_personel: hasil.jumlah_personel,
        rata_rata_nilai: hasil.rata_rata_nilai,
        pesan: format!(
            "Penilaian dihitung untuk {} personel",
            hasil.jumlah_personel
        ),
    }))
}

pub async fn detail_penilaian(
    State(state): State<AppState>,
    user: AuthUser,
    Path(personel_id): Path<Uuid>,
    Query(query): Query<ListQuery>,
) -> AppResult<Json<serde_json::Value>> {
    let periode_row = periode::find_or_current(&state.pool, query.periode_id).await?;

    let personel_row = crate::repo::personel::find_by_id(&state.pool, personel_id)
        .await?
        .ok_or_else(|| AppError::NotFound("Personel tidak ditemukan".into()))?;

    let boleh = akses::boleh_melihat_personel(
        user.role_scopes(),
        user.satker_akses(),
        user.id(),
        personel_row.id,
        personel_row.satker_id,
    );
    if !boleh {
        return Err(AppError::Forbidden(
            "Anda tidak berhak melihat penilaian personel ini".into(),
        ));
    }

    let row = penilaian::find(&state.pool, personel_id, periode_row.id)
        .await?
        .ok_or_else(|| AppError::NotFound("Penilaian belum dihitung untuk periode ini".into()))?;

    let detail = penilaian::detail(&state.pool, row.id).await?;
    let komponen: Vec<serde_json::Value> = detail
        .into_iter()
        .map(|(indikator, skor, komponen)| {
            serde_json::json!({
                "indikator": indikator,
                "skor": skor,
                "komponen": komponen,
            })
        })
        .collect();

    Ok(Json(serde_json::json!({
        "data": row,
        "komponen": komponen,
        "periode": {
            "id": periode_row.id,
            "nama": periode_row.nama,
            "tanggal_mulai": periode_row.tanggal_mulai,
            "tanggal_selesai": periode_row.tanggal_selesai,
        },
    })))
}

#[derive(Debug, Deserialize)]
pub struct KeputusanRequest {
    pub catatan: Option<String>,
    pub keputusan: KeputusanPenilaian,
}

pub async fn keputusan(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<Uuid>,
    Json(req): Json<KeputusanRequest>,
) -> AppResult<Json<serde_json::Value>> {
    if !(user.is_admin_any() || user.is_pimpinan_any()) {
        return Err(AppError::Forbidden(
            "Keputusan penilaian hanya untuk pimpinan/admin".into(),
        ));
    }

    penilaian::keputusan(
        &state.pool,
        id,
        req.catatan.as_deref(),
        req.keputusan,
        user.id(),
    )
    .await?;

    Ok(Json(
        serde_json::json!({ "message": "Keputusan penilaian disimpan" }),
    ))
}
