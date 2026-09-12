# 0006. Local CI via hk

## Context

Agents and humans both skip slow cloud CI. Formatting and tests need to fail on the laptop the same way they fail in GitHub Actions.

## Decision

hk (Pkl config `hk.pkl`): pre-commit runs cargo fmt + biome in fix mode; pre-push runs `just check` then `just test`. GitHub Actions is a thin `just ci` wrapper (check + test + e2e + docker-build) with Postgres and Mailpit services.

## Consequences

Broken fmt/clippy/tests never reach origin if hooks are installed (`just setup` / `hk install`). `hk.pkl` is data (Pkl), not a plugin script — see the `edit-hk-config` skill. Pre-push is slower than pre-commit on purpose.
