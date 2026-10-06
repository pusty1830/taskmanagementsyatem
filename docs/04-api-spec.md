# 4. API Specification

Base URL: `http://localhost:8080`. All bodies are JSON.

## Swagger / OpenAPI (mandatory: every endpoint)

**Every endpoint in this document appears in Swagger.** That includes health, seed, dev, auth, users and tasks. The whole validation flow can be run from the browser without curl or the frontend.

| Item | Detail |
|---|---|
| Swagger UI | `http://localhost:8080/swagger-ui` |
| Raw spec | `http://localhost:8080/api-docs/openapi.json` (OpenAPI 3.1) |
| Generator | `utoipa`: each handler has `#[utoipa::path(...)]`, and every DTO and enum derives `ToSchema` |
| Registration | One `ApiDoc` struct in `src/openapi.rs` lists all paths and schemas. Adding a route without registering it fails the coverage test (see [08](08-testing.md)) |
| Auth | `bearer_auth` security scheme (HTTP Bearer, JWT). Protected routes declare `security(("bearer_auth" = []))`, so the **Authorize** button sends the token |
| Tags | `health`, `seed`, `dev`, `auth`, `users`, `tasks`, in the order the validation flow uses them |
| Per-endpoint docs | Summary, request body schema with an example, **every** response status (200/201/400/401/403/404/429), and the `ErrorResponse` schema |
| Examples | Request examples are pre-filled (seed credentials, sample tasks), so "Try it out" works immediately |
| Dev routes | Shown under the `dev` and `seed` tags, with descriptions noting they only work when `APP_ENV=development` |

**Validation flow in Swagger:**

1. `seed` → `POST /seed/users`
2. `auth` → `POST /auth/login`
3. `dev` → `GET /dev/email-logs/latest`
4. `auth` → `POST /auth/verify-2fa`, then copy the token into **Authorize**
5. `tasks` → create ×5, then assign 3
6. Re-authorize with James Bond's token, then `POST /tasks` (403), then `GET /tasks/view-my-tasks` ×2

## Endpoint summary

| Method | Path | Auth | Purpose |
|---|---|---|---|
| GET | `/health` | — | Liveness check (DB ping) |
| POST | `/seed/users` | — *(dev only)* | Create Admin and James Bond. Idempotent |
| POST | `/dev/reset` | — *(dev only)* | Truncate tasks, challenges and email logs, and flush the cache, so the "exactly 5 / exactly 3" flow can be repeated |
| GET | `/dev/email-logs/latest?email=` | — *(dev only)* | Latest dev email, optionally filtered by recipient |
| POST | `/auth/login` | — | Check credentials, create a 2FA challenge, send the code. **No JWT** |
| POST | `/auth/verify-2fa` | — | Verify the code and return the JWT |
| GET | `/auth/me` | Any | Current user from the token |
| GET | `/users?role=staff` | Admin | List assignable users (feeds the frontend dropdown) |
| POST | `/tasks` | **Admin** | Create a task |
| GET | `/tasks` | **Admin** | List all tasks (admin screen: pick tasks to assign) |
| POST | `/tasks/assign` | **Admin** | Assign selected tasks to a user |
| PATCH | `/tasks/{id}` | Admin, or the assignee for `status` only | Update a task, which invalidates the affected caches |
| GET | `/tasks/view-my-tasks` | Any | Tasks assigned to the caller, with cache metadata |

"Dev only" routes are mounted only when `APP_ENV=development`. In any other environment they return 404.

## Error format

```json
{ "error": { "code": "forbidden", "message": "Only admins can create tasks" } }
```

| Status | `code` | When |
|---|---|---|
| 400 | `validation_error` | Bad body (also includes `details: { field: [msgs] }`) |
| 401 | `unauthorized` | Missing, invalid or expired JWT, or bad credentials |
| 401 | `invalid_code` | Wrong 2FA code |
| 401 | `code_expired` | 2FA challenge expired |
| 401 | `code_already_used` | 2FA challenge already consumed |
| 429 | `too_many_attempts` | 2FA attempt limit reached for this challenge |
| 403 | `forbidden` | Authenticated but the role is not allowed |
| 404 | `not_found` | Resource or task IDs not found |
| 409 | `conflict` | Unique violations |
| 500 | `internal` | Anything else (details only in the logs) |

---

## Seed and dev

### `POST /seed/users` → 200

Creates the users if they are missing and leaves existing ones unchanged.

```json
{
  "users": [
    { "id": "…", "full_name": "Admin",      "email": "admin@example.com",     "role": "admin" },
    { "id": "…", "full_name": "James Bond", "email": "jamesbond@example.com", "role": "staff" }
  ],
  "credentials_hint": {
    "admin@example.com": "Admin@12345",
    "jamesbond@example.com": "JamesBond@007"
  }
}
```

### `GET /dev/email-logs/latest?email=jamesbond@example.com` → 200

```json
{
  "id": "…", "to_email": "jamesbond@example.com",
  "subject": "Your login verification code",
  "body": "Your verification code is 482913. It expires in 5 minutes.",
  "code": "482913",
  "challenge_id": "…", "created_at": "2026-10-06T10:00:00Z"
}
```

`code` is parsed out of the body for convenience. It returns 404 if no email exists yet.

---

## Auth

### `POST /auth/login`

```json
{ "email": "admin@example.com", "password": "Admin@12345" }
```

→ **200**

```json
{
  "login_challenge_id": "9d1c…",
  "expires_in_seconds": 300,
  "message": "Verification code sent to a***n@example.com"
}
```

An unknown email and a wrong password both return the same `401 unauthorized` ("Invalid email or password").

### `POST /auth/verify-2fa`

```json
{ "login_challenge_id": "9d1c…", "code": "482913" }
```

→ **200**

```json
{
  "access_token": "eyJhbGciOi…",
  "token_type": "Bearer",
  "expires_in_seconds": 3600,
  "user": { "id": "…", "full_name": "Admin", "email": "admin@example.com", "role": "admin" }
}
```

Failures return `invalid_code`, `code_expired`, `code_already_used` or `too_many_attempts`.

---

## Tasks

### `POST /tasks` (admin) → 201

```json
{ "title": "Infiltrate SPECTRE HQ", "description": "Recon only", "priority": "high" }
```

`status` defaults to `todo`, and `priority` defaults to `medium`. Validation: `title` must be 1–200 characters after trimming, and `description` at most 2000.

→ returns a `TaskDto`:

```json
{ "id": "…", "title": "Infiltrate SPECTRE HQ", "description": "Recon only",
  "status": "todo", "priority": "high", "assigned_to": null,
  "created_by": "admin@example.com", "created_at": "…", "updated_at": "…" }
```

When **James Bond** calls it, the response is **403**:

```json
{ "error": { "code": "forbidden", "message": "Only admins can create tasks" } }
```

### `POST /tasks/assign` (admin) → 200

```json
{ "task_ids": ["…", "…", "…"], "assignee_email": "jamesbond@example.com" }
```

Rules:

- `task_ids` must be non-empty, contain no duplicates, and every ID must exist. Otherwise the whole request fails with 404 and nothing is partially applied.
- The assignee must exist.
- The update runs in one transaction.
- After commit, the service invalidates the cache for the new assignee **and** for any previous assignees of those tasks.

```json
{ "assigned_to": "jamesbond@example.com", "task_ids": ["…","…","…"], "updated_count": 3 }
```

### `PATCH /tasks/{id}`

```json
{ "status": "in_progress" }
```

Admins may change `title`, `description`, `status` and `priority`. The assignee may change `status` only, and any other caller gets 403. The service invalidates the cache of the task's assignee. Returns the `TaskDto`.

### `GET /tasks/view-my-tasks` → 200 *(the validation endpoint)*

```json
{
  "user": { "email": "jamesbond@example.com", "role": "staff" },
  "tasks": [
    { "id": "…", "title": "…", "status": "todo", "priority": "high",   "assigned_to": "jamesbond@example.com" },
    { "id": "…", "title": "…", "status": "todo", "priority": "medium", "assigned_to": "jamesbond@example.com" },
    { "id": "…", "title": "…", "status": "todo", "priority": "low",    "assigned_to": "jamesbond@example.com" }
  ],
  "summary": { "total_assigned_tasks": 3 },
  "cache": { "hit": false }
}
```

The second identical call returns `"cache": { "hit": true }`. Each item in `tasks` also carries `description`, `created_at` and `updated_at`. These are extra fields that don't break the expected shape.

## curl validation script (summary)

The full script will ship in the README as `scripts/validate.sh`, with a PowerShell twin:

```bash
curl -X POST :8080/dev/reset && curl -X POST :8080/seed/users
CH=$(curl -s -X POST :8080/auth/login -d '{"email":"admin@example.com","password":"Admin@12345"}' | jq -r .login_challenge_id)
CODE=$(curl -s ":8080/dev/email-logs/latest?email=admin@example.com" | jq -r .code)
ADMIN=$(curl -s -X POST :8080/auth/verify-2fa -d "{\"login_challenge_id\":\"$CH\",\"code\":\"$CODE\"}" | jq -r .access_token)
# 5× POST /tasks, then POST /tasks/assign with 3 ids
# James: login → code → verify → POST /tasks (expect 403) → view-my-tasks ×2
```
