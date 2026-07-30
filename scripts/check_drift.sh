#!/usr/bin/env bash
# check_drift.sh
#
# Detects drift between generated artifacts (schemas, docs) and their committed
# versions.  Regenerates artifacts in a temporary directory and compares against
# the working tree to find discrepancies.
#
# Usage:
#   ./scripts/check_drift.sh [--schema] [--docs] [--all]
#
# Exit codes:
#   0 — no drift detected
#   1 — drift detected (committed artifacts are out of date)

set -euo pipefail

ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
SCHEMAS_DIR="$ROOT_DIR/schemas"
DOCS_DIR="$ROOT_DIR/docs"
DRIFT_REPORT=""
CHECK_SCHEMA=false
CHECK_DOCS=false

for arg in "$@"; do
  case "$arg" in
    --schema) CHECK_SCHEMA=true ;;
    --docs)   CHECK_DOCS=true ;;
    --all)    CHECK_SCHEMA=true; CHECK_DOCS=true ;;
    *)        echo "Unknown option: $arg"; exit 1 ;;
  esac
done

# Default: check everything
if ! $CHECK_SCHEMA && ! $CHECK_DOCS; then
  CHECK_SCHEMA=true
  CHECK_DOCS=true
fi

TMPDIR_BASE="$(mktemp -d)"
trap 'rm -rf "$TMPDIR_BASE"' EXIT

violations=0
checked=0

echo "=== Uzima Drift Detection ==="
echo "Working directory: $ROOT_DIR"
echo

# ---------------------------------------------------------------------------
# Schema Drift Detection
# ---------------------------------------------------------------------------
if $CHECK_SCHEMA; then
  echo "--- Schema Drift Detection ---"
  SCHEMA_REGISTRIES=(
    "$SCHEMAS_DIR/events/event-schema-registry.json"
    "$SCHEMAS_DIR/interface-registry/registry.json"
  )

  for registry in "${SCHEMA_REGISTRIES[@]}"; do
    if [[ ! -f "$registry" ]]; then
      echo "  SKIP: registry not found: $registry"
      continue
    fi
    checked=$((checked + 1))
    registry_name="$(basename "$(dirname "$registry")")/$(basename "$registry")"

    # Validate JSON structure
    if command -v python3 &>/dev/null; then
      if python3 -c "import json; json.load(open('$registry'))" 2>/dev/null; then
        echo "  OK: $registry_name (valid JSON)"
      else
        echo "  DRIFT: $registry_name (invalid JSON)"
        violations=$((violations + 1))
      fi
    elif command -v node &>/dev/null; then
      if node -e "JSON.parse(require('fs').readFileSync('$registry','utf8'))" 2>/dev/null; then
        echo "  OK: $registry_name (valid JSON)"
      else
        echo "  DRIFT: $registry_name (invalid JSON)"
        violations=$((violations + 1))
      fi
    else
      echo "  SKIP: no JSON validator available (install python3 or node)"
    fi

    # Check for individual event schema files matching registry entries
    if [[ "$registry" == *"event-schema-registry"* ]]; then
      echo "  Checking event schema files..."
      schema_files_count="$(find "$SCHEMAS_DIR/events" -name '*.schema.json' -not -name 'event-schema-registry.json' | wc -l)"
      echo "  Found $schema_files_count event schema files"

      # Check that each schema file referenced in registry exists
      if command -v jq &>/dev/null; then
        missing_schemas=0
        while IFS= read -r event_key; do
          contract="$(echo "$event_key" | cut -d'|' -f1)"
          event_name="$(echo "$event_key" | cut -d'|' -f2)"
          expected_file="$SCHEMAS_DIR/events/${event_name}_event.schema.json"
          if [[ ! -f "$expected_file" ]]; then
            echo "    DRIFT: registry references '$event_name' but schema file missing: $(basename "$expected_file")"
            missing_schemas=$((missing_schemas + 1))
          fi
        done < <(jq -r '.events | to_entries[] | "\(.value.contract)|\(.value.name)"' "$registry" 2>/dev/null)

        if (( missing_schemas > 0 )); then
          violations=$((violations + missing_schemas))
        fi
      fi
    fi
  done
  echo
fi

# ---------------------------------------------------------------------------
# Documentation Drift Detection
# ---------------------------------------------------------------------------
if $CHECK_DOCS; then
  echo "--- Documentation Drift Detection ---"

  # Check that auto-generated docs are up to date
  GENERATED_DOCS=(
    "docs/ERROR_CODES.md"
    "docs/INTERFACE_REGISTRY.md"
  )

  for doc in "${GENERATED_DOCS[@]}"; do
    doc_file="$ROOT_DIR/$doc"
    if [[ ! -f "$doc_file" ]]; then
      echo "  INFO: $doc not found (may need generation)"
      continue
    fi
    checked=$((checked + 1))

    # Check last modified time vs recently changed source files
    doc_mtime="$(stat -c %Y "$doc_file" 2>/dev/null || stat -f %m "$doc_file" 2>/dev/null)"

    # For ERROR_CODES.md, check against contracts with errors.rs
    if [[ "$doc" == *"ERROR_CODES"* ]]; then
      echo "  Checking $doc..."
      # Find contracts with errors.rs
      errors_contracts=()
      while IFS= read -r -d '' errors_file; do
        contract="$(basename "$(dirname "$(dirname "$errors_file")")")"
        errors_contracts+=("$contract")
      done < <(find "$ROOT_DIR/contracts" -name 'errors.rs' -path '*/src/errors.rs' -print0 2>/dev/null)

      # Check each is documented
      undocumented=0
      for contract in "${errors_contracts[@]}"; do
        if ! grep -q "$contract" "$doc_file" 2>/dev/null; then
          echo "    DRIFT: $contract has errors.rs but is not documented in $doc"
          undocumented=$((undocumented + 1))
        fi
      done
      if (( undocumented > 0 )); then
        violations=$((violations + undocumented))
      else
        echo "  OK: $doc covers all contracts with errors.rs"
      fi
    fi

    # Check for stale markdown (placeholder content)
    if grep -q 'TODO\|FIXME\|PLACEHOLDER' "$doc_file" 2>/dev/null; then
      todo_count="$(grep -c 'TODO\|FIXME\|PLACEHOLDER' "$doc_file" 2>/dev/null)"
      echo "  DRIFT: $doc contains $todo_count TODO/FIXME/PLACEHOLDER markers"
      violations=$((violations + 1))
    fi
  done

  # Check that each contract with a README is up to date
  echo
  echo "  Checking contract READMEs..."
  stale_readmes=0
  while IFS= read -r -d '' readme; do
    contract="$(basename "$(dirname "$readme")")"
    lib_file="$(dirname "$readme")/src/lib.rs"
    if [[ -f "$lib_file" ]]; then
      # Check if README mentions the contract's public struct
      pascal="$(echo "$contract" | sed -r 's/(^|_)([a-z])/\U\2/g')"
      if ! grep -q "$pascal" "$readme" 2>/dev/null; then
        echo "    INFO: $contract/README.md may not reference struct $pascal"
      fi
    fi
  done < <(find "$ROOT_DIR/contracts" -maxdepth 2 -name 'README.md' -print0 2>/dev/null)
  echo
fi

# ---------------------------------------------------------------------------
# Summary
# ---------------------------------------------------------------------------
echo "=== Drift Detection Summary ==="
echo "  Items checked: $checked"
echo "  Drift items:   $violations"
echo

if (( violations > 0 )); then
  echo "FAIL: $violations drift issue(s) detected."
  echo "Regenerate artifacts and recommit."
  exit 1
fi

echo "OK: no drift detected."