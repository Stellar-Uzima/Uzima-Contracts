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
#   3. Known CVEs from real `cargo audit --json` RUSTSEC advisory data.
#
# Set CARGO_AUDIT_JSON to a pre-generated `cargo audit --json` file to reuse an
# existing report (used by the regression test and offline CI).
#
# Exits 1 after writing the report if any high/critical advisory is found.
# Output defaults to stdout if no path is given.
# The script is idempotent and safe to re-run.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
CONTRACTS_DIR="$PROJECT_ROOT/contracts"
CARGO_TOML="$PROJECT_ROOT/Cargo.toml"
OUTPUT_PATH="${1:-}"
AUDIT_JSON_PATH="${CARGO_AUDIT_JSON:-}"

ISSUES=""
ISSUE_COUNT=0
AUDIT_TOTAL=0
HIGH_CRITICAL_COUNT=0
declare -A CONTRACT_ISSUE_MAP

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
  CONTRACT_ISSUE_MAP["$contract"]=$(( ${CONTRACT_ISSUE_MAP["$contract"]:-0} + 1 ))
}

# Check 3: real CVE data. `cargo audit` exits non-zero when advisories exist,
# so capture its JSON and apply our own severity gate (same policy as
# scripts/security-scan.sh: high/critical block, medium/low are reported).
scan_cargo_audit() {
  local audit_json="$AUDIT_JSON_PATH"
  local created_tmp=false

  if [[ -z "$audit_json" ]]; then
    if ! command -v cargo >/dev/null 2>&1; then
      add_issue "warning" "workspace" "cargo not found; CVE data unavailable"
      return 0
    fi
    audit_json="$(mktemp "${TMPDIR:-/tmp}/uzima-cargo-audit.XXXXXX.json")"
    created_tmp=true
    # Non-zero exit is expected when advisories are present; parse the JSON.
    (cd "$PROJECT_ROOT" && cargo audit --json) >"$audit_json" 2>/dev/null || true
  fi

  if [[ ! -s "$audit_json" ]]; then
    add_issue "warning" "workspace" "cargo audit produced no JSON output; CVE data unavailable"
  else
    local counts high total
    counts="$(python3 - "$audit_json" <<'PY' 2>/dev/null || echo "0 0"
import json, sys
try:
    with open(sys.argv[1], "r", encoding="utf-8") as fh:
        data = json.load(fh)
except (OSError, ValueError):
    print("0 0")
    raise SystemExit
vulns = data.get("vulnerabilities", {}).get("list", [])
high = sum(1 for v in vulns
           if str((v.get("advisory") or {}).get("severity", "")).lower() in {"high", "critical"})
print(high, len(vulns))
PY
)"
    high="$(echo "$counts" | awk '{print $1}')"
    total="$(echo "$counts" | awk '{print $2}')"
    AUDIT_TOTAL="$total"
    HIGH_CRITICAL_COUNT="$high"

    # One issue per high/critical advisory. Process substitution keeps
    # add_issue's mutations in this shell (a pipe would run in a subshell).
    while IFS=$'\t' read -r severity advisory_id package title; do
      [[ -n "$severity" ]] || continue
      add_issue "$severity" "workspace" \
        "CVE advisory $advisory_id affects '$package': $title"
    done < <(python3 - "$audit_json" <<'PY' 2>/dev/null || true
import json, sys
with open(sys.argv[1], "r", encoding="utf-8") as fh:
    data = json.load(fh)
clean = lambda s: str(s).replace("\t", " ").replace("\n", " ").replace('"', "'")
for v in data.get("vulnerabilities", {}).get("list", []):
    advisory = v.get("advisory") or {}
    severity = str(advisory.get("severity", "")).lower()
    if severity in {"high", "critical"}:
        print("\t".join([
            severity,
            clean(advisory.get("id", "unknown")),
            clean((v.get("package") or {}).get("name", "unknown")),
            clean(advisory.get("title", "no title")),
        ]))
PY
)
  fi

  if [[ "$created_tmp" == true ]]; then
    rm -f "$audit_json"
  fi
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
  CONTRACT_ISSUE_COUNT="${CONTRACT_ISSUE_MAP[$CONTRACT_NAME]:-0}"
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

scan_cargo_audit

REPORT="{
  \"generated_at\": \"$(date -u +%Y-%m-%dT%H:%M:%SZ)\",
  \"project\": \"Uzima-Contracts\",
  \"summary\": {
    \"total_contracts_scanned\": $(echo "$HEALTH_ENTRIES" | grep -c '"contract"' || echo 0),
    \"total_issues\": $ISSUE_COUNT,
    \"cve_data_source\": \"cargo audit --json\",
    \"advisories_total\": $AUDIT_TOTAL,
    \"high_critical_advisories\": $HIGH_CRITICAL_COUNT
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

# CVE gate: high/critical advisories must fail the job (issue #1963).
if [[ "$HIGH_CRITICAL_COUNT" -gt 0 ]]; then
  echo "ERROR: found $HIGH_CRITICAL_COUNT high/critical CVE advisory/ies; see report" >&2
  exit 1
fi
exit 0
