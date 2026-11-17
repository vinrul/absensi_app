use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use crate::error::AppResult;

#[derive(Debug, Clone, serde::Serialize, sqlx::FromRow)]
pub struct NotifikasiRow {
    pub id: Uuid,
    pub tipe: String,
    pub judul: String,
    pub body: String,
    pub ref_type: Option<String>,
    pub ref_id: Option<Uuid>,
    pub read_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

#[allow(clippy::too_many_arguments)]
pub async fn kirim(
    pool: &PgPool,
    personel_id: Uuid,
    tipe: &str,
    judul: &str,
    body: &str,
    ref_type: Option<&str>,
    ref_id: Option<Uuid>,
) -> AppResult<()> {
    const SQL: &str = r#"
        INSERT INTO notifikasi (id, personel_id, tipe, judul, body, ref_type, ref_id)
        VALUES ($1, $2, $3, $4, $5, $6, $7)
    "#;
    sqlx::query(SQL)
        .bind(Uuid::now_v7())
        .bind(personel_id)
        .bind(tipe)
        .bind(judul)
        .bind(body)
        .bind(ref_type)
        .bind(ref_id)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn list(
    pool: &PgPool,
    personel_id: Uuid,
    hanya_belum_dibaca: bool,
) -> AppResult<Vec<NotifikasiRow>> {
    const SQL: &str = r#"
        SELECT id, tipe, judul, body, ref_type, ref_id, read_at, created_at
          FROM notifikasi
         WHERE personel_id = $1
           AND (NOT $2::bool OR read_at IS NULL)
         ORDER BY created_at DESC
         LIMIT 100
    "#;
    Ok(sqlx::query_as::<_, NotifikasiRow>(SQL)
        .bind(personel_id)
        .bind(hanya_belum_dibaca)
        .fetch_all(pool)
        .await?)
}

pub async fn tandai_dibaca(pool: &PgPool, id: Uuid, personel_id: Uuid) -> AppResult<()> {
    sqlx::query(
        "UPDATE notifikasi SET read_at = now()
          WHERE id = $1 AND personel_id = $2 AND read_at IS NULL",
    )
    .bind(id)
    .bind(personel_id)
    .execute(pool)
    .await?;
    Ok(())
}
