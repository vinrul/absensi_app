use chrono::NaiveDate;
use sqlx::PgPool;
use uuid::Uuid;

use crate::domain::enums::JenisLibur;
use crate::error::{AppError, AppResult};

#[derive(Debug, Clone, serde::Serialize, sqlx::FromRow)]
pub struct LiburRow {
    pub id: Uuid,
    pub tanggal: NaiveDate,
    pub nama: String,
    pub jenis: JenisLibur,
    pub satker_id: Option<Uuid>,
    pub satker_nama: Option<String>,
    pub is_active: bool,
}

pub struct LiburBaru<'a> {
    pub id: Uuid,
    pub tanggal: NaiveDate,
    pub nama: &'a str,
    pub jenis: JenisLibur,
    pub satker_id: Option<Uuid>,
    pub created_by: Uuid,
}

pub async fn list(
    pool: &PgPool,
    dari: NaiveDate,
    sampai: NaiveDate,
    satker_ids: &[Uuid],
) -> AppResult<Vec<LiburRow>> {
    const SQL: &str = r#"
        SELECT h.id, h.tanggal, h.nama, h.jenis, h.satker_id,
               s.nama AS satker_nama, h.is_active
          FROM hari_libur h
          LEFT JOIN satker s ON s.id = h.satker_id
         WHERE h.tanggal BETWEEN $1 AND $2
           AND (h.satker_id IS NULL OR h.satker_id = ANY($3::uuid[]))
         ORDER BY h.tanggal, h.nama
    "#;
    Ok(sqlx::query_as::<_, LiburRow>(SQL)
        .bind(dari)
        .bind(sampai)
        .bind(satker_ids)
        .fetch_all(pool)
        .await?)
}

pub async fn find(pool: &PgPool, id: Uuid) -> AppResult<Option<LiburRow>> {
    const SQL: &str = r#"
        SELECT h.id, h.tanggal, h.nama, h.jenis, h.satker_id,
               s.nama AS satker_nama, h.is_active
          FROM hari_libur h
          LEFT JOIN satker s ON s.id = h.satker_id
         WHERE h.id = $1
    "#;
    Ok(sqlx::query_as::<_, LiburRow>(SQL)
        .bind(id)
        .fetch_optional(pool)
        .await?)
}

pub async fn exists(
    pool: &PgPool,
    tanggal: NaiveDate,
    satker_id: Option<Uuid>,
    kecuali: Option<Uuid>,
) -> AppResult<bool> {
    Ok(sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS (
             SELECT 1 FROM hari_libur
              WHERE tanggal = $1
                AND satker_id IS NOT DISTINCT FROM $2
                AND ($3::uuid IS NULL OR id <> $3)
         )",
    )
    .bind(tanggal)
    .bind(satker_id)
    .bind(kecuali)
    .fetch_one(pool)
    .await?)
}

pub async fn create(pool: &PgPool, data: &LiburBaru<'_>) -> AppResult<()> {
    sqlx::query(
        "INSERT INTO hari_libur (id, tanggal, nama, jenis, satker_id, created_by)
         VALUES ($1, $2, $3, $4, $5, $6)",
    )
    .bind(data.id)
    .bind(data.tanggal)
    .bind(data.nama)
    .bind(data.jenis)
    .bind(data.satker_id)
    .bind(data.created_by)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn update(
    pool: &PgPool,
    id: Uuid,
    tanggal: Option<NaiveDate>,
    nama: Option<&str>,
    jenis: Option<JenisLibur>,
    is_active: Option<bool>,
) -> AppResult<()> {
    let result = sqlx::query(
        "UPDATE hari_libur
            SET tanggal = COALESCE($2, tanggal),
                nama = COALESCE($3, nama),
                jenis = COALESCE($4, jenis),
                is_active = COALESCE($5, is_active)
          WHERE id = $1",
    )
    .bind(id)
    .bind(tanggal)
    .bind(nama)
    .bind(jenis)
    .bind(is_active)
    .execute(pool)
    .await?;
    if result.rows_affected() == 0 {
        return Err(AppError::NotFound("Hari libur tidak ditemukan".into()));
    }
    Ok(())
}

pub async fn delete(pool: &PgPool, id: Uuid) -> AppResult<()> {
    let result = sqlx::query("DELETE FROM hari_libur WHERE id = $1")
        .bind(id)
        .execute(pool)
        .await?;
    if result.rows_affected() == 0 {
        return Err(AppError::NotFound("Hari libur tidak ditemukan".into()));
    }
    Ok(())
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct TanggalLiburRow {
    pub tanggal: NaiveDate,
    pub satker_id: Option<Uuid>,
}

/// Tanggal libur aktif untuk perhitungan penilaian.
pub async fn tanggal_aktif(
    pool: &PgPool,
    dari: NaiveDate,
    sampai: NaiveDate,
) -> AppResult<Vec<TanggalLiburRow>> {
    Ok(sqlx::query_as::<_, TanggalLiburRow>(
        "SELECT tanggal, satker_id
           FROM hari_libur
          WHERE is_active
            AND tanggal BETWEEN $1 AND $2
          ORDER BY tanggal",
    )
    .bind(dari)
    .bind(sampai)
    .fetch_all(pool)
    .await?)
}
