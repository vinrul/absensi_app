use axum::extract::{Request, State};
use axum::http::Method;
use axum::http::header::USER_AGENT;
use axum::middleware::Next;
use axum::response::Response;
use serde_json::json;
use uuid::Uuid;

use crate::auth::jwt;
use crate::middleware::rate_limit::ip_klien;
use crate::repo::audit;
use crate::state::AppState;

/// Mencatat setiap permintaan yang mengubah data (POST/PUT/PATCH/DELETE).
///
/// Detail semantik (before/after) sengaja tidak diambil dari body agar
/// middleware tetap murah; kolom `aksi`, `entitas`, `entitas_id`, dan status
/// respons sudah cukup untuk penelusuran forensik.
pub async fn catat_mutasi(State(state): State<AppState>, request: Request, next: Next) -> Response {
    let metode = request.method().clone();
    let jalur = request.uri().path().to_string();
    let ip = ip_klien(&request);
    let agen = request
        .headers()
        .get(USER_AGENT)
        .and_then(|v| v.to_str().ok())
        .map(|v| v.chars().take(200).collect::<String>());

    let pelaku = request
        .headers()
        .get(axum::http::header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .and_then(|token| jwt::decode(token, &state.settings.jwt_secret).ok())
        .map(|claims| claims.sub);

    let response = next.run(request).await;

    let perlu_catat = matches!(
        metode,
        Method::POST | Method::PUT | Method::PATCH | Method::DELETE
    );

    if perlu_catat {
        let status = response.status().as_u16();
        let (entitas, entitas_id) = uraikan_jalur(&jalur);
        let aksi = format!("{} {}", metode.as_str(), jalur);

        let state_salin = state.clone();
        tokio::spawn(async move {
            let hasil = audit::catat(
                &state_salin.pool,
                pelaku,
                &aksi,
                &entitas,
                entitas_id,
                None,
                Some(json!({ "status": status })),
                Some(&ip),
                agen.as_deref(),
            )
            .await;

            if let Err(galat) = hasil {
                tracing::warn!(error = %galat, "gagal menulis audit log");
            }
        });
    }

    response
}

/// `/api/v1/rencana-kerja/<uuid>/keputusan` -> ("rencana-kerja", Some(uuid))
fn uraikan_jalur(jalur: &str) -> (String, Option<Uuid>) {
    let bersih = jalur.trim_start_matches('/');
    let bersih = bersih.strip_prefix("api/v1/").unwrap_or(bersih);

    let mut segmen = bersih.split('/');
    let entitas = segmen.next().unwrap_or("lainnya").to_string();
    let entitas_id = segmen.find_map(|bagian| Uuid::parse_str(bagian).ok());

    (entitas, entitas_id)
}
