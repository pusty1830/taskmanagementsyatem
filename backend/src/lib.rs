pub mod config;
pub mod error;
pub mod openapi;
pub mod routes;
pub mod state;

use axum::{
    extract::DefaultBodyLimit,
    http::{header, HeaderValue, Method},
    routing::get,
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

    Router::new()
        .route("/health", get(routes::health::health))
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()))
        .layer(DefaultBodyLimit::max(MAX_BODY_BYTES))
        .layer(cors)
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}
