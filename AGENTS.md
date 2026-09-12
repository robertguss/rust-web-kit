# rwk — agent instructions

AI-first Axum + React starter. Placeholder name is `rwk` (crates, npm package, database, compose, env prefix `RWK_`). Rename with `just init NAME` before real work.

`CLAUDE.md` is the primary brief; `AGENTS.md` is the same content.

## Architecture

```
crates/core     rwk-core   config, db, AppError, auth, users, projects, jobs, mail
crates/api      rwk-api    Axum binary, routes, OpenAPI, SPA static files
apps/web                   React 19 + Vite + TanStack Router/Query + shadcn/ui
migrations/                SQLx (.sql)
docs/slices/               copy-this feature walkthroughs (projects, auth)
docs/decisions/            ADRs
.claude/skills/            add-endpoint, add-migration, add-page, add-job, edit-hk-config
```

- HTTP API lives under `/api`. Scalar UI at `/docs`. Spec JSON at `/api/openapi.json` and the committed export `apps/web/openapi.json`.
- Default process = HTTP + job worker. `--api-only` / `--worker-only` split them.
- Sessions: HttpOnly SameSite=Lax cookie via in-tree Postgres store. Not JWT.
- Jobs: in-tree Postgres queue (`jobs` table). Enqueue on the same executor as the business write.
- Frontend talks to `/api` with credentials included. Vite proxies `/api` → `:8080`.

Read `docs/slices/projects.md` before adding a CRUD resource. Read `docs/slices/auth.md` before touching auth.

## just vocabulary

| Recipe | What it does |
|---|---|
| `setup` | `mise install`, JS deps, `hk install` |
| `db-up` / `db-down` | Postgres 18 + Mailpit (`RWK_DB_PORT` moves the host port off 5432) |
| `dev` / `dev-api` / `dev-web` | API `:8080` + Vite `:5173` |
| `migrate` / `migrate-new NAME` | SQLx migrations |
| `sqlx-prepare` | Refresh committed `.sqlx/` |
| `gen` | Export OpenAPI + generate TS client |
| `check` | fmt, clippy `-D warnings`, tsc, biome, sqlx `--check`, gen freshness |
| `test` / `test-api` / `test-web` | nextest + Vitest |
| `e2e` | Playwright smoke (Postgres + Mailpit) |
| `ci` | `check` + `test` + `e2e` + `docker-build` |
| `build` / `docker-build` / `deploy HOST [PLATFORM]` | release binary, image, SSH deploy (defaults to `linux/amd64`) |
| `init NAME` | one-shot rename (deletes itself) |

Run `just check` before declaring work done. Run `just ci` before a PR.

## Conventions

- **Vertical slices.** Domain code is `crates/core/src/<feature>/{mod,model,repo,service}.rs`. HTTP is `crates/api/src/routes/<feature>.rs` plus DTOs in `crates/api/src/dto.rs`. Wire the router in `crates/api/src/router.rs`.
- **`AppError`.** Handlers return `Result<_, AppError>`. Map sqlx/anyhow/garde via existing `From` impls. Internal details are logged, never sent. Responses are RFC 9457 `application/problem+json`.
- **garde.** Validate request DTOs in the handler (`body.validate()?`) before calling the service. Service trusts validated input.
- **utoipa.** Every handler has `#[utoipa::path]`. Paths are relative to the `/api` mount (`servers[0].url = "/api"`). Register the handler with `utoipa_axum::routes!` and add new schemas to `ApiDoc`.
- **Generated client.** `just gen` writes `apps/web/openapi.json` and `apps/web/src/api/generated/`. Never edit `apps/web/src/api/generated/` by hand. Hand-written files: `client.ts`, `problem.ts`, `form-errors.ts`, `query.ts`.
- **Ownership.** Missing rows and other users' rows are `AppError::NotFound` (404), not 403.
- **Tests.** API: `#[sqlx::test]` + `crates/api/tests/common` (`TestApp`, cookie client, `drain_jobs`). Web: Vitest. Do not `#[ignore]` tests to go green.
- **Files.** Small, one job, short module docs. Edition 2024. Clippy pedantic (`-D warnings`). TypeScript strict. Biome clean.

## Secrets

Config is figment with prefix `RWK_` and nested `__` keys. Names live in `.env.example`. Values live in fnox (age) or the process environment.

- Never print `fnox` values, `.env` contents, age keys, or `RWK_SESSION__SECRET` / OAuth client secrets / SMTP passwords.
- `fnox exec -- <cmd>` injects secrets. `fnox.toml` recipients must be the operator's age public key.
- Dev DB password `rwk` is a published local default, not a secret.

## Add a feature (8 steps)

1. **Schema** — `just migrate-new NAME`, write SQL, `just migrate`.
2. **Core** — model / repo / service under `crates/core/src/<feature>/`. Repo talks to SQLx; service maps `None` → `AppError::NotFound` (or enqueue jobs on the same transaction).
3. **DTOs** — garde-validated structs in `crates/api/src/dto.rs`.
4. **HTTP** — handlers in `crates/api/src/routes/<feature>.rs` with `#[utoipa::path]`, `RequireAuth` when needed, `AppError` results. Nest in `router.rs`, add schemas/tags to `ApiDoc`.
5. **Client** — `just gen`. Import generated query/mutation hooks only.
6. **UI** — TanStack file route under `apps/web/src/routes/`. Forms: react-hook-form + zod in `apps/web/src/lib/schemas.ts`, map Problem Details with `applyProblem`.
7. **Tests** — integration tests in `crates/api/tests/` (auth, ownership, validation). Vitest for form/error mapping when the UI is non-trivial.
8. **Verify** — `just sqlx-prepare` if queries changed, then `just check`. `just ci` before a PR.

Skills under `.claude/skills/` expand steps 1, 4, 5–6, jobs, and `hk.pkl`.

## MCP (optional)

`.mcp.json` starts an OpenAPI server on `apps/web/openapi.json` and a read-only Postgres server on the local DB. They are optional: enable in the MCP client if useful; the repo builds and tests without them. Do not run writes through the Postgres MCP server.

## Local ports

API `8080`, Vite `5173`, Postgres `5432`, Mailpit UI `8025` / SMTP `1025`.
