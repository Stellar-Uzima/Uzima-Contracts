#!/usr/bin/env bash
# Report contracts with an admin-recovery-shaped function (recover/restore/
# override/emergency_*) that have no nearby test asserting an unauthorized
# caller is rejected.
#
# Heuristic check — see docs/audits/ADMIN_RECOVERY_HARDENING.md. With --strict,
# fails if a contract added vs BASE_REF (default origin/main) lacks negative tests;
# pre-existing contracts are reported only.
#
# Usage: ./scripts/check_admin_recovery_test_coverage.sh [--strict]

set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
CONTRACTS_DIR="$ROOT_DIR/contracts"

STRICT=false
for arg in "$@"; do
  case "$arg" in
    --strict) STRICT=true ;;
  esac
done

# Contracts added relative to the base branch (strict mode only gates these).
BASE_REF="${BASE_REF:-origin/main}"
NEW_CONTRACTS="$(git -C "$ROOT_DIR" diff --name-only --diff-filter=A "$BASE_REF...HEAD" -- 'contracts/*/Cargo.toml' 2>/dev/null | cut -d/ -f2 || true)"

RECOVERY_FN_PATTERN='pub fn (recover|restore|override|emergency_)[a-zA-Z_]*\(.*caller'
NEG_TEST_PATTERN='unauthorized|Unauthorized|non_admin|not_admin'

echo "Admin/recovery negative-test coverage check"
echo "============================================"
echo ""

flagged=0
flagged_new=0
checked=0

for dir in "$CONTRACTS_DIR"/*/; do
  name="$(basename "$dir")"
  [[ -f "$dir/Cargo.toml" ]] || continue
  [[ -d "$dir/src" ]] || continue

  grep -rqE "$RECOVERY_FN_PATTERN" "$dir/src" 2>/dev/null || continue
  checked=$((checked + 1))

  if grep -rqE "$NEG_TEST_PATTERN" "$dir/src" 2>/dev/null; then
    continue
  fi

  flagged=$((flagged + 1))
  if grep -qxF "$name" <<<"$NEW_CONTRACTS"; then flagged_new=$((flagged_new + 1)); echo "  NEW CONTRACT: $name"; fi
  echo "  MISSING NEGATIVE TEST: $name"
done

echo ""
echo "Checked $checked contract(s) with an admin-recovery-shaped function."
echo "$flagged have no unauthorized/negative-test pattern anywhere in src/."
echo ""
echo "See docs/audits/ADMIN_RECOVERY_HARDENING.md for the per-contract"
echo "breakdown (this heuristic can't tell whether an existing test truly"
echo "targets the recovery function vs. some other authorization check)."

if $STRICT && [[ "$flagged_new" -gt 0 ]]; then
  exit 1
fi

exit 0
