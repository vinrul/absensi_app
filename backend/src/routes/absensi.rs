use axum::Json;
use axum::Router;
use axum::extract::{Path, Query, State};
use axum::routing::{get, post};
use base64::Engine;
use base64::engine::general_purpose::STANDARD as BASE64;
use chrono::{DateTime, Datelike, Duration, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::auth::extractor::AuthUser;
use crate::domain::enums::{AbsensiJenis, StatusKehadiran};
use crate::error::{AppError, AppResult};
use crate::repo::absensi::{self, AbsensiRow, NewAbsensi};
use crate::repo::periode;
use crate::repo::personel;
use crate::repo::titik_dinas;
use crate::repo::{ListResponse, pagination};
use crate::service::geofence;
use crate::service::storage;
use crate::service::waktu::{self, ke_wita};
use crate::state::AppState;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/", get(list_absensi).post(create_absensi))
        .route("/saya", get(list_saya))
        .route("/hari-ini", get(hari_ini))
        .route("/rekap", get(rekap_absensi))
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
            foto_url: row.foto_path.map(|key| format!("/files/{key}")),
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
        || req.is_mock_suspect.unwrap_or(false)
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
            is_mock_suspect: req.is_mock_suspect.unwrap_or(false),
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
}

#[derive(Debug, Serialize)]
pub struct AbsensiListDto {
    pub id: Uuid,
    pub personel_id: Uuid,
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
        &scope,
        query.tanggal,
        query.perlu_verifikasi,
        personel_id,
        per_page,
        offset,
    )
    .await?;

    let data = rows
        .into_iter()
        .map(|row| AbsensiListDto {
            id: row.id,
            personel_id: row.personel_id,
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
            foto_url: row.foto_path.map(|key| format!("/files/{key}")),
        })
        .collect();

    Ok(Json(ListResponse::new(data, page, per_page, total)))
}

#[derive(Debug, Deserialize)]
pub struct RekapQuery {
    pub periode_id: Option<Uuid>,
    pub satker_id: Option<Uuid>,
    pub unit: Option<String>,
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
        query.unit.as_deref(),
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
