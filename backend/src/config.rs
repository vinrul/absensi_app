use std::path::PathBuf;

#[derive(Clone, Debug)]
pub struct Settings {
    pub database_url: String,
    pub host: String,
    pub port: u16,
    pub jwt_secret: String,
    pub access_token_ttl_minutes: i64,
    pub refresh_token_ttl_days: i64,
    pub upload_dir: PathBuf,
    pub run_migrations: bool,
    pub cors_origins: Vec<String>,
    pub cookie_secure: bool,
    /// Hanya untuk development: mengaktifkan endpoint jam server palsu.
    pub allow_fake_time: bool,
}

fn var(key: &str) -> Option<String> {
    std::env::var(format!("APP__{key}"))
        .ok()
        .filter(|v| !v.trim().is_empty())
}

impl Settings {
    pub fn from_env() -> Result<Self, String> {
        let database_url =
            var("DATABASE_URL").ok_or_else(|| "APP__DATABASE_URL wajib diisi".to_string())?;

        let jwt_secret =
            var("JWT_SECRET").ok_or_else(|| "APP__JWT_SECRET wajib diisi".to_string())?;
        if jwt_secret.len() < 32 {
            return Err("APP__JWT_SECRET minimal 32 karakter".to_string());
        }

        Ok(Self {
            database_url,
            host: var("HOST").unwrap_or_else(|| "127.0.0.1".into()),
            port: var("PORT")
                .map(|v| v.parse::<u16>())
                .transpose()
                .map_err(|_| "APP__PORT harus berupa angka".to_string())?
                .unwrap_or(8080),
            jwt_secret,
            access_token_ttl_minutes: var("ACCESS_TOKEN_TTL_MINUTES")
                .map(|v| v.parse::<i64>())
                .transpose()
                .map_err(|_| "APP__ACCESS_TOKEN_TTL_MINUTES tidak valid".to_string())?
                .unwrap_or(15),
            refresh_token_ttl_days: var("REFRESH_TOKEN_TTL_DAYS")
                .map(|v| v.parse::<i64>())
                .transpose()
                .map_err(|_| "APP__REFRESH_TOKEN_TTL_DAYS tidak valid".to_string())?
                .unwrap_or(30),
            upload_dir: PathBuf::from(var("UPLOAD_DIR").unwrap_or_else(|| "./data/uploads".into())),
            run_migrations: var("RUN_MIGRATIONS")
                .map(|v| v == "true" || v == "1")
                .unwrap_or(false),
            cors_origins: var("CORS_ORIGINS")
                .unwrap_or_else(|| "http://localhost:5173,http://localhost:8088".into())
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect(),
            cookie_secure: var("COOKIE_SECURE")
                .map(|v| v == "true" || v == "1")
                .unwrap_or(false),
            allow_fake_time: var("ALLOW_FAKE_TIME")
                .map(|v| v == "true" || v == "1")
                .unwrap_or(false),
        })
    }
}
