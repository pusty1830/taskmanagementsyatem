#![allow(dead_code)]

use axum::{
    body::Body,
    http::{Method, Request, StatusCode},
    Router,
};
use http_body_util::BodyExt;
use serde_json::Value;
use sqlx::PgPool;
use task_api::{config::Config, state::AppState};
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
    task_api::build_app(AppState::new(pool, config))
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
