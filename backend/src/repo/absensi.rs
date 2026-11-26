use chrono::{DateTime, NaiveDate, NaiveTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use crate::domain::enums::{AbsensiJenis, StatusKehadiran};
use crate::error::{AppError, AppResult};

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct AbsensiRow {
    pub id: Uuid,
    pub grup_id: Uuid,
    pub jadwal_ph_id: Option<Uuid>,
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
    pub satker_id: Uuid,
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
    pub grup_id: Uuid,
    pub jadwal_ph_id: Option<Uuid>,
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
            id, grup_id, jadwal_ph_id, client_uuid, personel_id, satker_id, tanggal, jenis,
            status_kehadiran, captured_at, jam, latitude, longitude, akurasi_m, jarak_m,
            dalam_radius, is_mock_suspect, is_offline, foto_path, device_id, perlu_verifikasi
        )
        VALUES (
            $1, $2, $3, $4, $5, $6, $7, $8, $9,
            $10, $11, $12::float8::numeric, $13::float8::numeric,
            $14::float8::numeric, $15::float8::numeric, $16,
            $17, $18, $19, $20, $21
        )
        ON CONFLICT (client_uuid) DO NOTHING
    "#;

    let result = sqlx::query(SQL)
        .bind(data.id)
        .bind(data.grup_id)
        .bind(data.jadwal_ph_id)
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
        SELECT id, grup_id, jadwal_ph_id, client_uuid, personel_id, satker_id, tanggal, jenis,
               status_kehadiran, captured_at, jam,
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
        SELECT a.id, a.grup_id, a.jadwal_ph_id, a.client_uuid, a.personel_id, a.satker_id, a.tanggal,
               a.jenis, a.status_kehadiran, a.captured_at, a.jam,
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

/// Catatan check-in terakhir yang belum memiliki check-out.
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct AbsensiTerbukaRow {
    pub id: Uuid,
    pub grup_id: Uuid,
    pub tanggal: NaiveDate,
    pub status_kehadiran: StatusKehadiran,
    pub captured_at: DateTime<Utc>,
    pub jam: NaiveTime,
}

pub async fn catatan_terbuka(
    pool: &PgPool,
    personel_id: Uuid,
    batas_tanggal: NaiveDate,
) -> AppResult<Option<AbsensiTerbukaRow>> {
    const SQL: &str = r#"
        SELECT m.id, coalesce(m.grup_id, m.id) AS grup_id, m.tanggal,
               m.status_kehadiran, m.captured_at, m.jam
          FROM absensi m
         WHERE m.personel_id = $1
           AND m.tanggal >= $2
           AND m.jenis = 'masuk'
           AND m.status_kehadiran IN ('dinas_kantor', 'dinas_luar', 'piket', 'operasi')
           AND NOT EXISTS (
               SELECT 1 FROM absensi p
                WHERE p.personel_id = m.personel_id
                  AND p.jenis = 'pulang'
                  AND coalesce(p.grup_id, p.id) = coalesce(m.grup_id, m.id)
           )
         ORDER BY m.captured_at DESC
         LIMIT 1
    "#;
    Ok(sqlx::query_as::<_, AbsensiTerbukaRow>(SQL)
        .bind(personel_id)
        .bind(batas_tanggal)
        .fetch_optional(pool)
        .await?)
}

pub async fn ada_status_hari_ini(
    pool: &PgPool,
    personel_id: Uuid,
    tanggal: NaiveDate,
    status: StatusKehadiran,
) -> AppResult<bool> {
    Ok(sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS (
             SELECT 1 FROM absensi
              WHERE personel_id = $1 AND tanggal = $2 AND status_kehadiran = $3
         )",
    )
    .bind(personel_id)
    .bind(tanggal)
    .bind(status)
    .fetch_one(pool)
    .await?)
}

pub async fn status_hari_ini(
    pool: &PgPool,
    personel_id: Uuid,
    tanggal: NaiveDate,
) -> AppResult<Vec<StatusKehadiran>> {
    Ok(sqlx::query_scalar::<_, StatusKehadiran>(
        "SELECT DISTINCT status_kehadiran FROM absensi
          WHERE personel_id = $1 AND tanggal = $2 AND jenis = 'masuk'",
    )
    .bind(personel_id)
    .bind(tanggal)
    .fetch_all(pool)
    .await?)
}

pub async fn list_personel(
    pool: &PgPool,
    personel_id: Uuid,
    dari: NaiveDate,
    sampai: NaiveDate,
) -> AppResult<Vec<AbsensiRow>> {
    const SQL: &str = r#"
        SELECT a.id, a.grup_id, a.jadwal_ph_id, a.client_uuid, a.personel_id, a.satker_id, a.tanggal,
               a.jenis, a.status_kehadiran, a.captured_at, a.jam,
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
    SELECT a.id, a.personel_id, a.satker_id, p.nama AS personel_nama,
           p.nrp AS personel_nrp, p.pangkat AS personel_pangkat, p.unit,
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
       AND ($5::date IS NULL OR a.tanggal >= $5)
       AND ($6::date IS NULL OR a.tanggal <= $6)
     ORDER BY a.tanggal DESC, a.jam DESC
     LIMIT $7 OFFSET $8
"#;

const LIST_COUNT: &str = r#"
    SELECT count(*)
      FROM absensi a
     WHERE a.satker_id = ANY($1::uuid[])
       AND ($2::date IS NULL OR a.tanggal = $2)
       AND ($3::bool IS NULL OR a.perlu_verifikasi = $3)
       AND ($4::uuid IS NULL OR a.personel_id = $4)
       AND ($5::date IS NULL OR a.tanggal >= $5)
       AND ($6::date IS NULL OR a.tanggal <= $6)
"#;

pub struct ListFilter {
    pub satker_ids: Vec<Uuid>,
    pub tanggal: Option<NaiveDate>,
    pub perlu_verifikasi: Option<bool>,
    pub personel_id: Option<Uuid>,
    pub dari: Option<NaiveDate>,
    pub sampai: Option<NaiveDate>,
    pub limit: i64,
    pub offset: i64,
}

pub async fn list(pool: &PgPool, filter: &ListFilter) -> AppResult<(Vec<AbsensiListItem>, i64)> {
    let rows = sqlx::query_as::<_, AbsensiListItem>(LIST_SELECT)
        .bind(&filter.satker_ids)
        .bind(filter.tanggal)
        .bind(filter.perlu_verifikasi)
        .bind(filter.personel_id)
        .bind(filter.dari)
        .bind(filter.sampai)
        .bind(filter.limit)
        .bind(filter.offset)
        .fetch_all(pool)
        .await?;

    let total: i64 = sqlx::query_scalar::<_, i64>(LIST_COUNT)
        .bind(&filter.satker_ids)
        .bind(filter.tanggal)
        .bind(filter.perlu_verifikasi)
        .bind(filter.personel_id)
        .bind(filter.dari)
        .bind(filter.sampai)
        .fetch_one(pool)
        .await?;

    Ok((rows, total))
}

// ------------------------------------------------------- listing harian (gabung)

/// Satu baris per personel per tanggal (absen masuk dan pulang digabung).
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct AbsensiHarianRow {
    pub personel_id: Uuid,
    pub tanggal: NaiveDate,
    pub satker_id: Uuid,
    pub personel_nama: String,
    pub personel_nrp: String,
    pub personel_pangkat: String,
    pub unit: Option<String>,
    pub masuk_jam: Option<NaiveTime>,
    pub pulang_jam: Option<NaiveTime>,
    pub masuk_latitude: Option<f64>,
    pub masuk_longitude: Option<f64>,
    pub pulang_latitude: Option<f64>,
    pub pulang_longitude: Option<f64>,
    pub status_masuk: Option<StatusKehadiran>,
    pub status_pulang: Option<StatusKehadiran>,
    pub is_offline: bool,
    pub is_mock_suspect: bool,
    pub luar_radius: bool,
    pub perlu_verifikasi: bool,
    /// ID absensi (masuk/pulang) yang masih perlu diverifikasi.
    pub perlu_verifikasi_ids: Vec<Uuid>,
    /// Ada jadwal strong point pada tanggal ini (bukan penanda check-in).
    pub ph_hadir: bool,
    pub verified_at: Option<DateTime<Utc>>,
    pub catatan_verifikator: Option<String>,
    pub menit_telat: i64,
    pub menit_pulang_cepat: i64,
    pub foto_path: Option<String>,
    pub foto_masuk_path: Option<String>,
    pub foto_pulang_path: Option<String>,
}

/// Satu hari yang dihitung "lepas dinas" karena piket/operasi check-out
/// pada hari berikutnya setelah shift malam.
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct LepasDinasRow {
    pub personel_id: Uuid,
    pub tanggal: NaiveDate,
}

pub struct ListHarianFilter {
    pub satker_ids: Vec<Uuid>,
    pub personel_id: Option<Uuid>,
    pub dari: Option<NaiveDate>,
    pub sampai: Option<NaiveDate>,
    /// `true` = hanya tanggal yang masih memiliki absensi menunggu verifikasi.
    pub perlu_verifikasi: Option<bool>,
    pub limit: i64,
    pub offset: i64,
}

const HARIAN_SELECT: &str = r#"
    SELECT a.personel_id,
           a.tanggal,
           a.satker_id,
           p.nama AS personel_nama,
           p.nrp AS personel_nrp,
           p.pangkat AS personel_pangkat,
           p.unit,
           CASE WHEN bool_or(phd.tanggal IS NOT NULL)
                THEN max(a.jam) FILTER (
                    WHERE a.jenis = 'masuk' AND a.status_kehadiran = 'ph'
                )
                ELSE max(a.jam) FILTER (WHERE a.jenis = 'masuk')
           END AS masuk_jam,
           CASE WHEN bool_or(phd.tanggal IS NOT NULL)
                THEN max(a.jam) FILTER (
                    WHERE a.jenis = 'pulang' AND a.status_kehadiran = 'ph'
                )
                ELSE max(a.jam) FILTER (WHERE a.jenis = 'pulang')
           END AS pulang_jam,
           CASE WHEN bool_or(phd.tanggal IS NOT NULL)
                THEN max(a.latitude::float8) FILTER (
                    WHERE a.jenis = 'masuk' AND a.status_kehadiran = 'ph'
                )
                ELSE max(a.latitude::float8) FILTER (WHERE a.jenis = 'masuk')
           END AS masuk_latitude,
           CASE WHEN bool_or(phd.tanggal IS NOT NULL)
                THEN max(a.longitude::float8) FILTER (
                    WHERE a.jenis = 'masuk' AND a.status_kehadiran = 'ph'
                )
                ELSE max(a.longitude::float8) FILTER (WHERE a.jenis = 'masuk')
           END AS masuk_longitude,
           CASE WHEN bool_or(phd.tanggal IS NOT NULL)
                THEN max(a.latitude::float8) FILTER (
                    WHERE a.jenis = 'pulang' AND a.status_kehadiran = 'ph'
                )
                ELSE max(a.latitude::float8) FILTER (WHERE a.jenis = 'pulang')
           END AS pulang_latitude,
           CASE WHEN bool_or(phd.tanggal IS NOT NULL)
                THEN max(a.longitude::float8) FILTER (
                    WHERE a.jenis = 'pulang' AND a.status_kehadiran = 'ph'
                )
                ELSE max(a.longitude::float8) FILTER (WHERE a.jenis = 'pulang')
           END AS pulang_longitude,
           (array_agg(a.status_kehadiran) FILTER (WHERE a.jenis = 'masuk'))[1]
               AS status_masuk,
           (array_agg(a.status_kehadiran) FILTER (WHERE a.jenis = 'pulang'))[1]
               AS status_pulang,
           bool_or(a.is_offline) AS is_offline,
           bool_or(a.is_mock_suspect) AS is_mock_suspect,
           bool_or(NOT a.dalam_radius) AS luar_radius,
           bool_or(a.perlu_verifikasi) AS perlu_verifikasi,
           coalesce(
               array_agg(a.id) FILTER (WHERE a.perlu_verifikasi),
               '{}'::uuid[]
           ) AS perlu_verifikasi_ids,
           bool_or(phd.tanggal IS NOT NULL) AS ph_hadir,
           max(a.verified_at) AS verified_at,
           max(a.catatan_verifikator) AS catatan_verifikator,
           coalesce(max(
               CASE
                   WHEN a.jenis = 'masuk'
                        AND a.status_kehadiran = 'dinas_kantor'
                        AND a.jam > s.jam_masuk
                        AND phd.tanggal IS NULL
                       THEN floor(extract(epoch FROM (a.jam - s.jam_masuk)) / 60)::bigint
                   WHEN a.jenis = 'masuk'
                        AND a.status_kehadiran = 'ph'
                        AND jph.jam_mulai IS NOT NULL
                        AND a.jam > jph.jam_mulai
                       THEN floor(extract(epoch FROM (a.jam - jph.jam_mulai)) / 60)::bigint
                   ELSE 0
               END
           ), 0) AS menit_telat,
           coalesce(max(
               CASE
                   WHEN a.jenis = 'pulang'
                        AND a.status_kehadiran = 'dinas_kantor'
                        AND a.jam < s.jam_pulang
                       THEN floor(extract(epoch FROM (s.jam_pulang - a.jam)) / 60)::bigint
                   WHEN a.jenis = 'pulang'
                        AND a.status_kehadiran = 'ph'
                        AND jph.jam_selesai IS NOT NULL
                        AND a.jam < jph.jam_selesai
                       THEN floor(extract(epoch FROM (jph.jam_selesai - a.jam)) / 60)::bigint
                   ELSE 0
               END
           ), 0) AS menit_pulang_cepat,
           (array_agg(a.foto_path) FILTER (WHERE a.jenis = 'masuk' AND a.foto_path IS NOT NULL))[1]
               AS foto_masuk_path,
           (array_agg(a.foto_path) FILTER (WHERE a.jenis = 'pulang' AND a.foto_path IS NOT NULL))[1]
               AS foto_pulang_path,
           (array_agg(a.foto_path) FILTER (
               WHERE a.perlu_verifikasi AND a.foto_path IS NOT NULL
           ))[1] AS foto_path
      FROM absensi a
      JOIN personel p ON p.id = a.personel_id
      JOIN satker s ON s.id = a.satker_id
      LEFT JOIN jadwal_ph jph ON jph.id = a.jadwal_ph_id
      LEFT JOIN (
          SELECT DISTINCT aph.personel_id, jph.tanggal
            FROM absensi_ph aph
            JOIN jadwal_ph jph ON jph.id = aph.jadwal_ph_id
           WHERE aph.jenis = 'masuk'
             AND jph.tanggal BETWEEN $3 AND $4
      ) phd ON phd.personel_id = a.personel_id AND phd.tanggal = a.tanggal
     WHERE a.satker_id = ANY($1::uuid[])
       AND ($2::uuid IS NULL OR a.personel_id = $2)
       AND ($3::date IS NULL OR a.tanggal >= $3)
       AND ($4::date IS NULL OR a.tanggal <= $4)
     GROUP BY a.personel_id, a.tanggal, a.satker_id,
              p.nama, p.nrp, p.pangkat, p.unit, s.jam_masuk
    HAVING ($5::bool IS NULL OR $5 = false OR bool_or(a.perlu_verifikasi))
     ORDER BY a.tanggal DESC, p.nama
     LIMIT $6 OFFSET $7
"#;

const HARIAN_COUNT: &str = r#"
    SELECT count(*)
      FROM (
          SELECT a.personel_id, a.tanggal
            FROM absensi a
           WHERE a.satker_id = ANY($1::uuid[])
             AND ($2::uuid IS NULL OR a.personel_id = $2)
             AND ($3::date IS NULL OR a.tanggal >= $3)
             AND ($4::date IS NULL OR a.tanggal <= $4)
           GROUP BY a.personel_id, a.tanggal
          HAVING ($5::bool IS NULL OR $5 = false OR bool_or(a.perlu_verifikasi))
      ) t
"#;

pub async fn list_harian(
    pool: &PgPool,
    filter: &ListHarianFilter,
) -> AppResult<(Vec<AbsensiHarianRow>, i64)> {
    let rows = sqlx::query_as::<_, AbsensiHarianRow>(HARIAN_SELECT)
        .bind(&filter.satker_ids)
        .bind(filter.personel_id)
        .bind(filter.dari)
        .bind(filter.sampai)
        .bind(filter.perlu_verifikasi)
        .bind(filter.limit)
        .bind(filter.offset)
        .fetch_all(pool)
        .await?;

    let total: i64 = sqlx::query_scalar::<_, i64>(HARIAN_COUNT)
        .bind(&filter.satker_ids)
        .bind(filter.personel_id)
        .bind(filter.dari)
        .bind(filter.sampai)
        .bind(filter.perlu_verifikasi)
        .fetch_one(pool)
        .await?;

    Ok((rows, total))
}

/// Tanggal-tanggal yang dihitung "lepas dinas": check-out piket/operasi yang
/// terjadi pada hari setelah check-in (shift lintas hari).
pub async fn lepas_dinas_matriks(
    pool: &PgPool,
    satker_ids: &[Uuid],
    dari: NaiveDate,
    sampai: NaiveDate,
    personel_id: Option<Uuid>,
) -> AppResult<Vec<LepasDinasRow>> {
    const SQL: &str = r#"
        SELECT p.personel_id,
               (p.captured_at AT TIME ZONE 'Asia/Makassar')::date AS tanggal
          FROM absensi p
          JOIN absensi m ON m.id = p.grup_id AND m.jenis = 'masuk'
         WHERE p.jenis = 'pulang'
           AND m.status_kehadiran IN ('piket', 'operasi')
           AND p.status_kehadiran IN ('piket', 'operasi')
           AND p.satker_id = ANY($1::uuid[])
           AND ($2::uuid IS NULL OR p.personel_id = $2)
           AND (p.captured_at AT TIME ZONE 'Asia/Makassar')::date
               BETWEEN greatest($3::date, m.tanggal + 1) AND $4
         GROUP BY 1, 2
         ORDER BY 2, 1
    "#;
    Ok(sqlx::query_as::<_, LepasDinasRow>(SQL)
        .bind(satker_ids)
        .bind(personel_id)
        .bind(dari)
        .bind(sampai)
        .fetch_all(pool)
        .await?)
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
    pub operasi: i64,
    pub telat: i64,
    /// Total menit keterlambatan untuk hari berstatus `dinas_kantor`.
    pub menit_telat: i64,
    /// Total menit pulang cepat untuk hari berstatus `dinas_kantor`.
    pub menit_pulang_cepat: i64,
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
               WHERE a.status_kehadiran IN ('dinas_kantor', 'dinas_luar', 'piket', 'operasi', 'ph')
           ) AS hadir,
           count(a.id) FILTER (WHERE a.status_kehadiran = 'dinas_luar') AS dinas_luar,
           count(a.id) FILTER (WHERE a.status_kehadiran = 'piket') AS piket,
           count(a.id) FILTER (WHERE a.status_kehadiran = 'operasi') AS operasi,
           count(a.id) FILTER (
               WHERE (a.status_kehadiran = 'dinas_kantor' AND a.jam > s.jam_masuk
                      AND phd.tanggal IS NULL)
                  OR (a.status_kehadiran = 'ph' AND a.jam > j.jam_mulai)
           ) AS telat,
           coalesce(sum(
               CASE WHEN a.status_kehadiran = 'dinas_kantor' AND a.jam > s.jam_masuk
                         AND phd.tanggal IS NULL
                    THEN floor(extract(epoch FROM (a.jam - s.jam_masuk)) / 60)::bigint
                    WHEN a.status_kehadiran = 'ph' AND a.jam > j.jam_mulai
                    THEN floor(extract(epoch FROM (a.jam - j.jam_mulai)) / 60)::bigint
                    ELSE 0 END
           ), 0)::bigint AS menit_telat,
           coalesce(sum(
               CASE WHEN pl.jenis = 'pulang' AND pl.status_kehadiran = 'dinas_kantor'
                         AND pl.jam < s.jam_pulang
                    THEN floor(extract(epoch FROM (s.jam_pulang - pl.jam)) / 60)::bigint
                    WHEN pl.jenis = 'pulang' AND pl.status_kehadiran = 'ph'
                         AND jp.id IS NOT NULL AND pl.jam < jp.jam_selesai
                    THEN floor(extract(epoch FROM (jp.jam_selesai - pl.jam)) / 60)::bigint
                    ELSE 0 END
           ), 0)::bigint AS menit_pulang_cepat,
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
      LEFT JOIN jadwal_ph j ON j.id = a.jadwal_ph_id
      LEFT JOIN absensi pl
             ON pl.personel_id = p.id
            AND pl.jenis = 'pulang'
            AND pl.tanggal = a.tanggal
            AND coalesce(pl.grup_id, pl.id) = coalesce(a.grup_id, a.id)
      LEFT JOIN jadwal_ph jp ON jp.id = pl.jadwal_ph_id
      LEFT JOIN (
          SELECT DISTINCT aph.personel_id, jph.tanggal
            FROM absensi_ph aph
            JOIN jadwal_ph jph ON jph.id = aph.jadwal_ph_id
           WHERE aph.jenis = 'masuk'
             AND jph.tanggal BETWEEN $2 AND $3
      ) phd ON phd.personel_id = p.id AND phd.tanggal = a.tanggal
     WHERE p.deleted_at IS NULL
       AND p.jenis_akun = 'personel'
       AND p.satker_id = ANY($1::uuid[])
     GROUP BY p.id, p.nama, p.nrp, p.pangkat, p.jabatan, p.unit, p.satker_id, s.jam_masuk
     ORDER BY p.nama
"#;

pub async fn rekap(
    pool: &PgPool,
    satker_ids: &[Uuid],
    dari: NaiveDate,
    sampai: NaiveDate,
) -> AppResult<Vec<RekapRow>> {
    Ok(sqlx::query_as::<_, RekapRow>(REKAP_SQL)
        .bind(satker_ids)
        .bind(dari)
        .bind(sampai)
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

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct FotoAbsensiRow {
    pub id: Uuid,
    pub tanggal: NaiveDate,
    pub jenis: AbsensiJenis,
    pub jam: NaiveTime,
    pub status_kehadiran: StatusKehadiran,
    pub foto_path: String,
}

/// Foto dokumentasi absensi pada rentang tanggal dan cakupan satker.
pub async fn list_foto(
    pool: &PgPool,
    satker_ids: &[Uuid],
    personel_id: Option<Uuid>,
    dari: NaiveDate,
    sampai: NaiveDate,
) -> AppResult<Vec<FotoAbsensiRow>> {
    const SQL: &str = r#"
        SELECT a.id, a.tanggal, a.jenis, a.jam, a.status_kehadiran, a.foto_path
          FROM absensi a
         WHERE a.satker_id = ANY($1::uuid[])
           AND ($2::uuid IS NULL OR a.personel_id = $2)
           AND a.tanggal BETWEEN $3 AND $4
           AND a.foto_path IS NOT NULL
         ORDER BY a.tanggal DESC, a.jam DESC
    "#;
    Ok(sqlx::query_as::<_, FotoAbsensiRow>(SQL)
        .bind(satker_ids)
        .bind(personel_id)
        .bind(dari)
        .bind(sampai)
        .fetch_all(pool)
        .await?)
}

/// Pemilik berkas foto absensi untuk pemeriksaan akses.
pub async fn pemilik_foto(pool: &PgPool, storage_key: &str) -> AppResult<Option<(Uuid, Uuid)>> {
    const SQL: &str = "SELECT personel_id, satker_id FROM absensi WHERE foto_path = $1";
    Ok(sqlx::query_as::<_, (Uuid, Uuid)>(SQL)
        .bind(storage_key)
        .fetch_optional(pool)
        .await?)
}
