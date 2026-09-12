# rust-web-kit

AI-first full-stack starter: Rust (Axum) API + React (Vite) frontend, shipped as a single Docker image.

The placeholder project name is `rwk` (crates, npm package, database, compose project). Rename it with `just init NAME` once Phase 8 lands.

## Quickstart

```bash
mise install
just setup
just db-up
just dev
```

Health check: `curl localhost:8080/api/health` → `{"status":"ok"}`.

`just dev` runs the API on port 8080 and Vite on port 5173 (proxies `/api`). After `pnpm --dir apps/web build`, the API also serves the SPA from `apps/web/dist`.

Outbound mail goes through SMTP (`RWK_MAIL__SMTP_HOST` / `RWK_MAIL__SMTP_PORT`, default Mailpit on `localhost:1025`). The Mailpit UI is at [http://localhost:8025](http://localhost:8025). The API process runs the job worker by default; use `--api-only` or `--worker-only` to split them.

See `docs/PLAN.md` for the full stack and phase plan.
