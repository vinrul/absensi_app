use axum::Json;
use axum::Router;
use axum::extract::{Path, State};
use axum::routing::get;
use chrono::NaiveTime;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::auth::extractor::AuthUser;
use crate::domain::enums::SatkerLevel;
use crate::error::{AppError, AppResult};
use crate::repo::satker::{self, SatkerRow};
use crate::service::akses;
use crate::state::AppState;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/", get(list_satker).post(create_satker))
        .route("/{id}", get(get_satker).patch(update_satker))
}

#[derive(Debug, Serialize)]
pub struct SatkerDto {
    pub id: Uuid,
    pub parent_id: Option<Uuid>,
    pub level: SatkerLevel,
    pub nama: String,
    pub kode: String,
    pub alamat: Option<String>,
    pub timezone: String,
    pub hari_kerja: Vec<i16>,
    pub jam_masuk: String,
    pub jam_pulang: String,
    pub jam_pulang_sabtu: Option<String>,
    pub is_active: bool,
}

impl From<SatkerRow> for SatkerDto {
    fn from(row: SatkerRow) -> Self {
        Self {
            id: row.id,
            parent_id: row.parent_id,
            level: row.level,
            nama: row.nama,
            kode: row.kode,
            alamat: row.alamat,
            timezone: row.timezone,
            hari_kerja: row.hari_kerja,
            jam_masuk: row.jam_masuk.format("%H:%M").to_string(),
            jam_pulang: row.jam_pulang.format("%H:%M").to_string(),
            jam_pulang_sabtu: row
                .jam_pulang_sabtu
                .map(|jam| jam.format("%H:%M").to_string()),
            is_active: row.is_active,
        }
    }
}

fn parse_time(value: &str) -> AppResult<NaiveTime> {
    NaiveTime::parse_from_str(value, "%H:%M")
        .or_else(|_| NaiveTime::parse_from_str(value, "%H:%M:%S"))
        .map_err(|_| AppError::Unprocessable(format!("Format jam tidak valid: {value}")))
}

fn validasi_hari_kerja(hari: &[i16]) -> AppResult<Vec<i16>> {
    if hari.is_empty() {
        return Err(AppError::Unprocessable(
            "Hari kerja minimal satu hari".into(),
        ));
    }
    if hari.iter().any(|h| !(1..=7).contains(h)) {
        return Err(AppError::Unprocessable(
            "Hari kerja harus bernilai 1 (Senin) sampai 7 (Minggu)".into(),
        ));
    }
    let mut bersih = hari.to_vec();
    bersih.sort_unstable();
    bersih.dedup();
    Ok(bersih)
}

fn validasi_timezone(value: &str) -> AppResult<&str> {
    let zona = value.trim();
    if !matches!(zona, "Asia/Jakarta" | "Asia/Makassar" | "Asia/Jayapura") {
        return Err(AppError::Unprocessable(
            "Zona waktu harus Asia/Jakarta, Asia/Makassar, atau Asia/Jayapura".into(),
        ));
    }
    Ok(zona)
}

fn validasi_kode(kode: &str) -> AppResult<&str> {
    let kode = kode.trim();
    if kode.is_empty() {
        return Err(AppError::BadRequest("Kode satker wajib diisi".into()));
    }
    if kode.len() > 40 {
        return Err(AppError::Unprocessable(
            "Kode satker maksimal 40 karakter".into(),
        ));
    }
    Ok(kode)
}

pub async fn list_satker(
    State(state): State<AppState>,
    user: AuthUser,
) -> AppResult<Json<Vec<SatkerDto>>> {
    let rows = satker::list_by_ids(&state.pool, user.satker_akses()).await?;
    Ok(Json(rows.into_iter().map(SatkerDto::from).collect()))
}

pub async fn get_satker(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<Uuid>,
) -> AppResult<Json<SatkerDto>> {
    user.require_satker(id)?;
    let row = satker::find(&state.pool, id)
        .await?
        .ok_or_else(|| AppError::NotFound("Satker tidak ditemukan".into()))?;
    Ok(Json(row.into()))
}

#[derive(Debug, Deserialize)]
pub struct CreateSatkerRequest {
    pub parent_id: Option<Uuid>,
    pub level: SatkerLevel,
    pub nama: String,
    pub kode: String,
    pub alamat: Option<String>,
    pub timezone: Option<String>,
    pub hari_kerja: Option<Vec<i16>>,
    pub jam_masuk: Option<String>,
    pub jam_pulang: Option<String>,
    pub jam_pulang_sabtu: Option<String>,
}

pub async fn create_satker(
    State(state): State<AppState>,
    user: AuthUser,
    Json(req): Json<CreateSatkerRequest>,
) -> AppResult<Json<SatkerDto>> {
    if !user.is_admin_any() {
        return Err(AppError::Forbidden(
            "Hanya admin yang dapat membuat satker".into(),
        ));
    }

    let nama = req.nama.trim();
    if nama.is_empty() {
        return Err(AppError::BadRequest("Nama satker wajib diisi".into()));
    }
    let kode = validasi_kode(&req.kode)?;

    let parent = match req.parent_id {
        Some(parent_id) => Some(
            satker::find(&state.pool, parent_id)
                .await?
                .ok_or_else(|| AppError::Unprocessable("Induk satker tidak ditemukan".into()))?,
        ),
        None => None,
    };
    if !akses::boleh_buat_satker(user.role_scopes(), req.level, parent.as_ref()) {
        return Err(AppError::Forbidden(
            "Anda tidak berhak membuat satker pada induk tersebut".into(),
        ));
    }
    if satker::find_by_kode(&state.pool, kode).await?.is_some() {
        return Err(AppError::Conflict(format!(
            "Kode satker {kode} sudah dipakai"
        )));
    }

    let hari_kerja = validasi_hari_kerja(req.hari_kerja.as_deref().unwrap_or(&[1, 2, 3, 4, 5]))?;
    let jam_masuk = parse_time(req.jam_masuk.as_deref().unwrap_or("08:00"))?;
    let jam_pulang = parse_time(req.jam_pulang.as_deref().unwrap_or("16:00"))?;
    let jam_pulang_sabtu = req
        .jam_pulang_sabtu
        .as_deref()
        .filter(|v| !v.trim().is_empty())
        .map(parse_time)
        .transpose()?;
    if jam_masuk == jam_pulang {
        return Err(AppError::Unprocessable(
            "Jam masuk dan jam pulang tidak boleh sama".into(),
        ));
    }
    if let Some(pulang_sabtu) = jam_pulang_sabtu
        && pulang_sabtu <= jam_masuk
    {
        return Err(AppError::Unprocessable(
            "Jam pulang Sabtu harus setelah jam masuk".into(),
        ));
    }
    let timezone = validasi_timezone(req.timezone.as_deref().unwrap_or("Asia/Makassar"))?;

    let id = Uuid::now_v7();
    satker::create(
        &state.pool,
        id,
        req.parent_id,
        req.level,
        nama,
        kode,
        req.alamat.as_deref(),
        timezone,
        &hari_kerja,
        jam_masuk,
        jam_pulang,
        jam_pulang_sabtu,
    )
    .await?;

    let row = satker::find(&state.pool, id)
        .await?
        .ok_or_else(|| AppError::Internal("satker gagal dibuat".into()))?;
    Ok(Json(row.into()))
}

#[derive(Debug, Deserialize)]
pub struct UpdateSatkerRequest {
    pub nama: Option<String>,
    pub kode: Option<String>,
    pub alamat: Option<String>,
    pub timezone: Option<String>,
    pub hari_kerja: Option<Vec<i16>>,
    pub is_active: Option<bool>,
    pub jam_masuk: Option<String>,
    pub jam_pulang: Option<String>,
    pub jam_pulang_sabtu: Option<String>,
    pub hapus_jam_pulang_sabtu: Option<bool>,
}

pub async fn update_satker(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateSatkerRequest>,
) -> AppResult<Json<SatkerDto>> {
    let existing = satker::find(&state.pool, id)
        .await?
        .ok_or_else(|| AppError::NotFound("Satker tidak ditemukan".into()))?;

    if !user.is_admin_any() || !akses::boleh_kelola_satker(user.role_scopes(), &existing) {
        return Err(AppError::Forbidden(
            "Anda tidak berhak mengubah satker ini".into(),
        ));
    }

    let kode = match req.kode.as_deref() {
        Some(value) => {
            let kode = validasi_kode(value)?;
            if let Some(lain) = satker::find_by_kode(&state.pool, kode).await?
                && lain.id != id
            {
                return Err(AppError::Conflict(format!(
                    "Kode satker {kode} sudah dipakai"
                )));
            }
            Some(kode.to_string())
        }
        None => None,
    };

    let hari_kerja = match req.hari_kerja.as_deref() {
        Some(value) => Some(validasi_hari_kerja(value)?),
        None => None,
    };
    let jam_masuk = req.jam_masuk.as_deref().map(parse_time).transpose()?;
    let jam_pulang = req.jam_pulang.as_deref().map(parse_time).transpose()?;
    let jam_pulang_sabtu = req
        .jam_pulang_sabtu
        .as_deref()
        .filter(|v| !v.trim().is_empty())
        .map(parse_time)
        .transpose()?;
    let masuk_efektif = jam_masuk.unwrap_or(existing.jam_masuk);
    let pulang_efektif = jam_pulang.unwrap_or(existing.jam_pulang);
    if masuk_efektif == pulang_efektif {
        return Err(AppError::Unprocessable(
            "Jam masuk dan jam pulang tidak boleh sama".into(),
        ));
    }
    let hapus_jam_pulang_sabtu = req.hapus_jam_pulang_sabtu.unwrap_or(false);
    let sabtu_efektif = if hapus_jam_pulang_sabtu {
        None
    } else {
        jam_pulang_sabtu.or(existing.jam_pulang_sabtu)
    };
    if let Some(pulang_sabtu) = sabtu_efektif {
        if pulang_sabtu <= masuk_efektif {
            return Err(AppError::Unprocessable(
                "Jam pulang Sabtu harus setelah jam masuk".into(),
            ));
        }
        if pulang_sabtu >= pulang_efektif {
            return Err(AppError::Unprocessable(
                "Jam pulang Sabtu harus sebelum jam pulang hari biasa".into(),
            ));
        }
    }
    let timezone = match req
        .timezone
        .as_deref()
        .map(str::trim)
        .filter(|v| !v.is_empty())
    {
        Some(value) => Some(validasi_timezone(value)?.to_string()),
        None => None,
    };

    satker::update(
        &state.pool,
        id,
        req.nama.as_deref().map(str::trim).filter(|v| !v.is_empty()),
        kode.as_deref(),
        req.alamat.as_deref(),
        timezone.as_deref(),
        hari_kerja.as_deref(),
        req.is_active,
        jam_masuk,
        jam_pulang,
        jam_pulang_sabtu,
        hapus_jam_pulang_sabtu,
    )
    .await?;

    let row = satker::find(&state.pool, id)
        .await?
        .ok_or_else(|| AppError::NotFound("Satker tidak ditemukan".into()))?;
    Ok(Json(row.into()))
}
