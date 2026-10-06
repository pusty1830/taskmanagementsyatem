use axum::{extract::State, Json};

use crate::{
    auth::extractor::AuthUser,
    dto::{
        auth::{LoginRequest, LoginResponse, TokenResponse, Verify2faRequest},
        user::UserDto,
    },
    error::{AppError, AppResult, ErrorResponse},
    extract::ValidatedJson,
    repositories::user_repo,
    services::auth_service,
    state::AppState,
};

/// Step 1 of login: check email/password and email a one-time code.
#[utoipa::path(
    post,
    path = "/auth/login",
    tag = "auth",
    description = "Validates credentials, creates a 2FA challenge and sends a 6-digit code by email \
        (development: read it from `GET /dev/email-logs/latest` or the server console). \
        **Does not return a JWT.** Starting a new login invalidates earlier pending codes.",
    request_body = LoginRequest,
    responses(
        (status = 200, description = "Challenge created and code sent", body = LoginResponse),
        (status = 400, description = "Invalid request body", body = ErrorResponse),
        (status = 401, description = "Invalid email or password", body = ErrorResponse),
    )
)]
pub async fn login(
    State(state): State<AppState>,
    ValidatedJson(req): ValidatedJson<LoginRequest>,
) -> AppResult<Json<LoginResponse>> {
    let issued = auth_service::start_login(&state, &req.email, &req.password).await?;
    Ok(Json(LoginResponse {
        login_challenge_id: issued.challenge_id,
        expires_in_seconds: issued.expires_in_seconds,
        message: format!("Verification code sent to {}", issued.masked_email),
    }))
}

/// Step 2 of login: verify the emailed code and receive a JWT.
#[utoipa::path(
    post,
    path = "/auth/verify-2fa",
    tag = "auth",
    description = "Codes expire after 5 minutes, work once, and a challenge locks after 5 wrong attempts. \
        Copy `access_token` into **Authorize** to call protected endpoints.",
    request_body = Verify2faRequest,
    responses(
        (status = 200, description = "Code accepted; JWT issued", body = TokenResponse),
        (status = 400, description = "Invalid request body", body = ErrorResponse),
        (status = 401, description = "invalid_code | code_expired | code_already_used", body = ErrorResponse),
        (status = 429, description = "too_many_attempts", body = ErrorResponse),
    )
)]
pub async fn verify_2fa(
    State(state): State<AppState>,
    ValidatedJson(req): ValidatedJson<Verify2faRequest>,
) -> AppResult<Json<TokenResponse>> {
    let verified = auth_service::verify_login(&state, req.login_challenge_id, &req.code).await?;
    Ok(Json(TokenResponse {
        access_token: verified.token,
        token_type: "Bearer",
        expires_in_seconds: verified.expires_in_seconds,
        user: verified.user.into(),
    }))
}

/// The currently authenticated user.
#[utoipa::path(
    get,
    path = "/auth/me",
    tag = "auth",
    security(("bearer_auth" = [])),
    responses(
        (status = 200, description = "Current user", body = UserDto),
        (status = 401, description = "Missing, invalid or expired token", body = ErrorResponse),
    )
)]
pub async fn me(State(state): State<AppState>, user: AuthUser) -> AppResult<Json<UserDto>> {
    let user = user_repo::find_by_id(&state.db, user.id)
        .await?
        .ok_or_else(|| AppError::Unauthorized("User no longer exists".into()))?;
    Ok(Json(user.into()))
}
