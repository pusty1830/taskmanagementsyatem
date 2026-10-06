use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use uuid::Uuid;

use crate::{
    auth::extractor::{AdminUser, AuthUser},
    dto::task::{
        AssignTasksRequest, AssignTasksResponse, CacheMeta, CreateTaskRequest, MyTasksResponse,
        MyTasksUser, TaskDto, UpdateTaskRequest,
    },
    error::{AppResult, ErrorResponse},
    extract::ValidatedJson,
    services::task_service,
    state::AppState,
};

/// Create a task (admin only).
#[utoipa::path(
    post,
    path = "/tasks",
    tag = "tasks",
    security(("bearer_auth" = [])),
    description = "Admin only. Staff users (e.g. James Bond) receive **403 Forbidden**. \
        `status` starts as `todo`; `priority` defaults to `medium`.",
    request_body = CreateTaskRequest,
    responses(
        (status = 201, description = "Task created", body = TaskDto),
        (status = 400, description = "Invalid request body", body = ErrorResponse),
        (status = 401, description = "Missing or invalid token", body = ErrorResponse),
        (status = 403, description = "Caller is not an admin", body = ErrorResponse),
    )
)]
pub async fn create_task(
    State(state): State<AppState>,
    AdminUser(admin): AdminUser,
    ValidatedJson(req): ValidatedJson<CreateTaskRequest>,
) -> AppResult<(StatusCode, Json<TaskDto>)> {
    let task = task_service::create_task(&state, &admin, req).await?;
    Ok((StatusCode::CREATED, Json(task)))
}

/// List all tasks (admin only).
#[utoipa::path(
    get,
    path = "/tasks",
    tag = "tasks",
    security(("bearer_auth" = [])),
    responses(
        (status = 200, description = "All tasks, oldest first", body = [TaskDto]),
        (status = 401, description = "Missing or invalid token", body = ErrorResponse),
        (status = 403, description = "Caller is not an admin", body = ErrorResponse),
    )
)]
pub async fn list_tasks(
    State(state): State<AppState>,
    _admin: AdminUser,
) -> AppResult<Json<Vec<TaskDto>>> {
    Ok(Json(task_service::list_tasks(&state).await?))
}

/// Assign selected tasks to a user (admin only).
#[utoipa::path(
    post,
    path = "/tasks/assign",
    tag = "tasks",
    security(("bearer_auth" = [])),
    description = "All-or-nothing: if any task id does not exist, nothing is assigned. \
        Invalidates the cached task list of the new assignee and of any previous assignees.",
    request_body = AssignTasksRequest,
    responses(
        (status = 200, description = "Tasks assigned", body = AssignTasksResponse),
        (status = 400, description = "Empty or duplicate task ids, invalid email", body = ErrorResponse),
        (status = 401, description = "Missing or invalid token", body = ErrorResponse),
        (status = 403, description = "Caller is not an admin", body = ErrorResponse),
        (status = 404, description = "Unknown task id or assignee", body = ErrorResponse),
    )
)]
pub async fn assign_tasks(
    State(state): State<AppState>,
    _admin: AdminUser,
    ValidatedJson(req): ValidatedJson<AssignTasksRequest>,
) -> AppResult<Json<AssignTasksResponse>> {
    Ok(Json(task_service::assign_tasks(&state, req).await?))
}

/// Update a task (admin: any field; assignee: status only).
#[utoipa::path(
    patch,
    path = "/tasks/{id}",
    tag = "tasks",
    security(("bearer_auth" = [])),
    description = "Invalidates the assignee's cached task list.",
    params(("id" = Uuid, Path, description = "Task id")),
    request_body = UpdateTaskRequest,
    responses(
        (status = 200, description = "Updated task", body = TaskDto),
        (status = 400, description = "Invalid or empty body", body = ErrorResponse),
        (status = 401, description = "Missing or invalid token", body = ErrorResponse),
        (status = 403, description = "Not allowed to make this change", body = ErrorResponse),
        (status = 404, description = "Task not found", body = ErrorResponse),
    )
)]
pub async fn update_task(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<Uuid>,
    ValidatedJson(req): ValidatedJson<UpdateTaskRequest>,
) -> AppResult<Json<TaskDto>> {
    Ok(Json(
        task_service::update_task(&state, &user, id, req).await?,
    ))
}

/// Tasks assigned to the logged-in user, with cache metadata.
#[utoipa::path(
    get,
    path = "/tasks/view-my-tasks",
    tag = "tasks",
    security(("bearer_auth" = [])),
    description = "Per-user cached (cache-aside). First call: `cache.hit = false` (loaded from the database); \
        an identical second call: `cache.hit = true`. Assigning or updating tasks invalidates the affected users.",
    responses(
        (status = 200, description = "Assigned tasks (high priority first)", body = MyTasksResponse,
            example = json!({
                "user": {"email": "jamesbond@example.com", "role": "staff"},
                "tasks": [
                    {"id": "6f0c...", "title": "Infiltrate SPECTRE HQ", "description": "", "status": "todo", "priority": "high",
                     "assigned_to": "jamesbond@example.com", "created_by": "admin@example.com",
                     "created_at": "2026-10-06T10:00:00Z", "updated_at": "2026-10-06T10:01:00Z"}
                ],
                "summary": {"total_assigned_tasks": 1},
                "cache": {"hit": false}
            })),
        (status = 401, description = "Missing or invalid token", body = ErrorResponse),
    )
)]
pub async fn view_my_tasks(
    State(state): State<AppState>,
    user: AuthUser,
) -> AppResult<Json<MyTasksResponse>> {
    let (payload, hit) = task_service::my_tasks(&state, &user).await?;
    Ok(Json(MyTasksResponse {
        user: MyTasksUser {
            email: user.email,
            role: user.role,
        },
        tasks: payload.tasks,
        summary: payload.summary,
        cache: CacheMeta { hit },
    }))
}
