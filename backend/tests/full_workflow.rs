//! The assignment's required validation flow, steps 1-11, in one test.

mod common;

use axum::http::StatusCode;
use common::{ADMIN_EMAIL, ADMIN_PASSWORD, JAMES_EMAIL, JAMES_PASSWORD};
use serde_json::json;
use sqlx::PgPool;

#[sqlx::test(migrations = "./migrations")]
async fn full_assignment_workflow(pool: PgPool) {
    let app = common::app(pool);

    // 1. Create Admin and James Bond.
    let (status, seeded) = common::post(&app, "/seed/users", None, json!({})).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(seeded["users"].as_array().unwrap().len(), 2);

    // 2. Admin login returns a challenge, not a JWT.
    let (status, login) = common::post(
        &app,
        "/auth/login",
        None,
        json!({ "email": ADMIN_EMAIL, "password": ADMIN_PASSWORD }),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert!(login.get("access_token").is_none());
    let challenge = login["login_challenge_id"].as_str().unwrap();

    // 3. Read the code from the dev mailbox.  4. Verify -> Admin JWT.
    let code = common::latest_code(&app, ADMIN_EMAIL).await;
    let (status, verified) = common::verify(&app, challenge, &code).await;
    assert_eq!(status, StatusCode::OK);
    let admin = verified["access_token"].as_str().unwrap().to_string();

    // 5. Create exactly 5 tasks.
    let specs = [
        ("Infiltrate SPECTRE HQ", "high"),
        ("Recover the Lektor", "medium"),
        ("Brief M on Blofeld", "low"),
        ("Service the Aston Martin", "medium"),
        ("Collect gadgets from Q", "low"),
    ];
    let mut ids = Vec::new();
    for (title, priority) in specs {
        ids.push(common::create_task(&app, &admin, title, priority).await);
    }
    let (_, all) = common::get(&app, "/tasks", Some(&admin)).await;
    assert_eq!(all.as_array().unwrap().len(), 5);

    // 6. Assign exactly 3 to James Bond.
    let (status, assigned) = common::assign(&app, &admin, &ids[..3], JAMES_EMAIL).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(assigned["updated_count"], 3);

    // 7-8. James logs in with 2FA.
    let james = common::login(&app, JAMES_EMAIL, JAMES_PASSWORD).await;

    // 9. James cannot create a task.
    let (status, denied) = common::post(
        &app,
        "/tasks",
        Some(&james),
        json!({ "title": "License to create", "priority": "high" }),
    )
    .await;
    assert_eq!(status, StatusCode::FORBIDDEN);
    assert_eq!(denied["error"]["code"], "forbidden");

    // 10. James sees exactly his 3 tasks (first call: from DB).
    let (status, first) = common::get(&app, "/tasks/view-my-tasks", Some(&james)).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(
        first["user"],
        json!({ "email": JAMES_EMAIL, "role": "staff" })
    );
    assert_eq!(first["summary"], json!({ "total_assigned_tasks": 3 }));
    assert_eq!(first["cache"], json!({ "hit": false }));
    let tasks = first["tasks"].as_array().unwrap();
    let titles: Vec<&str> = tasks.iter().map(|t| t["title"].as_str().unwrap()).collect();
    assert_eq!(
        titles,
        [
            "Infiltrate SPECTRE HQ",
            "Recover the Lektor",
            "Brief M on Blofeld"
        ]
    );
    let priorities: Vec<&str> = tasks
        .iter()
        .map(|t| t["priority"].as_str().unwrap())
        .collect();
    assert_eq!(priorities, ["high", "medium", "low"]);
    assert!(tasks
        .iter()
        .all(|t| t["status"] == "todo" && t["assigned_to"] == JAMES_EMAIL));

    // 11. Same call again comes from cache.
    let (_, second) = common::get(&app, "/tasks/view-my-tasks", Some(&james)).await;
    assert_eq!(second["cache"], json!({ "hit": true }));
    assert_eq!(second["tasks"], first["tasks"]);
}
