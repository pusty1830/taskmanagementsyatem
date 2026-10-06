//! Runs against a real Redis only when TEST_REDIS_URL is set, e.g.
//! `TEST_REDIS_URL=redis://localhost:6379 cargo test --test redis_cache`.

use task_api::{
    cache::{RedisTaskCache, TaskCache},
    dto::task::{MyTasksPayload, TaskSummary},
};
use uuid::Uuid;

fn payload(n: usize) -> MyTasksPayload {
    MyTasksPayload {
        tasks: vec![],
        summary: TaskSummary {
            total_assigned_tasks: n,
        },
    }
}

async fn redis_cache() -> Option<RedisTaskCache> {
    let url = std::env::var("TEST_REDIS_URL").ok()?;
    Some(
        RedisTaskCache::connect(&url, 60)
            .await
            .expect("connect to TEST_REDIS_URL"),
    )
}

#[tokio::test]
async fn redis_cache_round_trip_and_invalidate() {
    let Some(cache) = redis_cache().await else {
        eprintln!("skipped: TEST_REDIS_URL not set");
        return;
    };
    let (a, b) = (Uuid::new_v4(), Uuid::new_v4());

    assert!(cache.get_my_tasks(a).await.unwrap().is_none());
    cache.set_my_tasks(a, &payload(3)).await.unwrap();
    cache.set_my_tasks(b, &payload(1)).await.unwrap();
    let got = cache.get_my_tasks(a).await.unwrap().unwrap();
    assert_eq!(got.summary.total_assigned_tasks, 3);

    cache.invalidate(&[a]).await.unwrap();
    assert!(cache.get_my_tasks(a).await.unwrap().is_none());
    assert!(cache.get_my_tasks(b).await.unwrap().is_some());

    cache.clear().await.unwrap();
    assert!(cache.get_my_tasks(b).await.unwrap().is_none());
}
