use std::collections::HashSet;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Deserializer, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;
use validator::{Validate, ValidationError};

use crate::domain::{Role, Task, TaskPriority, TaskStatus};

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct TaskDto {
    pub id: Uuid,
    #[schema(example = "Infiltrate SPECTRE HQ")]
    pub title: String,
    #[schema(example = "Recon only")]
    pub description: String,
    pub status: TaskStatus,
    pub priority: TaskPriority,
    /// Assignee email, or null when unassigned.
    #[schema(example = "jamesbond@example.com")]
    pub assigned_to: Option<String>,
    #[schema(example = "admin@example.com")]
    pub created_by: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<Task> for TaskDto {
    fn from(t: Task) -> Self {
        Self {
            id: t.id,
            title: t.title,
            description: t.description,
            status: t.status,
            priority: t.priority,
            assigned_to: t.assigned_to_email,
            created_by: t.created_by_email,
            created_at: t.created_at,
            updated_at: t.updated_at,
        }
    }
}

fn trimmed<'de, D: Deserializer<'de>>(d: D) -> Result<String, D::Error> {
    Ok(String::deserialize(d)?.trim().to_string())
}

fn trimmed_opt<'de, D: Deserializer<'de>>(d: D) -> Result<Option<String>, D::Error> {
    Ok(Option::<String>::deserialize(d)?.map(|s| s.trim().to_string()))
}

#[derive(Debug, Deserialize, Validate, ToSchema)]
#[schema(example = json!({"title": "Infiltrate SPECTRE HQ", "description": "Recon only", "priority": "high"}))]
pub struct CreateTaskRequest {
    #[serde(deserialize_with = "trimmed")]
    #[validate(length(min = 1, max = 200, message = "must be 1-200 characters"))]
    pub title: String,
    #[serde(default)]
    #[validate(length(max = 2000, message = "must be at most 2000 characters"))]
    pub description: Option<String>,
    /// Defaults to `medium`.
    pub priority: Option<TaskPriority>,
}

#[derive(Debug, Deserialize, Validate, ToSchema)]
#[schema(example = json!({"task_ids": ["00000000-0000-0000-0000-000000000000"], "assignee_email": "jamesbond@example.com"}))]
pub struct AssignTasksRequest {
    #[validate(
        length(min = 1, max = 100, message = "must contain 1-100 task ids"),
        custom(function = "no_duplicates")
    )]
    pub task_ids: Vec<Uuid>,
    #[serde(deserialize_with = "super::normalized_email")]
    #[validate(email(message = "must be a valid email address"))]
    pub assignee_email: String,
}

fn no_duplicates(ids: &[Uuid]) -> Result<(), ValidationError> {
    let unique: HashSet<_> = ids.iter().collect();
    if unique.len() == ids.len() {
        Ok(())
    } else {
        Err(ValidationError::new("duplicates").with_message("must not contain duplicates".into()))
    }
}

#[derive(Debug, Serialize, ToSchema)]
pub struct AssignTasksResponse {
    #[schema(example = "jamesbond@example.com")]
    pub assigned_to: String,
    pub task_ids: Vec<Uuid>,
    #[schema(example = 3)]
    pub updated_count: u64,
}

/// Admins may change any field; the assignee may change `status` only.
#[derive(Debug, Deserialize, Validate, ToSchema)]
#[validate(schema(function = "has_any_change"))]
#[schema(example = json!({"status": "in_progress"}))]
pub struct UpdateTaskRequest {
    #[serde(default, deserialize_with = "trimmed_opt")]
    #[validate(length(min = 1, max = 200, message = "must be 1-200 characters"))]
    pub title: Option<String>,
    #[validate(length(max = 2000, message = "must be at most 2000 characters"))]
    pub description: Option<String>,
    pub status: Option<TaskStatus>,
    pub priority: Option<TaskPriority>,
}

impl UpdateTaskRequest {
    pub fn is_status_only(&self) -> bool {
        self.status.is_some()
            && self.title.is_none()
            && self.description.is_none()
            && self.priority.is_none()
    }
}

fn has_any_change(req: &UpdateTaskRequest) -> Result<(), ValidationError> {
    if req.title.is_none()
        && req.description.is_none()
        && req.status.is_none()
        && req.priority.is_none()
    {
        return Err(
            ValidationError::new("empty").with_message("at least one field is required".into())
        );
    }
    Ok(())
}

/// The cacheable part of `view-my-tasks` (user and cache metadata are added per response).
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct MyTasksPayload {
    pub tasks: Vec<TaskDto>,
    pub summary: TaskSummary,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct TaskSummary {
    #[schema(example = 3)]
    pub total_assigned_tasks: usize,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct MyTasksUser {
    #[schema(example = "jamesbond@example.com")]
    pub email: String,
    pub role: Role,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct CacheMeta {
    /// false on the first call (loaded from the database), true when served from cache.
    pub hit: bool,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct MyTasksResponse {
    pub user: MyTasksUser,
    pub tasks: Vec<TaskDto>,
    pub summary: TaskSummary,
    pub cache: CacheMeta,
}
