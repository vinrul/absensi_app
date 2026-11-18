use axum::Json;
use axum::Router;
use axum::body::Body;
use axum::extract::{Path, Query, State};
use axum::http::{HeaderValue, StatusCode, header};
use axum::response::Response;
use axum::routing::{get, post};
use base64::Engine;
use base64::engine::general_purpose::STANDARD as BASE64;
use chrono::{DateTime, Datelike, Duration, NaiveDate, NaiveTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

use crate::auth::extractor::AuthUser;
use crate::domain::enums::{AbsensiJenis, StatusKehadiran};
use crate::error::{AppError, AppResult};
use crate::repo::absensi::{
    self, AbsensiHarianRow, AbsensiRow, ListFilter, ListHarianFilter, NewAbsensi,
};
use crate::repo::periode;
use crate::repo::personel;
use crate::repo::satker;
use crate::repo::titik_dinas;
use crate::repo::{ListResponse, pagination};
use crate::service::geofence;
use crate::service::storage;
use crate::service::waktu::{self, ke_wita};
use crate::service::{laporan, pdf};
use crate::state::AppState;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/", get(list_absensi).post(create_absensi))
        .route("/saya", get(list_saya))
        .route("/hari-ini", get(hari_ini))
        .route("/harian", get(list_harian))
        .route("/rekap", get(rekap_absensi))
        .route("/rekap/ekspor", get(ekspor_rekap_pdf))
        .route("/foto", get(list_foto))
        .route("/verifikasi/ekspor", get(ekspor_verifikasi_pdf))
        .route("/detail/ekspor", get(ekspor_detail_pdf))
        .route("/titik-terdekat", get(titik_terdekat))
        .route("/{id}/verifikasi", post(verifikasi_absensi))
}

// ---------------------------------------------------------------------- DTO

#[derive(Debug, Serialize)]
pub struct AbsensiDto {
    pub id: Uuid,
    pub client_uuid: Uuid,
    pub personel_id: Uuid,
    pub tanggal: NaiveDate,
    pub jenis: AbsensiJenis,
    pub status_kehadiran: StatusKehadiran,
    pub jam: String,
    pub captured_at: DateTime<Utc>,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    pub akurasi_m: Option<f64>,
    pub jarak_m: Option<f64>,
    pub dalam_radius: bool,
    pub is_offline: bool,
    pub is_mock_suspect: bool,
    pub perlu_verifikasi: bool,
    pub verified_at: Option<DateTime<Utc>>,
    pub catatan_verifikator: Option<String>,
    pub foto_url: Option<String>,
}

impl From<AbsensiRow> for AbsensiDto {
    fn from(row: AbsensiRow) -> Self {
        Self {
            id: row.id,
            client_uuid: row.client_uuid,
            personel_id: row.personel_id,
            tanggal: row.tanggal,
            jenis: row.jenis,
            status_kehadiran: row.status_kehadiran,
            jam: row.jam.format("%H:%M").to_string(),
            captured_at: row.captured_at,
            latitude: row.latitude,
            longitude: row.longitude,
            akurasi_m: row.akurasi_m,
            jarak_m: row.jarak_m,
            dalam_radius: row.dalam_radius,
            is_offline: row.is_offline,
            is_mock_suspect: row.is_mock_suspect,
            perlu_verifikasi: row.perlu_verifikasi,
            verified_at: row.verified_at,
            catatan_verifikator: row.catatan_verifikator,
            foto_url: row.foto_path.map(|key| format!("/api/v1/files/{key}")),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct GeofenceInfo {
    pub titik_nama: Option<String>,
    pub jarak_m: Option<f64>,
    pub radius_m: Option<i32>,
    pub dalam_radius: bool,
}

#[derive(Debug, Serialize)]
pub struct AbsensiResponse {
    pub data: AbsensiDto,
    pub geofence: GeofenceInfo,
    pub server_time: DateTime<Utc>,
}

// ------------------------------------------------------------------- create

#[derive(Debug, Deserialize)]
pub struct AbsensiRequest {
    pub client_uuid: Uuid,
    pub jenis: AbsensiJenis,
    pub status_kehadiran: StatusKehadiran,
    pub captured_at: DateTime<Utc>,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    pub akurasi_m: Option<f64>,
    pub is_mock_suspect: Option<bool>,
    pub is_offline: Option<bool>,
    pub device_id: Option<String>,
    pub foto_base64: Option<String>,
}

fn koordinat_valid(lat: Option<f64>, lng: Option<f64>) -> Option<(f64, f64)> {
    match (lat, lng) {
        (Some(lat), Some(lng))
            if (-90.0..=90.0).contains(&lat) && (-180.0..=180.0).contains(&lng) =>
        {
            Some((lat, lng))
        }
        _ => None,
    }
}

pub async fn create_absensi(
    State(state): State<AppState>,
    user: AuthUser,
    Json(req): Json<AbsensiRequest>,
) -> AppResult<Json<AbsensiResponse>> {
    let personel = personel::active_personel(&state.pool, user.id()).await?;

    // Idempotensi: antrean offline boleh mencoba ulang berkali-kali.
    if let Some(existing) = absensi::find_by_client_uuid(&state.pool, req.client_uuid).await? {
        let geofence = GeofenceInfo {
            titik_nama: None,
            jarak_m: existing.jarak_m,
            radius_m: None,
            dalam_radius: existing.dalam_radius,
        };
        return Ok(Json(AbsensiResponse {
            data: existing.into(),
            geofence,
            server_time: waktu::sekarang_utc(),
        }));
    }

    // Saat jam palsu aktif (development), tanggal/jam absensi mengikuti jam palsu
    // agar skenario uji tidak bergantung pada jam perangkat.
    let acuan = waktu::override_aktif().unwrap_or(req.captured_at);
    let lokal = ke_wita(acuan);
    let tanggal = lokal.date_naive();
    let jam = lokal.time();
    let hari_ini = waktu::hari_ini_wita();

    if tanggal > hari_ini + Duration::days(1) {
        return Err(AppError::Unprocessable(
            "Tanggal absensi tidak boleh di masa depan".into(),
        ));
    }
    if tanggal < hari_ini - Duration::days(7) {
        return Err(AppError::Unprocessable(
            "Absensi hanya dapat dikirim maksimal 7 hari ke belakang".into(),
        ));
    }

    if absensi::exists_harian(&state.pool, personel.id, tanggal, req.jenis).await? {
        return Err(AppError::Conflict(format!(
            "Absen {} sudah tercatat pada tanggal {}",
            match req.jenis {
                AbsensiJenis::Masuk => "masuk",
                AbsensiJenis::Pulang => "pulang",
            },
            tanggal
        )));
    }

    if req.jenis == AbsensiJenis::Pulang
        && !absensi::exists_harian(&state.pool, personel.id, tanggal, AbsensiJenis::Masuk).await?
    {
        return Err(AppError::Unprocessable(
            "Absen masuk belum tercatat untuk hari ini".into(),
        ));
    }

    // Kebijakan server: development menandai, produksi menolak lokasi tiruan.
    let is_mock = req.is_mock_suspect.unwrap_or(false);
    if state.settings.mock_location_policy.menolak(is_mock) {
        return Err(AppError::Unprocessable(
            "Lokasi tiruan (mock GPS) terdeteksi. Matikan aplikasi lokasi tiruan/Developer Options lalu coba lagi."
                .into(),
        ));
    }

    // ------------------------------------------------------------- geofence
    let titik_rows = titik_dinas::list_aktif_for_satker(&state.pool, personel.satker_id).await?;
    let titik: Vec<geofence::Titik> = titik_rows
        .iter()
        .map(|row| geofence::Titik {
            id: row.id,
            nama: row.nama.clone(),
            latitude: row.latitude,
            longitude: row.longitude,
            radius_m: row.radius_m,
            jenis: row.jenis,
        })
        .collect();

    let butuh_gps = req.status_kehadiran.dihitung_hadir();
    let koordinat = koordinat_valid(req.latitude, req.longitude);

    if butuh_gps && koordinat.is_none() {
        return Err(AppError::Unprocessable(
            "Lokasi GPS wajib diambil untuk status kehadiran ini".into(),
        ));
    }

    if butuh_gps && titik.is_empty() {
        return Err(AppError::Unprocessable(
            "Satker belum memiliki titik dinas aktif, hubungi admin".into(),
        ));
    }

    let akurasi = req.akurasi_m.unwrap_or(0.0).max(0.0);
    let hasil = match koordinat {
        Some((lat, lng)) => geofence::evaluasi(&titik, lat, lng, akurasi),
        None => geofence::evaluasi(&[], 0.0, 0.0, 0.0),
    };

    if req.status_kehadiran.butuh_radius() && !hasil.dalam_radius {
        let jarak = hasil.jarak_m.unwrap_or(0.0);
        let radius = hasil.radius_m.unwrap_or(100);
        return Err(AppError::Unprocessable(format!(
            "Anda berada {jarak:.0} m dari titik dinas (radius {radius} m). Dekati lokasi atau ajukan koreksi ke atasan."
        )));
    }

    if req.status_kehadiran == StatusKehadiran::DinasLuar && hasil.dalam_radius {
        // Dinas luar yang ternyata di dalam radius kantor tetap dicatat apa adanya.
        tracing::info!(personel = %personel.nrp.as_deref().unwrap_or("-"), "dinas luar tercatat di dalam radius kantor");
    }

    // ----------------------------------------------------------------- foto
    let mut foto_path: Option<String> = None;
    if let Some(raw) = req.foto_base64.as_deref().filter(|v| !v.is_empty()) {
        let payload = raw
            .split_once(',')
            .filter(|(prefix, _)| prefix.starts_with("data:"))
            .map(|(_, data)| data)
            .unwrap_or(raw);

        let bytes = BASE64
            .decode(payload)
            .map_err(|_| AppError::BadRequest("Data foto tidak valid".into()))?;

        if bytes.len() > storage::MAX_FOTO_BYTES {
            return Err(AppError::PayloadTooLarge(
                "Ukuran foto maksimal 5 MB".into(),
            ));
        }

        let (ext, _mime) = storage::deteksi_berkas(&bytes)
            .ok_or_else(|| AppError::Unprocessable("Format foto harus JPG atau PNG".into()))?;

        if !matches!(ext, "jpg" | "png") {
            return Err(AppError::Unprocessable(
                "Format foto harus JPG atau PNG".into(),
            ));
        }

        let key = storage::kunci_absensi(personel.satker_id, tanggal, Uuid::now_v7(), ext);
        state.storage.save(&key, &bytes).await?;
        foto_path = Some(key);
    }

    // ---------------------------------------------------------------- simpan
    let perlu_verifikasi = req.is_offline.unwrap_or(false)
        || is_mock
        || akurasi > 100.0
        || (req.status_kehadiran.butuh_radius() && !hasil.dalam_radius)
        || req.status_kehadiran == StatusKehadiran::TanpaKeterangan;

    let id = Uuid::now_v7();

    let inserted = absensi::insert(
        &state.pool,
        &NewAbsensi {
            id,
            client_uuid: req.client_uuid,
            personel_id: personel.id,
            satker_id: personel.satker_id,
            tanggal,
            jenis: req.jenis,
            status_kehadiran: req.status_kehadiran,
            captured_at: acuan,
            jam,
            latitude: koordinat.map(|(lat, _)| lat),
            longitude: koordinat.map(|(_, lng)| lng),
            akurasi_m: req.akurasi_m,
            jarak_m: hasil.jarak_m,
            dalam_radius: hasil.dalam_radius,
            is_mock_suspect: is_mock,
            is_offline: req.is_offline.unwrap_or(false),
            foto_path: foto_path.as_deref(),
            device_id: req.device_id.as_deref(),
            perlu_verifikasi,
        },
    )
    .await?;

    let row = if inserted == 0 {
        absensi::find_by_client_uuid(&state.pool, req.client_uuid)
            .await?
            .ok_or_else(|| AppError::Internal("absensi gagal disimpan".into()))?
    } else {
        absensi::find(&state.pool, id)
            .await?
            .ok_or_else(|| AppError::Internal("absensi gagal dibaca".into()))?
    };

    Ok(Json(AbsensiResponse {
        data: row.into(),
        geofence: GeofenceInfo {
            titik_nama: hasil.titik_nama,
            jarak_m: hasil.jarak_m,
            radius_m: hasil.radius_m,
            dalam_radius: hasil.dalam_radius,
        },
        server_time: waktu::sekarang_utc(),
    }))
}

#[derive(Debug, Serialize)]
pub struct AbsensiHariIniResponse {
    pub tanggal: NaiveDate,
    pub masuk: Option<AbsensiDto>,
    pub pulang: Option<AbsensiDto>,
}

/// Status absensi personel untuk tanggal server hari ini (termasuk jam palsu).
pub async fn hari_ini(
    State(state): State<AppState>,
    user: AuthUser,
) -> AppResult<Json<AbsensiHariIniResponse>> {
    let tanggal = waktu::hari_ini_wita();
    let rows = absensi::list_personel(&state.pool, user.id(), tanggal, tanggal).await?;

    let mut masuk = None;
    let mut pulang = None;
    for row in rows {
        match row.jenis {
            AbsensiJenis::Masuk => masuk = Some(row.into()),
            AbsensiJenis::Pulang => pulang = Some(row.into()),
        }
    }

    Ok(Json(AbsensiHariIniResponse {
        tanggal,
        masuk,
        pulang,
    }))
}

// -------------------------------------------------------------------- query

#[derive(Debug, Deserialize)]
pub struct RiwayatQuery {
    pub dari: Option<NaiveDate>,
    pub sampai: Option<NaiveDate>,
}

pub async fn list_saya(
    State(state): State<AppState>,
    user: AuthUser,
    Query(query): Query<RiwayatQuery>,
) -> AppResult<Json<Vec<AbsensiDto>>> {
    let hari_ini = waktu::hari_ini_wita();
    let sampai = query.sampai.unwrap_or(hari_ini);
    let dari = query.dari.unwrap_or_else(|| {
        NaiveDate::from_ymd_opt(hari_ini.year(), hari_ini.month(), 1).unwrap_or(sampai)
    });

    let rows = absensi::list_personel(&state.pool, user.id(), dari, sampai).await?;
    Ok(Json(rows.into_iter().map(AbsensiDto::from).collect()))
}

#[derive(Debug, Deserialize)]
pub struct ListQuery {
    pub page: Option<i64>,
    pub per_page: Option<i64>,
    pub satker_id: Option<Uuid>,
    pub tanggal: Option<NaiveDate>,
    pub perlu_verifikasi: Option<bool>,
    pub personel_id: Option<Uuid>,
    pub dari: Option<NaiveDate>,
    pub sampai: Option<NaiveDate>,
}

#[derive(Debug, Serialize)]
pub struct AbsensiListDto {
    pub id: Uuid,
    pub personel_id: Uuid,
    pub satker_id: Uuid,
    pub personel_nama: String,
    pub personel_nrp: String,
    pub personel_pangkat: String,
    pub unit: Option<String>,
    pub tanggal: NaiveDate,
    pub jenis: AbsensiJenis,
    pub status_kehadiran: StatusKehadiran,
    pub jam: String,
    pub jarak_m: Option<f64>,
    pub dalam_radius: bool,
    pub is_offline: bool,
    pub is_mock_suspect: bool,
    pub perlu_verifikasi: bool,
    pub verified_at: Option<DateTime<Utc>>,
    pub catatan_verifikator: Option<String>,
    pub foto_url: Option<String>,
}

pub async fn list_absensi(
    State(state): State<AppState>,
    user: AuthUser,
    Query(query): Query<ListQuery>,
) -> AppResult<Json<ListResponse<AbsensiListDto>>> {
    let mut scope = crate::routes::personel::scope_for(&state, &user).await?;

    if let Some(satker_id) = query.satker_id {
        if !scope.contains(&satker_id) {
            return Err(AppError::Forbidden(
                "Anda tidak berhak melihat absensi satker tersebut".into(),
            ));
        }
        scope = vec![satker_id];
    }

    let (page, per_page, offset) = pagination(query.page, query.per_page);
    let hanya_pengelola = user.is_admin_any() || user.is_pimpinan_any() || user.is_atasan();
    let personel_id = if hanya_pengelola {
        query.personel_id
    } else {
        Some(user.id())
    };
    let (rows, total) = absensi::list(
        &state.pool,
        &ListFilter {
            satker_ids: scope,
            tanggal: query.tanggal,
            perlu_verifikasi: query.perlu_verifikasi,
            personel_id,
            dari: query.dari,
            sampai: query.sampai,
            limit: per_page,
            offset,
        },
    )
    .await?;

    let data = rows
        .into_iter()
        .map(|row| AbsensiListDto {
            id: row.id,
            personel_id: row.personel_id,
            satker_id: row.satker_id,
            personel_nama: row.personel_nama,
            personel_nrp: row.personel_nrp,
            personel_pangkat: row.personel_pangkat,
            unit: row.unit,
            tanggal: row.tanggal,
            jenis: row.jenis,
            status_kehadiran: row.status_kehadiran,
            jam: row.jam.format("%H:%M").to_string(),
            jarak_m: row.jarak_m,
            dalam_radius: row.dalam_radius,
            is_offline: row.is_offline,
            is_mock_suspect: row.is_mock_suspect,
            perlu_verifikasi: row.perlu_verifikasi,
            verified_at: row.verified_at,
            catatan_verifikator: row.catatan_verifikator,
            foto_url: row.foto_path.map(|key| format!("/api/v1/files/{key}")),
        })
        .collect();

    Ok(Json(ListResponse::new(data, page, per_page, total)))
}

#[derive(Debug, Deserialize)]
pub struct HarianQuery {
    pub page: Option<i64>,
    pub per_page: Option<i64>,
    pub satker_id: Option<Uuid>,
    pub personel_id: Option<Uuid>,
    pub dari: Option<NaiveDate>,
    pub sampai: Option<NaiveDate>,
    pub perlu_verifikasi: Option<bool>,
}

/// Absensi per tanggal: absen masuk dan pulang digabung dalam satu baris.
#[derive(Debug, Serialize)]
pub struct AbsensiHarianDto {
    pub personel_id: Uuid,
    pub satker_id: Uuid,
    pub personel_nama: String,
    pub personel_nrp: String,
    pub personel_pangkat: String,
    pub unit: Option<String>,
    pub tanggal: NaiveDate,
    pub masuk_jam: Option<String>,
    pub pulang_jam: Option<String>,
    pub status_masuk: Option<StatusKehadiran>,
    pub status_pulang: Option<StatusKehadiran>,
    pub is_offline: bool,
    pub is_mock_suspect: bool,
    pub luar_radius: bool,
    pub perlu_verifikasi: bool,
    pub perlu_verifikasi_ids: Vec<Uuid>,
    pub ph_hadir: bool,
    pub verified_at: Option<DateTime<Utc>>,
    pub catatan_verifikator: Option<String>,
    pub menit_telat: i64,
    pub foto_url: Option<String>,
    pub foto_masuk_url: Option<String>,
    pub foto_pulang_url: Option<String>,
}

impl From<AbsensiHarianRow> for AbsensiHarianDto {
    fn from(row: AbsensiHarianRow) -> Self {
        Self {
            personel_id: row.personel_id,
            satker_id: row.satker_id,
            personel_nama: row.personel_nama,
            personel_nrp: row.personel_nrp,
            personel_pangkat: row.personel_pangkat,
            unit: row.unit,
            tanggal: row.tanggal,
            masuk_jam: row.masuk_jam.map(|jam| jam.format("%H:%M").to_string()),
            pulang_jam: row.pulang_jam.map(|jam| jam.format("%H:%M").to_string()),
            status_masuk: row.status_masuk,
            status_pulang: row.status_pulang,
            is_offline: row.is_offline,
            is_mock_suspect: row.is_mock_suspect,
            luar_radius: row.luar_radius,
            perlu_verifikasi: row.perlu_verifikasi,
            perlu_verifikasi_ids: row.perlu_verifikasi_ids,
            ph_hadir: row.ph_hadir,
            verified_at: row.verified_at,
            catatan_verifikator: row.catatan_verifikator,
            menit_telat: row.menit_telat,
            foto_url: row.foto_path.map(|key| format!("/api/v1/files/{key}")),
            foto_masuk_url: row
                .foto_masuk_path
                .map(|key| format!("/api/v1/files/{key}")),
            foto_pulang_url: row
                .foto_pulang_path
                .map(|key| format!("/api/v1/files/{key}")),
        }
    }
}

pub async fn list_harian(
    State(state): State<AppState>,
    user: AuthUser,
    Query(query): Query<HarianQuery>,
) -> AppResult<Json<ListResponse<AbsensiHarianDto>>> {
    let mut scope = crate::routes::personel::scope_for(&state, &user).await?;

    if let Some(satker_id) = query.satker_id {
        if !scope.contains(&satker_id) {
            return Err(AppError::Forbidden(
                "Anda tidak berhak melihat absensi satker tersebut".into(),
            ));
        }
        scope = vec![satker_id];
    }

    let (page, per_page, offset) = pagination(query.page, query.per_page);
    let hanya_pengelola = user.is_admin_any() || user.is_pimpinan_any() || user.is_atasan();
    let personel_id = if hanya_pengelola {
        query.personel_id
    } else {
        Some(user.id())
    };

    let (rows, total) = absensi::list_harian(
        &state.pool,
        &ListHarianFilter {
            satker_ids: scope,
            personel_id,
            dari: query.dari,
            sampai: query.sampai,
            perlu_verifikasi: query.perlu_verifikasi,
            limit: per_page,
            offset,
        },
    )
    .await?;

    let data = rows.into_iter().map(AbsensiHarianDto::from).collect();
    Ok(Json(ListResponse::new(data, page, per_page, total)))
}

#[derive(Debug, Deserialize)]
pub struct FotoQuery {
    pub periode_id: Option<Uuid>,
    pub satker_id: Option<Uuid>,
    pub personel_id: Option<Uuid>,
}

#[derive(Debug, Serialize)]
pub struct FotoAbsensiDto {
    pub id: Uuid,
    pub tanggal: NaiveDate,
    pub jenis: AbsensiJenis,
    pub jam: String,
    pub status_kehadiran: StatusKehadiran,
    pub foto_url: String,
}

/// Daftar foto dokumentasi absensi untuk galeri rekap/detail per orang.
pub async fn list_foto(
    State(state): State<AppState>,
    user: AuthUser,
    Query(query): Query<FotoQuery>,
) -> AppResult<Json<Vec<FotoAbsensiDto>>> {
    pastikan_pengelola_absensi(&user)?;
    let (periode, scope) = periode_dan_scope(
        &state,
        &user,
        query.periode_id,
        query.satker_id,
        "Anda tidak berhak melihat foto absensi satker tersebut",
    )
    .await?;

    let rows = absensi::list_foto(
        &state.pool,
        &scope,
        query.personel_id,
        periode.tanggal_mulai,
        periode.tanggal_selesai,
    )
    .await?;

    Ok(Json(
        rows.into_iter()
            .map(|row| FotoAbsensiDto {
                id: row.id,
                tanggal: row.tanggal,
                jenis: row.jenis,
                jam: row.jam.format("%H:%M").to_string(),
                status_kehadiran: row.status_kehadiran,
                foto_url: format!("/api/v1/files/{}", row.foto_path),
            })
            .collect(),
    ))
}

#[derive(Debug, Deserialize)]
pub struct RekapQuery {
    pub periode_id: Option<Uuid>,
    pub satker_id: Option<Uuid>,
}

pub async fn rekap_absensi(
    State(state): State<AppState>,
    user: AuthUser,
    Query(query): Query<RekapQuery>,
) -> AppResult<Json<serde_json::Value>> {
    if !(user.is_admin_any() || user.is_pimpinan_any() || user.is_atasan()) {
        return Err(AppError::Forbidden(
            "Rekap absensi hanya untuk atasan/pimpinan/admin".into(),
        ));
    }
    let periode = periode::find_or_current(&state.pool, query.periode_id).await?;

    let mut scope = crate::routes::personel::scope_for(&state, &user).await?;
    if let Some(satker_id) = query.satker_id {
        if !scope.contains(&satker_id) {
            return Err(AppError::Forbidden(
                "Anda tidak berhak melihat rekap satker tersebut".into(),
            ));
        }
        scope = vec![satker_id];
    }

    let rows = absensi::rekap(
        &state.pool,
        &scope,
        periode.tanggal_mulai,
        periode.tanggal_selesai,
    )
    .await?;

    Ok(Json(serde_json::json!({
        "periode": {
            "id": periode.id,
            "nama": periode.nama,
            "tanggal_mulai": periode.tanggal_mulai,
            "tanggal_selesai": periode.tanggal_selesai,
        },
        "data": rows,
    })))
}

// ---------------------------------------------------------------- ekspor PDF

#[derive(Debug, Deserialize)]
pub struct RekapEksporQuery {
    pub periode_id: Option<Uuid>,
    pub satker_id: Option<Uuid>,
}

#[derive(Debug, Deserialize)]
pub struct EksporQuery {
    pub periode_id: Option<Uuid>,
    pub satker_id: Option<Uuid>,
    pub personel_id: Option<Uuid>,
}

fn pastikan_pengelola_absensi(user: &AuthUser) -> AppResult<()> {
    if user.is_admin_any() || user.is_pimpinan_any() || user.is_atasan() {
        Ok(())
    } else {
        Err(AppError::Forbidden(
            "Rekap absensi hanya untuk atasan/pimpinan/admin".into(),
        ))
    }
}

/// Periode terpilih + cakupan satker yang boleh diakses pengguna.
async fn periode_dan_scope(
    state: &AppState,
    user: &AuthUser,
    periode_id: Option<Uuid>,
    satker_id: Option<Uuid>,
    pesan_satker: &str,
) -> AppResult<(periode::PeriodeRow, Vec<Uuid>)> {
    let periode = periode::find_or_current(&state.pool, periode_id).await?;
    let mut scope = crate::routes::personel::scope_for(state, user).await?;

    if let Some(satker_id) = satker_id {
        if !scope.contains(&satker_id) {
            return Err(AppError::Forbidden(pesan_satker.into()));
        }
        scope = vec![satker_id];
    }

    Ok((periode, scope))
}

fn label_periode(periode: &periode::PeriodeRow) -> String {
    format!(
        "{} ({} s/d {})",
        periode.nama,
        laporan::format_tanggal(periode.tanggal_mulai),
        laporan::format_tanggal(periode.tanggal_selesai)
    )
}

async fn nama_satker_terpilih(state: &AppState, satker_id: Option<Uuid>) -> AppResult<String> {
    match satker_id {
        Some(id) => Ok(satker::find(&state.pool, id)
            .await?
            .map(|row| row.nama)
            .unwrap_or_else(|| "-".into())),
        None => Ok("Semua Satker".into()),
    }
}

async fn nama_personel_terpilih(state: &AppState, personel_id: Option<Uuid>) -> AppResult<String> {
    match personel_id {
        Some(id) => Ok(personel::find_by_id(&state.pool, id)
            .await?
            .map(|row| match row.nrp {
                Some(nrp) => format!("{} ({})", row.nama, nrp),
                None => row.nama,
            })
            .unwrap_or_else(|| "-".into())),
        None => Ok("Semua Personel".into()),
    }
}

async fn nama_pencetak(state: &AppState, user: &AuthUser) -> AppResult<String> {
    Ok(personel::find_by_id(&state.pool, user.id())
        .await?
        .map(|row| row.nama)
        .unwrap_or_else(|| "-".into()))
}

fn waktu_cetak() -> String {
    (chrono::Utc::now() + Duration::hours(8))
        .format("%d-%m-%Y %H:%M WITA")
        .to_string()
}

fn respons_pdf(isi: Vec<u8>, nama_berkas: String) -> Response {
    let disposisi = HeaderValue::from_str(&format!("attachment; filename=\"{nama_berkas}\""))
        .unwrap_or_else(|_| HeaderValue::from_static("attachment"));

    let mut response = Response::new(Body::from(isi));
    *response.status_mut() = StatusCode::OK;
    response.headers_mut().insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("application/pdf"),
    );
    response
        .headers_mut()
        .insert(header::CONTENT_DISPOSITION, disposisi);
    response
}

fn label_status_kehadiran(status: StatusKehadiran) -> &'static str {
    match status {
        StatusKehadiran::DinasKantor => "Dinas Kantor",
        StatusKehadiran::DinasLuar => "Dinas Luar",
        StatusKehadiran::Ph => "PH",
        StatusKehadiran::Piket => "Piket/Shift",
        StatusKehadiran::LiburNasional => "Libur Nasional",
        StatusKehadiran::Cuti => "Cuti",
        StatusKehadiran::Izin => "Izin",
        StatusKehadiran::Sakit => "Sakit",
        StatusKehadiran::TanpaKeterangan => "Tanpa Keterangan",
    }
}

fn label_status_harian(row: &AbsensiHarianRow) -> &'static str {
    match (row.status_masuk, row.status_pulang) {
        (Some(status), _) => label_status_kehadiran(status),
        (None, Some(status)) => label_status_kehadiran(status),
        _ => "-",
    }
}

fn label_jam(jam: Option<NaiveTime>) -> String {
    jam.map(|nilai| nilai.format("%H:%M").to_string())
        .unwrap_or_else(|| "-".into())
}

fn label_menit_telat(menit: i64) -> String {
    if menit > 0 {
        format!("{menit} mnt")
    } else {
        "-".into()
    }
}

fn lokasi_satker(
    nama_satker_map: &HashMap<Uuid, String>,
    satker_id: Uuid,
    unit: Option<&str>,
) -> String {
    match (nama_satker_map.get(&satker_id), unit) {
        (Some(satker), Some(unit)) => format!("{satker} · {unit}"),
        (Some(satker), None) => satker.clone(),
        (None, _) => "-".into(),
    }
}

/// Catatan ringkas kondisi absensi (offline / mock GPS / luar radius).
fn catatan_kondisi(offline: bool, mock: bool, dalam_radius: bool) -> String {
    let mut bagian = Vec::new();
    if offline {
        bagian.push("Absen offline");
    }
    if mock {
        bagian.push("Terdeteksi mock GPS");
    }
    if !dalam_radius {
        bagian.push("Di luar radius");
    }
    if bagian.is_empty() {
        "-".into()
    } else {
        bagian.join(", ")
    }
}

/// Ekspor rekap kehadiran per personel sesuai filter menjadi PDF.
pub async fn ekspor_rekap_pdf(
    State(state): State<AppState>,
    user: AuthUser,
    Query(query): Query<RekapEksporQuery>,
) -> AppResult<Response> {
    pastikan_pengelola_absensi(&user)?;
    let (periode, scope) = periode_dan_scope(
        &state,
        &user,
        query.periode_id,
        query.satker_id,
        "Anda tidak berhak mengekspor rekap satker tersebut",
    )
    .await?;

    let rows = absensi::rekap(
        &state.pool,
        &scope,
        periode.tanggal_mulai,
        periode.tanggal_selesai,
    )
    .await?;

    let nama_satker = nama_satker_terpilih(&state, query.satker_id).await?;
    let dicetak_oleh = nama_pencetak(&state, &user).await?;
    let dicetak_pada = waktu_cetak();
    let nama_satker_map: HashMap<Uuid, String> = satker::list(&state.pool)
        .await?
        .into_iter()
        .map(|row| (row.id, row.nama))
        .collect();

    let mut baris = String::new();
    for (indeks, row) in rows.iter().enumerate() {
        baris.push_str(&format!(
            "<tr>\
                <td style=\"text-align:center\">{}</td>\
                <td><strong>{}</strong><br><span style=\"color:#64748b\">{}</span></td>\
                <td>{}</td>\
                <td style=\"text-align:center\">{}</td>\
                <td style=\"text-align:center\">{}</td>\
                <td style=\"text-align:center\">{}</td>\
                <td style=\"text-align:center\">{}</td>\
                <td style=\"text-align:center\">{}</td>\
                <td style=\"text-align:center\">{}</td>\
                <td style=\"text-align:center\">{}</td>\
                <td style=\"text-align:center\">{}</td>\
                <td style=\"text-align:center\">{}</td>\
                <td style=\"text-align:center\">{}</td>\
                <td style=\"text-align:center\">{}</td>\
             </tr>",
            indeks + 1,
            laporan::esc(&row.nama),
            laporan::esc(&row.nrp),
            laporan::esc(&lokasi_satker(
                &nama_satker_map,
                row.satker_id,
                row.unit.as_deref()
            )),
            row.hadir,
            row.telat,
            row.menit_telat,
            row.dinas_luar,
            row.piket,
            row.cuti,
            row.izin,
            row.sakit,
            row.libur_nasional,
            row.tanpa_keterangan,
            row.menunggu_verifikasi,
        ));
    }

    if !rows.is_empty() {
        let jumlah = |ambil: fn(&absensi::RekapRow) -> i64| rows.iter().map(ambil).sum::<i64>();
        baris.push_str(&format!(
            "<tr style=\"font-weight:600;background:#e2e8f0\">\
                <td colspan=\"3\" style=\"text-align:right\">Jumlah</td>\
                <td style=\"text-align:center\">{}</td>\
                <td style=\"text-align:center\">{}</td>\
                <td style=\"text-align:center\">{}</td>\
                <td style=\"text-align:center\">{}</td>\
                <td style=\"text-align:center\">{}</td>\
                <td style=\"text-align:center\">{}</td>\
                <td style=\"text-align:center\">{}</td>\
                <td style=\"text-align:center\">{}</td>\
                <td style=\"text-align:center\">{}</td>\
                <td style=\"text-align:center\">{}</td>\
                <td style=\"text-align:center\">{}</td>\
             </tr>",
            jumlah(|row| row.hadir),
            jumlah(|row| row.telat),
            jumlah(|row| row.menit_telat),
            jumlah(|row| row.dinas_luar),
            jumlah(|row| row.piket),
            jumlah(|row| row.cuti),
            jumlah(|row| row.izin),
            jumlah(|row| row.sakit),
            jumlah(|row| row.libur_nasional),
            jumlah(|row| row.tanpa_keterangan),
            jumlah(|row| row.menunggu_verifikasi),
        ));
    }

    let html = laporan::html(&laporan::Laporan {
        judul: "Rekap Kehadiran Personel",
        periode: &label_periode(&periode),
        satker: &nama_satker,
        personel: "Semua Personel",
        status: "Semua status kehadiran",
        dicetak_oleh: &dicetak_oleh,
        dicetak_pada: &dicetak_pada,
        kolom: vec![
            laporan::Kolom {
                judul: "No",
                lebar: 4,
            },
            laporan::Kolom {
                judul: "Personel",
                lebar: 22,
            },
            laporan::Kolom {
                judul: "Satker / Unit",
                lebar: 16,
            },
            laporan::Kolom {
                judul: "Hadir",
                lebar: 5,
            },
            laporan::Kolom {
                judul: "Telat",
                lebar: 5,
            },
            laporan::Kolom {
                judul: "Menit",
                lebar: 5,
            },
            laporan::Kolom {
                judul: "DL",
                lebar: 5,
            },
            laporan::Kolom {
                judul: "Piket",
                lebar: 5,
            },
            laporan::Kolom {
                judul: "Cuti",
                lebar: 5,
            },
            laporan::Kolom {
                judul: "Izin",
                lebar: 5,
            },
            laporan::Kolom {
                judul: "Sakit",
                lebar: 5,
            },
            laporan::Kolom {
                judul: "PH",
                lebar: 5,
            },
            laporan::Kolom {
                judul: "TK",
                lebar: 5,
            },
            laporan::Kolom {
                judul: "Menunggu",
                lebar: 8,
            },
        ],
        baris,
        jumlah: rows.len(),
        total: rows.len() as i64,
        catatan: Some(format!(
            "Menit telat hanya menghitung hari tanpa check-in strong point. Total {} personel.",
            rows.len()
        )),
    });
    let isi_pdf = pdf::render_pdf(&html).await?;

    Ok(respons_pdf(
        isi_pdf,
        format!(
            "rekap-absensi-{}.pdf",
            waktu::hari_ini_wita().format("%Y%m%d")
        ),
    ))
}

/// Ekspor daftar absensi yang menunggu verifikasi sesuai filter menjadi PDF.
pub async fn ekspor_verifikasi_pdf(
    State(state): State<AppState>,
    user: AuthUser,
    Query(query): Query<EksporQuery>,
) -> AppResult<Response> {
    pastikan_pengelola_absensi(&user)?;
    let (periode, scope) = periode_dan_scope(
        &state,
        &user,
        query.periode_id,
        query.satker_id,
        "Anda tidak berhak mengekspor absensi satker tersebut",
    )
    .await?;

    let (rows, total) = absensi::list_harian(
        &state.pool,
        &ListHarianFilter {
            satker_ids: scope,
            personel_id: query.personel_id,
            dari: Some(periode.tanggal_mulai),
            sampai: Some(periode.tanggal_selesai),
            perlu_verifikasi: Some(true),
            limit: 5000,
            offset: 0,
        },
    )
    .await?;

    let nama_satker = nama_satker_terpilih(&state, query.satker_id).await?;
    let nama_personel = nama_personel_terpilih(&state, query.personel_id).await?;
    let dicetak_oleh = nama_pencetak(&state, &user).await?;
    let dicetak_pada = waktu_cetak();
    let nama_satker_map: HashMap<Uuid, String> = satker::list(&state.pool)
        .await?
        .into_iter()
        .map(|row| (row.id, row.nama))
        .collect();

    let mut baris = String::new();
    for (indeks, row) in rows.iter().enumerate() {
        baris.push_str(&format!(
            "<tr>\
                <td style=\"text-align:center\">{}</td>\
                <td><strong>{}</strong><br><span style=\"color:#64748b\">{}</span></td>\
                <td>{}</td>\
                <td>{}</td>\
                <td style=\"text-align:center\">{}</td>\
                <td style=\"text-align:center\">{}</td>\
                <td>{}</td>\
                <td style=\"text-align:center\">{}</td>\
                <td>{}</td>\
             </tr>",
            indeks + 1,
            laporan::esc(&row.personel_nama),
            laporan::esc(&row.personel_nrp),
            laporan::esc(&lokasi_satker(
                &nama_satker_map,
                row.satker_id,
                row.unit.as_deref()
            )),
            laporan::format_tanggal(row.tanggal),
            label_jam(row.masuk_jam),
            label_jam(row.pulang_jam),
            label_status_harian(row),
            label_menit_telat(row.menit_telat),
            laporan::esc(&catatan_kondisi(
                row.is_offline,
                row.is_mock_suspect,
                !row.luar_radius
            )),
        ));
    }

    let html = laporan::html(&laporan::Laporan {
        judul: "Daftar Absensi Menunggu Verifikasi",
        periode: &label_periode(&periode),
        satker: &nama_satker,
        personel: &nama_personel,
        status: "Menunggu verifikasi",
        dicetak_oleh: &dicetak_oleh,
        dicetak_pada: &dicetak_pada,
        kolom: vec![
            laporan::Kolom {
                judul: "No",
                lebar: 4,
            },
            laporan::Kolom {
                judul: "Personel",
                lebar: 20,
            },
            laporan::Kolom {
                judul: "Satker / Unit",
                lebar: 15,
            },
            laporan::Kolom {
                judul: "Tanggal",
                lebar: 9,
            },
            laporan::Kolom {
                judul: "Masuk",
                lebar: 6,
            },
            laporan::Kolom {
                judul: "Pulang",
                lebar: 6,
            },
            laporan::Kolom {
                judul: "Status",
                lebar: 12,
            },
            laporan::Kolom {
                judul: "Telat",
                lebar: 6,
            },
            laporan::Kolom {
                judul: "Kondisi",
                lebar: 22,
            },
        ],
        baris,
        jumlah: rows.len(),
        total,
        catatan: None,
    });
    let isi_pdf = pdf::render_pdf(&html).await?;

    Ok(respons_pdf(
        isi_pdf,
        format!(
            "verifikasi-absensi-{}.pdf",
            waktu::hari_ini_wita().format("%Y%m%d")
        ),
    ))
}

/// Ekspor detail absensi satu personel pada periode terpilih menjadi PDF.
pub async fn ekspor_detail_pdf(
    State(state): State<AppState>,
    user: AuthUser,
    Query(query): Query<EksporQuery>,
) -> AppResult<Response> {
    pastikan_pengelola_absensi(&user)?;
    let (periode, scope) = periode_dan_scope(
        &state,
        &user,
        query.periode_id,
        query.satker_id,
        "Anda tidak berhak mengekspor absensi satker tersebut",
    )
    .await?;

    let personel_id = query.personel_id.ok_or_else(|| {
        AppError::BadRequest("Pilih personel terlebih dahulu sebelum mengekspor detail".into())
    })?;

    let orang = personel::find_by_id(&state.pool, personel_id)
        .await?
        .ok_or_else(|| AppError::NotFound("Personel tidak ditemukan".into()))?;
    if !scope.contains(&orang.satker_id) {
        return Err(AppError::Forbidden(
            "Anda tidak berhak mengekspor absensi personel tersebut".into(),
        ));
    }

    let (rows, total) = absensi::list_harian(
        &state.pool,
        &ListHarianFilter {
            satker_ids: scope,
            personel_id: Some(personel_id),
            dari: Some(periode.tanggal_mulai),
            sampai: Some(periode.tanggal_selesai),
            perlu_verifikasi: None,
            limit: 5000,
            offset: 0,
        },
    )
    .await?;

    let nama_personel = match orang.nrp.clone() {
        Some(nrp) => format!("{} ({})", orang.nama, nrp),
        None => orang.nama.clone(),
    };
    let dicetak_oleh = nama_pencetak(&state, &user).await?;
    let dicetak_pada = waktu_cetak();
    let nama_satker_map: HashMap<Uuid, String> = satker::list(&state.pool)
        .await?
        .into_iter()
        .map(|row| (row.id, row.nama))
        .collect();
    let mut nama_satker = nama_satker_terpilih(&state, query.satker_id).await?;
    if query.satker_id.is_none() {
        nama_satker = lokasi_satker(&nama_satker_map, orang.satker_id, orang.unit.as_deref());
    }

    let mut baris = String::new();
    for (indeks, row) in rows.iter().enumerate() {
        let verifikasi = if row.perlu_verifikasi {
            "Menunggu".to_string()
        } else if let Some(waktu) = row.verified_at {
            format!("Terverifikasi {}", ke_wita(waktu).format("%d-%m-%Y %H:%M"))
        } else {
            "-".to_string()
        };

        let catatan = match row.catatan_verifikator.as_deref() {
            Some(catatan) if !catatan.trim().is_empty() => format!(
                "{} · {}",
                catatan_kondisi(row.is_offline, row.is_mock_suspect, !row.luar_radius),
                catatan
            ),
            _ => catatan_kondisi(row.is_offline, row.is_mock_suspect, !row.luar_radius),
        };
        let catatan = if row.ph_hadir {
            if catatan == "-" {
                "Strong point".to_string()
            } else {
                format!("Strong point · {catatan}")
            }
        } else {
            catatan
        };

        baris.push_str(&format!(
            "<tr>\
                <td style=\"text-align:center\">{}</td>\
                <td>{}</td>\
                <td style=\"text-align:center\">{}</td>\
                <td style=\"text-align:center\">{}</td>\
                <td>{}</td>\
                <td style=\"text-align:center\">{}</td>\
                <td style=\"text-align:center\">{}</td>\
                <td>{}</td>\
                <td>{}</td>\
             </tr>",
            indeks + 1,
            laporan::format_tanggal(row.tanggal),
            label_jam(row.masuk_jam),
            label_jam(row.pulang_jam),
            label_status_harian(row),
            label_menit_telat(row.menit_telat),
            if row.luar_radius { "Tidak" } else { "Ya" },
            laporan::esc(&verifikasi),
            laporan::esc(&catatan),
        ));
    }

    let html = laporan::html(&laporan::Laporan {
        judul: "Detail Absensi Personel",
        periode: &label_periode(&periode),
        satker: &nama_satker,
        personel: &nama_personel,
        status: "Semua jenis absensi",
        dicetak_oleh: &dicetak_oleh,
        dicetak_pada: &dicetak_pada,
        kolom: vec![
            laporan::Kolom {
                judul: "No",
                lebar: 4,
            },
            laporan::Kolom {
                judul: "Tanggal",
                lebar: 11,
            },
            laporan::Kolom {
                judul: "Masuk",
                lebar: 7,
            },
            laporan::Kolom {
                judul: "Pulang",
                lebar: 7,
            },
            laporan::Kolom {
                judul: "Status",
                lebar: 16,
            },
            laporan::Kolom {
                judul: "Telat",
                lebar: 7,
            },
            laporan::Kolom {
                judul: "Radius",
                lebar: 8,
            },
            laporan::Kolom {
                judul: "Verifikasi",
                lebar: 14,
            },
            laporan::Kolom {
                judul: "Catatan",
                lebar: 26,
            },
        ],
        baris,
        jumlah: rows.len(),
        total,
        catatan: None,
    });
    let isi_pdf = pdf::render_pdf(&html).await?;

    Ok(respons_pdf(
        isi_pdf,
        format!(
            "detail-absensi-{}.pdf",
            waktu::hari_ini_wita().format("%Y%m%d")
        ),
    ))
}

#[derive(Debug, Deserialize)]
pub struct TitikQuery {
    pub latitude: f64,
    pub longitude: f64,
    pub akurasi_m: Option<f64>,
    pub satker_id: Option<Uuid>,
}

#[derive(Debug, Serialize)]
pub struct TitikTerdekatDto {
    pub titik: Vec<serde_json::Value>,
    pub terdekat: GeofenceInfo,
}

pub async fn titik_terdekat(
    State(state): State<AppState>,
    user: AuthUser,
    Query(query): Query<TitikQuery>,
) -> AppResult<Json<TitikTerdekatDto>> {
    let satker_id = match query.satker_id {
        Some(id) => {
            user.require_satker(id)?;
            id
        }
        None => user.claims().satker_id,
    };

    let rows = titik_dinas::list_aktif_for_satker(&state.pool, satker_id).await?;
    let titik: Vec<geofence::Titik> = rows
        .iter()
        .map(|row| geofence::Titik {
            id: row.id,
            nama: row.nama.clone(),
            latitude: row.latitude,
            longitude: row.longitude,
            radius_m: row.radius_m,
            jenis: row.jenis,
        })
        .collect();

    let hasil = geofence::evaluasi(
        &titik,
        query.latitude,
        query.longitude,
        query.akurasi_m.unwrap_or(0.0),
    );

    let daftar = rows
        .iter()
        .map(|row| {
            serde_json::json!({
                "id": row.id,
                "nama": row.nama,
                "jenis": row.jenis,
                "latitude": row.latitude,
                "longitude": row.longitude,
                "radius_m": row.radius_m,
                "jarak_m": (geofence::haversine_m(query.latitude, query.longitude, row.latitude, row.longitude) * 100.0).round() / 100.0,
                "is_active": row.is_active,
            })
        })
        .collect();

    Ok(Json(TitikTerdekatDto {
        titik: daftar,
        terdekat: GeofenceInfo {
            titik_nama: hasil.titik_nama,
            jarak_m: hasil.jarak_m,
            radius_m: hasil.radius_m,
            dalam_radius: hasil.dalam_radius,
        },
    }))
}

// --------------------------------------------------------------- verifikasi

#[derive(Debug, Deserialize)]
pub struct VerifikasiRequest {
    pub catatan: Option<String>,
}

pub async fn verifikasi_absensi(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<Uuid>,
    Json(req): Json<VerifikasiRequest>,
) -> AppResult<Json<AbsensiDto>> {
    if !(user.is_admin_any() || user.is_atasan()) {
        return Err(AppError::Forbidden(
            "Hanya atasan atau admin yang dapat memverifikasi absensi".into(),
        ));
    }

    let existing = absensi::find(&state.pool, id)
        .await?
        .ok_or_else(|| AppError::NotFound("Data absensi tidak ditemukan".into()))?;
    user.require_satker(existing.satker_id)?;

    absensi::verifikasi(&state.pool, id, user.id(), req.catatan.as_deref()).await?;

    let row = absensi::find(&state.pool, id)
        .await?
        .ok_or_else(|| AppError::NotFound("Data absensi tidak ditemukan".into()))?;
    Ok(Json(row.into()))
}
