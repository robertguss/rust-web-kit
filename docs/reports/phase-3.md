# Phase 3 report — Users, sessions, password auth

Status: complete. All Phase 3 acceptance items were run on this machine. Nothing was committed.

## What was built

### `rwk-core`

- `users/`: `User` / `UserResponse`, repo (`create`, `find_by_email`, `find_by_id`, `set_verified`, `update_password`), thin service wrappers.
- `auth/password.rs`: Argon2id via `argon2` 0.6 + `password-hash` 0.6.
- `auth/tokens.rs`: 32 random bytes (URL-safe base64), SHA-256 hash stored; consume is a single atomic `UPDATE … RETURNING user_id`. TODO: Phase 5 cleanup must also delete expired `sessions` rows.
- `auth/service.rs`: register, login, logout, verify-email, forgot-password (always 204), reset-password. Login always runs Argon2 against a real hash or a static dummy hash so missing users do not leak via timing.
- `auth/extract.rs`: `CurrentUser(Option<User>)`, `RequireAuth(User)` (401 if missing). `login` calls `session.cycle_id()` before inserting `user_id`.
- `auth/session_store.rs`: Postgres `SessionStore` for tower-sessions 0.15 + sqlx 0.9.
- `mail.rs`: `Mailer` trait, `LogMailer` (logs the link), `RecordingMailer` (tests).
- `AppState` now holds `Arc<dyn Mailer>`.

### `rwk-api`

- Library crate + binary. `OpenApiRouter` from utoipa-axum; every handler has `#[utoipa::path]`.
- Routes under `/api/auth/*`: register (201), login (200), logout (204), me (200/401), verify-email (204), forgot-password (204), reset-password (204).
- Scalar at `/docs`, spec at `/api/openapi.json`.
- tower-sessions cookie: HttpOnly, SameSite=Lax, Secure in production, name from `Config`.
- `tower_governor` on auth routes (skipped when `RWK_ENV=test` so oneshot tests have no `SocketAddr`).
- Origin check on mutating requests (skipped in test env).
- Security stack: request-id, TraceLayer, timeout 30s, gzip, CORS (credentials + `app_url` origin), `X-Content-Type-Options: nosniff`, `X-Frame-Options: DENY`, `Referrer-Policy: no-referrer`. Layer order is SetRequestId (outer) → Propagate → Trace so spans see `x-request-id`.
- Binary uses `into_make_service_with_connect_info::<SocketAddr>()` for governor in non-test.

### Tests

`crates/api/tests/common`: builds the app with `Config::for_tests()`, a `#[sqlx::test]` pool, `RecordingMailer`, and a cookie-aware oneshot client.

### Migration

`migrations/2_sessions.sql`: `sessions(id, data, expiry_date)` for the in-tree store.

### Versions (`cargo search`, 2026-09-12)

tower-sessions 0.15.0, argon2 0.6.0, password-hash 0.6.1, rand 0.10.2, utoipa 5.5.0, utoipa-axum 0.2.0, utoipa-scalar 0.3.0, tower_governor 0.8.0, garde 0.23.0.

**Note:** `tower-sessions-sqlx-store` 0.15.0 still depends on sqlx ^0.8 and tower-sessions-core ^0.14. That cannot share a `PgPool` with this workspace (sqlx 0.9 / tower-sessions 0.15). The store is implemented in-tree against the current crates; schema matches the published store.

## Verification

Postgres was up via `just db-up`. `just migrate` applied `2_sessions`.

### `just sqlx-prepare`

```
cargo sqlx prepare --workspace -- --all-targets
query data written to .sqlx in the workspace root
```

Exit 0. `.sqlx/` now has 16 query files.

### `just check`

```
cargo fmt --all -- --check
cargo clippy --all-targets -- -D warnings
pnpm --dir apps/web typecheck
pnpm --dir apps/web lint
cargo sqlx prepare --check --workspace -- --all-targets
```

Exit 0.

### `just test`

24 passed, 0 skipped (after review fixes):

- register → login → me → logout → me (401)
- duplicate email → 409 problem+json
- bad password → 401
- validation → 422 with field errors
- verify-email happy path (`email_verified: true`)
- verify-email expired → 400
- forgot-password always 204; reset happy path (old password 401, new 200); expired reset → 400
- `/api/openapi.json` and `/docs` 200
- every response has `x-request-id`
- session cookie value changes across login
- existing config / AppError / migration tests

`test-web` still prints Phase 6 TODO.

## Incomplete / notes

- OAuth routes are Phase 3 stack mention only for password auth; PLAN lists OAuth under later/stack table, not Phase 3 handlers.
- Governor is off in `Environment::Test` because `tower::ServiceExt::oneshot` has no connect info.
- Phase 5 cleanup job still needs to delete expired `sessions` rows (noted in `mail.rs` and `tokens.rs`).
- No git commit, as requested.
