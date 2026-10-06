# Task Manager: Rust API + React

A full-stack task manager:

- **Rust (Axum) API** with email-based **2FA**, **JWT** auth, **role-based access** (admin / staff), and a **per-user Redis cache**
- **React + TypeScript** frontend
- **Swagger UI** documenting every endpoint
- One **Docker Compose** file runs everything

| | |
|---|---|
| Backend | Rust 2021, Axum 0.8, Tokio, SQLx 0.8 + PostgreSQL 16, Redis 7, Argon2id, jsonwebtoken (HS256), utoipa + Swagger UI |
| Frontend | React 19, TypeScript, Vite, React Router, Vitest + Testing Library |
| Design docs | [`docs/`](docs/README.md): architecture, data model, API spec, auth/2FA, caching, frontend, testing, Docker |

```
backend/     Rust API (src/, migrations/, tests/)
frontend/    React app (src/api, src/auth, src/components, src/pages)
docs/        Architecture and design docs
scripts/     validate.sh / validate.ps1: the full validation flow via curl
screenshots/ Running UI and code
```

---

## 1. Quick start with Docker (recommended)

Requires Docker Desktop.

```bash
cp .env.example .env
docker compose up --build
```

### All URLs

| What | URL |
|---|---|
| **Frontend** (React app) | http://localhost:5173 |
| Frontend: login (email/password, then 2FA code) | http://localhost:5173/login |
| Frontend: admin screen (create + assign tasks) | http://localhost:5173/admin |
| Frontend: my tasks (assigned tasks, cache badge) | http://localhost:5173/my-tasks |
| **API** base URL | http://localhost:8080 |
| **Swagger UI** (every endpoint, try it out) | http://localhost:8080/swagger-ui |
| OpenAPI JSON spec | http://localhost:8080/api-docs/openapi.json |
| Health check | http://localhost:8080/health |
| Dev mailbox (latest 2FA code) | http://localhost:8080/dev/email-logs/latest?email=jamesbond@example.com |
| PostgreSQL | `localhost:5432` (user/password/db: `taskapp`) |
| Redis | `localhost:6379` |

The same URLs apply to the native setup (section 2): the backend serves on `127.0.0.1:8080` and Vite on `localhost:5173`.

Startup order:

1. Postgres and Redis start and become healthy.
2. The backend runs its **migrations automatically**, then reports healthy on `/health`.
3. The frontend starts.

2FA codes also appear in `docker compose logs -f backend`.

> **Port already in use?** Set `POSTGRES_HOST_PORT` / `REDIS_HOST_PORT` in `.env`, e.g. 5434 / 6380. If you also run the backend natively, update `DATABASE_URL` / `REDIS_URL` to match.

Stop with `docker compose down`, or `docker compose down -v` to also wipe the database.

## 2. Native setup (for development)

Requires Rust stable (1.80+), Node 20+, and Docker (only for Postgres and Redis).

```bash
cp .env.example .env
docker compose up -d postgres redis      # database + cache only
```

**Backend:**

```bash
cd backend
cargo run          # loads ../.env, runs migrations, serves on http://127.0.0.1:8080
```

**Frontend:**

```bash
cd frontend
cp .env.example .env        # VITE_API_BASE_URL=http://localhost:8080
npm install
npm run dev                 # http://localhost:5173
```

### Migrations

Migrations live in `backend/migrations/` and are **applied automatically on startup** (`sqlx::migrate!`). To run them manually with sqlx-cli:

```bash
cargo install sqlx-cli --no-default-features --features postgres
cd backend && sqlx migrate run        # uses DATABASE_URL from the environment / .env
```

### Environment variables

Defaults are in [`.env.example`](.env.example):

| Variable | Purpose |
|---|---|
| `APP_ENV` | `development` mounts `/seed/*` and `/dev/*`. Any other value hides them (404) |
| `DATABASE_URL`, `REDIS_URL` | Connections for native runs (Compose overrides these inside the network) |
| `CACHE_BACKEND` | `redis` (default) or `memory` (see limitation below) |
| `CACHE_TTL_SECONDS` | Safety-net TTL for cached task lists (300) |
| `JWT_SECRET`, `JWT_TTL_MINUTES` | HS256 signing key (≥ 32 chars) and token lifetime (60) |
| `OTP_SECRET`, `OTP_TTL_SECONDS`, `OTP_MAX_ATTEMPTS` | 2FA HMAC key (≥ 32 chars), code expiry (300s), wrong-code limit (5) |
| `CORS_ORIGIN` | Allowed frontend origin |

---

## 3. Validation workflow

Seeded users:

| User | Email | Password | Role |
|---|---|---|---|
| Admin | `admin@example.com` | `Admin@12345` | admin |
| James Bond | `jamesbond@example.com` | `JamesBond@007` | staff |

### Option A: one script (curl)

```bash
./scripts/validate.sh                                             # bash (needs curl + jq or node)
powershell -ExecutionPolicy Bypass -File scripts\validate.ps1     # Windows, no extra deps
```

The script follows the brief's steps, starting from a clean slate:

1. `POST /dev/reset` clears tasks, codes and the cache (users are kept).
2. `POST /seed/users`
3. Admin `POST /auth/login` returns a `login_challenge_id` and **no JWT**.
4. `GET /dev/email-logs/latest` returns the code.
5. `POST /auth/verify-2fa` returns the Admin JWT.
6. 5× `POST /tasks`, then `POST /tasks/assign` with 3 task ids for James Bond.
7. James logs in with 2FA in the same way.
8. James's `POST /tasks` returns **403**.
9. `GET /tasks/view-my-tasks` twice returns `cache.hit` **false**, then **true**.

### Option B: Swagger UI

Open http://localhost:8080/swagger-ui. Every endpoint is listed with schemas, examples and error responses.

1. **seed** → `POST /seed/users`
2. **auth** → `POST /auth/login`
3. **dev** → `GET /dev/email-logs/latest`
4. **auth** → `POST /auth/verify-2fa`. Click **Authorize** and paste the `access_token`.
5. **tasks** → `POST /tasks` five times, then `POST /tasks/assign` with three ids.
6. Log in as James the same way, re-**Authorize** with his token, then:
   - `POST /tasks` returns 403.
   - `GET /tasks/view-my-tasks` twice.

### Option C: Frontend

1. Seed the users: `curl -X POST localhost:8080/seed/users`, or use Swagger.
2. Open http://localhost:5173.
3. Log in as **Admin**. On the code step, click **Fetch code from dev mailbox**, then **Verify**.
4. On the Admin screen:
   - create 5 tasks
   - tick 3 of them
   - **Assign selected (3)** to James Bond
5. Log out. Log in as **James Bond** with 2FA.
6. **My tasks** shows exactly 3 tasks, with a **cache: MISS** badge.
7. Click **Refresh**. The badge changes to **cache: HIT**.
8. **Try creating a task** shows: *"⛔ Forbidden: Only admins can create tasks (403)"*.

### Final `GET /tasks/view-my-tasks` response

Captured from the running Docker stack with `scripts/validate.sh`:

```http
GET /tasks/view-my-tasks
Authorization: Bearer <JAMES_BOND_TOKEN>
```

First call (loaded from the database):

```json
{
  "user": {
    "email": "jamesbond@example.com",
    "role": "staff"
  },
  "tasks": [
    {
      "id": "40330432-b5e9-4547-b6d0-fda831d7266e",
      "title": "Infiltrate SPECTRE HQ",
      "description": "",
      "status": "todo",
      "priority": "high",
      "assigned_to": "jamesbond@example.com",
      "created_by": "admin@example.com",
      "created_at": "2026-10-06T09:31:42.759486Z",
      "updated_at": "2026-10-06T09:31:43.813459Z"
    },
    {
      "id": "0df52a70-301c-4343-a2b7-7a96698941cf",
      "title": "Recover the Lektor",
      "description": "",
      "status": "todo",
      "priority": "medium",
      "assigned_to": "jamesbond@example.com",
      "created_by": "admin@example.com",
      "created_at": "2026-10-06T09:31:43.043305Z",
      "updated_at": "2026-10-06T09:31:43.813459Z"
    },
    {
      "id": "9ee09c5e-af5d-4104-83f1-834cc7bf3e8e",
      "title": "Brief M on Blofeld",
      "description": "",
      "status": "todo",
      "priority": "low",
      "assigned_to": "jamesbond@example.com",
      "created_by": "admin@example.com",
      "created_at": "2026-10-06T09:31:43.249647Z",
      "updated_at": "2026-10-06T09:31:43.813459Z"
    }
  ],
  "summary": {
    "total_assigned_tasks": 3
  },
  "cache": {
    "hit": false
  }
}
```

Second identical call: same body, served from cache:

```json
"cache": {
    "hit": true
  }
```

---

## 4. API overview

| Method | Path | Auth | Purpose |
|---|---|---|---|
| GET | `/health` | — | Liveness + DB ping |
| POST | `/seed/users` | — *(dev)* | Create Admin and James Bond (idempotent) |
| POST | `/dev/reset` | — *(dev)* | Clear tasks, challenges, email logs and cache |
| GET | `/dev/email-logs/latest?email=` | — *(dev)* | Latest dev email + parsed `code` |
| POST | `/auth/login` | — | Check credentials, create a 2FA challenge, email the code (**no JWT**) |
| POST | `/auth/verify-2fa` | — | Verify the code and return the JWT |
| GET | `/auth/me` | any | Current user |
| GET | `/users?role=staff` | admin | Assignee list |
| POST | `/tasks` | **admin** | Create a task (staff get **403**) |
| GET | `/tasks` | **admin** | All tasks |
| POST | `/tasks/assign` | **admin** | Assign tasks (all-or-nothing) |
| PATCH | `/tasks/{id}` | admin / assignee (status only) | Update a task |
| GET | `/tasks/view-my-tasks` | any | Caller's tasks + `cache.hit` |

Errors always look like `{"error": {"code": "forbidden", "message": "..."}}`. Full spec: [`docs/04-api-spec.md`](docs/04-api-spec.md).

## 5. How the key requirements are met

**2FA** ([`backend/src/services/auth_service.rs`](backend/src/services/auth_service.rs), [`auth/otp.rs`](backend/src/auth/otp.rs))

- Login creates a challenge with a random 6-digit code and returns only `login_challenge_id`.
- The code is stored as **HMAC-SHA256(OTP_SECRET, challenge_id:code)**, never in plain text. A slow hash can't protect a 6-digit code from brute force, but a server-side key that isn't in the database can.
- Verification runs in one transaction with `SELECT … FOR UPDATE`. The order of checks is:
  1. already consumed → `code_already_used`
  2. expired (5 min) → `code_expired`
  3. attempt limit reached → `too_many_attempts`
  4. constant-time compare of the code
- On success the challenge is marked consumed and the JWT is issued. A new login invalidates earlier pending codes.
- In development, the `DevMailer` writes the email to `email_logs` (the "dev mailbox") and logs it to the console. A real SMTP mailer would implement the same `Mailer` trait.

**Security**

- Argon2id password hashing.
- Unknown email and wrong password return the same 401, and both take one Argon2 verification, so timing doesn't reveal which emails exist.
- HS256 JWT with `exp` and `iss` checks.
- CORS is locked to the frontend origin, with a request body size limit.

**Role-based access** ([`backend/src/auth/extractor.rs`](backend/src/auth/extractor.rs))

- `AuthUser` decodes the Bearer token. `AdminUser` returns 403 for non-admins **before the handler runs**, so "admin only" is visible in each handler's signature.
- `view-my-tasks` filters on the user id from the verified token, so a client can't ask for someone else's tasks.

**Caching** ([`backend/src/services/task_service.rs`](backend/src/services/task_service.rs), [`backend/src/cache/`](backend/src/cache/))

- Cache-aside per user, with key `tasks:my:{user_id}` and a TTL of 300s.
- The first call loads from the database (`hit:false`). The second is served from the cache (`hit:true`).
- Invalidation happens **after the database commit**:
  - **assign** clears the new assignee and every previous assignee
  - **update** clears the task's assignee
  - **dev reset** clears all entries
- If Redis is down, requests log a warning and fall back to the database. They don't fail with 500.
- **In-memory limitation:** `CACHE_BACKEND=memory` (moka) is per process. Entries aren't shared between API instances and are lost on restart. Use Redis (the default) for anything beyond local development.

## 6. Tests

```bash
# Backend: needs Postgres reachable via DATABASE_URL; each test gets its own fresh database (#[sqlx::test])
docker compose up -d postgres
cd backend
cargo test
cargo fmt --check && cargo clippy --all-targets -- -D warnings
TEST_REDIS_URL=redis://localhost:6379 cargo test --test redis_cache   # optional: real Redis

# Frontend
cd frontend
npm test && npm run lint && npm run build
```

| Suite | Covers |
|---|---|
| `tests/full_workflow.rs` | The brief's steps 1–11 end to end in one test |
| `tests/auth_flow.rs` | Login returns a challenge (no JWT); correct, wrong, expired, reused, unknown-challenge and attempt-limited codes; superseded challenges; code not stored in plain text; JWT checks |
| `tests/tasks_rbac.rs` | Admin creates 5 and assigns 3; James gets 403 on create/assign/list; all-or-nothing assignment; validation; only own tasks visible; status-only updates for the assignee |
| `tests/cache_behaviour.rs` | `hit` false then true; per-user keys; invalidation on assign, reassign (previous assignee) and update; reset; cache outage falls back to the DB |
| `tests/openapi_coverage.rs` | **Every endpoint is in Swagger**, with tags, responses and `bearer_auth` on protected routes; documented routes are mounted |
| `tests/seed.rs`, `tests/health.rs`, unit tests | Seeding (idempotent, Argon2), dev routes hidden outside development, OTP/JWT/password units, Redis cache |
| `frontend/src/**/*.test.tsx` | Login + 2FA flow, dev mailbox fill, staff 403 message, cache MISS→HIT, loading/empty/error states, admin create + assign, route guards, 401 auto-logout |

CI runs both suites (`.github/workflows/ci.yml`).

## 7. Screenshots

See [`screenshots/`](screenshots/).

## 8. Troubleshooting

**"Cannot reach the API at http://localhost:8080"** in the frontend:

1. Check the backend: `curl http://localhost:8080/health` should return `{"status":"ok","database":"ok"}`.
2. If the backend is up, the browser blocked the request with **CORS**. The page's origin must appear in the backend's `CORS_ORIGIN`, a comma-separated list. The default is `http://localhost:5173,http://127.0.0.1:5173`.
3. **Don't run `npm run dev` while the Docker frontend is up.** Both want port 5173. Vite uses `strictPort` and will refuse to start rather than move to 5174, which CORS would block. Either:
   - use the Docker frontend at http://localhost:5173, or
   - run `docker compose stop frontend` first, then `npm run dev`.

## 9. Known limitations / trade-offs

- **Dev mailbox:** `email_logs.body` holds the code because it stands in for an inbox. The verification record itself stores only an HMAC. The table and its endpoint exist only when `APP_ENV=development`.
- **Role changes and tokens:** the JWT's role claim is trusted until the token expires (60 min). There are no refresh tokens.
- **Token storage:** the frontend keeps the JWT in `sessionStorage`, which is simple but readable by JavaScript. Production would use an httpOnly SameSite cookie with CSRF protection.
- **Cache race:** a read that misses could, in a microsecond window, write a stale value just after an invalidation. The TTL caps how long that value can survive. See [`docs/06-caching.md`](docs/06-caching.md).

See [`AI_USAGE.md`](AI_USAGE.md) for how AI tools were used.
