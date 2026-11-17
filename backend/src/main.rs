use std::time::Duration;

use kinerja_api::config::Settings;
use kinerja_api::router;
use kinerja_api::state::AppState;
use kinerja_api::telemetry;
use sqlx::postgres::PgPoolOptions;
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();
    telemetry::init();

    let settings = Settings::from_env().map_err(|err| {
        tracing::error!("konfigurasi tidak valid: {err}");
        err
    })?;

    std::fs::create_dir_all(&settings.upload_dir)?;

    let pool = PgPoolOptions::new()
        .max_connections(10)
        .acquire_timeout(Duration::from_secs(10))
        .connect(&settings.database_url)
        .await?;

    if settings.run_migrations {
        sqlx::migrate!("./migrations").run(&pool).await?;
        tracing::info!("migrasi database selesai");
    }

    let addr = format!("{}:{}", settings.host, settings.port);
    let listener = TcpListener::bind(&addr).await?;
    tracing::info!("API berjalan di http://{addr}");

    let app = router::build(AppState::new(pool, settings));
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    Ok(())
}

async fn shutdown_signal() {
    let _ = tokio::signal::ctrl_c().await;
    tracing::info!("menerima sinyal berhenti, menutup server");
}
