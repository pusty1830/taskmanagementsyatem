use utoipa::{
    openapi::security::{HttpAuthScheme, HttpBuilder, SecurityScheme},
    Modify, OpenApi,
};

use crate::{error, routes};

#[derive(OpenApi)]
#[openapi(
    info(
        title = "Task Management API",
        version = "0.1.0",
        description = "Rust (Axum) task manager with email 2FA, JWT auth, role-based access and per-user caching.\n\n\
            **Validation flow:** seed → auth/login → dev/email-logs/latest → auth/verify-2fa → \
            click *Authorize* and paste the token → tasks."
    ),
    paths(
        routes::health::health,
    ),
    components(schemas(
        error::ErrorResponse,
        error::ErrorBody,
        routes::health::HealthResponse,
    )),
    modifiers(&BearerAuth),
    tags(
        (name = "health", description = "Liveness"),
    )
)]
pub struct ApiDoc;

/// Registers the `bearer_auth` scheme used by every protected route.
struct BearerAuth;

impl Modify for BearerAuth {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        let components = openapi.components.get_or_insert_with(Default::default);
        components.add_security_scheme(
            "bearer_auth",
            SecurityScheme::Http(
                HttpBuilder::new()
                    .scheme(HttpAuthScheme::Bearer)
                    .bearer_format("JWT")
                    .build(),
            ),
        );
    }
}
