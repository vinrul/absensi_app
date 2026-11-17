//! Pengelolaan akun admin (pengelola sistem).
//!
//! Akun admin berada pada tabel `personel` dengan `jenis_akun = 'admin'`,
//! tidak memiliki NRP, dan hanya dapat login melalui dashboard web.

use std::collections::HashMap;

use axum::Json;
use axum::Router;
use axum::extract::{Path, Query, State};
use axum::routing::{get, post};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::auth::extractor::AuthUser;
use crate::auth::password;
use crate::domain::enums::{AppRole, JenisAkun, SatkerLevel};
use crate::error::{AppError, AppResult};
use crate::repo::personel::{self, AdminListItem, RoleScopeInput, RoleScopeRow};
use crate::repo::satker;
use crate::repo::{ListResponse, pagination};
use crate::routes::personel::RoleScopeDto;
use crate::service::akses;
use crate::state::AppState;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/", get(list_admin).post(create_admin))
        .route("/{id}", get(get_admin).patch(update_admin))
        .route("/{id}/reset-password", post(reset_password))
}

#[derive(Debug, Serialize)]
pub struct AdminDto {
    pub id: Uuid,
    pub username: String,
    pub nama: String,
    pub satker_id: Uuid,
    pub satker_nama: String,
    pub satker_level: SatkerLevel,
    pub is_active: bool,
    pub must_change_password: bool,
    pub role: Option<RoleScopeDto>,
}

fn dari_list(row: AdminListItem, role: Option<RoleScopeDto>) -> AdminDto {
    AdminDto {
        id: row.id,
        username: row.username,
        nama: row.nama,
        satker_id: row.satker_id,
        satker_nama: row.satker_nama,
        satker_level: row.satker_level,
        is_active: row.is_active,
        must_change_password: row.must_change_password,
        role,
    }
}

#[derive(Debug, Deserialize)]
pub struct ListQuery {
    pub page: Option<i64>,
    pub per_page: Option<i64>,
    pub q: Option<String>,
}

fn boleh_lihat_akun_admin(user: &AuthUser) -> bool {
    user.role_scopes().iter().any(|r| {
        matches!(
            r.role,
            AppRole::AdminSistem | AppRole::AdminPolda | AppRole::AdminPolres
        )
    })
}

fn super_admin(user: &AuthUser) -> bool {
    user.role_scopes()
        .iter()
        .any(|r| r.role == AppRole::AdminSistem)
}

fn role_admin_dari(scopes: &[RoleScopeRow]) -> Option<AppRole> {
    scopes.iter().map(|r| r.role).find(|r| r.admin_any())
}

fn target_boleh_dikelola(
    user: &AuthUser,
    target_id: Uuid,
    target_role: Option<AppRole>,
    satker_id: Uuid,
) -> bool {
    if super_admin(user) || user.id() == target_id {
        return true;
    }
    if !user.satker_akses().contains(&satker_id) {
        return false;
    }
    match target_role {
        Some(AppRole::AdminSatkerPolda) => user
            .role_scopes()
            .iter()
            .any(|r| r.role == AppRole::AdminPolda),
        Some(AppRole::AdminSatkerPolres) => user
            .role_scopes()
            .iter()
            .any(|r| r.role == AppRole::AdminPolres),
        _ => false,
    }
}

fn boleh_membuat_role(user: &AuthUser, role: AppRole) -> bool {
    if super_admin(user) {
        return role.admin_any();
    }
    if role == AppRole::AdminSatkerPolda {
        return user
            .role_scopes()
            .iter()
            .any(|r| r.role == AppRole::AdminPolda);
    }
    if role == AppRole::AdminSatkerPolres {
        return user
            .role_scopes()
            .iter()
            .any(|r| r.role == AppRole::AdminPolres);
    }
    false
}

fn validasi_username(username: &str) -> AppResult<()> {
    let valid = username.len() >= 3
        && username.len() <= 32
        && username
            .chars()
            .next()
            .is_some_and(|c| c.is_ascii_alphabetic())
        && username
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '_' | '.' | '-'));
    if !valid {
        return Err(AppError::Unprocessable(
            "Username 3–32 karakter, diawali huruf, hanya huruf/angka/titik/garis bawah/strip"
                .into(),
        ));
    }
    Ok(())
}

async fn detail(state: &AppState, id: Uuid) -> AppResult<AdminDto> {
    let row = personel::find_by_id(&state.pool, id)
        .await?
        .ok_or_else(|| AppError::NotFound("Akun admin tidak ditemukan".into()))?;
    if row.jenis_akun != JenisAkun::Admin {
        return Err(AppError::NotFound("Akun admin tidak ditemukan".into()));
    }
    let role = personel::role_scopes(&state.pool, id)
        .await?
        .into_iter()
        .find(|r| r.role.admin_any())
        .map(RoleScopeDto::from);
    Ok(AdminDto {
        id: row.id,
        username: row.username.unwrap_or_default(),
        nama: row.nama,
        satker_id: row.satker_id,
        satker_nama: row.satker_nama,
        satker_level: row.satker_level,
        is_active: row.is_active,
        must_change_password: row.must_change_password,
        role,
    })
}

pub async fn list_admin(
    State(state): State<AppState>,
    user: AuthUser,
    Query(query): Query<ListQuery>,
) -> AppResult<Json<ListResponse<AdminDto>>> {
    if !boleh_lihat_akun_admin(&user) {
        return Err(AppError::Forbidden(
            "Hanya admin Polda/Polres atau admin global yang dapat melihat akun admin".into(),
        ));
    }
    let (page, per_page, offset) = pagination(query.page, query.per_page);
    let q = query.q.as_deref().filter(|v| !v.is_empty());
    let (rows, total) =
        personel::list_admin(&state.pool, user.satker_akses(), q, per_page, offset).await?;

    let ids: Vec<Uuid> = rows.iter().map(|row| row.id).collect();
    let role_rows = personel::role_scopes_for_ids(&state.pool, &ids).await?;
    let mut peta: HashMap<Uuid, RoleScopeDto> = HashMap::new();
    for row in role_rows {
        if row.role.admin_any() {
            peta.entry(row.personel_id).or_insert_with(|| RoleScopeDto {
                role: row.role,
                scope_satker_id: row.scope_satker_id,
                scope_satker_nama: row.scope_satker_nama,
                scope_level: row.scope_level,
            });
        }
    }

    let data = rows
        .into_iter()
        .map(|row| {
            let role = peta.remove(&row.id);
            dari_list(row, role)
        })
        .collect();

    Ok(Json(ListResponse::new(data, page, per_page, total)))
}

pub async fn get_admin(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<Uuid>,
) -> AppResult<Json<AdminDto>> {
    let row = personel::find_by_id(&state.pool, id)
        .await?
        .ok_or_else(|| AppError::NotFound("Akun admin tidak ditemukan".into()))?;
    if row.jenis_akun != JenisAkun::Admin {
        return Err(AppError::NotFound("Akun admin tidak ditemukan".into()));
    }
    let scopes = personel::role_scopes(&state.pool, id).await?;
    if !target_boleh_dikelola(&user, id, role_admin_dari(&scopes), row.satker_id) {
        return Err(AppError::Forbidden(
            "Anda tidak berhak melihat akun admin ini".into(),
        ));
    }
    Ok(Json(detail(&state, id).await?))
}

#[derive(Debug, Deserialize)]
pub struct CreateAdminRequest {
    pub username: String,
    pub nama: String,
    pub pangkat: Option<String>,
    pub jabatan: Option<String>,
    pub satker_id: Uuid,
    pub role: AppRole,
    pub scope_satker_id: Option<Uuid>,
    pub password: Option<String>,
}

pub async fn create_admin(
    State(state): State<AppState>,
    user: AuthUser,
    Json(req): Json<CreateAdminRequest>,
) -> AppResult<Json<AdminDto>> {
    if !boleh_lihat_akun_admin(&user) {
        return Err(AppError::Forbidden(
            "Hanya admin Polda/Polres atau admin global yang dapat menambah akun admin".into(),
        ));
    }
    if !boleh_membuat_role(&user, req.role) {
        return Err(AppError::Forbidden(
            "Anda tidak berhak membuat akun admin dengan peran tersebut".into(),
        ));
    }
    if !req.role.admin_any() {
        return Err(AppError::Unprocessable(
            "Akun admin hanya boleh memakai peran admin".into(),
        ));
    }

    let username = req.username.trim().to_lowercase();
    validasi_username(&username)?;
    if personel::username_exists(&state.pool, &username).await? {
        return Err(AppError::Conflict(format!(
            "Username {username} sudah dipakai"
        )));
    }

    let nama = req.nama.trim();
    if nama.is_empty() {
        return Err(AppError::BadRequest("Nama wajib diisi".into()));
    }

    let satker_row = satker::find(&state.pool, req.satker_id)
        .await?
        .ok_or_else(|| AppError::Unprocessable("Satker tidak ditemukan".into()))?;
    if !user.satker_akses().contains(&req.satker_id) {
        return Err(AppError::Forbidden(
            "Satker akun berada di luar cakupan Anda".into(),
        ));
    }
    if req.role.butuh_scope() && req.scope_satker_id.is_none() {
        return Err(AppError::Unprocessable(
            "Peran admin ini membutuhkan cakupan satker".into(),
        ));
    }
    if !req.role.butuh_scope() && req.scope_satker_id.is_some() {
        return Err(AppError::Unprocessable(
            "Peran admin global tidak memakai cakupan satker".into(),
        ));
    }

    let roles = vec![RoleScopeInput {
        role: req.role,
        scope_satker_id: req.scope_satker_id,
    }];
    akses::validasi_role_assignments(&state.pool, user.role_scopes(), satker_row.id, &roles)
        .await?;

    let password_awal = req.password.unwrap_or_else(|| "Admin#2026".to_string());
    password::validate_strength(&password_awal)?;
    let hash = password::hash(&password_awal)?;

    let id = Uuid::now_v7();
    personel::create(
        &state.pool,
        &personel::PersonelBaru {
            id,
            jenis_akun: JenisAkun::Admin,
            nrp: None,
            username: Some(&username),
            nama,
            pangkat: req.pangkat.as_deref().unwrap_or(""),
            jabatan: req.jabatan.as_deref().unwrap_or("Administrator"),
            unit: None,
            satker_id: req.satker_id,
            atasan_id: None,
            email: None,
            phone: None,
            password_hash: &hash,
            roles: &roles,
        },
    )
    .await?;

    Ok(Json(detail(&state, id).await?))
}

#[derive(Debug, Deserialize)]
pub struct UpdateAdminRequest {
    pub username: Option<String>,
    pub nama: Option<String>,
    pub satker_id: Option<Uuid>,
    pub is_active: Option<bool>,
    pub role: Option<AppRole>,
    pub scope_satker_id: Option<Uuid>,
}

pub async fn update_admin(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateAdminRequest>,
) -> AppResult<Json<AdminDto>> {
    let existing = personel::find_by_id(&state.pool, id)
        .await?
        .ok_or_else(|| AppError::NotFound("Akun admin tidak ditemukan".into()))?;
    if existing.jenis_akun != JenisAkun::Admin {
        return Err(AppError::NotFound("Akun admin tidak ditemukan".into()));
    }
    let scopes_lama = personel::role_scopes(&state.pool, id).await?;
    let role_lama = role_admin_dari(&scopes_lama);
    if !target_boleh_dikelola(&user, id, role_lama, existing.satker_id) {
        return Err(AppError::Forbidden(
            "Anda tidak berhak mengubah akun admin ini".into(),
        ));
    }

    let username = match req.username {
        Some(ref value) => {
            let value = value.trim().to_lowercase();
            validasi_username(&value)?;
            if value != existing.username.clone().unwrap_or_default().to_lowercase()
                && personel::username_exists(&state.pool, &value).await?
            {
                return Err(AppError::Conflict(format!(
                    "Username {value} sudah dipakai"
                )));
            }
            Some(value)
        }
        None => None,
    };

    if let Some(satker_id) = req.satker_id
        && !user.satker_akses().contains(&satker_id)
    {
        return Err(AppError::Forbidden(
            "Satker akun berada di luar cakupan Anda".into(),
        ));
    }

    let roles_baru = match req.role {
        Some(role) => {
            if !boleh_membuat_role(&user, role) {
                return Err(AppError::Forbidden(
                    "Anda tidak berhak memberikan peran admin tersebut".into(),
                ));
            }
            if role.butuh_scope() && req.scope_satker_id.is_none() {
                return Err(AppError::Unprocessable(
                    "Peran admin ini membutuhkan cakupan satker".into(),
                ));
            }
            if !role.butuh_scope() && req.scope_satker_id.is_some() {
                return Err(AppError::Unprocessable(
                    "Peran admin global tidak memakai cakupan satker".into(),
                ));
            }
            let target_satker = req.satker_id.unwrap_or(existing.satker_id);
            let roles = vec![RoleScopeInput {
                role,
                scope_satker_id: req.scope_satker_id,
            }];
            akses::validasi_role_assignments(
                &state.pool,
                user.role_scopes(),
                target_satker,
                &roles,
            )
            .await?;
            Some(roles)
        }
        None => None,
    };

    let admin_global_lama = role_lama == Some(AppRole::AdminSistem);
    let tetap_global = roles_baru
        .as_ref()
        .map(|r| r.iter().any(|x| x.role == AppRole::AdminSistem))
        .unwrap_or(admin_global_lama);
    let aktif = req.is_active.unwrap_or(existing.is_active);
    if admin_global_lama
        && (!tetap_global || !aktif)
        && personel::count_admin_global(&state.pool, id).await? == 0
    {
        return Err(AppError::Unprocessable(
            "Tidak boleh menonaktifkan atau menurunkan admin global terakhir".into(),
        ));
    }

    personel::update_admin(
        &state.pool,
        id,
        username.as_deref(),
        req.nama.as_deref(),
        req.satker_id,
        req.is_active,
    )
    .await?;
    if let Some(roles) = roles_baru {
        personel::set_roles(&state.pool, id, &roles).await?;
        personel::revoke_all_refresh_tokens(&state.pool, id).await?;
    }

    Ok(Json(detail(&state, id).await?))
}

#[derive(Debug, Deserialize)]
pub struct ResetPasswordRequest {
    pub password: Option<String>,
}

pub async fn reset_password(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<Uuid>,
    Json(req): Json<ResetPasswordRequest>,
) -> AppResult<Json<serde_json::Value>> {
    let existing = personel::find_by_id(&state.pool, id)
        .await?
        .ok_or_else(|| AppError::NotFound("Akun admin tidak ditemukan".into()))?;
    if existing.jenis_akun != JenisAkun::Admin {
        return Err(AppError::NotFound("Akun admin tidak ditemukan".into()));
    }
    let scopes = personel::role_scopes(&state.pool, id).await?;
    if !target_boleh_dikelola(&user, id, role_admin_dari(&scopes), existing.satker_id) {
        return Err(AppError::Forbidden(
            "Anda tidak berhak mereset password akun admin ini".into(),
        ));
    }

    let password_baru = req.password.unwrap_or_else(|| "Admin#2026".to_string());
    password::validate_strength(&password_baru)?;
    let hash = password::hash(&password_baru)?;
    personel::reset_password(&state.pool, id, &hash).await?;
    personel::revoke_all_refresh_tokens(&state.pool, id).await?;

    Ok(Json(serde_json::json!({
        "message": "Password admin direset. Wajib diganti saat login berikutnya.",
        "password_sementara": password_baru,
    })))
}
