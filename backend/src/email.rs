use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;

use crate::repositories::email_log_repo;

pub const VERIFICATION_SUBJECT: &str = "Your login verification code";

/// Outbound email. Swap `DevMailer` for an SMTP implementation without touching the services.
#[async_trait]
pub trait Mailer: Send + Sync {
    async fn send_verification_code(
        &self,
        to: &str,
        code: &str,
        challenge_id: Uuid,
        ttl_minutes: i64,
    ) -> anyhow::Result<()>;
}

/// Development mailer: records the email in `email_logs` (the dev mailbox) and logs it.
pub struct DevMailer {
    db: PgPool,
}

impl DevMailer {
    pub fn new(db: PgPool) -> Self {
        Self { db }
    }
}

#[async_trait]
impl Mailer for DevMailer {
    async fn send_verification_code(
        &self,
        to: &str,
        code: &str,
        challenge_id: Uuid,
        ttl_minutes: i64,
    ) -> anyhow::Result<()> {
        let body = format!(
            "Your verification code is {code}. It expires in {ttl_minutes} minutes and can be used once."
        );
        email_log_repo::insert(&self.db, to, VERIFICATION_SUBJECT, &body, Some(challenge_id)).await?;
        tracing::info!(%challenge_id, "[dev-mail] 2FA code for {to}: {code}");
        Ok(())
    }
}
