mod common;

use axum::http::StatusCode;
use serde_json::json;
use sqlx::PgPool;

#[sqlx::test(migrations = "./migrations")]
async fn seed_creates_admin_and_james_idempotently(pool: PgPool) {
    let app = common::app(pool.clone());

    let (status, first) = common::post(&app, "/seed/users", None, json!({})).await;
    assert_eq!(status, StatusCode::OK);
    let users = first["users"].as_array().unwrap();
    assert_eq!(users.len(), 2);
    assert_eq!(users[0]["email"], "admin@example.com");
    assert_eq!(users[0]["role"], "admin");
    assert_eq!(users[0]["full_name"], "Admin");
    assert_eq!(users[1]["email"], "jamesbond@example.com");
    assert_eq!(users[1]["role"], "staff");
    assert_eq!(users[1]["full_name"], "James Bond");
    assert!(users[0].get("hashed_password").is_none());

    let (status, second) = common::post(&app, "/seed/users", None, json!({})).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(first["users"], second["users"], "re-seeding must not create new users");

    let count: i64 = sqlx::query_scalar("SELECT count(*) FROM users")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(count, 2);
}

#[sqlx::test(migrations = "./migrations")]
async fn seeded_passwords_are_hashed_with_argon2(pool: PgPool) {
    let app = common::app(pool.clone());
    common::post(&app, "/seed/users", None, json!({})).await;

    let hashes: Vec<String> = sqlx::query_scalar("SELECT hashed_password FROM users")
        .fetch_all(&pool)
        .await
        .unwrap();
    assert_eq!(hashes.len(), 2);
    assert!(hashes.iter().all(|h| h.starts_with("$argon2id$")));
}

#[sqlx::test(migrations = "./migrations")]
async fn seed_and_dev_routes_are_hidden_outside_development(pool: PgPool) {
    let mut config = common::test_config();
    config.app_env = "production".into();
    let app = common::app_with_config(pool, config);

    let (seed_status, _) = common::post(&app, "/seed/users", None, json!({})).await;
    let (reset_status, _) = common::post(&app, "/dev/reset", None, json!({})).await;

    assert_eq!(seed_status, StatusCode::NOT_FOUND);
    assert_eq!(reset_status, StatusCode::NOT_FOUND);
}

#[sqlx::test(migrations = "./migrations")]
async fn dev_reset_keeps_users(pool: PgPool) {
    let app = common::app(pool.clone());
    common::post(&app, "/seed/users", None, json!({})).await;

    let (status, body) = common::post(&app, "/dev/reset", None, json!({})).await;

    assert_eq!(status, StatusCode::OK);
    assert!(body["message"].is_string());
    let count: i64 = sqlx::query_scalar("SELECT count(*) FROM users")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(count, 2);
}
