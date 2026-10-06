use sqlx::{PgConnection, PgPool};
use uuid::Uuid;

use crate::domain::{Task, TaskPriority, TaskStatus};

/// Joined projection shared by every read, so all callers get the same `Task` shape.
const SELECT_TASK: &str = "
    SELECT t.id, t.title, t.description, t.status, t.priority,
           t.created_by_id, t.assigned_to_id, t.created_at, t.updated_at,
           creator.email AS created_by_email,
           assignee.email AS assigned_to_email
    FROM tasks t
    JOIN users creator ON creator.id = t.created_by_id
    LEFT JOIN users assignee ON assignee.id = t.assigned_to_id";

pub async fn insert(
    db: &PgPool,
    title: &str,
    description: &str,
    priority: TaskPriority,
    created_by_id: Uuid,
) -> sqlx::Result<Task> {
    let id: Uuid = sqlx::query_scalar(
        "INSERT INTO tasks (title, description, priority, created_by_id)
         VALUES ($1, $2, $3, $4) RETURNING id",
    )
    .bind(title)
    .bind(description)
    .bind(priority)
    .bind(created_by_id)
    .fetch_one(db)
    .await?;

    find_by_id(db, id).await?.ok_or(sqlx::Error::RowNotFound)
}

pub async fn find_by_id(db: &PgPool, id: Uuid) -> sqlx::Result<Option<Task>> {
    sqlx::query_as::<_, Task>(&format!("{SELECT_TASK} WHERE t.id = $1"))
        .bind(id)
        .fetch_optional(db)
        .await
}

pub async fn list_all(db: &PgPool) -> sqlx::Result<Vec<Task>> {
    sqlx::query_as::<_, Task>(&format!("{SELECT_TASK} ORDER BY t.created_at, t.id"))
        .fetch_all(db)
        .await
}

/// Highest priority first (enum order is low < medium < high), then oldest first.
pub async fn list_assigned_to(db: &PgPool, user_id: Uuid) -> sqlx::Result<Vec<Task>> {
    sqlx::query_as::<_, Task>(&format!(
        "{SELECT_TASK} WHERE t.assigned_to_id = $1
         ORDER BY t.priority DESC, t.created_at, t.id"
    ))
    .bind(user_id)
    .fetch_all(db)
    .await
}

/// Locks the given tasks and returns `(task_id, current_assignee)` for each one that exists.
pub async fn lock_for_assignment(
    conn: &mut PgConnection,
    ids: &[Uuid],
) -> sqlx::Result<Vec<(Uuid, Option<Uuid>)>> {
    sqlx::query_as::<_, (Uuid, Option<Uuid>)>(
        "SELECT id, assigned_to_id FROM tasks WHERE id = ANY($1) FOR UPDATE",
    )
    .bind(ids)
    .fetch_all(conn)
    .await
}

pub async fn assign(conn: &mut PgConnection, ids: &[Uuid], assignee_id: Uuid) -> sqlx::Result<u64> {
    let result =
        sqlx::query("UPDATE tasks SET assigned_to_id = $1, updated_at = now() WHERE id = ANY($2)")
            .bind(assignee_id)
            .bind(ids)
            .execute(conn)
            .await?;
    Ok(result.rows_affected())
}

/// Partial update: `None` fields keep their current value.
pub struct TaskChanges<'a> {
    pub title: Option<&'a str>,
    pub description: Option<&'a str>,
    pub status: Option<TaskStatus>,
    pub priority: Option<TaskPriority>,
}

pub async fn update(db: &PgPool, id: Uuid, changes: TaskChanges<'_>) -> sqlx::Result<Option<Task>> {
    let updated = sqlx::query(
        "UPDATE tasks SET
            title       = COALESCE($2, title),
            description = COALESCE($3, description),
            status      = COALESCE($4, status),
            priority    = COALESCE($5, priority),
            updated_at  = now()
         WHERE id = $1",
    )
    .bind(id)
    .bind(changes.title)
    .bind(changes.description)
    .bind(changes.status)
    .bind(changes.priority)
    .execute(db)
    .await?;

    if updated.rows_affected() == 0 {
        return Ok(None);
    }
    find_by_id(db, id).await
}
