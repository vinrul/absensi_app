use axum::Json;
use axum::Router;
use axum::extract::{Query, State};
use axum::routing::get;
use chrono::NaiveDate;
use serde::Deserialize;
use uuid::Uuid;

use crate::auth::extractor::AuthUser;
use crate::error::AppResult;
use crate::repo::audit::{self, AuditRow};
use crate::repo::{ListResponse, pagination};
use crate::state::AppState;

pub fn routes() -> Router<AppState> {
    Router::new().route("/", get(list_audit))
}

#[derive(Debug, Deserialize)]
pub struct ListQuery {
    pub page: Option<i64>,
    pub per_page: Option<i64>,
    pub entitas: Option<String>,
    pub actor_id: Option<Uuid>,
    pub dari: Option<NaiveDate>,
    pub sampai: Option<NaiveDate>,
}

pub async fn list_audit(
    State(state): State<AppState>,
    user: AuthUser,
    Query(query): Query<ListQuery>,
) -> AppResult<Json<ListResponse<AuditRow>>> {
    user.require_audit()?;

    let (page, per_page, offset) = pagination(query.page, query.per_page);

    let (rows, total) = audit::list(
        &state.pool,
        &audit::ListFilter {
            entitas: query.entitas.filter(|v| !v.is_empty()),
            actor_id: query.actor_id,
            dari: query.dari,
            sampai: query.sampai,
            limit: per_page,
            offset,
        },
    )
    .await?;

    Ok(Json(ListResponse::new(rows, page, per_page, total)))
}
