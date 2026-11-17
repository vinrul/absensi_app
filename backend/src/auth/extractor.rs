use axum::extract::FromRequestParts;
use axum::http::request::Parts;
use uuid::Uuid;

use crate::auth::jwt::{self, Claims};
use crate::domain::enums::AppRole;
use crate::error::{AppError, AppResult};
use crate::repo::personel::{self, RoleScopeRow};
use crate::service::akses;
use crate::state::AppState;

pub struct AuthUser {
    claims: Claims,
    role_scopes: Vec<RoleScopeRow>,
    satker_akses: Vec<Uuid>,
}

impl AuthUser {
    pub fn claims(&self) -> &Claims {
        &self.claims
    }

    pub fn id(&self) -> Uuid {
        self.claims.sub
    }

    pub fn role_scopes(&self) -> &[RoleScopeRow] {
        &self.role_scopes
    }

    pub fn satker_akses(&self) -> &[Uuid] {
        &self.satker_akses
    }

    pub fn is_admin_global(&self) -> bool {
        self.claims.is_admin_global()
    }

    pub fn is_admin_any(&self) -> bool {
        self.role_scopes.iter().any(|r| r.role.admin_any())
    }

    pub fn is_pimpinan_any(&self) -> bool {
        self.role_scopes.iter().any(|r| r.role.pimpinan_any())
    }

    pub fn is_atasan(&self) -> bool {
        self.role_scopes.iter().any(|r| r.role == AppRole::Atasan)
    }

    pub fn require_satker(&self, satker_id: Uuid) -> AppResult<()> {
        if self.satker_akses.contains(&satker_id) {
            Ok(())
        } else {
            Err(AppError::Forbidden(
                "Anda tidak berhak mengakses data satker lain".into(),
            ))
        }
    }

    pub fn require_admin_global(&self) -> AppResult<()> {
        if self.is_admin_global() {
            Ok(())
        } else {
            Err(AppError::Forbidden(
                "Aksi ini hanya untuk admin global".into(),
            ))
        }
    }
}

impl FromRequestParts<AppState> for AuthUser {
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &AppState) -> AppResult<Self> {
        let token = jwt::bearer_token(&parts.headers)
            .ok_or_else(|| AppError::Unauthorized("Header Authorization tidak ditemukan".into()))?;

        let claims = jwt::decode(&token, &state.settings.jwt_secret)?;
        let role_scopes = personel::role_scopes(&state.pool, claims.sub).await?;
        let satker_akses =
            akses::hitung_satker_akses(&state.pool, claims.sub, &role_scopes).await?;
        Ok(AuthUser {
            claims,
            role_scopes,
            satker_akses,
        })
    }
}

/// Opsional: dipakai pada endpoint yang boleh diakses tanpa login tetapi
/// memberi perilaku berbeda bila sesi ada.
pub struct MaybeAuthUser(pub Option<Claims>);

impl FromRequestParts<AppState> for MaybeAuthUser {
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &AppState) -> AppResult<Self> {
        let Some(token) = jwt::bearer_token(&parts.headers) else {
            return Ok(MaybeAuthUser(None));
        };
        Ok(MaybeAuthUser(
            jwt::decode(&token, &state.settings.jwt_secret).ok(),
        ))
    }
}
