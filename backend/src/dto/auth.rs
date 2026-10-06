use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;
use validator::Validate;

use crate::dto::user::UserDto;

#[derive(Debug, Deserialize, Validate, ToSchema)]
#[schema(example = json!({"email": "admin@example.com", "password": "Admin@12345"}))]
pub struct LoginRequest {
    #[serde(deserialize_with = "super::normalized_email")]
    #[validate(email(message = "must be a valid email address"))]
    pub email: String,
    #[validate(length(min = 1, max = 256, message = "is required"))]
    pub password: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct LoginResponse {
    /// Pass this with the emailed code to `POST /auth/verify-2fa`.
    pub login_challenge_id: Uuid,
    #[schema(example = 300)]
    pub expires_in_seconds: i64,
    #[schema(example = "Verification code sent to a***n@example.com")]
    pub message: String,
}

#[derive(Debug, Deserialize, Validate, ToSchema)]
#[schema(example = json!({"login_challenge_id": "00000000-0000-0000-0000-000000000000", "code": "123456"}))]
pub struct Verify2faRequest {
    pub login_challenge_id: Uuid,
    #[validate(length(equal = 6, message = "must be 6 digits"))]
    pub code: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct TokenResponse {
    pub access_token: String,
    #[schema(example = "Bearer")]
    pub token_type: &'static str,
    #[schema(example = 3600)]
    pub expires_in_seconds: i64,
    pub user: UserDto,
}
