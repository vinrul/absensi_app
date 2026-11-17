use axum::Json;
use axum::Router;
use axum::extract::State;
use axum::http::{HeaderMap, HeaderValue, header::SET_COOKIE};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use chrono::{Duration, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::auth::extractor::AuthUser;
use crate::auth::{self, jwt, password};
use crate::domain::enums::{AppRole, JenisAkun, KanalLogin, SatkerLevel};
use crate::error::{AppError, AppResult};
use crate::repo::personel::{self, PersonelRow};
use crate::routes::personel::RoleScopeDto;
use crate::service::akses;
use crate::state::AppState;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/login", post(login))
        .route("/refresh", post(refresh))
        .route("/logout", post(logout))
        .route("/me", get(me))
        .route("/change-password", post(change_password))
}

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    /// NRP untuk personel (termasuk pimpinan/atasan).
    pub nrp: Option<String>,
    /// Username untuk akun admin.
    pub username: Option<String>,
    pub password: String,
    pub device_id: Option<String>,
    /// `web` (default) atau `android`.
    pub client: Option<KanalLogin>,
}

#[derive(Debug, Serialize)]
pub struct UserDto {
    pub id: Uuid,
    pub nrp: Option<String>,
    pub username: Option<String>,
    pub jenis_akun: JenisAkun,
    pub nama: String,
    pub pangkat: String,
    pub jabatan: String,
    pub satker_id: Uuid,
    pub satker_nama: String,
    pub satker_level: SatkerLevel,
    pub roles: Vec<AppRole>,
    pub role_scopes: Vec<RoleScopeDto>,
    pub must_change_password: bool,
    pub atasan_id: Option<Uuid>,
}

#[derive(Debug, Serialize)]
pub struct LoginResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub token_type: &'static str,
    pub expires_in: i64,
    pub user: UserDto,
}

fn user_dto(
    personel: &PersonelRow,
    roles: Vec<AppRole>,
    role_scopes: Vec<RoleScopeDto>,
) -> UserDto {
    UserDto {
        id: personel.id,
        nrp: personel.nrp.clone(),
        username: personel.username.clone(),
        jenis_akun: personel.jenis_akun,
        nama: personel.nama.clone(),
        pangkat: personel.pangkat.clone(),
        jabatan: personel.jabatan.clone(),
        satker_id: personel.satker_id,
        satker_nama: personel.satker_nama.clone(),
        satker_level: personel.satker_level,
        roles,
        role_scopes,
        must_change_password: personel.must_change_password,
        atasan_id: personel.atasan_id,
    }
}

fn with_cookie(mut response: Response, cookie: String) -> AppResult<Response> {
    let value = HeaderValue::from_str(&cookie)
        .map_err(|_| AppError::Internal("cookie tidak valid".into()))?;
    response.headers_mut().insert(SET_COOKIE, value);
    Ok(response)
}

async fn issue_session(
    state: &AppState,
    personel: &PersonelRow,
    roles: &[AppRole],
    device_id: Option<&str>,
    user_agent: Option<&str>,
) -> AppResult<(String, i64, String)> {
    let (access_token, expires_in) = jwt::issue(
        jwt::TokenSubject {
            id: personel.id,
            nrp: personel.nrp.as_deref(),
            username: personel.username.as_deref(),
            jenis_akun: personel.jenis_akun,
            nama: &personel.nama,
            satker_id: personel.satker_id,
            satker_level: personel.satker_level,
            roles,
        },
        &state.settings.jwt_secret,
        state.settings.access_token_ttl_minutes,
    )?;

    let refresh_raw = auth::generate_refresh_token();
    let expires_at = Utc::now() + Duration::days(state.settings.refresh_token_ttl_days);

    personel::create_refresh_token(
        &state.pool,
        personel.id,
        &auth::hash_token(&refresh_raw),
        device_id,
        user_agent,
        expires_at,
    )
    .await?;

    Ok((access_token, expires_in, refresh_raw))
}

pub async fn login(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(req): Json<LoginRequest>,
) -> AppResult<Response> {
    let identitas = req
        .username
        .as_deref()
        .map(str::trim)
        .filter(|v| !v.is_empty())
        .or_else(|| req.nrp.as_deref().map(str::trim).filter(|v| !v.is_empty()));
    let Some(identitas) = identitas else {
        return Err(AppError::BadRequest(
            "Username/NRP dan password wajib diisi".into(),
        ));
    };
    if req.password.is_empty() {
        return Err(AppError::BadRequest(
            "Username/NRP dan password wajib diisi".into(),
        ));
    }

    let personel = personel::find_by_login(&state.pool, identitas)
        .await?
        .ok_or_else(|| AppError::Unauthorized("Username/NRP atau password salah".into()))?;

    if !personel.is_active {
        return Err(AppError::Forbidden("Akun sudah dinonaktifkan".into()));
    }

    if !password::verify(&req.password, &personel.password_hash) {
        return Err(AppError::Unauthorized(
            "Username/NRP atau password salah".into(),
        ));
    }

    let kanal = req.client.unwrap_or_default();
    if !akses::kanal_login_diizinkan(personel.jenis_akun, kanal) {
        return Err(AppError::Forbidden(
            "Akun admin hanya dapat masuk melalui dashboard web".into(),
        ));
    }

    let roles = personel::roles(&state.pool, personel.id).await?;
    if roles.is_empty() {
        return Err(AppError::Forbidden("Akun belum memiliki peran".into()));
    }
    let role_scopes = personel::role_scopes(&state.pool, personel.id).await?;

    let user_agent = headers
        .get(axum::http::header::USER_AGENT)
        .and_then(|v| v.to_str().ok())
        .map(|v| v.to_string());

    let (access_token, expires_in, refresh_raw) = issue_session(
        &state,
        &personel,
        &roles,
        req.device_id.as_deref(),
        user_agent.as_deref(),
    )
    .await?;

    personel::touch_login(&state.pool, personel.id).await?;
    tracing::info!(
        identitas = %identitas,
        kanal = ?kanal,
        "login berhasil"
    );

    let cookie = auth::refresh_cookie(
        &refresh_raw,
        state.settings.refresh_token_ttl_days * 86_400,
        state.settings.cookie_secure,
    );

    let body = LoginResponse {
        access_token,
        refresh_token: refresh_raw.clone(),
        token_type: "Bearer",
        expires_in,
        user: user_dto(
            &personel,
            roles,
            role_scopes.into_iter().map(RoleScopeDto::from).collect(),
        ),
    };

    with_cookie(Json(body).into_response(), cookie)
}

#[derive(Debug, Serialize)]
pub struct TokenResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub token_type: &'static str,
    pub expires_in: i64,
}

#[derive(Debug, Deserialize)]
pub struct RefreshRequest {
    /// Klien mobile mengirim refresh token lewat body; browser memakai cookie.
    pub refresh_token: Option<String>,
}

pub async fn refresh(
    State(state): State<AppState>,
    headers: HeaderMap,
    payload: Option<Json<RefreshRequest>>,
) -> AppResult<Response> {
    let raw = payload
        .and_then(|Json(body)| body.refresh_token)
        .filter(|token| !token.is_empty())
        .or_else(|| auth::read_cookie(&headers, auth::REFRESH_COOKIE))
        .ok_or_else(|| AppError::Unauthorized("Sesi tidak ditemukan".into()))?;

    let stored = personel::find_refresh_token(&state.pool, &auth::hash_token(&raw))
        .await?
        .ok_or_else(|| AppError::Unauthorized("Sesi tidak valid".into()))?;

    if stored.revoked_at.is_some() || stored.expires_at <= Utc::now() {
        return Err(AppError::Unauthorized("Sesi sudah berakhir".into()));
    }

    let personel = personel::active_personel(&state.pool, stored.personel_id).await?;
    let roles = personel::roles(&state.pool, personel.id).await?;

    let (access_token, expires_in) = jwt::issue(
        jwt::TokenSubject {
            id: personel.id,
            nrp: personel.nrp.as_deref(),
            username: personel.username.as_deref(),
            jenis_akun: personel.jenis_akun,
            nama: &personel.nama,
            satker_id: personel.satker_id,
            satker_level: personel.satker_level,
            roles: &roles,
        },
        &state.settings.jwt_secret,
        state.settings.access_token_ttl_minutes,
    )?;

    // Rotasi token: token lama dicabut, token baru diterbitkan.
    personel::revoke_refresh_token(&state.pool, stored.id).await?;
    let new_raw = auth::generate_refresh_token();
    personel::create_refresh_token(
        &state.pool,
        personel.id,
        &auth::hash_token(&new_raw),
        None,
        None,
        Utc::now() + Duration::days(state.settings.refresh_token_ttl_days),
    )
    .await?;

    let cookie = auth::refresh_cookie(
        &new_raw,
        state.settings.refresh_token_ttl_days * 86_400,
        state.settings.cookie_secure,
    );

    with_cookie(
        Json(TokenResponse {
            access_token,
            refresh_token: new_raw.clone(),
            token_type: "Bearer",
            expires_in,
        })
        .into_response(),
        cookie,
    )
}

pub async fn logout(State(state): State<AppState>, headers: HeaderMap) -> AppResult<Response> {
    if let Some(raw) = auth::read_cookie(&headers, auth::REFRESH_COOKIE)
        && let Some(stored) =
            personel::find_refresh_token(&state.pool, &auth::hash_token(&raw)).await?
    {
        personel::revoke_refresh_token(&state.pool, stored.id).await?;
    }

    with_cookie(
        Json(serde_json::json!({ "message": "Berhasil keluar" })).into_response(),
        auth::clear_refresh_cookie(),
    )
}

pub async fn me(State(state): State<AppState>, user: AuthUser) -> AppResult<Json<UserDto>> {
    let personel = personel::active_personel(&state.pool, user.id()).await?;
    let roles = personel::roles(&state.pool, personel.id).await?;
    let role_scopes = personel::role_scopes(&state.pool, personel.id).await?;
    Ok(Json(user_dto(
        &personel,
        roles,
        role_scopes.into_iter().map(RoleScopeDto::from).collect(),
    )))
}

#[derive(Debug, Deserialize)]
pub struct ChangePasswordRequest {
    pub old_password: String,
    pub new_password: String,
}

pub async fn change_password(
    State(state): State<AppState>,
    user: AuthUser,
    Json(req): Json<ChangePasswordRequest>,
) -> AppResult<Response> {
    let personel = personel::active_personel(&state.pool, user.id()).await?;

    if !password::verify(&req.old_password, &personel.password_hash) {
        return Err(AppError::Unauthorized("Password lama salah".into()));
    }
    password::validate_strength(&req.new_password)?;

    let new_hash = password::hash(&req.new_password)?;
    personel::set_password(&state.pool, personel.id, &new_hash).await?;
    personel::revoke_all_refresh_tokens(&state.pool, personel.id).await?;

    with_cookie(
        Json(serde_json::json!({
            "message": "Password berhasil diubah, silakan login kembali"
        }))
        .into_response(),
        auth::clear_refresh_cookie(),
    )
}
