pub mod extractor;
pub mod jwt;
pub mod password;

use axum::http::HeaderMap;

pub const REFRESH_COOKIE: &str = "kinerja_refresh";

pub fn refresh_cookie(token: &str, max_age_secs: i64, secure: bool) -> String {
    let mut cookie =
        format!("{REFRESH_COOKIE}={token}; Path=/; HttpOnly; SameSite=Lax; Max-Age={max_age_secs}");
    if secure {
        cookie.push_str("; Secure");
    }
    cookie
}

pub fn clear_refresh_cookie() -> String {
    format!("{REFRESH_COOKIE}=; Path=/; HttpOnly; SameSite=Lax; Max-Age=0")
}

pub fn read_cookie(headers: &HeaderMap, name: &str) -> Option<String> {
    let raw = headers.get(axum::http::header::COOKIE)?.to_str().ok()?;
    raw.split(';')
        .filter_map(|part| part.trim().split_once('='))
        .find(|(key, _)| *key == name)
        .map(|(_, value)| value.to_string())
}

/// Token refresh disimpan sebagai hash SHA-256, bukan plaintext.
pub fn hash_token(token: &str) -> String {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(token.as_bytes());
    hex::encode(hasher.finalize())
}

pub fn generate_refresh_token() -> String {
    format!(
        "{}{}",
        uuid::Uuid::now_v7().simple(),
        uuid::Uuid::now_v7().simple()
    )
}
