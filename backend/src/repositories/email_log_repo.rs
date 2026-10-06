use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct EmailLog {
    pub id: Uuid,
    pub to_email: String,
    pub subject: String,
    pub body: String,
    pub challenge_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
}

pub async fn insert(
    db: &PgPool,
    to_email: &str,
    subject: &str,
    body: &str,
    challenge_id: Option<Uuid>,
) -> sqlx::Result<()> {
    sqlx::query(
        "INSERT INTO email_logs (to_email, subject, body, challenge_id) VALUES ($1, $2, $3, $4)",
    )
    .bind(to_email)
    .bind(subject)
    .bind(body)
    .bind(challenge_id)
    .execute(db)
    .await?;
    Ok(())
}

/// Most recent email, optionally only for one recipient.
pub async fn latest(db: &PgPool, to_email: Option<&str>) -> sqlx::Result<Option<EmailLog>> {
    sqlx::query_as::<_, EmailLog>(
        "SELECT id, to_email, subject, body, challenge_id, created_at
         FROM email_logs
         WHERE ($1::text IS NULL OR to_email = $1)
         ORDER BY created_at DESC, id DESC
         LIMIT 1",
    )
    .bind(to_email)
    .fetch_optional(db)
    .await
}
