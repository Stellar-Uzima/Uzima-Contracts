#!/usr/bin/env bash
# check_sdk_version.sh — CI guard that fails if any workspace-member crate
# overrides the soroban-sdk pin instead of inheriting from the workspace.
#
# Exit codes:
#   0 — all workspace-member crates use workspace = true
#   1 — one or more workspace-member crates override the workspace pin
#
# Non-member crates (excluded/deferred) are scanned and reported as warnings
# but do not cause a CI failure.
#
# Usage (CI or local):
#   ./scripts/check_sdk_version.sh

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
CARGO_TOML="$REPO_ROOT/Cargo.toml"

if [[ ! -f "$CARGO_TOML" ]]; then
  echo "ERROR: workspace Cargo.toml not found at $CARGO_TOML" >&2
  exit 1
fi

# Extract workspace pin version (informational).
EXPECTED=$(sed -n '/^\[workspace\.dependencies\]/,/^\[/p' "$CARGO_TOML" \
  | grep '^\s*soroban-sdk' \
  | sed -E 's/.*version\s*=\s*"([^"]+)".*/\1/; t; s/.*=\s*"([^"]+)".*/\1/' \
  | head -1)

echo "Workspace soroban-sdk pin: ${EXPECTED:-unknown}"
echo ""

# Build list of workspace member Cargo.toml paths.
# The workspace root lists members = ["contracts/*", ...] and exclude = [...].
# We parse both to get the actual active member set.
MEMBER_DIRS=()
while IFS= read -r line; do
  # Extract path entries from members = [...] or exclude = [...]
  path=$(echo "$line" | sed -E 's/.*"([^"]+)".*/\1/' | xargs)
  [[ -z "$path" ]] && continue
  MEMBER_DIRS+=("$path")
done < <(sed -n '/^members\s*=/,/^\]/p; /^exclude\s*=/,/^\]/p' "$CARGO_TOML" \
  | grep '"' | sed 's/^[[:space:]]*//')

EXCLUDE_DIRS=()
while IFS= read -r line; do
  path=$(echo "$line" | sed -E 's/.*"([^"]+)".*/\1/' | xargs)
  [[ -z "$path" ]] && continue
  EXCLUDE_DIRS+=("$path")
done < <(sed -n '/^exclude\s*=/,/^\]/p' "$CARGO_TOML" \
  | grep '"' | sed 's/^[[:space:]]*//')

# Check if a path is an active member (not excluded).
is_active_member() {
  local f="$1"
  for excl in "${EXCLUDE_DIRS[@]}"; do
    case "$f" in
      "$excl"/*|"$excl") return 1 ;;
    esac
  done
  for mem in "${MEMBER_DIRS[@]}"; do
    # Expand glob for contracts/*
    case "$mem" in
      */\*)
        dir="${mem%/*}"
        [[ -f "$REPO_ROOT/$dir" ]] && return 0
        ;;
      *)
        [[ "$f" == "$mem"/* || "$f" == "$mem" ]] && return 0
        ;;
    esac
  done
  return 1
}

DRIFT=0
WARNINGS=0

while IFS= read -r member_toml; do
  [[ "$member_toml" == "$CARGO_TOML" ]] && continue
  rel="${member_toml#$REPO_ROOT/}"

  # Find soroban-sdk lines that are NOT workspace-inherited.
  bad_lines=$(grep -Pn '^\s*soroban-sdk\b' "$member_toml" 2>/dev/null \
    | grep -v '\.workspace\s*=\s*true' \
    | grep -v 'workspace\s*=\s*true' \
    || true)

  [[ -z "$bad_lines" ]] && continue

  if is_active_member "$rel"; then
    echo "DRIFT (member)  $rel"
    echo "$bad_lines" | sed 's/^/        /'
    DRIFT=1
  else
    echo "WARN (excluded) $rel"
    echo "$bad_lines" | sed 's/^/        /'
    WARNINGS=$((WARNINGS + 1))
  fi
done < <(find "$REPO_ROOT" -name 'Cargo.toml' \
  -not -path '*/target/*' \
  -not -path '*/node_modules/*')

echo ""

if [[ "$DRIFT" -ne 0 ]]; then
  echo "❌ Drift detected in active workspace member crates."
  echo "   All member Cargo.toml files must use: soroban-sdk = { workspace = true }"
  exit 1
fi

if [[ "$WARNINGS" -gt 0 ]]; then
  echo "⚠  $WARNINGS excluded/deferred crate(s) have hardcoded soroban-sdk versions."
  echo "   These do not block CI but should be cleaned up (see issue #828)."
fi

echo "✅ All active workspace member crates inherit soroban-sdk from the workspace root."
exit 0
