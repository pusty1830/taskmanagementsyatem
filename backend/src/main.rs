use std::{sync::Arc, time::Duration};

use sqlx::postgres::PgPoolOptions;
use task_api::{
    cache::{MemoryTaskCache, RedisTaskCache, TaskCache},
    config::{CacheBackend, Config},
    state::AppState,
};
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

    let cache = build_cache(&config).await?;
    let bind_addr = config.bind_addr.clone();
    let app = task_api::build_app(AppState::new(db, config, cache));

    let listener = tokio::net::TcpListener::bind(&bind_addr).await?;
    tracing::info!("listening on http://{bind_addr} (Swagger UI at /swagger-ui)");
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;
    Ok(())
}

async fn build_cache(config: &Config) -> anyhow::Result<Arc<dyn TaskCache>> {
    Ok(match config.cache_backend {
        CacheBackend::Redis => {
            let cache =
                RedisTaskCache::connect(&config.redis_url, config.cache_ttl_seconds).await?;
            tracing::info!("task cache: redis ({})", config.redis_url);
            Arc::new(cache)
        }
        CacheBackend::Memory => {
            tracing::warn!("task cache: in-memory (per process; not shared between instances)");
            Arc::new(MemoryTaskCache::new(config.cache_ttl_seconds))
        }
    })
}

async fn shutdown_signal() {
    let _ = tokio::signal::ctrl_c().await;
    tracing::info!("shutdown signal received");
}
