mod common;

use axum::http::StatusCode;
use sqlx::PgPool;

#[sqlx::test(migrations = "./migrations")]
async fn health_reports_database_ok(pool: PgPool) {
    let app = common::app(pool);

    let (status, body) = common::get(&app, "/health", None).await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["status"], "ok");
    assert_eq!(body["database"], "ok");
}

#[sqlx::test(migrations = "./migrations")]
async fn unknown_route_returns_404(pool: PgPool) {
    let app = common::app(pool);

    let (status, _) = common::get(&app, "/does-not-exist", None).await;

    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[sqlx::test(migrations = "./migrations")]
async fn swagger_spec_is_served(pool: PgPool) {
    let app = common::app(pool);

    let (status, body) = common::get(&app, "/api-docs/openapi.json", None).await;

    assert_eq!(status, StatusCode::OK);
    assert!(body["paths"]["/health"]["get"].is_object());
}

async fn preflight_allow_origin(app: &axum::Router, origin: &str) -> Option<String> {
    use tower::ServiceExt;
    let req = axum::http::Request::builder()
        .method("OPTIONS")
        .uri("/auth/login")
        .header("origin", origin)
        .header("access-control-request-method", "POST")
        .header("access-control-request-headers", "content-type")
        .body(axum::body::Body::empty())
        .unwrap();
    let res = app.clone().oneshot(req).await.unwrap();
    res.headers()
        .get("access-control-allow-origin")
        .map(|v| v.to_str().unwrap().to_string())
}

#[sqlx::test(migrations = "./migrations")]
async fn cors_allows_every_configured_origin(pool: PgPool) {
    let mut config = common::test_config();
    config.cors_origin = "http://localhost:5173, http://127.0.0.1:5173".into();
    let app = common::app_with_config(pool, config);

    assert_eq!(
        preflight_allow_origin(&app, "http://localhost:5173")
            .await
            .as_deref(),
        Some("http://localhost:5173")
    );
    assert_eq!(
        preflight_allow_origin(&app, "http://127.0.0.1:5173")
            .await
            .as_deref(),
        Some("http://127.0.0.1:5173")
    );
    assert_eq!(
        preflight_allow_origin(&app, "http://evil.example").await,
        None
    );
}
