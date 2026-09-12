# rust-web-kit — Implementation Plan

A production-ready, AI-first full-stack starter kit: Rust (Axum) API + React (Vite) frontend, shipped as a single Docker image.

This document is the source of truth for the build. Work proceeds phase by phase. Each phase must leave the repo compiling, linted, and with all tests passing before it is committed.

## Non-negotiable rules for the implementing agent

- **Do not run `git commit`, `git push`, or any history-changing git command.** The reviewer commits.
- Do not skip, stub, or `#[ignore]` a test to make a phase pass. If something cannot be done, say so explicitly in your final report.
- Every phase ends with a short written report: what was built, how it was verified (commands + result), and anything left incomplete.
- Prefer the crate/library versions that are current on crates.io / npm today (check with `cargo search` / `pnpm view`). Do not guess versions from memory.
- Rust edition 2024. Deny clippy warnings. TypeScript strict. Biome clean.
- Placeholder project name is `rwk` everywhere (crate names, package name, DB name, compose project). Never use any other project-name token, because `just init` does an exact-match replace on `rwk`.
- Keep files small and single-purpose. Agents will read these; favor clarity over cleverness.
- No secrets in the repo. `.env.example` documents variable names only.

## Final architecture (reference)

```
rust-web-kit/
├── Cargo.toml                 # workspace
├── crates/
│   ├── core/                  # rwk-core: config, db, errors, auth, users, projects, jobs, mail
│   └── api/                   # rwk-api: Axum binary, routes, OpenAPI, static file serving
├── apps/web/                  # React 19 + Vite + TanStack Router/Query + shadcn/ui
├── migrations/                # SQLx migrations (.sql)
├── docs/                      # PLAN.md, decisions/ (ADRs), slices/ (feature pattern docs)
├── .claude/                   # skills, hooks, settings
├── .mcp.json                  # MCP servers exposing OpenAPI spec + DB schema
├── CLAUDE.md, AGENTS.md
├── justfile
├── mise.toml
├── hk.pkl
├── fnox.toml
├── docker-compose.yml         # postgres + mailpit (dev)
├── docker-compose.prod.yml    # app + postgres for a single VM
├── Dockerfile                 # multi-stage: pnpm build → cargo build → distroless/debian-slim runtime
└── .github/workflows/ci.yml   # thin: runs `just ci`
```

### Stack decisions (already made, do not relitigate)

| Concern | Choice |
|---|---|
| Web framework | Axum (latest), tokio |
| DB | PostgreSQL 17+, SQLx (postgres, runtime-tokio, tls-rustls, macros, migrate, uuid, chrono, json), offline `.sqlx/` cache committed |
| OpenAPI | utoipa + utoipa-axum typed router, Scalar UI at `/docs`, spec at `/api/openapi.json` |
| Jobs | apalis + apalis-postgres, worker runs inside the api binary by default, `--worker-only` / `--api-only` flags |
| Sessions | tower-sessions with Postgres store, secure HttpOnly SameSite=Lax cookie |
| Password auth | argon2 (password-hash), email verification + password reset tokens |
| OAuth | `oauth2` crate, Google + GitHub, PKCE, state in session |
| Email | lettre (tokio1, rustls), SMTP config, Mailpit locally, templates via askama or maud (pick askama) |
| Config | figment (env + defaults) into a typed `Config` struct validated at startup |
| Errors | one `AppError` (thiserror) → RFC 9457 Problem Details JSON; validation via garde |
| Observability | tracing, tracing-subscriber (json + pretty by env), tower-http TraceLayer, request IDs, optional OTLP via feature flag `otel` |
| Security | tower-http: cors, compression, timeout, request-id, set-header (security headers); tower_governor rate limiting on auth routes; origin check on mutating requests |
| Frontend | React 19, Vite, TanStack Router (file-based) + Query, @hey-api/openapi-ts with tanstack-query plugin, Tailwind v4, shadcn/ui, lucide-react, react-hook-form + zod |
| JS tooling | pnpm, Biome, TypeScript strict, Vitest, Playwright |
| Rust tooling | cargo-nextest, sqlx-cli, cargo-watch or bacon, clippy pedantic subset |
| Toolchain / hooks / secrets | mise, hk, fnox (age provider) |
| Tasks | just |
| Tests | `#[sqlx::test]` (per-test DB), testcontainers for the Postgres instance in CI, Vitest, Playwright smoke |
| Deploy | single image; `just deploy` builds, pushes to VM over SSH, `docker compose up -d` |
| License | MIT |

### Local ports

| Service | Port |
|---|---|
| API (serves frontend in prod) | 8080 |
| Vite dev server (proxies `/api` to 8080) | 5173 |
| Postgres | 5432 (default) |
| Mailpit UI / SMTP | 8025 / 1025 |

### Example domain

- `users`: id (uuid v7), email (citext unique), password_hash (nullable for OAuth-only), email_verified_at, created_at, updated_at
- `oauth_accounts`: id, user_id, provider, provider_user_id, created_at; unique(provider, provider_user_id)
- `sessions` (tower-sessions table)
- `projects`: id, owner_id → users, name, description, created_at, updated_at
- Auth tokens: `auth_tokens` (id, user_id, kind enum [email_verify, password_reset], token_hash, expires_at, used_at)
- apalis job tables via its migrations

### API surface (all under `/api`)

```
POST   /auth/register            {email,password}          → 201 {user}
POST   /auth/login               {email,password}          → 200 {user}
POST   /auth/logout                                        → 204
GET    /auth/me                                            → 200 {user} | 401
POST   /auth/verify-email        {token}                   → 204
POST   /auth/forgot-password     {email}                   → 204 (always)
POST   /auth/reset-password      {token,password}          → 204
GET    /auth/oauth/{provider}                              → 302 to provider
GET    /auth/oauth/{provider}/callback?code&state          → 302 to /
GET    /projects?page&per_page                             → 200 {items,total,page,per_page}
POST   /projects                 {name,description?}       → 201
GET    /projects/{id}                                      → 200 | 404
PATCH  /projects/{id}            {name?,description?}      → 200
DELETE /projects/{id}                                      → 204
GET    /health                                             → 200 {status,db}
GET    /openapi.json
```

Errors: `application/problem+json`, shape `{type,title,status,detail,instance?,errors?:{field:[msg]}}`.

## Phases

### Phase 1 — Scaffold and toolchain

Deliverables:
- Cargo workspace with `crates/core` (`rwk-core`, lib) and `crates/api` (`rwk-api`, bin). Edition 2024. Workspace-level dependency table. `[workspace.lints]` with clippy `pedantic` enabled and a short documented allow-list.
- `apps/web` created with Vite React-TS template, pnpm, Biome config, strict tsconfig, Tailwind v4, shadcn/ui initialized with a couple of components (button, input, card, form pieces).
- `mise.toml` pinning: rust (stable), node (LTS), pnpm, just, hk, fnox, sqlx-cli, cargo-nextest, bacon, age.
- `justfile` with recipes (may be stubs that print "TODO" where the phase hasn't landed yet): `setup`, `dev`, `dev-api`, `dev-web`, `db-up`, `db-down`, `migrate`, `migrate-new NAME`, `sqlx-prepare`, `gen` (openapi → TS client), `check`, `test`, `test-api`, `test-web`, `e2e`, `ci`, `build`, `docker-build`, `deploy`, `init NAME`, `clean`.
- `hk.pkl`: pre-commit = fix mode (cargo fmt, biome), pre-push = check mode (`just check`) + `just test`.
- `fnox.toml` with age provider, documented; `.env.example` listing every variable.
- `docker-compose.yml`: postgres:17 on 5432 (db `rwk`, user `rwk`, password `rwk`), mailpit.
- `.gitignore`, `LICENSE` (MIT), `rust-toolchain.toml`, `.editorconfig`, `README.md` (short, will be expanded in Phase 8).
- `crates/api` main just starts Axum with `/api/health` returning `{"status":"ok"}` and tracing initialized.

Acceptance:
- `mise install` succeeds. `just db-up` starts Postgres and Mailpit. `cargo build` clean. `cargo clippy --all-targets -- -D warnings` clean. `pnpm -C apps/web typecheck && pnpm -C apps/web lint` clean. `curl localhost:8080/api/health` returns ok.

### Phase 2 — Core: config, db, errors, tracing

Deliverables in `rwk-core`:
- `config.rs`: `Config` struct (server, database, session, mail, oauth, app_url, log format) loaded via figment from env with prefix `RWK_`, validated, with a `Config::from_env()` and a `Config::for_tests()`.
- `db.rs`: `PgPool` creation with sensible pool settings, `run_migrations`.
- `error.rs`: `AppError` enum (NotFound, Unauthorized, Forbidden, Validation(garde report), Conflict, BadRequest, Internal(anyhow)), `IntoResponse` producing Problem Details, `From` impls for sqlx, anyhow, garde. Internal errors logged with request id, never leaked.
- `telemetry.rs`: init tracing (json in prod, pretty in dev), optional `otel` feature.
- `state.rs`: `AppState { config, db, mailer, jobs }` (mailer/jobs may be placeholders until later phases) as `Clone` with `Arc` internals.
- First migration: enable `citext`, create `users`, `oauth_accounts`, `auth_tokens`, `projects`.
- `.sqlx/` offline cache committed; `SQLX_OFFLINE=true` builds work.
- `just migrate`, `just migrate-new`, `just sqlx-prepare` real.
- `/api/health` now checks DB with `SELECT 1`.

Acceptance:
- Tests: config loads from env and rejects invalid values; `AppError` → correct status codes and JSON body; `#[sqlx::test]` proves migrations apply. `cargo nextest run` green. Offline build green.

### Phase 3 — Users, sessions, password auth

Deliverables:
- `users/` module: model, repo functions (`create`, `find_by_email`, `find_by_id`, `set_verified`, `update_password`), service functions.
- `auth/` module: argon2 hashing, register/login/logout, `CurrentUser` Axum extractor (from session), `RequireAuth` variant, token generation (random 32 bytes, stored hashed), email-verify and password-reset flows (email sending wired to a trait/placeholder that the mail phase fills in; for now log the link).
- tower-sessions with `tower-sessions-sqlx-store` Postgres store, cookie config from `Config`.
- Routes in `rwk-api` under `/api/auth/*` with utoipa annotations. `utoipa-axum` `OpenApiRouter`. Scalar at `/docs`.
- Rate limiting on `/api/auth/*` via tower_governor. Origin check middleware for non-GET.
- All security middleware from the stack table.

Acceptance:
- Integration tests using `axum::serve` on an ephemeral port or `tower::ServiceExt::oneshot`: register → login → me → logout → me(401); duplicate email → 409 problem; bad password → 401; validation → 422 with field errors; verify-email and reset-password happy and expired paths. Green under nextest.

### Phase 4 — Projects CRUD, OpenAPI export, TS client generation

Deliverables:
- `projects/` module with owner-scoped repo and service, pagination, garde validation.
- Routes with utoipa, all response types in the schema.
- `rwk-api` gets a `--export-openapi PATH` flag (or `cargo run --bin openapi`) that writes the spec without starting the server.
- `just gen`: exports spec to `apps/web/openapi.json` and runs `@hey-api/openapi-ts` with the `@tanstack/react-query` plugin into `apps/web/src/api/`. Generated code committed. `just check` fails if regenerating produces a diff (`git diff --exit-code`).
- Frontend: `src/api/client.ts` configured with base `/api`, credentials include, Problem Details error parsing helper.

Acceptance:
- Integration tests: full CRUD, ownership enforced (other user → 404), pagination, validation errors. `just gen` idempotent. Frontend typecheck green against the generated client.

### Phase 5 — Jobs and email

Deliverables:
- `jobs/` module: apalis Postgres storage, `Job` enum or per-job structs, worker setup, `enqueue` helper that accepts a `&mut PgConnection` / transaction so enqueueing is transactional with business writes. Jobs: `SendEmail { to, template, data }`, `CleanupExpiredTokens` (cron, hourly).
- `mail/` module: lettre SMTP transport from config, askama templates (welcome/verify, password reset), `Mailer` trait with `SmtpMailer` and `LogMailer` (used in tests).
- Auth flows now enqueue `SendEmail` inside the same transaction as the user insert/token insert.
- Binary flags: default runs API + worker; `--api-only`, `--worker-only`.
- Mailpit in compose; docs on viewing mail at :8025.

Acceptance:
- Tests: enqueue inside a rolled-back transaction leaves no job; worker processes a `SendEmail` using `LogMailer` and the assertion sees the rendered email; cron job registered. Manual: register locally and see the email in Mailpit (document the steps in the report).

### Phase 6 — Frontend

Deliverables in `apps/web`:
- TanStack Router file-based routes: `/` (landing), `/login`, `/register`, `/forgot-password`, `/reset-password`, `/verify-email`, `/app` (authed layout with nav), `/app/projects`, `/app/projects/new`, `/app/projects/$id`. Route guards via `beforeLoad` using a `me` query; redirect to `/login` with return-to.
- Auth forms with react-hook-form + zod, server field errors mapped from Problem Details onto form fields.
- Projects list with pagination, create/edit form, delete with confirm dialog, optimistic update or invalidation via generated hooks.
- OAuth buttons linking to `/api/auth/oauth/{provider}`.
- Layout, dark mode toggle, toast notifications (sonner), loading and error boundaries.
- Vite dev proxy `/api` → `localhost:8080`. Production build output at `apps/web/dist`, served by Axum with SPA fallback (any non-`/api`, non-file path → `index.html`).
- Vitest with a few component tests (form validation, error mapping).

Acceptance:
- `pnpm typecheck`, `pnpm lint`, `pnpm test` green. `pnpm build` produces `dist`. Running the api serves the built app at `localhost:8080`, and the full auth + projects flow works in a browser (verified in the next phase by Playwright).

### Phase 7 — OAuth, Playwright, `just ci`, Docker, deploy

Deliverables:
- OAuth Google + GitHub: authorize redirect with PKCE + state in session, callback exchanges code, fetches profile email, links or creates user, logs in. Providers enabled only when their client id/secret are set. Unit tests for state validation and account linking logic; live flow documented as manually verified.
- Playwright smoke: register → verify via Mailpit API → login → create project → edit → delete → logout. Runs against `just dev` stack or a test compose. `just e2e`.
- `just ci`: `just check && just test && just e2e && just docker-build`. `just check` = fmt check, clippy -D warnings, biome ci, tsc, `just gen` freshness, `cargo sqlx prepare --check`.
- `Dockerfile` multi-stage (node build → cargo chef/cargo build with cache mounts → slim runtime, non-root, healthcheck). Image runs migrations on start (flag `RWK_RUN_MIGRATIONS=true`).
- `docker-compose.prod.yml`: app + postgres with volume, env from fnox (`fnox exec -- docker compose -f docker-compose.prod.yml up -d`). Notes for exe.dev: app listens on 8080, expose per https://exe.dev/docs/proxy.md.
- `just deploy HOST`: `docker build`, `docker save | ssh HOST docker load`, sync compose file, `docker compose up -d` remotely.
- `.github/workflows/ci.yml`: single job, installs mise, runs `just ci` with a Postgres service.

Acceptance:
- `just ci` fully green locally. `docker compose -f docker-compose.prod.yml up` serves the working app on 8080.

### Phase 8 — AI-first scaffolding and docs

Deliverables:
- `CLAUDE.md` (and `AGENTS.md` as the same content): architecture map, the `just` vocabulary, conventions (vertical slices, AppError, garde, utoipa annotations, generated client, never edit `src/api/` by hand), the rule "run `just check` before declaring done, `just ci` before a PR", secret handling rules (never print fnox values), how to add a feature end-to-end in 8 steps.
- `.claude/settings.json` hooks: PostToolUse on `.rs` → `cargo fmt -- <file>` + `cargo check -p <crate>` (fast path); on `.ts/.tsx` → `biome check --write <file>`. Keep hooks under a couple of seconds.
- `.claude/skills/`: `add-endpoint`, `add-migration`, `add-page`, `add-job`, `edit-hk-config` (explains Pkl), each a SKILL.md with the exact files to touch and a checklist.
- `docs/slices/projects.md`: walkthrough of the projects slice as the pattern to copy. `docs/slices/auth.md`.
- `docs/decisions/`: one ADR per major choice (SQLx, OpenAPI-from-Rust, Postgres jobs, sessions over JWT, single image, local CI via hk, fnox, vertical modules).
- `.mcp.json`: an OpenAPI MCP server pointed at the exported spec and a Postgres MCP server pointed at the local DB (read-only), with notes on how to run them.
- `just init NAME`: script (`scripts/init.sh`) that replaces `rwk` → NAME in file contents and paths, rewrites README title, deletes `scripts/init.sh` and the `init` recipe, `rm -rf .git && git init`, runs `mise install`, `hk install`, `pnpm install`. Verify by running it on a copy in `/tmp` and building the result.
- `README.md`: what it is, quickstart (template → `just init` → `just setup` → `just dev`), stack table, commands, deploy to a VM.
- Template repo setting is done on GitHub by the owner (note in README).

Acceptance:
- `just ci` green. `just init demo` on a copy in a temp dir yields a repo where `cargo build` and `pnpm build` succeed and no `rwk` token remains (`grep -r rwk` empty).

## Review checklist applied to every phase

- Compiles with `-D warnings`; Biome and tsc clean; `just check` green.
- Tests exist for new behavior and run against a real Postgres.
- No secrets, no hard-coded URLs/ports outside `Config` and compose.
- Errors go through `AppError`; every handler annotated for OpenAPI; generated client fresh.
- Files are small, named for their feature, and documented with a short module doc comment.
- Phase report is accurate.
