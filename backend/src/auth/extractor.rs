use axum::{extract::FromRequestParts, http::request::Parts};
use uuid::Uuid;

use crate::{auth::jwt, domain::Role, error::AppError, state::AppState};

/// The authenticated caller, decoded from a valid `Authorization: Bearer <JWT>` header.
#[derive(Debug, Clone)]
pub struct AuthUser {
    pub id: Uuid,
    pub email: String,
    pub role: Role,
}

impl AuthUser {
    pub fn is_admin(&self) -> bool {
        self.role == Role::Admin
    }
}

impl FromRequestParts<AppState> for AuthUser {
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let token = parts
            .headers
            .get(axum::http::header::AUTHORIZATION)
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.strip_prefix("Bearer "))
            .ok_or_else(|| AppError::Unauthorized("Missing bearer token".into()))?;

        let claims = jwt::decode_token(token.trim(), &state.config.jwt_secret)
            .map_err(|_| AppError::Unauthorized("Invalid or expired token".into()))?;

        Ok(Self {
            id: claims.sub,
            email: claims.email,
            role: claims.role,
        })
    }
}

/// An authenticated caller with the admin role. Non-admins get 403 before the handler runs.
#[derive(Debug, Clone)]
pub struct AdminUser(pub AuthUser);

impl FromRequestParts<AppState> for AdminUser {
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let user = AuthUser::from_request_parts(parts, state).await?;
        if !user.is_admin() {
            return Err(AppError::Forbidden(forbidden_message(parts)));
        }
        Ok(Self(user))
    }
}

/// Action-specific wording so the frontend can show e.g. "Only admins can create tasks".
fn forbidden_message(parts: &Parts) -> String {
    let path = parts.uri.path();
    let action = match (parts.method.as_str(), path) {
        ("POST", "/tasks") => "create tasks",
        ("POST", "/tasks/assign") => "assign tasks",
        ("GET", "/tasks") => "list all tasks",
        ("GET", "/users") => "list users",
        _ => "perform this action",
    };
    format!("Only admins can {action}")
}
