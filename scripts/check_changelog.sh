#!/usr/bin/env bash
# check_changelog.sh — Validate that CHANGELOG.md contains entries for modified contracts.
#
# Usage:
#   ./scripts/check_changelog.sh [base_ref]
#
# If base_ref is omitted, defaults to upstream/main. The script finds all
# modified files under contracts/ and checks that CHANGELOG.md mentions
# each contract directory at least once.
#
# Exit codes:
#   0 — all modified contracts have changelog entries (or no contracts modified)
#   1 — one or more contracts lack changelog entries

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
BASE_REF="${1:-upstream/main}"
CHANGELOG="$REPO_ROOT/CHANGELOG.md"

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[0;33m'
NC='\033[0m'

if [ ! -f "$CHANGELOG" ]; then
  echo -e "${RED}ERROR: CHANGELOG.md not found at $CHANGELOG${NC}"
  exit 1
fi

# Ensure the base ref exists
if ! git -C "$REPO_ROOT" rev-parse --verify "$BASE_REF" >/dev/null 2>&1; then
  echo -e "${YELLOW}WARNING: Base ref '$BASE_REF' not found locally. Falling back to HEAD~5.${NC}"
  BASE_REF="HEAD~5"
fi

# Collect modified contract directories
CONTRACT_DIRS=$(git -C "$REPO_ROOT" diff --name-only "$BASE_REF" HEAD -- 'contracts/' \
  | grep -E '^contracts/[^/]+/' \
  | cut -d'/' -f1,2 \
  | sort -u || true)

if [ -z "$CONTRACT_DIRS" ]; then
  echo -e "${GREEN}No modified contracts detected. Nothing to check.${NC}"
  exit 0
fi

FAILURES=0

echo "Checking changelog entries for modified contracts..."
echo ""

for DIR in $CONTRACT_DIRS; do
  CONTRACT_NAME=$(basename "$DIR")
  # Search for the contract name in CHANGELOG.md (case-insensitive)
  if grep -qi "$CONTRACT_NAME" "$CHANGELOG"; then
    echo -e "  ${GREEN}✓${NC} $DIR — entry found"
  else
    echo -e "  ${RED}✗${NC} $DIR — ${RED}no changelog entry found${NC}"
    FAILURES=$((FAILURES + 1))
  fi
done

echo ""

if [ "$FAILURES" -gt 0 ]; then
  echo -e "${RED}FAILED: $FAILURES contract(s) missing changelog entries.${NC}"
  echo "Please add entries to CHANGELOG.md under '### Storage & Schema Changes'"
  echo "or '### Added' / '### Changed' as appropriate."
  exit 1
else
  echo -e "${GREEN}PASSED: All modified contracts have changelog entries.${NC}"
  exit 0
fi
