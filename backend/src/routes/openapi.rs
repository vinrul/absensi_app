use axum::Router;
use axum::response::{Html, IntoResponse, Response};
use axum::routing::get;

use crate::state::AppState;

const SPESIFIKASI: &str = include_str!("../../openapi.json");
const DOKUMEN: &str = include_str!("../../assets/api-docs.html");

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/docs", get(dokumen))
        .route("/api-docs/openapi.json", get(spesifikasi))
}

async fn spesifikasi() -> Response {
    (
        [("content-type", "application/json; charset=utf-8")],
        SPESIFIKASI,
    )
        .into_response()
}

async fn dokumen() -> Html<&'static str> {
    Html(DOKUMEN)
}
