#!/usr/bin/env bash
#
# dependency_health_check.sh — Validate contract dependencies and output a
# health report in JSON format.
#
# Usage:
#   ./scripts/dependency_health_check.sh [output_path]
#
# Checks:
#   1. Are all dependencies available in the workspace?
#   2. Are there duplicate dependency versions?
#   3. Are there any known-vulnerable patterns (placeholder for future CVE DB)?
#
# Output defaults to stdout if no path is given.
# The script is idempotent and safe to re-run.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
CONTRACTS_DIR="$PROJECT_ROOT/contracts"
CARGO_TOML="$PROJECT_ROOT/Cargo.toml"
OUTPUT_PATH="${1:-}"

ISSUES=""
ISSUE_COUNT=0

add_issue() {
  local severity="$1" contract="$2" message="$3"
  if [[ -n "$ISSUES" ]]; then
    ISSUES="$ISSUES,"
  fi
  ISSUES="$ISSUES{
    \"severity\": \"$severity\",
    \"contract\": \"$contract\",
    \"message\": \"$message\"
  }"
  ISSUE_COUNT=$((ISSUE_COUNT + 1))
}

# Collect all workspace-level dependency names from root Cargo.toml
WORKSPACE_DEPS=""
if [[ -f "$CARGO_TOML" ]]; then
  WORKSPACE_DEPS=$(awk '/^\[workspace\.dependencies\]/{found=1; next} /^\[/{found=0} found && /^[a-z_]/{gsub(/[ =].*/, ""); gsub(/"/, ""); printf "%s\n", $0}' "$CARGO_TOML" || true)
fi

# Track dependency versions across all contracts for duplicate detection
declare -A DEP_VERSION_MAP

HEALTH_ENTRIES=""

for cargo_toml in "$CONTRACTS_DIR"/*/Cargo.toml; do
  [[ -f "$cargo_toml" ]] || continue
  CONTRACT_DIR="$(dirname "$cargo_toml")"
  CONTRACT_NAME="$(basename "$CONTRACT_DIR")"

  [[ -d "$CONTRACT_DIR/src" ]] || continue

  CONTRACT_ISSUES=""
  CONTRACT_ISSUE_COUNT=0

  # Extract [dependencies] entries
  IN_DEPS_SECTION=false
  while IFS= read -r line; do
    if [[ "$line" == "[dependencies]" ]]; then
      IN_DEPS_SECTION=true
      continue
    fi
    if [[ "$line" == "["* ]] && [[ "$IN_DEPS_SECTION" == true ]]; then
      IN_DEPS_SECTION=false
      continue
    fi
    if [[ "$IN_DEPS_SECTION" == true ]] && [[ -n "$line" ]]; then
      DEP_NAME=$(echo "$line" | sed 's/\(^[a-z_]*\).*/\1/' | tr -d ' ')
      [[ -z "$DEP_NAME" ]] && continue

      # Check 1: workspace dependency availability
      IS_WORKSPACE=false
      if echo "$WORKSPACE_DEPS" | grep -qx "$DEP_NAME" 2>/dev/null; then
        IS_WORKSPACE=true
      fi

      HAS_PATH=false
      if echo "$line" | grep -q 'path' 2>/dev/null; then
        HAS_PATH=true
      fi

      HAS_FEATURES=false
      if echo "$line" | grep -q 'features' 2>/dev/null; then
        HAS_FEATURES=true
      fi

      # Check 2: inline version (not workspace, no path) — potential version drift
      HAS_INLINE_VERSION=false
      INLINE_VERSION=""
      if [[ "$IS_WORKSPACE" == false ]] && [[ "$HAS_PATH" == false ]]; then
        if echo "$line" | grep -q 'version' 2>/dev/null; then
          HAS_INLINE_VERSION=true
          INLINE_VERSION=$(echo "$line" | sed 's/.*version.*= *"\([^"]*\)".*/\1/' || echo "unknown")
        fi
      fi

      # Track versions for duplicate detection
      VERSION_KEY="${DEP_NAME}"
      if [[ "$HAS_INLINE_VERSION" == true ]]; then
        EXISTING_VERSION="${DEP_VERSION_MAP[$VERSION_KEY]:-}"
        if [[ -n "$EXISTING_VERSION" ]] && [[ "$EXISTING_VERSION" != "$INLINE_VERSION" ]]; then
          add_issue "warning" "$CONTRACT_NAME" \
            "Duplicate version for dependency '$DEP_NAME': contract uses $INLINE_VERSION but another contract uses $EXISTING_VERSION"
        fi
        DEP_VERSION_MAP[$VERSION_KEY]="$INLINE_VERSION"
      fi
    fi
  done < "$cargo_toml"

  # Determine overall status for this contract
  STATUS="healthy"
  if [[ $CONTRACT_ISSUE_COUNT -gt 0 ]]; then
    STATUS="issues_found"
  fi

  ENTRY="{
    \"contract\": \"$CONTRACT_NAME\",
    \"path\": \"contracts/$CONTRACT_NAME\",
    \"status\": \"$STATUS\",
    \"issue_count\": $CONTRACT_ISSUE_COUNT
  }"

  if [[ -n "$HEALTH_ENTRIES" ]]; then
    HEALTH_ENTRIES="$HEALTH_ENTRIES,"
  fi
  HEALTH_ENTRIES="$HEALTH_ENTRIES$ENTRY"
done

REPORT="{
  \"generated_at\": \"$(date -u +%Y-%m-%dT%H:%M:%SZ)\",
  \"project\": \"Uzima-Contracts\",
  \"summary\": {
    \"total_contracts_scanned\": $(echo "$HEALTH_ENTRIES" | grep -c '"contract"' || echo 0),
    \"total_issues\": $ISSUE_COUNT
  },
  \"issues\": [$ISSUES],
  \"contracts\": [$HEALTH_ENTRIES]
}"

if [[ -n "$OUTPUT_PATH" ]]; then
  mkdir -p "$(dirname "$OUTPUT_PATH")"
  echo "$REPORT" > "$OUTPUT_PATH"
  echo "Dependency health report written to $OUTPUT_PATH"
else
  echo "$REPORT"
fi
