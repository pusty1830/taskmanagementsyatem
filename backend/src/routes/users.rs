use axum::{
    extract::{Query, State},
    Json,
};
use serde::Deserialize;
use utoipa::IntoParams;

use crate::{
    auth::extractor::AdminUser,
    domain::Role,
    dto::user::UserDto,
    error::{AppResult, ErrorResponse},
    repositories::user_repo,
    state::AppState,
};

#[derive(Deserialize, IntoParams)]
pub struct ListUsersQuery {
    /// Filter by role, e.g. `staff` for the assignee dropdown.
    pub role: Option<Role>,
}

/// List users, optionally by role (admin only).
#[utoipa::path(
    get,
    path = "/users",
    tag = "users",
    security(("bearer_auth" = [])),
    params(ListUsersQuery),
    responses(
        (status = 200, description = "Users ordered by name", body = [UserDto]),
        (status = 400, description = "Unknown role", body = ErrorResponse),
        (status = 401, description = "Missing or invalid token", body = ErrorResponse),
        (status = 403, description = "Caller is not an admin", body = ErrorResponse),
    )
)]
pub async fn list_users(
    State(state): State<AppState>,
    _admin: AdminUser,
    Query(query): Query<ListUsersQuery>,
) -> AppResult<Json<Vec<UserDto>>> {
    let users = user_repo::list_by_role(&state.db, query.role).await?;
    Ok(Json(users.into_iter().map(UserDto::from).collect()))
}
