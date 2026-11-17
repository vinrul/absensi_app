use sqlx::PgPool;
use uuid::Uuid;

use crate::domain::enums::TitikDinasJenis;
use crate::error::{AppError, AppResult};

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct TitikDinasRow {
    pub id: Uuid,
    pub satker_id: Uuid,
    pub nama: String,
    pub jenis: TitikDinasJenis,
    pub latitude: f64,
    pub longitude: f64,
    pub radius_m: i32,
    pub is_active: bool,
}

pub async fn list(pool: &PgPool, satker_ids: &[Uuid]) -> AppResult<Vec<TitikDinasRow>> {
    const SQL: &str = r#"
        SELECT id, satker_id, nama, jenis,
               latitude::float8  AS latitude,
               longitude::float8 AS longitude,
               radius_m, is_active
          FROM titik_dinas
         WHERE deleted_at IS NULL
           AND satker_id = ANY($1::uuid[])
         ORDER BY nama
    "#;
    Ok(sqlx::query_as::<_, TitikDinasRow>(SQL)
        .bind(satker_ids)
        .fetch_all(pool)
        .await?)
}

pub async fn list_aktif_for_satker(
    pool: &PgPool,
    satker_id: Uuid,
) -> AppResult<Vec<TitikDinasRow>> {
    const SQL: &str = r#"
        SELECT id, satker_id, nama, jenis,
               latitude::float8  AS latitude,
               longitude::float8 AS longitude,
               radius_m, is_active
          FROM titik_dinas
         WHERE deleted_at IS NULL AND is_active AND satker_id = $1
         ORDER BY nama
    "#;
    Ok(sqlx::query_as::<_, TitikDinasRow>(SQL)
        .bind(satker_id)
        .fetch_all(pool)
        .await?)
}

pub async fn find(pool: &PgPool, id: Uuid) -> AppResult<Option<TitikDinasRow>> {
    const SQL: &str = r#"
        SELECT id, satker_id, nama, jenis,
               latitude::float8  AS latitude,
               longitude::float8 AS longitude,
               radius_m, is_active
          FROM titik_dinas
         WHERE id = $1 AND deleted_at IS NULL
    "#;
    Ok(sqlx::query_as::<_, TitikDinasRow>(SQL)
        .bind(id)
        .fetch_optional(pool)
        .await?)
}

#[allow(clippy::too_many_arguments)]
pub async fn create(
    pool: &PgPool,
    id: Uuid,
    satker_id: Uuid,
    nama: &str,
    jenis: TitikDinasJenis,
    latitude: f64,
    longitude: f64,
    radius_m: i32,
) -> AppResult<()> {
    const SQL: &str = r#"
        INSERT INTO titik_dinas (id, satker_id, nama, jenis, latitude, longitude, radius_m)
        VALUES ($1, $2, $3, $4, $5::float8::numeric, $6::float8::numeric, $7)
    "#;
    sqlx::query(SQL)
        .bind(id)
        .bind(satker_id)
        .bind(nama)
        .bind(jenis)
        .bind(latitude)
        .bind(longitude)
        .bind(radius_m)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn update(
    pool: &PgPool,
    id: Uuid,
    nama: Option<&str>,
    latitude: Option<f64>,
    longitude: Option<f64>,
    radius_m: Option<i32>,
    is_active: Option<bool>,
) -> AppResult<()> {
    const SQL: &str = r#"
        UPDATE titik_dinas
           SET nama = COALESCE($2, nama),
               latitude = COALESCE($3::float8::numeric, latitude),
               longitude = COALESCE($4::float8::numeric, longitude),
               radius_m = COALESCE($5, radius_m),
               is_active = COALESCE($6, is_active)
         WHERE id = $1 AND deleted_at IS NULL
    "#;
    let result = sqlx::query(SQL)
        .bind(id)
        .bind(nama)
        .bind(latitude)
        .bind(longitude)
        .bind(radius_m)
        .bind(is_active)
        .execute(pool)
        .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound("Titik dinas tidak ditemukan".into()));
    }
    Ok(())
}

pub async fn soft_delete(pool: &PgPool, id: Uuid) -> AppResult<()> {
    sqlx::query("UPDATE titik_dinas SET deleted_at = now() WHERE id = $1 AND deleted_at IS NULL")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(())
}
