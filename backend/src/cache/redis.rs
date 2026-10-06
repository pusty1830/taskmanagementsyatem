use async_trait::async_trait;
use redis::{aio::ConnectionManager, AsyncCommands};
use uuid::Uuid;

use super::{my_tasks_key, TaskCache, KEY_PREFIX};
use crate::dto::task::MyTasksPayload;

/// Redis-backed cache shared by every API instance. Values are JSON with `EX` = TTL.
#[derive(Clone)]
pub struct RedisTaskCache {
    conn: ConnectionManager,
    ttl_seconds: u64,
}

impl RedisTaskCache {
    pub async fn connect(url: &str, ttl_seconds: u64) -> anyhow::Result<Self> {
        let client = redis::Client::open(url)?;
        // ConnectionManager reconnects automatically if Redis restarts.
        let conn = ConnectionManager::new(client).await?;
        Ok(Self { conn, ttl_seconds })
    }
}

#[async_trait]
impl TaskCache for RedisTaskCache {
    async fn get_my_tasks(&self, user_id: Uuid) -> anyhow::Result<Option<MyTasksPayload>> {
        let mut conn = self.conn.clone();
        let raw: Option<String> = conn.get(my_tasks_key(user_id)).await?;
        Ok(raw.map(|s| serde_json::from_str(&s)).transpose()?)
    }

    async fn set_my_tasks(&self, user_id: Uuid, payload: &MyTasksPayload) -> anyhow::Result<()> {
        let mut conn = self.conn.clone();
        let json = serde_json::to_string(payload)?;
        let _: () = conn
            .set_ex(my_tasks_key(user_id), json, self.ttl_seconds)
            .await?;
        Ok(())
    }

    async fn invalidate(&self, user_ids: &[Uuid]) -> anyhow::Result<()> {
        if user_ids.is_empty() {
            return Ok(());
        }
        let keys: Vec<String> = user_ids.iter().copied().map(my_tasks_key).collect();
        let mut conn = self.conn.clone();
        let _: () = conn.del(keys).await?;
        Ok(())
    }

    async fn clear(&self) -> anyhow::Result<()> {
        // SCAN + DEL on our prefix only; never FLUSHALL a shared Redis.
        let mut conn = self.conn.clone();
        let keys: Vec<String> = {
            let mut scan_conn = self.conn.clone();
            let mut iter = scan_conn
                .scan_match::<_, String>(format!("{KEY_PREFIX}*"))
                .await?;
            let mut keys = Vec::new();
            while let Some(key) = iter.next_item().await {
                keys.push(key);
            }
            keys
        };
        if !keys.is_empty() {
            let _: () = conn.del(keys).await?;
        }
        Ok(())
    }
}
