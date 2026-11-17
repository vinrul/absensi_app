use std::sync::Arc;

use sqlx::PgPool;

use crate::config::Settings;
use crate::middleware::rate_limit::PenjagaLaju;
use crate::service::storage::LocalStorage;

#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
    pub settings: Arc<Settings>,
    pub storage: LocalStorage,
    pub rate: Arc<PenjagaLaju>,
}

impl AppState {
    pub fn new(pool: PgPool, settings: Settings) -> Self {
        let storage = LocalStorage::new(settings.upload_dir.clone());
        Self {
            pool,
            settings: Arc::new(settings),
            storage,
            rate: Arc::new(PenjagaLaju::new()),
        }
    }
}
