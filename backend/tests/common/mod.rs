#![allow(dead_code)]

use axum::{
    body::Body,
    http::{Method, Request, StatusCode},
    Router,
};
use http_body_util::BodyExt;
use serde_json::Value;
use sqlx::PgPool;
use std::sync::Arc;
use task_api::{
    cache::{MemoryTaskCache, TaskCache},
    config::Config,
    state::AppState,
};
use tower::ServiceExt;

pub fn test_config() -> Config {
    Config {
        app_env: "development".into(),
        bind_addr: "127.0.0.1:0".into(),
        database_url: String::new(),
        redis_url: String::new(),
        cache_backend: task_api::config::CacheBackend::Memory,
        cache_ttl_seconds: 300,
        jwt_secret: "test-jwt-secret-that-is-long-enough-123".into(),
        jwt_ttl_minutes: 60,
        otp_secret: "test-otp-secret-that-is-long-enough-456".into(),
        otp_ttl_seconds: 300,
        otp_max_attempts: 5,
        cors_origin: "http://localhost:5173".into(),
    }
}

pub fn app(pool: PgPool) -> Router {
    app_with_config(pool, test_config())
}

pub fn app_with_config(pool: PgPool, config: Config) -> Router {
    let cache = Arc::new(MemoryTaskCache::new(config.cache_ttl_seconds));
    app_with_cache(pool, config, cache)
}

pub fn app_with_cache(pool: PgPool, config: Config, cache: Arc<dyn TaskCache>) -> Router {
    task_api::build_app(AppState::new(pool, config, cache))
}

/// Sends a JSON request through the router and returns status + parsed body (Null if empty).
pub async fn send(
    app: &Router,
    method: Method,
    path: &str,
    token: Option<&str>,
    body: Option<Value>,
) -> (StatusCode, Value) {
    let mut req = Request::builder().method(method).uri(path);
    if let Some(t) = token {
        req = req.header("authorization", format!("Bearer {t}"));
    }
    let req = match body {
        Some(b) => req
            .header("content-type", "application/json")
            .body(Body::from(b.to_string())),
        None => req.body(Body::empty()),
    }
    .unwrap();

    let res = app.clone().oneshot(req).await.unwrap();
    let status = res.status();
    let bytes = res.into_body().collect().await.unwrap().to_bytes();
    let json = if bytes.is_empty() {
        Value::Null
    } else {
        serde_json::from_slice(&bytes).unwrap_or(Value::Null)
    };
    (status, json)
}

pub async fn get(app: &Router, path: &str, token: Option<&str>) -> (StatusCode, Value) {
    send(app, Method::GET, path, token, None).await
}

pub async fn post(
    app: &Router,
    path: &str,
    token: Option<&str>,
    body: Value,
) -> (StatusCode, Value) {
    send(app, Method::POST, path, token, Some(body)).await
}

pub const ADMIN_EMAIL: &str = "admin@example.com";
pub const ADMIN_PASSWORD: &str = "Admin@12345";
pub const JAMES_EMAIL: &str = "jamesbond@example.com";
pub const JAMES_PASSWORD: &str = "JamesBond@007";

pub async fn seed(app: &Router) {
    let (status, _) = post(app, "/seed/users", None, serde_json::json!({})).await;
    assert_eq!(status, StatusCode::OK, "seeding failed");
}

/// Step 1 of login: returns the login_challenge_id.
pub async fn start_login(app: &Router, email: &str, password: &str) -> String {
    let (status, body) = post(
        app,
        "/auth/login",
        None,
        serde_json::json!({ "email": email, "password": password }),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "login failed: {body}");
    body["login_challenge_id"].as_str().unwrap().to_string()
}

/// Reads the most recent verification code sent to `email` from the dev mailbox.
pub async fn latest_code(app: &Router, email: &str) -> String {
    let (status, body) = get(app, &format!("/dev/email-logs/latest?email={email}"), None).await;
    assert_eq!(status, StatusCode::OK, "no dev email for {email}: {body}");
    body["code"].as_str().unwrap().to_string()
}

pub async fn verify(app: &Router, challenge_id: &str, code: &str) -> (StatusCode, Value) {
    post(
        app,
        "/auth/verify-2fa",
        None,
        serde_json::json!({ "login_challenge_id": challenge_id, "code": code }),
    )
    .await
}

/// Full login → dev mailbox → verify flow; returns the JWT.
pub async fn login(app: &Router, email: &str, password: &str) -> String {
    let challenge_id = start_login(app, email, password).await;
    let code = latest_code(app, email).await;
    let (status, body) = verify(app, &challenge_id, &code).await;
    assert_eq!(status, StatusCode::OK, "verify failed: {body}");
    body["access_token"].as_str().unwrap().to_string()
}

/// Returns a 6-digit code guaranteed to differ from `code`.
pub fn wrong_code(code: &str) -> String {
    let n: u32 = code.parse().unwrap();
    format!("{:06}", (n + 1) % 1_000_000)
}

pub async fn patch(
    app: &Router,
    path: &str,
    token: Option<&str>,
    body: Value,
) -> (StatusCode, Value) {
    send(app, Method::PATCH, path, token, Some(body)).await
}

/// Creates a task as `token`'s user and returns its id (asserts 201).
pub async fn create_task(app: &Router, token: &str, title: &str, priority: &str) -> String {
    let (status, body) = post(
        app,
        "/tasks",
        Some(token),
        serde_json::json!({ "title": title, "description": format!("{title} details"), "priority": priority }),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED, "create task failed: {body}");
    body["id"].as_str().unwrap().to_string()
}

pub async fn assign(
    app: &Router,
    token: &str,
    task_ids: &[String],
    email: &str,
) -> (StatusCode, Value) {
    post(
        app,
        "/tasks/assign",
        Some(token),
        serde_json::json!({ "task_ids": task_ids, "assignee_email": email }),
    )
    .await
}

/// Seeds users and returns (admin_token, james_token).
pub async fn seeded_tokens(app: &Router) -> (String, String) {
    seed(app).await;
    let admin = login(app, ADMIN_EMAIL, ADMIN_PASSWORD).await;
    let james = login(app, JAMES_EMAIL, JAMES_PASSWORD).await;
    (admin, james)
}
