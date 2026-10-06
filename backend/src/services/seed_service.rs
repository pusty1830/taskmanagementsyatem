use sqlx::PgPool;

use crate::{
    auth::password::hash_password,
    domain::{Role, User},
    error::AppResult,
    repositories::user_repo,
};

pub struct SeedUser {
    pub full_name: &'static str,
    pub email: &'static str,
    pub password: &'static str,
    pub role: Role,
}

/// The two validation users from the assignment brief.
pub const SEED_USERS: [SeedUser; 2] = [
    SeedUser {
        full_name: "Admin",
        email: "admin@example.com",
        password: "Admin@12345",
        role: Role::Admin,
    },
    SeedUser {
        full_name: "James Bond",
        email: "jamesbond@example.com",
        password: "JamesBond@007",
        role: Role::Staff,
    },
];

/// Creates missing seed users; existing ones are returned unchanged.
pub async fn seed_users(db: &PgPool) -> AppResult<Vec<User>> {
    let mut users = Vec::with_capacity(SEED_USERS.len());
    for seed in &SEED_USERS {
        if user_repo::find_by_email(db, seed.email).await?.is_none() {
            let hash = hash_password(seed.password)?;
            user_repo::insert_if_absent(db, seed.full_name, seed.email, &hash, seed.role).await?;
        }
        let user = user_repo::find_by_email(db, seed.email)
            .await?
            .ok_or_else(|| anyhow::anyhow!("seed user {} missing after insert", seed.email))?;
        users.push(user);
    }
    Ok(users)
}
