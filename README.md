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

OAuth (Google / GitHub) is enabled when both `RWK_OAUTH__<PROVIDER>_CLIENT_ID` and `_CLIENT_SECRET` are set. Playwright smoke: `just e2e` (Postgres + Mailpit). CI: `just ci`.

Production image: `just docker-build` then `RWK_SESSION__SECRET=… docker compose -f docker-compose.prod.yml up -d`. The app listens on 8080. On exe.dev, `EXPOSE 8080` is what the HTTPS proxy should target (`ssh exe.dev share port <vm> 8080`); see https://exe.dev/docs/proxy.md. Deploy to a VM: `just deploy HOST`.

See `docs/PLAN.md` for the full stack and phase plan.
