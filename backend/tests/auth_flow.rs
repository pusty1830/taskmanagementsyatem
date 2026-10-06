mod common;

use axum::http::StatusCode;
use common::{ADMIN_EMAIL, ADMIN_PASSWORD, JAMES_EMAIL, JAMES_PASSWORD};
use serde_json::json;
use sqlx::PgPool;
use uuid::Uuid;

#[sqlx::test(migrations = "./migrations")]
async fn login_returns_challenge_not_token(pool: PgPool) {
    let app = common::app(pool);
    common::seed(&app).await;

    let (status, body) = common::post(
        &app,
        "/auth/login",
        None,
        json!({ "email": ADMIN_EMAIL, "password": ADMIN_PASSWORD }),
    )
    .await;

    assert_eq!(status, StatusCode::OK);
    assert!(Uuid::parse_str(body["login_challenge_id"].as_str().unwrap()).is_ok());
    assert_eq!(body["expires_in_seconds"], 300);
    assert!(body.get("access_token").is_none(), "login must not issue a JWT");
}

#[sqlx::test(migrations = "./migrations")]
async fn login_email_is_case_insensitive(pool: PgPool) {
    let app = common::app(pool);
    common::seed(&app).await;

    let (status, _) = common::post(
        &app,
        "/auth/login",
        None,
        json!({ "email": "  Admin@Example.COM ", "password": ADMIN_PASSWORD }),
    )
    .await;

    assert_eq!(status, StatusCode::OK);
}

#[sqlx::test(migrations = "./migrations")]
async fn wrong_password_and_unknown_email_give_same_401(pool: PgPool) {
    let app = common::app(pool);
    common::seed(&app).await;

    let (s1, wrong_pw) = common::post(
        &app,
        "/auth/login",
        None,
        json!({ "email": ADMIN_EMAIL, "password": "nope" }),
    )
    .await;
    let (s2, unknown) = common::post(
        &app,
        "/auth/login",
        None,
        json!({ "email": "nobody@example.com", "password": "nope" }),
    )
    .await;

    assert_eq!(s1, StatusCode::UNAUTHORIZED);
    assert_eq!(s2, StatusCode::UNAUTHORIZED);
    assert_eq!(wrong_pw["error"]["code"], "unauthorized");
    assert_eq!(wrong_pw, unknown, "responses must not reveal which emails exist");
}

#[sqlx::test(migrations = "./migrations")]
async fn login_with_invalid_body_is_400(pool: PgPool) {
    let app = common::app(pool);

    let (status, body) = common::post(
        &app,
        "/auth/login",
        None,
        json!({ "email": "not-an-email", "password": "" }),
    )
    .await;

    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(body["error"]["code"], "validation_error");
}

#[sqlx::test(migrations = "./migrations")]
async fn malformed_json_uses_error_envelope(pool: PgPool) {
    let app = common::app(pool);

    let (status, body) = common::post(&app, "/auth/login", None, json!({ "email": 42 })).await;

    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(body["error"]["code"], "validation_error");
}

#[sqlx::test(migrations = "./migrations")]
async fn dev_mailbox_returns_latest_code(pool: PgPool) {
    let app = common::app(pool);
    common::seed(&app).await;
    let challenge_id = common::start_login(&app, JAMES_EMAIL, JAMES_PASSWORD).await;

    let (status, body) = common::get(
        &app,
        &format!("/dev/email-logs/latest?email={JAMES_EMAIL}"),
        None,
    )
    .await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["to_email"], JAMES_EMAIL);
    assert_eq!(body["challenge_id"], challenge_id);
    let code = body["code"].as_str().unwrap();
    assert_eq!(code.len(), 6);
    assert!(code.chars().all(|c| c.is_ascii_digit()));
    assert!(body["body"].as_str().unwrap().contains(code));
}

#[sqlx::test(migrations = "./migrations")]
async fn dev_mailbox_is_404_when_empty(pool: PgPool) {
    let app = common::app(pool);

    let (status, body) = common::get(&app, "/dev/email-logs/latest", None).await;

    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_eq!(body["error"]["code"], "not_found");
}

#[sqlx::test(migrations = "./migrations")]
async fn code_is_not_stored_in_plaintext(pool: PgPool) {
    let app = common::app(pool.clone());
    common::seed(&app).await;
    let challenge_id = common::start_login(&app, ADMIN_EMAIL, ADMIN_PASSWORD).await;
    let code = common::latest_code(&app, ADMIN_EMAIL).await;

    let stored: String =
        sqlx::query_scalar("SELECT code_hash FROM login_challenges WHERE id = $1::uuid")
            .bind(&challenge_id)
            .fetch_one(&pool)
            .await
            .unwrap();

    assert_ne!(stored, code);
    assert!(!stored.contains(&code));
    assert_eq!(stored.len(), 64, "expected hex-encoded HMAC-SHA256");
}

#[sqlx::test(migrations = "./migrations")]
async fn verify_with_correct_code_returns_jwt(pool: PgPool) {
    let app = common::app(pool);
    common::seed(&app).await;
    let challenge_id = common::start_login(&app, ADMIN_EMAIL, ADMIN_PASSWORD).await;
    let code = common::latest_code(&app, ADMIN_EMAIL).await;

    let (status, body) = common::verify(&app, &challenge_id, &code).await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["token_type"], "Bearer");
    assert_eq!(body["expires_in_seconds"], 3600);
    assert_eq!(body["user"]["email"], ADMIN_EMAIL);
    assert_eq!(body["user"]["role"], "admin");
    let token = body["access_token"].as_str().unwrap();
    assert_eq!(token.split('.').count(), 3, "expected a JWT");

    let (me_status, me) = common::get(&app, "/auth/me", Some(token)).await;
    assert_eq!(me_status, StatusCode::OK);
    assert_eq!(me["email"], ADMIN_EMAIL);
    assert_eq!(me["role"], "admin");
}

#[sqlx::test(migrations = "./migrations")]
async fn verify_with_wrong_code_is_rejected(pool: PgPool) {
    let app = common::app(pool);
    common::seed(&app).await;
    let challenge_id = common::start_login(&app, ADMIN_EMAIL, ADMIN_PASSWORD).await;
    let code = common::latest_code(&app, ADMIN_EMAIL).await;

    let (status, body) = common::verify(&app, &challenge_id, &common::wrong_code(&code)).await;

    assert_eq!(status, StatusCode::UNAUTHORIZED);
    assert_eq!(body["error"]["code"], "invalid_code");
    assert!(body.get("access_token").is_none());
}

#[sqlx::test(migrations = "./migrations")]
async fn verify_with_unknown_challenge_is_rejected(pool: PgPool) {
    let app = common::app(pool);

    let (status, body) = common::verify(&app, &Uuid::new_v4().to_string(), "123456").await;

    assert_eq!(status, StatusCode::UNAUTHORIZED);
    assert_eq!(body["error"]["code"], "invalid_code");
}

#[sqlx::test(migrations = "./migrations")]
async fn verify_with_expired_code_is_rejected(pool: PgPool) {
    let app = common::app(pool.clone());
    common::seed(&app).await;
    let challenge_id = common::start_login(&app, ADMIN_EMAIL, ADMIN_PASSWORD).await;
    let code = common::latest_code(&app, ADMIN_EMAIL).await;
    sqlx::query(
        "UPDATE login_challenges SET expires_at = now() - interval '1 second' WHERE id = $1::uuid",
    )
    .bind(&challenge_id)
    .execute(&pool)
    .await
    .unwrap();

    let (status, body) = common::verify(&app, &challenge_id, &code).await;

    assert_eq!(status, StatusCode::UNAUTHORIZED);
    assert_eq!(body["error"]["code"], "code_expired");
}

#[sqlx::test(migrations = "./migrations")]
async fn challenge_expires_after_configured_ttl(pool: PgPool) {
    let app = common::app(pool.clone());
    common::seed(&app).await;
    let challenge_id = common::start_login(&app, ADMIN_EMAIL, ADMIN_PASSWORD).await;

    let ttl_seconds: f64 = sqlx::query_scalar(
        "SELECT EXTRACT(EPOCH FROM (expires_at - created_at))::float8 FROM login_challenges WHERE id = $1::uuid",
    )
    .bind(&challenge_id)
    .fetch_one(&pool)
    .await
    .unwrap();

    assert!((ttl_seconds - 300.0).abs() < 2.0, "ttl was {ttl_seconds}s");
}

#[sqlx::test(migrations = "./migrations")]
async fn verify_reused_code_is_rejected(pool: PgPool) {
    let app = common::app(pool);
    common::seed(&app).await;
    let challenge_id = common::start_login(&app, ADMIN_EMAIL, ADMIN_PASSWORD).await;
    let code = common::latest_code(&app, ADMIN_EMAIL).await;
    let (first, _) = common::verify(&app, &challenge_id, &code).await;
    assert_eq!(first, StatusCode::OK);

    let (status, body) = common::verify(&app, &challenge_id, &code).await;

    assert_eq!(status, StatusCode::UNAUTHORIZED);
    assert_eq!(body["error"]["code"], "code_already_used");
}

#[sqlx::test(migrations = "./migrations")]
async fn too_many_wrong_attempts_locks_challenge(pool: PgPool) {
    let app = common::app(pool);
    common::seed(&app).await;
    let challenge_id = common::start_login(&app, ADMIN_EMAIL, ADMIN_PASSWORD).await;
    let code = common::latest_code(&app, ADMIN_EMAIL).await;
    let wrong = common::wrong_code(&code);
    for _ in 0..5 {
        let (status, _) = common::verify(&app, &challenge_id, &wrong).await;
        assert_eq!(status, StatusCode::UNAUTHORIZED);
    }

    // Even the correct code is refused once the attempt budget is spent.
    let (status, body) = common::verify(&app, &challenge_id, &code).await;

    assert_eq!(status, StatusCode::TOO_MANY_REQUESTS);
    assert_eq!(body["error"]["code"], "too_many_attempts");
}

#[sqlx::test(migrations = "./migrations")]
async fn new_login_invalidates_previous_challenge(pool: PgPool) {
    let app = common::app(pool);
    common::seed(&app).await;
    let old_challenge = common::start_login(&app, ADMIN_EMAIL, ADMIN_PASSWORD).await;
    let old_code = common::latest_code(&app, ADMIN_EMAIL).await;
    let new_challenge = common::start_login(&app, ADMIN_EMAIL, ADMIN_PASSWORD).await;
    let new_code = common::latest_code(&app, ADMIN_EMAIL).await;

    let (old_status, _) = common::verify(&app, &old_challenge, &old_code).await;
    let (new_status, _) = common::verify(&app, &new_challenge, &new_code).await;

    assert_eq!(old_status, StatusCode::UNAUTHORIZED);
    assert_eq!(new_status, StatusCode::OK);
}

#[sqlx::test(migrations = "./migrations")]
async fn me_requires_valid_token(pool: PgPool) {
    let app = common::app(pool);

    let (missing, body) = common::get(&app, "/auth/me", None).await;
    let (garbage, _) = common::get(&app, "/auth/me", Some("not.a.jwt")).await;

    assert_eq!(missing, StatusCode::UNAUTHORIZED);
    assert_eq!(garbage, StatusCode::UNAUTHORIZED);
    assert_eq!(body["error"]["code"], "unauthorized");
}

#[sqlx::test(migrations = "./migrations")]
async fn token_signed_with_other_secret_is_rejected(pool: PgPool) {
    let mut other = common::test_config();
    other.jwt_secret = "a-completely-different-secret-value-xyz".into();
    let issuing_app = common::app_with_config(pool.clone(), other);
    common::seed(&issuing_app).await;
    let foreign_token = common::login(&issuing_app, JAMES_EMAIL, JAMES_PASSWORD).await;

    let app = common::app(pool);
    let (status, _) = common::get(&app, "/auth/me", Some(&foreign_token)).await;

    assert_eq!(status, StatusCode::UNAUTHORIZED);
}
