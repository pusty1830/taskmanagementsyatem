# 6. Caching

## What is cached

Only `GET /tasks/view-my-tasks`, **per user**.

| Item | Value |
|---|---|
| Key | `tasks:my:{user_id}` |
| Value | JSON of `{ tasks: [TaskDto], summary: { total_assigned_tasks } }`. The cache metadata is **not** stored; it is added per response |
| TTL | `CACHE_TTL_SECONDS` (default 300s), a safety net in case an invalidation is ever missed |
| Pattern | Cache-aside (read-through in the service) |

The `user` block of the response comes from the verified JWT, so it isn't cached either.

## Abstraction

```rust
#[async_trait]
pub trait TaskCache: Send + Sync {
    async fn get_my_tasks(&self, user_id: Uuid) -> Result<Option<MyTasksPayload>, CacheError>;
    async fn set_my_tasks(&self, user_id: Uuid, payload: &MyTasksPayload) -> Result<(), CacheError>;
    async fn invalidate(&self, user_ids: &[Uuid]) -> Result<(), CacheError>;
    async fn clear(&self) -> Result<(), CacheError>;   // used by /dev/reset
}
```

| Impl | Backing | Used when |
|---|---|---|
| `RedisTaskCache` | `redis` crate, `ConnectionManager` (auto-reconnect), `SET key val EX ttl`, `GET`, `DEL k1 k2…` | `CACHE_BACKEND=redis` (default with Docker Compose) |
| `MemoryTaskCache` | `moka::future::Cache<Uuid, MyTasksPayload>` with `time_to_live` | `CACHE_BACKEND=memory`, and in integration tests |

**Documented limitation of the in-memory cache:** it is per process. Running several API instances would give each its own cache, and an invalidation on one instance wouldn't reach the others. It is also lost on restart. That's fine for local development and tests, but production would use Redis.

## Read path

```
get_my_tasks(user):
    match cache.get(user.id):
        Ok(Some(p))  -> return (p, hit = true)
        Ok(None)     -> {}
        Err(e)       -> warn!("cache read failed: {e}")   // degrade, don't fail
    p = repo.list_assigned_to(user.id)  // DB is the source of truth
    if let Err(e) = cache.set(user.id, &p): warn!(...)
    return (p, hit = false)
```

## Invalidation matrix

Invalidation runs **after the DB transaction commits**, so a concurrent read can't repopulate the cache with stale pre-commit data from the same flow.

| Mutation | Keys deleted |
|---|---|
| `POST /tasks/assign` | New assignee, plus every **previous** assignee of the reassigned tasks (collected inside the transaction with `SELECT assigned_to_id … FOR UPDATE`) |
| `PATCH /tasks/{id}` | The task's current assignee (and the previous one, if the assignee changed) |
| `POST /tasks` | None. New tasks are unassigned, so no one's list changes |
| `POST /dev/reset` | `clear()`. With Redis this uses `SCAN MATCH tasks:my:*` + `DEL`, never `FLUSHALL` |

### Remaining race (acknowledged)

Suppose a read misses, loads from the DB, and then a write commits and invalidates *before* the read writes its now-stale value to the cache. The stale value then survives until the TTL expires. The window is microseconds, the TTL caps the damage, and the fix (versioned keys) is more than this scope needs. This is noted in the README.

## Tests that prove it

1. View twice gives `hit=false` then `hit=true`.
2. View (cached), then assign another task, then view gives `hit=false` and the new count.
3. View (cached), then PATCH status, then view gives `hit=false` and the new status.
4. Reassigning a task from user A to user B invalidates **A's** cache too.
5. User A's cache entry never appears in user B's response (key isolation).
