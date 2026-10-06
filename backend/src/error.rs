use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;
use serde_json::Value;
use utoipa::ToSchema;

/// Every handler failure is one of these; each maps to exactly one HTTP status.
#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("{message}")]
    Validation {
        message: String,
        details: Option<Value>,
    },
    #[error("{0}")]
    Unauthorized(String),
    #[error("Invalid verification code")]
    InvalidCode,
    #[error("Verification code has expired")]
    CodeExpired,
    #[error("Verification code has already been used")]
    CodeAlreadyUsed,
    #[error("Too many incorrect attempts; please log in again")]
    TooManyAttempts,
    #[error("{0}")]
    Forbidden(String),
    #[error("{0}")]
    NotFound(String),
    #[error("{0}")]
    Conflict(String),
    #[error(transparent)]
    Internal(#[from] anyhow::Error),
}

#[derive(Serialize, ToSchema)]
pub struct ErrorBody {
    #[schema(example = "forbidden")]
    pub code: String,
    #[schema(example = "Only admins can create tasks")]
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<Value>,
}

/// Error envelope returned by every endpoint on failure.
#[derive(Serialize, ToSchema)]
pub struct ErrorResponse {
    pub error: ErrorBody,
}

impl AppError {
    pub fn validation(message: impl Into<String>) -> Self {
        Self::Validation {
            message: message.into(),
            details: None,
        }
    }

    fn status_and_code(&self) -> (StatusCode, &'static str) {
        match self {
            Self::Validation { .. } => (StatusCode::BAD_REQUEST, "validation_error"),
            Self::Unauthorized(_) => (StatusCode::UNAUTHORIZED, "unauthorized"),
            Self::InvalidCode => (StatusCode::UNAUTHORIZED, "invalid_code"),
            Self::CodeExpired => (StatusCode::UNAUTHORIZED, "code_expired"),
            Self::CodeAlreadyUsed => (StatusCode::UNAUTHORIZED, "code_already_used"),
            Self::TooManyAttempts => (StatusCode::TOO_MANY_REQUESTS, "too_many_attempts"),
            Self::Forbidden(_) => (StatusCode::FORBIDDEN, "forbidden"),
            Self::NotFound(_) => (StatusCode::NOT_FOUND, "not_found"),
            Self::Conflict(_) => (StatusCode::CONFLICT, "conflict"),
            Self::Internal(_) => (StatusCode::INTERNAL_SERVER_ERROR, "internal"),
        }
    }
}

impl From<sqlx::Error> for AppError {
    fn from(err: sqlx::Error) -> Self {
        Self::Internal(err.into())
    }
}

impl From<validator::ValidationErrors> for AppError {
    fn from(errors: validator::ValidationErrors) -> Self {
        let details = errors
            .field_errors()
            .iter()
            .map(|(field, errs)| {
                let messages: Vec<String> = errs
                    .iter()
                    .map(|e| {
                        e.message
                            .as_ref()
                            .map(|m| m.to_string())
                            .unwrap_or_else(|| e.code.to_string())
                    })
                    .collect();
                (field.to_string(), Value::from(messages))
            })
            .collect::<serde_json::Map<_, _>>();
        Self::Validation {
            message: "Request validation failed".into(),
            details: Some(Value::Object(details)),
        }
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, code) = self.status_and_code();

        let (message, details) = match self {
            Self::Internal(err) => {
                // Full detail goes to the logs; the client gets a generic message.
                tracing::error!(error = ?err, "internal error");
                ("Internal server error".to_string(), None)
            }
            Self::Validation { message, details } => (message, details),
            other => (other.to_string(), None),
        };

        let body = ErrorResponse {
            error: ErrorBody {
                code: code.to_string(),
                message,
                details,
            },
        };
        (status, Json(body)).into_response()
    }
}

pub type AppResult<T> = Result<T, AppError>;
