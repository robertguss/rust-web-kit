# Phase 5 report — Jobs and email

Status: complete. All Phase 5 acceptance items were run on this machine. Nothing was committed.

## What was built

### Jobs (in-tree Postgres queue)

`apalis-postgres` 1.0.0-rc.8 still depends on **sqlx 0.8.1** (`[dependencies.sqlx] version = "0.8.1"`). This workspace is on sqlx 0.9 and cannot share a `PgPool` with it, so the queue is implemented in-tree (same pattern as the Phase 3 session store).

- Migration `migrations/3_jobs.sql`: `jobs(id, kind, payload jsonb, run_at, attempts, max_attempts, last_error, locked_at, locked_by, completed_at, created_at)` with `jobs_due_idx` (partial, due rows), `jobs_kind_idx`, and unique `jobs_cleanup_pending_idx` on `(kind)` where `completed_at IS NULL AND kind = 'cleanup_expired_tokens'`.
- `jobs::enqueue` takes `impl Executor<'_, Database = Postgres>` so it can join a caller transaction (`&mut *tx`) or a pool. Cron ticks use `enqueue_cron` (`ON CONFLICT … DO NOTHING`) so a pending cleanup is never duplicated across workers or restarts.
- Claim uses a single `WITH … FOR UPDATE SKIP LOCKED` / `UPDATE … RETURNING` statement. Stale locks older than 5 minutes are reclaimable. Failures clear the lock, store `last_error`, and set `run_at` to now + exponential backoff (`2^attempts` seconds, capped). After `max_attempts` the row is marked complete (dead).
- Jobs: `SendEmail { to, template, data }` and `CleanupExpiredTokens`.
- Cron registry: `cron_jobs()` returns hourly `CleanupExpiredTokens`. The worker ticks that interval (first tick at startup) and enqueues those jobs. Poll interval is 400ms.
- `CleanupExpiredTokens` deletes **expired `auth_tokens` and expired `sessions`**, and `jobs` rows with `completed_at` older than 7 days.
- Binary: default runs API + worker; `--api-only` / `--worker-only` (mutually exclusive). Worker is aborted when the HTTP server exits.

### Mail

- `Mailer` trait: `send(&Email)`.
- `SmtpMailer` via lettre 0.11 (tokio1 + rustls). No credentials → `builder_dangerous` (Mailpit). Username set → STARTTLS relay + credentials.
- `LogMailer` logs and captures rendered messages (worker tests).
- `RecordingMailer` captures messages (HTTP tests).
- Askama 0.16 templates: `crates/core/templates/verify_email.html`, `password_reset.html`, plus a plain-text fallback. Multipart alternative is what SMTP sends.

### Auth wiring

`register` and `forgot_password` open a transaction, insert the user/token, enqueue `SendEmail` on that same executor, then commit. HTTP tests drain the queue through `Worker` + `RecordingMailer` before reading the token from the rendered URL.

### Docs

README notes Mailpit at http://localhost:8025 and the `--api-only` / `--worker-only` flags.

## Versions (`cargo search` / crate Cargo.toml, 2026-09-12)

- lettre 0.11.23 (`builder`, `hostname`, `smtp-transport`, `pool`, `tokio1-rustls-tls`; default `native-tls` off)
- askama 0.16.1
- apalis-postgres 1.0.0-rc.8 — **not used** (sqlx 0.8.1)

## Verification

Postgres and Mailpit were already up via `just db-up`. `just migrate` applied `3_jobs`. Review follow-up edited `3_jobs.sql` in place, then `sqlx database reset -y --source migrations` and `just migrate`.

### `just sqlx-prepare`

```
cargo sqlx prepare --workspace -- --all-targets
query data written to .sqlx in the workspace root
```

Exit 0. `.sqlx/` now has 39 query files after the review follow-up.

### `just check`

```
cargo fmt --all -- --check
cargo clippy --all-targets -- -D warnings
pnpm --dir apps/web typecheck
pnpm --dir apps/web lint
cargo sqlx prepare --check --workspace -- --all-targets
just gen
git diff --exit-code -- apps/web/src/api apps/web/openapi.json
```

Exit 0.

### `just test`

39 passed, 0 skipped (nextest). `just test-web` still prints Phase 6 TODO.

Phase 5 coverage:

- enqueue inside a transaction then rollback → `jobs` count 0
- worker processes `SendEmail` with `LogMailer`; captured HTML/text include the URL and the welcome copy
- `cron_jobs()` registers hourly `CleanupExpiredTokens`
- `enqueue_cron` twice → one pending `cleanup_expired_tokens` row
- cleanup deletes expired `auth_tokens` and expired `sessions`, keeps live rows; deletes `jobs` completed > 7 days ago, keeps a 1-day-old completed row
- failed send retries with `run_at` in the future
- existing auth/projects tests still pass (register/forgot drain jobs for tokens)

### Manual Mailpit

With compose Postgres + Mailpit running:

```
cargo run -p rwk-api
curl -sS -X POST http://127.0.0.1:8080/api/auth/register \
  -H 'content-type: application/json' \
  -d '{"email":"mailpit-1789186006@example.com","password":"password12"}'
# → 201
# open http://localhost:8025  (or GET /api/v1/messages)
```

Observed: register 201, then Mailpit total=1 within ~1s.

- From: `noreply@localhost`
- To: `mailpit-1789186006@example.com`
- Subject: `Verify your email`
- HTML/text contain `http://localhost:8080/verify-email?token=…`

API process was stopped afterward (8080 free). Compose is still running.

## Review follow-up

- Cleanup also `DELETE FROM jobs WHERE completed_at < now() - interval '7 days'`.
- Unique partial index `jobs_cleanup_pending_idx`; worker cron path uses `enqueue_cron` (`ON CONFLICT DO NOTHING`).
- Re-ran `just sqlx-prepare`, `just check`, `just test` after the reset. All green (39 tests).

## Incomplete / notes for the reviewer

- In-tree queue instead of apalis, because apalis-postgres is still on sqlx 0.8. Documented above.
- Worker interval first tick still enqueues cleanup at process start (then hourly); duplicates are now a no-op while one is pending.
- Do not commit from this agent.
