# 9. Implementation Plan

Each phase ends in a verifiable checkpoint. Phases are committed separately so the history tells the story.

| Phase | Work | Checkpoint |
|---|---|---|
| **0. Scaffold** | `git init`, `.env.example`, `.gitignore`, Cargo project, Vite React-TS project, **one `docker-compose.yml` with all 4 services (postgres, redis, backend, frontend)**, `backend/Dockerfile`, `frontend/Dockerfile` + `nginx.conf`, `.dockerignore` files ([10](10-docker.md)) | `docker compose up --build` brings up all 4 services healthy (backend serves a placeholder `/health`, frontend serves the Vite starter) |
| **1. Backend core** | `config`, `AppState`, `AppError`, tracing, CORS, `/health`, migrations 0001–0003, **utoipa `ApiDoc` + Swagger UI mounted** | `cargo run` migrates; `/health` returns 200; `/health` is visible in `/swagger-ui` |
| **2. Users and seed** | Domain enums, `user_repo`, Argon2 password module, `POST /seed/users`, `/dev/reset` | Seeding twice yields 2 users |
| **3. Auth and 2FA** | OTP module (HMAC), `challenge_repo`, `email_log_repo`, `DevMailer`, `/auth/login`, `/auth/verify-2fa`, `/dev/email-logs/latest`, JWT, `AuthUser` and `AdminUser` extractors, `/auth/me` | Auth integration tests are green |
| **4. Tasks and RBAC** | `task_repo`, `task_service`, `POST/GET /tasks`, `/tasks/assign`, `PATCH /tasks/{id}`, `/users?role=staff` | RBAC tests are green; James gets 403 |
| **5. Caching** | `TaskCache` trait, memory and Redis impls, `view-my-tasks` cache-aside, invalidation | Cache tests are green; curl shows false then true |
| **6. Swagger completeness** | Swagger UI is mounted in Phase 1. **From then on, every phase adds `#[utoipa::path]` + `ToSchema` for each new endpoint and DTO in the same commit.** This phase adds the `bearer_auth` scheme review, examples, all error responses, tags, and the route-coverage test | All 13 endpoints appear in `/swagger-ui`; the full validation flow runs from Swagger alone; `openapi_documents_every_route` is green |
| **7. Frontend** | API layer, AuthContext, LoginPage (2 steps), AdminPage, MyTasksPage, state components | The full flow works in the browser |
| **8. Frontend tests** | Vitest and RTL tests from [08](08-testing.md) | `npm test` is green |
| **9. Deliverables** | `README.md` (setup, migrate, run, seed, validate, test, final response), validation scripts, `AI_USAGE.md`, screenshots, optional CI workflow | A fresh clone runs end to end with `cp .env.example .env && docker compose up --build`, following only the README |

## Submission checklist (from the brief)

- [ ] GitHub repo with `backend/` and `frontend/`
- [ ] README: frontend setup, backend setup, migration, run, seed, validation, tests
- [ ] Final `GET /tasks/view-my-tasks` response pasted into the README
- [ ] `AI_USAGE.md`: tools used and what was manually changed
- [ ] **Every API documented in Swagger UI (`/swagger-ui`) with schemas, examples, error responses and Bearer auth**
- [ ] `.env.example`
- [ ] **One `docker-compose.yml`: `docker compose up --build` starts Postgres, Redis, backend and frontend**
- [ ] Migrations and tests
- [ ] 2–3 screenshots (running UI and relevant code)

## Open decisions

These are my recommended defaults. Approve them as-is or tell me which to change.

| # | Decision | Recommendation | Alternative |
|---|---|---|---|
| 1 | Backend framework | **Axum** | Actix Web |
| 2 | Database | **PostgreSQL in Docker** | SQLite file (no Docker needed, but the brief prefers Postgres) |
| 3 | Cache | **Redis in Docker**, plus an in-memory impl for tests and fallback | In-memory only |
| 4 | Frontend | **React + Vite + TypeScript**, no UI library | Next.js; JavaScript instead of TypeScript |
| 5 | Dev mailbox | **`email_logs` table** (body contains the code; dev-only), plus a console log. The verification record stores only an HMAC | Keep the dev mailbox in memory only, so the code never touches the DB at all |
| 6 | Seed credentials | `admin@example.com` / `Admin@12345`, `jamesbond@example.com` / `JamesBond@007` | Your choice |
| 7 | Extra endpoints beyond the brief | `GET /tasks`, `GET /users?role=staff`, `PATCH /tasks/{id}`, `/dev/reset`, `/auth/me` (needed for the admin UI, the "update invalidates cache" requirement, and repeatable validation) | Drop the ones you don't want |
| 8 | Git | `git init` locally, one commit per phase; **you** create and push the GitHub repo | I create the repo with `gh` |
| 9 | CI | Add a GitHub Actions workflow (fmt, clippy, tests, frontend build) | Skip |
