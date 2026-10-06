use axum::{extract::State, http::StatusCode, Json};
use serde::Serialize;
use utoipa::ToSchema;

use crate::state::AppState;

#[derive(Serialize, ToSchema)]
pub struct HealthResponse {
    #[schema(example = "ok")]
    pub status: &'static str,
    #[schema(example = "ok")]
    pub database: &'static str,
}

/// Liveness check, including a database ping.
#[utoipa::path(
    get,
    path = "/health",
    tag = "health",
    responses(
        (status = 200, description = "API and database are up", body = HealthResponse),
        (status = 503, description = "Database unreachable", body = HealthResponse),
    )
)]
pub async fn health(State(state): State<AppState>) -> (StatusCode, Json<HealthResponse>) {
    match sqlx::query("SELECT 1").execute(&state.db).await {
        Ok(_) => (
            StatusCode::OK,
            Json(HealthResponse {
                status: "ok",
                database: "ok",
            }),
        ),
        Err(err) => {
            tracing::warn!(error = %err, "health check: database unreachable");
            (
                StatusCode::SERVICE_UNAVAILABLE,
                Json(HealthResponse {
                    status: "degraded",
                    database: "unreachable",
                }),
            )
        }
    }
}
