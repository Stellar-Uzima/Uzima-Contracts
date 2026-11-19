#!/usr/bin/env bash
# Report (and optionally fail on) any deployable contract (has a
# #[contract] item, so it produces a WASM artifact) that's missing a
# scripts/wasm_size_baselines.json entry and isn't in that file's
# "excluded" list. Catches new contracts shipping without their baseline
# being added in the same PR, per #1576.
#
# Non-blocking by default: as of this writing 31 already-shipped contracts
# are missing a baseline (see docs/audits/WASM_BASELINE_STALENESS.md) —
# populating real sizes needs an actual release build, which isn't run
# here. Pass --strict once that backlog is cleared, so this only blocks
# *new* contracts from shipping without a baseline going forward.
#
# --strict-new=<git-ref> blocks only contracts whose Cargo.toml was added since
# <git-ref> (e.g. origin/main), so the existing backlog is grandfathered while
# newly added contracts must ship with a baseline. Requires full git history.
#
# Usage: ./scripts/check_wasm_baseline_staleness.sh [--strict | --strict-new=<git-ref>]

set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
CONTRACTS_DIR="$ROOT_DIR/contracts"
BASELINE_FILE="$ROOT_DIR/scripts/wasm_size_baselines.json"

STRICT=false
STRICT_NEW_REF=""
for arg in "$@"; do
  case "$arg" in
    --strict) STRICT=true ;;
    --strict-new=*) STRICT_NEW_REF="${arg#--strict-new=}" ;;
  esac
done

if [[ ! -f "$BASELINE_FILE" ]]; then
  echo "FATAL: baseline file not found: $BASELINE_FILE"
  exit 1
fi

# Pull the keys under "contracts" and the "excluded" list out of the JSON
# without a JSON parser dependency (jq may not be present everywhere this
# runs). Both are simple flat structures, so a line-anchored grep is safe.
baseline_entries="$(sed -n '/"contracts": {/,/^  }/p' "$BASELINE_FILE" \
  | grep -oE '"[a-zA-Z0-9_-]+": [0-9]+' \
  | sed -E 's/^"([^"]+)".*/\1/' | sort -u)"
excluded_entries="$(sed -n '/"excluded": \[/,/\]/p' "$BASELINE_FILE" \
  | grep -oE '"[a-zA-Z0-9_-]+"' | tr -d '"' | sort -u)"

missing=0
new_missing=0

if [[ -n "$STRICT_NEW_REF" ]]; then
  added_manifests="$(git -C "$ROOT_DIR" diff --diff-filter=A --name-only "$STRICT_NEW_REF"...HEAD -- 'contracts/*/Cargo.toml')" || {
    echo "FATAL: could not diff against $STRICT_NEW_REF"
    exit 1
  }
fi
for dir in "$CONTRACTS_DIR"/*/; do
  name="$(basename "$dir")"
  [[ -f "$dir/Cargo.toml" ]] || continue

  # Only contracts that actually produce a WASM artifact need a baseline —
  # shared library crates (no #[contract] item) are exempt.
  grep -rq '#\[contract\]' "$dir/src" 2>/dev/null || continue

  if grep -qxF "$name" <<<"$baseline_entries"; then
    continue
  fi
  if grep -qxF "$name" <<<"$excluded_entries"; then
    continue
  fi

  missing=$((missing + 1))
  if [[ -n "$STRICT_NEW_REF" ]] && grep -qxF "contracts/$name/Cargo.toml" <<<"$added_manifests"; then
    new_missing=$((new_missing + 1))
    echo "NEW CONTRACT MISSING BASELINE: $name"
  fi
  echo "MISSING BASELINE: $name (deployable contract, no wasm_size_baselines.json entry and not excluded)"
done

echo ""
if [[ "$missing" -gt 0 ]]; then
  echo "$missing deployable contract(s) missing a wasm_size_baselines.json entry."
  echo "Add an entry under \"contracts\" (measure with scripts/measure_storage.sh or"
  echo "scripts/wasm_size_monitor.sh after a release build) or add the contract name"
  echo "to \"excluded\" if it's intentionally not size-gated."
  if $STRICT; then
    exit 1
  fi
  if [[ "$new_missing" -gt 0 ]]; then
    echo "ERROR: $new_missing newly added contract(s) have no baseline (existing backlog is grandfathered)."
    exit 1
  fi
  exit 0
fi

echo "OK: every deployable contract has a baseline entry or is excluded."
exit 0
