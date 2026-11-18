use std::path::PathBuf;

/// Kebijakan server terhadap absensi/strong point yang terdeteksi memakai lokasi tiruan.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MockLocationPolicy {
    /// Simpan data dan tandai `perlu_verifikasi` (default development).
    Flag,
    /// Tolak dengan HTTP 422 (disarankan untuk produksi).
    Reject,
}

impl MockLocationPolicy {
    pub fn parse(value: &str) -> Result<Self, String> {
        match value.trim().to_ascii_lowercase().as_str() {
            "flag" | "tandai" => Ok(Self::Flag),
            "reject" | "tolak" => Ok(Self::Reject),
            other => Err(format!(
                "APP__MOCK_LOCATION_POLICY tidak dikenal: {other} (gunakan flag|reject)"
            )),
        }
    }

    pub fn menolak(self, is_mock: bool) -> bool {
        matches!(self, Self::Reject) && is_mock
    }
}

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
    /// `flag` (tandai) atau `reject` (tolak) lokasi tiruan.
    pub mock_location_policy: MockLocationPolicy,
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
            mock_location_policy: MockLocationPolicy::parse(
                &var("MOCK_LOCATION_POLICY").unwrap_or_else(|| "flag".into()),
            )?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::MockLocationPolicy;

    #[test]
    fn policy_mock_default_dan_parsing() {
        assert_eq!(
            MockLocationPolicy::parse("flag"),
            Ok(MockLocationPolicy::Flag)
        );
        assert_eq!(
            MockLocationPolicy::parse("REJECT"),
            Ok(MockLocationPolicy::Reject)
        );
        assert!(MockLocationPolicy::parse("lain").is_err());
    }

    #[test]
    fn policy_mock_menolak_hanya_saat_reject() {
        assert!(!MockLocationPolicy::Flag.menolak(true));
        assert!(MockLocationPolicy::Reject.menolak(true));
        assert!(!MockLocationPolicy::Reject.menolak(false));
    }
}
