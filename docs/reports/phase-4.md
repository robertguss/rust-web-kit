# Phase 4 report — Projects CRUD, OpenAPI export, TS client generation

Status: complete. All Phase 4 acceptance items were run on this machine. Nothing was committed.

## What was built

### `rwk-core`

- `projects/`: `Project` / `ProjectPage`, owner-scoped repo (`create`, `find_owned`, `count_owned`, `list_owned`, `update_owned`, `delete_owned`), service mapping a missing row to `AppError::NotFound` (other users see 404, not 403).
- Pagination: `page` default 1, `per_page` default 20 (garde 1–100). List is newest-first (`created_at DESC, id DESC`).
- `update_owned` uses `COALESCE` so omitted PATCH fields are unchanged (cannot null-clear `description`).

### `rwk-api`

- `/api/projects` CRUD with `RequireAuth` + utoipa. Request bodies: `CreateProject` `{name, description?}`, `UpdateProject` `{name?, description?}`.
- HTTP router nests the documented routes at `/api`. Spec paths are unprefixed (`/health`, `/auth/*`, `/projects`, `/projects/{id}`) with `servers[0].url = "/api"` so a generated client can use `baseUrl: '/api'`.
- Scalar remains at `/docs`. Spec JSON at `/api/openapi.json` (added after `split_for_parts`, not itself a spec path).
- `rwk-api --export-openapi PATH`: clap is parsed first; the process writes pretty-printed JSON and exits. No config load, no tracing init, no database.

### Frontend

- `just gen` writes `apps/web/openapi.json` then runs `@hey-api/openapi-ts` into `apps/web/src/api/generated/` (`clean: true`).
- Hand-written `apps/web/src/api/client.ts` is the hey-api `runtimeConfigPath`: `baseUrl: '/api'`, `credentials: 'include'`.
- Hand-written `apps/web/src/api/problem.ts`: `ProblemError` with `fieldErrors`, plus `parseProblem` / `isProblemDetails`.
- Generated output is under `src/api/generated` rather than the `src/api` root because hey-api also emits a `client/` directory; a sibling `client.ts` would collide with `import … from './client'`.
- Biome ignores `src/api/generated`. `just check` runs `just gen` then `git diff --exit-code -- apps/web/src/api apps/web/openapi.json`.

### Tests

`crates/api/tests/projects.rs`:

- unauthenticated list/create → 401 problem+json
- full CRUD (create 201, get, patch name keeps description, list defaults, delete 204, get 404)
- other user get/patch/delete → 404; their list is empty; owner still sees the row
- pagination: 3 items, `per_page=2` → page 1 has 2, page 2 has 1, `total=3`
- validation: empty name 422 with `errors.name`; `page=0` / `per_page=101` 422; empty PATCH name 422
- missing id 404
- `openapi()` (no DB) includes `/health`, `/auth/register`, `/projects`, `/projects/{id}`, server `/api`

## Versions (`pnpm view` / `cargo search`, 2026-09-12)

- clap 4.6.6 (workspace)
- `@hey-api/openapi-ts` 0.99.0
- `@tanstack/react-query` 5.102.8

`@hey-api/client-fetch` 0.13.1 is deprecated (bundled inside generated output as of openapi-ts 0.73). It is not a runtime dependency; the Fetch client is generated into `src/api/generated/client/`.

## Verification

Postgres was already up via `just db-up`.

### `just sqlx-prepare`

```
cargo sqlx prepare --workspace -- --all-targets
query data written to .sqlx in the workspace root
```

Exit 0. `.sqlx/` now has 22 query files (6 new project queries).

### `just gen`

```
cargo run -p rwk-api -- --export-openapi apps/web/openapi.json
pnpm --dir apps/web exec openapi-ts
```

Exit 0. Ran twice and `diff` of `openapi.json` + `src/api/generated` was empty (idempotent).

`--export-openapi` also succeeds with `DATABASE_URL` unset (`env -u DATABASE_URL cargo run -p rwk-api -- --export-openapi /tmp/rwk-openapi-nodb.json`).

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

Exit 0. Frontend typecheck is green against the generated client.

### `just test`

31 passed, 0 skipped (nextest). `just test-web` is still the Phase 6 stub.

## Incomplete / notes for the reviewer

- Generated client + `apps/web/openapi.json` are untracked until you commit them. `git diff --exit-code` only flags staleness once those paths are tracked.
- Do not commit from this agent.
