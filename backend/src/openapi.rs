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
        routes::seed::seed_users,
        routes::dev::reset,
        routes::dev::latest_email,
        routes::auth::login,
        routes::auth::verify_2fa,
        routes::auth::me,
        routes::users::list_users,
        routes::tasks::create_task,
        routes::tasks::list_tasks,
        routes::tasks::assign_tasks,
        routes::tasks::update_task,
        routes::tasks::view_my_tasks,
    ),
    components(schemas(
        error::ErrorResponse,
        error::ErrorBody,
        routes::health::HealthResponse,
        routes::seed::SeedResponse,
        routes::dev::MessageResponse,
        crate::dto::user::UserDto,
        routes::dev::EmailLogDto,
        crate::domain::Role,
        crate::dto::auth::LoginRequest,
        crate::dto::auth::LoginResponse,
        crate::dto::auth::Verify2faRequest,
        crate::dto::auth::TokenResponse,
        crate::domain::TaskStatus,
        crate::domain::TaskPriority,
        crate::dto::task::TaskDto,
        crate::dto::task::CreateTaskRequest,
        crate::dto::task::AssignTasksRequest,
        crate::dto::task::AssignTasksResponse,
        crate::dto::task::UpdateTaskRequest,
        crate::dto::task::MyTasksResponse,
        crate::dto::task::MyTasksUser,
        crate::dto::task::TaskSummary,
        crate::dto::task::CacheMeta,
    )),
    modifiers(&BearerAuth),
    tags(
        (name = "health", description = "Liveness"),
        (name = "seed", description = "Create validation users (development only)"),
        (name = "dev", description = "Development helpers: dev mailbox, reset (development only)"),
        (name = "auth", description = "Email + password login with email 2FA, then JWT"),
        (name = "users", description = "User directory (admin only)"),
        (name = "tasks", description = "Task management with role-based access and per-user caching"),
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
