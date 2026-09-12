# Slice: auth

Password auth, sessions, email tokens, OAuth. Read this before changing login, cookies, or identity.

## Shape

- `users`: uuid v7, `citext` email unique, `password_hash` nullable (OAuth-only), `email_verified_at`.
- `oauth_accounts`: unique `(provider, provider_user_id)`.
- `auth_tokens`: kind `email_verify` | `password_reset`, SHA-256 of 32 random bytes, `expires_at`, `used_at`. Consume is one `UPDATE … RETURNING`.
- `sessions`: tower-sessions Postgres store (`migrations/2_sessions.sql`). Cookie: HttpOnly, SameSite=Lax, Secure in production, name `rwk_session`.
- Rate limit (`tower_governor`) on `/api/auth/*` except `RWK_ENV=test`. Origin check on mutating requests (also skipped in test).

## Files

| Layer | Path |
|---|---|
| Migrations | `migrations/1_init.sql`, `2_sessions.sql` |
| Users | `crates/core/src/users/{model,repo,service}.rs` |
| Password | `crates/core/src/auth/password.rs` (argon2id + dummy hash on unknown email) |
| Tokens | `crates/core/src/auth/tokens.rs` |
| Session | `crates/core/src/auth/session_store.rs`, `extract.rs` (`CurrentUser`, `RequireAuth`, `RequireVerified`) |
| Service | `crates/core/src/auth/service.rs` |
| OAuth | `crates/core/src/auth/oauth.rs` — PKCE + state in session, Google + GitHub |
| HTTP | `crates/api/src/routes/auth.rs`, `oauth.rs` |
| Router | `crates/api/src/router.rs` — `/auth` nest + governor |
| Mail jobs | `register` / `forgot_password` enqueue `SendEmail` on the same transaction |
| Templates | `crates/core/templates/verify_email.html`, `password_reset.html` |
| Tests | `crates/api/tests/auth.rs`, `oauth.rs` |
| UI | `apps/web/src/routes/{login,register,forgot-password,reset-password,verify-email}.tsx`, `apps/web/src/components/unverified-email-banner.tsx` |
| Forms | `apps/web/src/components/auth/*` |
| OAuth buttons | `apps/web/src/components/oauth-buttons.tsx` → `/api/auth/oauth/{google,github}` |

## HTTP

| Method | Path | Notes |
|---|---|---|
| POST | `/auth/register` | 201 `{user}`; session cookie; verify email enqueued |
| POST | `/auth/login` | 200 `{user}` |
| POST | `/auth/logout` | 204 |
| GET | `/auth/me` | 200 `{user}` or 401 |
| POST | `/auth/verify-email` | `{token}` → 204 |
| POST | `/auth/resend-verification` | session required; 204; no-op if already verified |
| POST | `/auth/forgot-password` | `{email}` → **always 204** |
| POST | `/auth/reset-password` | `{token,password}` → 204 |
| GET | `/auth/oauth/{provider}` | 302 to provider (404 if unset) |
| GET | `/auth/oauth/{provider}/callback` | 302 `/app/projects` or `/login?error=` |

Forgot-password must not leak whether the email exists.

## Session

`CurrentUser::login` calls `session.cycle_id()` then stores `user_id`. `RequireAuth` is 401 when missing. `RequireVerified` wraps `RequireAuth` and returns `AppError::EmailNotVerified` (403, type `/problems/email-not-verified`) when `email_verified_at` is null. It is **opt-in per route** — login, session, and existing handlers do not require a verified address. `POST /auth/resend-verification` needs `RequireAuth` only: it issues a fresh `email_verify` token and enqueues `SendEmail` on the same transaction (or 204 with no token/mail if already verified). Integration tests use the cookie-aware client in `crates/api/tests/common`.

## OAuth linking

1. Existing `(provider, provider_user_id)`.
2. Else existing **verified** email → insert `oauth_accounts`.
3. Else create user with `password_hash` null and `email_verified_at` set.

Unverified email already present → 409 (no takeover). Providers register only when **both** client id and secret are set.

## Frontend

`/app` `beforeLoad` uses `fetchCurrentUser` (`apps/web/src/api/query.ts`). 401 → `/login?redirect=` (return-to restricted to `/app*`). Forms map 422 `errors` with `applyProblem`. The authed layout shows a dismissible banner when `email_verified` is false; Resend calls `POST /auth/resend-verification`.

## Tests to keep green

Register → login → me → logout → me 401; duplicate email 409; bad password 401; validation 422; verify/reset happy and expired; resend-verification 204 (+ one job, or none if already verified); unauthenticated resend 401; `RequireVerified` 403 until verified; forgot-password always 204; session cookie value changes on login; OAuth unconfigured 404 and bad state → `/login?error=`.
