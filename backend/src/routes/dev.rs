use axum::{
    extract::{Query, State},
    Json,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};
use uuid::Uuid;

use crate::{
    error::{AppError, AppResult, ErrorResponse},
    repositories::email_log_repo::{self, EmailLog},
    services::auth_service::normalize_email,
    state::AppState,
};

#[derive(Serialize, ToSchema)]
pub struct MessageResponse {
    #[schema(example = "Tasks, login challenges, email logs and task cache cleared")]
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
    state.cache.clear().await?;
    Ok(Json(MessageResponse {
        message: "Tasks, login challenges, email logs and task cache cleared".into(),
    }))
}

#[derive(Deserialize, IntoParams)]
pub struct LatestEmailQuery {
    /// Only return the latest email sent to this address.
    #[param(example = "jamesbond@example.com")]
    pub email: Option<String>,
}

#[derive(Serialize, ToSchema)]
pub struct EmailLogDto {
    pub id: Uuid,
    #[schema(example = "jamesbond@example.com")]
    pub to_email: String,
    #[schema(example = "Your login verification code")]
    pub subject: String,
    pub body: String,
    /// The 6-digit code parsed from the body, for convenience.
    #[schema(example = "482913")]
    pub code: Option<String>,
    pub challenge_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
}

impl From<EmailLog> for EmailLogDto {
    fn from(log: EmailLog) -> Self {
        Self {
            code: extract_code(&log.body),
            id: log.id,
            to_email: log.to_email,
            subject: log.subject,
            body: log.body,
            challenge_id: log.challenge_id,
            created_at: log.created_at,
        }
    }
}

/// First standalone run of exactly six digits in the email body.
fn extract_code(body: &str) -> Option<String> {
    body.split(|c: char| !c.is_ascii_digit())
        .find(|run| run.len() == 6)
        .map(str::to_string)
}

/// Latest email from the development mailbox (where 2FA codes are delivered). Development only.
#[utoipa::path(
    get,
    path = "/dev/email-logs/latest",
    tag = "dev",
    description = "Stand-in for a real inbox: returns the most recent verification email, optionally filtered by recipient. Only available when `APP_ENV=development`.",
    params(LatestEmailQuery),
    responses(
        (status = 200, description = "Latest email", body = EmailLogDto),
        (status = 404, description = "No email sent yet", body = ErrorResponse),
    )
)]
pub async fn latest_email(
    State(state): State<AppState>,
    Query(query): Query<LatestEmailQuery>,
) -> AppResult<Json<EmailLogDto>> {
    let email = query.email.as_deref().map(normalize_email);
    let log = email_log_repo::latest(&state.db, email.as_deref())
        .await?
        .ok_or_else(|| AppError::NotFound("No emails have been sent yet".into()))?;
    Ok(Json(log.into()))
}

#[cfg(test)]
mod tests {
    use super::extract_code;

    #[test]
    fn extracts_six_digit_code() {
        assert_eq!(
            extract_code("Your verification code is 012345. It expires in 5 minutes.").as_deref(),
            Some("012345")
        );
        assert_eq!(extract_code("no code here 12345"), None);
    }
}
