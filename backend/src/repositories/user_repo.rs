use sqlx::PgPool;
use uuid::Uuid;

use crate::domain::{Role, User};

const USER_COLUMNS: &str =
    "id, full_name, email, hashed_password, role, created_at, updated_at";

pub async fn find_by_email(db: &PgPool, email: &str) -> sqlx::Result<Option<User>> {
    sqlx::query_as::<_, User>(&format!("SELECT {USER_COLUMNS} FROM users WHERE email = $1"))
        .bind(email)
        .fetch_optional(db)
        .await
}

pub async fn find_by_id(db: &PgPool, id: Uuid) -> sqlx::Result<Option<User>> {
    sqlx::query_as::<_, User>(&format!("SELECT {USER_COLUMNS} FROM users WHERE id = $1"))
        .bind(id)
        .fetch_optional(db)
        .await
}

pub async fn list_by_role(db: &PgPool, role: Option<Role>) -> sqlx::Result<Vec<User>> {
    sqlx::query_as::<_, User>(&format!(
        "SELECT {USER_COLUMNS} FROM users WHERE ($1::user_role IS NULL OR role = $1) ORDER BY full_name"
    ))
    .bind(role)
    .fetch_all(db)
    .await
}

/// Inserts the user unless the email already exists (concurrency-safe).
pub async fn insert_if_absent(
    db: &PgPool,
    full_name: &str,
    email: &str,
    hashed_password: &str,
    role: Role,
) -> sqlx::Result<()> {
    sqlx::query(
        "INSERT INTO users (full_name, email, hashed_password, role)
         VALUES ($1, $2, $3, $4)
         ON CONFLICT (email) DO NOTHING",
    )
    .bind(full_name)
    .bind(email)
    .bind(hashed_password)
    .bind(role)
    .execute(db)
    .await?;
    Ok(())
}
