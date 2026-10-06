use std::collections::BTreeMap;

use axum::{extract::State, Json};
use serde::Serialize;
use utoipa::ToSchema;

use crate::{
    dto::user::UserDto,
    error::{AppResult, ErrorResponse},
    services::seed_service::{self, SEED_USERS},
    state::AppState,
};

#[derive(Serialize, ToSchema)]
pub struct SeedResponse {
    pub users: Vec<UserDto>,
    /// Development-only convenience: the seeded login credentials.
    #[schema(example = json!({"admin@example.com": "Admin@12345", "jamesbond@example.com": "JamesBond@007"}))]
    pub credentials_hint: BTreeMap<String, String>,
}

/// Create the Admin and James Bond users (idempotent). Development only.
#[utoipa::path(
    post,
    path = "/seed/users",
    tag = "seed",
    description = "Creates `admin@example.com` (admin) and `jamesbond@example.com` (staff) if missing. \
        Safe to call repeatedly. Only available when `APP_ENV=development`.",
    responses(
        (status = 200, description = "Seed users exist", body = SeedResponse),
        (status = 500, description = "Unexpected error", body = ErrorResponse),
    )
)]
pub async fn seed_users(State(state): State<AppState>) -> AppResult<Json<SeedResponse>> {
    let users = seed_service::seed_users(&state.db).await?;
    let credentials_hint = SEED_USERS
        .iter()
        .map(|s| (s.email.to_string(), s.password.to_string()))
        .collect();
    Ok(Json(SeedResponse {
        users: users.into_iter().map(UserDto::from).collect(),
        credentials_hint,
    }))
}
