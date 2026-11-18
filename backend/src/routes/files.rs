use axum::Router;
use axum::body::Body;
use axum::extract::{Path, State};
use axum::http::{HeaderValue, StatusCode, header};
use axum::response::Response;
use axum::routing::get;

use crate::auth::extractor::AuthUser;
use crate::error::{AppError, AppResult};
use crate::repo::{absensi, ph};
use crate::state::AppState;

pub fn routes() -> Router<AppState> {
    Router::new().route("/{*key}", get(baca))
}

/// Menyajikan foto absensi/strong point dengan cek kepemilikan atau cakupan satker.
async fn baca(
    State(state): State<AppState>,
    user: AuthUser,
    Path(key): Path<String>,
) -> AppResult<Response> {
    let pemilik = match absensi::pemilik_foto(&state.pool, &key).await? {
        Some(pemilik) => pemilik,
        None => ph::pemilik_foto_ph(&state.pool, &key)
            .await?
            .ok_or_else(|| AppError::NotFound("Foto tidak ditemukan".into()))?,
    };
    let (personel_id, satker_id) = pemilik;
    let pengelola = user.is_admin_any() || user.is_pimpinan_any() || user.is_atasan();
    if user.id() != personel_id && !(pengelola && user.satker_akses().contains(&satker_id)) {
        return Err(AppError::Forbidden(
            "Anda tidak berhak melihat foto ini".into(),
        ));
    }

    let bytes = state.storage.read(&key).await?;
    let mime = mime_guess::from_path(&key).first_or_octet_stream();

    let mut response = Response::new(Body::from(bytes));
    *response.status_mut() = StatusCode::OK;
    response.headers_mut().insert(
        header::CONTENT_TYPE,
        HeaderValue::from_str(mime.as_ref())
            .unwrap_or_else(|_| HeaderValue::from_static("application/octet-stream")),
    );
    response.headers_mut().insert(
        header::CACHE_CONTROL,
        HeaderValue::from_static("private, max-age=600"),
    );
    Ok(response)
}
