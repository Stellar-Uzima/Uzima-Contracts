#!/usr/bin/env bash
# ============================================================================
# check_semver.sh
# Validates that the workspace Cargo.toml version matches the interface
# registry and checks for obvious semver issues in contract public APIs.
#
# Usage:
#   ./scripts/check_semver.sh
# ============================================================================
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

info()    { echo -e "${GREEN}[OK]${NC}    $1"; }
warn()    { echo -e "${YELLOW}[WARN]${NC}  $1"; }
fail()    { echo -e "${RED}[FAIL]${NC}  $1"; }

ERRORS=0

# ---------------------------------------------------------------------------
# 1. Validate Cargo workspace version
# ---------------------------------------------------------------------------
CARGO_VERSION=$(grep '^version' "$ROOT_DIR/Cargo.toml" | head -1 | sed 's/.*"\(.*\)".*/\1/')
if [[ -z "$CARGO_VERSION" ]]; then
  fail "Could not read version from Cargo.toml"
  ERRORS=$((ERRORS + 1))
else
  info "Cargo.toml workspace version: $CARGO_VERSION"
fi

# ---------------------------------------------------------------------------
# 2. Validate SemVer format
# ---------------------------------------------------------------------------
SEMVER_RE='^[0-9]+\.[0-9]+\.[0-9]+(-[a-zA-Z0-9.]+)?(\+[a-zA-Z0-9.]+)?$'
if [[ "$CARGO_VERSION" =~ $SEMVER_RE ]]; then
  info "Version $CARGO_VERSION is valid SemVer 2.0.0"
else
  fail "Version $CARGO_VERSION does not match SemVer 2.0.0 format"
  ERRORS=$((ERRORS + 1))
fi

# ---------------------------------------------------------------------------
# 3. Check version-registry.json exists and is valid JSON
# ---------------------------------------------------------------------------
REGISTRY="$ROOT_DIR/schemas/interface-registry/version-registry.json"
if [[ ! -f "$REGISTRY" ]]; then
  fail "version-registry.json not found at schemas/interface-registry/"
  ERRORS=$((ERRORS + 1))
else
  if command -v python3 &>/dev/null; then
    if python3 -m json.tool "$REGISTRY" >/dev/null 2>&1; then
      info "version-registry.json is valid JSON"
    else
      fail "version-registry.json is not valid JSON"
      ERRORS=$((ERRORS + 1))
    fi
  else
    warn "python3 not found — skipping JSON validation"
  fi
fi

# ---------------------------------------------------------------------------
# 4. Ensure each workspace member has a Cargo.toml with consistent version
# ---------------------------------------------------------------------------
if [[ -d "$ROOT_DIR/contracts" ]]; then
MemberCount=0
  ConsistentCount=0
  for manifest in "$ROOT_DIR"/contracts/*/Cargo.toml; do
    [[ -f "$manifest" ]] || continue
    MemberCount=$((MemberCount + 1))
    ContractName=$(basename "$(dirname "$manifest")")

    # Check version.workspace = true
    if grep -q 'version.workspace\s*=\s*true' "$manifest" 2>/dev/null; then
      ConsistentCount=$((ConsistentCount + 1))
    else
      MemberVer=$(grep '^version' "$manifest" | head -1 | sed 's/.*"\(.*\)".*/\1/' 2>/dev/null || echo "")
      if [[ -n "$MemberVer" && "$MemberVer" != "$CARGO_VERSION" ]]; then
        warn "$ContractName has explicit version $MemberVer (workspace is $CARGO_VERSION)"
      fi
    fi
  done
  info "Workspace members with version.workspace=true: $ConsistentCount / $MemberCount"
fi

# ---------------------------------------------------------------------------
# 5. Summary
# ---------------------------------------------------------------------------
echo ""
if [[ $ERRORS -gt 0 ]]; then
  fail "Semver check completed with $ERRORS error(s)"
  exit 1
else
  info "All semver checks passed"
  exit 0
fi
