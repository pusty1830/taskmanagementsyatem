mod common;

use axum::http::{Method, StatusCode};
use serde_json::{json, Value};
use sqlx::PgPool;

/// (method, path, requires_bearer): the full public API contract. Adding a route means adding it here.
const ROUTES: &[(&str, &str, bool)] = &[
    ("get", "/health", false),
    ("post", "/seed/users", false),
    ("post", "/dev/reset", false),
    ("get", "/dev/email-logs/latest", false),
    ("post", "/auth/login", false),
    ("post", "/auth/verify-2fa", false),
    ("get", "/auth/me", true),
    ("get", "/users", true),
    ("post", "/tasks", true),
    ("get", "/tasks", true),
    ("post", "/tasks/assign", true),
    ("patch", "/tasks/{id}", true),
    ("get", "/tasks/view-my-tasks", true),
];

async fn spec(pool: PgPool) -> Value {
    let app = common::app(pool);
    let (status, spec) = common::get(&app, "/api-docs/openapi.json", None).await;
    assert_eq!(status, StatusCode::OK);
    spec
}

#[sqlx::test(migrations = "./migrations")]
async fn openapi_documents_every_route(pool: PgPool) {
    let spec = spec(pool).await;

    for (method, path, requires_bearer) in ROUTES {
        let op = &spec["paths"][path][method];
        assert!(
            op.is_object(),
            "{} {path} missing from OpenAPI",
            method.to_uppercase()
        );
        assert!(op["tags"][0].is_string(), "{method} {path} has no tag");
        assert!(
            op["responses"].as_object().is_some_and(|r| !r.is_empty()),
            "{method} {path} has no responses"
        );

        let has_bearer = op["security"]
            .as_array()
            .is_some_and(|s| s.iter().any(|req| req.get("bearer_auth").is_some()));
        assert_eq!(
            has_bearer, *requires_bearer,
            "{method} {path} bearer_auth mismatch"
        );
        if *requires_bearer {
            assert!(
                op["responses"]["401"].is_object(),
                "{method} {path} must document 401"
            );
        }
    }

    let documented: usize = spec["paths"]
        .as_object()
        .unwrap()
        .values()
        .map(|item| item.as_object().unwrap().len())
        .sum();
    assert_eq!(
        documented,
        ROUTES.len(),
        "OpenAPI documents operations not in the contract"
    );
    assert_eq!(
        spec["components"]["securitySchemes"]["bearer_auth"]["scheme"],
        "bearer"
    );
}

#[sqlx::test(migrations = "./migrations")]
async fn admin_only_routes_document_403(pool: PgPool) {
    let spec = spec(pool).await;

    for (method, path) in [
        ("post", "/tasks"),
        ("get", "/tasks"),
        ("post", "/tasks/assign"),
        ("get", "/users"),
        ("patch", "/tasks/{id}"),
    ] {
        assert!(
            spec["paths"][path][method]["responses"]["403"].is_object(),
            "{method} {path} must document 403"
        );
    }
}

/// Every documented route is actually mounted (the router and the docs agree).
#[sqlx::test(migrations = "./migrations")]
async fn every_documented_route_is_mounted(pool: PgPool) {
    let app = common::app(pool);

    for (method, path, _) in ROUTES {
        let concrete = path.replace("{id}", "00000000-0000-0000-0000-000000000000");
        let method = Method::from_bytes(method.to_uppercase().as_bytes()).unwrap();
        let body = (method != Method::GET).then(|| json!({}));
        let (status, _) = common::send(&app, method.clone(), &concrete, None, body).await;
        assert!(
            status != StatusCode::METHOD_NOT_ALLOWED
                && !(status == StatusCode::NOT_FOUND && *path != "/dev/email-logs/latest"),
            "{method} {path} is documented but not mounted (got {status})"
        );
    }
}
