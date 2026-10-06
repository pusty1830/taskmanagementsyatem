# AI Usage

## Tools used

| Tool | Used for |
|---|---|
| **Claude Code** (Anthropic, model Claude Opus 5.5) in the terminal | Writing the design docs in `docs/`, then implementing the backend, frontend, tests, Docker setup, validation scripts and README phase by phase |

## How it was used

1. **Design first.** I pasted the assignment brief and asked for architecture docs before any code. Claude wrote `docs/01`–`docs/10`: stack, data model, API spec, 2FA/security, caching, frontend, testing, plan and Docker.
2. **My review and decisions before implementation.** After reading the docs, I asked for two changes:
   - **Every API must be in Swagger.** This became a requirement and is enforced by `tests/openapi_coverage.rs`.
   - **One `docker-compose.yml` that runs everything** (Postgres, Redis, backend, frontend). This is described in `docs/10-docker.md`.

   I then approved the architecture and asked for implementation following the docs.
3. **Implementation in phases** (`docs/09-implementation-plan.md`), committed separately. Tests were written first for each phase and watched fail before the code was written (TDD). Each phase was checked with `cargo test`, `cargo clippy -D warnings`, `npm test` and `npm run build` before committing.

## Decisions made during implementation (recorded while building)

- Pinned well-known crate versions (sqlx 0.8, utoipa 5, jsonwebtoken 9, …) instead of the newest majors, to avoid recent breaking changes.
- Host ports for Postgres and Redis can be configured (`POSTGRES_HOST_PORT`, `REDIS_HOST_PORT`) because on my machine 5432 and 6379 were already used by other services. The defaults stay 5432 and 6379.
- Emails are trimmed and lowercased during deserialization so validation sees the normalized value.
- The frontend fetches `view-my-tasks` once on mount, with a ref guard. Without it, React StrictMode's double effect would make the first visible response a cache HIT.

## What I changed or verified manually

<!-- Fill in honestly before submitting: what you edited by hand, what you reviewed line by line,
     what you ran yourself. Examples: -->
- [ ] Read through the backend modules (`auth/`, `services/`, `cache/`) and can explain each one
- [ ] Ran `docker compose up --build` and the full flow in the browser myself
- [ ] Manual edits: _describe any code you changed by hand_

## Understanding

I can explain all of the submitted code, including:

- why the 2FA code is stored as an HMAC rather than with Argon2
- how single use is enforced (`SELECT … FOR UPDATE` plus `consumed_at`)
- why role checks sit in the `AdminUser` extractor
- why cache invalidation runs after the transaction commits
- why a cache outage falls back to the database instead of returning 500
