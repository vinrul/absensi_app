use std::sync::Arc;
use std::time::{Duration, Instant};

use axum::Json;
use axum::extract::{Request, State};
use axum::http::StatusCode;
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use serde_json::json;
use std::collections::HashMap;
use std::sync::Mutex;

use crate::state::AppState;

/// Pembatas laju sederhana berbasis jendela waktu di memori.
///
/// Cukup untuk satu instans API (deployment Polda memakai satu kontainer).
/// Bila nanti di-scale horizontal, ganti dengan Redis.
pub struct PenjagaLaju {
    jendela: Duration,
    pub batas_umum: usize,
    pub batas_login: usize,
    catatan: Mutex<HashMap<String, Vec<Instant>>>,
}

impl PenjagaLaju {
    pub fn new() -> Self {
        Self {
            jendela: Duration::from_secs(60),
            batas_umum: 120,
            batas_login: 5,
            catatan: Mutex::new(HashMap::new()),
        }
    }

    /// Mengembalikan (diizinkan, sisa_waktu_tunggu_detik).
    fn izinkan(&self, kunci: &str, batas: usize) -> (bool, u64) {
        let sekarang = Instant::now();
        let mut peta = match self.catatan.lock() {
            Ok(peta) => peta,
            Err(racun) => racun.into_inner(),
        };

        // Hindari pertumbuhan tanpa batas pada serangan acak IP.
        if peta.len() > 10_000 {
            peta.clear();
        }

        let daftar = peta.entry(kunci.to_string()).or_default();
        daftar.retain(|waktu| sekarang.duration_since(*waktu) < self.jendela);

        if daftar.len() >= batas {
            let tertua = daftar.first().copied().unwrap_or(sekarang);
            let tunggu = self
                .jendela
                .saturating_sub(sekarang.duration_since(tertua))
                .as_secs()
                .max(1);
            return (false, tunggu);
        }

        daftar.push(sekarang);
        (true, 0)
    }
}

impl Default for PenjagaLaju {
    fn default() -> Self {
        Self::new()
    }
}

pub async fn batasi_laju(State(state): State<AppState>, request: Request, next: Next) -> Response {
    let jalur = request.uri().path().to_string();
    let ip = ip_klien(&request);

    let (batas, kelompok) = if jalur.ends_with("/auth/login") {
        (state.rate.batas_login, "login")
    } else {
        (state.rate.batas_umum, "umum")
    };

    let kunci = format!("{kelompok}:{ip}");
    let (boleh, tunggu) = state.rate.izinkan(&kunci, batas);

    if !boleh {
        tracing::warn!(ip = %ip, jalur = %jalur, "permintaan dibatasi");
        return (
            StatusCode::TOO_MANY_REQUESTS,
            [("retry-after", tunggu.to_string())],
            Json(json!({
                "error": {
                    "code": "too_many_requests",
                    "message": format!("Terlalu banyak permintaan. Coba lagi dalam {tunggu} detik.")
                }
            })),
        )
            .into_response();
    }

    next.run(request).await
}

pub fn ip_klien(request: &Request) -> String {
    request
        .headers()
        .get("x-forwarded-for")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.split(',').next())
        .map(|v| v.trim().to_string())
        .filter(|v| !v.is_empty())
        .unwrap_or_else(|| "lokal".to_string())
}

pub type PenjagaLajuRef = Arc<PenjagaLaju>;
