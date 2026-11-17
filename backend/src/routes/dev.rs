use axum::Json;
use axum::Router;
use axum::extract::State;
use axum::routing::{get, post};
use chrono::{FixedOffset, NaiveDate, NaiveTime, TimeZone, Utc};
use serde::Deserialize;
use serde_json::{Value, json};

use crate::error::{AppError, AppResult};
use crate::service::waktu;
use crate::state::AppState;

/// Endpoint bantu pengembangan. Router ini hanya dipasang bila
/// `APP__ALLOW_FAKE_TIME=true`; produksi tidak boleh mengaktifkannya.
pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/waktu", get(status).post(set_waktu))
        .route("/waktu/reset", post(reset_waktu))
}

#[derive(Debug, Deserialize)]
pub struct WaktuPalsuRequest {
    /// Format `YYYY-MM-DD`.
    pub tanggal: String,
    /// Format `HH:MM` atau `HH:MM:SS`.
    pub jam: String,
    /// `Asia/Jakarta`, `Asia/Makassar` (default), atau `Asia/Jayapura`.
    pub zona: Option<String>,
}

fn pastikan_dev(state: &AppState) -> AppResult<()> {
    if state.settings.allow_fake_time {
        Ok(())
    } else {
        Err(AppError::Forbidden(
            "Waktu palsu hanya tersedia di development (APP__ALLOW_FAKE_TIME=true)".into(),
        ))
    }
}

fn offset_zona(zona: &str) -> AppResult<i32> {
    match zona {
        "Asia/Jakarta" => Ok(7 * 3600),
        "Asia/Makassar" => Ok(8 * 3600),
        "Asia/Jayapura" => Ok(9 * 3600),
        _ => Err(AppError::Unprocessable(
            "Zona harus Asia/Jakarta, Asia/Makassar, atau Asia/Jayapura".into(),
        )),
    }
}

pub async fn status(State(state): State<AppState>) -> AppResult<Json<Value>> {
    pastikan_dev(&state)?;

    let sekarang = waktu::sekarang_utc();
    let lokal = waktu::ke_wita(sekarang);
    Ok(Json(json!({
        "aktif": waktu::override_aktif().is_some(),
        "tanggal": lokal.format("%Y-%m-%d").to_string(),
        "jam": lokal.format("%H:%M:%S").to_string(),
        "zona": "Asia/Makassar",
        "utc": sekarang.to_rfc3339(),
    })))
}

pub async fn set_waktu(
    State(state): State<AppState>,
    Json(req): Json<WaktuPalsuRequest>,
) -> AppResult<Json<Value>> {
    pastikan_dev(&state)?;

    let tanggal = NaiveDate::parse_from_str(req.tanggal.trim(), "%Y-%m-%d")
        .map_err(|_| AppError::BadRequest("Tanggal harus berformat YYYY-MM-DD".into()))?;
    let jam = NaiveTime::parse_from_str(req.jam.trim(), "%H:%M")
        .or_else(|_| NaiveTime::parse_from_str(req.jam.trim(), "%H:%M:%S"))
        .map_err(|_| AppError::BadRequest("Jam harus berformat HH:MM atau HH:MM:SS".into()))?;

    let zona = req.zona.as_deref().unwrap_or("Asia/Makassar");
    let offset = FixedOffset::east_opt(offset_zona(zona)?)
        .ok_or_else(|| AppError::Unprocessable("Offset zona tidak valid".into()))?;

    let lokal = offset
        .from_local_datetime(&tanggal.and_time(jam))
        .single()
        .ok_or_else(|| AppError::Unprocessable("Waktu lokal tidak valid".into()))?;
    let utc = lokal.with_timezone(&Utc);

    waktu::set_override(Some(utc));
    tracing::warn!(
        tanggal = %tanggal,
        jam = %jam.format("%H:%M:%S"),
        zona,
        "waktu server palsu diaktifkan (mode development)"
    );

    Ok(Json(json!({
        "aktif": true,
        "tanggal": tanggal.to_string(),
        "jam": jam.format("%H:%M:%S").to_string(),
        "zona": zona,
        "utc": utc.to_rfc3339(),
        "pesan": "Waktu server palsu aktif. Gunakan endpoint reset untuk kembali ke jam sistem.",
    })))
}

pub async fn reset_waktu(State(state): State<AppState>) -> AppResult<Json<Value>> {
    pastikan_dev(&state)?;

    waktu::set_override(None);
    let sekarang = waktu::sekarang_utc();
    tracing::info!("waktu server palsu dimatikan, kembali ke jam sistem");

    Ok(Json(json!({
        "aktif": false,
        "utc": sekarang.to_rfc3339(),
        "pesan": "Waktu server kembali memakai jam sistem.",
    })))
}
