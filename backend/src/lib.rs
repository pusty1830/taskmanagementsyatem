pub mod auth;
pub mod config;
pub mod domain;
pub mod dto;
pub mod email;
pub mod error;
pub mod extract;
pub mod openapi;
pub mod repositories;
pub mod routes;
pub mod services;
pub mod state;

use axum::{
    extract::DefaultBodyLimit,
    http::{header, HeaderValue, Method},
    routing::{get, post},
    Router,
};
use tower_http::{cors::CorsLayer, trace::TraceLayer};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use crate::{openapi::ApiDoc, state::AppState};

const MAX_BODY_BYTES: usize = 64 * 1024;

pub fn build_app(state: AppState) -> Router {
    let cors = CorsLayer::new()
        .allow_origin(
            state
                .config
                .cors_origin
                .parse::<HeaderValue>()
                .expect("CORS_ORIGIN must be a valid origin"),
        )
        .allow_methods([Method::GET, Method::POST, Method::PATCH, Method::OPTIONS])
        .allow_headers([header::AUTHORIZATION, header::CONTENT_TYPE]);

    let mut router = Router::new()
        .route("/health", get(routes::health::health))
        .route("/auth/login", post(routes::auth::login))
        .route("/auth/verify-2fa", post(routes::auth::verify_2fa))
        .route("/auth/me", get(routes::auth::me));

    if state.config.is_development() {
        router = router.merge(dev_routes());
    }

    router
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()))
        .layer(DefaultBodyLimit::max(MAX_BODY_BYTES))
        .layer(cors)
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}

/// Seed and dev helpers; mounted only when `APP_ENV=development`.
fn dev_routes() -> Router<AppState> {
    Router::new()
        .route("/seed/users", post(routes::seed::seed_users))
        .route("/dev/reset", post(routes::dev::reset))
        .route("/dev/email-logs/latest", get(routes::dev::latest_email))
}
