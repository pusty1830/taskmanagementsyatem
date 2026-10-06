use std::time::Duration;

use sqlx::postgres::PgPoolOptions;
use task_api::{config::Config, state::AppState};
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Searches the working directory and its parents, so the repo-root .env works from backend/.
    dotenvy::dotenv().ok();
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()))
        .init();

    let config = Config::from_env()?;

    let db = PgPoolOptions::new()
        .max_connections(10)
        .acquire_timeout(Duration::from_secs(5))
        .connect(&config.database_url)
        .await?;
    sqlx::migrate!("./migrations").run(&db).await?;
    tracing::info!("database migrations applied");

    let bind_addr = config.bind_addr.clone();
    let app = task_api::build_app(AppState::new(db, config));

    let listener = tokio::net::TcpListener::bind(&bind_addr).await?;
    tracing::info!("listening on http://{bind_addr} (Swagger UI at /swagger-ui)");
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;
    Ok(())
}

async fn shutdown_signal() {
    let _ = tokio::signal::ctrl_c().await;
    tracing::info!("shutdown signal received");
}
