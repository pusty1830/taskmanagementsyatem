# 8. Testing Strategy

## Backend

### Unit tests (`#[cfg(test)]` in modules)

- `auth::password`: a hash verifies, and a wrong password fails.
- `auth::otp`: the code is always 6 digits; the HMAC verifies only with the same challenge ID and code.
- `auth::jwt`: encode → decode round-trip; expired and tampered tokens are rejected.
- `cache::memory`: get, set, invalidate and TTL.
- DTO validation: an empty title and an over-long title are rejected.

### Integration tests (`backend/tests/`)

- Each test uses `#[sqlx::test(migrations = "./migrations")]`, so **every test gets its own fresh Postgres database**. Tests are isolated and can run in parallel.
- The app is built with `build_app(state)` using `MemoryTaskCache`, and requests go through `tower::ServiceExt::oneshot`. No server process is needed.
- A helper `login(app, email, pw) -> token` performs the real login → `/dev/email-logs/latest` → verify flow, so every authenticated test also exercises 2FA.

| Brief's testing expectation | Test |
|---|---|
| Admin and James Bond can be created | `seed_creates_admin_and_james_idempotently` |
| Login creates a challenge and returns no JWT | `login_returns_challenge_not_token` |
| Correct code returns a JWT | `verify_with_correct_code_returns_jwt` |
| Incorrect code rejected | `verify_with_wrong_code_is_rejected` |
| Expired code rejected | `verify_with_expired_code_is_rejected` (the test sets `expires_at` to the past directly in the DB) |
| Reused code rejected | `verify_reused_code_is_rejected` |
| Attempt limit | `too_many_wrong_attempts_locks_challenge` |
| A new login supersedes the old code | `new_login_invalidates_previous_challenge` |
| Admin can create 5 tasks | `admin_creates_five_tasks` |
| Admin assigns exactly 3 | `admin_assigns_three_tasks_to_james` |
| James cannot create a task | `staff_create_task_is_forbidden` (403 with `code=forbidden`) |
| James cannot assign | `staff_assign_is_forbidden` |
| No or invalid token | `missing_or_bad_token_is_401` |
| James sees exactly 3 | `james_views_exactly_three_tasks` |
| Cache false then true | `view_my_tasks_second_call_is_cache_hit` |
| Assignment invalidates | `assign_invalidates_assignee_cache` |
| Reassignment invalidates the old assignee | `reassign_invalidates_previous_assignee_cache` |
| Update invalidates | `status_update_invalidates_cache` |
| Every API is in Swagger | `openapi_documents_every_route`: loads `/api-docs/openapi.json` and asserts that each of the 13 method+path pairs is present, and that protected routes declare `bearer_auth` |
| **Full validation flow** | `full_assignment_workflow` replays the brief's steps 1–11 in one test and asserts the final JSON shape |

An optional `redis_cache_roundtrip` test runs only when `TEST_REDIS_URL` is set, so `cargo test` still passes without Redis.

### Commands

```bash
docker compose up -d postgres            # tests need DATABASE_URL pointing at a server
cd backend && cargo test
cargo fmt --check && cargo clippy --all-targets -- -D warnings
```

## Frontend (Vitest + React Testing Library, `fetch` mocked)

| Test | Asserts |
|---|---|
| `LoginPage` | Step 1 → shows the code step; a wrong code shows the error; success stores the token and redirects |
| `MyTasksPage` | Loading → renders 3 tasks from the mocked API; the empty state; the error state; the cache badge shows MISS, then HIT after refresh |
| `TaskForm` on staff | A 403 response shows the "only admins can create tasks" message |
| `api/client` | Attaches the `Authorization` header; maps an error body to `ApiError` |

```bash
cd frontend && npm test
```

## Manual end-to-end check

The validation scripts (`scripts/validate.sh`, `scripts/validate.ps1`) run the brief's flow against the running stack and print the final `view-my-tasks` response. That output is pasted into the README. Screenshots of the UI go in `screenshots/`.
