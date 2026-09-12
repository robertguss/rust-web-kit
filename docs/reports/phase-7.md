# Phase 7 report — OAuth, Playwright, `just ci`, Docker, deploy

Status: complete. All Phase 7 acceptance items were run on this machine. Nothing was committed.

## What was built

### SPA `/assets/` 404

`ServeDir` is nested at `/assets` with no index fallback. Missing hashed files return HTTP 404. Other unknown paths still get `index.html` (200). Covered in `crates/api/tests/spa.rs`.

### OAuth (Google + GitHub)

`oauth2` 5.0.0 with PKCE (S256) and CSRF `state` stored in the tower-session (`oauth_pending`). Providers register only when **both** client id and secret are set; unknown or unconfigured provider → 404.

- `GET /api/auth/oauth/{provider}` → 302 to the provider
- `GET /api/auth/oauth/{provider}/callback?code&state` → 302 to `/app/projects` on success; browser-facing failures 302 to `/login?error=<state|provider|exchange|conflict>`. Unknown/unconfigured provider stays 404 problem+json. The login page shows a short message for that search param.

Account linking (`link_or_create`):

1. existing `(provider, provider_user_id)`
2. else existing **verified** email (insert `oauth_accounts`)
3. else create user with `password_hash` null and `email_verified_at` set

Unverified email that already exists → 409 (no takeover). GitHub profile email comes from `GET /user/emails` (verified primary). Google requires `email_verified` on userinfo.

### Playwright

`apps/web/e2e/smoke.spec.ts`: register → verify link from Mailpit REST (`http://localhost:8025/api/v1`) → login → create / edit / delete project → logout. `just e2e` builds the SPA, starts `rwk-api` if 8080 is free, installs Chromium, runs Playwright against `http://localhost:8080`.

### CI / Docker / deploy

- `just ci` = `just check && just test && just e2e && just docker-build`
- `just check` includes fmt, clippy `-D warnings`, biome, tsc, `just gen-check`, `cargo sqlx prepare --check`
- `just gen-check` hashes `apps/web/openapi.json` + `apps/web/src/api/generated` (find | sort | xargs sha256sum | sha256sum), runs `just gen`, hashes again; fails if they differ. Independent of git index/HEAD.
- `Dockerfile`: node `pnpm build` → cargo-chef 0.1.78 + BuildKit cache mounts → `debian:bookworm-slim`, user `rwk` (uid 1000), `HEALTHCHECK` on `/api/health`, `RWK_SERVER__STATIC_DIR=/app/www`, `RWK_RUN_MIGRATIONS=true`, `EXPOSE 8080`
- `docker-compose.prod.yml` (project `rwk-prod`): app + postgres volume. Secrets via `fnox exec -- docker compose -f docker-compose.prod.yml up -d`. Requires `RWK_SESSION__SECRET`.
- exe.dev: image exposes 8080 so the HTTPS proxy should target 8080 (`ssh exe.dev share port <vmname> 8080`). Notes in the compose file follow https://exe.dev/docs/proxy.md.
- `just deploy HOST`: `docker build`, `docker save | ssh HOST docker load`, scp `docker-compose.prod.yml` **and** `fnox.toml` into `rwk/` on the VM, remote `fnox exec -- docker compose … up -d`
- `.github/workflows/ci.yml`: mise-action, Postgres 17 + Mailpit services, `just ci`

## Versions (`cargo search` / `pnpm view`, 2026-09-12)

- oauth2 5.0.0 (reqwest + rustls-tls; pulls reqwest 0.12.28)
- cargo-chef 0.1.78
- `@playwright/test` 1.63.0

## Verification

Postgres + Mailpit were up via `just db-up` (dev compose).

### `just sqlx-prepare`

```
cargo sqlx prepare --workspace -- --all-targets
query data written to .sqlx in the workspace root
```

Exit 0. Four new query files for oauth account insert/lookup, oauth user insert, and the link test count.

### `just ci`

Exit 0:

| Step | Result |
|---|---|
| `just check` | fmt, clippy `-D warnings`, tsc, biome, sqlx `--check`, `just gen-check` |
| `just test` | 54 nextest passed, 0 skipped; Vitest 7 passed |
| `just e2e` | Playwright chromium smoke passed (3.3s) |
| `just docker-build` | `rwk:latest` |

### `docker compose -f docker-compose.prod.yml up`

```
RWK_SESSION__SECRET=a-sufficiently-long-production-secret \
RWK_APP_URL=http://localhost:8080 \
docker compose -f docker-compose.prod.yml up -d --no-build
```

| Request | Result |
|---|---|
| `GET /api/health` | 200 `{"status":"ok","db":"ok"}` |
| `GET /` | 200 `text/html` (built SPA) |
| `GET /app/projects` | 200 (SPA fallback) |
| `GET /assets/missing.js` | 404 |

Prod compose is left running on 8080 (`rwk-prod`). Dev compose Postgres remains on host 5432.

### Live OAuth (manual)

Not run here: no Google/GitHub client credentials on this machine. To verify:

1. Create OAuth apps with callback `{RWK_APP_URL}/api/auth/oauth/{google|github}/callback`
2. Set both `RWK_OAUTH__*_CLIENT_ID` and `*_CLIENT_SECRET`
3. Restart the API, click Continue with Google/GitHub on `/login`
4. Confirm session cookie, `/api/auth/me`, and a row in `oauth_accounts`

Covered automatically: unconfigured 404, start 302 + PKCE, bad/missing state and provider `error=` → 302 `/login?error=…`, linking unit tests.

## Review follow-up

1. Replaced `git diff --exit-code` in `just check` with `just gen-check` (content hash before/after `just gen`).
2. OAuth callback is a browser navigation: success → `/app/projects`; failures → `/login?error=`; 404 unchanged for unknown/unconfigured.
3. `just deploy` also copies `fnox.toml` next to the compose file.

## Incomplete / notes for the reviewer

- Live provider login was not executed (no secrets). Linking and state tests are automated.
- Do not commit from this agent.
