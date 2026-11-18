use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::PgPool;
use uuid::Uuid;

use crate::domain::enums::{AppRole, JenisAkun, SatkerLevel};
use crate::error::{AppError, AppResult};

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct PersonelRow {
    pub id: Uuid,
    pub nrp: Option<String>,
    pub username: Option<String>,
    pub jenis_akun: JenisAkun,
    pub nama: String,
    pub pangkat: String,
    pub jabatan: String,
    pub unit: Option<String>,
    pub satker_id: Uuid,
    pub atasan_id: Option<Uuid>,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub foto_path: Option<String>,
    pub password_hash: String,
    pub must_change_password: bool,
    pub is_active: bool,
    pub satker_level: SatkerLevel,
    pub satker_nama: String,
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct PersonelListItem {
    pub id: Uuid,
    pub nrp: String,
    pub nama: String,
    pub pangkat: String,
    pub jabatan: String,
    pub unit: Option<String>,
    pub satker_id: Uuid,
    pub satker_nama: String,
    pub satker_level: SatkerLevel,
    pub atasan_id: Option<Uuid>,
    pub atasan_nama: Option<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub is_active: bool,
}

#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct RoleScopeRow {
    pub role: AppRole,
    pub scope_satker_id: Option<Uuid>,
    pub scope_satker_nama: Option<String>,
    pub scope_level: Option<SatkerLevel>,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct RoleScopeInput {
    pub role: AppRole,
    pub scope_satker_id: Option<Uuid>,
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct PersonelRoleScopeRow {
    pub personel_id: Uuid,
    pub role: AppRole,
    pub scope_satker_id: Option<Uuid>,
    pub scope_satker_nama: Option<String>,
    pub scope_level: Option<SatkerLevel>,
}

const FIND_BY_ID: &str = r#"
    SELECT p.id, p.nrp, p.username, p.jenis_akun, p.nama, p.pangkat, p.jabatan, p.unit,
           p.satker_id, p.atasan_id, p.email, p.phone, p.foto_path, p.password_hash,
           p.must_change_password, p.is_active, s.level AS satker_level, s.nama AS satker_nama
      FROM personel p
      JOIN satker s ON s.id = p.satker_id
     WHERE p.id = $1 AND p.deleted_at IS NULL
"#;

const FIND_BY_LOGIN: &str = r#"
    SELECT p.id, p.nrp, p.username, p.jenis_akun, p.nama, p.pangkat, p.jabatan, p.unit,
           p.satker_id, p.atasan_id, p.email, p.phone, p.foto_path, p.password_hash,
           p.must_change_password, p.is_active, s.level AS satker_level, s.nama AS satker_nama
      FROM personel p
      JOIN satker s ON s.id = p.satker_id
     WHERE (lower(p.username) = lower($1) OR p.nrp = $1)
       AND p.deleted_at IS NULL
"#;

pub async fn find_by_login(pool: &PgPool, identitas: &str) -> AppResult<Option<PersonelRow>> {
    Ok(sqlx::query_as::<_, PersonelRow>(FIND_BY_LOGIN)
        .bind(identitas)
        .fetch_optional(pool)
        .await?)
}

pub async fn find_by_nrp(pool: &PgPool, nrp: &str) -> AppResult<Option<PersonelRow>> {
    const SQL: &str = r#"
        SELECT p.id, p.nrp, p.username, p.jenis_akun, p.nama, p.pangkat, p.jabatan, p.unit,
               p.satker_id, p.atasan_id, p.email, p.phone, p.foto_path, p.password_hash,
               p.must_change_password, p.is_active, s.level AS satker_level, s.nama AS satker_nama
          FROM personel p
          JOIN satker s ON s.id = p.satker_id
         WHERE p.nrp = $1 AND p.deleted_at IS NULL
    "#;
    Ok(sqlx::query_as::<_, PersonelRow>(SQL)
        .bind(nrp)
        .fetch_optional(pool)
        .await?)
}

pub async fn username_exists(pool: &PgPool, username: &str) -> AppResult<bool> {
    Ok(sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS (SELECT 1 FROM personel
                         WHERE lower(username) = lower($1) AND deleted_at IS NULL)",
    )
    .bind(username)
    .fetch_one(pool)
    .await?)
}

pub async fn find_by_id(pool: &PgPool, id: Uuid) -> AppResult<Option<PersonelRow>> {
    Ok(sqlx::query_as::<_, PersonelRow>(FIND_BY_ID)
        .bind(id)
        .fetch_optional(pool)
        .await?)
}

pub async fn role_scopes(pool: &PgPool, personel_id: Uuid) -> AppResult<Vec<RoleScopeRow>> {
    const SQL: &str = r#"
        SELECT pr.role, pr.scope_satker_id,
               s.nama AS scope_satker_nama, s.level AS scope_level
          FROM personel_role pr
          LEFT JOIN satker s ON s.id = pr.scope_satker_id
         WHERE pr.personel_id = $1
         ORDER BY pr.role
    "#;
    Ok(sqlx::query_as::<_, RoleScopeRow>(SQL)
        .bind(personel_id)
        .fetch_all(pool)
        .await?)
}

pub async fn role_scopes_for_ids(
    pool: &PgPool,
    personel_ids: &[Uuid],
) -> AppResult<Vec<PersonelRoleScopeRow>> {
    const SQL: &str = r#"
        SELECT pr.personel_id, pr.role, pr.scope_satker_id,
               s.nama AS scope_satker_nama, s.level AS scope_level
          FROM personel_role pr
          LEFT JOIN satker s ON s.id = pr.scope_satker_id
         WHERE pr.personel_id = ANY($1::uuid[])
         ORDER BY pr.personel_id, pr.role
    "#;
    Ok(sqlx::query_as::<_, PersonelRoleScopeRow>(SQL)
        .bind(personel_ids)
        .fetch_all(pool)
        .await?)
}

pub async fn roles(pool: &PgPool, personel_id: Uuid) -> AppResult<Vec<AppRole>> {
    let rows =
        sqlx::query_scalar::<_, AppRole>("SELECT role FROM personel_role WHERE personel_id = $1")
            .bind(personel_id)
            .fetch_all(pool)
            .await?;
    Ok(rows)
}

pub async fn touch_login(pool: &PgPool, id: Uuid) -> AppResult<()> {
    sqlx::query("UPDATE personel SET last_login_at = now() WHERE id = $1")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn set_password(pool: &PgPool, id: Uuid, password_hash: &str) -> AppResult<()> {
    sqlx::query(
        "UPDATE personel
            SET password_hash = $2, must_change_password = false
          WHERE id = $1",
    )
    .bind(id)
    .bind(password_hash)
    .execute(pool)
    .await?;
    Ok(())
}

/// Personel mengubah data profilnya sendiri (nama, pangkat, jabatan, telepon, email).
pub async fn update_profil(
    pool: &PgPool,
    id: Uuid,
    nama: Option<&str>,
    pangkat: Option<&str>,
    jabatan: Option<&str>,
    phone: Option<&str>,
    email: Option<&str>,
) -> AppResult<()> {
    const SQL: &str = r#"
        UPDATE personel
           SET nama = COALESCE($2, nama),
               pangkat = COALESCE($3, pangkat),
               jabatan = COALESCE($4, jabatan),
               phone = COALESCE($5, phone),
               email = COALESCE($6, email)
         WHERE id = $1 AND deleted_at IS NULL
    "#;
    let result = sqlx::query(SQL)
        .bind(id)
        .bind(nama)
        .bind(pangkat)
        .bind(jabatan)
        .bind(phone)
        .bind(email)
        .execute(pool)
        .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound("Personel tidak ditemukan".into()));
    }
    Ok(())
}

pub async fn set_foto_path(pool: &PgPool, id: Uuid, foto_path: Option<&str>) -> AppResult<()> {
    const SQL: &str = "UPDATE personel SET foto_path = $2 WHERE id = $1";
    let result = sqlx::query(SQL)
        .bind(id)
        .bind(foto_path)
        .execute(pool)
        .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound("Personel tidak ditemukan".into()));
    }
    Ok(())
}

/// Reset oleh admin: password baru wajib diganti saat login berikutnya.
pub async fn reset_password(pool: &PgPool, id: Uuid, password_hash: &str) -> AppResult<()> {
    let result = sqlx::query(
        "UPDATE personel
            SET password_hash = $2, must_change_password = true
          WHERE id = $1 AND deleted_at IS NULL",
    )
    .bind(id)
    .bind(password_hash)
    .execute(pool)
    .await?;
    if result.rows_affected() == 0 {
        return Err(AppError::NotFound("Personel tidak ditemukan".into()));
    }
    Ok(())
}

pub async fn count_admin_global(pool: &PgPool, kecuali: Uuid) -> AppResult<i64> {
    Ok(sqlx::query_scalar::<_, i64>(
        "SELECT count(DISTINCT pr.personel_id)
           FROM personel_role pr
           JOIN personel p ON p.id = pr.personel_id
          WHERE pr.role = 'admin_sistem'
            AND pr.personel_id <> $1
            AND p.deleted_at IS NULL
            AND p.is_active",
    )
    .bind(kecuali)
    .fetch_one(pool)
    .await?)
}

// ------------------------------------------------------------------- listing

pub async fn list(
    pool: &PgPool,
    satker_ids: &[Uuid],
    unit: Option<&str>,
    q: Option<&str>,
    personel_id: Option<Uuid>,
    limit: i64,
    offset: i64,
) -> AppResult<(Vec<PersonelListItem>, i64)> {
    const SELECT: &str = r#"
        SELECT p.id, p.nrp, p.nama, p.pangkat, p.jabatan, p.unit, p.satker_id,
               s.nama AS satker_nama, s.level AS satker_level, p.atasan_id,
               a.nama AS atasan_nama, p.email, p.phone, p.is_active
          FROM personel p
          JOIN satker s ON s.id = p.satker_id
          LEFT JOIN personel a ON a.id = p.atasan_id
         WHERE p.deleted_at IS NULL
           AND p.jenis_akun = 'personel'
           AND p.satker_id = ANY($1::uuid[])
           AND ($2::text IS NULL OR p.unit = $2)
           AND (
                 $3::text IS NULL
                 OR p.nama ILIKE '%' || $3 || '%'
                 OR p.nrp ILIKE '%' || $3 || '%'
               )
           AND ($4::uuid IS NULL OR p.id = $4)
         ORDER BY p.nama
         LIMIT $5 OFFSET $6
    "#;
    const COUNT: &str = r#"
        SELECT count(*)
          FROM personel p
         WHERE p.deleted_at IS NULL
           AND p.jenis_akun = 'personel'
           AND p.satker_id = ANY($1::uuid[])
           AND ($2::text IS NULL OR p.unit = $2)
           AND (
                 $3::text IS NULL
                 OR p.nama ILIKE '%' || $3 || '%'
                 OR p.nrp ILIKE '%' || $3 || '%'
               )
           AND ($4::uuid IS NULL OR p.id = $4)
    "#;

    let rows = sqlx::query_as::<_, PersonelListItem>(SELECT)
        .bind(satker_ids)
        .bind(unit)
        .bind(q)
        .bind(personel_id)
        .bind(limit)
        .bind(offset)
        .fetch_all(pool)
        .await?;

    let total: i64 = sqlx::query_scalar::<_, i64>(COUNT)
        .bind(satker_ids)
        .bind(unit)
        .bind(q)
        .bind(personel_id)
        .fetch_one(pool)
        .await?;

    Ok((rows, total))
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct AdminListItem {
    pub id: Uuid,
    pub username: String,
    pub nama: String,
    pub satker_id: Uuid,
    pub satker_nama: String,
    pub satker_level: SatkerLevel,
    pub is_active: bool,
    pub must_change_password: bool,
}

pub async fn list_admin(
    pool: &PgPool,
    satker_ids: &[Uuid],
    q: Option<&str>,
    limit: i64,
    offset: i64,
) -> AppResult<(Vec<AdminListItem>, i64)> {
    const SELECT: &str = r#"
        SELECT p.id, p.username, p.nama, p.satker_id, s.nama AS satker_nama,
               s.level AS satker_level, p.is_active, p.must_change_password
          FROM personel p
          JOIN satker s ON s.id = p.satker_id
         WHERE p.deleted_at IS NULL
           AND p.jenis_akun = 'admin'
           AND p.satker_id = ANY($1::uuid[])
           AND (
                 $2::text IS NULL
                 OR p.nama ILIKE '%' || $2 || '%'
                 OR p.username ILIKE '%' || $2 || '%'
               )
         ORDER BY p.username
         LIMIT $3 OFFSET $4
    "#;
    const COUNT: &str = r#"
        SELECT count(*)
          FROM personel p
         WHERE p.deleted_at IS NULL
           AND p.jenis_akun = 'admin'
           AND p.satker_id = ANY($1::uuid[])
           AND (
                 $2::text IS NULL
                 OR p.nama ILIKE '%' || $2 || '%'
                 OR p.username ILIKE '%' || $2 || '%'
               )
    "#;

    let rows = sqlx::query_as::<_, AdminListItem>(SELECT)
        .bind(satker_ids)
        .bind(q)
        .bind(limit)
        .bind(offset)
        .fetch_all(pool)
        .await?;
    let total: i64 = sqlx::query_scalar::<_, i64>(COUNT)
        .bind(satker_ids)
        .bind(q)
        .fetch_one(pool)
        .await?;
    Ok((rows, total))
}

pub async fn update_admin(
    pool: &PgPool,
    id: Uuid,
    username: Option<&str>,
    nama: Option<&str>,
    satker_id: Option<Uuid>,
    is_active: Option<bool>,
) -> AppResult<()> {
    const SQL: &str = r#"
        UPDATE personel
           SET username = COALESCE($2, username),
               nama = COALESCE($3, nama),
               satker_id = COALESCE($4, satker_id),
               is_active = COALESCE($5, is_active)
         WHERE id = $1 AND jenis_akun = 'admin' AND deleted_at IS NULL
    "#;
    let result = sqlx::query(SQL)
        .bind(id)
        .bind(username)
        .bind(nama)
        .bind(satker_id)
        .bind(is_active)
        .execute(pool)
        .await?;
    if result.rows_affected() == 0 {
        return Err(AppError::NotFound("Akun admin tidak ditemukan".into()));
    }
    Ok(())
}

// ---------------------------------------------------------------------- CRUD

pub struct PersonelBaru<'a> {
    pub id: Uuid,
    pub jenis_akun: JenisAkun,
    pub nrp: Option<&'a str>,
    pub username: Option<&'a str>,
    pub nama: &'a str,
    pub pangkat: &'a str,
    pub jabatan: &'a str,
    pub unit: Option<&'a str>,
    pub satker_id: Uuid,
    pub atasan_id: Option<Uuid>,
    pub email: Option<&'a str>,
    pub phone: Option<&'a str>,
    pub password_hash: &'a str,
    pub roles: &'a [RoleScopeInput],
}

pub async fn create(pool: &PgPool, data: &PersonelBaru<'_>) -> AppResult<()> {
    const SQL: &str = r#"
        INSERT INTO personel (id, jenis_akun, nrp, username, nama, pangkat, jabatan, unit,
                              satker_id, atasan_id, email, phone, password_hash,
                              must_change_password, is_active)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, true, true)
    "#;

    let mut tx = pool.begin().await?;

    sqlx::query(SQL)
        .bind(data.id)
        .bind(data.jenis_akun)
        .bind(data.nrp)
        .bind(data.username)
        .bind(data.nama)
        .bind(data.pangkat)
        .bind(data.jabatan)
        .bind(data.unit)
        .bind(data.satker_id)
        .bind(data.atasan_id)
        .bind(data.email)
        .bind(data.phone)
        .bind(data.password_hash)
        .execute(&mut *tx)
        .await?;

    for role in data.roles {
        sqlx::query(
            "INSERT INTO personel_role (id, personel_id, role, scope_satker_id)
             VALUES ($1, $2, $3, $4)
             ON CONFLICT (personel_id, role, scope_satker_id) DO NOTHING",
        )
        .bind(Uuid::now_v7())
        .bind(data.id)
        .bind(role.role)
        .bind(role.scope_satker_id)
        .execute(&mut *tx)
        .await?;
    }

    tx.commit().await?;
    Ok(())
}

#[allow(clippy::too_many_arguments)]
pub async fn update(
    pool: &PgPool,
    id: Uuid,
    nama: Option<&str>,
    pangkat: Option<&str>,
    jabatan: Option<&str>,
    unit: Option<&str>,
    satker_id: Option<Uuid>,
    atasan_id: Option<Uuid>,
    hapus_atasan: bool,
    email: Option<&str>,
    phone: Option<&str>,
    is_active: Option<bool>,
) -> AppResult<()> {
    const SQL: &str = r#"
        UPDATE personel
           SET nama = COALESCE($2, nama),
               pangkat = COALESCE($3, pangkat),
               jabatan = COALESCE($4, jabatan),
               unit = COALESCE($5, unit),
               satker_id = COALESCE($6, satker_id),
               atasan_id = CASE WHEN $11 THEN NULL ELSE COALESCE($7, atasan_id) END,
               email = COALESCE($8, email),
               phone = COALESCE($9, phone),
               is_active = COALESCE($10, is_active)
         WHERE id = $1 AND deleted_at IS NULL
    "#;

    let result = sqlx::query(SQL)
        .bind(id)
        .bind(nama)
        .bind(pangkat)
        .bind(jabatan)
        .bind(unit)
        .bind(satker_id)
        .bind(atasan_id)
        .bind(email)
        .bind(phone)
        .bind(is_active)
        .bind(hapus_atasan)
        .execute(pool)
        .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound("Personel tidak ditemukan".into()));
    }
    Ok(())
}

pub async fn set_roles(
    pool: &PgPool,
    personel_id: Uuid,
    roles: &[RoleScopeInput],
) -> AppResult<()> {
    let mut tx = pool.begin().await?;

    sqlx::query("DELETE FROM personel_role WHERE personel_id = $1")
        .bind(personel_id)
        .execute(&mut *tx)
        .await?;

    for role in roles {
        sqlx::query(
            "INSERT INTO personel_role (id, personel_id, role, scope_satker_id)
             VALUES ($1, $2, $3, $4)",
        )
        .bind(Uuid::now_v7())
        .bind(personel_id)
        .bind(role.role)
        .bind(role.scope_satker_id)
        .execute(&mut *tx)
        .await?;
    }

    tx.commit().await?;
    Ok(())
}

/// Personel yang menjadi bawahan langsung dari seorang atasan.
pub async fn bawahan_ids(pool: &PgPool, atasan_id: Uuid) -> AppResult<Vec<Uuid>> {
    Ok(sqlx::query_scalar::<_, Uuid>(
        "SELECT id FROM personel WHERE atasan_id = $1 AND deleted_at IS NULL AND jenis_akun = 'personel'",
    )
    .bind(atasan_id)
    .fetch_all(pool)
    .await?)
}

// --------------------------------------------------------------- refresh token

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct RefreshRow {
    pub id: Uuid,
    pub personel_id: Uuid,
    pub device_id: Option<String>,
    pub expires_at: DateTime<Utc>,
    pub revoked_at: Option<DateTime<Utc>>,
}

pub async fn create_refresh_token(
    pool: &PgPool,
    personel_id: Uuid,
    token_hash: &str,
    device_id: Option<&str>,
    user_agent: Option<&str>,
    expires_at: DateTime<Utc>,
) -> AppResult<Uuid> {
    let id = Uuid::now_v7();
    sqlx::query(
        "INSERT INTO refresh_token (id, personel_id, token_hash, device_id, user_agent, expires_at)
         VALUES ($1, $2, $3, $4, $5, $6)",
    )
    .bind(id)
    .bind(personel_id)
    .bind(token_hash)
    .bind(device_id)
    .bind(user_agent)
    .bind(expires_at)
    .execute(pool)
    .await?;
    Ok(id)
}

pub async fn find_refresh_token(pool: &PgPool, token_hash: &str) -> AppResult<Option<RefreshRow>> {
    Ok(sqlx::query_as::<_, RefreshRow>(
        "SELECT id, personel_id, device_id, expires_at, revoked_at
           FROM refresh_token
          WHERE token_hash = $1",
    )
    .bind(token_hash)
    .fetch_optional(pool)
    .await?)
}

pub async fn revoke_refresh_token(pool: &PgPool, id: Uuid) -> AppResult<()> {
    sqlx::query("UPDATE refresh_token SET revoked_at = now() WHERE id = $1 AND revoked_at IS NULL")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn revoke_all_refresh_tokens(pool: &PgPool, personel_id: Uuid) -> AppResult<()> {
    sqlx::query(
        "UPDATE refresh_token SET revoked_at = now()
          WHERE personel_id = $1 AND revoked_at IS NULL",
    )
    .bind(personel_id)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn active_personel(pool: &PgPool, id: Uuid) -> AppResult<PersonelRow> {
    let personel = find_by_id(pool, id)
        .await?
        .ok_or_else(|| AppError::Unauthorized("Sesi tidak valid".into()))?;
    if !personel.is_active {
        return Err(AppError::Forbidden(
            "Akun personel sudah dinonaktifkan".into(),
        ));
    }
    Ok(personel)
}
