use axum::Json;
use axum::http::header;
use axum::response::IntoResponse;
use chrono::SecondsFormat;
use serde::Serialize;

use crate::service::waktu;

#[derive(Debug, Serialize)]
pub struct WaktuDto {
    /// Epoch milidetik UTC; dipakai klien untuk menghitung offset jam.
    pub epoch_ms: i64,
    pub utc: String,
    pub wita: String,
    pub tanggal: String,
    pub jam: String,
    pub zona: &'static str,
    /// True bila server sedang memakai jam palsu development.
    pub palsu: bool,
}

/// Jam server ringan tanpa akses database, untuk sinkronisasi jam klien.
pub async fn sekarang() -> impl IntoResponse {
    let utc = waktu::sekarang_utc();
    let lokal = waktu::ke_wita(utc);

    let dto = WaktuDto {
        epoch_ms: utc.timestamp_millis(),
        utc: utc.to_rfc3339_opts(SecondsFormat::Millis, true),
        wita: lokal.to_rfc3339_opts(SecondsFormat::Millis, false),
        tanggal: lokal.format("%Y-%m-%d").to_string(),
        jam: lokal.format("%H:%M:%S").to_string(),
        zona: "Asia/Makassar",
        palsu: waktu::override_aktif().is_some(),
    };

    ([(header::CACHE_CONTROL, "no-store")], Json(dto))
}
