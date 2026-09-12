# Phase 8 report — AI-first scaffolding and docs

Status: complete. All Phase 8 acceptance items were run on this machine. Nothing was committed.

`CLAUDE.md` is the primary agent brief. `AGENTS.md` is **identical content** (not a one-line pointer).

## What was built

### Agent docs

- `CLAUDE.md` / `AGENTS.md`: architecture map, `just` vocabulary, conventions (vertical slices, `AppError`, garde, utoipa, generated client, never edit `src/api/generated/`), `just check` before done / `just ci` before a PR, secret rules (never print fnox values), 8-step feature path, optional MCP.
- `README.md`: what it is, template → `just init` → `just setup` → `just dev`, stack table, commands, VM deploy, GitHub “Template repository” note (owner sets that checkbox; it is not in git).
- `docs/slices/projects.md`, `docs/slices/auth.md`.
- ADRs in `docs/decisions/`:
  - `0001-sqlx.md`
  - `0002-openapi-from-rust.md`
  - `0003-postgres-jobs.md`
  - `0004-sessions-over-jwt.md`
  - `0005-single-image.md`
  - `0006-local-ci-via-hk.md`
  - `0007-fnox.md`
  - `0008-vertical-modules.md`

### Claude Code

- `.claude/settings.json`: `PostToolUse` matcher `Edit|Write`. Command reads JSON on stdin with `jq` (`.tool_input.file_path`) and runs `.claude/hooks/post-tool-use.sh`.
  - `.rs` → `cargo fmt -- <file>` then `cargo check -p rwk-core|rwk-api --quiet --offline` (crate from path).
  - `.ts`/`.tsx` under `apps/web` → `pnpm --dir apps/web exec biome check --write --no-errors-on-unmatched <rel>`.
  - Other files: exit 0. Timeout 15s. Hook exit 0 on success / irrelevant path.
- Skills (frontmatter `name` + `description`):
  - `.claude/skills/add-endpoint/SKILL.md`
  - `.claude/skills/add-migration/SKILL.md`
  - `.claude/skills/add-page/SKILL.md`
  - `.claude/skills/add-job/SKILL.md`
  - `.claude/skills/edit-hk-config/SKILL.md` (Pkl: `hk.pkl` amends hk Config, builtins, `{{files}}`)

### MCP (optional)

`.mcp.json` (stdio via `npx -y`; not required to build/test):

- OpenAPI: `@ivotoby/openapi-mcp-server` → `apps/web/openapi.json`, base `http://localhost:8080/api`
- Postgres: `@modelcontextprotocol/server-postgres` `postgresql://rwk:rwk@localhost:5432/rwk` (read-only transaction; local published creds)

Documented in README and `CLAUDE.md`. Delete or disable the file if the client should not spawn them.

### `just init NAME`

`scripts/init.sh` (POSIX-ish bash, executable):

- `NAME` must match `[a-z][a-z0-9-]*`
- Replaces `rwk_` → snake-case `NAME_`, remaining `rwk` → `NAME`, `RWK` → upper snake (so `rwk-api` / `rwk_api` / `RWK_` stay valid when `NAME` has hyphens)
- Rewrites file contents and file/dir names; skips `.git`, `node_modules`, `target`, itself
- README first heading → `# NAME`
- Deletes `scripts/init.sh`, empty `scripts/`, and the `init` just recipe
- `rm -rf .git && git init`
- `mise trust && mise install`, `pnpm --dir apps/web install`, `hk install`
- `just sqlx-prepare` if `DATABASE_URL` is reachable, else a note to run it after `just db-up`

`justfile` `init NAME:` calls `./scripts/init.sh {{NAME}}`.

## Verification

Postgres + Mailpit were already up (`rwk-postgres-1`, `rwk-mailpit-1`). Port 8080 was free.

### Hook smoke

```
echo '{"tool_name":"Edit","tool_input":{"file_path":"crates/core/src/lib.rs"}}' | bash -c "$(jq -r '.hooks.PostToolUse[0].hooks[0].command' .claude/settings.json)"
# rs_exit=0

echo '{"tool_name":"Write","tool_input":{"file_path":"apps/web/src/lib/utils.ts"}}' | bash -c "$(jq -r '.hooks.PostToolUse[0].hooks[0].command' .claude/settings.json)"
# Checked 1 file in 4ms. No fixes applied.  ts_exit=0
```

### `just init demo` on a copy

Commands:

```bash
DEST=/tmp/rwk-init-demo
rm -rf "$DEST"
mkdir -p "$DEST"
rsync -a \
  --exclude target --exclude node_modules --exclude dist \
  --exclude test-results --exclude playwright-report --exclude .git \
  /home/exedev/Projects/rust-web-kit/ "$DEST/"
cd "$DEST"
./scripts/init.sh demo
# same as: just init demo
grep -r --exclude-dir=.git --exclude-dir=node_modules --exclude-dir=target --exclude-dir=dist rwk .
SQLX_OFFLINE=true cargo build
pnpm --dir apps/web build
```

`./scripts/init.sh demo` output (exit 0):

```
Initialized empty Git repository in /tmp/rwk-init-demo/.git/
mise trusted /tmp/rwk-init-demo
mise all tools are installed
✓ Lockfile passes supply-chain policies (verified 28m ago)
Lockfile is up to date, resolution step is skipped
Packages are hard linked from the content-addressable store to the virtual store.
  Content-addressable store is at: /home/exedev/.local/share/pnpm/store/v11
  Virtual store is at:             node_modules/.pnpm
Packages: +560
Progress: resolved 560, reused 560, downloaded 0, added 560, done
…
Done in 425ms using pnpm v12.4.1
hk Installed hk hook: /tmp/rwk-init-demo/.git/hooks/pre-commit
hk Installed hk hook: /tmp/rwk-init-demo/.git/hooks/pre-push
note: DATABASE_URL is not reachable; skip sqlx-prepare.
After Postgres is up (just db-up), run: just sqlx-prepare
Initialized project as demo.
Next: just setup (if you still need it), just db-up, just dev
```

After init:

| Check | Result |
|---|---|
| `grep -r rwk` (exclude `.git` / `node_modules` / `target` / `dist`) | no matches |
| `scripts/init.sh` | removed; `scripts/` gone |
| `init` recipe in justfile | removed |
| README title | `# demo` |
| crate / package names | `demo-core`, `demo-api` / `demo_api`, web `demo` |
| `SQLX_OFFLINE=true cargo build` | `Finished dev profile … in 1m 48s` (compiles `demo-core`, `demo-api`) |
| `pnpm --dir apps/web build` | Vite production build, `✓ built in 756ms` |

sqlx-prepare was skipped: after rename the default URL is `postgres://demo:demo@localhost:5432/demo`, while this machine’s compose is still `rwk`. Offline `.sqlx/` was enough for `cargo build`.

### `just ci`

Exit 0 (48s; docker layers cached):

| Step | Result |
|---|---|
| `just check` | fmt, clippy `-D warnings`, tsc, biome 60 files, sqlx `--check`, `just gen-check` |
| `just test` | 54 nextest passed, 0 skipped; Vitest 7 passed |
| `just e2e` | Playwright chromium smoke passed (3.2s) |
| `just docker-build` | `rwk:latest` |

## Incomplete / notes for the reviewer

- GitHub “Template repository” must be enabled by the owner in the GitHub UI.
- MCP servers are optional; the official `@modelcontextprotocol/server-postgres` package is archived but is the widely documented npx read-only example. Use only against the local published `rwk`/`rwk` database.
- `just init` runs `mise trust` so a copy outside the original trust set can `mise install`.
- Do not commit from this agent.
