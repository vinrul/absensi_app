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
use crate::repo::personel::{self, PersonelListItem, RoleScopeInput, RoleScopeRow};
use crate::repo::satker;
use crate::repo::{ListResponse, pagination};
use crate::service::akses;
use crate::state::AppState;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/", get(list_personel).post(create_personel))
        .route("/{id}", get(get_personel).patch(update_personel))
        .route("/{id}/roles", get(list_roles).post(set_roles))
        .route("/{id}/reset-password", post(reset_password))
}

#[derive(Debug, Serialize)]
pub struct RoleScopeDto {
    pub role: AppRole,
    pub scope_satker_id: Option<Uuid>,
    pub scope_satker_nama: Option<String>,
    pub scope_level: Option<SatkerLevel>,
}

impl From<RoleScopeRow> for RoleScopeDto {
    fn from(row: RoleScopeRow) -> Self {
        Self {
            role: row.role,
            scope_satker_id: row.scope_satker_id,
            scope_satker_nama: row.scope_satker_nama,
            scope_level: row.scope_level,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct PersonelDto {
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
    pub roles: Vec<RoleScopeDto>,
}

fn dari_list(row: PersonelListItem, roles: Vec<RoleScopeDto>) -> PersonelDto {
    PersonelDto {
        id: row.id,
        nrp: row.nrp,
        nama: row.nama,
        pangkat: row.pangkat,
        jabatan: row.jabatan,
        unit: row.unit,
        satker_id: row.satker_id,
        satker_nama: row.satker_nama,
        satker_level: row.satker_level,
        atasan_id: row.atasan_id,
        atasan_nama: row.atasan_nama,
        email: row.email,
        phone: row.phone,
        is_active: row.is_active,
        roles,
    }
}

#[derive(Debug, Deserialize)]
pub struct ListQuery {
    pub page: Option<i64>,
    pub per_page: Option<i64>,
    pub satker_id: Option<Uuid>,
    pub unit: Option<String>,
    pub q: Option<String>,
}

/// Cakupan satker yang boleh dibaca oleh pemakai.
pub async fn scope_for(state: &AppState, user: &AuthUser) -> AppResult<Vec<Uuid>> {
    let _ = state;
    Ok(user.satker_akses().to_vec())
}

pub async fn list_personel(
    State(state): State<AppState>,
    user: AuthUser,
    Query(query): Query<ListQuery>,
) -> AppResult<Json<ListResponse<PersonelDto>>> {
    let mut scope = scope_for(&state, &user).await?;

    if let Some(filter) = query.satker_id {
        if !scope.contains(&filter) {
            return Err(AppError::Forbidden(
                "Anda tidak berhak melihat satker tersebut".into(),
            ));
        }
        scope = vec![filter];
    }

    let (page, per_page, offset) = pagination(query.page, query.per_page);
    let unit = query.unit.as_deref().filter(|v| !v.is_empty());
    let q = query.q.as_deref().filter(|v| !v.is_empty());

    // Personel biasa hanya boleh melihat dirinya sendiri.
    let hanya_admin = user.is_admin_any() || user.is_pimpinan_any() || user.is_atasan();
    let personel_id = if hanya_admin { None } else { Some(user.id()) };

    let (rows, total) =
        personel::list(&state.pool, &scope, unit, q, personel_id, per_page, offset).await?;

    let ids: Vec<Uuid> = rows.iter().map(|row| row.id).collect();
    let role_rows = personel::role_scopes_for_ids(&state.pool, &ids).await?;
    let mut peta: HashMap<Uuid, Vec<RoleScopeDto>> = HashMap::new();
    for row in role_rows {
        peta.entry(row.personel_id).or_default().push(RoleScopeDto {
            role: row.role,
            scope_satker_id: row.scope_satker_id,
            scope_satker_nama: row.scope_satker_nama,
            scope_level: row.scope_level,
        });
    }

    let data = rows
        .into_iter()
        .map(|row| {
            let roles = peta.remove(&row.id).unwrap_or_default();
            dari_list(row, roles)
        })
        .collect();

    Ok(Json(ListResponse::new(data, page, per_page, total)))
}

async fn detail(state: &AppState, id: Uuid) -> AppResult<PersonelDto> {
    let row = personel::find_by_id(&state.pool, id)
        .await?
        .ok_or_else(|| AppError::NotFound("Personel tidak ditemukan".into()))?;
    if row.jenis_akun != JenisAkun::Personel {
        return Err(AppError::NotFound("Personel tidak ditemukan".into()));
    }
    let roles = personel::role_scopes(&state.pool, id)
        .await?
        .into_iter()
        .map(RoleScopeDto::from)
        .collect();
    let atasan_nama = match row.atasan_id {
        Some(atasan_id) => personel::find_by_id(&state.pool, atasan_id)
            .await?
            .map(|p| p.nama),
        None => None,
    };

    Ok(PersonelDto {
        id: row.id,
        nrp: row.nrp.unwrap_or_default(),
        nama: row.nama,
        pangkat: row.pangkat,
        jabatan: row.jabatan,
        unit: row.unit,
        satker_id: row.satker_id,
        satker_nama: row.satker_nama,
        satker_level: row.satker_level,
        atasan_id: row.atasan_id,
        atasan_nama,
        email: row.email,
        phone: row.phone,
        is_active: row.is_active,
        roles,
    })
}

pub async fn get_personel(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<Uuid>,
) -> AppResult<Json<PersonelDto>> {
    let row = personel::find_by_id(&state.pool, id)
        .await?
        .ok_or_else(|| AppError::NotFound("Personel tidak ditemukan".into()))?;
    if row.jenis_akun != JenisAkun::Personel {
        return Err(AppError::NotFound("Personel tidak ditemukan".into()));
    }

    let boleh = akses::boleh_melihat_personel(
        user.role_scopes(),
        user.satker_akses(),
        user.id(),
        row.id,
        row.satker_id,
    );
    if !boleh {
        return Err(AppError::Forbidden(
            "Anda tidak berhak melihat data personel ini".into(),
        ));
    }

    Ok(Json(detail(&state, id).await?))
}

#[derive(Debug, Deserialize)]
pub struct CreatePersonelRequest {
    pub nrp: String,
    pub nama: String,
    pub pangkat: String,
    pub jabatan: String,
    pub satker_id: Uuid,
    pub atasan_id: Option<Uuid>,
    pub phone: Option<String>,
    pub password: Option<String>,
    pub roles: Option<Vec<RoleScopeInput>>,
}

#[derive(Debug, Deserialize)]
pub struct UpdatePersonelRequest {
    pub nama: Option<String>,
    pub pangkat: Option<String>,
    pub jabatan: Option<String>,
    pub satker_id: Option<Uuid>,
    pub atasan_id: Option<Uuid>,
    pub hapus_atasan: Option<bool>,
    pub phone: Option<String>,
    pub is_active: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct SetRolesRequest {
    pub roles: Vec<RoleScopeInput>,
}

#[derive(Debug, Deserialize)]
pub struct ResetPasswordRequest {
    pub password: Option<String>,
}

pub async fn create_personel(
    State(state): State<AppState>,
    user: AuthUser,
    Json(req): Json<CreatePersonelRequest>,
) -> AppResult<Json<PersonelDto>> {
    if !user.is_admin_any() {
        return Err(AppError::Forbidden(
            "Hanya admin yang dapat menambah personel".into(),
        ));
    }
    if !akses::boleh_kelola_personel_id(&state.pool, user.role_scopes(), req.satker_id).await? {
        return Err(AppError::Forbidden(
            "Satker tujuan berada di luar cakupan admin".into(),
        ));
    }

    let nrp = req.nrp.trim();
    let nama = req.nama.trim();
    if nrp.is_empty() || nama.is_empty() {
        return Err(AppError::BadRequest("NRP dan nama wajib diisi".into()));
    }
    if personel::find_by_nrp(&state.pool, nrp).await?.is_some() {
        return Err(AppError::Conflict(format!("NRP {nrp} sudah terdaftar")));
    }
    if satker::find(&state.pool, req.satker_id).await?.is_none() {
        return Err(AppError::Unprocessable("Satker tidak ditemukan".into()));
    }

    if let Some(atasan_id) = req.atasan_id {
        let atasan = personel::find_by_id(&state.pool, atasan_id)
            .await?
            .ok_or_else(|| AppError::Unprocessable("Atasan tidak ditemukan".into()))?;
        if atasan.satker_id != req.satker_id {
            return Err(AppError::Unprocessable(
                "Atasan langsung harus berada di satker yang sama".into(),
            ));
        }
        if !atasan.is_active {
            return Err(AppError::Unprocessable("Atasan tidak aktif".into()));
        }
    }

    let roles = req.roles.unwrap_or_else(|| {
        vec![RoleScopeInput {
            role: AppRole::Personel,
            scope_satker_id: None,
        }]
    });
    if roles.iter().any(|r| r.role.admin_any()) {
        return Err(AppError::Unprocessable(
            "Peran admin hanya dapat diberikan melalui halaman Akun Admin".into(),
        ));
    }
    akses::validasi_role_assignments(&state.pool, user.role_scopes(), req.satker_id, &roles)
        .await?;

    let password_awal = req.password.unwrap_or_else(|| "Polri#2026".to_string());
    password::validate_strength(&password_awal)?;
    let hash = password::hash(&password_awal)?;

    let id = Uuid::now_v7();
    personel::create(
        &state.pool,
        &personel::PersonelBaru {
            id,
            jenis_akun: JenisAkun::Personel,
            nrp: Some(nrp),
            username: None,
            nama,
            pangkat: req.pangkat.trim(),
            jabatan: req.jabatan.trim(),
            unit: None,
            satker_id: req.satker_id,
            atasan_id: req.atasan_id,
            email: None,
            phone: req.phone.as_deref(),
            password_hash: &hash,
            roles: &roles,
        },
    )
    .await?;

    Ok(Json(detail(&state, id).await?))
}

pub async fn update_personel(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdatePersonelRequest>,
) -> AppResult<Json<PersonelDto>> {
    let existing = personel::find_by_id(&state.pool, id)
        .await?
        .ok_or_else(|| AppError::NotFound("Personel tidak ditemukan".into()))?;
    if existing.jenis_akun != JenisAkun::Personel {
        return Err(AppError::NotFound("Personel tidak ditemukan".into()));
    }

    if !user.is_admin_any()
        || !akses::boleh_kelola_personel_id(&state.pool, user.role_scopes(), existing.satker_id)
            .await?
    {
        return Err(AppError::Forbidden(
            "Anda tidak berhak mengubah personel ini".into(),
        ));
    }

    if let Some(satker_id) = req.satker_id
        && !akses::boleh_kelola_personel_id(&state.pool, user.role_scopes(), satker_id).await?
    {
        return Err(AppError::Forbidden(
            "Satker tujuan berada di luar cakupan admin".into(),
        ));
    }

    let atasan_id = match req.atasan_id {
        Some(atasan_id) => {
            if atasan_id == id {
                return Err(AppError::Unprocessable(
                    "Personel tidak boleh menjadi atasan dirinya sendiri".into(),
                ));
            }
            let atasan = personel::find_by_id(&state.pool, atasan_id)
                .await?
                .ok_or_else(|| AppError::Unprocessable("Atasan tidak ditemukan".into()))?;
            let satker_target = req.satker_id.unwrap_or(existing.satker_id);
            if atasan.satker_id != satker_target {
                return Err(AppError::Unprocessable(
                    "Atasan langsung harus berada di satker yang sama".into(),
                ));
            }
            if !atasan.is_active {
                return Err(AppError::Unprocessable("Atasan tidak aktif".into()));
            }
            Some(atasan_id)
        }
        None => None,
    };

    personel::update(
        &state.pool,
        id,
        req.nama.as_deref(),
        req.pangkat.as_deref(),
        req.jabatan.as_deref(),
        None,
        req.satker_id,
        atasan_id,
        req.hapus_atasan.unwrap_or(false),
        None,
        req.phone.as_deref(),
        req.is_active,
    )
    .await?;

    Ok(Json(detail(&state, id).await?))
}

pub async fn list_roles(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<Uuid>,
) -> AppResult<Json<Vec<RoleScopeDto>>> {
    let row = personel::find_by_id(&state.pool, id)
        .await?
        .ok_or_else(|| AppError::NotFound("Personel tidak ditemukan".into()))?;
    if row.jenis_akun != JenisAkun::Personel {
        return Err(AppError::NotFound("Personel tidak ditemukan".into()));
    }
    if !user.is_admin_any()
        || !akses::boleh_kelola_personel_id(&state.pool, user.role_scopes(), row.satker_id).await?
    {
        return Err(AppError::Forbidden(
            "Anda tidak berhak melihat peran personel ini".into(),
        ));
    }
    Ok(Json(
        personel::role_scopes(&state.pool, id)
            .await?
            .into_iter()
            .map(RoleScopeDto::from)
            .collect(),
    ))
}

pub async fn set_roles(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<Uuid>,
    Json(req): Json<SetRolesRequest>,
) -> AppResult<Json<Vec<RoleScopeDto>>> {
    let existing = personel::find_by_id(&state.pool, id)
        .await?
        .ok_or_else(|| AppError::NotFound("Personel tidak ditemukan".into()))?;
    if existing.jenis_akun != JenisAkun::Personel {
        return Err(AppError::NotFound("Personel tidak ditemukan".into()));
    }

    if !user.is_admin_any()
        || !akses::boleh_kelola_personel_id(&state.pool, user.role_scopes(), existing.satker_id)
            .await?
    {
        return Err(AppError::Forbidden(
            "Anda tidak berhak mengubah peran personel ini".into(),
        ));
    }

    if req.roles.iter().any(|r| r.role.admin_any()) {
        return Err(AppError::Unprocessable(
            "Peran admin hanya dapat diberikan melalui halaman Akun Admin".into(),
        ));
    }

    akses::validasi_role_assignments(
        &state.pool,
        user.role_scopes(),
        existing.satker_id,
        &req.roles,
    )
    .await?;

    let admin_global_baru = req.roles.iter().any(|r| r.role == AppRole::AdminSistem);
    let admin_global_lama = personel::role_scopes(&state.pool, id)
        .await?
        .iter()
        .any(|r| r.role == AppRole::AdminSistem);
    if admin_global_lama
        && !admin_global_baru
        && personel::count_admin_global(&state.pool, id).await? == 0
    {
        return Err(AppError::Unprocessable(
            "Tidak boleh menghapus admin global terakhir".into(),
        ));
    }

    personel::set_roles(&state.pool, id, &req.roles).await?;
    personel::revoke_all_refresh_tokens(&state.pool, id).await?;

    Ok(Json(
        personel::role_scopes(&state.pool, id)
            .await?
            .into_iter()
            .map(RoleScopeDto::from)
            .collect(),
    ))
}

pub async fn reset_password(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<Uuid>,
    Json(req): Json<ResetPasswordRequest>,
) -> AppResult<Json<serde_json::Value>> {
    let existing = personel::find_by_id(&state.pool, id)
        .await?
        .ok_or_else(|| AppError::NotFound("Personel tidak ditemukan".into()))?;
    if existing.jenis_akun != JenisAkun::Personel {
        return Err(AppError::NotFound("Personel tidak ditemukan".into()));
    }

    if !user.is_admin_any()
        || !akses::boleh_kelola_personel_id(&state.pool, user.role_scopes(), existing.satker_id)
            .await?
    {
        return Err(AppError::Forbidden(
            "Anda tidak berhak mereset password personel ini".into(),
        ));
    }

    let password_baru = req.password.unwrap_or_else(|| "Polri#2026".to_string());
    password::validate_strength(&password_baru)?;
    let hash = password::hash(&password_baru)?;
    personel::reset_password(&state.pool, id, &hash).await?;
    personel::revoke_all_refresh_tokens(&state.pool, id).await?;

    Ok(Json(serde_json::json!({
        "message": "Password direset. Personel wajib mengganti password saat login berikutnya.",
        "password_sementara": password_baru,
    })))
}
