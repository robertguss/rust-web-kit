# Phase 6 report — Frontend

Status: complete. All Phase 6 acceptance items were run on this machine. Nothing was committed.

## What was built

### Routes (TanStack Router file-based + Vite plugin)

`apps/web/src/routes/`: `/`, `/login`, `/register`, `/forgot-password`, `/reset-password`, `/verify-email`, `/app` (authed layout + nav), `/app/projects`, `/app/projects/new`, `/app/projects/$id`. Generated `src/routeTree.gen.ts` (Biome-ignored).

`/app` `beforeLoad` uses `fetchCurrentUser` (`me` query, 401 → logged out) and redirects to `/login?redirect=…`. Return-to is restricted to `/app*` paths. Login/register bounce an already-signed-in user into the app.

### Auth + projects UI

- react-hook-form + zod 4 + `@hookform/resolvers`, shadcn Field/Input/Button.
- Server `fieldErrors` mapped via `applyProblem` / `applyFieldErrors`.
- OAuth buttons link to `/api/auth/oauth/{google,github}` (handlers land in Phase 7).
- Projects list with `page` search param, create/edit form, delete confirm (`alert-dialog`). Mutations invalidate generated query keys.
- Layout, `next-themes` dark mode (`rwk-theme`), sonner toasts, pending skeletons, root error/not-found boundaries.

### API client

Hand-written `src/api/client.ts` (hey-api `runtimeConfigPath`) still sets `baseUrl: '/api'` and `credentials: 'include'`. `installProblemInterceptor` (called from `main.tsx` and Vitest setup) is a response interceptor: non-2xx Problem Details become `ProblemError`. An error interceptor covers the parsed-JSON throw path. Generated `src/api/generated` is not edited.

Vite proxies `/api` to `:8080` and rewrites `Origin` to `http://localhost:8080` so the API origin check matches `app_url` during `just dev`.

### Axum SPA

`RWK_SERVER__STATIC_DIR` (default `apps/web/dist`). `tower-http` `fs` feature. `ServeDir` + `fallback(ServeFile index.html)` on the outer router so `/api` and `/docs` stay first. **`not_found_service` is not used** — it forces HTTP 404 even when serving `index.html`.

### Tasks

- `just dev`: API + Vite together (bash trap, both recipes).
- `just test-web`: `pnpm --dir apps/web test` (Vitest).
- `just check` freshness diff is `apps/web/src/api/generated` + `openapi.json` (hand-written `client.ts` / `problem.ts` are allowed to change).

### Tests

Vitest + jsdom + Testing Library:

- Login form client validation
- Login form maps 422 `errors.email` onto the field
- Interceptor throws `ProblemError`
- `parseProblem` / `applyProblem` / `applyFieldErrors`

Rust: `crates/api/tests/spa.rs` — temp dist, `/` and nested path return the SPA, asset served, `/api/health` still JSON.

## Versions (`pnpm view` / `cargo search`, 2026-09-12)

- `@tanstack/react-router` 1.170.35
- `@tanstack/router-plugin` 1.168.37 (peer `^1.170.34`)
- `@tanstack/react-router-devtools` 1.167.1
- `@tanstack/react-query` 5.102.8 (already in tree)
- `vitest` 5.0.0, `jsdom` 30.0.1
- `@testing-library/react` 16.3.3, `user-event` 14.6.7, `jest-dom` 7.0.1
- `sonner` 2.0.8, `next-themes` 0.4.6 (pulled by shadcn sonner)
- `tower-http` 0.7.1 + `fs`

## Verification

Postgres and Mailpit were already up via `just db-up`.

### `just check`

Exit 0: fmt, clippy `-D warnings`, `pnpm typecheck`, biome, sqlx prepare `--check`, `just gen`, generated-client freshness.

### `just test`

40 nextest passed, 0 skipped. Vitest 7 passed (3 files).

### `pnpm --dir apps/web build`

Exit 0. `apps/web/dist` produced.

### API serves the built app

`cargo run -p rwk-api` (8080). Then:

| Request | Result |
|---|---|
| `GET /` | 200 `text/html`, built `index.html` + hashed assets |
| `GET /app/projects` | 200, same body as `/` |
| `GET /api/health` | `{"status":"ok","db":"ok"}` |

Cookie-aware curl against the same process:

- register 201 → `GET /api/auth/me` 200
- create project 200/201, list, PATCH name, GET, DELETE 204
- Mailpit message for the new user; `POST /api/auth/verify-email` 204; `me.email_verified` true
- logout 204 → me 401 → login 200 → me 200

API process was stopped afterward (8080 free). Compose is still running.

Playwright browser smoke is Phase 7.

## Incomplete / notes for the reviewer

- OAuth buttons hit routes that do not exist until Phase 7.
- `just check` freshness path was narrowed from `apps/web/src/api` to `apps/web/src/api/generated` so hand-written interceptors are not flagged as a stale client.
- Do not commit from this agent.
