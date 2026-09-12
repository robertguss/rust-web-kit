#!/usr/bin/env bash
# Fast PostToolUse formatter/check. Path comes from the Claude hook JSON
# (jq in .claude/settings.json) as $1. Always exit 0 when the path is
# irrelevant; propagate fmt/check status otherwise.
set -euo pipefail

file=${1:-}
[ -n "$file" ] || exit 0

root=$(git rev-parse --show-toplevel 2>/dev/null || pwd)
cd "$root"

# Accept repo-relative or absolute paths.
case "$file" in
  /*) ;;
  *) file="$root/$file" ;;
esac
[ -f "$file" ] || exit 0

case "$file" in
  *.rs)
    cargo fmt -- "$file"
    crate=
    case "$file" in
      */crates/core/*) crate=rwk-core ;;
      */crates/api/*) crate=rwk-api ;;
    esac
    [ -n "$crate" ] || exit 0
    cargo check -p "$crate" --quiet --offline
    ;;
  *.ts|*.tsx)
    rel=${file#"$root"/}
    case "$rel" in
      apps/web/*) web_rel=${rel#apps/web/} ;;
      *) exit 0 ;;
    esac
    pnpm --dir apps/web exec biome check --write --no-errors-on-unmatched "$web_rel"
    ;;
  *) exit 0 ;;
esac
