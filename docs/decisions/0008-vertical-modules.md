# 0008. Vertical modules

## Context

Agents navigate by feature name. Horizontal layers (`models/`, `handlers/` dumping every type together) force wide reads and invite cross-feature coupling.

## Decision

Each domain is a module: `crates/core/src/<feature>/{mod,model,repo,service}.rs`. HTTP stays in `crates/api/src/routes/<feature>.rs`. Shared kernel is config, db, `AppError`, telemetry, `AppState`. The projects slice (`docs/slices/projects.md`) is the pattern to copy.

## Consequences

A feature's types, SQL, and rules sit in one directory. New resources do not grow a central `models.rs`. Cross-cutting changes (error shape, sessions) still land in the kernel. Keep files small; split when a module stops fitting in one screen of intent.
