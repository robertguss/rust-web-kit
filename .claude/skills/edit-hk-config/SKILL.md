---
name: edit-hk-config
description: Edit hk git hooks (hk.pkl Pkl config). Use when changing pre-commit, pre-push, formatters, or linters.
---

# Edit hk config

hk is configured in **Pkl**, not TOML. The file is `hk.pkl` at the repo root. Schema comes from the hk release pinned in `mise.toml` (`hk = "1.58.1"`).

## What Pkl is here

`hk.pkl` is data: it amends hk's published `Config.pkl` and maps hook names to steps. It is not a general-purpose script. `amends "package://…/Config.pkl"` plus `import "…/Builtins.pkl"` pull the types.

Top-level:

- `hooks["pre-commit"]` — `fix = true`, `stash = "git"`, `steps = linters`
- `hooks["pre-push"]` — exclusive steps that shell out to `just check` then `just test`

`linters` is a `Mapping<String, Step>`:

- `cargo-fmt` = `Builtins.cargo_fmt`
- `biome` = `Builtins.biome` with `check` / `fix` overridden to `pnpm --dir apps/web exec biome …` because the builtin argv cannot take a `pnpm` prefix

`{{files}}` is hk's placeholder for the staged paths.

## Files to touch

- `hk.pkl` only, unless a new tool also needs a `just` recipe (`justfile`) or a mise pin (`mise.toml`)

## Rules

- Keep pre-commit **fast and fix-mode** (fmt). Keep pre-push **check-mode** (`just check`, `just test`).
- After edits: `hk validate` then `hk install`.
- Do not put secrets in `hk.pkl`.
- Do not replace `just check` / `just test` with a partial subset unless `justfile` changes too — those recipes are the CI vocabulary.

## Checklist

- [ ] Pkl still amends the same hk version as `mise.toml`
- [ ] `hk validate` succeeds
- [ ] `hk install` refreshed `.git/hooks`
- [ ] A trivial staged file still formats on commit locally
