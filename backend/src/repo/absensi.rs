use chrono::{DateTime, NaiveDate, NaiveTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use crate::domain::enums::{AbsensiJenis, StatusKehadiran};
use crate::error::{AppError, AppResult};

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct AbsensiRow {
    pub id: Uuid,
    pub client_uuid: Uuid,
    pub personel_id: Uuid,
    pub satker_id: Uuid,
    pub tanggal: NaiveDate,
    pub jenis: AbsensiJenis,
    pub status_kehadiran: StatusKehadiran,
    pub captured_at: DateTime<Utc>,
    pub jam: NaiveTime,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    pub akurasi_m: Option<f64>,
    pub jarak_m: Option<f64>,
    pub dalam_radius: bool,
    pub is_mock_suspect: bool,
    pub is_offline: bool,
    pub foto_path: Option<String>,
    pub perlu_verifikasi: bool,
    pub verified_at: Option<DateTime<Utc>>,
    pub catatan_verifikator: Option<String>,
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct AbsensiListItem {
    pub id: Uuid,
    pub personel_id: Uuid,
    pub personel_nama: String,
    pub personel_nrp: String,
    pub personel_pangkat: String,
    pub unit: Option<String>,
    pub tanggal: NaiveDate,
    pub jenis: AbsensiJenis,
    pub status_kehadiran: StatusKehadiran,
    pub jam: NaiveTime,
    pub jarak_m: Option<f64>,
    pub dalam_radius: bool,
    pub is_offline: bool,
    pub is_mock_suspect: bool,
    pub perlu_verifikasi: bool,
    pub verified_at: Option<DateTime<Utc>>,
    pub catatan_verifikator: Option<String>,
    pub foto_path: Option<String>,
}

pub struct NewAbsensi<'a> {
    pub id: Uuid,
    pub client_uuid: Uuid,
    pub personel_id: Uuid,
    pub satker_id: Uuid,
    pub tanggal: NaiveDate,
    pub jenis: AbsensiJenis,
    pub status_kehadiran: StatusKehadiran,
    pub captured_at: DateTime<Utc>,
    pub jam: NaiveTime,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    pub akurasi_m: Option<f64>,
    pub jarak_m: Option<f64>,
    pub dalam_radius: bool,
    pub is_mock_suspect: bool,
    pub is_offline: bool,
    pub foto_path: Option<&'a str>,
    pub device_id: Option<&'a str>,
    pub perlu_verifikasi: bool,
}

/// Idempotent terhadap `client_uuid` (antrean offline boleh mengirim ulang).
pub async fn insert(pool: &PgPool, data: &NewAbsensi<'_>) -> AppResult<u16> {
    const SQL: &str = r#"
        INSERT INTO absensi (
            id, client_uuid, personel_id, satker_id, tanggal, jenis, status_kehadiran,
            captured_at, jam, latitude, longitude, akurasi_m, jarak_m, dalam_radius,
            is_mock_suspect, is_offline, foto_path, device_id, perlu_verifikasi
        )
        VALUES (
            $1, $2, $3, $4, $5, $6, $7,
            $8, $9, $10::float8::numeric, $11::float8::numeric,
            $12::float8::numeric, $13::float8::numeric, $14,
            $15, $16, $17, $18, $19
        )
        ON CONFLICT (client_uuid) DO NOTHING
    "#;

    let result = sqlx::query(SQL)
        .bind(data.id)
        .bind(data.client_uuid)
        .bind(data.personel_id)
        .bind(data.satker_id)
        .bind(data.tanggal)
        .bind(data.jenis)
        .bind(data.status_kehadiran)
        .bind(data.captured_at)
        .bind(data.jam)
        .bind(data.latitude)
        .bind(data.longitude)
        .bind(data.akurasi_m)
        .bind(data.jarak_m)
        .bind(data.dalam_radius)
        .bind(data.is_mock_suspect)
        .bind(data.is_offline)
        .bind(data.foto_path)
        .bind(data.device_id)
        .bind(data.perlu_verifikasi)
        .execute(pool)
        .await?;

    Ok(result.rows_affected() as u16)
}

pub async fn find_by_client_uuid(
    pool: &PgPool,
    client_uuid: Uuid,
) -> AppResult<Option<AbsensiRow>> {
    const SQL: &str = r#"
        SELECT id, client_uuid, personel_id, satker_id, tanggal, jenis, status_kehadiran,
               captured_at, jam,
               latitude::float8  AS latitude,
               longitude::float8 AS longitude,
               akurasi_m::float8 AS akurasi_m,
               jarak_m::float8   AS jarak_m,
               dalam_radius, is_mock_suspect, is_offline, foto_path,
               perlu_verifikasi, verified_at, catatan_verifikator
          FROM absensi
         WHERE client_uuid = $1
    "#;
    Ok(sqlx::query_as::<_, AbsensiRow>(SQL)
        .bind(client_uuid)
        .fetch_optional(pool)
        .await?)
}

pub async fn find(pool: &PgPool, id: Uuid) -> AppResult<Option<AbsensiRow>> {
    const SQL: &str = r#"
        SELECT a.id, a.client_uuid, a.personel_id, a.satker_id, a.tanggal, a.jenis,
               a.status_kehadiran, a.captured_at, a.jam,
               a.latitude::float8  AS latitude,
               a.longitude::float8 AS longitude,
               a.akurasi_m::float8 AS akurasi_m,
               a.jarak_m::float8   AS jarak_m,
               a.dalam_radius, a.is_mock_suspect, a.is_offline, a.foto_path,
               a.perlu_verifikasi, a.verified_at, a.catatan_verifikator
          FROM absensi a
         WHERE a.id = $1
    "#;
    Ok(sqlx::query_as::<_, AbsensiRow>(SQL)
        .bind(id)
        .fetch_optional(pool)
        .await?)
}

pub async fn exists_harian(
    pool: &PgPool,
    personel_id: Uuid,
    tanggal: NaiveDate,
    jenis: AbsensiJenis,
) -> AppResult<bool> {
    let found = sqlx::query_scalar::<_, i64>(
        "SELECT 1::bigint FROM absensi WHERE personel_id = $1 AND tanggal = $2 AND jenis = $3",
    )
    .bind(personel_id)
    .bind(tanggal)
    .bind(jenis)
    .fetch_optional(pool)
    .await?;
    Ok(found.is_some())
}

pub async fn list_personel(
    pool: &PgPool,
    personel_id: Uuid,
    dari: NaiveDate,
    sampai: NaiveDate,
) -> AppResult<Vec<AbsensiRow>> {
    const SQL: &str = r#"
        SELECT a.id, a.client_uuid, a.personel_id, a.satker_id, a.tanggal, a.jenis,
               a.status_kehadiran, a.captured_at, a.jam,
               a.latitude::float8  AS latitude,
               a.longitude::float8 AS longitude,
               a.akurasi_m::float8 AS akurasi_m,
               a.jarak_m::float8   AS jarak_m,
               a.dalam_radius, a.is_mock_suspect, a.is_offline, a.foto_path,
               a.perlu_verifikasi, a.verified_at, a.catatan_verifikator
          FROM absensi a
         WHERE a.personel_id = $1 AND a.tanggal BETWEEN $2 AND $3
         ORDER BY a.tanggal DESC, a.jenis
    "#;
    Ok(sqlx::query_as::<_, AbsensiRow>(SQL)
        .bind(personel_id)
        .bind(dari)
        .bind(sampai)
        .fetch_all(pool)
        .await?)
}

// ------------------------------------------------------------------- listing

const LIST_SELECT: &str = r#"
    SELECT a.id, a.personel_id, p.nama AS personel_nama, p.nrp AS personel_nrp,
           p.pangkat AS personel_pangkat, p.unit,
           a.tanggal, a.jenis, a.status_kehadiran, a.jam,
           a.jarak_m::float8 AS jarak_m, a.dalam_radius, a.is_offline,
           a.is_mock_suspect, a.perlu_verifikasi, a.verified_at,
           a.catatan_verifikator, a.foto_path
      FROM absensi a
      JOIN personel p ON p.id = a.personel_id
     WHERE a.satker_id = ANY($1::uuid[])
       AND ($2::date IS NULL OR a.tanggal = $2)
       AND ($3::bool IS NULL OR a.perlu_verifikasi = $3)
       AND ($4::uuid IS NULL OR a.personel_id = $4)
     ORDER BY a.tanggal DESC, a.jam DESC
     LIMIT $5 OFFSET $6
"#;

const LIST_COUNT: &str = r#"
    SELECT count(*)
      FROM absensi a
     WHERE a.satker_id = ANY($1::uuid[])
       AND ($2::date IS NULL OR a.tanggal = $2)
       AND ($3::bool IS NULL OR a.perlu_verifikasi = $3)
       AND ($4::uuid IS NULL OR a.personel_id = $4)
"#;

pub async fn list(
    pool: &PgPool,
    satker_ids: &[Uuid],
    tanggal: Option<NaiveDate>,
    perlu_verifikasi: Option<bool>,
    personel_id: Option<Uuid>,
    limit: i64,
    offset: i64,
) -> AppResult<(Vec<AbsensiListItem>, i64)> {
    let rows = sqlx::query_as::<_, AbsensiListItem>(LIST_SELECT)
        .bind(satker_ids)
        .bind(tanggal)
        .bind(perlu_verifikasi)
        .bind(personel_id)
        .bind(limit)
        .bind(offset)
        .fetch_all(pool)
        .await?;

    let total: i64 = sqlx::query_scalar::<_, i64>(LIST_COUNT)
        .bind(satker_ids)
        .bind(tanggal)
        .bind(perlu_verifikasi)
        .bind(personel_id)
        .fetch_one(pool)
        .await?;

    Ok((rows, total))
}

// --------------------------------------------------------------- rekap & verif

#[derive(Debug, Clone, serde::Serialize, sqlx::FromRow)]
pub struct RekapRow {
    pub personel_id: Uuid,
    pub nama: String,
    pub nrp: String,
    pub pangkat: String,
    pub jabatan: String,
    pub unit: Option<String>,
    pub satker_id: Uuid,
    pub hadir: i64,
    pub dinas_luar: i64,
    pub piket: i64,
    pub telat: i64,
    pub cuti: i64,
    pub izin: i64,
    pub sakit: i64,
    pub libur_nasional: i64,
    pub tanpa_keterangan: i64,
    pub menunggu_verifikasi: i64,
}

const REKAP_SQL: &str = r#"
    SELECT p.id AS personel_id, p.nama, p.nrp, p.pangkat, p.jabatan, p.unit, p.satker_id,
           count(a.id) FILTER (
               WHERE a.status_kehadiran IN ('dinas_kantor', 'dinas_luar', 'piket')
           ) AS hadir,
           count(a.id) FILTER (WHERE a.status_kehadiran = 'dinas_luar') AS dinas_luar,
           count(a.id) FILTER (WHERE a.status_kehadiran = 'piket') AS piket,
           count(a.id) FILTER (
               WHERE a.status_kehadiran IN ('dinas_kantor', 'piket') AND a.jam > s.jam_masuk
           ) AS telat,
           count(a.id) FILTER (WHERE a.status_kehadiran = 'cuti') AS cuti,
           count(a.id) FILTER (WHERE a.status_kehadiran = 'izin') AS izin,
           count(a.id) FILTER (WHERE a.status_kehadiran = 'sakit') AS sakit,
           count(a.id) FILTER (WHERE a.status_kehadiran = 'libur_nasional') AS libur_nasional,
           count(a.id) FILTER (WHERE a.status_kehadiran = 'tanpa_keterangan') AS tanpa_keterangan,
           count(a.id) FILTER (WHERE a.perlu_verifikasi AND a.verified_at IS NULL)
               AS menunggu_verifikasi
      FROM personel p
      JOIN satker s ON s.id = p.satker_id
      LEFT JOIN absensi a
             ON a.personel_id = p.id
            AND a.jenis = 'masuk'
            AND a.tanggal BETWEEN $2 AND $3
     WHERE p.deleted_at IS NULL
       AND p.jenis_akun = 'personel'
       AND p.satker_id = ANY($1::uuid[])
       AND ($4::text IS NULL OR p.unit = $4)
     GROUP BY p.id, p.nama, p.nrp, p.pangkat, p.jabatan, p.unit, p.satker_id, s.jam_masuk
     ORDER BY p.nama
"#;

pub async fn rekap(
    pool: &PgPool,
    satker_ids: &[Uuid],
    dari: NaiveDate,
    sampai: NaiveDate,
    unit: Option<&str>,
) -> AppResult<Vec<RekapRow>> {
    Ok(sqlx::query_as::<_, RekapRow>(REKAP_SQL)
        .bind(satker_ids)
        .bind(dari)
        .bind(sampai)
        .bind(unit)
        .fetch_all(pool)
        .await?)
}

pub async fn verifikasi(
    pool: &PgPool,
    id: Uuid,
    verified_by: Uuid,
    catatan: Option<&str>,
) -> AppResult<()> {
    const SQL: &str = r#"
        UPDATE absensi
           SET perlu_verifikasi = false,
               verified_by = $2,
               verified_at = now(),
               catatan_verifikator = $3
         WHERE id = $1
    "#;
    let result = sqlx::query(SQL)
        .bind(id)
        .bind(verified_by)
        .bind(catatan)
        .execute(pool)
        .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound("Data absensi tidak ditemukan".into()));
    }
    Ok(())
}
