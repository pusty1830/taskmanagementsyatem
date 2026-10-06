use chrono::{DateTime, Utc};
use sqlx::PgConnection;
use uuid::Uuid;

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct LoginChallenge {
    pub id: Uuid,
    pub user_id: Uuid,
    pub code_hash: String,
    pub attempts: i32,
    pub expires_at: DateTime<Utc>,
    pub consumed_at: Option<DateTime<Utc>>,
}

/// Marks every still-pending challenge of the user as consumed, so only the newest code works.
pub async fn invalidate_pending(conn: &mut PgConnection, user_id: Uuid) -> sqlx::Result<()> {
    sqlx::query(
        "UPDATE login_challenges SET consumed_at = now()
         WHERE user_id = $1 AND consumed_at IS NULL",
    )
    .bind(user_id)
    .execute(conn)
    .await?;
    Ok(())
}

pub async fn insert(
    conn: &mut PgConnection,
    id: Uuid,
    user_id: Uuid,
    code_hash: &str,
    ttl_seconds: i64,
) -> sqlx::Result<()> {
    sqlx::query(
        "INSERT INTO login_challenges (id, user_id, code_hash, expires_at)
         VALUES ($1, $2, $3, now() + make_interval(secs => $4))",
    )
    .bind(id)
    .bind(user_id)
    .bind(code_hash)
    .bind(ttl_seconds as f64)
    .execute(conn)
    .await?;
    Ok(())
}

/// Row-locks the challenge for the rest of the transaction, serialising concurrent verifies.
pub async fn find_for_update(
    conn: &mut PgConnection,
    id: Uuid,
) -> sqlx::Result<Option<LoginChallenge>> {
    sqlx::query_as::<_, LoginChallenge>(
        "SELECT id, user_id, code_hash, attempts, expires_at, consumed_at
         FROM login_challenges WHERE id = $1 FOR UPDATE",
    )
    .bind(id)
    .fetch_optional(conn)
    .await
}

pub async fn increment_attempts(conn: &mut PgConnection, id: Uuid) -> sqlx::Result<()> {
    sqlx::query("UPDATE login_challenges SET attempts = attempts + 1 WHERE id = $1")
        .bind(id)
        .execute(conn)
        .await?;
    Ok(())
}

pub async fn consume(conn: &mut PgConnection, id: Uuid) -> sqlx::Result<()> {
    sqlx::query("UPDATE login_challenges SET consumed_at = now() WHERE id = $1")
        .bind(id)
        .execute(conn)
        .await?;
    Ok(())
}
