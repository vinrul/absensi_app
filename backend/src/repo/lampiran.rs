use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use crate::domain::enums::{BerkasKategori, BerkasOwner};
use crate::error::{AppError, AppResult};

#[derive(Debug, Clone, serde::Serialize, sqlx::FromRow)]
pub struct LampiranRow {
    pub id: Uuid,
    pub owner_type: BerkasOwner,
    pub owner_id: Uuid,
    pub kategori: BerkasKategori,
    pub file_name: String,
    pub mime: String,
    pub size_bytes: i64,
    pub storage_key: String,
    pub created_at: DateTime<Utc>,
}

pub async fn list(
    pool: &PgPool,
    owner_type: BerkasOwner,
    owner_id: Uuid,
) -> AppResult<Vec<LampiranRow>> {
    const SQL: &str = r#"
        SELECT id, owner_type, owner_id, kategori, file_name, mime, size_bytes, storage_key,
               created_at
          FROM lampiran
         WHERE owner_type = $1 AND owner_id = $2
         ORDER BY created_at
    "#;
    Ok(sqlx::query_as::<_, LampiranRow>(SQL)
        .bind(owner_type)
        .bind(owner_id)
        .fetch_all(pool)
        .await?)
}

pub async fn count(pool: &PgPool, owner_type: BerkasOwner, owner_id: Uuid) -> AppResult<i64> {
    const SQL: &str = "SELECT count(*) FROM lampiran WHERE owner_type = $1 AND owner_id = $2";
    Ok(sqlx::query_scalar::<_, i64>(SQL)
        .bind(owner_type)
        .bind(owner_id)
        .fetch_one(pool)
        .await?)
}

pub async fn find(pool: &PgPool, id: Uuid) -> AppResult<Option<LampiranRow>> {
    const SQL: &str = r#"
        SELECT id, owner_type, owner_id, kategori, file_name, mime, size_bytes, storage_key,
               created_at
          FROM lampiran
         WHERE id = $1
    "#;
    Ok(sqlx::query_as::<_, LampiranRow>(SQL)
        .bind(id)
        .fetch_optional(pool)
        .await?)
}

pub async fn find_by_storage_key(pool: &PgPool, key: &str) -> AppResult<Option<LampiranRow>> {
    const SQL: &str = r#"
        SELECT id, owner_type, owner_id, kategori, file_name, mime, size_bytes, storage_key,
               created_at
          FROM lampiran
         WHERE storage_key = $1
    "#;
    Ok(sqlx::query_as::<_, LampiranRow>(SQL)
        .bind(key)
        .fetch_optional(pool)
        .await?)
}

pub struct LampiranBaru<'a> {
    pub owner_type: BerkasOwner,
    pub owner_id: Uuid,
    pub kategori: BerkasKategori,
    pub file_name: &'a str,
    pub mime: &'a str,
    pub size_bytes: i64,
    pub storage_key: &'a str,
    pub uploaded_by: Uuid,
}

pub async fn create(pool: &PgPool, data: &LampiranBaru<'_>) -> AppResult<Uuid> {
    const SQL: &str = r#"
        INSERT INTO lampiran (id, owner_type, owner_id, kategori, file_name, mime, size_bytes,
                              storage_key, uploaded_by)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
    "#;
    let id = Uuid::now_v7();
    sqlx::query(SQL)
        .bind(id)
        .bind(data.owner_type)
        .bind(data.owner_id)
        .bind(data.kategori)
        .bind(data.file_name)
        .bind(data.mime)
        .bind(data.size_bytes)
        .bind(data.storage_key)
        .bind(data.uploaded_by)
        .execute(pool)
        .await?;
    Ok(id)
}

pub async fn delete(pool: &PgPool, id: Uuid) -> AppResult<()> {
    let result = sqlx::query("DELETE FROM lampiran WHERE id = $1")
        .bind(id)
        .execute(pool)
        .await?;
    if result.rows_affected() == 0 {
        return Err(AppError::NotFound("Lampiran tidak ditemukan".into()));
    }
    Ok(())
}
