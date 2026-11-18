use axum::Json;
use axum::Router;
use axum::body::Body;
use axum::extract::{Path, Query, State};
use axum::http::{HeaderValue, StatusCode, header};
use axum::response::Response;
use axum::routing::{get, patch, post};
use base64::Engine;
use base64::engine::general_purpose::STANDARD as BASE64;
use chrono::{Datelike, Duration, NaiveDate, NaiveTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use uuid::Uuid;

use crate::auth::extractor::AuthUser;
use crate::domain::enums::{AbsensiJenis, JenisAkun, TitikDinasJenis};
use crate::error::{AppError, AppResult};
use crate::repo::personel;
use crate::repo::ph::{
    self, AbsensiPhRow, JadwalPhRow, JadwalPhSayaRow, NewAbsensiPh, StrongPointRow,
};
use crate::repo::satker;
use crate::service::waktu::{hari_ini_wita, ke_wita};
use crate::service::{akses, geofence, laporan, pdf, storage};
use crate::state::AppState;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route(
            "/strong-point",
            get(list_strong_point).post(create_strong_point),
        )
        .route(
            "/strong-point/{id}",
            patch(update_strong_point).delete(delete_strong_point),
        )
        .route("/jadwal", get(list_jadwal).post(create_jadwal))
        .route("/jadwal/ekspor", get(ekspor_jadwal_pdf))
        .route("/jadwal/saya", get(jadwal_saya))
        .route(
            "/jadwal/{id}",
            get(detail_jadwal).put(update_jadwal).delete(delete_jadwal),
        )
        .route("/absensi", post(create_absensi_ph))
        .route("/absensi/saya", get(absensi_saya))
        .route("/absensi/jadwal/{id}", get(absensi_jadwal))
        .route("/absensi/{id}/verifikasi", post(verifikasi_absensi_ph))
        .route("/rekap", get(rekap_ph))
        .route("/rekap/ekspor", get(ekspor_rekap_pdf))
}

#[derive(Debug, Serialize)]
pub struct StrongPointDto {
    pub id: Uuid,
    pub satker_id: Uuid,
    pub nama: String,
    pub alamat: Option<String>,
    pub latitude: f64,
    pub longitude: f64,
    pub radius_m: i32,
    pub is_active: bool,
}

impl From<StrongPointRow> for StrongPointDto {
    fn from(row: StrongPointRow) -> Self {
        Self {
            id: row.id,
            satker_id: row.satker_id,
            nama: row.nama,
            alamat: row.alamat,
            latitude: row.latitude,
            longitude: row.longitude,
            radius_m: row.radius_m,
            is_active: row.is_active,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct PersonelJadwalDto {
    pub personel_id: Uuid,
    pub nama: String,
    pub nrp: String,
    pub pangkat: String,
    pub unit: Option<String>,
    pub peran: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct JadwalDto {
    pub id: Uuid,
    pub satker_id: Uuid,
    pub strong_point_id: Uuid,
    pub strong_point_nama: String,
    pub tanggal: NaiveDate,
    pub jam_mulai: String,
    pub jam_selesai: String,
    pub regu: Option<String>,
    pub catatan: Option<String>,
    pub jumlah_personel: i64,
    pub personel: Vec<PersonelJadwalDto>,
}

#[derive(Debug, Serialize)]
pub struct JadwalSayaDto {
    pub id: Uuid,
    pub satker_id: Uuid,
    pub strong_point_id: Uuid,
    pub strong_point_nama: String,
    pub latitude: f64,
    pub longitude: f64,
    pub radius_m: i32,
    pub tanggal: NaiveDate,
    pub jam_mulai: String,
    pub jam_selesai: String,
    pub regu: Option<String>,
    pub catatan: Option<String>,
    pub peran: Option<String>,
}

impl From<JadwalPhSayaRow> for JadwalSayaDto {
    fn from(row: JadwalPhSayaRow) -> Self {
        Self {
            id: row.id,
            satker_id: row.satker_id,
            strong_point_id: row.strong_point_id,
            strong_point_nama: row.strong_point_nama,
            latitude: row.latitude,
            longitude: row.longitude,
            radius_m: row.radius_m,
            tanggal: row.tanggal,
            jam_mulai: row.jam_mulai.format("%H:%M").to_string(),
            jam_selesai: row.jam_selesai.format("%H:%M").to_string(),
            regu: row.regu,
            catatan: row.catatan,
            peran: row.peran,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct AbsensiPhDto {
    pub id: Uuid,
    pub jadwal_ph_id: Uuid,
    pub personel_id: Uuid,
    pub personel_nama: String,
    pub personel_nrp: String,
    pub strong_point_id: Uuid,
    pub strong_point_nama: String,
    pub satker_id: Uuid,
    pub jenis: AbsensiJenis,
    pub captured_at: chrono::DateTime<chrono::Utc>,
    pub jam: String,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    pub akurasi_m: Option<f64>,
    pub jarak_m: Option<f64>,
    pub dalam_radius: bool,
    pub foto_url: Option<String>,
    pub is_offline: bool,
    pub is_mock_suspect: bool,
    pub perlu_verifikasi: bool,
    pub verified_at: Option<chrono::DateTime<chrono::Utc>>,
    pub catatan_verifikator: Option<String>,
}

impl From<AbsensiPhRow> for AbsensiPhDto {
    fn from(row: AbsensiPhRow) -> Self {
        Self {
            id: row.id,
            jadwal_ph_id: row.jadwal_ph_id,
            personel_id: row.personel_id,
            personel_nama: row.personel_nama,
            personel_nrp: row.personel_nrp,
            strong_point_id: row.strong_point_id,
            strong_point_nama: row.strong_point_nama,
            satker_id: row.satker_id,
            jenis: row.jenis,
            captured_at: row.captured_at,
            jam: row.jam.format("%H:%M").to_string(),
            latitude: row.latitude,
            longitude: row.longitude,
            akurasi_m: row.akurasi_m,
            jarak_m: row.jarak_m,
            dalam_radius: row.dalam_radius,
            foto_url: row.foto_path.map(|key| format!("/files/{key}")),
            is_offline: row.is_offline,
            is_mock_suspect: row.is_mock_suspect,
            perlu_verifikasi: row.perlu_verifikasi,
            verified_at: row.verified_at,
            catatan_verifikator: row.catatan_verifikator,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct RekapPhDto {
    pub personel_id: Uuid,
    pub nama: String,
    pub nrp: String,
    pub pangkat: String,
    pub unit: Option<String>,
    pub jumlah_jadwal: i64,
    pub hadir: i64,
    pub hanya_check_in: i64,
    pub tidak_hadir: i64,
    pub tanggal_tidak_hadir: Vec<NaiveDate>,
    pub tanggal_hanya_check_in: Vec<NaiveDate>,
    pub menunggu_verifikasi: i64,
}

#[derive(Debug, Deserialize)]
pub struct BulanQuery {
    pub bulan: Option<String>,
    pub satker_id: Option<Uuid>,
}

fn bulan_ini_query() -> (NaiveDate, NaiveDate) {
    let hari = hari_ini_wita();
    awal_akhir_bulan(hari.year(), hari.month()).expect("bulan valid")
}

fn awal_akhir_bulan(tahun: i32, bulan: u32) -> AppResult<(NaiveDate, NaiveDate)> {
    let mulai = NaiveDate::from_ymd_opt(tahun, bulan, 1)
        .ok_or_else(|| AppError::Unprocessable("Bulan tidak valid".into()))?;
    let (tahun_akhir, bulan_akhir) = if bulan == 12 {
        (tahun + 1, 1)
    } else {
        (tahun, bulan + 1)
    };
    let akhir = NaiveDate::from_ymd_opt(tahun_akhir, bulan_akhir, 1)
        .ok_or_else(|| AppError::Unprocessable("Bulan tidak valid".into()))?
        - chrono::Duration::days(1);
    Ok((mulai, akhir))
}

fn rentang_bulan(bulan: Option<&str>) -> AppResult<(NaiveDate, NaiveDate)> {
    let Some(value) = bulan.filter(|v| !v.trim().is_empty()) else {
        return Ok(bulan_ini_query());
    };
    let (tahun, bulan) = value
        .split_once('-')
        .ok_or_else(|| AppError::Unprocessable("Format bulan harus YYYY-MM".into()))?;
    let tahun: i32 = tahun
        .parse()
        .map_err(|_| AppError::Unprocessable("Tahun tidak valid".into()))?;
    let bulan: u32 = bulan
        .parse()
        .map_err(|_| AppError::Unprocessable("Bulan tidak valid".into()))?;
    awal_akhir_bulan(tahun, bulan)
}

fn parse_time(value: &str) -> AppResult<NaiveTime> {
    NaiveTime::parse_from_str(value, "%H:%M")
        .or_else(|_| NaiveTime::parse_from_str(value, "%H:%M:%S"))
        .map_err(|_| AppError::Unprocessable(format!("Format jam tidak valid: {value}")))
}

fn validasi_koordinat(lat: f64, lng: f64) -> AppResult<()> {
    if !(-90.0..=90.0).contains(&lat) || !(-180.0..=180.0).contains(&lng) {
        return Err(AppError::Unprocessable(
            "Koordinat strong point tidak valid".into(),
        ));
    }
    Ok(())
}

fn validasi_radius(radius: i32) -> AppResult<()> {
    if !(10..=5000).contains(&radius) {
        return Err(AppError::Unprocessable(
            "Radius harus antara 10 dan 5000 meter".into(),
        ));
    }
    Ok(())
}

fn pastikan_pengelola(user: &AuthUser) -> AppResult<()> {
    if user.is_admin_any() || user.is_pimpinan_any() || user.is_atasan() {
        Ok(())
    } else {
        Err(AppError::Forbidden(
            "Menu PH hanya untuk atasan/pimpinan/admin".into(),
        ))
    }
}

fn pastikan_admin(user: &AuthUser) -> AppResult<()> {
    if user.is_admin_any() {
        Ok(())
    } else {
        Err(AppError::Forbidden(
            "Hanya admin yang dapat mengelola jadwal PH".into(),
        ))
    }
}

async fn pastikan_satker_dikelola(
    state: &AppState,
    user: &AuthUser,
    satker_id: Uuid,
) -> AppResult<()> {
    user.require_satker(satker_id)?;
    if !akses::boleh_kelola_personel_id(&state.pool, user.role_scopes(), satker_id).await? {
        return Err(AppError::Forbidden(
            "Satker berada di luar cakupan admin".into(),
        ));
    }
    Ok(())
}

async fn jadwal_dto(state: &AppState, row: JadwalPhRow) -> AppResult<JadwalDto> {
    let personel = ph::personel_jadwal(&state.pool, row.id)
        .await?
        .into_iter()
        .map(|p| PersonelJadwalDto {
            personel_id: p.personel_id,
            nama: p.nama,
            nrp: p.nrp,
            pangkat: p.pangkat,
            unit: p.unit,
            peran: p.peran,
        })
        .collect();
    Ok(JadwalDto {
        id: row.id,
        satker_id: row.satker_id,
        strong_point_id: row.strong_point_id,
        strong_point_nama: row.strong_point_nama,
        tanggal: row.tanggal,
        jam_mulai: row.jam_mulai.format("%H:%M").to_string(),
        jam_selesai: row.jam_selesai.format("%H:%M").to_string(),
        regu: row.regu,
        catatan: row.catatan,
        jumlah_personel: row.jumlah_personel,
        personel,
    })
}

// ---------------------------------------------------------------- strong point

pub async fn list_strong_point(
    State(state): State<AppState>,
    user: AuthUser,
) -> AppResult<Json<Vec<StrongPointDto>>> {
    let rows = ph::list_strong_points(&state.pool, user.satker_akses()).await?;
    Ok(Json(rows.into_iter().map(StrongPointDto::from).collect()))
}

#[derive(Debug, Deserialize)]
pub struct StrongPointRequest {
    pub satker_id: Uuid,
    pub nama: String,
    pub alamat: Option<String>,
    pub latitude: f64,
    pub longitude: f64,
    pub radius_m: Option<i32>,
}

pub async fn create_strong_point(
    State(state): State<AppState>,
    user: AuthUser,
    Json(req): Json<StrongPointRequest>,
) -> AppResult<Json<StrongPointDto>> {
    pastikan_admin(&user)?;
    pastikan_satker_dikelola(&state, &user, req.satker_id).await?;
    let nama = req.nama.trim();
    if nama.is_empty() {
        return Err(AppError::BadRequest("Nama strong point wajib diisi".into()));
    }
    validasi_koordinat(req.latitude, req.longitude)?;
    let radius = req.radius_m.unwrap_or(100);
    validasi_radius(radius)?;

    let id = Uuid::now_v7();
    ph::create_strong_point(
        &state.pool,
        id,
        req.satker_id,
        nama,
        req.alamat.as_deref(),
        req.latitude,
        req.longitude,
        radius,
    )
    .await?;
    let row = ph::find_strong_point(&state.pool, id)
        .await?
        .ok_or_else(|| AppError::Internal("strong point gagal dibuat".into()))?;
    Ok(Json(row.into()))
}

#[derive(Debug, Deserialize)]
pub struct UpdateStrongPointRequest {
    pub nama: Option<String>,
    pub alamat: Option<String>,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    pub radius_m: Option<i32>,
    pub is_active: Option<bool>,
}

pub async fn update_strong_point(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateStrongPointRequest>,
) -> AppResult<Json<StrongPointDto>> {
    pastikan_admin(&user)?;
    let existing = ph::find_strong_point(&state.pool, id)
        .await?
        .ok_or_else(|| AppError::NotFound("Strong point tidak ditemukan".into()))?;
    pastikan_satker_dikelola(&state, &user, existing.satker_id).await?;
    if let Some(lat) = req.latitude {
        validasi_koordinat(lat, req.longitude.unwrap_or(existing.longitude))?;
    }
    if let Some(lng) = req.longitude {
        validasi_koordinat(req.latitude.unwrap_or(existing.latitude), lng)?;
    }
    if let Some(radius) = req.radius_m {
        validasi_radius(radius)?;
    }
    ph::update_strong_point(
        &state.pool,
        id,
        req.nama.as_deref().map(str::trim).filter(|v| !v.is_empty()),
        req.alamat.as_deref(),
        req.latitude,
        req.longitude,
        req.radius_m,
        req.is_active,
    )
    .await?;
    let row = ph::find_strong_point(&state.pool, id)
        .await?
        .ok_or_else(|| AppError::NotFound("Strong point tidak ditemukan".into()))?;
    Ok(Json(row.into()))
}

pub async fn delete_strong_point(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<Uuid>,
) -> AppResult<Json<serde_json::Value>> {
    pastikan_admin(&user)?;
    let existing = ph::find_strong_point(&state.pool, id)
        .await?
        .ok_or_else(|| AppError::NotFound("Strong point tidak ditemukan".into()))?;
    pastikan_satker_dikelola(&state, &user, existing.satker_id).await?;
    ph::delete_strong_point(&state.pool, id).await?;
    Ok(Json(
        serde_json::json!({ "message": "Strong point dihapus" }),
    ))
}

// ------------------------------------------------------------------ jadwal PH

pub async fn list_jadwal(
    State(state): State<AppState>,
    user: AuthUser,
    Query(query): Query<BulanQuery>,
) -> AppResult<Json<Vec<JadwalDto>>> {
    pastikan_pengelola(&user)?;
    let (dari, sampai) = rentang_bulan(query.bulan.as_deref())?;
    let mut scope = user.satker_akses().to_vec();
    if let Some(satker_id) = query.satker_id {
        if !scope.contains(&satker_id) {
            return Err(AppError::Forbidden(
                "Anda tidak berhak melihat satker tersebut".into(),
            ));
        }
        scope = vec![satker_id];
    }
    let rows = ph::list_jadwal(&state.pool, &scope, dari, sampai).await?;
    let mut data = Vec::with_capacity(rows.len());
    for row in rows {
        data.push(jadwal_dto(&state, row).await?);
    }
    Ok(Json(data))
}

pub async fn jadwal_saya(
    State(state): State<AppState>,
    user: AuthUser,
    Query(query): Query<BulanQuery>,
) -> AppResult<Json<Vec<JadwalSayaDto>>> {
    let (dari, sampai) = rentang_bulan(query.bulan.as_deref())?;
    let rows = ph::jadwal_saya(&state.pool, user.id(), dari, sampai).await?;
    Ok(Json(rows.into_iter().map(JadwalSayaDto::from).collect()))
}

pub async fn detail_jadwal(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<Uuid>,
) -> AppResult<Json<JadwalDto>> {
    let row = ph::find_jadwal(&state.pool, id)
        .await?
        .ok_or_else(|| AppError::NotFound("Jadwal PH tidak ditemukan".into()))?;
    if !(user.is_admin_any() || user.is_pimpinan_any() || user.is_atasan()) {
        if !user.satker_akses().contains(&row.satker_id)
            || !ph::personel_ditugaskan(&state.pool, id, user.id()).await?
        {
            return Err(AppError::Forbidden(
                "Anda tidak berhak melihat jadwal PH ini".into(),
            ));
        }
    } else {
        user.require_satker(row.satker_id)?;
    }
    Ok(Json(jadwal_dto(&state, row).await?))
}

#[derive(Debug, Deserialize)]
pub struct PersonelJadwalInput {
    pub personel_id: Uuid,
    pub peran: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct JadwalRequest {
    pub satker_id: Uuid,
    /// Opsional: dashboard tidak lagi meminta strong point (belum ada titik tetap).
    pub strong_point_id: Option<Uuid>,
    pub tanggal: NaiveDate,
    /// Opsional: bila diisi, satu jadwal dibuat untuk setiap tanggal dalam rentang.
    pub tanggal_selesai: Option<NaiveDate>,
    /// Opsional: jadwal dashboard kini hanya per tanggal.
    pub jam_mulai: Option<String>,
    pub jam_selesai: Option<String>,
    /// Nama regu untuk tampilan roster (mis. "Regu 1").
    pub regu: Option<String>,
    pub catatan: Option<String>,
    pub personel: Vec<PersonelJadwalInput>,
}

const JAM_MULAI_BAWAAN: &str = "00:00";
const JAM_SELESAI_BAWAAN: &str = "23:59";

/// Strong point dipilih eksplisit bila dikirim; jika tidak, pakai strong point aktif
/// pertama milik satker agar check-in/radius lama tetap berjalan.
async fn strong_point_jadwal(
    state: &AppState,
    satker_id: Uuid,
    diminta: Option<Uuid>,
) -> AppResult<Uuid> {
    if let Some(id) = diminta {
        let titik = ph::find_strong_point(&state.pool, id)
            .await?
            .ok_or_else(|| AppError::Unprocessable("Strong point tidak ditemukan".into()))?;
        if titik.satker_id != satker_id {
            return Err(AppError::Unprocessable(
                "Strong point bukan milik satker jadwal".into(),
            ));
        }
        return Ok(id);
    }

    let rows = ph::list_strong_points(&state.pool, &[satker_id]).await?;
    rows.into_iter()
        .find(|row| row.is_active)
        .map(|row| row.id)
        .ok_or_else(|| {
            AppError::Unprocessable(
                "Satker belum memiliki strong point aktif. Hubungi admin satker.".into(),
            )
        })
}

async fn validasi_personel_jadwal(
    state: &AppState,
    satker_id: Uuid,
    personel: &[PersonelJadwalInput],
) -> AppResult<Vec<(Uuid, Option<String>)>> {
    if personel.is_empty() {
        return Err(AppError::Unprocessable(
            "Jadwal PH minimal memiliki satu personel".into(),
        ));
    }
    let mut hasil = Vec::with_capacity(personel.len());
    let mut sudah = std::collections::HashSet::new();
    for item in personel {
        if !sudah.insert(item.personel_id) {
            return Err(AppError::Unprocessable(
                "Personel yang sama muncul lebih dari sekali".into(),
            ));
        }
        let row = personel::find_by_id(&state.pool, item.personel_id)
            .await?
            .ok_or_else(|| AppError::Unprocessable("Personel jadwal tidak ditemukan".into()))?;
        if row.jenis_akun != JenisAkun::Personel {
            return Err(AppError::Unprocessable(
                "Akun admin tidak dapat dijadwalkan PH".into(),
            ));
        }
        if row.satker_id != satker_id {
            return Err(AppError::Unprocessable(
                "Semua personel jadwal harus berasal dari satker yang sama".into(),
            ));
        }
        if !row.is_active {
            return Err(AppError::Unprocessable(format!(
                "Personel {} tidak aktif",
                row.nama
            )));
        }
        hasil.push((item.personel_id, item.peran.clone()));
    }
    Ok(hasil)
}

pub async fn create_jadwal(
    State(state): State<AppState>,
    user: AuthUser,
    Json(req): Json<JadwalRequest>,
) -> AppResult<Json<JadwalDto>> {
    pastikan_admin(&user)?;
    pastikan_satker_dikelola(&state, &user, req.satker_id).await?;
    let strong_point_id = strong_point_jadwal(&state, req.satker_id, req.strong_point_id).await?;
    let jam_mulai = parse_time(req.jam_mulai.as_deref().unwrap_or(JAM_MULAI_BAWAAN))?;
    let jam_selesai = parse_time(req.jam_selesai.as_deref().unwrap_or(JAM_SELESAI_BAWAAN))?;
    if jam_mulai == jam_selesai {
        return Err(AppError::Unprocessable(
            "Jam mulai dan jam selesai tidak boleh sama".into(),
        ));
    }
    let personel = validasi_personel_jadwal(&state, req.satker_id, &req.personel).await?;
    let ids: Vec<Uuid> = personel.iter().map(|(id, _)| *id).collect();

    let sampai = req.tanggal_selesai.unwrap_or(req.tanggal);
    if sampai < req.tanggal {
        return Err(AppError::Unprocessable(
            "Tanggal selesai tidak boleh sebelum tanggal mulai".into(),
        ));
    }
    if (sampai - req.tanggal).num_days() > 62 {
        return Err(AppError::Unprocessable(
            "Rentang jadwal maksimal 63 hari".into(),
        ));
    }

    let mut tanggal = req.tanggal;
    while tanggal <= sampai {
        if ph::ada_bentrok(&state.pool, &ids, tanggal, jam_mulai, jam_selesai, None).await? {
            return Err(AppError::Conflict(format!(
                "Sebagian personel sudah memiliki jadwal strong point pada {tanggal}"
            )));
        }
        tanggal += chrono::Duration::days(1);
    }

    let mut pertama = None;
    let mut tanggal = req.tanggal;
    while tanggal <= sampai {
        let id = Uuid::now_v7();
        ph::create_jadwal(
            &state.pool,
            &ph::JadwalBaru {
                id,
                satker_id: req.satker_id,
                strong_point_id,
                tanggal,
                jam_mulai,
                jam_selesai,
                regu: req.regu.as_deref(),
                catatan: req.catatan.as_deref(),
                created_by: user.id(),
                personel: &personel,
            },
        )
        .await?;
        if pertama.is_none() {
            pertama = Some(id);
        }
        tanggal += chrono::Duration::days(1);
    }

    let id = pertama.ok_or_else(|| AppError::Internal("jadwal PH gagal dibuat".into()))?;
    let row = ph::find_jadwal(&state.pool, id)
        .await?
        .ok_or_else(|| AppError::Internal("jadwal PH gagal dibuat".into()))?;
    Ok(Json(jadwal_dto(&state, row).await?))
}

pub async fn update_jadwal(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<Uuid>,
    Json(req): Json<JadwalRequest>,
) -> AppResult<Json<JadwalDto>> {
    pastikan_admin(&user)?;
    let existing = ph::find_jadwal(&state.pool, id)
        .await?
        .ok_or_else(|| AppError::NotFound("Jadwal PH tidak ditemukan".into()))?;
    pastikan_satker_dikelola(&state, &user, existing.satker_id).await?;
    if req.satker_id != existing.satker_id {
        return Err(AppError::Unprocessable(
            "Satker jadwal tidak boleh dipindah lewat pembaruan".into(),
        ));
    }
    let strong_point_id = match req.strong_point_id {
        Some(id) => strong_point_jadwal(&state, existing.satker_id, Some(id)).await?,
        None => existing.strong_point_id,
    };
    let jam_mulai = match req.jam_mulai.as_deref() {
        Some(nilai) => parse_time(nilai)?,
        None => existing.jam_mulai,
    };
    let jam_selesai = match req.jam_selesai.as_deref() {
        Some(nilai) => parse_time(nilai)?,
        None => existing.jam_selesai,
    };
    if jam_mulai == jam_selesai {
        return Err(AppError::Unprocessable(
            "Jam mulai dan jam selesai tidak boleh sama".into(),
        ));
    }
    let personel = validasi_personel_jadwal(&state, existing.satker_id, &req.personel).await?;
    let ids: Vec<Uuid> = personel.iter().map(|(id, _)| *id).collect();
    if ph::ada_bentrok(
        &state.pool,
        &ids,
        req.tanggal,
        jam_mulai,
        jam_selesai,
        Some(id),
    )
    .await?
    {
        return Err(AppError::Conflict(
            "Sebagian personel sudah memiliki jadwal PH lain yang beririsan".into(),
        ));
    }
    ph::update_jadwal(
        &state.pool,
        id,
        strong_point_id,
        req.tanggal,
        jam_mulai,
        jam_selesai,
        req.regu.as_deref(),
        req.catatan.as_deref(),
        &personel,
    )
    .await?;
    let row = ph::find_jadwal(&state.pool, id)
        .await?
        .ok_or_else(|| AppError::NotFound("Jadwal PH tidak ditemukan".into()))?;
    Ok(Json(jadwal_dto(&state, row).await?))
}

pub async fn delete_jadwal(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<Uuid>,
) -> AppResult<Json<serde_json::Value>> {
    pastikan_admin(&user)?;
    let existing = ph::find_jadwal(&state.pool, id)
        .await?
        .ok_or_else(|| AppError::NotFound("Jadwal PH tidak ditemukan".into()))?;
    pastikan_satker_dikelola(&state, &user, existing.satker_id).await?;
    ph::delete_jadwal(&state.pool, id).await?;
    Ok(Json(serde_json::json!({ "message": "Jadwal PH dihapus" })))
}

// ---------------------------------------------------------------- absensi PH

#[derive(Debug, Deserialize)]
pub struct AbsensiPhRequest {
    pub client_uuid: Uuid,
    pub jadwal_ph_id: Uuid,
    pub jenis: AbsensiJenis,
    pub captured_at: chrono::DateTime<chrono::Utc>,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    pub akurasi_m: Option<f64>,
    pub is_mock_suspect: Option<bool>,
    pub is_offline: Option<bool>,
    pub foto_base64: Option<String>,
}

pub async fn create_absensi_ph(
    State(state): State<AppState>,
    user: AuthUser,
    Json(req): Json<AbsensiPhRequest>,
) -> AppResult<Json<AbsensiPhDto>> {
    if let Some(existing) = ph::find_absensi_ph_by_client(&state.pool, req.client_uuid).await? {
        if existing.personel_id != user.id() {
            return Err(AppError::Conflict("client_uuid sudah dipakai".into()));
        }
        return Ok(Json(existing.into()));
    }

    let jadwal = ph::find_jadwal(&state.pool, req.jadwal_ph_id)
        .await?
        .ok_or_else(|| AppError::NotFound("Jadwal PH tidak ditemukan".into()))?;
    if !ph::personel_ditugaskan(&state.pool, jadwal.id, user.id()).await? {
        return Err(AppError::Forbidden(
            "Anda tidak ditugaskan pada jadwal PH ini".into(),
        ));
    }
    let strong_point = ph::find_strong_point(&state.pool, jadwal.strong_point_id)
        .await?
        .ok_or_else(|| AppError::Unprocessable("Strong point jadwal tidak ditemukan".into()))?;

    let lokal = ke_wita(req.captured_at);
    let tanggal = lokal.date_naive();
    let jam = lokal.time();
    if tanggal != jadwal.tanggal {
        return Err(AppError::Unprocessable(
            "Absensi PH harus dilakukan pada tanggal jadwal".into(),
        ));
    }

    // Satu jadwal dapat memiliki beberapa sesi check-in (mis. pagi dan sore).
    // Sesi dianggap terbuka bila check-in terakhir lebih baru dari check-out terakhir.
    let sesi = ph::sesi_ph_terakhir(&state.pool, jadwal.id, user.id()).await?;
    let terbuka = match (sesi.masuk_terakhir, sesi.pulang_terakhir) {
        (Some(masuk), Some(pulang)) => masuk > pulang,
        (Some(_), None) => true,
        _ => false,
    };
    match req.jenis {
        AbsensiJenis::Masuk => {
            if terbuka {
                let masuk = sesi.masuk_terakhir.expect("sesi terbuka memiliki check-in");
                let selisih = req.captured_at.signed_duration_since(masuk);
                if selisih < Duration::hours(2) {
                    return Err(AppError::Conflict(format!(
                        "Check-in strong point pukul {} masih berlangsung. Lakukan check-out atau tunggu 2 jam untuk check-in baru.",
                        ke_wita(masuk).format("%H:%M")
                    )));
                }
            }
        }
        AbsensiJenis::Pulang => {
            if !terbuka {
                return Err(AppError::Unprocessable(
                    "Check-in strong point belum tercatat".into(),
                ));
            }
            if let Some(masuk) = sesi.masuk_terakhir
                && req.captured_at < masuk
            {
                return Err(AppError::Unprocessable(
                    "Waktu check-out tidak valid".into(),
                ));
            }
        }
    }

    let (lat, lng) = match (req.latitude, req.longitude) {
        (Some(lat), Some(lng))
            if (-90.0..=90.0).contains(&lat) && (-180.0..=180.0).contains(&lng) =>
        {
            (lat, lng)
        }
        _ => {
            return Err(AppError::Unprocessable(
                "Lokasi GPS wajib diambil untuk absensi PH".into(),
            ));
        }
    };
    let akurasi = req.akurasi_m.unwrap_or(0.0).max(0.0);
    let titik = vec![geofence::Titik {
        id: strong_point.id,
        nama: strong_point.nama.clone(),
        latitude: strong_point.latitude,
        longitude: strong_point.longitude,
        radius_m: strong_point.radius_m,
        jenis: TitikDinasJenis::Kantor,
    }];
    let hasil = geofence::evaluasi(&titik, lat, lng, akurasi);
    if !hasil.dalam_radius {
        return Err(AppError::Unprocessable(format!(
            "Anda berada {:.0} m dari strong point {} (radius {} m). Absensi PH ditolak.",
            hasil.jarak_m.unwrap_or(0.0),
            strong_point.nama,
            strong_point.radius_m
        )));
    }

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
        let (ext, _) = storage::deteksi_berkas(&bytes)
            .ok_or_else(|| AppError::Unprocessable("Format foto harus JPG atau PNG".into()))?;
        if !matches!(ext, "jpg" | "png") {
            return Err(AppError::Unprocessable(
                "Format foto harus JPG atau PNG".into(),
            ));
        }
        let id = Uuid::now_v7();
        let key = storage::kunci_ph(strong_point.satker_id, tanggal, id, ext);
        state.storage.save(&key, &bytes).await?;
        foto_path = Some(key);
    }

    let perlu_verifikasi =
        req.is_offline.unwrap_or(false) || req.is_mock_suspect.unwrap_or(false) || akurasi > 100.0;
    let id = Uuid::now_v7();
    ph::insert_absensi_ph(
        &state.pool,
        &NewAbsensiPh {
            id,
            client_uuid: req.client_uuid,
            jadwal_ph_id: jadwal.id,
            personel_id: user.id(),
            strong_point_id: strong_point.id,
            satker_id: jadwal.satker_id,
            jenis: req.jenis,
            captured_at: req.captured_at,
            jam,
            latitude: Some(lat),
            longitude: Some(lng),
            akurasi_m: req.akurasi_m,
            jarak_m: hasil.jarak_m,
            dalam_radius: hasil.dalam_radius,
            foto_path: foto_path.clone(),
            is_offline: req.is_offline.unwrap_or(false),
            is_mock_suspect: req.is_mock_suspect.unwrap_or(false),
            perlu_verifikasi,
        },
    )
    .await?;

    let row = ph::find_absensi_ph(&state.pool, id)
        .await?
        .ok_or_else(|| AppError::Internal("absensi PH gagal dibaca".into()))?;
    Ok(Json(row.into()))
}

pub async fn absensi_saya(
    State(state): State<AppState>,
    user: AuthUser,
    Query(query): Query<BulanQuery>,
) -> AppResult<Json<Vec<AbsensiPhDto>>> {
    let (dari, sampai) = rentang_bulan(query.bulan.as_deref())?;
    let rows = ph::list_absensi_ph_saya(&state.pool, user.id(), dari, sampai).await?;
    Ok(Json(rows.into_iter().map(AbsensiPhDto::from).collect()))
}

pub async fn absensi_jadwal(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<Uuid>,
) -> AppResult<Json<Vec<AbsensiPhDto>>> {
    pastikan_pengelola(&user)?;
    let jadwal = ph::find_jadwal(&state.pool, id)
        .await?
        .ok_or_else(|| AppError::NotFound("Jadwal PH tidak ditemukan".into()))?;
    user.require_satker(jadwal.satker_id)?;
    let rows = ph::list_absensi_ph_jadwal(&state.pool, id).await?;
    Ok(Json(rows.into_iter().map(AbsensiPhDto::from).collect()))
}

pub async fn verifikasi_absensi_ph(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<Uuid>,
    Json(req): Json<VerifikasiPhRequest>,
) -> AppResult<Json<AbsensiPhDto>> {
    if !(user.is_admin_any() || user.is_atasan()) {
        return Err(AppError::Forbidden(
            "Hanya atasan atau admin yang dapat memverifikasi absensi PH".into(),
        ));
    }
    let row = ph::find_absensi_ph(&state.pool, id)
        .await?
        .ok_or_else(|| AppError::NotFound("Absensi PH tidak ditemukan".into()))?;
    user.require_satker(row.satker_id)?;
    ph::verifikasi_absensi_ph(&state.pool, id, user.id(), req.catatan.as_deref()).await?;
    let row = ph::find_absensi_ph(&state.pool, id)
        .await?
        .ok_or_else(|| AppError::NotFound("Absensi PH tidak ditemukan".into()))?;
    Ok(Json(row.into()))
}

#[derive(Debug, Deserialize)]
pub struct VerifikasiPhRequest {
    pub catatan: Option<String>,
}

pub async fn rekap_ph(
    State(state): State<AppState>,
    user: AuthUser,
    Query(query): Query<BulanQuery>,
) -> AppResult<Json<serde_json::Value>> {
    pastikan_pengelola(&user)?;
    let (dari, sampai_bulan) = rentang_bulan(query.bulan.as_deref())?;
    // Rekap hanya menghitung jadwal sampai hari ini; tanggal yang belum berjalan
    // tidak boleh dianggap "tidak strong point".
    let sampai = sampai_bulan.min(hari_ini_wita());
    let mut scope = user.satker_akses().to_vec();
    if let Some(satker_id) = query.satker_id {
        if !scope.contains(&satker_id) {
            return Err(AppError::Forbidden(
                "Anda tidak berhak melihat satker tersebut".into(),
            ));
        }
        scope = vec![satker_id];
    }
    let rows = if dari > sampai {
        Vec::new()
    } else {
        ph::rekap_ph(&state.pool, &scope, dari, sampai).await?
    };
    let data: Vec<RekapPhDto> = rows
        .into_iter()
        .map(|row| RekapPhDto {
            personel_id: row.personel_id,
            nama: row.nama,
            nrp: row.nrp,
            pangkat: row.pangkat,
            unit: row.unit,
            jumlah_jadwal: row.jumlah_jadwal,
            hadir: row.hadir,
            hanya_check_in: row.hanya_check_in,
            tidak_hadir: row.tidak_hadir,
            tanggal_tidak_hadir: row.tanggal_tidak_hadir,
            tanggal_hanya_check_in: row.tanggal_hanya_check_in,
            menunggu_verifikasi: row.menunggu_verifikasi,
        })
        .collect();
    Ok(Json(serde_json::json!({
        "periode": { "dari": dari, "sampai": sampai },
        "data": data,
    })))
}

// --------------------------------------------------------------- ekspor PDF

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

fn scope_ekspor(user: &AuthUser, satker_id: Option<Uuid>) -> AppResult<Vec<Uuid>> {
    let mut scope = user.satker_akses().to_vec();
    if let Some(id) = satker_id {
        if !scope.contains(&id) {
            return Err(AppError::Forbidden(
                "Anda tidak berhak mengekspor satker tersebut".into(),
            ));
        }
        scope = vec![id];
    }
    Ok(scope)
}

async fn nama_satker_ekspor(state: &AppState, satker_id: Option<Uuid>) -> AppResult<String> {
    match satker_id {
        Some(id) => Ok(satker::find(&state.pool, id)
            .await?
            .map(|row| row.nama)
            .unwrap_or_else(|| "-".into())),
        None => Ok("Semua Satker".into()),
    }
}

async fn nama_pencetak(state: &AppState, user: &AuthUser) -> AppResult<String> {
    Ok(personel::find_by_id(&state.pool, user.id())
        .await?
        .map(|row| row.nama)
        .unwrap_or_else(|| "-".into()))
}

fn waktu_cetak() -> String {
    (Utc::now() + Duration::hours(8))
        .format("%d-%m-%Y %H:%M WITA")
        .to_string()
}

fn label_periode_bulan(dari: NaiveDate, sampai: NaiveDate) -> String {
    format!(
        "{} s.d. {}",
        laporan::format_tanggal(dari),
        laporan::format_tanggal(sampai)
    )
}

fn label_tanggal_pendek(daftar: &[NaiveDate]) -> String {
    if daftar.is_empty() {
        return "-".into();
    }
    daftar
        .iter()
        .map(|tanggal| tanggal.format("%d/%m").to_string())
        .collect::<Vec<_>>()
        .join(", ")
}

/// Sel jumlah yang dilengkapi daftar tanggal pendek di bawahnya.
fn sel_jumlah_tanggal(jumlah: i64, tanggal: &[NaiveDate], warna: &str) -> String {
    if jumlah == 0 {
        return "-".into();
    }
    format!(
        "<strong>{jumlah}</strong><br><span style=\"color:{warna};font-size:8px\">{}</span>",
        laporan::esc(&label_tanggal_pendek(tanggal))
    )
}

const CSS_ROSTER: &str = r#"
table.data th, table.data td { padding: 2px 1px; text-align: center; }
table.data th:nth-child(-n+3) { text-align: left; padding-left: 4px; }
table.data td.nama { text-align: left; padding-left: 4px; white-space: nowrap; }
table.data tr:nth-child(even) td { background: transparent; }
"#;

struct BlokRoster {
    satker_id: Uuid,
    regu: String,
    /// (personel_id, nama, pangkat)
    anggota: Vec<(Uuid, String, String)>,
    per_hari: Vec<HashSet<Uuid>>,
}

/// Ekspor jadwal strong point sebulan menjadi PDF roster (anggota × tanggal).
pub async fn ekspor_jadwal_pdf(
    State(state): State<AppState>,
    user: AuthUser,
    Query(query): Query<BulanQuery>,
) -> AppResult<Response> {
    pastikan_pengelola(&user)?;
    let (dari, sampai) = rentang_bulan(query.bulan.as_deref())?;
    let scope = scope_ekspor(&user, query.satker_id)?;
    let rows = ph::list_jadwal(&state.pool, &scope, dari, sampai).await?;
    let jumlah_hari = sampai.day() as usize;

    let nama_satker_map: HashMap<Uuid, String> = satker::list(&state.pool)
        .await?
        .into_iter()
        .map(|row| (row.id, row.nama))
        .collect();

    let mut daftar: Vec<BlokRoster> = Vec::new();
    let mut posisi: HashMap<(Uuid, String), usize> = HashMap::new();
    for row in &rows {
        let regu = row
            .regu
            .clone()
            .filter(|nilai| !nilai.trim().is_empty())
            .unwrap_or_else(|| "Tanpa Regu".into());
        let kunci = (row.satker_id, regu.clone());
        let pos = if let Some(pos) = posisi.get(&kunci) {
            *pos
        } else {
            daftar.push(BlokRoster {
                satker_id: row.satker_id,
                regu,
                anggota: Vec::new(),
                per_hari: vec![HashSet::new(); jumlah_hari],
            });
            let pos = daftar.len() - 1;
            posisi.insert(kunci, pos);
            pos
        };
        let blok = &mut daftar[pos];
        let hari = row.tanggal.day() as usize - 1;
        for personel in ph::personel_jadwal(&state.pool, row.id).await? {
            if !blok
                .anggota
                .iter()
                .any(|(id, _, _)| *id == personel.personel_id)
            {
                blok.anggota
                    .push((personel.personel_id, personel.nama, personel.pangkat));
            }
            if let Some(set) = blok.per_hari.get_mut(hari) {
                set.insert(personel.personel_id);
            }
        }
    }
    for blok in &mut daftar {
        blok.anggota.sort_by(|a, b| a.1.cmp(&b.1));
    }

    let mut baris = String::new();
    for blok in &daftar {
        if blok.anggota.is_empty() {
            continue;
        }
        let satker_nama = nama_satker_map
            .get(&blok.satker_id)
            .cloned()
            .unwrap_or_else(|| "-".into());
        baris.push_str(&format!(
            "<tr><td colspan=\"{}\" style=\"background:#dbeafe;color:#0c4a6e;font-weight:600;text-align:left\">{} — {} · {} anggota</td></tr>",
            3 + jumlah_hari,
            laporan::esc(&satker_nama),
            laporan::esc(&blok.regu),
            blok.anggota.len()
        ));

        let lengkap: Vec<bool> = blok
            .per_hari
            .iter()
            .map(|set| set.len() == blok.anggota.len())
            .collect();

        for (nomor, (personel_id, nama, pangkat)) in blok.anggota.iter().enumerate() {
            baris.push_str("<tr>");
            baris.push_str(&format!(
                "<td style=\"text-align:center\">{}</td><td class=\"nama\">{}</td><td>{}</td>",
                nomor + 1,
                laporan::esc(nama),
                laporan::esc(pangkat)
            ));
            for i in 0..jumlah_hari {
                if lengkap[i] {
                    if nomor == 0 && (i == 0 || !lengkap[i - 1]) {
                        let panjang = lengkap[i..].iter().take_while(|nilai| **nilai).count();
                        baris.push_str(&format!(
                            "<td rowspan=\"{}\" colspan=\"{panjang}\" style=\"background:#bae6fd;border:1px solid #64748b;text-align:center;vertical-align:middle;font-weight:600;color:#0c4a6e\">{}</td>",
                            blok.anggota.len(),
                            laporan::esc(&blok.regu)
                        ));
                    }
                } else {
                    let dijadwalkan = blok.per_hari[i].contains(personel_id);
                    baris.push_str(&format!(
                        "<td style=\"text-align:center{}\">{}</td>",
                        if dijadwalkan {
                            ";background:#bae6fd;font-weight:600"
                        } else {
                            ""
                        },
                        if dijadwalkan { "•" } else { "" }
                    ));
                }
            }
            baris.push_str("</tr>");
        }
    }

    let judul_tanggal: Vec<String> = (1..=jumlah_hari).map(|hari| hari.to_string()).collect();
    let mut kolom = vec![
        laporan::Kolom {
            judul: "No",
            lebar: 3,
        },
        laporan::Kolom {
            judul: "Anggota",
            lebar: 24,
        },
        laporan::Kolom {
            judul: "Pangkat",
            lebar: 11,
        },
    ];
    for judul in &judul_tanggal {
        kolom.push(laporan::Kolom { judul, lebar: 2 });
    }

    let html = laporan::html_dengan_css(
        &laporan::Laporan {
            judul: "Jadwal Strong Point",
            periode: &label_periode_bulan(dari, sampai),
            satker: &nama_satker_ekspor(&state, query.satker_id).await?,
            personel: "Semua Personel",
            status: "Semua regu",
            dicetak_oleh: &nama_pencetak(&state, &user).await?,
            dicetak_pada: &waktu_cetak(),
            kolom,
            baris,
            jumlah: daftar.len(),
            total: daftar.len() as i64,
            catatan: Some(format!(
                "Blok biru menandai seluruh anggota regu bertugas; titik menandai anggota yang bertugas sendiri. Total {} regu.",
                daftar.len()
            )),
        },
        CSS_ROSTER,
    );
    let pdf = pdf::render_pdf(&html).await?;
    Ok(respons_pdf(
        pdf,
        format!(
            "jadwal-strong-point-{}-{}.pdf",
            dari.format("%Y%m%d"),
            sampai.format("%Y%m%d")
        ),
    ))
}

/// Ekspor rekap kehadiran strong point sebulan menjadi PDF.
pub async fn ekspor_rekap_pdf(
    State(state): State<AppState>,
    user: AuthUser,
    Query(query): Query<BulanQuery>,
) -> AppResult<Response> {
    pastikan_pengelola(&user)?;
    let (dari, sampai_bulan) = rentang_bulan(query.bulan.as_deref())?;
    // Sama seperti tampilan rekap: hitung sampai hari ini saja.
    let sampai = sampai_bulan.min(hari_ini_wita());
    let scope = scope_ekspor(&user, query.satker_id)?;
    let rows = if dari > sampai {
        Vec::new()
    } else {
        ph::rekap_ph(&state.pool, &scope, dari, sampai).await?
    };

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
             </tr>",
            indeks + 1,
            laporan::esc(&row.nama),
            laporan::esc(&row.nrp),
            laporan::esc(row.unit.as_deref().unwrap_or("-")),
            row.jumlah_jadwal,
            row.hadir,
            sel_jumlah_tanggal(row.hanya_check_in, &row.tanggal_hanya_check_in, "#b45309"),
            sel_jumlah_tanggal(row.tidak_hadir, &row.tanggal_tidak_hadir, "#be123c"),
            row.menunggu_verifikasi,
        ));
    }

    let html = laporan::html(&laporan::Laporan {
        judul: "Rekap Strong Point",
        periode: &label_periode_bulan(dari, sampai),
        satker: &nama_satker_ekspor(&state, query.satker_id).await?,
        personel: "Semua Personel",
        status: "Semua jadwal strong point",
        dicetak_oleh: &nama_pencetak(&state, &user).await?,
        dicetak_pada: &waktu_cetak(),
        kolom: vec![
            laporan::Kolom {
                judul: "No",
                lebar: 4,
            },
            laporan::Kolom {
                judul: "Personel",
                lebar: 24,
            },
            laporan::Kolom {
                judul: "Unit",
                lebar: 16,
            },
            laporan::Kolom {
                judul: "Jadwal",
                lebar: 7,
            },
            laporan::Kolom {
                judul: "Check-in",
                lebar: 8,
            },
            laporan::Kolom {
                judul: "Hanya Check-in",
                lebar: 15,
            },
            laporan::Kolom {
                judul: "Tidak Strong Point",
                lebar: 18,
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
            "Perhitungan rekap sampai {} (tanggal setelahnya belum dinilai). Total {} personel.",
            laporan::format_tanggal(sampai),
            rows.len()
        )),
    });
    let pdf = pdf::render_pdf(&html).await?;
    Ok(respons_pdf(
        pdf,
        format!(
            "rekap-strong-point-{}-{}.pdf",
            dari.format("%Y%m%d"),
            sampai.format("%Y%m%d")
        ),
    ))
}
