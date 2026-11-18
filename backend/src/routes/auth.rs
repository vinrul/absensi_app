use axum::Json;
use axum::Router;
use axum::body::Body;
use axum::extract::{Multipart, Path, State};
use axum::http::{HeaderMap, HeaderValue, StatusCode, header, header::SET_COOKIE};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, patch, post};
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
use crate::service::storage;
use crate::state::AppState;

/// Pangkat Polri dari terendah ke tertinggi (selaras dengan combobox dashboard).
const PANGKAT_POLRI: &[&str] = &[
    "BHARADA",
    "BHARATU",
    "BHARAKA",
    "ABRIPDA",
    "ABRIPTU",
    "ABRIPKA",
    "BRIPDA",
    "BRIPTU",
    "BRIPKA",
    "AIPDA",
    "AIPTU",
    "IPDA",
    "IPTU",
    "AKP",
    "KOMPOL",
    "AKBP",
    "KOMBES POL",
    "BRIGJEN POL",
    "IRJEN POL",
    "KOMJEN POL",
    "JENDERAL POL",
];

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/login", post(login))
        .route("/refresh", post(refresh))
        .route("/logout", post(logout))
        .route("/me", get(me))
        .route("/profile", patch(update_profile))
        .route("/foto", post(upload_foto))
        .route("/foto/{id}", get(foto))
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
    pub email: Option<String>,
    pub phone: Option<String>,
    pub foto_url: Option<String>,
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
        email: personel.email.clone(),
        phone: personel.phone.clone(),
        foto_url: personel
            .foto_path
            .as_ref()
            .map(|_| format!("auth/foto/{}", personel.id)),
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
    if matches!(kanal, KanalLogin::Web) && !akses::boleh_login_dashboard(&roles) {
        return Err(AppError::Forbidden(
            "Dashboard hanya untuk akun pimpinan dan admin. Personel dan atasan langsung menggunakan aplikasi Android."
                .into(),
        ));
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
    let lewat_body = payload.is_some();
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

    // Cookie refresh dipakai dashboard web dan web personel; token lewat body dipakai Android.
    if !lewat_body && !akses::refresh_cookie_diizinkan(&roles, stored.device_id.as_deref()) {
        return Err(AppError::Forbidden(
            "Sesi web ini tidak diizinkan. Silakan login melalui aplikasi yang sesuai.".into(),
        ));
    }

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

    // Rotasi token: token lama dicabut, token baru diterbitkan dengan penanda perangkat yang sama
    // agar refresh cookie web personel tetap dikenali pada rotasi berikutnya.
    personel::revoke_refresh_token(&state.pool, stored.id).await?;
    let new_raw = auth::generate_refresh_token();
    personel::create_refresh_token(
        &state.pool,
        personel.id,
        &auth::hash_token(&new_raw),
        stored.device_id.as_deref(),
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

async fn user_json(state: &AppState, personel: &PersonelRow) -> AppResult<Json<UserDto>> {
    let roles = personel::roles(&state.pool, personel.id).await?;
    let role_scopes = personel::role_scopes(&state.pool, personel.id).await?;
    Ok(Json(user_dto(
        personel,
        roles,
        role_scopes.into_iter().map(RoleScopeDto::from).collect(),
    )))
}

#[derive(Debug, Deserialize)]
pub struct UpdateProfileRequest {
    pub nama: Option<String>,
    pub pangkat: Option<String>,
    pub jabatan: Option<String>,
    pub phone: Option<String>,
    pub email: Option<String>,
}

/// Personel memperbarui nama, pangkat, jabatan, telepon, dan email miliknya sendiri.
pub async fn update_profile(
    State(state): State<AppState>,
    user: AuthUser,
    Json(req): Json<UpdateProfileRequest>,
) -> AppResult<Json<UserDto>> {
    let nama = req.nama.as_deref().map(str::trim).filter(|v| !v.is_empty());
    if let Some(nama) = nama {
        let panjang = nama.chars().count();
        if panjang < 3 {
            return Err(AppError::Unprocessable("Nama minimal 3 karakter".into()));
        }
        if panjang > 120 {
            return Err(AppError::Unprocessable("Nama maksimal 120 karakter".into()));
        }
    }

    let pangkat = req
        .pangkat
        .as_deref()
        .map(str::trim)
        .filter(|v| !v.is_empty());
    if let Some(pangkat) = pangkat
        && !PANGKAT_POLRI.contains(&pangkat)
    {
        return Err(AppError::Unprocessable(
            "Pangkat tidak termasuk tingkatan Polri".into(),
        ));
    }

    let jabatan = req
        .jabatan
        .as_deref()
        .map(str::trim)
        .filter(|v| !v.is_empty());
    if let Some(jabatan) = jabatan {
        let panjang = jabatan.chars().count();
        if panjang < 3 {
            return Err(AppError::Unprocessable("Jabatan minimal 3 karakter".into()));
        }
        if panjang > 120 {
            return Err(AppError::Unprocessable(
                "Jabatan maksimal 120 karakter".into(),
            ));
        }
    }

    let phone = req
        .phone
        .as_deref()
        .map(str::trim)
        .filter(|v| !v.is_empty());
    if let Some(phone) = phone {
        let valid = phone.chars().count() <= 30
            && phone
                .chars()
                .all(|c| c.is_ascii_digit() || matches!(c, '+' | '-' | ' ' | '(' | ')'));
        if !valid {
            return Err(AppError::Unprocessable("Nomor telepon tidak valid".into()));
        }
    }

    let email = req
        .email
        .as_deref()
        .map(str::trim)
        .filter(|v| !v.is_empty());
    if let Some(email) = email
        && (!email.contains('@') || email.chars().count() > 120)
    {
        return Err(AppError::Unprocessable("Alamat email tidak valid".into()));
    }

    personel::update_profil(&state.pool, user.id(), nama, pangkat, jabatan, phone, email).await?;
    let personel = personel::active_personel(&state.pool, user.id()).await?;
    user_json(&state, &personel).await
}

/// Unggah foto profil (JPG/PNG, maks. 5 MB).
pub async fn upload_foto(
    State(state): State<AppState>,
    user: AuthUser,
    mut multipart: Multipart,
) -> AppResult<Json<UserDto>> {
    let mut bytes: Option<Vec<u8>> = None;

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|_| AppError::BadRequest("Formulir unggahan tidak valid".into()))?
    {
        if field.name() == Some("file") {
            bytes = Some(
                field
                    .bytes()
                    .await
                    .map_err(|_| AppError::BadRequest("berkas gagal dibaca".into()))?
                    .to_vec(),
            );
        } else {
            let _ = field.bytes().await;
        }
    }

    let bytes = bytes.ok_or_else(|| AppError::BadRequest("Berkas foto wajib diunggah".into()))?;
    if bytes.is_empty() {
        return Err(AppError::BadRequest("Berkas foto kosong".into()));
    }
    if bytes.len() > storage::MAX_FOTO_BYTES {
        return Err(AppError::PayloadTooLarge(
            "Ukuran foto maksimal 5 MB".into(),
        ));
    }

    let Some((ext, _mime)) = storage::deteksi_berkas(&bytes) else {
        return Err(AppError::Unprocessable(
            "Foto profil harus berformat JPG atau PNG".into(),
        ));
    };
    if ext != "jpg" && ext != "png" {
        return Err(AppError::Unprocessable(
            "Foto profil harus berformat JPG atau PNG".into(),
        ));
    }

    let personel = personel::active_personel(&state.pool, user.id()).await?;
    let key = format!("foto/{}/{}.{ext}", personel.id, Uuid::now_v7());
    state.storage.save(&key, &bytes).await?;

    if let Some(lama) = personel.foto_path.as_deref() {
        state.storage.remove(lama).await.ok();
    }
    personel::set_foto_path(&state.pool, personel.id, Some(&key)).await?;

    let personel = personel::active_personel(&state.pool, user.id()).await?;
    user_json(&state, &personel).await
}

/// Mengambil foto profil menurut id personel (wajib login).
pub async fn foto(
    State(state): State<AppState>,
    _user: AuthUser,
    Path(id): Path<Uuid>,
) -> AppResult<Response> {
    let personel = personel::find_by_id(&state.pool, id)
        .await?
        .ok_or_else(|| AppError::NotFound("Foto profil tidak ditemukan".into()))?;
    let key = personel
        .foto_path
        .ok_or_else(|| AppError::NotFound("Foto profil belum diunggah".into()))?;

    let bytes = state.storage.read(&key).await?;
    let mime = mime_guess::from_path(&key).first_or_octet_stream();

    let mut response = Response::new(Body::from(bytes));
    *response.status_mut() = StatusCode::OK;
    response.headers_mut().insert(
        header::CONTENT_TYPE,
        HeaderValue::from_str(mime.as_ref())
            .unwrap_or_else(|_| HeaderValue::from_static("application/octet-stream")),
    );
    response.headers_mut().insert(
        header::CACHE_CONTROL,
        HeaderValue::from_static("private, max-age=300"),
    );
    Ok(response)
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
