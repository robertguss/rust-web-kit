# Phase 2 report — Core: config, db, errors, tracing

Status: complete. All Phase 2 acceptance items were run on this machine. Nothing was committed.

## What was built

In `rwk-core`:

- `config.rs`: `Config` (env, server, database, session, mail, oauth, `app_url`, `log_format`, `run_migrations`) loaded with figment from `RWK_` env (`__` nested keys) over defaults, then garde-validated. `env` is `Environment` (`development` / `test` / `production`, default `development`, `RWK_ENV`). Production rejects the default session secret. `Config::from_env()`, `Config::from_figment()`, `Config::for_tests()`, `Config::listen_addr()`.
- `db.rs`: `PgPool` with max 10 / min 1 / 5s acquire / 10m idle, `MIGRATOR` from workspace `migrations/`, `run_migrations`, `ping` (`SELECT 1`).
- `error.rs`: `AppError` (`NotFound`, `Unauthorized`, `Forbidden`, `Validation`, `Conflict`, `BadRequest`, `Internal`) → RFC 9457 `application/problem+json`. `From` for `sqlx::Error`, `anyhow::Error`, `garde::Report`. Internal errors are logged; the client body is always `"An internal error occurred."`
- `telemetry.rs`: pretty vs json from `RWK_LOG_FORMAT`. Optional OTLP via feature `otel` (opentelemetry 0.32 + tracing-opentelemetry 0.33).
- `state.rs`: `AppState { config, db, mailer, jobs }` as `Clone` over `Arc`. Mailer/jobs are placeholders.

API:

- Bind address comes from `Config` (default `0.0.0.0:8080`).
- `GET /api/health` → `{"status":"ok","db":"ok"}` after `SELECT 1` (200). Failed ping returns **503** with `{"status":"degraded","db":"error"}`.
- `tower-http` `SetRequestId` / `PropagateRequestId` / `TraceLayer` so internal logs sit on a request span.

Migrations:

- `migrations/1_init.sql`: `citext`, `users`, `oauth_accounts`, `auth_tokens` (`auth_token_kind` enum), `projects`. Indexes: `projects(owner_id)`, `oauth_accounts(user_id)`, `auth_tokens(user_id)`, `auth_tokens(token_hash)`, `auth_tokens_expires_at_idx`.

Review follow-up (uncommitted migration edited in place): `sqlx database reset -y --source migrations` then `just migrate`. Indexes confirmed via `\di`.

Just recipes:

- `just migrate` → `sqlx migrate run --source migrations`
- `just migrate-new NAME` → `sqlx migrate add --source migrations --simple NAME`
- `just sqlx-prepare` → `cargo sqlx prepare --workspace -- --all-targets`
- `just check` also runs `cargo sqlx prepare --check --workspace -- --all-targets`
- `export DATABASE_URL := postgres://rwk:rwk@localhost:5432/rwk` (overridable)

`.sqlx/` offline cache is in the tree (three query files, including test queries). `.env.example` uses `RWK_DATABASE__URL` plus unprefixed `DATABASE_URL` for sqlx.

Crate versions from `cargo search` (2026-09-12): sqlx 0.9.0, figment 0.10.19, thiserror 2.0.20, anyhow 1.0.104, garde 0.23.0, uuid 1.26.1, chrono 0.4.45, tower-http 0.7.1, opentelemetry 0.32.0, opentelemetry_sdk 0.32.1, opentelemetry-otlp 0.32.0, tracing-opentelemetry 0.33.0. SQLx timestamps use chrono (as in the stack table); jiff 0.2.35 exists but is not wired.

## Verification

Commands from repo root with mise shims. Postgres was already up via `just db-up`.

### `just migrate`

Original apply:

```
sqlx migrate run --source migrations
Applied 1/migrate init (14.763301ms)
```

After in-place index edits (review):

```
sqlx database reset -y --source migrations
Applied 1/migrate init (20.855672ms)
just migrate   # no pending
```

`\di` shows `projects_owner_id_idx`, `oauth_accounts_user_id_idx`, `auth_tokens_user_id_idx`, `auth_tokens_token_hash_idx`, `auth_tokens_expires_at_idx`.

### Tests (`just test` / `cargo nextest run --workspace`)

14 passed, 0 skipped:

- config: defaults validate; `for_tests`; `from_env` loads `RWK_SERVER__PORT` / `RWK_LOG_FORMAT` / `RWK_APP_URL`; rejects port 0, invalid `app_url`, short session secret; production + default secret rejected; production + real secret accepted
- `AppError`: 404/401/403/409/400 problem+json; 422 with `errors.password`; 500 does not leak `"secret sauce"`; `sqlx::Error::RowNotFound` → 404
- `#[sqlx::test]`: citext + four tables exist; `ping` succeeds

`just test` exit 0 after the review fixes (`test-web` still prints Phase 6 TODO). `just sqlx-prepare` and `just check` also exit 0.

### `SQLX_OFFLINE=true cargo build --workspace`

```
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.87s
```

Exit 0. `SQLX_OFFLINE=true cargo clippy --all-targets -- -D warnings` also exit 0 after the test queries were prepared.

### `just check`

```
cargo fmt --all -- --check
cargo clippy --all-targets -- -D warnings
pnpm --dir apps/web typecheck
pnpm --dir apps/web lint
cargo sqlx prepare --check --workspace -- --all-targets
```

Exit 0.

### Health (manual, after `cargo run -p rwk-api`)

```
HTTP/1.1 200 OK
content-type: application/json
x-request-id: 1d27dae1-b390-4864-b0b9-91a3b95aa6ca

{"status":"ok","db":"ok"}
```

API process was stopped afterward (8080 free). Compose is still running.

### `otel` feature

`cargo check -p rwk-core --features otel` exit 0. Not part of default `just check`.

## Incomplete / notes

- `just test-web` remains a Phase 6 TODO.
- `just sqlx-prepare` / prepare `--check` pass `-- --all-targets` so `#[cfg(test)]` query macros are in `.sqlx/` (needed for offline `clippy --all-targets`).
- Figment `Error` is boxed inside `ConfigError` to satisfy `clippy::result_large_err`.
- `.env.example` documents `RWK_ENV`.
- No git commit, as requested. `.sqlx/` is untracked until the reviewer commits.
