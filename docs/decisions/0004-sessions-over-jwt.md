# 0004. Sessions over JWT

## Context

The SPA needs login state. JWTs in localStorage are XSS-stealable and hard to revoke. The API already has Postgres.

## Decision

Cookie sessions via tower-sessions: HttpOnly, SameSite=Lax, Secure in production, Postgres store (in-tree; the published sqlx store is on sqlx 0.8). Login cycles the session id. `CurrentUser` / `RequireAuth` extractors read `user_id` from the session.

OAuth PKCE `state` also lives in the session.

## Consequences

Logout is a server-side delete. CSRF is mitigated by SameSite=Lax plus an origin check on mutating requests. Mobile native clients would need a different authenticator; this kit is a cookie-session web app.
