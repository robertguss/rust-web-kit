#!/usr/bin/env bash
# Rename the rwk placeholder to NAME, then drop this script.
# Usage: ./scripts/init.sh NAME   (or: just init NAME)
set -euo pipefail

usage() {
  echo "usage: $0 NAME" >&2
  echo "NAME must match [a-z][a-z0-9-]*" >&2
  exit 1
}

[ "${1:-}" = "-h" ] || [ "${1:-}" = "--help" ] && usage
[ "$#" -eq 1 ] || usage

NAME=$1
printf '%s' "$NAME" | grep -Eq '^[a-z][a-z0-9-]*$' || usage

if [ "$NAME" = "rwk" ]; then
  echo "NAME is already rwk; nothing to rename." >&2
  exit 1
fi

ROOT=$(CDPATH= cd -- "$(dirname "$0")/.." && pwd)
cd "$ROOT"

NAME_SNAKE=${NAME//-/_}
NAME_UPPER=$(printf '%s' "$NAME_SNAKE" | tr '[:lower:]' '[:upper:]')

skip_path() {
  case "$1" in
    ./.git|.git|.git/*|./.git/*) return 0 ;;
    */.git/*) return 0 ;;
    ./target|./target/*|*/target|*/target/*) return 0 ;;
    */node_modules|*/node_modules/*) return 0 ;;
    ./scripts/init.sh) return 0 ;;
  esac
  return 1
}

rewrite_file() {
  file=$1
  python3 - "$file" "$NAME" "$NAME_SNAKE" "$NAME_UPPER" <<'PY'
import pathlib, sys
path = pathlib.Path(sys.argv[1])
name, snake, upper = sys.argv[2], sys.argv[3], sys.argv[4]
try:
    text = path.read_text(encoding="utf-8")
except (UnicodeDecodeError, OSError):
    sys.exit(0)
new = text.replace("rwk_", snake + "_").replace("rwk", name).replace("RWK", upper)
if new != text:
    path.write_text(new, encoding="utf-8")
PY
}

# Drop build artifacts so leftover binaries cannot keep the old token.
rm -rf target apps/web/node_modules apps/web/dist apps/web/test-results apps/web/playwright-report

# File contents (text only). Skip .git, node_modules, target, this script.
find . -print0 | while IFS= read -r -d '' path; do
  skip_path "$path" && continue
  [ -f "$path" ] || continue
  rewrite_file "$path"
done

# File and directory names, deepest first.
find . -depth -print0 | while IFS= read -r -d '' path; do
  skip_path "$path" && continue
  base=$(basename "$path")
  case "$base" in
    *rwk*)
      dir=$(dirname "$path")
      new_base=$(printf '%s' "$base" | python3 -c "import sys; n=sys.argv[1]; s=sys.argv[2]; b=sys.stdin.read(); print(b.replace('rwk_', s+'_').replace('rwk', n), end='')" "$NAME" "$NAME_SNAKE")
      if [ "$new_base" != "$base" ]; then
        mv "$path" "$dir/$new_base"
      fi
      ;;
  esac
done

# README title is the first heading, independent of the rust-web-kit token.
if [ -f README.md ]; then
  python3 - "$NAME" <<'PY'
import pathlib, sys
name = sys.argv[1]
path = pathlib.Path("README.md")
lines = path.read_text(encoding="utf-8").splitlines(keepends=True)
if not lines:
    path.write_text(f"# {name}\n", encoding="utf-8")
else:
    if lines[0].lstrip().startswith("#"):
        nl = "\n" if lines[0].endswith("\n") else ""
        lines[0] = f"# {name}{nl}"
    else:
        lines.insert(0, f"# {name}\n")
    path.write_text("".join(lines), encoding="utf-8")
PY
fi

# Remove the init recipe (and a comment immediately above it).
if [ -f justfile ]; then
  python3 <<'PY'
from pathlib import Path
path = Path("justfile")
lines = path.read_text(encoding="utf-8").splitlines(keepends=True)
out = []
i = 0
while i < len(lines):
    raw = lines[i]
    if raw.startswith("init NAME:"):
        while out and out[-1].lstrip().startswith("#"):
            out.pop()
        i += 1
        while i < len(lines) and (lines[i].startswith(" ") or lines[i].startswith("\t")):
            i += 1
        if i < len(lines) and lines[i].strip() == "":
            i += 1
        continue
    out.append(raw)
    i += 1
path.write_text("".join(out), encoding="utf-8")
PY
fi

rm -f scripts/init.sh
rmdir scripts 2>/dev/null || true

rm -rf .git
git init

mise trust
mise install
eval "$(mise hook-env -s bash)"
pnpm --dir apps/web install
hk install

if [ -n "${DATABASE_URL:-}" ]; then
  db_url=$DATABASE_URL
else
  db_url=$(just --evaluate DATABASE_URL 2>/dev/null || true)
fi
if [ -z "${db_url:-}" ]; then
  db_url="postgres://${NAME}:${NAME}@localhost:5432/${NAME}"
fi

if command -v pg_isready >/dev/null 2>&1 && pg_isready -d "$db_url" >/dev/null 2>&1; then
  echo "DATABASE_URL reachable; running just sqlx-prepare"
  DATABASE_URL=$db_url just sqlx-prepare
elif command -v psql >/dev/null 2>&1 && psql "$db_url" -c 'SELECT 1' >/dev/null 2>&1; then
  echo "DATABASE_URL reachable; running just sqlx-prepare"
  DATABASE_URL=$db_url just sqlx-prepare
else
  echo "note: DATABASE_URL is not reachable; skip sqlx-prepare."
  echo "After Postgres is up (just db-up), run: just sqlx-prepare"
fi

echo "Initialized project as ${NAME}."
echo "Next: just setup (if you still need it), just db-up, just dev"
