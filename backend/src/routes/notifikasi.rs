use axum::Json;
use axum::Router;
use axum::extract::{Path, Query, State};
use axum::routing::{get, post};
use serde::Deserialize;
use uuid::Uuid;

use crate::auth::extractor::AuthUser;
use crate::error::AppResult;
use crate::repo::notifikasi::{self, NotifikasiRow};
use crate::state::AppState;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/", get(list_notifikasi))
        .route("/{id}/baca", post(baca))
}

#[derive(Debug, Deserialize)]
pub struct ListQuery {
    pub belum_dibaca: Option<bool>,
}

pub async fn list_notifikasi(
    State(state): State<AppState>,
    user: AuthUser,
    Query(query): Query<ListQuery>,
) -> AppResult<Json<Vec<NotifikasiRow>>> {
    let rows =
        notifikasi::list(&state.pool, user.id(), query.belum_dibaca.unwrap_or(false)).await?;
    Ok(Json(rows))
}

pub async fn baca(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<Uuid>,
) -> AppResult<Json<serde_json::Value>> {
    notifikasi::tandai_dibaca(&state.pool, id, user.id()).await?;
    Ok(Json(
        serde_json::json!({ "message": "Notifikasi ditandai dibaca" }),
    ))
}
