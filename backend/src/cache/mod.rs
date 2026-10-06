//! Per-user cache for `GET /tasks/view-my-tasks` (cache-aside; see docs/06-caching.md).

mod memory;
mod redis;

use async_trait::async_trait;
use uuid::Uuid;

pub use self::{memory::MemoryTaskCache, redis::RedisTaskCache};
use crate::dto::task::MyTasksPayload;

pub const KEY_PREFIX: &str = "tasks:my:";

pub fn my_tasks_key(user_id: Uuid) -> String {
    format!("{KEY_PREFIX}{user_id}")
}

/// Cache errors are reported but never fail a request: callers fall back to the database.
#[async_trait]
pub trait TaskCache: Send + Sync {
    async fn get_my_tasks(&self, user_id: Uuid) -> anyhow::Result<Option<MyTasksPayload>>;
    async fn set_my_tasks(&self, user_id: Uuid, payload: &MyTasksPayload) -> anyhow::Result<()>;
    async fn invalidate(&self, user_ids: &[Uuid]) -> anyhow::Result<()>;
    /// Drops every `tasks:my:*` entry (used by `/dev/reset`).
    async fn clear(&self) -> anyhow::Result<()>;
}
