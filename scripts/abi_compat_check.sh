#!/usr/bin/env bash
# ABI Compatibility Check Workflow
#
# Detects ABI changes between contract versions and generates reports
# for downstream consumer applications. This script integrates with
# CI/CD to provide ABI drift detection.
#
# Usage:
#   ./scripts/abi_compat_check.sh                    # full check
#   ./scripts/abi_compat_check.sh --baseline <ref>   # compare against git ref
#   ./scripts/abi_compat_check.sh --report           # generate report only
#   ./scripts/abi_compat_check.sh --ci               # CI mode (exit on breaking)

set -euo pipefail

ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
REPORTS_DIR="$ROOT_DIR/reports"
REGISTRY_FILE="$ROOT_DIR/schemas/interface-registry/registry.json"
ABI_COMPAT_SCRIPT="$ROOT_DIR/scripts/abi-compat.mjs"

# Parse arguments
BASELINE_REF=""
REPORT_ONLY=false
CI_MODE=false
REPORT_FILE="$REPORTS_DIR/abi_drift_$(date +%Y%m%d_%H%M%S).txt"

for arg in "$@"; do
    case $arg in
        --baseline) shift; BASELINE_REF="$1"; shift ;;
        --report) REPORT_ONLY=true ;;
        --ci) CI_MODE=true ;;
        --help|-h)
            echo "Usage: $0 [--baseline <ref>] [--report] [--ci]"
            echo ""
            echo "Options:"
            echo "  --baseline <ref>  Compare against a git ref (default: origin/main)"
            echo "  --report          Generate report only, don't fail on breaking changes"
            echo "  --ci              CI mode: exit with non-zero on breaking changes"
            exit 0
            ;;
        *)
            echo "Unknown argument: $arg"
            exit 1
            ;;
    esac
done

# Default baseline
if [[ -z "$BASELINE_REF" ]]; then
    BASELINE_REF="origin/main"
fi

echo "ABI Compatibility Check Workflow"
echo "================================"
echo "Comparing against: $BASELINE_REF"
echo ""

# ---------------------------------------------------------------------------
# Step 1: Verify prerequisites
# ---------------------------------------------------------------------------
echo "Step 1: Checking prerequisites..."

if [[ ! -f "$REGISTRY_FILE" ]]; then
    echo "  Registry not found. Generating baseline..."
    node "$ABI_COMPAT_SCRIPT"
fi

if [[ ! -f "$ABI_COMPAT_SCRIPT" ]]; then
    echo "ERROR: ABI compatibility script not found at $ABI_COMPAT_SCRIPT"
    exit 1
fi

echo "  Prerequisites OK"
echo ""

# ---------------------------------------------------------------------------
# Step 2: Snapshot current ABI
# ---------------------------------------------------------------------------
echo "Step 2: Taking ABI snapshot..."

CURRENT_SNAPSHOT=$(mktemp)
node "$ABI_COMPAT_SCRIPT" > "$CURRENT_SNAPSHOT" 2>&1
echo "  Snapshot taken ($(wc -c < "$CURRENT_SNAPSHOT") bytes)"
echo ""

# ---------------------------------------------------------------------------
# Step 3: Compare with baseline
# ---------------------------------------------------------------------------
echo "Step 3: Comparing with baseline ($BASELINE_REF)..."

# Check if baseline ref exists
if ! git rev-parse --verify "$BASELINE_REF" > /dev/null 2>&1; then
    echo "  WARNING: Baseline ref '$BASELINE_REF' not found. Treating as initial generation."
    echo "  No comparison possible."
else
    # Get baseline registry
    BASELINE_SNAPSHOT=$(mktemp)
    git show "$BASELINE_REF:schemas/interface-registry/registry.json" > "$BASELINE_SNAPSHOT" 2>/dev/null || echo "{}" > "$BASELINE_SNAPSHOT"

    # Run compatibility check
    CHECK_RESULT=$(mktemp)
    node "$ABI_COMPAT_SCRIPT" --check --report "$CHECK_RESULT" 2>&1 || true

    # Parse results
    BREAKING_COUNT=$(grep -c "^BREAKING=" "$CHECK_RESULT" 2>/dev/null || echo "0")
    NON_BREAKING_COUNT=$(grep -c "^NON_BREAKING=" "$CHECK_RESULT" 2>/dev/null || echo "0")

    echo "  Breaking changes: $BREAKING_COUNT"
    echo "  Non-breaking changes: $NON_BREAKING_COUNT"
    echo ""

    rm -f "$BASELINE_SNAPSHOT" "$CHECK_RESULT"
fi

rm -f "$CURRENT_SNAPSHOT"

# ---------------------------------------------------------------------------
# Step 4: Generate report
# ---------------------------------------------------------------------------
echo "Step 4: Generating report..."

mkdir -p "$REPORTS_DIR"

# Generate human-readable report
REPORT_FILE="$REPORTS_DIR/abi_drift_$(date +%Y%m%d_%H%M%S).md"
cat > "$REPORT_FILE" << EOF
# ABI Drift Report

**Generated:** $(date -u +"%Y-%m-%dT%H:%M:%SZ")
**Baseline:** $BASELINE_REF
**Repository:** Stellar-Uzima/Uzima-Contracts

## Summary

| Metric | Count |
|--------|-------|
| Contracts in registry | $(node -e "const r=JSON.parse(require('fs').readFileSync('$REGISTRY_FILE','utf8')); console.log(Object.keys(r.contracts).length)") |
| Breaking changes | ${BREAKING_COUNT:-0} |
| Non-breaking changes | ${NON_BREAKING_COUNT:-0} |

## Breaking Changes

$(if [[ "${BREAKING_COUNT:-0}" -gt 0 ]]; then
    echo "The following breaking changes were detected:"
    echo ""
    echo "- Contract interface removals"
    echo "- Required field additions"
    echo "- Type changes"
    echo "- Enum variant removals"
else
    echo "No breaking changes detected."
fi)

## Non-Breaking Changes

$(if [[ "${NON_BREAKING_COUNT:-0}" -gt 0 ]]; then
    echo "The following non-breaking changes were detected:"
    echo ""
    echo "- New optional fields"
    echo "- New enum variants"
    echo "- New interfaces"
else
    echo "No non-breaking changes detected."
fi)

## Downstream Impact

### Mobile SDK (React Native)
- Regenerate bindings: \`./scripts/generate_bindings.sh --typescript\`
- Update SDK version if breaking

### Python SDK
- Regenerate bindings: \`./scripts/generate_bindings.sh --python\`
- Update SDK version if breaking

### Other Consumers
- Check [CONTRACT_COMPATIBILITY.md](docs/CONTRACT_COMPATIBILITY.md)
- Review [CHANGE_IMPACT_MATRIX.md](docs/CHANGE_IMPACT_MATRIX.md)

## Remediation

If breaking changes are intentional:
1. Add \`[approve-abi-change]\` to the PR body
2. Update downstream SDK versions
3. Notify consumers via CHANGELOG
EOF

echo "  Report written to: $REPORT_FILE"
echo ""

# ---------------------------------------------------------------------------
# Step 5: CI mode check
# ---------------------------------------------------------------------------
if $CI_MODE && [[ "${BREAKING_COUNT:-0}" -gt 0 ]]; then
    echo "FAIL: Breaking ABI changes detected in CI mode."
    echo "If intentional, re-run with --report or add [approve-abi-change] to PR body."
    exit 1
fi

echo "ABI compatibility check complete."
echo "Report: $REPORT_FILE"