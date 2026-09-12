# Phase 1 report — Scaffold and toolchain

Status: complete. All Phase 1 acceptance items were run on this machine.

## What was built

- Cargo workspace, edition 2024, resolver 3:
  - `crates/core` (`rwk-core`, lib)
  - `crates/api` (`rwk-api`, bin)
  - Workspace dependency table (current crates.io versions, 2026-09-12)
  - `[workspace.lints.clippy]` enables `pedantic` at warn (so `clippy -- -D warnings` denies it) with a short allow-list: `missing_errors_doc`, `missing_panics_doc`, `module_name_repetitions`, `must_use_candidate`
- `rwk-api` binds `0.0.0.0:8080`, initializes tracing, serves `GET /api/health` → `{"status":"ok"}`
- `apps/web`: Vite React-TS, package name `rwk`, Biome 2.5.13, TypeScript strict, Tailwind v4 (`@tailwindcss/vite`), shadcn/ui (radix-nova) with button, input, card, label, textarea, field/separator (form pieces). Vite proxies `/api` → `:8080`
- `mise.toml` pins: rust `stable`, node `lts`, pnpm `12.4.1`, just `1.58.0`, hk `1.58.1`, fnox `1.35.1`, age `1.3.2`, sqlx-cli `0.9.0` (rustls, no default OpenSSL), cargo-nextest `0.9.144`, bacon `3.25.0`
- `justfile` recipes listed in PLAN.md. Later-phase recipes print `TODO`
- `hk.pkl`: pre-commit fix = cargo fmt + biome; pre-push = `just check` then `just test`
- `fnox.toml` age provider (placeholder recipient, no plaintext secrets)
- `.env.example` lists `RWK_*` variable names only
- `docker-compose.yml` project `rwk`: postgres:17 on 5432 (db/user/password `rwk`), mailpit on 8025/1025
- `.gitignore`, MIT `LICENSE`, `rust-toolchain.toml`, `.editorconfig`, short `README.md`

Crate versions from `cargo search` (2026-09-12): axum 0.8.9, tokio 1.53.1, tracing 0.1.44, tracing-subscriber 0.3.23, serde 1.0.229, serde_json 1.0.151.

## Verification

Commands run from the repo root with project tools on `PATH` (`mise` shims). Toolchain after `mise install`: rustc 1.98.1, cargo 1.98.1, node v24.20.0 (LTS), pnpm 12.4.1.

### `mise install`

```
mise all tools are installed
```

Exit 0. First attempt failed on `cargo:sqlx-cli@0.9.0` because default features pull `openssl-sys` and this VM has no OpenSSL headers. Fixed by pinning sqlx-cli with `default-features = false` and `features = ["postgres", "rustls"]`. Re-run succeeded.

### `just db-up`

```
docker compose up -d
 Container rwk-postgres-1  Running
 Container rwk-mailpit-1  Running
/var/run/postgresql:5432 - accepting connections
```

`docker compose ps`: `rwk-postgres-1` postgres:17 healthy on 5432; `rwk-mailpit-1` healthy on 8025/1025. `curl -fsS -o /dev/null -w '%{http_code}\n' http://localhost:8025/` → `200`.

`just db-up` waits up to 30s for `pg_isready` (Postgres is not ready on the first tick after `up -d`).

### `cargo build`

```
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.03s
```

Exit 0, no warnings.

### `cargo clippy --all-targets -- -D warnings`

```
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.05s
```

Exit 0.

### `pnpm -C apps/web typecheck && pnpm -C apps/web lint`

```
$ tsc -b --noEmit
$ biome check .
Checked 20 files in 15ms. No fixes applied.
```

Exit 0.

### `curl localhost:8080/api/health`

After `cargo run -p rwk-api` (listening on `0.0.0.0:8080`):

```
HTTP/1.1 200 OK
content-type: application/json
content-length: 15

{"status":"ok"}
```

The API process was stopped after this check.

### Extra (not in the acceptance list)

- `just check` green (fmt check, clippy -D warnings, typecheck, lint)
- `just test-api` / `cargo nextest run --workspace --no-tests=pass` green (0 tests; `--no-tests=pass` required because nextest 0.9.144 exits 4 when there are no tests)
- `hk validate` → `hk.pkl is valid`

## Incomplete / notes for later phases

- `just` recipes still stubs (print TODO): `dev`, `migrate`, `migrate-new`, `sqlx-prepare`, `gen`, `test-web`, `e2e`, `ci`, `docker-build`, `deploy`, `init`
- No domain tests yet (Phase 2+)
- Current shadcn registry item `@shadcn/form` emits no files; form pieces are `field` + `label` + `input` + `textarea` + `button` plus `react-hook-form` / `zod` / `@hookform/resolvers`
- `fnox.toml` recipient is a template placeholder public key; there is no matching identity in this repo. Replace before `fnox set`
- `mise.toml` must be trusted once (`mise trust`) on a new clone
- Biome step in `hk.pkl` overrides the builtin to `pnpm --dir apps/web exec biome ...` because the builtin uses structured argv and cannot take a shell prefix
