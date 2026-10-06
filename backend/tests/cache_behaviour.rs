mod common;

use std::sync::Arc;

use async_trait::async_trait;
use axum::http::StatusCode;
use common::{JAMES_EMAIL, JAMES_PASSWORD};
use serde_json::json;
use sqlx::PgPool;
use task_api::{auth::password::hash_password, cache::TaskCache, dto::task::MyTasksPayload};
use uuid::Uuid;

async fn my_tasks(app: &axum::Router, token: &str) -> serde_json::Value {
    let (status, body) = common::get(app, "/tasks/view-my-tasks", Some(token)).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    body
}

#[sqlx::test(migrations = "./migrations")]
async fn view_my_tasks_second_call_is_cache_hit(pool: PgPool) {
    let app = common::app(pool);
    let (admin, james) = common::seeded_tokens(&app).await;
    let id = common::create_task(&app, &admin, "Mission", "high").await;
    common::assign(&app, &admin, &[id], JAMES_EMAIL).await;

    let first = my_tasks(&app, &james).await;
    let second = my_tasks(&app, &james).await;

    assert_eq!(first["cache"]["hit"], false);
    assert_eq!(second["cache"]["hit"], true);
    assert_eq!(first["tasks"], second["tasks"]);
    assert_eq!(second["user"]["email"], JAMES_EMAIL);
}

#[sqlx::test(migrations = "./migrations")]
async fn cache_is_per_user(pool: PgPool) {
    let app = common::app(pool);
    let (admin, james) = common::seeded_tokens(&app).await;

    my_tasks(&app, &admin).await;
    let james_first = my_tasks(&app, &james).await;

    assert_eq!(
        james_first["cache"]["hit"], false,
        "admin's entry must not serve James"
    );
}

#[sqlx::test(migrations = "./migrations")]
async fn assign_invalidates_assignee_cache(pool: PgPool) {
    let app = common::app(pool);
    let (admin, james) = common::seeded_tokens(&app).await;
    let a = common::create_task(&app, &admin, "A", "high").await;
    let b = common::create_task(&app, &admin, "B", "low").await;
    common::assign(&app, &admin, &[a], JAMES_EMAIL).await;
    my_tasks(&app, &james).await;
    assert_eq!(my_tasks(&app, &james).await["cache"]["hit"], true);

    common::assign(&app, &admin, &[b], JAMES_EMAIL).await;
    let after = my_tasks(&app, &james).await;

    assert_eq!(after["cache"]["hit"], false);
    assert_eq!(after["summary"]["total_assigned_tasks"], 2);
}

#[sqlx::test(migrations = "./migrations")]
async fn reassign_invalidates_previous_assignee_cache(pool: PgPool) {
    let app = common::app(pool.clone());
    let (admin, james) = common::seeded_tokens(&app).await;
    sqlx::query(
        "INSERT INTO users (full_name, email, hashed_password, role) VALUES ('Moneypenny', 'moneypenny@example.com', $1, 'staff')",
    )
    .bind(hash_password("Penny@123").unwrap())
    .execute(&pool)
    .await
    .unwrap();
    let id = common::create_task(&app, &admin, "Shared", "medium").await;
    common::assign(&app, &admin, std::slice::from_ref(&id), JAMES_EMAIL).await;
    my_tasks(&app, &james).await;
    assert_eq!(my_tasks(&app, &james).await["cache"]["hit"], true);

    common::assign(&app, &admin, &[id], "moneypenny@example.com").await;
    let james_after = my_tasks(&app, &james).await;

    assert_eq!(james_after["cache"]["hit"], false);
    assert_eq!(james_after["summary"]["total_assigned_tasks"], 0);
}

#[sqlx::test(migrations = "./migrations")]
async fn status_update_invalidates_cache(pool: PgPool) {
    let app = common::app(pool);
    let (admin, james) = common::seeded_tokens(&app).await;
    let id = common::create_task(&app, &admin, "Mission", "high").await;
    common::assign(&app, &admin, std::slice::from_ref(&id), JAMES_EMAIL).await;
    my_tasks(&app, &james).await;
    assert_eq!(my_tasks(&app, &james).await["cache"]["hit"], true);

    let (status, _) = common::patch(
        &app,
        &format!("/tasks/{id}"),
        Some(&james),
        json!({ "status": "done" }),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let after = my_tasks(&app, &james).await;

    assert_eq!(after["cache"]["hit"], false);
    assert_eq!(after["tasks"][0]["status"], "done");
}

#[sqlx::test(migrations = "./migrations")]
async fn dev_reset_clears_cache(pool: PgPool) {
    let app = common::app(pool);
    let (admin, james) = common::seeded_tokens(&app).await;
    let id = common::create_task(&app, &admin, "Mission", "high").await;
    common::assign(&app, &admin, &[id], JAMES_EMAIL).await;
    my_tasks(&app, &james).await;

    common::post(&app, "/dev/reset", None, json!({})).await;
    let after = my_tasks(&app, &james).await;

    assert_eq!(after["cache"]["hit"], false);
    assert_eq!(after["summary"]["total_assigned_tasks"], 0);
}

/// A cache backend that is always down (e.g. Redis unreachable).
struct BrokenCache;

#[async_trait]
impl TaskCache for BrokenCache {
    async fn get_my_tasks(&self, _: Uuid) -> anyhow::Result<Option<MyTasksPayload>> {
        anyhow::bail!("cache down")
    }
    async fn set_my_tasks(&self, _: Uuid, _: &MyTasksPayload) -> anyhow::Result<()> {
        anyhow::bail!("cache down")
    }
    async fn invalidate(&self, _: &[Uuid]) -> anyhow::Result<()> {
        anyhow::bail!("cache down")
    }
    async fn clear(&self) -> anyhow::Result<()> {
        anyhow::bail!("cache down")
    }
}

#[sqlx::test(migrations = "./migrations")]
async fn cache_outage_falls_back_to_database(pool: PgPool) {
    let app = common::app_with_cache(pool, common::test_config(), Arc::new(BrokenCache));
    common::seed(&app).await;
    let admin = common::login(&app, common::ADMIN_EMAIL, common::ADMIN_PASSWORD).await;
    let james = common::login(&app, JAMES_EMAIL, JAMES_PASSWORD).await;
    let id = common::create_task(&app, &admin, "Mission", "high").await;
    let (assign_status, _) = common::assign(&app, &admin, &[id], JAMES_EMAIL).await;
    assert_eq!(
        assign_status,
        StatusCode::OK,
        "writes must not fail when the cache is down"
    );

    let first = my_tasks(&app, &james).await;
    let second = my_tasks(&app, &james).await;

    assert_eq!(first["summary"]["total_assigned_tasks"], 1);
    assert_eq!(first["cache"]["hit"], false);
    assert_eq!(second["cache"]["hit"], false);
}
