use std::time::Duration;

use axum::Router;
use axum::http::{HeaderName, HeaderValue, Method, StatusCode, header};
use axum::middleware::from_fn_with_state;
use axum::routing::get;
use tower_http::catch_panic::CatchPanicLayer;
use tower_http::compression::CompressionLayer;
use tower_http::cors::CorsLayer;
use tower_http::limit::RequestBodyLimitLayer;
use tower_http::request_id::{MakeRequestUuid, PropagateRequestIdLayer, SetRequestIdLayer};
use tower_http::timeout::TimeoutLayer;
use tower_http::trace::TraceLayer;

use crate::middleware::{audit, rate_limit};
use crate::routes;
use crate::state::AppState;

const MAX_BODY_BYTES: usize = 64 * 1024 * 1024;

pub fn build(state: AppState) -> Router {
    let api = Router::new()
        .nest("/auth", routes::auth::routes())
        .nest("/satker", routes::satker::routes())
        .nest("/admin", routes::admin::routes())
        .nest("/personel", routes::personel::routes())
        .nest("/titik-dinas", routes::titik_dinas::routes())
        .nest("/ph", routes::ph::routes())
        .nest("/absensi", routes::absensi::routes())
        .nest("/rencana-kerja", routes::rencana::routes())
        .nest("/hasil-kerja", routes::hasil::routes())
        .nest("/output", routes::output::routes())
        .nest("/lampiran", routes::lampiran::routes())
        .nest("/periode", routes::periode::routes())
        .nest("/libur", routes::libur::routes())
        .nest("/penilaian", routes::penilaian::routes())
        .nest("/dashboard", routes::dashboard::routes())
        .nest("/laporan", routes::laporan::routes())
        .nest("/notifikasi", routes::notifikasi::routes())
        .nest("/audit", routes::audit::routes())
        .with_state(state.clone())
        // Audit di dalam, pembatas laju di luar: permintaan yang ditolak
        // karena limit tidak perlu dicatat sebagai mutasi.
        .layer(from_fn_with_state(state.clone(), audit::catat_mutasi))
        .layer(from_fn_with_state(state.clone(), rate_limit::batasi_laju));

    let cors = build_cors(&state);

    Router::new()
        .route("/health", get(routes::health::health))
        .merge(routes::openapi::routes())
        .nest("/api/v1", api)
        .layer(PropagateRequestIdLayer::new(HeaderName::from_static(
            "x-request-id",
        )))
        .layer(SetRequestIdLayer::new(
            HeaderName::from_static("x-request-id"),
            MakeRequestUuid,
        ))
        .layer(TimeoutLayer::with_status_code(
            StatusCode::REQUEST_TIMEOUT,
            Duration::from_secs(120),
        ))
        .layer(RequestBodyLimitLayer::new(MAX_BODY_BYTES))
        .layer(CompressionLayer::new())
        .layer(CatchPanicLayer::new())
        .layer(TraceLayer::new_for_http())
        .layer(cors)
        .with_state(state)
}

fn build_cors(state: &AppState) -> CorsLayer {
    let origins: Vec<HeaderValue> = state
        .settings
        .cors_origins
        .iter()
        .filter_map(|origin| HeaderValue::from_str(origin).ok())
        .collect();

    CorsLayer::new()
        .allow_origin(origins)
        .allow_credentials(true)
        .allow_methods([
            Method::GET,
            Method::POST,
            Method::PATCH,
            Method::PUT,
            Method::DELETE,
            Method::OPTIONS,
        ])
        .allow_headers([
            header::AUTHORIZATION,
            header::CONTENT_TYPE,
            header::ACCEPT,
            HeaderName::from_static("x-request-id"),
        ])
        .expose_headers([HeaderName::from_static("x-request-id")])
}
