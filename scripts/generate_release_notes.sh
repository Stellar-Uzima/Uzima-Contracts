#!/usr/bin/env bash
# ============================================================================
# generate_release_notes.sh
# Generates structured release notes from the interface registry, event
# registry, and git commit history since the last tag.
#
# Usage:
#   ./scripts/generate_release_notes.sh [version]
#
# The optional `version` argument is used as the heading for the release.
# When omitted the script auto-detects the latest git tag.
# ============================================================================
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

info()  { echo -e "${GREEN}[INFO]${NC}  $1"; }
warn()  { echo -e "${YELLOW}[WARN]${NC}  $1"; }
error() { echo -e "${RED}[ERROR]${NC} $1"; }

# ---------------------------------------------------------------------------
# Determine version & previous tag
# ---------------------------------------------------------------------------
if [[ $# -ge 1 ]]; then
  VERSION="$1"
  PREV_TAG=$(git describe --tags --abbrev=0 2>/dev/null || echo "")
else
  VERSION=$(git describe --tags --abbrev=0 2>/dev/null || echo "unreleased")
  PREV_TAG=$(git describe --tags --abbrev=0 --tags "${VERSION}^" 2>/dev/null || echo "")
fi

TAG_DATE=$(git log -1 --format='%ai' "$VERSION" 2>/dev/null || date '+%Y-%m-%d')
echo ""
echo -e "${BLUE}═══════════════════════════════════════════════════════════${NC}"
echo -e "${BLUE}  Release Notes Generator — Stellar Uzima Contracts${NC}"
echo -e "${BLUE}═══════════════════════════════════════════════════════════${NC}"
echo ""
info "Version : $VERSION"
info "Date    : $TAG_DATE"
info "Prev    : ${PREV_TAG:-<initial release>}"
echo ""

# ---------------------------------------------------------------------------
# Collect interface changes (new / changed public functions in contracts)
# ---------------------------------------------------------------------------
INTERFACE_CHANGES=()
if [[ -n "$PREV_TAG" ]]; then
  while IFS= read -r line; do
    INTERFACE_CHANGES+=("$line")
  done < <(git diff "$PREV_TAG"..HEAD --name-only -- 'contracts/*/src/lib.rs' 2>/dev/null || true)
fi

# ---------------------------------------------------------------------------
# Collect high-risk file changes
# ---------------------------------------------------------------------------
HIGH_RISK_DIRS=("contracts/identity_registry" "contracts/governor" "contracts/medical_records" "contracts/fido2_authenticator")
HIGH_RISK_CHANGES=()
if [[ -n "$PREV_TAG" ]]; then
  for dir in "${HIGH_RISK_DIRS[@]}"; do
    while IFS= read -r line; do
      HIGH_RISK_CHANGES+=("$line")
    done < <(git diff "$PREV_TAG"..HEAD --name-only -- "$dir" 2>/dev/null || true)
  done
fi

# ---------------------------------------------------------------------------
# Generate release notes
# ---------------------------------------------------------------------------
OUTPUT="$ROOT_DIR/RELEASE_NOTES_${VERSION}.md"

{
  echo "# Release Notes — $VERSION"
  echo ""
  echo "**Date:** $TAG_DATE"
  echo ""
  echo "---"
  echo ""

  # Interface changes
  echo "## Interface Changes"
  echo ""
  if [[ ${#INTERFACE_CHANGES[@]} -gt 0 ]]; then
    for f in "${INTERFACE_CHANGES[@]}"; do
      echo "- \`$f\`"
    done
  else
    echo "_No contract interface changes in this release._"
  fi
  echo ""

  # High-risk areas
  echo "## High-Risk Areas"
  echo ""
  if [[ ${#HIGH_RISK_CHANGES[@]} -gt 0 ]]; then
    echo "> The following files in critical contract paths were modified:"
    echo ""
    for f in "${HIGH_RISK_CHANGES[@]}"; do
      echo "- \`$f\`"
    done
  else
    echo "_No changes to high-risk contract paths._"
  fi
  echo ""

  # Event changes
  echo "## Event Registry"
  echo ""
  if [[ -f "$ROOT_DIR/schemas/events/registry.json" ]]; then
    CONTRACT_COUNT=$(grep -c '"name":' "$ROOT_DIR/schemas/events/registry.json" 2>/dev/null || echo "?")
    EVENT_COUNT=$(grep -c '"id":' "$ROOT_DIR/schemas/events/registry.json" 2>/dev/null || echo "?")
    echo "- Registry version: $(grep '"registry_version"' "$ROOT_DIR/schemas/events/registry.json" | head -1 | tr -d ' ",' || echo '?')"
    echo "- Contracts with events: $CONTRACT_COUNT"
    echo "- Total event definitions: $EVENT_COUNT"
  else
    echo "_Event registry not found._"
  fi
  echo ""

  # Resource budgets summary
  echo "## Resource Budgets"
  echo ""
  if [[ -d "$ROOT_DIR/resource-budgets" ]]; then
    for budget_file in "$ROOT_DIR"/resource-budgets/*.json; do
      [[ -f "$budget_file" ]] || continue
      contract_name=$(basename "$budget_file" .json)
      echo "- \`$contract_name\`"
    done
  else
    echo "_Resource budgets directory not found._"
  fi
  echo ""

  # Commits summary
  echo "## Commits"
  echo ""
  if [[ -n "$PREV_TAG" ]]; then
    COMMIT_COUNT=$(git rev-list "$PREV_TAG"..HEAD --count 2>/dev/null || echo "?")
    echo "Total commits since $PREV_TAG: **$COMMIT_COUNT**"
    echo ""
    echo "### Notable changes"
    echo ""
    git log "$PREV_TAG"..HEAD --pretty=format:"- %s (%h)" --no-merges 2>/dev/null | head -30 || true
    echo ""
  else
    echo "_Initial release — no previous tag to compare against._"
  fi
  echo ""
  echo "---"
  echo ""
  echo "## Checklist"
  echo ""
  echo "- [ ] All interface changes reviewed"
  echo "- [ ] High-risk contract changes tested"
  echo "- [ ] Event registry up to date"
  echo "- [ ] Resource budgets reviewed"
  echo "- [ ] Migration / rollout guidance documented"
  echo ""

} > "$OUTPUT"

info "Release notes written to: $OUTPUT"
echo ""
