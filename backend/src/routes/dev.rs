use axum::{extract::State, Json};
use serde::Serialize;
use utoipa::ToSchema;

use crate::{
    error::{AppResult, ErrorResponse},
    state::AppState,
};

#[derive(Serialize, ToSchema)]
pub struct MessageResponse {
    #[schema(example = "Tasks, login challenges and email logs cleared")]
    pub message: String,
}

/// Clear tasks, 2FA challenges, email logs and the task cache (users are kept). Development only.
#[utoipa::path(
    post,
    path = "/dev/reset",
    tag = "dev",
    description = "Lets the validation flow (exactly 5 tasks, exactly 3 assigned) be repeated from a clean slate. \
        Seeded users are kept. Only available when `APP_ENV=development`.",
    responses(
        (status = 200, description = "Data cleared", body = MessageResponse),
        (status = 500, description = "Unexpected error", body = ErrorResponse),
    )
)]
pub async fn reset(State(state): State<AppState>) -> AppResult<Json<MessageResponse>> {
    sqlx::query("TRUNCATE tasks, email_logs, login_challenges")
        .execute(&state.db)
        .await?;
    Ok(Json(MessageResponse {
        message: "Tasks, login challenges and email logs cleared".into(),
    }))
}
