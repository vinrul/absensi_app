use axum::Json;
use axum::Router;
use axum::extract::{Path, State};
use axum::routing::{get, post, put};
use chrono::NaiveDate;
use serde::Deserialize;
use uuid::Uuid;

use crate::auth::extractor::AuthUser;
use crate::domain::enums::IndikatorKinerja;
use crate::error::{AppError, AppResult};
use crate::repo::periode::{self, KesiapanRow, PeriodeRow};
use crate::repo::satker;
use crate::state::AppState;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/", get(list_periode).post(create_periode))
        .route("/{id}/bobot", put(set_bobot))
        .route("/{id}/tutup", post(tutup_periode))
        .route("/{id}/buka", post(buka_periode))
        .route("/{id}/kesiapan", get(kesiapan_periode))
        .route("/{id}/kesiapan/{satker_id}", post(tandai_kesiapan))
}

pub async fn list_periode(
    State(state): State<AppState>,
    _user: AuthUser,
) -> AppResult<Json<Vec<PeriodeRow>>> {
    Ok(Json(periode::list(&state.pool).await?))
}

#[derive(Debug, Deserialize)]
pub struct CreatePeriodeRequest {
    pub nama: Option<String>,
    pub tanggal_mulai: NaiveDate,
    pub tanggal_selesai: NaiveDate,
}

pub async fn create_periode(
    State(state): State<AppState>,
    user: AuthUser,
    Json(req): Json<CreatePeriodeRequest>,
) -> AppResult<Json<PeriodeRow>> {
    user.require_admin_global()?;

    if req.tanggal_selesai < req.tanggal_mulai {
        return Err(AppError::Unprocessable(
            "Tanggal selesai tidak boleh sebelum tanggal mulai".into(),
        ));
    }

    let nama = req
        .nama
        .unwrap_or_else(|| format!("Periode {}", req.tanggal_mulai.format("%B %Y")));

    let id = Uuid::now_v7();
    periode::create(
        &state.pool,
        id,
        &nama,
        req.tanggal_mulai,
        req.tanggal_selesai,
    )
    .await?;

    let row = periode::find(&state.pool, id)
        .await?
        .ok_or_else(|| AppError::Internal("periode gagal dibuat".into()))?;
    Ok(Json(row))
}

#[derive(Debug, Deserialize)]
pub struct BobotRequest {
    pub absensi: f64,
    pub rencana_kerja: f64,
    pub hasil_kerja: f64,
    pub output: f64,
}

pub async fn set_bobot(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<Uuid>,
    Json(req): Json<BobotRequest>,
) -> AppResult<Json<serde_json::Value>> {
    user.require_admin_global()?;
    periode::pastikan_terbuka(&state.pool, id).await?;

    let nilai = [req.absensi, req.rencana_kerja, req.hasil_kerja, req.output];
    if nilai.iter().any(|v| !(0.0..=100.0).contains(v)) {
        return Err(AppError::Unprocessable(
            "Setiap bobot harus antara 0 dan 100".into(),
        ));
    }
    let total: f64 = nilai.iter().sum();
    if (total - 100.0).abs() > 0.01 {
        return Err(AppError::Unprocessable(format!(
            "Total bobot harus 100 (saat ini {total})"
        )));
    }

    for (indikator, bobot) in [
        (IndikatorKinerja::Absensi, req.absensi),
        (IndikatorKinerja::RencanaKerja, req.rencana_kerja),
        (IndikatorKinerja::HasilKerja, req.hasil_kerja),
        (IndikatorKinerja::Output, req.output),
    ] {
        periode::set_bobot(&state.pool, id, indikator, bobot).await?;
    }

    Ok(Json(serde_json::json!({ "message": "Bobot diperbarui" })))
}

pub async fn tutup_periode(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<Uuid>,
) -> AppResult<Json<serde_json::Value>> {
    user.require_admin_global()?;
    periode::tutup(&state.pool, id, user.id()).await?;
    Ok(Json(serde_json::json!({ "message": "Periode ditutup" })))
}

#[derive(Debug, Deserialize)]
pub struct BukaPeriodeRequest {
    pub alasan: Option<String>,
}

pub async fn buka_periode(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<Uuid>,
    payload: Option<Json<BukaPeriodeRequest>>,
) -> AppResult<Json<serde_json::Value>> {
    user.require_admin_global()?;
    let alasan = payload
        .and_then(|Json(req)| req.alasan)
        .filter(|v| !v.trim().is_empty());
    periode::buka(&state.pool, id, user.id(), alasan.as_deref()).await?;
    Ok(Json(
        serde_json::json!({ "message": "Periode dibuka kembali" }),
    ))
}

#[derive(Debug, serde::Serialize)]
pub struct KesiapanSummary {
    pub total_satker: usize,
    pub siap: usize,
    pub belum_siap: usize,
    pub total_personel: i64,
    pub sudah_dinilai: i64,
    pub belum_dinilai: i64,
}

#[derive(Debug, serde::Serialize)]
pub struct KesiapanResponse {
    pub periode_id: Uuid,
    pub periode_nama: String,
    pub periode_status: crate::domain::enums::PeriodeStatus,
    pub data: Vec<KesiapanRow>,
    pub ringkasan: KesiapanSummary,
}

pub async fn kesiapan_periode(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<Uuid>,
) -> AppResult<Json<KesiapanResponse>> {
    if !user.is_admin_any() {
        return Err(AppError::Forbidden(
            "Hanya admin yang dapat melihat kesiapan periode".into(),
        ));
    }
    let periode_row = periode::find(&state.pool, id)
        .await?
        .ok_or_else(|| AppError::NotFound("Periode tidak ditemukan".into()))?;
    let data = periode::kesiapan(&state.pool, id, user.satker_akses()).await?;
    let ringkasan = KesiapanSummary {
        total_satker: data.len(),
        siap: data.iter().filter(|row| row.siap_ditutup).count(),
        belum_siap: data.iter().filter(|row| !row.siap_ditutup).count(),
        total_personel: data.iter().map(|row| row.jumlah_personel).sum(),
        sudah_dinilai: data.iter().map(|row| row.sudah_dinilai).sum(),
        belum_dinilai: data.iter().map(|row| row.belum_dinilai).sum(),
    };
    Ok(Json(KesiapanResponse {
        periode_id: periode_row.id,
        periode_nama: periode_row.nama,
        periode_status: periode_row.status,
        data,
        ringkasan,
    }))
}

#[derive(Debug, Deserialize)]
pub struct TandaiKesiapanRequest {
    pub siap: bool,
    pub catatan: Option<String>,
}

pub async fn tandai_kesiapan(
    State(state): State<AppState>,
    user: AuthUser,
    Path((id, satker_id)): Path<(Uuid, Uuid)>,
    Json(req): Json<TandaiKesiapanRequest>,
) -> AppResult<Json<serde_json::Value>> {
    if !user.is_admin_any() {
        return Err(AppError::Forbidden(
            "Hanya admin yang dapat menandai kesiapan satker".into(),
        ));
    }
    periode::pastikan_terbuka(&state.pool, id).await?;
    let target = satker::find(&state.pool, satker_id)
        .await?
        .ok_or_else(|| AppError::NotFound("Satker tidak ditemukan".into()))?;
    if target.level != crate::domain::enums::SatkerLevel::Satker {
        return Err(AppError::Unprocessable(
            "Kesiapan hanya untuk satker (bukan Polda/Polres)".into(),
        ));
    }
    user.require_satker(satker_id)?;

    let catatan = req
        .catatan
        .as_deref()
        .map(str::trim)
        .filter(|v| !v.is_empty());
    if catatan.is_some_and(|v| v.chars().count() > 500) {
        return Err(AppError::Unprocessable(
            "Catatan kesiapan maksimal 500 karakter".into(),
        ));
    }
    periode::tandai_siap(&state.pool, id, satker_id, req.siap, user.id(), catatan).await?;
    Ok(Json(serde_json::json!({
        "message": if req.siap { "Satker ditandai siap ditutup" } else { "Tanda siap ditutup dibatalkan" }
    })))
}
