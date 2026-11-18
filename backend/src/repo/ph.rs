//! Repositori PH / Strong Point.
//!
//! Modul ini sengaja terpisah dari tabel `absensi` agar jadwal penjagaan
//! tidak mengubah arti status kehadiran harian.

use chrono::{DateTime, NaiveDate, NaiveTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use crate::domain::enums::AbsensiJenis;
use crate::error::{AppError, AppResult};

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct StrongPointRow {
    pub id: Uuid,
    pub satker_id: Uuid,
    pub nama: String,
    pub alamat: Option<String>,
    pub latitude: f64,
    pub longitude: f64,
    pub radius_m: i32,
    pub is_active: bool,
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct JadwalPhRow {
    pub id: Uuid,
    pub satker_id: Uuid,
    pub strong_point_id: Uuid,
    pub strong_point_nama: String,
    pub tanggal: NaiveDate,
    pub jam_mulai: NaiveTime,
    pub jam_selesai: NaiveTime,
    pub regu: Option<String>,
    pub catatan: Option<String>,
    pub jumlah_personel: i64,
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct JadwalPhSayaRow {
    pub id: Uuid,
    pub satker_id: Uuid,
    pub strong_point_id: Uuid,
    pub strong_point_nama: String,
    pub latitude: f64,
    pub longitude: f64,
    pub radius_m: i32,
    pub tanggal: NaiveDate,
    pub jam_mulai: NaiveTime,
    pub jam_selesai: NaiveTime,
    pub regu: Option<String>,
    pub catatan: Option<String>,
    pub peran: Option<String>,
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct PersonelJadwalRow {
    pub personel_id: Uuid,
    pub nama: String,
    pub nrp: String,
    pub pangkat: String,
    pub unit: Option<String>,
    pub peran: Option<String>,
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct AbsensiPhRow {
    pub id: Uuid,
    pub client_uuid: Uuid,
    pub jadwal_ph_id: Uuid,
    pub personel_id: Uuid,
    pub personel_nama: String,
    pub personel_nrp: String,
    pub strong_point_id: Uuid,
    pub strong_point_nama: String,
    pub satker_id: Uuid,
    pub jenis: AbsensiJenis,
    pub captured_at: DateTime<Utc>,
    pub jam: NaiveTime,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    pub akurasi_m: Option<f64>,
    pub jarak_m: Option<f64>,
    pub dalam_radius: bool,
    pub foto_path: Option<String>,
    pub is_offline: bool,
    pub is_mock_suspect: bool,
    pub perlu_verifikasi: bool,
    pub verified_at: Option<DateTime<Utc>>,
    pub catatan_verifikator: Option<String>,
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct SesiPhRow {
    pub masuk_terakhir: Option<DateTime<Utc>>,
    pub pulang_terakhir: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct RekapPhRow {
    pub personel_id: Uuid,
    pub nama: String,
    pub nrp: String,
    pub pangkat: String,
    pub unit: Option<String>,
    pub jumlah_jadwal: i64,
    pub hadir: i64,
    /// Jadwal yang check-in tetapi tidak check-out.
    pub hanya_check_in: i64,
    pub tidak_hadir: i64,
    /// Tanggal jadwal yang tidak memiliki check-in sama sekali.
    pub tanggal_tidak_hadir: Vec<NaiveDate>,
    /// Tanggal jadwal yang hanya memiliki check-in.
    pub tanggal_hanya_check_in: Vec<NaiveDate>,
    pub menunggu_verifikasi: i64,
}

pub struct NewAbsensiPh {
    pub id: Uuid,
    pub client_uuid: Uuid,
    pub jadwal_ph_id: Uuid,
    pub personel_id: Uuid,
    pub strong_point_id: Uuid,
    pub satker_id: Uuid,
    pub jenis: AbsensiJenis,
    pub captured_at: DateTime<Utc>,
    pub jam: NaiveTime,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    pub akurasi_m: Option<f64>,
    pub jarak_m: Option<f64>,
    pub dalam_radius: bool,
    pub foto_path: Option<String>,
    pub is_offline: bool,
    pub is_mock_suspect: bool,
    pub perlu_verifikasi: bool,
}

// ---------------------------------------------------------------- strong point

pub async fn list_strong_points(
    pool: &PgPool,
    satker_ids: &[Uuid],
) -> AppResult<Vec<StrongPointRow>> {
    const SQL: &str = r#"
        SELECT id, satker_id, nama, alamat, latitude::float8 AS latitude,
               longitude::float8 AS longitude, radius_m, is_active
          FROM strong_point
         WHERE satker_id = ANY($1::uuid[]) AND deleted_at IS NULL
         ORDER BY nama
    "#;
    Ok(sqlx::query_as::<_, StrongPointRow>(SQL)
        .bind(satker_ids)
        .fetch_all(pool)
        .await?)
}

pub async fn find_strong_point(pool: &PgPool, id: Uuid) -> AppResult<Option<StrongPointRow>> {
    const SQL: &str = r#"
        SELECT id, satker_id, nama, alamat, latitude::float8 AS latitude,
               longitude::float8 AS longitude, radius_m, is_active
          FROM strong_point
         WHERE id = $1 AND deleted_at IS NULL
    "#;
    Ok(sqlx::query_as::<_, StrongPointRow>(SQL)
        .bind(id)
        .fetch_optional(pool)
        .await?)
}

#[allow(clippy::too_many_arguments)]
pub async fn create_strong_point(
    pool: &PgPool,
    id: Uuid,
    satker_id: Uuid,
    nama: &str,
    alamat: Option<&str>,
    latitude: f64,
    longitude: f64,
    radius_m: i32,
) -> AppResult<()> {
    sqlx::query(
        "INSERT INTO strong_point (id, satker_id, nama, alamat, latitude, longitude, radius_m)
         VALUES ($1, $2, $3, $4, $5::float8::numeric, $6::float8::numeric, $7)",
    )
    .bind(id)
    .bind(satker_id)
    .bind(nama)
    .bind(alamat)
    .bind(latitude)
    .bind(longitude)
    .bind(radius_m)
    .execute(pool)
    .await?;
    Ok(())
}

#[allow(clippy::too_many_arguments)]
pub async fn update_strong_point(
    pool: &PgPool,
    id: Uuid,
    nama: Option<&str>,
    alamat: Option<&str>,
    latitude: Option<f64>,
    longitude: Option<f64>,
    radius_m: Option<i32>,
    is_active: Option<bool>,
) -> AppResult<()> {
    let result = sqlx::query(
        "UPDATE strong_point
            SET nama = COALESCE($2, nama),
                alamat = COALESCE($3, alamat),
                latitude = COALESCE($4::float8::numeric, latitude),
                longitude = COALESCE($5::float8::numeric, longitude),
                radius_m = COALESCE($6, radius_m),
                is_active = COALESCE($7, is_active)
          WHERE id = $1 AND deleted_at IS NULL",
    )
    .bind(id)
    .bind(nama)
    .bind(alamat)
    .bind(latitude)
    .bind(longitude)
    .bind(radius_m)
    .bind(is_active)
    .execute(pool)
    .await?;
    if result.rows_affected() == 0 {
        return Err(AppError::NotFound("Strong point tidak ditemukan".into()));
    }
    Ok(())
}

pub async fn delete_strong_point(pool: &PgPool, id: Uuid) -> AppResult<()> {
    let result = sqlx::query(
        "UPDATE strong_point SET deleted_at = now()
          WHERE id = $1 AND deleted_at IS NULL",
    )
    .bind(id)
    .execute(pool)
    .await?;
    if result.rows_affected() == 0 {
        return Err(AppError::NotFound("Strong point tidak ditemukan".into()));
    }
    Ok(())
}

// ------------------------------------------------------------------ jadwal PH

pub async fn list_jadwal(
    pool: &PgPool,
    satker_ids: &[Uuid],
    dari: NaiveDate,
    sampai: NaiveDate,
) -> AppResult<Vec<JadwalPhRow>> {
    const SQL: &str = r#"
        SELECT j.id, j.satker_id, j.strong_point_id, sp.nama AS strong_point_nama,
               j.tanggal, j.jam_mulai, j.jam_selesai, j.regu, j.catatan,
               (SELECT count(*) FROM jadwal_ph_personel x WHERE x.jadwal_ph_id = j.id)
                   AS jumlah_personel
          FROM jadwal_ph j
          JOIN strong_point sp ON sp.id = j.strong_point_id
         WHERE j.satker_id = ANY($1::uuid[]) AND j.tanggal BETWEEN $2 AND $3
         ORDER BY j.tanggal, j.regu, j.jam_mulai
    "#;
    Ok(sqlx::query_as::<_, JadwalPhRow>(SQL)
        .bind(satker_ids)
        .bind(dari)
        .bind(sampai)
        .fetch_all(pool)
        .await?)
}

pub async fn find_jadwal(pool: &PgPool, id: Uuid) -> AppResult<Option<JadwalPhRow>> {
    const SQL: &str = r#"
        SELECT j.id, j.satker_id, j.strong_point_id, sp.nama AS strong_point_nama,
               j.tanggal, j.jam_mulai, j.jam_selesai, j.regu, j.catatan,
               (SELECT count(*) FROM jadwal_ph_personel x WHERE x.jadwal_ph_id = j.id)
                   AS jumlah_personel
          FROM jadwal_ph j
          JOIN strong_point sp ON sp.id = j.strong_point_id
         WHERE j.id = $1
    "#;
    Ok(sqlx::query_as::<_, JadwalPhRow>(SQL)
        .bind(id)
        .fetch_optional(pool)
        .await?)
}

pub async fn jadwal_saya(
    pool: &PgPool,
    personel_id: Uuid,
    dari: NaiveDate,
    sampai: NaiveDate,
) -> AppResult<Vec<JadwalPhSayaRow>> {
    const SQL: &str = r#"
        SELECT j.id, j.satker_id, j.strong_point_id, sp.nama AS strong_point_nama,
               sp.latitude::float8 AS latitude, sp.longitude::float8 AS longitude,
               sp.radius_m, j.tanggal, j.jam_mulai, j.jam_selesai, j.regu, j.catatan, jp.peran
          FROM jadwal_ph j
          JOIN jadwal_ph_personel jp ON jp.jadwal_ph_id = j.id
          JOIN strong_point sp ON sp.id = j.strong_point_id
         WHERE jp.personel_id = $1 AND j.tanggal BETWEEN $2 AND $3
         ORDER BY j.tanggal, j.jam_mulai
    "#;
    Ok(sqlx::query_as::<_, JadwalPhSayaRow>(SQL)
        .bind(personel_id)
        .bind(dari)
        .bind(sampai)
        .fetch_all(pool)
        .await?)
}

pub async fn personel_jadwal(pool: &PgPool, jadwal_id: Uuid) -> AppResult<Vec<PersonelJadwalRow>> {
    const SQL: &str = r#"
        SELECT jp.personel_id, p.nama, p.nrp, p.pangkat, p.unit, jp.peran
          FROM jadwal_ph_personel jp
          JOIN personel p ON p.id = jp.personel_id
         WHERE jp.jadwal_ph_id = $1
         ORDER BY p.nama
    "#;
    Ok(sqlx::query_as::<_, PersonelJadwalRow>(SQL)
        .bind(jadwal_id)
        .fetch_all(pool)
        .await?)
}

pub struct JadwalBaru<'a> {
    pub id: Uuid,
    pub satker_id: Uuid,
    pub strong_point_id: Uuid,
    pub tanggal: NaiveDate,
    pub jam_mulai: NaiveTime,
    pub jam_selesai: NaiveTime,
    pub regu: Option<&'a str>,
    pub catatan: Option<&'a str>,
    pub created_by: Uuid,
    pub personel: &'a [(Uuid, Option<String>)],
}

pub async fn create_jadwal(pool: &PgPool, data: &JadwalBaru<'_>) -> AppResult<()> {
    let mut tx = pool.begin().await?;
    sqlx::query(
        "INSERT INTO jadwal_ph (id, satker_id, strong_point_id, tanggal, jam_mulai,
                                jam_selesai, regu, catatan, created_by)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)",
    )
    .bind(data.id)
    .bind(data.satker_id)
    .bind(data.strong_point_id)
    .bind(data.tanggal)
    .bind(data.jam_mulai)
    .bind(data.jam_selesai)
    .bind(data.regu)
    .bind(data.catatan)
    .bind(data.created_by)
    .execute(&mut *tx)
    .await?;

    for (personel_id, peran) in data.personel {
        sqlx::query(
            "INSERT INTO jadwal_ph_personel (jadwal_ph_id, personel_id, peran)
             VALUES ($1, $2, $3)",
        )
        .bind(data.id)
        .bind(personel_id)
        .bind(peran)
        .execute(&mut *tx)
        .await?;
    }
    tx.commit().await?;
    Ok(())
}

#[allow(clippy::too_many_arguments)]
pub async fn update_jadwal(
    pool: &PgPool,
    id: Uuid,
    strong_point_id: Uuid,
    tanggal: NaiveDate,
    jam_mulai: NaiveTime,
    jam_selesai: NaiveTime,
    regu: Option<&str>,
    catatan: Option<&str>,
    personel: &[(Uuid, Option<String>)],
) -> AppResult<()> {
    let mut tx = pool.begin().await?;
    let result = sqlx::query(
        "UPDATE jadwal_ph
            SET strong_point_id = $2, tanggal = $3, jam_mulai = $4,
                jam_selesai = $5, regu = $6, catatan = $7
          WHERE id = $1",
    )
    .bind(id)
    .bind(strong_point_id)
    .bind(tanggal)
    .bind(jam_mulai)
    .bind(jam_selesai)
    .bind(regu)
    .bind(catatan)
    .execute(&mut *tx)
    .await?;
    if result.rows_affected() == 0 {
        return Err(AppError::NotFound("Jadwal PH tidak ditemukan".into()));
    }

    sqlx::query("DELETE FROM jadwal_ph_personel WHERE jadwal_ph_id = $1")
        .bind(id)
        .execute(&mut *tx)
        .await?;
    for (personel_id, peran) in personel {
        sqlx::query(
            "INSERT INTO jadwal_ph_personel (jadwal_ph_id, personel_id, peran)
             VALUES ($1, $2, $3)",
        )
        .bind(id)
        .bind(personel_id)
        .bind(peran.as_deref())
        .execute(&mut *tx)
        .await?;
    }

    tx.commit().await?;
    Ok(())
}

pub async fn delete_jadwal(pool: &PgPool, id: Uuid) -> AppResult<()> {
    let result = sqlx::query("DELETE FROM jadwal_ph WHERE id = $1")
        .bind(id)
        .execute(pool)
        .await?;
    if result.rows_affected() == 0 {
        return Err(AppError::NotFound("Jadwal PH tidak ditemukan".into()));
    }
    Ok(())
}

pub async fn tanggal_personel(
    pool: &PgPool,
    personel_id: Uuid,
    dari: NaiveDate,
    sampai: NaiveDate,
) -> AppResult<Vec<NaiveDate>> {
    const SQL: &str = r#"
        SELECT DISTINCT j.tanggal
          FROM jadwal_ph j
          JOIN jadwal_ph_personel jp ON jp.jadwal_ph_id = j.id
         WHERE jp.personel_id = $1 AND j.tanggal BETWEEN $2 AND $3
    "#;
    Ok(sqlx::query_scalar::<_, NaiveDate>(SQL)
        .bind(personel_id)
        .bind(dari)
        .bind(sampai)
        .fetch_all(pool)
        .await?)
}

/// Tanggal dengan check-in strong point; dipakai untuk menetralkan telat absensi kantor.
pub async fn tanggal_personel_hadir(
    pool: &PgPool,
    personel_id: Uuid,
    dari: NaiveDate,
    sampai: NaiveDate,
) -> AppResult<Vec<NaiveDate>> {
    const SQL: &str = r#"
        SELECT DISTINCT j.tanggal
          FROM absensi_ph ap
          JOIN jadwal_ph j ON j.id = ap.jadwal_ph_id
         WHERE ap.personel_id = $1
           AND ap.jenis = 'masuk'
           AND j.tanggal BETWEEN $2 AND $3
         ORDER BY j.tanggal
    "#;
    Ok(sqlx::query_scalar::<_, NaiveDate>(SQL)
        .bind(personel_id)
        .bind(dari)
        .bind(sampai)
        .fetch_all(pool)
        .await?)
}

pub async fn personel_ditugaskan(
    pool: &PgPool,
    jadwal_id: Uuid,
    personel_id: Uuid,
) -> AppResult<bool> {
    Ok(sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS (
             SELECT 1 FROM jadwal_ph_personel
              WHERE jadwal_ph_id = $1 AND personel_id = $2
         )",
    )
    .bind(jadwal_id)
    .bind(personel_id)
    .fetch_one(pool)
    .await?)
}

/// Mendeteksi personel yang sudah punya jadwal PH lain pada tanggal dan jam
/// yang beririsan. `kecuali` dipakai saat memperbarui jadwal yang sama.
pub async fn ada_bentrok(
    pool: &PgPool,
    personel_ids: &[Uuid],
    tanggal: NaiveDate,
    jam_mulai: NaiveTime,
    jam_selesai: NaiveTime,
    kecuali: Option<Uuid>,
) -> AppResult<bool> {
    const SQL: &str = r#"
        SELECT EXISTS (
            SELECT 1
              FROM jadwal_ph j
              JOIN jadwal_ph_personel jp ON jp.jadwal_ph_id = j.id
             WHERE j.tanggal = $1
               AND ($2::uuid IS NULL OR j.id <> $2)
               AND jp.personel_id = ANY($3::uuid[])
               AND j.jam_mulai < $5
               AND j.jam_selesai > $4
        )
    "#;
    Ok(sqlx::query_scalar::<_, bool>(SQL)
        .bind(tanggal)
        .bind(kecuali)
        .bind(personel_ids)
        .bind(jam_mulai)
        .bind(jam_selesai)
        .fetch_one(pool)
        .await?)
}

// ---------------------------------------------------------------- absensi PH

pub async fn find_absensi_ph(pool: &PgPool, id: Uuid) -> AppResult<Option<AbsensiPhRow>> {
    const SQL: &str = r#"
        SELECT ap.id, ap.client_uuid, ap.jadwal_ph_id, ap.personel_id,
               p.nama AS personel_nama, p.nrp AS personel_nrp,
               ap.strong_point_id, sp.nama AS strong_point_nama, ap.satker_id,
               ap.jenis, ap.captured_at, ap.jam,
               ap.latitude::float8 AS latitude, ap.longitude::float8 AS longitude,
               ap.akurasi_m::float8 AS akurasi_m, ap.jarak_m::float8 AS jarak_m,
               ap.dalam_radius, ap.foto_path, ap.is_offline, ap.is_mock_suspect,
               ap.perlu_verifikasi, ap.verified_at, ap.catatan_verifikator
          FROM absensi_ph ap
          JOIN personel p ON p.id = ap.personel_id
          JOIN strong_point sp ON sp.id = ap.strong_point_id
         WHERE ap.id = $1
    "#;
    Ok(sqlx::query_as::<_, AbsensiPhRow>(SQL)
        .bind(id)
        .fetch_optional(pool)
        .await?)
}

pub async fn find_absensi_ph_by_client(
    pool: &PgPool,
    client_uuid: Uuid,
) -> AppResult<Option<AbsensiPhRow>> {
    const SQL: &str = r#"
        SELECT ap.id, ap.client_uuid, ap.jadwal_ph_id, ap.personel_id,
               p.nama AS personel_nama, p.nrp AS personel_nrp,
               ap.strong_point_id, sp.nama AS strong_point_nama, ap.satker_id,
               ap.jenis, ap.captured_at, ap.jam,
               ap.latitude::float8 AS latitude, ap.longitude::float8 AS longitude,
               ap.akurasi_m::float8 AS akurasi_m, ap.jarak_m::float8 AS jarak_m,
               ap.dalam_radius, ap.foto_path, ap.is_offline, ap.is_mock_suspect,
               ap.perlu_verifikasi, ap.verified_at, ap.catatan_verifikator
          FROM absensi_ph ap
          JOIN personel p ON p.id = ap.personel_id
          JOIN strong_point sp ON sp.id = ap.strong_point_id
         WHERE ap.client_uuid = $1
    "#;
    Ok(sqlx::query_as::<_, AbsensiPhRow>(SQL)
        .bind(client_uuid)
        .fetch_optional(pool)
        .await?)
}

pub async fn exists_absensi_ph(
    pool: &PgPool,
    jadwal_id: Uuid,
    personel_id: Uuid,
    jenis: AbsensiJenis,
) -> AppResult<bool> {
    Ok(sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS (
             SELECT 1 FROM absensi_ph
              WHERE jadwal_ph_id = $1 AND personel_id = $2 AND jenis = $3
         )",
    )
    .bind(jadwal_id)
    .bind(personel_id)
    .bind(jenis)
    .fetch_one(pool)
    .await?)
}

/// Waktu check-in dan check-out terakhir untuk menentukan sesi yang masih terbuka.
pub async fn sesi_ph_terakhir(
    pool: &PgPool,
    jadwal_id: Uuid,
    personel_id: Uuid,
) -> AppResult<SesiPhRow> {
    const SQL: &str = r#"
        SELECT
            (SELECT max(captured_at) FROM absensi_ph
              WHERE jadwal_ph_id = $1 AND personel_id = $2 AND jenis = 'masuk')
                AS masuk_terakhir,
            (SELECT max(captured_at) FROM absensi_ph
              WHERE jadwal_ph_id = $1 AND personel_id = $2 AND jenis = 'pulang')
                AS pulang_terakhir
    "#;
    Ok(sqlx::query_as::<_, SesiPhRow>(SQL)
        .bind(jadwal_id)
        .bind(personel_id)
        .fetch_one(pool)
        .await?)
}

pub async fn insert_absensi_ph(pool: &PgPool, data: &NewAbsensiPh) -> AppResult<i64> {
    let result = sqlx::query(
        "INSERT INTO absensi_ph (
             id, client_uuid, jadwal_ph_id, personel_id, strong_point_id, satker_id,
             jenis, captured_at, jam, latitude, longitude, akurasi_m, jarak_m,
             dalam_radius, foto_path, is_offline, is_mock_suspect, perlu_verifikasi
         )
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9,
                 $10::float8::numeric, $11::float8::numeric, $12::float8::numeric,
                 $13::float8::numeric, $14, $15, $16, $17, $18)
         ON CONFLICT (client_uuid) DO NOTHING",
    )
    .bind(data.id)
    .bind(data.client_uuid)
    .bind(data.jadwal_ph_id)
    .bind(data.personel_id)
    .bind(data.strong_point_id)
    .bind(data.satker_id)
    .bind(data.jenis)
    .bind(data.captured_at)
    .bind(data.jam)
    .bind(data.latitude)
    .bind(data.longitude)
    .bind(data.akurasi_m)
    .bind(data.jarak_m)
    .bind(data.dalam_radius)
    .bind(data.foto_path.as_deref())
    .bind(data.is_offline)
    .bind(data.is_mock_suspect)
    .bind(data.perlu_verifikasi)
    .execute(pool)
    .await?;
    Ok(result.rows_affected() as i64)
}

pub async fn list_absensi_ph_saya(
    pool: &PgPool,
    personel_id: Uuid,
    dari: NaiveDate,
    sampai: NaiveDate,
) -> AppResult<Vec<AbsensiPhRow>> {
    const SQL: &str = r#"
        SELECT ap.id, ap.client_uuid, ap.jadwal_ph_id, ap.personel_id,
               p.nama AS personel_nama, p.nrp AS personel_nrp,
               ap.strong_point_id, sp.nama AS strong_point_nama, ap.satker_id,
               ap.jenis, ap.captured_at, ap.jam,
               ap.latitude::float8 AS latitude, ap.longitude::float8 AS longitude,
               ap.akurasi_m::float8 AS akurasi_m, ap.jarak_m::float8 AS jarak_m,
               ap.dalam_radius, ap.foto_path, ap.is_offline, ap.is_mock_suspect,
               ap.perlu_verifikasi, ap.verified_at, ap.catatan_verifikator
          FROM absensi_ph ap
          JOIN personel p ON p.id = ap.personel_id
          JOIN strong_point sp ON sp.id = ap.strong_point_id
         WHERE ap.personel_id = $1 AND ap.captured_at::date BETWEEN $2 AND $3
         ORDER BY ap.captured_at
    "#;
    Ok(sqlx::query_as::<_, AbsensiPhRow>(SQL)
        .bind(personel_id)
        .bind(dari)
        .bind(sampai)
        .fetch_all(pool)
        .await?)
}

pub async fn list_absensi_ph_jadwal(
    pool: &PgPool,
    jadwal_id: Uuid,
) -> AppResult<Vec<AbsensiPhRow>> {
    const SQL: &str = r#"
        SELECT ap.id, ap.client_uuid, ap.jadwal_ph_id, ap.personel_id,
               p.nama AS personel_nama, p.nrp AS personel_nrp,
               ap.strong_point_id, sp.nama AS strong_point_nama, ap.satker_id,
               ap.jenis, ap.captured_at, ap.jam,
               ap.latitude::float8 AS latitude, ap.longitude::float8 AS longitude,
               ap.akurasi_m::float8 AS akurasi_m, ap.jarak_m::float8 AS jarak_m,
               ap.dalam_radius, ap.foto_path, ap.is_offline, ap.is_mock_suspect,
               ap.perlu_verifikasi, ap.verified_at, ap.catatan_verifikator
          FROM absensi_ph ap
          JOIN personel p ON p.id = ap.personel_id
          JOIN strong_point sp ON sp.id = ap.strong_point_id
         WHERE ap.jadwal_ph_id = $1
         ORDER BY ap.jam
    "#;
    Ok(sqlx::query_as::<_, AbsensiPhRow>(SQL)
        .bind(jadwal_id)
        .fetch_all(pool)
        .await?)
}

pub async fn rekap_ph(
    pool: &PgPool,
    satker_ids: &[Uuid],
    dari: NaiveDate,
    sampai: NaiveDate,
) -> AppResult<Vec<RekapPhRow>> {
    const SQL: &str = r#"
        SELECT p.id AS personel_id, p.nama, p.nrp, p.pangkat, p.unit,
               count(DISTINCT jp.jadwal_ph_id) AS jumlah_jadwal,
               count(DISTINCT ap.jadwal_ph_id) FILTER (WHERE ap.jenis = 'masuk') AS hadir,
               count(DISTINCT jp.jadwal_ph_id) FILTER (
                   WHERE EXISTS (
                       SELECT 1 FROM absensi_ph ap2
                        WHERE ap2.jadwal_ph_id = jp.jadwal_ph_id
                          AND ap2.personel_id = p.id
                          AND ap2.jenis = 'masuk'
                   )
                   AND NOT EXISTS (
                       SELECT 1 FROM absensi_ph ap3
                        WHERE ap3.jadwal_ph_id = jp.jadwal_ph_id
                          AND ap3.personel_id = p.id
                          AND ap3.jenis = 'pulang'
                   )
               ) AS hanya_check_in,
               count(DISTINCT jp.jadwal_ph_id)
                   - count(DISTINCT ap.jadwal_ph_id) FILTER (WHERE ap.jenis = 'masuk')
                   AS tidak_hadir,
               coalesce(
                   array_agg(DISTINCT j.tanggal) FILTER (
                       WHERE NOT EXISTS (
                           SELECT 1 FROM absensi_ph ap2
                            WHERE ap2.jadwal_ph_id = jp.jadwal_ph_id
                              AND ap2.personel_id = p.id
                              AND ap2.jenis = 'masuk'
                       )
                   ),
                   '{}'::date[]
               ) AS tanggal_tidak_hadir,
               coalesce(
                   array_agg(DISTINCT j.tanggal) FILTER (
                       WHERE EXISTS (
                           SELECT 1 FROM absensi_ph ap2
                            WHERE ap2.jadwal_ph_id = jp.jadwal_ph_id
                              AND ap2.personel_id = p.id
                              AND ap2.jenis = 'masuk'
                       )
                       AND NOT EXISTS (
                           SELECT 1 FROM absensi_ph ap3
                            WHERE ap3.jadwal_ph_id = jp.jadwal_ph_id
                              AND ap3.personel_id = p.id
                              AND ap3.jenis = 'pulang'
                       )
                   ),
                   '{}'::date[]
               ) AS tanggal_hanya_check_in,
               count(DISTINCT ap.id) FILTER (
                   WHERE ap.perlu_verifikasi AND ap.verified_at IS NULL
               ) AS menunggu_verifikasi
          FROM jadwal_ph j
          JOIN jadwal_ph_personel jp ON jp.jadwal_ph_id = j.id
          JOIN personel p ON p.id = jp.personel_id
          LEFT JOIN absensi_ph ap
                 ON ap.jadwal_ph_id = j.id AND ap.personel_id = p.id
         WHERE j.satker_id = ANY($1::uuid[]) AND j.tanggal BETWEEN $2 AND $3
         GROUP BY p.id, p.nama, p.nrp, p.pangkat, p.unit
         ORDER BY p.nama
    "#;
    Ok(sqlx::query_as::<_, RekapPhRow>(SQL)
        .bind(satker_ids)
        .bind(dari)
        .bind(sampai)
        .fetch_all(pool)
        .await?)
}

pub async fn verifikasi_absensi_ph(
    pool: &PgPool,
    id: Uuid,
    verified_by: Uuid,
    catatan: Option<&str>,
) -> AppResult<()> {
    let result = sqlx::query(
        "UPDATE absensi_ph
            SET perlu_verifikasi = false, verified_by = $2, verified_at = now(),
                catatan_verifikator = $3
          WHERE id = $1",
    )
    .bind(id)
    .bind(verified_by)
    .bind(catatan)
    .execute(pool)
    .await?;
    if result.rows_affected() == 0 {
        return Err(AppError::NotFound("Absensi PH tidak ditemukan".into()));
    }
    Ok(())
}
