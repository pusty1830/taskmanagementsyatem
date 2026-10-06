use std::sync::LazyLock;

use uuid::Uuid;

use crate::{
    auth::{jwt, otp, password},
    domain::User,
    error::{AppError, AppResult},
    repositories::{challenge_repo, user_repo},
    state::AppState,
};

const INVALID_CREDENTIALS: &str = "Invalid email or password";

/// Verified against when the email is unknown, so both failure paths cost one Argon2 run
/// and response timing does not reveal which emails exist.
static DUMMY_HASH: LazyLock<String> = LazyLock::new(|| {
    password::hash_password("timing-equaliser-not-a-real-password").expect("argon2 hashing")
});

pub struct LoginChallengeIssued {
    pub challenge_id: Uuid,
    pub expires_in_seconds: i64,
    pub masked_email: String,
}

pub struct VerifiedLogin {
    pub token: String,
    pub expires_in_seconds: i64,
    pub user: User,
}

pub fn normalize_email(email: &str) -> String {
    email.trim().to_lowercase()
}

/// Step 1: check credentials, create a 2FA challenge and email the code. Never issues a JWT.
pub async fn start_login(
    state: &AppState,
    email: &str,
    password_attempt: &str,
) -> AppResult<LoginChallengeIssued> {
    let email = normalize_email(email);
    let user = user_repo::find_by_email(&state.db, &email).await?;

    let password_ok = match &user {
        Some(u) => password::verify_password(password_attempt, &u.hashed_password),
        None => {
            password::verify_password(password_attempt, &DUMMY_HASH);
            false
        }
    };
    let user = match (user, password_ok) {
        (Some(u), true) => u,
        _ => return Err(AppError::Unauthorized(INVALID_CREDENTIALS.into())),
    };

    let cfg = &state.config;
    let challenge_id = Uuid::new_v4();
    let code = otp::generate_code();
    let code_hash = otp::hash_code(&cfg.otp_secret, challenge_id, &code);

    let mut tx = state.db.begin().await?;
    challenge_repo::invalidate_pending(&mut tx, user.id).await?;
    challenge_repo::insert(&mut tx, challenge_id, user.id, &code_hash, cfg.otp_ttl_seconds).await?;
    tx.commit().await?;

    state
        .mailer
        .send_verification_code(&user.email, &code, challenge_id, cfg.otp_ttl_seconds / 60)
        .await?;

    Ok(LoginChallengeIssued {
        challenge_id,
        expires_in_seconds: cfg.otp_ttl_seconds,
        masked_email: mask_email(&user.email),
    })
}

/// Step 2: verify the code (single use, unexpired, attempt-limited) and issue the JWT.
pub async fn verify_login(state: &AppState, challenge_id: Uuid, code: &str) -> AppResult<VerifiedLogin> {
    let cfg = &state.config;
    let mut tx = state.db.begin().await?;

    let challenge = challenge_repo::find_for_update(&mut tx, challenge_id)
        .await?
        .ok_or(AppError::InvalidCode)?;

    if challenge.consumed_at.is_some() {
        return Err(AppError::CodeAlreadyUsed);
    }
    if challenge.expires_at <= chrono::Utc::now() {
        return Err(AppError::CodeExpired);
    }
    if challenge.attempts >= cfg.otp_max_attempts {
        return Err(AppError::TooManyAttempts);
    }
    if !otp::verify_code(&cfg.otp_secret, challenge.id, code, &challenge.code_hash) {
        challenge_repo::increment_attempts(&mut tx, challenge.id).await?;
        tx.commit().await?;
        return Err(AppError::InvalidCode);
    }

    challenge_repo::consume(&mut tx, challenge.id).await?;
    tx.commit().await?;

    let user = user_repo::find_by_id(&state.db, challenge.user_id)
        .await?
        .ok_or_else(|| anyhow::anyhow!("challenge references a missing user"))?;
    let (token, expires_in_seconds) = jwt::issue_token(&user, &cfg.jwt_secret, cfg.jwt_ttl_minutes)?;

    Ok(VerifiedLogin {
        token,
        expires_in_seconds,
        user,
    })
}

/// "admin@example.com" -> "a***n@example.com"
fn mask_email(email: &str) -> String {
    match email.split_once('@') {
        Some((local, domain)) if local.chars().count() >= 2 => {
            let first = local.chars().next().unwrap();
            let last = local.chars().last().unwrap();
            format!("{first}***{last}@{domain}")
        }
        Some((_, domain)) => format!("***@{domain}"),
        None => "***".into(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn masks_email_local_part() {
        assert_eq!(mask_email("admin@example.com"), "a***n@example.com");
        assert_eq!(mask_email("j@example.com"), "***@example.com");
    }

    #[test]
    fn normalizes_email() {
        assert_eq!(normalize_email("  Admin@Example.COM "), "admin@example.com");
    }
}
