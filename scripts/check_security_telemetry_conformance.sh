#!/usr/bin/env bash
# Report which contracts do not emit the standardized telemetry events
# described in docs/TELEMETRY_SCHEMA.md, and which emit a *different* schema
# under the same names.
#
# Two independent gaps are reported:
#
#   MISSING   — a contract whose directory name suggests a security-sensitive
#               responsibility (auth, treasury, admin, bridge, consent,
#               credential, custody, escrow, recovery) that emits no telemetry
#               at all.
#   DIVERGENT — a contract that references the telemetry API but not the schema
#               in docs/TELEMETRY_SCHEMA.md: a different topic root, and/or no
#               `trace_id`, and/or no `schema_version`. Such an event cannot be
#               grouped with conforming events by `trace_id` and cannot be
#               version-filtered, so it silently breaks the unified querying the
#               schema exists to provide (#1586).
#
# The DIVERGENT pass scans every contract rather than only name-matched ones,
# and that is deliberate: a name-matched pass cannot find this class of bug,
# because a contract that invents its own `TelemetryEvent` still satisfies a
# name-based "does it mention telemetry?" search.
#
# Heuristic, informational check (see docs/audits/
# SECURITY_TELEMETRY_CONFORMANCE.md for the full gap analysis and why it's
# non-blocking today): the flagged lists are long and each entry needs an
# owner decision (instrument it, conform it, or document why it's exempt), not
# an automatic CI failure.
#
# Usage: ./scripts/check_security_telemetry_conformance.sh [--strict]
#   --strict   exit 1 if any MISSING contract is found (default: exit 0,
#              report only). DIVERGENT never gates: a divergent contract is
#              already deployed and emitting events, so failing the build on it
#              would be reporting a finding as a build break.

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

# The three properties docs/TELEMETRY_SCHEMA.md requires of a conforming
# emitter. Each is checked independently so the report says which one is
# missing rather than just "not conformant".
CANONICAL_TOPIC_PATTERN='symbol_short!\("TEL"\)'
TRACE_ID_PATTERN='trace_id'
SCHEMA_VERSION_PATTERN='schema_version'

echo "Security telemetry conformance check"
echo "====================================="
echo ""

flagged=0
checked=0
divergent=0
emitting=0

# ── Pass 1: security-sensitive contracts with no telemetry at all ──────────
echo "Contracts with no telemetry (by name heuristic):"
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
echo "Checked $checked security-sensitive contract(s) by name; $flagged emit no telemetry."
echo ""

# ── Pass 2: contracts that emit telemetry but not the documented schema ────
echo "Contracts emitting telemetry, checked against docs/TELEMETRY_SCHEMA.md:"
for dir in "$CONTRACTS_DIR"/*/; do
  name="$(basename "$dir")"
  [[ -f "$dir/Cargo.toml" ]] || continue
  [[ -d "$dir/src" ]] || continue

  grep -rqE "$TELEMETRY_PATTERN" "$dir/src" 2>/dev/null || continue

  emitting=$((emitting + 1))
  missing_markers=()

  grep -rqE "$CANONICAL_TOPIC_PATTERN" "$dir/src" 2>/dev/null ||
    missing_markers+=('canonical topic root symbol_short!("TEL")')
  grep -rq "$TRACE_ID_PATTERN" "$dir/src" 2>/dev/null ||
    missing_markers+=("trace_id")
  grep -rq "$SCHEMA_VERSION_PATTERN" "$dir/src" 2>/dev/null ||
    missing_markers+=("schema_version")

  if [[ ${#missing_markers[@]} -eq 0 ]]; then
    echo "  CONFORMS: $name"
    continue
  fi

  divergent=$((divergent + 1))
  echo "  DIVERGENT SCHEMA: $name"
  for marker in "${missing_markers[@]}"; do
    echo "      missing: $marker"
  done
done

echo ""
echo "$emitting contract(s) reference the telemetry API; $divergent do not match"
echo "the schema in docs/TELEMETRY_SCHEMA.md."
echo ""
echo "A DIVERGENT contract emits events under its own topic and without a"
echo "trace_id, so its events cannot be joined with conforming ones. Either"
echo "conform it or record an explicit exemption."
echo ""
echo "See docs/audits/SECURITY_TELEMETRY_CONFORMANCE.md for the full picture."

if $STRICT && [[ "$flagged" -gt 0 ]]; then
  exit 1
fi

exit 0
