#!/usr/bin/env bash
# ============================================================================
# dependency_hazard_report.sh
# Generates a contract dependency hazard report for a release candidate.
#
# Usage:
#   ./scripts/dependency_hazard_report.sh [version]
#
# Output:
#   schemas/dependency-hazard-report.json  — machine-readable report
#   Prints human-readable summary to stdout
# ============================================================================
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

info()    { echo -e "${GREEN}[OK]${NC}    $1"; }
warn()    { echo -e "${YELLOW}[WARN]${NC}  $1"; }
fail()    { echo -e "${RED}[HAZARD]${NC} $1"; }

VERSION="${1:-$(git -C "$ROOT_DIR" describe --tags --abbrev=0 2>/dev/null || echo 'unreleased')}"
TIMESTAMP=$(date -u +%Y-%m-%dT%H:%M:%SZ)
REPORT_FILE="$ROOT_DIR/schemas/dependency-hazard-report.json"

HAZARD_COUNT=0
WARN_COUNT=0

echo ""
echo -e "${BLUE}═══════════════════════════════════════════════════════════${NC}"
echo -e "${BLUE}  Contract Dependency Hazard Report — $VERSION${NC}"
echo -e "${BLUE}═══════════════════════════════════════════════════════════${NC}"
echo ""

# ── 1. Workspace members vs Cargo.toml excludes ──────────────────────────────
echo -e "${BLUE}── Workspace Membership ──${NC}"
MEMBERS=$(grep -c '^    "contracts/' "$ROOT_DIR/Cargo.toml" 2>/dev/null || echo 0)
EXCLUDES=$(sed -n '/^exclude = \[/,/^]/p' "$ROOT_DIR/Cargo.toml" | grep -c '"contracts/' 2>/dev/null || echo 0)
TOTAL_CONTRACTS=$(ls -d "$ROOT_DIR"/contracts/*/Cargo.toml 2>/dev/null | wc -l | tr -d ' ')
ACTIVE=$((TOTAL_CONTRACTS - EXCLUDES))

info "Total contract directories: $TOTAL_CONTRACTS"
info "Active workspace members: $ACTIVE"
info "Excluded contracts: $EXCLUDES"

if [[ $EXCLUDES -gt 0 ]]; then
  echo ""
  warn "Excluded contracts (may have stale dependencies):"
  sed -n '/^exclude = \[/,/^]/p' "$ROOT_DIR/Cargo.toml" | grep '"contracts/' | sed 's/.*"contracts\//  - /;s/".*//' | while read -r c; do
    echo "    $c"
  done
  WARN_COUNT=$((WARN_COUNT + EXCLUDES))
fi

# ── 2. Dependency version alignment ───────────────────────────────────────────
echo ""
echo -e "${BLUE}── Dependency Version Alignment ──${NC}"

WORKSPACE_SOROBAN=$(grep 'soroban-sdk' "$ROOT_DIR/Cargo.toml" | head -1 | grep -o '"[^"]*"' | tr -d '"' || echo "unknown")
info "Workspace soroban-sdk: $WORKSPACE_SOROBAN"

MISALIGNED=0
for manifest in "$ROOT_DIR"/contracts/*/Cargo.toml; do
  [[ -f "$manifest" ]] || continue
  CONTRACT=$(basename "$(dirname "$manifest")")
  # Skip excluded
  if sed -n '/^exclude = \[/,/^]/p' "$ROOT_DIR/Cargo.toml" | grep -q "\"contracts/$CONTRACT\""; then
    continue
  fi
  LOCAL_VER=$(grep 'soroban-sdk' "$manifest" 2>/dev/null | grep -o 'version[^,]*' | head -1 | grep -o '"[^"]*"' | tr -d '"' || echo "")
  if [[ -n "$LOCAL_VER" && "$LOCAL_VER" != "$WORKSPACE_SOROBAN" && "$LOCAL_VER" != *"workspace"* ]]; then
    fail "$CONTRACT uses soroban-sdk $LOCAL_VER (workspace: $WORKSPACE_SOROBAN)"
    MISALIGNED=$((MISALIGNED + 1))
  fi
done

if [[ $MISALIGNED -eq 0 ]]; then
  info "All active contracts use aligned soroban-sdk version"
else
  HAZARD_COUNT=$((HAZARD_COUNT + MISALIGNED))
fi

# ── 3. Cross-contract dependency graph ────────────────────────────────────────
echo ""
echo -e "${BLUE}── Cross-Contract Dependencies ──${NC}"

DEP_GRAPH=""
for manifest in "$ROOT_DIR"/contracts/*/Cargo.toml; do
  [[ -f "$manifest" ]] || continue
  CONTRACT=$(basename "$(dirname "$manifest")")
  if sed -n '/^exclude = \[/,/^]/p' "$ROOT_DIR/Cargo.toml" | grep -q "\"contracts/$CONTRACT\""; then
    continue
  fi
  DEPS=$(grep -E '^\w+_' "$manifest" 2>/dev/null | grep -v 'soroban' | grep -v '\[dev' | sed 's/ = .*//' || true)
  if [[ -n "$DEPS" ]]; then
    for dep in $DEPS; do
      if [[ -d "$ROOT_DIR/contracts/$dep" ]]; then
        echo "  $CONTRACT -> $dep"
      fi
    done
  fi
done

# ── 4. Circular dependency detection (simple 2-hop) ───────────────────────────
echo ""
echo -e "${BLUE}── Circular Dependency Check ──${NC}"

CIRCULAR_FOUND=0
for manifest in "$ROOT_DIR"/contracts/*/Cargo.toml; do
  [[ -f "$manifest" ]] || continue
  A=$(basename "$(dirname "$manifest")")
  DEPS_A=$(grep -E '^\w+_' "$manifest" 2>/dev/null | grep -v 'soroban' | sed 's/ = .*//' || true)
  for dep_a in $DEPS_A; do
    if [[ -d "$ROOT_DIR/contracts/$dep_a" ]]; then
      DEP_MANIFEST="$ROOT_DIR/contracts/$dep_a/Cargo.toml"
      if grep -qE "^${A} " "$DEP_MANIFEST" 2>/dev/null; then
        fail "Circular dependency: $A <-> $dep_a"
        CIRCULAR_FOUND=1
        HAZARD_COUNT=$((HAZARD_COUNT + 1))
      fi
    fi
  done
done

if [[ $CIRCULAR_FOUND -eq 0 ]]; then
  info "No circular dependencies detected"
fi

# ── 5. WASM size budget check ────────────────────────────────────────────────
echo ""
echo -e "${BLUE}── WASM Size Budget ──${NC}"

WASM_DIR="$ROOT_DIR/target/wasm32-unknown-unknown/release"
MAX_WASM_KB=640  # Soroban max contract size
OVERSIZE=0

if [[ -d "$WASM_DIR" ]]; then
  for wasm in "$WASM_DIR"/*.wasm; do
    [[ -f "$wasm" ]] || continue
    NAME=$(basename "$wasm" .wasm)
    SIZE_KB=$(($(stat -f%z "$wasm" 2>/dev/null || stat -c%s "$wasm" 2>/dev/null || echo 0) / 1024))
    if [[ $SIZE_KB -gt $MAX_WASM_KB ]]; then
      fail "$NAME: ${SIZE_KB}KB exceeds ${MAX_WASM_KB}KB budget"
      OVERSIZE=$((OVERSIZE + 1))
      HAZARD_COUNT=$((HAZARD_COUNT + 1))
    fi
  done
fi

if [[ $OVERSIZE -eq 0 ]]; then
  info "All WASM artifacts within size budget"
fi

# ── 6. Generate JSON report ──────────────────────────────────────────────────
echo ""
echo -e "${BLUE}── Generating Report ──${NC}"

cat > "$REPORT_FILE" <<EOF
{
  "version": "$VERSION",
  "generated_at": "$TIMESTAMP",
  "summary": {
    "total_contracts": $TOTAL_CONTRACTS,
    "active_contracts": $ACTIVE,
    "excluded_contracts": $EXCLUDES,
    "hazard_count": $HAZARD_COUNT,
    "warning_count": $WARN_COUNT,
    "workspace_soroban_sdk": "$WORKSPACE_SOROBAN"
  },
  "checks": {
    "workspace_membership": { "status": "pass", "total": $TOTAL_CONTRACTS, "active": $ACTIVE, "excluded": $EXCLUDES },
    "version_alignment": { "status": "$([ $MISALIGNED -eq 0 ] && echo pass || echo fail)", "misaligned": $MISALIGNED },
    "circular_dependencies": { "status": "$([ $CIRCULAR_FOUND -eq 0 ] && echo pass || echo fail)" },
    "wasm_size_budget": { "status": "$([ $OVERSIZE -eq 0 ] && echo pass || echo fail)", "oversized": $OVERSIZE }
  }
}
EOF

info "Report written to: $REPORT_FILE"

# ── Summary ───────────────────────────────────────────────────────────────────
echo ""
if [[ $HAZARD_COUNT -gt 0 ]]; then
  fail "Dependency hazard report: $HAZARD_COUNT hazard(s), $WARN_COUNT warning(s)"
  exit 1
else
  info "Dependency hazard report: 0 hazards, $WARN_COUNT warning(s)"
  exit 0
fi
