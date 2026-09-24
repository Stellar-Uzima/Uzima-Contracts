#!/usr/bin/env bash
# Heuristic, informational checks for docs/CRYPTOGRAPHIC_SECURITY_MODEL.md
# conformance. See docs/audits/CRYPTOGRAPHIC_SECURITY_MODEL_CONFORMANCE.md
# for the full analysis, false-positive caveats, and why this doesn't fail
# the build yet.
#
# Usage: ./scripts/check_crypto_conformance.sh [--strict]

set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
CONTRACTS_DIR="$ROOT_DIR/contracts"

STRICT=false
for arg in "$@"; do
  case "$arg" in
    --strict) STRICT=true ;;
  esac
done

echo "Cryptographic security model conformance check"
echo "================================================"
echo ""

# ── Check 1: weak hash algorithm dependencies ──────────────────────────────
echo "-- Weak hash algorithm dependencies --"
weak_deps=0
if grep -qE '^name = "(md5|sha1)"' "$ROOT_DIR/Cargo.lock" 2>/dev/null; then
  echo "  FOUND: md5 or sha1 present in Cargo.lock"
  weak_deps=1
fi
for manifest in "$CONTRACTS_DIR"/*/Cargo.toml; do
  [[ -f "$manifest" ]] || continue
  if grep -qE '^(md5|sha1) *=' "$manifest" 2>/dev/null; then
    echo "  FOUND: $(basename "$(dirname "$manifest")") depends directly on md5/sha1"
    weak_deps=1
  fi
done
if [[ "$weak_deps" -eq 0 ]]; then
  echo "  None found."
fi

# ── Check 2: crypto-named functions without an env.crypto() call nearby ────
echo ""
echo "-- Contracts with encrypt/decrypt/hash functions not using env.crypto() --"
echo "   (heuristic — some of these are legitimate, e.g. homomorphic encryption;"
echo "    see the audit doc before treating any of these as confirmed findings)"
flagged=0
for dir in "$CONTRACTS_DIR"/*/; do
  name="$(basename "$dir")"
  [[ -d "$dir/src" ]] || continue
  grep -rqE 'fn [a-zA-Z_]*(encrypt|decrypt|_hash|hash_)' "$dir/src" --include="*.rs" 2>/dev/null || continue
  if grep -rq 'env.crypto()\|Env::crypto' "$dir/src" --include="*.rs" 2>/dev/null; then
    continue
  fi
  flagged=$((flagged + 1))
  echo "  $name"
done
if [[ "$flagged" -eq 0 ]]; then
  echo "  None found."
fi

echo ""
echo "See docs/audits/CRYPTOGRAPHIC_SECURITY_MODEL_CONFORMANCE.md for context"
echo "and open follow-up items (including the governance-pattern check that"
echo "isn't automated here)."

if $STRICT && { [[ "$weak_deps" -gt 0 ]] || [[ "$flagged" -gt 0 ]]; }; then
  exit 1
fi

exit 0
