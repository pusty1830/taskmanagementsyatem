mod common;

use axum::http::StatusCode;
use common::{ADMIN_EMAIL, JAMES_EMAIL};
use serde_json::json;
use sqlx::PgPool;
use uuid::Uuid;

async fn task_count(pool: &PgPool) -> i64 {
    sqlx::query_scalar("SELECT count(*) FROM tasks")
        .fetch_one(pool)
        .await
        .unwrap()
}

#[sqlx::test(migrations = "./migrations")]
async fn admin_creates_five_tasks(pool: PgPool) {
    let app = common::app(pool.clone());
    let (admin, _) = common::seeded_tokens(&app).await;

    let (status, first) = common::post(
        &app,
        "/tasks",
        Some(&admin),
        json!({ "title": "  Infiltrate SPECTRE HQ  ", "description": "Recon only", "priority": "high" }),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(first["title"], "Infiltrate SPECTRE HQ", "title is trimmed");
    assert_eq!(first["status"], "todo");
    assert_eq!(first["priority"], "high");
    assert_eq!(first["created_by"], ADMIN_EMAIL);
    assert!(first["assigned_to"].is_null());

    for i in 2..=5 {
        common::create_task(&app, &admin, &format!("Task {i}"), "medium").await;
    }

    let (status, all) = common::get(&app, "/tasks", Some(&admin)).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(all.as_array().unwrap().len(), 5);
    assert_eq!(task_count(&pool).await, 5);
}

#[sqlx::test(migrations = "./migrations")]
async fn priority_defaults_to_medium(pool: PgPool) {
    let app = common::app(pool);
    let (admin, _) = common::seeded_tokens(&app).await;

    let (status, body) =
        common::post(&app, "/tasks", Some(&admin), json!({ "title": "Minimal" })).await;

    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(body["priority"], "medium");
    assert_eq!(body["description"], "");
}

#[sqlx::test(migrations = "./migrations")]
async fn create_task_validates_input(pool: PgPool) {
    let app = common::app(pool.clone());
    let (admin, _) = common::seeded_tokens(&app).await;

    for bad in [
        json!({ "title": "" }),
        json!({ "title": "   " }),
        json!({ "title": "x".repeat(201) }),
        json!({ "title": "ok", "priority": "urgent" }),
        json!({ "title": "ok", "description": "d".repeat(2001) }),
    ] {
        let (status, body) = common::post(&app, "/tasks", Some(&admin), bad.clone()).await;
        assert_eq!(status, StatusCode::BAD_REQUEST, "expected 400 for {bad}");
        assert_eq!(body["error"]["code"], "validation_error");
    }
    assert_eq!(task_count(&pool).await, 0);
}

#[sqlx::test(migrations = "./migrations")]
async fn staff_create_task_is_forbidden(pool: PgPool) {
    let app = common::app(pool.clone());
    let (_, james) = common::seeded_tokens(&app).await;

    let (status, body) = common::post(
        &app,
        "/tasks",
        Some(&james),
        json!({ "title": "James tries", "priority": "high" }),
    )
    .await;

    assert_eq!(status, StatusCode::FORBIDDEN);
    assert_eq!(body["error"]["code"], "forbidden");
    assert_eq!(body["error"]["message"], "Only admins can create tasks");
    assert_eq!(task_count(&pool).await, 0);
}

#[sqlx::test(migrations = "./migrations")]
async fn task_endpoints_require_token(pool: PgPool) {
    let app = common::app(pool);

    let (create, _) = common::post(&app, "/tasks", None, json!({ "title": "x" })).await;
    let (mine, _) = common::get(&app, "/tasks/view-my-tasks", None).await;

    assert_eq!(create, StatusCode::UNAUTHORIZED);
    assert_eq!(mine, StatusCode::UNAUTHORIZED);
}

#[sqlx::test(migrations = "./migrations")]
async fn staff_cannot_list_all_tasks_or_users(pool: PgPool) {
    let app = common::app(pool);
    let (_, james) = common::seeded_tokens(&app).await;

    let (tasks, _) = common::get(&app, "/tasks", Some(&james)).await;
    let (users, _) = common::get(&app, "/users?role=staff", Some(&james)).await;

    assert_eq!(tasks, StatusCode::FORBIDDEN);
    assert_eq!(users, StatusCode::FORBIDDEN);
}

#[sqlx::test(migrations = "./migrations")]
async fn admin_lists_staff_users(pool: PgPool) {
    let app = common::app(pool);
    let (admin, _) = common::seeded_tokens(&app).await;

    let (status, staff) = common::get(&app, "/users?role=staff", Some(&admin)).await;
    let (_, everyone) = common::get(&app, "/users", Some(&admin)).await;

    assert_eq!(status, StatusCode::OK);
    let staff = staff.as_array().unwrap();
    assert_eq!(staff.len(), 1);
    assert_eq!(staff[0]["email"], JAMES_EMAIL);
    assert_eq!(everyone.as_array().unwrap().len(), 2);
}

#[sqlx::test(migrations = "./migrations")]
async fn admin_assigns_three_tasks_to_james(pool: PgPool) {
    let app = common::app(pool.clone());
    let (admin, _) = common::seeded_tokens(&app).await;
    let mut ids = Vec::new();
    for i in 1..=5 {
        ids.push(common::create_task(&app, &admin, &format!("Task {i}"), "low").await);
    }

    let (status, body) = common::assign(&app, &admin, &ids[..3], JAMES_EMAIL).await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["updated_count"], 3);
    assert_eq!(body["assigned_to"], JAMES_EMAIL);
    let assigned: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM tasks t JOIN users u ON u.id = t.assigned_to_id WHERE u.email = $1",
    )
    .bind(JAMES_EMAIL)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(assigned, 3);
}

#[sqlx::test(migrations = "./migrations")]
async fn assign_with_unknown_task_changes_nothing(pool: PgPool) {
    let app = common::app(pool.clone());
    let (admin, _) = common::seeded_tokens(&app).await;
    let real = common::create_task(&app, &admin, "Real", "low").await;

    let (status, body) = common::assign(
        &app,
        &admin,
        &[real, Uuid::new_v4().to_string()],
        JAMES_EMAIL,
    )
    .await;

    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_eq!(body["error"]["code"], "not_found");
    let assigned: i64 =
        sqlx::query_scalar("SELECT count(*) FROM tasks WHERE assigned_to_id IS NOT NULL")
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(assigned, 0, "assignment must be all-or-nothing");
}

#[sqlx::test(migrations = "./migrations")]
async fn assign_to_unknown_user_is_404(pool: PgPool) {
    let app = common::app(pool);
    let (admin, _) = common::seeded_tokens(&app).await;
    let id = common::create_task(&app, &admin, "Task", "low").await;

    let (status, _) = common::assign(&app, &admin, &[id], "ghost@example.com").await;

    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[sqlx::test(migrations = "./migrations")]
async fn assign_rejects_empty_or_duplicate_ids(pool: PgPool) {
    let app = common::app(pool);
    let (admin, _) = common::seeded_tokens(&app).await;
    let id = common::create_task(&app, &admin, "Task", "low").await;

    let (empty, _) = common::assign(&app, &admin, &[], JAMES_EMAIL).await;
    let (dupes, _) = common::assign(&app, &admin, &[id.clone(), id], JAMES_EMAIL).await;

    assert_eq!(empty, StatusCode::BAD_REQUEST);
    assert_eq!(dupes, StatusCode::BAD_REQUEST);
}

#[sqlx::test(migrations = "./migrations")]
async fn staff_assign_is_forbidden(pool: PgPool) {
    let app = common::app(pool);
    let (admin, james) = common::seeded_tokens(&app).await;
    let id = common::create_task(&app, &admin, "Task", "low").await;

    let (status, body) = common::assign(&app, &james, &[id], JAMES_EMAIL).await;

    assert_eq!(status, StatusCode::FORBIDDEN);
    assert_eq!(body["error"]["message"], "Only admins can assign tasks");
}

#[sqlx::test(migrations = "./migrations")]
async fn james_views_exactly_three_tasks(pool: PgPool) {
    let app = common::app(pool);
    let (admin, james) = common::seeded_tokens(&app).await;
    let low = common::create_task(&app, &admin, "Low task", "low").await;
    let high = common::create_task(&app, &admin, "High task", "high").await;
    let _unassigned1 = common::create_task(&app, &admin, "Unassigned 1", "high").await;
    let medium = common::create_task(&app, &admin, "Medium task", "medium").await;
    let _unassigned2 = common::create_task(&app, &admin, "Unassigned 2", "low").await;
    common::assign(&app, &admin, &[low, high, medium], JAMES_EMAIL).await;

    let (status, body) = common::get(&app, "/tasks/view-my-tasks", Some(&james)).await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(
        body["user"],
        json!({ "email": JAMES_EMAIL, "role": "staff" })
    );
    assert_eq!(body["summary"]["total_assigned_tasks"], 3);
    let tasks = body["tasks"].as_array().unwrap();
    assert_eq!(tasks.len(), 3);
    let priorities: Vec<&str> = tasks
        .iter()
        .map(|t| t["priority"].as_str().unwrap())
        .collect();
    assert_eq!(priorities, ["high", "medium", "low"]);
    for t in tasks {
        assert_eq!(t["assigned_to"], JAMES_EMAIL);
        assert_eq!(t["status"], "todo");
        assert!(t["id"].is_string() && t["title"].is_string());
    }
    assert!(body["cache"]["hit"].is_boolean());
}

#[sqlx::test(migrations = "./migrations")]
async fn users_only_see_their_own_tasks(pool: PgPool) {
    let app = common::app(pool);
    let (admin, _) = common::seeded_tokens(&app).await;
    let id = common::create_task(&app, &admin, "For James", "high").await;
    common::assign(&app, &admin, &[id], JAMES_EMAIL).await;

    let (_, admin_view) = common::get(&app, "/tasks/view-my-tasks", Some(&admin)).await;

    assert_eq!(admin_view["summary"]["total_assigned_tasks"], 0);
    assert_eq!(admin_view["tasks"], json!([]));
}

#[sqlx::test(migrations = "./migrations")]
async fn assignee_can_update_status_only(pool: PgPool) {
    let app = common::app(pool);
    let (admin, james) = common::seeded_tokens(&app).await;
    let id = common::create_task(&app, &admin, "Mission", "high").await;
    common::assign(&app, &admin, std::slice::from_ref(&id), JAMES_EMAIL).await;

    let (ok, updated) = common::patch(
        &app,
        &format!("/tasks/{id}"),
        Some(&james),
        json!({ "status": "in_progress" }),
    )
    .await;
    let (forbidden, _) = common::patch(
        &app,
        &format!("/tasks/{id}"),
        Some(&james),
        json!({ "title": "Renamed" }),
    )
    .await;

    assert_eq!(ok, StatusCode::OK);
    assert_eq!(updated["status"], "in_progress");
    assert_eq!(forbidden, StatusCode::FORBIDDEN);
}

#[sqlx::test(migrations = "./migrations")]
async fn staff_cannot_update_unassigned_task(pool: PgPool) {
    let app = common::app(pool);
    let (admin, james) = common::seeded_tokens(&app).await;
    let id = common::create_task(&app, &admin, "Not yours", "high").await;

    let (status, _) = common::patch(
        &app,
        &format!("/tasks/{id}"),
        Some(&james),
        json!({ "status": "done" }),
    )
    .await;

    assert_eq!(status, StatusCode::FORBIDDEN);
}

#[sqlx::test(migrations = "./migrations")]
async fn admin_can_update_any_field(pool: PgPool) {
    let app = common::app(pool);
    let (admin, _) = common::seeded_tokens(&app).await;
    let id = common::create_task(&app, &admin, "Draft", "low").await;

    let (status, body) = common::patch(
        &app,
        &format!("/tasks/{id}"),
        Some(&admin),
        json!({ "title": "Final", "priority": "high", "status": "done", "description": "updated" }),
    )
    .await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["title"], "Final");
    assert_eq!(body["priority"], "high");
    assert_eq!(body["status"], "done");
    assert_eq!(body["description"], "updated");
}

#[sqlx::test(migrations = "./migrations")]
async fn update_validates_and_404s(pool: PgPool) {
    let app = common::app(pool);
    let (admin, _) = common::seeded_tokens(&app).await;
    let id = common::create_task(&app, &admin, "Task", "low").await;

    let (empty, _) = common::patch(&app, &format!("/tasks/{id}"), Some(&admin), json!({})).await;
    let (missing, _) = common::patch(
        &app,
        &format!("/tasks/{}", Uuid::new_v4()),
        Some(&admin),
        json!({ "status": "done" }),
    )
    .await;

    assert_eq!(empty, StatusCode::BAD_REQUEST);
    assert_eq!(missing, StatusCode::NOT_FOUND);
}
