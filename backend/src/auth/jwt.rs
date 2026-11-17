use axum::http::HeaderMap;
use chrono::Utc;
use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::domain::enums::{AppRole, JenisAkun, SatkerLevel};
use crate::error::{AppError, AppResult};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    pub sub: Uuid,
    #[serde(default)]
    pub nrp: Option<String>,
    #[serde(default)]
    pub username: Option<String>,
    #[serde(default)]
    pub jenis_akun: JenisAkun,
    pub nama: String,
    pub satker_id: Uuid,
    pub satker_level: SatkerLevel,
    pub roles: Vec<AppRole>,
    pub iat: i64,
    pub exp: i64,
}

impl Claims {
    pub fn has_role(&self, role: AppRole) -> bool {
        self.roles.contains(&role)
    }

    /// Admin global (super admin) yang boleh mengelola seluruh jajaran.
    pub fn is_admin_global(&self) -> bool {
        self.has_role(AppRole::AdminSistem)
    }

    /// Semua admin, termasuk admin ber-scope. Cakupan data tetap dibatasi
    /// oleh `AuthUser::satker_akses`.
    pub fn is_admin(&self) -> bool {
        self.roles.iter().any(|role| role.admin_any())
    }

    /// Semua pimpinan, termasuk pimpinan ber-scope.
    pub fn is_pimpinan(&self) -> bool {
        self.roles.iter().any(|role| role.pimpinan_any())
    }

    pub fn is_atasan(&self) -> bool {
        self.has_role(AppRole::Atasan)
    }

    pub fn require_any(&self, roles: &[AppRole]) -> AppResult<()> {
        if roles.iter().any(|role| self.has_role(*role)) {
            Ok(())
        } else {
            Err(AppError::Forbidden(
                "Anda tidak memiliki akses untuk aksi ini".into(),
            ))
        }
    }

    /// Catatan: cek cakupan satker yang lengkap dilakukan oleh `AuthUser`,
    /// yang memuat hasil pohon organisasi. Method ini hanya fallback sinkron.
    pub fn can_view_satker(&self, satker_id: Uuid) -> bool {
        self.is_admin_global() || self.satker_id == satker_id
    }

    pub fn require_satker(&self, satker_id: Uuid) -> AppResult<()> {
        if self.can_view_satker(satker_id) {
            Ok(())
        } else {
            Err(AppError::Forbidden(
                "Anda tidak berhak mengakses data satker lain".into(),
            ))
        }
    }
}

pub struct TokenSubject<'a> {
    pub id: Uuid,
    pub nrp: Option<&'a str>,
    pub username: Option<&'a str>,
    pub jenis_akun: JenisAkun,
    pub nama: &'a str,
    pub satker_id: Uuid,
    pub satker_level: SatkerLevel,
    pub roles: &'a [AppRole],
}

/// Mengembalikan (token, expires_in_detik).
pub fn issue(
    subject: TokenSubject<'_>,
    secret: &str,
    ttl_minutes: i64,
) -> AppResult<(String, i64)> {
    let now = Utc::now().timestamp();
    let expires_in = ttl_minutes * 60;

    let claims = Claims {
        sub: subject.id,
        nrp: subject.nrp.map(str::to_string),
        username: subject.username.map(str::to_string),
        jenis_akun: subject.jenis_akun,
        nama: subject.nama.to_string(),
        satker_id: subject.satker_id,
        satker_level: subject.satker_level,
        roles: subject.roles.to_vec(),
        iat: now,
        exp: now + expires_in,
    };

    let token = jsonwebtoken::encode(
        &Header::new(Algorithm::HS256),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )?;

    Ok((token, expires_in))
}

pub fn decode(token: &str, secret: &str) -> AppResult<Claims> {
    let mut validation = Validation::new(Algorithm::HS256);
    validation.leeway = 30;

    let data = jsonwebtoken::decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &validation,
    )?;

    Ok(data.claims)
}

pub fn bearer_token(headers: &HeaderMap) -> Option<String> {
    let value = headers
        .get(axum::http::header::AUTHORIZATION)?
        .to_str()
        .ok()?;
    let token = value
        .strip_prefix("Bearer ")
        .or_else(|| value.strip_prefix("bearer "))?;
    Some(token.trim().to_string())
}
