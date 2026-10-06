# 2. System Architecture

## Components

```mermaid
flowchart LR
    subgraph Browser
        UI[React SPA<br/>Vite :5173]
    end
    subgraph Backend["Rust API — Axum :8080"]
        R[Routes / Handlers]
        MW[Auth extractors<br/>AuthUser / AdminUser]
        S[Services<br/>auth · tasks · email]
        RP[Repositories<br/>SQLx]
        CA[TaskCache trait]
    end
    PG[(PostgreSQL)]
    RD[(Redis)]
    LOG[[Console log]]

    UI -- "JSON over HTTP<br/>Authorization: Bearer JWT" --> R
    R --> MW
    R --> S
    S --> RP --> PG
    S --> CA --> RD
    S -- "dev mailer" --> LOG
    S -- "email_logs row" --> RP
```

## Backend layering

Each layer depends only on the layer beneath it:

| Layer | Responsibility | Never does |
|---|---|---|
| **routes/** (handlers) | Parse and validate the request DTO, call a service, shape the response DTO | Run SQL or business rules |
| **auth/** (extractors) | Decode the JWT into `AuthUser`. `AdminUser` rejects non-admins with 403 | Touch tasks |
| **services/** | Business rules: 2FA lifecycle, role rules, assignment, cache-aside, invalidation | Know about HTTP types |
| **repositories/** | SQL only, typed rows (`FromRow`) | Make policy decisions |
| **cache/** | `TaskCache` trait with `RedisTaskCache` and `MemoryTaskCache` impls | Know about the DB |
| **email/** | `Mailer` trait. `DevMailer` writes an `email_logs` row and a `tracing::info!` line | — |

Shared state is passed to handlers through `State<AppState>`:

```rust
#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
    pub cache: Arc<dyn TaskCache>,
    pub mailer: Arc<dyn Mailer>,
    pub config: Arc<Config>,   // jwt secret, ttl values, app_env
}
```

`lib.rs` exposes `build_app(state) -> Router`, so integration tests create the real router against a test database and send requests with `tower::ServiceExt::oneshot`. No network port is needed.

## Request lifecycle: `GET /tasks/view-my-tasks`

```mermaid
sequenceDiagram
    participant H as Handler
    participant X as AuthUser extractor
    participant S as TaskService
    participant C as TaskCache
    participant R as TaskRepo
    H->>X: Authorization header
    X-->>H: AuthUser{id,email,role} (or 401)
    H->>S: my_tasks(user)
    S->>C: get(user.id)
    alt hit
        C-->>S: Some(payload)
        S-->>H: (payload, hit=true)
    else miss / cache error
        S->>R: list_assigned_to(user.id)
        R-->>S: Vec<TaskRow>
        S->>C: set(user.id, payload, ttl)
        S-->>H: (payload, hit=false)
    end
    H-->>H: build MyTasksResponse{user,tasks,summary,cache}
```

## Repository layout

```
task/
├── backend/
│   ├── Cargo.toml
│   ├── Dockerfile             # multi-stage: cargo-chef → debian-slim runtime
│   ├── migrations/
│   │   ├── 0001_create_enums_and_users.sql
│   │   ├── 0002_create_tasks.sql
│   │   └── 0003_create_login_challenges_and_email_logs.sql
│   ├── src/
│   │   ├── main.rs            # load config, tracing, pool, migrate, serve
│   │   ├── lib.rs             # build_app(), module wiring
│   │   ├── config.rs          # env → Config
│   │   ├── state.rs           # AppState
│   │   ├── error.rs           # AppError + IntoResponse
│   │   ├── openapi.rs         # utoipa doc
│   │   ├── domain/            # Role, TaskStatus, TaskPriority enums, User, Task
│   │   ├── dto/               # request/response structs (serde + validator + utoipa)
│   │   ├── auth/              # jwt.rs, password.rs, otp.rs, extractor.rs
│   │   ├── repositories/      # user_repo, task_repo, challenge_repo, email_log_repo
│   │   ├── services/          # auth_service, task_service, seed_service
│   │   ├── cache/             # mod.rs (trait), redis.rs, memory.rs
│   │   ├── email/             # mod.rs (Mailer trait), dev_mailer.rs
│   │   └── routes/            # auth.rs, tasks.rs, users.rs, seed.rs, dev.rs, health.rs
│   └── tests/
│       ├── common/mod.rs      # test app builder, login helper
│       ├── auth_flow.rs
│       ├── tasks_rbac.rs
│       └── cache_behaviour.rs
├── frontend/
│   ├── package.json, vite.config.ts, tsconfig.json
│   ├── Dockerfile, nginx.conf # node build → nginx serves dist/
│   └── src/
│       ├── api/               # client.ts, auth.ts, tasks.ts, users.ts, dev.ts, types.ts
│       ├── auth/              # AuthContext.tsx, ProtectedRoute.tsx
│       ├── hooks/             # useAsync.ts
│       ├── components/        # TaskList, TaskForm, AssignPanel, StatusMessage, CacheBadge, NavBar
│       ├── pages/             # LoginPage, AdminPage, MyTasksPage
│       └── main.tsx, App.tsx
├── docs/                      # (this folder)
├── screenshots/
├── docker-compose.yml         # postgres + redis + backend + frontend (see 10-docker.md)
├── .env.example
├── README.md
└── AI_USAGE.md
```

## Configuration (`.env.example`)

```dotenv
# backend
APP_ENV=development              # enables /seed and /dev routes only when "development"
BIND_ADDR=127.0.0.1:8080
DATABASE_URL=postgres://taskapp:taskapp@localhost:5432/taskapp
REDIS_URL=redis://localhost:6379
CACHE_BACKEND=redis              # redis | memory
CACHE_TTL_SECONDS=300
JWT_SECRET=change-me-to-a-long-random-string
JWT_TTL_MINUTES=60
OTP_SECRET=change-me-another-long-random-string
OTP_TTL_SECONDS=300
OTP_MAX_ATTEMPTS=5
CORS_ORIGIN=http://localhost:5173
RUST_LOG=info,task_api=debug,tower_http=info

# frontend (frontend/.env)
VITE_API_BASE_URL=http://localhost:8080
```

## Error handling and graceful degradation

- Every failure becomes an `AppError` variant, and each variant maps to exactly one HTTP status. Internal errors are logged with their details, and the client receives only a generic message.
- **Cache failures do not fail requests.** If Redis is down, `view-my-tasks` logs a warning, reads from the DB, and returns `cache.hit=false`.
