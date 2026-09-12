# Recipes are the agent-facing command vocabulary; CLAUDE.md documents them.
set dotenv-load := false
set shell := ["bash", "-eu", "-o", "pipefail", "-c"]

# Host port for the dev Postgres. Override when 5432 is already taken:
#   RWK_DB_PORT=5433 just db-up
# Export it in your shell so every recipe and the app agree.
export RWK_DB_PORT := env("RWK_DB_PORT", "5432")

# 127.0.0.1, not localhost: localhost resolves to ::1 first on macOS, which
# can reach a different Postgres than the one Docker published.
export DATABASE_URL := env("DATABASE_URL", "postgres://rwk:rwk@127.0.0.1:" + RWK_DB_PORT + "/rwk")

# The app reads RWK_DATABASE__URL; keep it in step with DATABASE_URL.
export RWK_DATABASE__URL := DATABASE_URL

# List recipes.
default:
    @just --list

# Install tools, JS deps, and git hooks.
setup:
    mise install
    pnpm --dir apps/web install
    hk install

# Start API + Vite together.
dev:
    #!/usr/bin/env bash
    set -euo pipefail
    trap 'kill 0' EXIT
    just dev-api &
    just dev-web &
    wait

# Run the API on :8080.
dev-api:
    cargo run -p rwk-api

# Run the Vite dev server on :5173.
dev-web:
    pnpm --dir apps/web dev

# Start Postgres and Mailpit.
db-up:
    #!/usr/bin/env bash
    set -euo pipefail
    docker compose up -d
    # A first run initializes the cluster, which is slow on Docker Desktop.
    # initdb starts a temporary server and restarts it, so pg_isready flips
    # between accepting and rejecting. Probe once per tick and require two
    # consecutive successes, so a transient reject does not end the wait.
    ok=0
    ready=0
    out=""
    for _ in $(seq 1 90); do
      if out=$(docker compose exec -T postgres pg_isready -U rwk -d rwk 2>&1); then
        ok=$((ok + 1))
        if [ "$ok" -ge 2 ]; then
          ready=1
          break
        fi
      else
        ok=0
      fi
      sleep 1
    done
    if [ "$ready" != 1 ]; then
      echo "postgres did not become ready; last 40 log lines:" >&2
      docker compose logs --tail 40 postgres >&2
      echo >&2
      echo "If this volume was created by Postgres 17, remove it: docker compose down -v" >&2
      exit 1
    fi
    echo "$out"
    # pg_isready only proves the container answers on its own socket. Confirm
    # DATABASE_URL actually reaches it, because another Postgres may already
    # own the published port and will answer instead.
    if ! sqlx migrate info --source migrations >/dev/null 2>&1; then
      echo >&2
      echo "The container is healthy, but $DATABASE_URL does not reach it." >&2
      probed=${DATABASE_URL##*:}; probed=${probed%%/*}
      echo "Another Postgres probably owns port $probed:" >&2
      lsof -nP -iTCP:"$probed" -sTCP:LISTEN 2>/dev/null >&2 || true
      echo >&2
      echo "Pick a free port and export it, then re-run:" >&2
      echo "  export RWK_DB_PORT=5433 && just db-up" >&2
      exit 1
    fi
    # Apply migrations here so a fresh clone can run `just check` and
    # `just ci` without starting the app first.
    just migrate

# Stop local data services.
db-down:
    docker compose down

# Apply SQLx migrations.
migrate:
    sqlx migrate run --source migrations

# Create a new SQLx migration.
migrate-new NAME:
    sqlx migrate add --source migrations --simple {{NAME}}

# Refresh the committed .sqlx offline cache.
sqlx-prepare:
    cargo sqlx prepare --workspace -- --all-targets

# Export OpenAPI and generate the TS client.
gen:
    cargo run -p rwk-api -- --export-openapi apps/web/openapi.json
    pnpm --dir apps/web exec openapi-ts

# Fail if `just gen` would change the committed-on-disk client (git-independent).
gen-check:
    #!/usr/bin/env bash
    set -euo pipefail
    hash_client() {
      find apps/web/openapi.json apps/web/src/api/generated -type f | sort | xargs sha256sum | sha256sum
    }
    before=$(hash_client)
    just gen
    after=$(hash_client)
    if [ "$before" != "$after" ]; then
      echo "generated OpenAPI client is stale." >&2
      echo "apps/web/openapi.json and apps/web/src/api/generated changed after 'just gen'." >&2
      echo "Run 'just gen' and include those files." >&2
      exit 1
    fi

# Format/lint/typecheck everything that exists in this phase.
check:
    cargo fmt --all -- --check
    # Offline, so a missing or unmigrated database cannot bury the real
    # errors under one macro failure per query. A query with no cached entry
    # says so plainly: run `just sqlx-prepare`.
    SQLX_OFFLINE=true cargo clippy --all-targets -- -D warnings
    pnpm --dir apps/web typecheck
    pnpm --dir apps/web lint
    cargo sqlx prepare --check --workspace -- --all-targets
    just gen-check

# Run API and web tests.
test: test-api test-web

# Rust tests.
test-api:
    cargo nextest run --workspace

# Frontend unit tests.
test-web:
    pnpm --dir apps/web test

# Playwright smoke against the API + built SPA (Postgres + Mailpit required).
e2e:
    #!/usr/bin/env bash
    set -euo pipefail
    if ! curl -fsS --max-time 1 http://127.0.0.1:8025/api/v1/info >/dev/null 2>&1; then
      just db-up
    fi
    pnpm --dir apps/web build
    if [ -n "${CI:-}" ]; then
      pnpm --dir apps/web exec playwright install --with-deps chromium
    else
      pnpm --dir apps/web exec playwright install chromium
    fi
    cargo build -p rwk-api
    started_api=0
    if ! curl -fsS --max-time 1 http://127.0.0.1:8080/api/health >/dev/null 2>&1; then
      ./target/debug/rwk-api &
      api_pid=$!
      started_api=1
      trap 'if [ "$started_api" = 1 ]; then kill "$api_pid" 2>/dev/null || true; fi' EXIT
      for _ in $(seq 1 60); do
        if curl -fsS --max-time 1 http://127.0.0.1:8080/api/health >/dev/null 2>&1; then
          break
        fi
        if ! kill -0 "$api_pid" 2>/dev/null; then
          echo "rwk-api exited before becoming healthy"
          exit 1
        fi
        sleep 0.5
      done
      curl -fsS --max-time 1 http://127.0.0.1:8080/api/health >/dev/null
    fi
    pnpm --dir apps/web exec playwright test

# Full CI pipeline.
ci:
    just check
    just test
    just e2e
    just docker-build

# Release-ish local build.
build:
    cargo build --release -p rwk-api
    pnpm --dir apps/web build

# Build the production image for the local architecture.
docker-build:
    DOCKER_BUILDKIT=1 docker build -t rwk:latest .

# Deploy the image to a VM over SSH. PLATFORM must match the VM's
# architecture, not your laptop's (Apple Silicon builds arm64 by default).
deploy HOST PLATFORM="linux/amd64":
    #!/usr/bin/env bash
    set -euo pipefail
    DOCKER_BUILDKIT=1 docker build --platform {{PLATFORM}} -t rwk:latest .
    docker save rwk:latest | ssh {{HOST}} docker load
    ssh {{HOST}} mkdir -p rwk
    scp docker-compose.prod.yml fnox.toml {{HOST}}:rwk/
    ssh {{HOST}} 'cd rwk && fnox exec -- docker compose -f docker-compose.prod.yml up -d'

# Rename the placeholder `rwk` project. Deletes this recipe.
init NAME:
    ./scripts/init.sh {{NAME}}

# Remove build artifacts.
clean:
    cargo clean
    rm -rf apps/web/dist apps/web/node_modules
