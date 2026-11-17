use chrono::NaiveTime;
use sqlx::PgPool;
use uuid::Uuid;

use crate::domain::enums::SatkerLevel;
use crate::error::AppResult;

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct SatkerRow {
    pub id: Uuid,
    pub parent_id: Option<Uuid>,
    pub level: SatkerLevel,
    pub nama: String,
    pub kode: String,
    pub alamat: Option<String>,
    pub timezone: String,
    pub hari_kerja: Vec<i16>,
    pub jam_masuk: NaiveTime,
    pub jam_pulang: NaiveTime,
    pub jam_pulang_sabtu: Option<NaiveTime>,
    pub is_active: bool,
}

pub async fn list(pool: &PgPool) -> AppResult<Vec<SatkerRow>> {
    const SQL: &str = r#"
        SELECT id, parent_id, level, nama, kode, alamat, timezone, hari_kerja,
               jam_masuk, jam_pulang, jam_pulang_sabtu, is_active
          FROM satker
         WHERE deleted_at IS NULL
         ORDER BY level, nama
    "#;
    Ok(sqlx::query_as::<_, SatkerRow>(SQL).fetch_all(pool).await?)
}

pub async fn list_by_ids(pool: &PgPool, ids: &[Uuid]) -> AppResult<Vec<SatkerRow>> {
    const SQL: &str = r#"
        SELECT id, parent_id, level, nama, kode, alamat, timezone, hari_kerja,
               jam_masuk, jam_pulang, jam_pulang_sabtu, is_active
          FROM satker
         WHERE deleted_at IS NULL AND id = ANY($1::uuid[])
         ORDER BY level, nama
    "#;
    Ok(sqlx::query_as::<_, SatkerRow>(SQL)
        .bind(ids)
        .fetch_all(pool)
        .await?)
}

pub async fn find_by_kode(pool: &PgPool, kode: &str) -> AppResult<Option<SatkerRow>> {
    const SQL: &str = r#"
        SELECT id, parent_id, level, nama, kode, alamat, timezone, hari_kerja,
               jam_masuk, jam_pulang, jam_pulang_sabtu, is_active
          FROM satker
         WHERE kode = $1 AND deleted_at IS NULL
    "#;
    Ok(sqlx::query_as::<_, SatkerRow>(SQL)
        .bind(kode)
        .fetch_optional(pool)
        .await?)
}

pub async fn find(pool: &PgPool, id: Uuid) -> AppResult<Option<SatkerRow>> {
    const SQL: &str = r#"
        SELECT id, parent_id, level, nama, kode, alamat, timezone, hari_kerja,
               jam_masuk, jam_pulang, jam_pulang_sabtu, is_active
          FROM satker
         WHERE id = $1 AND deleted_at IS NULL
    "#;
    Ok(sqlx::query_as::<_, SatkerRow>(SQL)
        .bind(id)
        .fetch_optional(pool)
        .await?)
}

#[allow(clippy::too_many_arguments)]
pub async fn create(
    pool: &PgPool,
    id: Uuid,
    parent_id: Option<Uuid>,
    level: SatkerLevel,
    nama: &str,
    kode: &str,
    alamat: Option<&str>,
    timezone: &str,
    hari_kerja: &[i16],
    jam_masuk: NaiveTime,
    jam_pulang: NaiveTime,
    jam_pulang_sabtu: Option<NaiveTime>,
) -> AppResult<()> {
    const SQL: &str = r#"
        INSERT INTO satker (id, parent_id, level, nama, kode, alamat, timezone,
                            hari_kerja, jam_masuk, jam_pulang, jam_pulang_sabtu)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
    "#;
    sqlx::query(SQL)
        .bind(id)
        .bind(parent_id)
        .bind(level)
        .bind(nama)
        .bind(kode)
        .bind(alamat)
        .bind(timezone)
        .bind(hari_kerja)
        .bind(jam_masuk)
        .bind(jam_pulang)
        .bind(jam_pulang_sabtu)
        .execute(pool)
        .await?;
    Ok(())
}

#[allow(clippy::too_many_arguments)]
pub async fn update(
    pool: &PgPool,
    id: Uuid,
    nama: Option<&str>,
    kode: Option<&str>,
    alamat: Option<&str>,
    timezone: Option<&str>,
    hari_kerja: Option<&[i16]>,
    is_active: Option<bool>,
    jam_masuk: Option<NaiveTime>,
    jam_pulang: Option<NaiveTime>,
    jam_pulang_sabtu: Option<NaiveTime>,
    hapus_jam_pulang_sabtu: bool,
) -> AppResult<()> {
    const SQL: &str = r#"
        UPDATE satker
           SET nama = COALESCE($2, nama),
               kode = COALESCE($3, kode),
               alamat = COALESCE($4, alamat),
               timezone = COALESCE($5, timezone),
               hari_kerja = COALESCE($6, hari_kerja),
               is_active = COALESCE($7, is_active),
               jam_masuk = COALESCE($8, jam_masuk),
               jam_pulang = COALESCE($9, jam_pulang),
               jam_pulang_sabtu = CASE WHEN $11 THEN NULL ELSE COALESCE($10, jam_pulang_sabtu) END
         WHERE id = $1 AND deleted_at IS NULL
    "#;
    sqlx::query(SQL)
        .bind(id)
        .bind(nama)
        .bind(kode)
        .bind(alamat)
        .bind(timezone)
        .bind(hari_kerja)
        .bind(is_active)
        .bind(jam_masuk)
        .bind(jam_pulang)
        .bind(jam_pulang_sabtu)
        .bind(hapus_jam_pulang_sabtu)
        .execute(pool)
        .await?;
    Ok(())
}

/// Satker yang boleh dilihat oleh pemakai dengan satker & level tertentu.
pub async fn visible_ids(pool: &PgPool, satker_id: Uuid, semua: bool) -> AppResult<Vec<Uuid>> {
    if semua {
        return Ok(
            sqlx::query_scalar::<_, Uuid>("SELECT id FROM satker WHERE deleted_at IS NULL")
                .fetch_all(pool)
                .await?,
        );
    }

    const SQL: &str = r#"
        SELECT id FROM satker
         WHERE deleted_at IS NULL
           AND (id = $1 OR parent_id = $1)
    "#;
    Ok(sqlx::query_scalar::<_, Uuid>(SQL)
        .bind(satker_id)
        .fetch_all(pool)
        .await?)
}
