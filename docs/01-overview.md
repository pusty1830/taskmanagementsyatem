# 1. Overview & Tech Stack

## Goal

A small full-stack task manager that runs the assignment's validation flow from start to finish:

1. Seed **Admin** and **James Bond**.
2. Log in with email and password. The API returns a `login_challenge_id` and sends a 6-digit code by email (development mailbox). It does **not** return a JWT at this step.
3. Verify the code. The API returns a JWT.
4. Admin creates 5 tasks and assigns 3 of them to James Bond.
5. James Bond logs in with 2FA. His attempt to create a task returns **403**.
6. James Bond's task view shows exactly **3 tasks**. The first call returns `cache.hit = false` and the second returns `cache.hit = true`.

## Out of scope (deliberately)

- Production deployment, HTTPS termination, and real SMTP delivery (an SMTP adapter can be added later behind the same trait).
- Refresh tokens and logout-everywhere. The app uses short-lived access tokens only.
- Visual polish. The UI is plain, responsive and functional.

## Tech stack

### Backend (Rust, edition 2021)

| Concern | Choice | Why |
|---|---|---|
| Web framework | **Axum 0.8** | Tower-based, built on Tokio, typed extractors that fit auth and role checks well |
| Async runtime | **Tokio** | Required by the brief; Axum runs on it |
| Database | **PostgreSQL 16** (Docker) | Preferred by the brief; native enums and UUIDs |
| DB access | **SQLx** (runtime-checked `query_as` + `FromRow`) | Async, no ORM magic. Runtime queries let reviewers build without a live DB or a `sqlx prepare` step |
| Migrations | **sqlx migrations** (`backend/migrations`) | Run automatically on startup (`sqlx::migrate!`) and also with `sqlx-cli` |
| Cache | **Redis 7** (Docker) with an **in-memory fallback** (`moka`) | Redis is preferred. In-memory is used in tests and when no Redis is available |
| Password hashing | **Argon2id** (`argon2` crate) | Current best-practice KDF |
| 2FA code hashing | **HMAC-SHA256** with a server secret | See [05](05-auth-and-2fa.md#why-hmac-and-not-argon2-for-the-code) |
| JWT | **jsonwebtoken** (HS256) | Simple and well maintained |
| Serialization | **Serde** | Required |
| Validation | **validator** crate + domain checks | Declarative field rules on DTOs |
| Errors | **thiserror** → one `AppError` that implements `IntoResponse` | Consistent JSON error body |
| Observability | **tracing** + `tower-http::TraceLayer` | Structured request logs; 2FA emails also print to the console |
| API docs | **utoipa** + Swagger UI at `/swagger-ui`. **Required: every endpoint is documented** | Lets reviewers run the whole flow without the frontend. A test enforces that no route is missing |
| CORS | `tower-http::CorsLayer` | Allows the frontend dev origin |

### Frontend

| Concern | Choice | Why |
|---|---|---|
| Framework | **React 18 + TypeScript**, built with **Vite** | Fast dev server; types mirror the API DTOs |
| Routing | **React Router** | Login, admin and staff screens |
| HTTP | Thin `fetch` wrapper (`src/api/client.ts`) | Attaches the JWT and maps errors to a typed `ApiError`. No extra dependency |
| State | React Context for auth plus a small `useAsync` hook | Easy to explain. Server data is fetched on demand, so client-side caching never masks `cache.hit` |
| Tests | **Vitest + React Testing Library** | Component state tests (loading, error, empty, success) |

### Tooling

- **One `docker-compose.yml` runs the whole stack** (Postgres, Redis, the Rust backend and the React frontend) with `docker compose up --build`. Both apps get multi-stage Dockerfiles. Running natively (`cargo run` / `npm run dev` against the Compose DB and cache) is also supported for development. See [10](10-docker.md).
- `rustfmt`, `clippy -D warnings`, `eslint`.
- Optional GitHub Actions workflow: fmt, clippy, tests against service containers, and the frontend build and test.
