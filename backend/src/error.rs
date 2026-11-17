use axum::Json;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde_json::json;

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("{0}")]
    BadRequest(String),
    #[error("{0}")]
    Unauthorized(String),
    #[error("{0}")]
    Forbidden(String),
    #[error("{0}")]
    NotFound(String),
    #[error("{0}")]
    Conflict(String),
    #[error("{0}")]
    Unprocessable(String),
    #[error("{0}")]
    PayloadTooLarge(String),
    #[error("kesalahan internal")]
    Internal(String),
}

pub type AppResult<T> = Result<T, AppError>;

impl AppError {
    fn parts(&self) -> (StatusCode, &'static str) {
        match self {
            Self::BadRequest(_) => (StatusCode::BAD_REQUEST, "bad_request"),
            Self::Unauthorized(_) => (StatusCode::UNAUTHORIZED, "unauthorized"),
            Self::Forbidden(_) => (StatusCode::FORBIDDEN, "forbidden"),
            Self::NotFound(_) => (StatusCode::NOT_FOUND, "not_found"),
            Self::Conflict(_) => (StatusCode::CONFLICT, "conflict"),
            Self::Unprocessable(_) => (StatusCode::UNPROCESSABLE_ENTITY, "unprocessable_entity"),
            Self::PayloadTooLarge(_) => (StatusCode::PAYLOAD_TOO_LARGE, "payload_too_large"),
            Self::Internal(_) => (StatusCode::INTERNAL_SERVER_ERROR, "internal_error"),
        }
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, code) = self.parts();

        let message = match &self {
            Self::Internal(detail) => {
                tracing::error!(detail = %detail, "internal error");
                "Terjadi kesalahan pada server".to_string()
            }
            other => other.to_string(),
        };

        (
            status,
            Json(json!({ "error": { "code": code, "message": message } })),
        )
            .into_response()
    }
}

impl From<sqlx::Error> for AppError {
    fn from(err: sqlx::Error) -> Self {
        match err {
            sqlx::Error::RowNotFound => Self::NotFound("Data tidak ditemukan".into()),
            sqlx::Error::Database(db) if db.code().as_deref() == Some("23505") => {
                Self::Conflict("Data sudah ada".into())
            }
            sqlx::Error::Database(db) if db.code().as_deref() == Some("23503") => {
                Self::Unprocessable("Data terkait tidak ditemukan".into())
            }
            other => Self::Internal(format!("database: {other}")),
        }
    }
}

impl From<jsonwebtoken::errors::Error> for AppError {
    fn from(_: jsonwebtoken::errors::Error) -> Self {
        Self::Unauthorized("Token tidak valid atau kedaluwarsa".into())
    }
}

impl From<argon2::password_hash::Error> for AppError {
    fn from(_: argon2::password_hash::Error) -> Self {
        Self::Internal("argon2".into())
    }
}

impl From<std::io::Error> for AppError {
    fn from(err: std::io::Error) -> Self {
        Self::Internal(format!("io: {err}"))
    }
}
