use std::collections::BTreeSet;

use uuid::Uuid;

use crate::{
    auth::extractor::AuthUser,
    domain::TaskPriority,
    dto::task::{
        AssignTasksRequest, AssignTasksResponse, CreateTaskRequest, MyTasksPayload, TaskDto,
        TaskSummary, UpdateTaskRequest,
    },
    error::{AppError, AppResult},
    repositories::{
        task_repo::{self, TaskChanges},
        user_repo,
    },
    state::AppState,
};

pub async fn create_task(
    state: &AppState,
    admin: &AuthUser,
    req: CreateTaskRequest,
) -> AppResult<TaskDto> {
    let task = task_repo::insert(
        &state.db,
        &req.title,
        req.description.as_deref().unwrap_or(""),
        req.priority.unwrap_or(TaskPriority::Medium),
        admin.id,
    )
    .await?;
    Ok(task.into())
}

pub async fn list_tasks(state: &AppState) -> AppResult<Vec<TaskDto>> {
    let tasks = task_repo::list_all(&state.db).await?;
    Ok(tasks.into_iter().map(TaskDto::from).collect())
}

/// Assigns every task in the request or none of them (single transaction, rows locked).
pub async fn assign_tasks(
    state: &AppState,
    req: AssignTasksRequest,
) -> AppResult<AssignTasksResponse> {
    let assignee = user_repo::find_by_email(&state.db, &req.assignee_email)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("User {} not found", req.assignee_email)))?;

    let mut tx = state.db.begin().await?;
    let locked = task_repo::lock_for_assignment(&mut tx, &req.task_ids).await?;
    if locked.len() != req.task_ids.len() {
        let found: BTreeSet<Uuid> = locked.iter().map(|(id, _)| *id).collect();
        let missing: Vec<String> = req
            .task_ids
            .iter()
            .filter(|id| !found.contains(id))
            .map(Uuid::to_string)
            .collect();
        return Err(AppError::NotFound(format!(
            "Tasks not found: {}",
            missing.join(", ")
        )));
    }
    let updated_count = task_repo::assign(&mut tx, &req.task_ids, assignee.id).await?;
    tx.commit().await?;

    Ok(AssignTasksResponse {
        assigned_to: assignee.email,
        task_ids: req.task_ids,
        updated_count,
    })
}

pub async fn update_task(
    state: &AppState,
    user: &AuthUser,
    task_id: Uuid,
    req: UpdateTaskRequest,
) -> AppResult<TaskDto> {
    let task = task_repo::find_by_id(&state.db, task_id)
        .await?
        .ok_or_else(|| AppError::NotFound("Task not found".into()))?;

    if !user.is_admin() {
        let is_assignee = task.assigned_to_id == Some(user.id);
        if !is_assignee {
            return Err(AppError::Forbidden(
                "You can only update tasks assigned to you".into(),
            ));
        }
        if !req.is_status_only() {
            return Err(AppError::Forbidden(
                "Staff can only change the status of their tasks".into(),
            ));
        }
    }

    let changes = TaskChanges {
        title: req.title.as_deref(),
        description: req.description.as_deref(),
        status: req.status,
        priority: req.priority,
    };
    let updated = task_repo::update(&state.db, task_id, changes)
        .await?
        .ok_or_else(|| AppError::NotFound("Task not found".into()))?;
    Ok(updated.into())
}

/// Tasks assigned to `user`, plus whether the result came from cache.
pub async fn my_tasks(state: &AppState, user: &AuthUser) -> AppResult<(MyTasksPayload, bool)> {
    let tasks: Vec<TaskDto> = task_repo::list_assigned_to(&state.db, user.id)
        .await?
        .into_iter()
        .map(TaskDto::from)
        .collect();
    let payload = MyTasksPayload {
        summary: TaskSummary {
            total_assigned_tasks: tasks.len(),
        },
        tasks,
    };
    Ok((payload, false))
}
