# rust-web-kit

AI-first full-stack starter: Rust (Axum) API + React (Vite) frontend, shipped as a single Docker image.

The placeholder project name is `rwk` (crates, npm package, database, compose project, env prefix `RWK_`). Rename it before you start building your product.

On GitHub, mark this repository as a **template repository** (Settings → General → Template repository) so others can use “Use this template”. That checkbox is set by the owner; it is not in git.

## Quickstart

```bash
# 1. Create a repo from this template (GitHub “Use this template”), then clone it.
# 2. Rename the placeholder:
just init my-app          # NAME must match [a-z][a-z0-9-]*
# 3. Install tools, JS deps, git hooks:
just setup
# 4. Data plane + app:
just db-up
just dev
```

`just init NAME` rewrites `rwk` → `NAME` in file contents and paths, retitles this README, deletes the init script, and runs `git init` plus `mise install` / `pnpm` / `hk install`. Run it once, on a fresh copy.

If port 5432 is already used by another Postgres on your machine, pick a free one. Export it so every recipe and the app agree.

```bash
export RWK_DB_PORT=5433
just db-up
```

`just db-up` fails with the conflicting listener if `DATABASE_URL` reaches a Postgres other than the container.

Health: `curl localhost:8080/api/health` → `{"status":"ok","db":"ok"}`.

`just dev` runs the API on port 8080 and Vite on port 5173 (proxies `/api`). After `pnpm --dir apps/web build`, the API serves the SPA from `apps/web/dist`.

OpenAPI: [http://localhost:8080/docs](http://localhost:8080/docs) (Scalar). Mailpit: [http://localhost:8025](http://localhost:8025). The API process runs the job worker by default (`--api-only` / `--worker-only` to split).

OAuth (Google / GitHub) is on when both `RWK_OAUTH__<PROVIDER>_CLIENT_ID` and `_CLIENT_SECRET` are set.

## Stack

| Concern | Choice |
|---|---|
| API | Axum, tokio, utoipa + Scalar |
| DB | PostgreSQL 18, SQLx (offline `.sqlx/` committed) |
| Auth | argon2 passwords, tower-sessions cookie, Google/GitHub OAuth (PKCE) |
| Jobs | in-tree Postgres queue (transactional enqueue) |
| Email | lettre + askama; Mailpit locally |
| Frontend | React 19, Vite, TanStack Router/Query, Tailwind v4, shadcn/ui |
| Client | `@hey-api/openapi-ts` from the Rust spec (`just gen`) |
| Tooling | mise, just, hk, fnox (age), pnpm, Biome, nextest, Playwright |
| Deploy | one image; `just deploy HOST` |

## Commands

| Command | Purpose |
|---|---|
| `just setup` | mise + JS deps + git hooks |
| `just db-up` | Postgres + Mailpit |
| `just dev` | API + Vite |
| `just migrate` / `just migrate-new NAME` | SQLx migrations |
| `just sqlx-prepare` | refresh `.sqlx/` |
| `just gen` | OpenAPI export + TS client |
| `just check` | fmt, clippy `-D warnings`, tsc, biome, sqlx, gen freshness |
| `just test` | nextest + Vitest |
| `just e2e` | Playwright smoke |
| `just ci` | check + test + e2e + docker-build |
| `just docker-build` | production image `rwk:latest` |
| `just deploy HOST` | build, `docker save \| ssh`, compose up |

Config names: `.env.example`. Secrets: `fnox.toml` (never print `fnox` values). Agent brief: `CLAUDE.md` (same text as `AGENTS.md`).

Optional MCP servers are listed in `.mcp.json` (not required to build or test). Disable or delete that file if you do not want the client to spawn them. Equivalents:

```bash
npx -y @ivotoby/openapi-mcp-server --openapi-spec apps/web/openapi.json --api-base-url http://localhost:8080/api
npx -y @modelcontextprotocol/server-postgres postgresql://rwk:rwk@localhost:5432/rwk
```

The Postgres server is intended for read-only schema/SQL against the local dev database.

## Deploy to a VM

```bash
just docker-build
# or, build + copy + compose on the host:
just deploy user@host
```

`just deploy` builds for `linux/amd64` by default, because most VMs are x86 while Apple Silicon builds arm64 natively. Pass a second argument for an arm VM: `just deploy user@host linux/arm64`.

Production compose is `docker-compose.prod.yml` (app + Postgres). Inject secrets with `fnox exec -- docker compose -f docker-compose.prod.yml up -d`. Required: `RWK_SESSION__SECRET` (not the development default).

The image listens on **8080**. On [exe.dev](https://exe.dev/docs/proxy.md), expose 8080 (`ssh exe.dev share port <vm> 8080`). Set `RWK_APP_URL` to the public `https://…` origin so OAuth redirects and email links match.

## License

MIT
