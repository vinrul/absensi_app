use argon2::password_hash::SaltString;
use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use rand_core::OsRng;

use crate::error::{AppError, AppResult};

pub fn hash(password: &str) -> AppResult<String> {
    let salt = SaltString::generate(&mut OsRng);
    let hash = Argon2::default()
        .hash_password(password.as_bytes(), &salt)
        .map_err(|_| AppError::Internal("gagal membuat hash password".into()))?;
    Ok(hash.to_string())
}

pub fn verify(password: &str, stored_hash: &str) -> bool {
    PasswordHash::new(stored_hash)
        .map(|parsed| {
            Argon2::default()
                .verify_password(password.as_bytes(), &parsed)
                .is_ok()
        })
        .unwrap_or(false)
}

/// Kebijakan password: minimal 8 karakter, mengandung huruf dan angka.
pub fn validate_strength(password: &str) -> AppResult<()> {
    if password.chars().count() < 8 {
        return Err(AppError::Unprocessable(
            "Password minimal 8 karakter".into(),
        ));
    }
    let has_letter = password.chars().any(|c| c.is_alphabetic());
    let has_digit = password.chars().any(|c| c.is_numeric());
    if !has_letter || !has_digit {
        return Err(AppError::Unprocessable(
            "Password harus mengandung huruf dan angka".into(),
        ));
    }
    Ok(())
}
