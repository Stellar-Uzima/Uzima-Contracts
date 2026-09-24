#!/usr/bin/env bash
# Report which security-sensitive contracts do NOT emit the standardized
# telemetry events described in docs/TELEMETRY_SCHEMA.md (i.e. they don't
# reference the shared TelemetryEvent/derive_trace_id/emit_telemetry API).
#
# This is a heuristic, informational check (see docs/audits/
# SECURITY_TELEMETRY_CONFORMANCE.md for the full gap analysis and why it's
# non-blocking today): it flags contracts whose directory name suggests a
# security-sensitive responsibility (auth, treasury, admin, bridge, consent,
# credential, custody, escrow, recovery) but that don't reference the
# telemetry API anywhere in their source.
#
# Usage: ./scripts/check_security_telemetry_conformance.sh [--strict]
#   --strict   exit 1 if any flagged contract is found (default: exit 0,
#              report only)

set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
CONTRACTS_DIR="$ROOT_DIR/contracts"

STRICT=false
for arg in "$@"; do
  case "$arg" in
    --strict) STRICT=true ;;
  esac
done

# Heuristic: directory name suggests security-sensitive responsibility.
SENSITIVE_PATTERN='auth|treasury|admin|bridge|consent|credential|custody|escrow|recovery|access_control|identity|key_'
TELEMETRY_PATTERN='TelemetryEvent|derive_trace_id|emit_telemetry'

echo "Security telemetry conformance check"
echo "====================================="
echo ""

flagged=0
checked=0

for dir in "$CONTRACTS_DIR"/*/; do
  name="$(basename "$dir")"
  [[ -f "$dir/Cargo.toml" ]] || continue
  [[ "$name" =~ $SENSITIVE_PATTERN ]] || continue

  checked=$((checked + 1))

  if grep -rqE "$TELEMETRY_PATTERN" "$dir/src" 2>/dev/null; then
    continue
  fi

  flagged=$((flagged + 1))
  echo "  MISSING TELEMETRY: $name"
done

echo ""
echo "Checked $checked security-sensitive contract(s) (by name heuristic)."
echo "$flagged do not reference the shared telemetry API."
echo ""
echo "See docs/audits/SECURITY_TELEMETRY_CONFORMANCE.md for the full picture"
echo "(only 2/114 contracts repo-wide currently emit standardized telemetry)."

if $STRICT && [[ "$flagged" -gt 0 ]]; then
  exit 1
fi

exit 0
