use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use crate::error::AppResult;

#[derive(Debug, Clone, serde::Serialize, sqlx::FromRow)]
pub struct AuditRow {
    pub id: Uuid,
    pub actor_id: Option<Uuid>,
    pub actor_nama: Option<String>,
    pub aksi: String,
    pub entitas: String,
    pub entitas_id: Option<Uuid>,
    pub ip: Option<String>,
    pub user_agent: Option<String>,
    pub before_data: Option<serde_json::Value>,
    pub after_data: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
}

#[allow(clippy::too_many_arguments)]
pub async fn catat(
    pool: &PgPool,
    actor_id: Option<Uuid>,
    aksi: &str,
    entitas: &str,
    entitas_id: Option<Uuid>,
    before: Option<serde_json::Value>,
    after: Option<serde_json::Value>,
    ip: Option<&str>,
    user_agent: Option<&str>,
) -> AppResult<()> {
    const SQL: &str = r#"
        INSERT INTO audit_log (id, actor_id, aksi, entitas, entitas_id, before_data, after_data,
                               ip, user_agent)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
    "#;

    sqlx::query(SQL)
        .bind(Uuid::now_v7())
        .bind(actor_id)
        .bind(aksi)
        .bind(entitas)
        .bind(entitas_id)
        .bind(before)
        .bind(after)
        .bind(ip)
        .bind(user_agent)
        .execute(pool)
        .await?;
    Ok(())
}

pub struct ListFilter {
    pub entitas: Option<String>,
    pub actor_id: Option<Uuid>,
    pub dari: Option<chrono::NaiveDate>,
    pub sampai: Option<chrono::NaiveDate>,
    pub limit: i64,
    pub offset: i64,
}

pub async fn list(pool: &PgPool, filter: &ListFilter) -> AppResult<(Vec<AuditRow>, i64)> {
    const SQL: &str = r#"
        SELECT a.id, a.actor_id, p.nama AS actor_nama, a.aksi, a.entitas, a.entitas_id,
               a.ip, a.user_agent, a.before_data, a.after_data, a.created_at
          FROM audit_log a
          LEFT JOIN personel p ON p.id = a.actor_id
         WHERE ($1::text IS NULL OR a.entitas = $1)
           AND ($2::uuid IS NULL OR a.actor_id = $2)
           AND ($3::date IS NULL OR a.created_at >= $3)
           AND ($4::date IS NULL OR a.created_at < ($4::date + 1))
         ORDER BY a.created_at DESC
         LIMIT $5 OFFSET $6
    "#;

    const COUNT: &str = r#"
        SELECT count(*)
          FROM audit_log a
         WHERE ($1::text IS NULL OR a.entitas = $1)
           AND ($2::uuid IS NULL OR a.actor_id = $2)
           AND ($3::date IS NULL OR a.created_at >= $3)
           AND ($4::date IS NULL OR a.created_at < ($4::date + 1))
    "#;

    let rows = sqlx::query_as::<_, AuditRow>(SQL)
        .bind(filter.entitas.as_deref())
        .bind(filter.actor_id)
        .bind(filter.dari)
        .bind(filter.sampai)
        .bind(filter.limit)
        .bind(filter.offset)
        .fetch_all(pool)
        .await?;

    let total: i64 = sqlx::query_scalar::<_, i64>(COUNT)
        .bind(filter.entitas.as_deref())
        .bind(filter.actor_id)
        .bind(filter.dari)
        .bind(filter.sampai)
        .fetch_one(pool)
        .await?;

    Ok((rows, total))
}
