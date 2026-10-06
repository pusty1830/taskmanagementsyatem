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
