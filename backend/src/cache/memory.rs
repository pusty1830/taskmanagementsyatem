use std::time::Duration;

use async_trait::async_trait;
use moka::future::Cache;
use uuid::Uuid;

use super::TaskCache;
use crate::dto::task::MyTasksPayload;

/// In-process cache. Limitation: not shared between API instances and lost on restart,
/// so it suits local development and tests; use Redis when running more than one instance.
pub struct MemoryTaskCache {
    inner: Cache<Uuid, MyTasksPayload>,
}

impl MemoryTaskCache {
    pub fn new(ttl_seconds: u64) -> Self {
        Self {
            inner: Cache::builder()
                .max_capacity(10_000)
                .time_to_live(Duration::from_secs(ttl_seconds))
                .build(),
        }
    }
}

#[async_trait]
impl TaskCache for MemoryTaskCache {
    async fn get_my_tasks(&self, user_id: Uuid) -> anyhow::Result<Option<MyTasksPayload>> {
        Ok(self.inner.get(&user_id).await)
    }

    async fn set_my_tasks(&self, user_id: Uuid, payload: &MyTasksPayload) -> anyhow::Result<()> {
        self.inner.insert(user_id, payload.clone()).await;
        Ok(())
    }

    async fn invalidate(&self, user_ids: &[Uuid]) -> anyhow::Result<()> {
        for id in user_ids {
            self.inner.invalidate(id).await;
        }
        Ok(())
    }

    async fn clear(&self) -> anyhow::Result<()> {
        self.inner.invalidate_all();
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dto::task::TaskSummary;

    fn payload(n: usize) -> MyTasksPayload {
        MyTasksPayload {
            tasks: vec![],
            summary: TaskSummary {
                total_assigned_tasks: n,
            },
        }
    }

    #[tokio::test]
    async fn set_get_invalidate() {
        let cache = MemoryTaskCache::new(60);
        let (a, b) = (Uuid::new_v4(), Uuid::new_v4());
        cache.set_my_tasks(a, &payload(1)).await.unwrap();
        cache.set_my_tasks(b, &payload(2)).await.unwrap();

        cache.invalidate(&[a]).await.unwrap();

        assert!(cache.get_my_tasks(a).await.unwrap().is_none());
        let b_payload = cache.get_my_tasks(b).await.unwrap().unwrap();
        assert_eq!(b_payload.summary.total_assigned_tasks, 2);
    }

    #[tokio::test]
    async fn entries_expire_after_ttl() {
        let cache = MemoryTaskCache::new(1);
        let id = Uuid::new_v4();
        cache.set_my_tasks(id, &payload(1)).await.unwrap();

        tokio::time::sleep(Duration::from_millis(1200)).await;

        assert!(cache.get_my_tasks(id).await.unwrap().is_none());
    }
}
