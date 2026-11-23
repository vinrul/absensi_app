use axum::Json;
use axum::Router;
use axum::body::Body;
use axum::extract::{Path, Query, State};
use axum::http::{HeaderValue, StatusCode, header};
use axum::response::Response;
use axum::routing::{get, patch, post};
use base64::Engine;
use base64::engine::general_purpose::STANDARD as BASE64;
use chrono::{Duration, NaiveDate, NaiveTime, Timelike, Utc};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use uuid::Uuid;

use crate::auth::extractor::AuthUser;
use crate::domain::enums::{AbsensiJenis, JenisAkun, SatkerLevel, TitikDinasJenis};
use crate::error::{AppError, AppResult};
use crate::repo::personel;
use crate::repo::ph::{
    self, AbsensiPhRow, JadwalPhRow, JadwalPhSayaRow, NewAbsensiPh, PenempatanBaru, StrongPointRow,
};
use crate::repo::satker;
use crate::service::waktu::{
    akhir_bulan, awal_bulan, geser_bulan, hari_ini_wita, ke_wita, parse_bulan,
};
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
        .route("/jadwal/salin", post(salin_jadwal))
        .route("/jadwal/status", get(status_jadwal))
        .route(
            "/jadwal/{id}",
            get(detail_jadwal).put(update_jadwal).delete(delete_jadwal),
        )
        .route("/penempatan", get(list_penempatan).post(simpan_penempatan))
        .route("/absensi", post(create_absensi_ph))
        .route("/absensi/saya", get(absensi_saya))
        .route("/absensi/jadwal/{id}", get(absensi_jadwal))
        .route("/absensi/{id}/verifikasi", post(verifikasi_absensi_ph))
        .route("/rekap", get(rekap_ph))
        .route("/rekap/ekspor", get(ekspor_rekap_pdf))
        .route("/foto", get(list_foto_ph))
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
    pub sesi: Option<String>,
    pub regu: Option<String>,
    pub catatan: Option<String>,
    pub disalin_dari_bulan: Option<NaiveDate>,
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
    pub sesi: Option<String>,
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
            sesi: row.sesi,
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
    pub menit_telat: i32,
    pub menit_pulang_cepat: i32,
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
            foto_url: row.foto_path.map(|key| format!("/api/v1/files/{key}")),
            is_offline: row.is_offline,
            is_mock_suspect: row.is_mock_suspect,
            perlu_verifikasi: row.perlu_verifikasi,
            verified_at: row.verified_at,
            catatan_verifikator: row.catatan_verifikator,
            menit_telat: row.menit_telat,
            menit_pulang_cepat: row.menit_pulang_cepat,
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
    pub total_menit_telat: i64,
    pub total_menit_pulang_cepat: i64,
}

#[derive(Debug, Deserialize)]
pub struct RentangQuery {
    pub dari: Option<NaiveDate>,
    pub sampai: Option<NaiveDate>,
    pub satker_id: Option<Uuid>,
}

/// Rentang laporan default: 30 hari terakhir sampai hari ini. Batas maksimal
/// satu tahun agar laporan/PDF tetap wajar.
fn rentang(
    dari: Option<NaiveDate>,
    sampai: Option<NaiveDate>,
) -> AppResult<(NaiveDate, NaiveDate)> {
    let hari_ini = hari_ini_wita();
    let sampai = sampai.unwrap_or(hari_ini);
    let dari = dari.unwrap_or_else(|| sampai - chrono::Duration::days(29));
    if sampai < dari {
        return Err(AppError::Unprocessable(
            "Tanggal selesai tidak boleh sebelum tanggal mulai".into(),
        ));
    }
    if (sampai - dari).num_days() > 366 {
        return Err(AppError::Unprocessable(
            "Rentang laporan maksimal 366 hari".into(),
        ));
    }
    Ok((dari, sampai))
}

fn parse_time(value: &str) -> AppResult<NaiveTime> {
    NaiveTime::parse_from_str(value, "%H:%M")
        .or_else(|_| NaiveTime::parse_from_str(value, "%H:%M:%S"))
        .map_err(|_| AppError::Unprocessable(format!("Format jam tidak valid: {value}")))
}

fn bulan_query(value: Option<&str>) -> AppResult<NaiveDate> {
    match value {
        Some(label) => parse_bulan(label)
            .ok_or_else(|| AppError::Unprocessable("Format bulan harus YYYY-MM".into())),
        None => Ok(awal_bulan(hari_ini_wita())),
    }
}

/// Materialisasi fallback jadwal bulan sebelumnya untuk bulan yang sudah berjalan.
async fn pastikan_jadwal_rentang(
    state: &AppState,
    scope: &[Uuid],
    dari: NaiveDate,
    sampai: NaiveDate,
) -> AppResult<()> {
    let batas = awal_bulan(hari_ini_wita());
    let akhir = awal_bulan(sampai);
    let mut bulan = awal_bulan(dari);
    while bulan <= akhir {
        ph::pastikan_jadwal_scope(&state.pool, scope, bulan, batas).await?;
        bulan = geser_bulan(bulan, 1);
    }
    Ok(())
}

fn geser_menit(jam: NaiveTime, menit: i64) -> NaiveTime {
    let detik = (jam.num_seconds_from_midnight() as i64 + menit * 60).clamp(0, 86_399) as u32;
    NaiveTime::from_num_seconds_from_midnight_opt(detik, 0).unwrap_or(jam)
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
        sesi: row.sesi,
        regu: row.regu,
        catatan: row.catatan,
        disalin_dari_bulan: row.disalin_dari_bulan,
        jumlah_personel: row.jumlah_personel,
        personel,
    })
}

// ---------------------------------------------------------------- strong point

#[derive(Debug, Deserialize)]
pub struct StrongPointQuery {
    pub satker_id: Option<Uuid>,
}

pub async fn list_strong_point(
    State(state): State<AppState>,
    user: AuthUser,
    Query(query): Query<StrongPointQuery>,
) -> AppResult<Json<Vec<StrongPointDto>>> {
    let scope = if let Some(id) = query.satker_id {
        user.require_satker(id)?;
        vec![id]
    } else {
        user.satker_akses().to_vec()
    };
    let induk = ph::induk_titik_ids(&state.pool, &scope).await?;
    let rows = ph::list_strong_points(&state.pool, &induk).await?;
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
    let owner = satker::find(&state.pool, req.satker_id)
        .await?
        .ok_or_else(|| AppError::Unprocessable("Satker pemilik titik tidak ditemukan".into()))?;
    if owner.level == SatkerLevel::Satker {
        return Err(AppError::Unprocessable(
            "Titik strong point hanya dibuat pada level Polda atau Polres".into(),
        ));
    }
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

// ---------------------------------------------------------- penempatan

#[derive(Debug, Serialize)]
pub struct PenempatanDto {
    pub personel_id: Uuid,
    pub nama: String,
    pub nrp: String,
    pub pangkat: String,
    pub unit: Option<String>,
    pub satker_id: Uuid,
    pub strong_point_id: Option<Uuid>,
    pub strong_point_nama: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct PenempatanQuery {
    pub satker_id: Uuid,
}

pub async fn list_penempatan(
    State(state): State<AppState>,
    user: AuthUser,
    Query(query): Query<PenempatanQuery>,
) -> AppResult<Json<Vec<PenempatanDto>>> {
    pastikan_pengelola(&user)?;
    user.require_satker(query.satker_id)?;
    let rows = ph::list_penempatan_ph(&state.pool, &[query.satker_id]).await?;
    Ok(Json(
        rows.into_iter()
            .map(|row| PenempatanDto {
                personel_id: row.personel_id,
                nama: row.nama,
                nrp: row.nrp,
                pangkat: row.pangkat,
                unit: row.unit,
                satker_id: row.satker_id,
                strong_point_id: row.strong_point_id,
                strong_point_nama: row.strong_point_nama,
            })
            .collect(),
    ))
}

#[derive(Debug, Deserialize)]
pub struct PenempatanItem {
    pub personel_id: Uuid,
    pub strong_point_id: Uuid,
}

#[derive(Debug, Deserialize)]
pub struct PenempatanRequest {
    pub satker_id: Uuid,
    pub items: Vec<PenempatanItem>,
}

pub async fn simpan_penempatan(
    State(state): State<AppState>,
    user: AuthUser,
    Json(req): Json<PenempatanRequest>,
) -> AppResult<Json<serde_json::Value>> {
    pastikan_admin(&user)?;
    pastikan_satker_dikelola(&state, &user, req.satker_id).await?;
    if req.items.is_empty() {
        return Err(AppError::Unprocessable(
            "Pilih minimal satu anggota untuk ditempatkan".into(),
        ));
    }
    if req.items.len() > 1000 {
        return Err(AppError::Unprocessable(
            "Maksimal 1000 anggota per penyimpanan".into(),
        ));
    }

    let induk = ph::induk_titik_ids(&state.pool, &[req.satker_id]).await?;
    let titik = ph::list_strong_points(&state.pool, &induk).await?;
    let peta_titik: HashMap<Uuid, &StrongPointRow> =
        titik.iter().map(|row| (row.id, row)).collect();

    let mut sudah = HashSet::new();
    for item in &req.items {
        if !sudah.insert(item.personel_id) {
            return Err(AppError::Unprocessable(
                "Personel yang sama muncul lebih dari sekali".into(),
            ));
        }
        let personel = personel::find_by_id(&state.pool, item.personel_id)
            .await?
            .ok_or_else(|| AppError::Unprocessable("Personel tidak ditemukan".into()))?;
        if personel.satker_id != req.satker_id {
            return Err(AppError::Unprocessable(
                "Semua anggota harus berasal dari satker yang sama".into(),
            ));
        }
        if personel.jenis_akun != JenisAkun::Personel {
            return Err(AppError::Unprocessable(format!(
                "Akun {} bukan personel dan tidak dapat ditempatkan",
                personel.nama
            )));
        }
        if !personel.is_active {
            return Err(AppError::Unprocessable(format!(
                "Personel {} tidak aktif",
                personel.nama
            )));
        }
        let Some(titik) = peta_titik.get(&item.strong_point_id) else {
            return Err(AppError::Unprocessable(
                "Titik strong point tidak sesuai dengan induk satker".into(),
            ));
        };
        if !titik.is_active {
            return Err(AppError::Unprocessable(format!(
                "Titik {} tidak aktif",
                titik.nama
            )));
        }
    }

    for item in &req.items {
        ph::simpan_penempatan(
            &state.pool,
            &PenempatanBaru {
                personel_id: item.personel_id,
                strong_point_id: item.strong_point_id,
            },
            user.id(),
        )
        .await?;
    }

    Ok(Json(
        serde_json::json!({ "message": "Penempatan disimpan" }),
    ))
}

// ------------------------------------------------------------------ jadwal PH

pub async fn list_jadwal(
    State(state): State<AppState>,
    user: AuthUser,
    Query(query): Query<RentangQuery>,
) -> AppResult<Json<Vec<JadwalDto>>> {
    pastikan_pengelola(&user)?;
    let (dari, sampai) = rentang(query.dari, query.sampai)?;
    let mut scope = user.satker_akses().to_vec();
    if let Some(satker_id) = query.satker_id {
        if !scope.contains(&satker_id) {
            return Err(AppError::Forbidden(
                "Anda tidak berhak melihat satker tersebut".into(),
            ));
        }
        scope = vec![satker_id];
    }
    pastikan_jadwal_rentang(&state, &scope, dari, sampai).await?;
    let rows = ph::list_jadwal(&state.pool, &scope, dari, sampai).await?;
    let mut data = Vec::with_capacity(rows.len());
    for row in rows {
        data.push(jadwal_dto(&state, row).await?);
    }
    Ok(Json(data))
}

#[derive(Debug, Deserialize)]
pub struct SalinJadwalRequest {
    pub satker_id: Uuid,
    /// Bulan tujuan dalam format `YYYY-MM`.
    pub bulan: String,
    pub timpa: Option<bool>,
}

pub async fn salin_jadwal(
    State(state): State<AppState>,
    user: AuthUser,
    Json(req): Json<SalinJadwalRequest>,
) -> AppResult<Json<serde_json::Value>> {
    pastikan_admin(&user)?;
    pastikan_satker_dikelola(&state, &user, req.satker_id).await?;
    let target = bulan_query(Some(&req.bulan))?;
    let (target_awal, target_akhir) = (awal_bulan(target), akhir_bulan(target));
    let sumber = ph::bulan_jadwal_terakhir(&state.pool, req.satker_id, target_awal)
        .await?
        .ok_or_else(|| {
            AppError::Unprocessable("Belum ada jadwal pada bulan sebelumnya untuk disalin".into())
        })?;

    let status = ph::status_jadwal_bulan(&state.pool, &[req.satker_id], target).await?;
    let mut dihapus = 0;
    if status.iter().any(|row| row.jumlah > 0) {
        if !req.timpa.unwrap_or(false) {
            return Err(AppError::Conflict(
                "Bulan tujuan sudah memiliki jadwal. Gunakan timpa=true untuk mengganti jadwal tanpa absensi."
                    .into(),
            ));
        }
        dihapus =
            ph::hapus_jadwal_tanpa_absensi(&state.pool, req.satker_id, target_awal, target_akhir)
                .await?;
        let sisa = ph::status_jadwal_bulan(&state.pool, &[req.satker_id], target).await?;
        if sisa.iter().any(|row| row.jumlah > 0) {
            return Err(AppError::Conflict(
                "Sebagian jadwal bulan tujuan sudah memiliki absensi sehingga tidak dapat diganti"
                    .into(),
            ));
        }
    }

    let dibuat = ph::clone_jadwal_bulan(&state.pool, req.satker_id, sumber, target, false).await?;
    Ok(Json(serde_json::json!({
        "dibuat": dibuat,
        "dihapus": dihapus,
        "sumber": sumber,
        "bulan": target,
    })))
}

#[derive(Debug, Deserialize)]
pub struct StatusJadwalQuery {
    pub satker_id: Option<Uuid>,
    pub bulan: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct StatusJadwalResponse {
    pub bulan: NaiveDate,
    pub bulan_depan: NaiveDate,
    pub jumlah_jadwal: i64,
    pub jumlah_bulan_depan: i64,
    pub perlu_susun_bulan_depan: bool,
    pub disalin_dari_bulan: Option<NaiveDate>,
}

pub async fn status_jadwal(
    State(state): State<AppState>,
    user: AuthUser,
    Query(query): Query<StatusJadwalQuery>,
) -> AppResult<Json<StatusJadwalResponse>> {
    pastikan_pengelola(&user)?;
    let bulan = bulan_query(query.bulan.as_deref())?;
    let mut scope = user.satker_akses().to_vec();
    if let Some(satker_id) = query.satker_id {
        if !scope.contains(&satker_id) {
            return Err(AppError::Forbidden(
                "Anda tidak berhak melihat satker tersebut".into(),
            ));
        }
        scope = vec![satker_id];
    }
    let rows = ph::status_jadwal_bulan(&state.pool, &scope, bulan).await?;
    let jumlah_jadwal = rows.iter().map(|row| row.jumlah).sum();
    let jumlah_bulan_depan = rows.iter().map(|row| row.jumlah_depan).sum();
    let perlu_susun_bulan_depan = rows
        .iter()
        .any(|row| row.punya_personel && row.jumlah_depan == 0);
    let disalin_dari_bulan = rows.iter().filter_map(|row| row.disalin_dari_bulan).min();
    Ok(Json(StatusJadwalResponse {
        bulan: awal_bulan(bulan),
        bulan_depan: geser_bulan(bulan, 1),
        jumlah_jadwal,
        jumlah_bulan_depan,
        perlu_susun_bulan_depan,
        disalin_dari_bulan,
    }))
}

pub async fn jadwal_saya(
    State(state): State<AppState>,
    user: AuthUser,
    Query(query): Query<RentangQuery>,
) -> AppResult<Json<Vec<JadwalSayaDto>>> {
    let (dari, sampai) = rentang(query.dari, query.sampai)?;
    pastikan_jadwal_rentang(&state, user.satker_akses(), dari, sampai).await?;
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
    pub tanggal: NaiveDate,
    /// Bila diisi, satu jadwal dibuat untuk setiap tanggal dalam rentang.
    pub tanggal_selesai: Option<NaiveDate>,
    pub jam_mulai: Option<String>,
    pub jam_selesai: Option<String>,
    /// Sesi jadwal: pagi atau sore.
    pub sesi: Option<String>,
    /// Nama regu untuk tampilan roster (mis. "Regu 1").
    pub regu: Option<String>,
    pub catatan: Option<String>,
    pub personel: Vec<PersonelJadwalInput>,
}

const JAM_MULAI_BAWAAN: &str = "00:00";
const JAM_SELESAI_BAWAAN: &str = "23:59";

/// Mengelompokkan anggota berdasarkan titik penempatan tetapnya. Jadwal tidak
/// lagi memilih titik manual; satu jadwal dibuat per titik yang dipakai anggota.
async fn kelompokkan_titik(
    state: &AppState,
    personel: &[(Uuid, Option<String>)],
) -> AppResult<Vec<(Uuid, Vec<(Uuid, Option<String>)>)>> {
    let ids: Vec<Uuid> = personel.iter().map(|(id, _)| *id).collect();
    let peta: HashMap<Uuid, Uuid> = ph::titik_personel(&state.pool, &ids)
        .await?
        .into_iter()
        .collect();
    if personel.iter().any(|(id, _)| !peta.contains_key(id)) {
        return Err(AppError::Unprocessable(
            "Sebagian anggota belum ditempatkan pada titik strong point. Atur penempatan anggota terlebih dahulu."
                .into(),
        ));
    }

    let mut grup: HashMap<Uuid, Vec<(Uuid, Option<String>)>> = HashMap::new();
    for (id, peran) in personel {
        if let Some(titik) = peta.get(id) {
            grup.entry(*titik).or_default().push((*id, peran.clone()));
        }
    }
    let mut hasil: Vec<_> = grup.into_iter().collect();
    hasil.sort_by_key(|(titik, _)| *titik);
    Ok(hasil)
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
    let sesi = req.sesi.as_deref().unwrap_or("pagi");
    if sesi != "pagi" && sesi != "sore" {
        return Err(AppError::Unprocessable(
            "Sesi harus 'pagi' atau 'sore'".into(),
        ));
    }
    let jam_mulai = parse_time(req.jam_mulai.as_deref().unwrap_or(JAM_MULAI_BAWAAN))?;
    let jam_selesai = parse_time(req.jam_selesai.as_deref().unwrap_or(JAM_SELESAI_BAWAAN))?;
    if jam_selesai <= jam_mulai {
        return Err(AppError::Unprocessable(
            "Jam selesai harus setelah jam mulai".into(),
        ));
    }
    let personel = validasi_personel_jadwal(&state, req.satker_id, &req.personel).await?;
    let grup = kelompokkan_titik(&state, &personel).await?;

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
        for (_, anggota) in &grup {
            let ids: Vec<Uuid> = anggota.iter().map(|(id, _)| *id).collect();
            if ph::ada_bentrok(&state.pool, &ids, tanggal, jam_mulai, jam_selesai, None).await? {
                return Err(AppError::Conflict(format!(
                    "Sebagian personel sudah memiliki jadwal strong point pada {tanggal}"
                )));
            }
        }
        tanggal += chrono::Duration::days(1);
    }

    let mut pertama = None;
    let mut tanggal = req.tanggal;
    while tanggal <= sampai {
        for (titik, anggota) in &grup {
            let id = Uuid::now_v7();
            ph::create_jadwal(
                &state.pool,
                &ph::JadwalBaru {
                    id,
                    satker_id: req.satker_id,
                    strong_point_id: *titik,
                    tanggal,
                    jam_mulai,
                    jam_selesai,
                    sesi,
                    regu: req.regu.as_deref(),
                    catatan: req.catatan.as_deref(),
                    disalin_dari_bulan: None,
                    created_by: user.id(),
                    personel: anggota,
                },
            )
            .await?;
            if pertama.is_none() {
                pertama = Some(id);
            }
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
    let jam_mulai = match req.jam_mulai.as_deref() {
        Some(nilai) => parse_time(nilai)?,
        None => existing.jam_mulai,
    };
    let jam_selesai = match req.jam_selesai.as_deref() {
        Some(nilai) => parse_time(nilai)?,
        None => existing.jam_selesai,
    };
    if jam_selesai <= jam_mulai {
        return Err(AppError::Unprocessable(
            "Jam selesai harus setelah jam mulai".into(),
        ));
    }
    let sesi = req
        .sesi
        .as_deref()
        .or(existing.sesi.as_deref())
        .unwrap_or("pagi");
    if sesi != "pagi" && sesi != "sore" {
        return Err(AppError::Unprocessable(
            "Sesi harus 'pagi' atau 'sore'".into(),
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

    let mut grup = kelompokkan_titik(&state, &personel).await?;
    let posisi = grup
        .iter()
        .position(|(titik, _)| *titik == existing.strong_point_id);
    let utama = match posisi {
        Some(indeks) => indeks,
        None => {
            let absensi = ph::list_absensi_ph_jadwal(&state.pool, id).await?;
            if !absensi.is_empty() {
                return Err(AppError::Unprocessable(
                    "Jadwal ini sudah memiliki absensi pada titik lama. Buat jadwal baru untuk titik lain."
                        .into(),
                ));
            }
            0
        }
    };
    let (titik_utama, personel_utama) = grup.remove(utama);
    ph::update_jadwal(
        &state.pool,
        id,
        titik_utama,
        req.tanggal,
        jam_mulai,
        jam_selesai,
        sesi,
        req.regu.as_deref(),
        req.catatan.as_deref(),
        &personel_utama,
    )
    .await?;

    // Bila anggota yang dipilih memiliki titik berbeda, buat jadwal tambahan
    // untuk titik lain agar satu jadwal tetap satu titik.
    for (titik, anggota) in grup {
        ph::create_jadwal(
            &state.pool,
            &ph::JadwalBaru {
                id: Uuid::now_v7(),
                satker_id: existing.satker_id,
                strong_point_id: titik,
                tanggal: req.tanggal,
                jam_mulai,
                jam_selesai,
                sesi,
                regu: req.regu.as_deref(),
                catatan: req.catatan.as_deref(),
                disalin_dari_bulan: existing.disalin_dari_bulan,
                created_by: user.id(),
                personel: &anggota,
            },
        )
        .await?;
    }
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

    // Kebijakan server: development menandai, produksi menolak lokasi tiruan.
    let is_mock = req.is_mock_suspect.unwrap_or(false);
    if state.settings.mock_location_policy.menolak(is_mock) {
        return Err(AppError::Unprocessable(
            "Lokasi tiruan (mock GPS) terdeteksi. Matikan aplikasi lokasi tiruan/Developer Options lalu coba lagi."
                .into(),
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

    // Satu jadwal = satu sesi (pagi/sore) dan satu titik. Satu anggota hanya
    // boleh check-in sekali dan check-out sekali pada jadwal tersebut.
    let masuk_terakhir = ph::sesi_ph_terakhir(&state.pool, jadwal.id, user.id())
        .await?
        .masuk_terakhir;
    match req.jenis {
        AbsensiJenis::Masuk => {
            if ph::exists_absensi_ph(&state.pool, jadwal.id, user.id(), AbsensiJenis::Masuk).await?
            {
                return Err(AppError::Conflict(
                    "Anda sudah check-in pada sesi strong point ini".into(),
                ));
            }
            let batas_awal = geser_menit(jadwal.jam_mulai, -30);
            if jam < batas_awal {
                return Err(AppError::Unprocessable(format!(
                    "Sesi belum dimulai. Check-in dibuka pukul {}.",
                    batas_awal.format("%H:%M")
                )));
            }
            if jam > jadwal.jam_selesai {
                return Err(AppError::Unprocessable(format!(
                    "Sesi sudah berakhir pukul {}.",
                    jadwal.jam_selesai.format("%H:%M")
                )));
            }
        }
        AbsensiJenis::Pulang => {
            let Some(masuk) = masuk_terakhir else {
                return Err(AppError::Unprocessable(
                    "Check-in strong point belum tercatat".into(),
                ));
            };
            if ph::exists_absensi_ph(&state.pool, jadwal.id, user.id(), AbsensiJenis::Pulang)
                .await?
            {
                return Err(AppError::Conflict(
                    "Anda sudah check-out pada sesi strong point ini".into(),
                ));
            }
            if req.captured_at < masuk {
                return Err(AppError::Unprocessable(
                    "Waktu check-out tidak valid".into(),
                ));
            }
            let batas_akhir = geser_menit(jadwal.jam_selesai, 60);
            if jam > batas_akhir {
                return Err(AppError::Unprocessable(format!(
                    "Batas check-out sesi ini pukul {}.",
                    batas_akhir.format("%H:%M")
                )));
            }
        }
    }

    let menit_telat = if req.jenis == AbsensiJenis::Masuk && jam > jadwal.jam_mulai {
        (jam - jadwal.jam_mulai).num_minutes() as i32
    } else {
        0
    };
    let menit_pulang_cepat = if req.jenis == AbsensiJenis::Pulang && jam < jadwal.jam_selesai {
        (jadwal.jam_selesai - jam).num_minutes() as i32
    } else {
        0
    };

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

    let perlu_verifikasi = req.is_offline.unwrap_or(false) || is_mock || akurasi > 100.0;
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
            is_mock_suspect: is_mock,
            perlu_verifikasi,
            menit_telat,
            menit_pulang_cepat,
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
    Query(query): Query<RentangQuery>,
) -> AppResult<Json<Vec<AbsensiPhDto>>> {
    let (dari, sampai) = rentang(query.dari, query.sampai)?;
    pastikan_jadwal_rentang(&state, user.satker_akses(), dari, sampai).await?;
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
    Query(query): Query<RentangQuery>,
) -> AppResult<Json<serde_json::Value>> {
    pastikan_pengelola(&user)?;
    let (dari, sampai_diminta) = rentang(query.dari, query.sampai)?;
    // Rekap hanya menghitung jadwal sampai hari ini; tanggal yang belum berjalan
    // tidak boleh dianggap "tidak strong point".
    let sampai = sampai_diminta.min(hari_ini_wita());
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
        pastikan_jadwal_rentang(&state, &scope, dari, sampai).await?;
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
            total_menit_telat: row.total_menit_telat,
            total_menit_pulang_cepat: row.total_menit_pulang_cepat,
        })
        .collect();
    Ok(Json(serde_json::json!({
        "rentang": { "dari": dari, "sampai": sampai_diminta },
        "data": data,
    })))
}

#[derive(Debug, Deserialize)]
pub struct FotoPhQuery {
    pub dari: Option<NaiveDate>,
    pub sampai: Option<NaiveDate>,
    pub satker_id: Option<Uuid>,
    pub personel_id: Option<Uuid>,
}

#[derive(Debug, Serialize)]
pub struct FotoPhDto {
    pub id: Uuid,
    pub tanggal: NaiveDate,
    pub jenis: AbsensiJenis,
    pub jam: String,
    pub strong_point_nama: String,
    pub foto_url: String,
}

/// Daftar foto dokumentasi strong point untuk galeri rekap.
pub async fn list_foto_ph(
    State(state): State<AppState>,
    user: AuthUser,
    Query(query): Query<FotoPhQuery>,
) -> AppResult<Json<Vec<FotoPhDto>>> {
    pastikan_pengelola(&user)?;
    let (dari, sampai) = rentang(query.dari, query.sampai)?;
    let scope = scope_ekspor(&user, query.satker_id)?;
    pastikan_jadwal_rentang(&state, &scope, dari, sampai).await?;
    let rows = ph::list_foto_ph(&state.pool, &scope, query.personel_id, dari, sampai).await?;

    Ok(Json(
        rows.into_iter()
            .map(|row| FotoPhDto {
                id: row.id,
                tanggal: row.tanggal,
                jenis: row.jenis,
                jam: row.jam.format("%H:%M").to_string(),
                strong_point_nama: row.strong_point_nama,
                foto_url: format!("/api/v1/files/{}", row.foto_path),
            })
            .collect(),
    ))
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

fn label_rentang(dari: NaiveDate, sampai: NaiveDate) -> String {
    format!(
        "{} s.d. {}",
        laporan::format_tanggal(dari),
        laporan::format_tanggal(sampai)
    )
}

fn label_sesi(sesi: Option<&str>) -> &'static str {
    match sesi {
        Some("sore") => "Sore",
        _ => "Pagi",
    }
}

/// Ekspor daftar jadwal strong point pada rentang tanggal menjadi PDF.
pub async fn ekspor_jadwal_pdf(
    State(state): State<AppState>,
    user: AuthUser,
    Query(query): Query<RentangQuery>,
) -> AppResult<Response> {
    pastikan_pengelola(&user)?;
    let (dari, sampai) = rentang(query.dari, query.sampai)?;
    let scope = scope_ekspor(&user, query.satker_id)?;
    pastikan_jadwal_rentang(&state, &scope, dari, sampai).await?;
    let rows = ph::list_jadwal(&state.pool, &scope, dari, sampai).await?;

    let mut baris = String::new();
    for (indeks, row) in rows.iter().enumerate() {
        let anggota = ph::personel_jadwal(&state.pool, row.id)
            .await?
            .into_iter()
            .map(|p| p.nama)
            .collect::<Vec<_>>()
            .join(", ");
        baris.push_str(&format!(
            "<tr>\
                <td style=\"text-align:center\">{}</td>\
                <td>{}</td>\
                <td style=\"text-align:center\">{}</td>\
                <td style=\"text-align:center\">{} – {}</td>\
                <td>{}</td>\
                <td>{}</td>\
                <td>{}</td>\
             </tr>",
            indeks + 1,
            laporan::format_tanggal(row.tanggal),
            label_sesi(row.sesi.as_deref()),
            row.jam_mulai.format("%H:%M"),
            row.jam_selesai.format("%H:%M"),
            laporan::esc(&row.strong_point_nama),
            laporan::esc(row.regu.as_deref().unwrap_or("-")),
            laporan::esc(&anggota),
        ));
    }

    let html = laporan::html(&laporan::Laporan {
        judul: "Jadwal Strong Point",
        periode: &label_rentang(dari, sampai),
        satker: &nama_satker_ekspor(&state, query.satker_id).await?,
        personel: "Semua Personel",
        status: "Semua sesi",
        dicetak_oleh: &nama_pencetak(&state, &user).await?,
        dicetak_pada: &waktu_cetak(),
        kolom: vec![
            laporan::Kolom {
                judul: "No",
                lebar: 4,
            },
            laporan::Kolom {
                judul: "Tanggal",
                lebar: 12,
            },
            laporan::Kolom {
                judul: "Sesi",
                lebar: 6,
            },
            laporan::Kolom {
                judul: "Jam",
                lebar: 10,
            },
            laporan::Kolom {
                judul: "Titik",
                lebar: 20,
            },
            laporan::Kolom {
                judul: "Regu",
                lebar: 10,
            },
            laporan::Kolom {
                judul: "Anggota",
                lebar: 38,
            },
        ],
        baris,
        jumlah: rows.len(),
        total: rows.len() as i64,
        catatan: Some(format!(
            "Rentang {} s.d. {} · {} jadwal.",
            laporan::format_tanggal(dari),
            laporan::format_tanggal(sampai),
            rows.len()
        )),
    });
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

/// Ekspor rekap kehadiran strong point pada rentang tanggal menjadi PDF.
pub async fn ekspor_rekap_pdf(
    State(state): State<AppState>,
    user: AuthUser,
    Query(query): Query<RentangQuery>,
) -> AppResult<Response> {
    pastikan_pengelola(&user)?;
    let (dari, sampai_diminta) = rentang(query.dari, query.sampai)?;
    // Sama seperti tampilan rekap: hitung sampai hari ini saja.
    let sampai = sampai_diminta.min(hari_ini_wita());
    let scope = scope_ekspor(&user, query.satker_id)?;
    let rows = if dari > sampai {
        Vec::new()
    } else {
        pastikan_jadwal_rentang(&state, &scope, dari, sampai).await?;
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
            row.total_menit_telat,
            row.total_menit_pulang_cepat,
            row.menunggu_verifikasi,
        ));
    }

    let html = laporan::html(&laporan::Laporan {
        judul: "Rekap Strong Point",
        periode: &label_rentang(dari, sampai_diminta),
        satker: &nama_satker_ekspor(&state, query.satker_id).await?,
        personel: "Semua Personel",
        status: "Semua jadwal strong point",
        dicetak_oleh: &nama_pencetak(&state, &user).await?,
        dicetak_pada: &waktu_cetak(),
        kolom: vec![
            laporan::Kolom {
                judul: "No",
                lebar: 3,
            },
            laporan::Kolom {
                judul: "Personel",
                lebar: 20,
            },
            laporan::Kolom {
                judul: "Unit",
                lebar: 13,
            },
            laporan::Kolom {
                judul: "Jadwal",
                lebar: 6,
            },
            laporan::Kolom {
                judul: "Check-in",
                lebar: 7,
            },
            laporan::Kolom {
                judul: "Hanya Check-in",
                lebar: 13,
            },
            laporan::Kolom {
                judul: "Tidak Strong Point",
                lebar: 16,
            },
            laporan::Kolom {
                judul: "Telat (menit)",
                lebar: 8,
            },
            laporan::Kolom {
                judul: "Pulang Cepat (menit)",
                lebar: 10,
            },
            laporan::Kolom {
                judul: "Menunggu",
                lebar: 7,
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
