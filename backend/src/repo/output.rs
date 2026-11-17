use chrono::{DateTime, NaiveDate, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use crate::domain::enums::{JenisOutput, StatusOutput};
use crate::error::{AppError, AppResult};

#[derive(Debug, Clone, serde::Serialize, sqlx::FromRow)]
pub struct OutputRow {
    pub id: Uuid,
    pub hasil_kerja_id: Uuid,
    pub rencana_id: Uuid,
    pub personel_id: Uuid,
    pub satker_id: Uuid,
    pub jenis_output: JenisOutput,
    pub judul: String,
    pub uraian: Option<String>,
    pub tanggal_hasil: NaiveDate,
    pub url_eksternal: Option<String>,
    pub status: StatusOutput,
    pub skor_kelengkapan: Option<f64>,
    pub skor_kesesuaian: Option<f64>,
    pub skor_keabsahan: Option<f64>,
    pub skor_total: Option<f64>,
    pub verified_by: Option<Uuid>,
    pub catatan_verifikator: Option<String>,
    pub verified_at: Option<DateTime<Utc>>,
    pub personel_nama: Option<String>,
    pub personel_nrp: Option<String>,
    pub rencana_kegiatan: Option<String>,
}

const LIST_SQL: &str = r#"
    SELECT o.id, o.hasil_kerja_id, o.rencana_id, o.personel_id, o.satker_id, o.jenis_output,
           o.judul, o.uraian, o.tanggal_hasil, o.url_eksternal, o.status,
           o.skor_kelengkapan::float8 AS skor_kelengkapan,
           o.skor_kesesuaian::float8 AS skor_kesesuaian,
           o.skor_keabsahan::float8 AS skor_keabsahan,
           o.skor_total::float8 AS skor_total,
           o.verified_by, o.catatan_verifikator, o.verified_at,
           p.nama AS personel_nama, p.nrp AS personel_nrp,
           r.kegiatan AS rencana_kegiatan
      FROM output o
      JOIN personel p ON p.id = o.personel_id
      JOIN rencana_kerja r ON r.id = o.rencana_id
     WHERE o.satker_id = ANY($1::uuid[])
       AND ($2::uuid IS NULL OR o.personel_id = $2)
       AND ($3::status_output IS NULL OR o.status = $3)
       AND ($4::uuid IS NULL OR r.periode_id = $4)
       AND ($5::uuid IS NULL OR r.atasan_id = $5)
       AND (NOT $6::bool OR o.status = 'dalam_verifikasi')
     ORDER BY o.created_at DESC
     LIMIT $7 OFFSET $8
"#;

const LIST_COUNT: &str = r#"
    SELECT count(*)
      FROM output o
      JOIN rencana_kerja r ON r.id = o.rencana_id
     WHERE o.satker_id = ANY($1::uuid[])
       AND ($2::uuid IS NULL OR o.personel_id = $2)
       AND ($3::status_output IS NULL OR o.status = $3)
       AND ($4::uuid IS NULL OR r.periode_id = $4)
       AND ($5::uuid IS NULL OR r.atasan_id = $5)
       AND (NOT $6::bool OR o.status = 'dalam_verifikasi')
"#;

pub struct ListFilter {
    pub satker_ids: Vec<Uuid>,
    pub personel_id: Option<Uuid>,
    pub status: Option<StatusOutput>,
    pub periode_id: Option<Uuid>,
    pub atasan_id: Option<Uuid>,
    pub hanya_diajukan: bool,
    pub limit: i64,
    pub offset: i64,
}

pub async fn list(pool: &PgPool, filter: &ListFilter) -> AppResult<(Vec<OutputRow>, i64)> {
    let rows = sqlx::query_as::<_, OutputRow>(LIST_SQL)
        .bind(&filter.satker_ids)
        .bind(filter.personel_id)
        .bind(filter.status)
        .bind(filter.periode_id)
        .bind(filter.atasan_id)
        .bind(filter.hanya_diajukan)
        .bind(filter.limit)
        .bind(filter.offset)
        .fetch_all(pool)
        .await?;

    let total: i64 = sqlx::query_scalar::<_, i64>(LIST_COUNT)
        .bind(&filter.satker_ids)
        .bind(filter.personel_id)
        .bind(filter.status)
        .bind(filter.periode_id)
        .bind(filter.atasan_id)
        .bind(filter.hanya_diajukan)
        .fetch_one(pool)
        .await?;

    Ok((rows, total))
}

pub async fn find(pool: &PgPool, id: Uuid) -> AppResult<Option<OutputRow>> {
    const SQL: &str = r#"
        SELECT o.id, o.hasil_kerja_id, o.rencana_id, o.personel_id, o.satker_id, o.jenis_output,
               o.judul, o.uraian, o.tanggal_hasil, o.url_eksternal, o.status,
               o.skor_kelengkapan::float8 AS skor_kelengkapan,
               o.skor_kesesuaian::float8 AS skor_kesesuaian,
               o.skor_keabsahan::float8 AS skor_keabsahan,
               o.skor_total::float8 AS skor_total,
               o.verified_by, o.catatan_verifikator, o.verified_at,
               p.nama AS personel_nama, p.nrp AS personel_nrp,
               r.kegiatan AS rencana_kegiatan
          FROM output o
          JOIN personel p ON p.id = o.personel_id
          JOIN rencana_kerja r ON r.id = o.rencana_id
         WHERE o.id = $1
    "#;
    Ok(sqlx::query_as::<_, OutputRow>(SQL)
        .bind(id)
        .fetch_optional(pool)
        .await?)
}

pub async fn by_personel(
    pool: &PgPool,
    personel_id: Uuid,
    periode_id: Uuid,
) -> AppResult<Vec<OutputRow>> {
    const SQL: &str = r#"
        SELECT o.id, o.hasil_kerja_id, o.rencana_id, o.personel_id, o.satker_id, o.jenis_output,
               o.judul, o.uraian, o.tanggal_hasil, o.url_eksternal, o.status,
               o.skor_kelengkapan::float8 AS skor_kelengkapan,
               o.skor_kesesuaian::float8 AS skor_kesesuaian,
               o.skor_keabsahan::float8 AS skor_keabsahan,
               o.skor_total::float8 AS skor_total,
               o.verified_by, o.catatan_verifikator, o.verified_at,
               p.nama AS personel_nama, p.nrp AS personel_nrp,
               r.kegiatan AS rencana_kegiatan
          FROM output o
          JOIN personel p ON p.id = o.personel_id
          JOIN rencana_kerja r ON r.id = o.rencana_id
         WHERE o.personel_id = $1 AND r.periode_id = $2
    "#;
    Ok(sqlx::query_as::<_, OutputRow>(SQL)
        .bind(personel_id)
        .bind(periode_id)
        .fetch_all(pool)
        .await?)
}

pub struct NewOutput {
    pub id: Uuid,
    pub hasil_kerja_id: Uuid,
    pub rencana_id: Uuid,
    pub personel_id: Uuid,
    pub satker_id: Uuid,
    pub jenis_output: JenisOutput,
    pub judul: String,
    pub uraian: Option<String>,
    pub tanggal_hasil: NaiveDate,
    pub url_eksternal: Option<String>,
    pub status: StatusOutput,
}

pub async fn create(pool: &PgPool, data: &NewOutput) -> AppResult<()> {
    const SQL: &str = r#"
        INSERT INTO output (id, hasil_kerja_id, rencana_id, personel_id, satker_id, jenis_output,
                            judul, uraian, tanggal_hasil, url_eksternal, status)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
    "#;
    sqlx::query(SQL)
        .bind(data.id)
        .bind(data.hasil_kerja_id)
        .bind(data.rencana_id)
        .bind(data.personel_id)
        .bind(data.satker_id)
        .bind(data.jenis_output)
        .bind(&data.judul)
        .bind(&data.uraian)
        .bind(data.tanggal_hasil)
        .bind(&data.url_eksternal)
        .bind(data.status)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn set_status(pool: &PgPool, id: Uuid, status: StatusOutput) -> AppResult<()> {
    let result = sqlx::query("UPDATE output SET status = $2 WHERE id = $1")
        .bind(id)
        .bind(status)
        .execute(pool)
        .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound("Output tidak ditemukan".into()));
    }
    Ok(())
}

pub struct UpdateOutput {
    pub jenis_output: JenisOutput,
    pub judul: String,
    pub uraian: Option<String>,
    pub tanggal_hasil: NaiveDate,
    pub url_eksternal: Option<String>,
}

pub async fn update(pool: &PgPool, id: Uuid, data: &UpdateOutput) -> AppResult<()> {
    const SQL: &str = r#"
        UPDATE output
           SET jenis_output = $2,
               judul = $3,
               uraian = $4,
               tanggal_hasil = $5,
               url_eksternal = $6
         WHERE id = $1
    "#;
    let result = sqlx::query(SQL)
        .bind(id)
        .bind(data.jenis_output)
        .bind(&data.judul)
        .bind(&data.uraian)
        .bind(data.tanggal_hasil)
        .bind(&data.url_eksternal)
        .execute(pool)
        .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound("Output tidak ditemukan".into()));
    }
    Ok(())
}

/// Mengajukan ulang output dan membersihkan hasil verifikasi sebelumnya.
pub async fn ajukan(pool: &PgPool, id: Uuid) -> AppResult<()> {
    const SQL: &str = r#"
        UPDATE output
           SET status = 'dalam_verifikasi',
               skor_kelengkapan = NULL,
               skor_kesesuaian = NULL,
               skor_keabsahan = NULL,
               skor_total = NULL,
               verified_by = NULL,
               catatan_verifikator = NULL,
               verified_at = NULL
         WHERE id = $1
    "#;
    let result = sqlx::query(SQL).bind(id).execute(pool).await?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound("Output tidak ditemukan".into()));
    }
    Ok(())
}

pub struct SkorOutput {
    pub kelengkapan: f64,
    pub kesesuaian: f64,
    pub keabsahan: f64,
    pub total: f64,
}

pub async fn verifikasi(
    pool: &PgPool,
    id: Uuid,
    verified_by: Uuid,
    skor: &SkorOutput,
    status: StatusOutput,
    catatan: Option<&str>,
) -> AppResult<()> {
    const SQL: &str = r#"
        UPDATE output
           SET status = $2,
               skor_kelengkapan = $3::float8::numeric,
               skor_kesesuaian = $4::float8::numeric,
               skor_keabsahan = $5::float8::numeric,
               skor_total = $6::float8::numeric,
               verified_by = $7,
               catatan_verifikator = $8,
               verified_at = now()
         WHERE id = $1
    "#;
    let result = sqlx::query(SQL)
        .bind(id)
        .bind(status)
        .bind(skor.kelengkapan)
        .bind(skor.kesesuaian)
        .bind(skor.keabsahan)
        .bind(skor.total)
        .bind(verified_by)
        .bind(catatan)
        .execute(pool)
        .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound("Output tidak ditemukan".into()));
    }
    Ok(())
}
