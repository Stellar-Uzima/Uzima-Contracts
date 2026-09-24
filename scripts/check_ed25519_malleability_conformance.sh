#!/usr/bin/env bash
# Report contracts that call Soroban's ed25519_verify() without any nearby
# reference to the canonical signature-malleability check
# (common_error::sig_malleability::check_s_value).
#
# Informational check — see
# docs/audits/ED25519_SIGNATURE_MALLEABILITY_CONFORMANCE.md for the full
# analysis and why this doesn't fail the build yet.
#
# Usage: ./scripts/check_ed25519_malleability_conformance.sh [--strict]

set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
CONTRACTS_DIR="$ROOT_DIR/contracts"

STRICT=false
for arg in "$@"; do
  case "$arg" in
    --strict) STRICT=true ;;
  esac
done

echo "Ed25519 signature-malleability check conformance"
echo "==================================================="
echo ""

flagged=0
checked=0

for dir in "$CONTRACTS_DIR"/*/; do
  name="$(basename "$dir")"
  [[ -d "$dir/src" ]] || continue

  grep -rq 'ed25519_verify' "$dir/src" --include="*.rs" 2>/dev/null || continue
  checked=$((checked + 1))

  if grep -rq 'check_s_value\|sig_malleability' "$dir/src" --include="*.rs" 2>/dev/null; then
    continue
  fi

  flagged=$((flagged + 1))
  echo "  MISSING MALLEABILITY CHECK: $name"
done

echo ""
echo "Checked $checked contract(s) calling ed25519_verify()."
echo "$flagged do not reference the canonical sig_malleability check."
echo ""
echo "See docs/audits/ED25519_SIGNATURE_MALLEABILITY_CONFORMANCE.md for the"
echo "per-contract follow-up list."

if $STRICT && [[ "$flagged" -gt 0 ]]; then
  exit 1
fi

exit 0
