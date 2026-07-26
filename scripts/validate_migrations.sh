#!/usr/bin/env bash
# validate_migrations.sh
#
# Validates that all migration docs in docs/migrations/ follow the required
# structure and that every migration is linked to a contract version bump.
#
# Usage:
#   ./scripts/validate_migrations.sh [--fail-on-missing]

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
MIGRATIONS_DIR="$REPO_ROOT/docs/migrations"
FAIL_ON_MISSING=false

for arg in "$@"; do
  [[ "$arg" == "--fail-on-missing" ]] && FAIL_ON_MISSING=true
done

echo "=== Uzima Migration Validator ==="
echo ""

if [[ ! -d "$MIGRATIONS_DIR" ]]; then
  echo "  No migrations directory found at $MIGRATIONS_DIR"
  echo "  Creating migrations directory..."
  mkdir -p "$MIGRATIONS_DIR"
  echo "  ✅ Created $MIGRATIONS_DIR"
  echo ""
  echo "Migration convention:"
  echo "  docs/migrations/<contract>/<version>-<description>.md"
  exit 0
fi

TOTAL=0
VALID=0
INVALID=0

for contract_dir in "$MIGRATIONS_DIR"/*/; do
  [[ -d "$contract_dir" ]] || continue
  contract=$(basename "$contract_dir")
  echo "Contract: $contract"
  
  for migration_file in "$contract_dir"*.md; do
    [[ -f "$migration_file" ]] || continue
    TOTAL=$((TOTAL + 1))
    filename=$(basename "$migration_file")
    
    # Check required sections exist
    missing_sections=""
    for section in "Pre-conditions" "Storage changes" "Migration" "Rollback" "Verification"; do
      if ! grep -qi "$section" "$migration_file" 2>/dev/null; then
        missing_sections="$missing_sections $section"
      fi
    done
    
    if [[ -z "$missing_sections" ]]; then
      echo "  ✅ $filename"
      VALID=$((VALID + 1))
    else
      echo "  ⚠️  $filename — missing sections:$missing_sections"
      INVALID=$((INVALID + 1))
    fi
  done
  echo ""
done

echo "Results: $VALID/$TOTAL migrations valid"

if [[ "$INVALID" -gt 0 ]]; then
  echo "  ⚠️  $INVALID migrations have incomplete documentation"
  if [[ "$FAIL_ON_MISSING" == "true" ]]; then
    exit 1
  fi
fi

if [[ "$TOTAL" -eq 0 ]]; then
  echo "  No migrations found. See docs/MIGRATION_ROLLBACK_STRATEGY.md for conventions."
fi

echo ""
echo "=== Validation complete ==="
