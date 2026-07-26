#!/usr/bin/env bash
# legacy_migrator.sh — Offline migration tool for legacy medical record formats.
#
# Reads legacy JSON medical record files, detects their format, validates them
# against the current schema, and outputs transformed records.
#
# Usage:
#   ./scripts/legacy_migrator.sh [--dry-run] [--input DIR] [--output DIR] [--summary FILE]
#
# Options:
#   --dry-run       Validate and transform without writing output files
#   --input  DIR    Directory containing legacy JSON files (default: ./legacy_data)
#   --output DIR    Directory for transformed output (default: ./migrated_data)
#   --summary FILE  Write summary report to FILE (default: ./migration_report.txt)
#   -h, --help      Show this help message

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

# Defaults
DRY_RUN=false
INPUT_DIR="$REPO_ROOT/legacy_data"
OUTPUT_DIR="$REPO_ROOT/migrated_data"
SUMMARY_FILE="$REPO_ROOT/migration_report.txt"

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[0;33m'
CYAN='\033[0;36m'
NC='\033[0m'

# Counters
TOTAL=0
LEGACY_V1=0
LEGACY_V2=0
CURRENT=0
UNKNOWN=0
MIGRATED=0
FAILED=0
SKIPPED=0

usage() {
  sed -n '/^# Usage:/,/^$/p' "$0" | sed 's/^# \?//'
  sed -n '/^# Options:/,/^#$/p' "$0" | sed 's/^# \?//'
  exit 0
}

log_info()  { echo -e "${CYAN}[INFO]${NC}  $*"; }
log_ok()    { echo -e "${GREEN}[OK]${NC}    $*"; }
log_warn()  { echo -e "${YELLOW}[WARN]${NC}  $*"; }
log_error() { echo -e "${RED}[ERROR]${NC} $*"; }

# ── Parse arguments ──────────────────────────────────────────────────────────

while [ $# -gt 0 ]; do
  case "$1" in
    --dry-run)   DRY_RUN=true; shift ;;
    --input)     INPUT_DIR="$2"; shift 2 ;;
    --output)    OUTPUT_DIR="$2"; shift 2 ;;
    --summary)   SUMMARY_FILE="$2"; shift 2 ;;
    -h|--help)   usage ;;
    *)           log_error "Unknown option: $1"; usage ;;
  esac
done

# ── Detect format ────────────────────────────────────────────────────────────

detect_format() {
  local file="$1"
  # legacy_v1: has "record_type" and "patient_name" (flat fields, no version)
  # legacy_v2: has "schema_version" == "2" and "metadata" nested object
  # current:   has "schema_version" == "3" or "4" and "record_id"

  if ! command -v jq &>/dev/null; then
    log_error "jq is required but not installed. Install with: brew install jq"
    exit 1
  fi

  local schema_version
  schema_version=$(jq -r '.schema_version // empty' "$file" 2>/dev/null || echo "")

  if [ "$schema_version" = "" ]; then
    # Check for legacy v1 indicators
    if jq -e '.record_type' "$file" >/dev/null 2>&1 && \
       jq -e '.patient_name' "$file" >/dev/null 2>&1; then
      echo "legacy_v1"
      return
    fi
    echo "unknown"
    return
  fi

  case "$schema_version" in
    "2")  echo "legacy_v2" ;;
    "3"|"4") echo "current" ;;
    *)    echo "unknown" ;;
  esac
}

# ── Validate a record ────────────────────────────────────────────────────────

validate_record() {
  local file="$1"
  local format="$2"

  case "$format" in
    legacy_v1)
      # Check required v1 fields
      local missing=""
      for field in record_type patient_name; do
        if ! jq -e ".$field" "$file" >/dev/null 2>&1; then
          missing="$missing $field"
        fi
      done
      if [ -n "$missing" ]; then
        log_error "  Missing required v1 fields:$missing"
        return 1
      fi
      ;;
    legacy_v2)
      local missing=""
      for field in schema_version metadata patient_id; do
        if ! jq -e ".$field" "$file" >/dev/null 2>&1; then
          missing="$missing $field"
        fi
      done
      if [ -n "$missing" ]; then
        log_error "  Missing required v2 fields:$missing"
        return 1
      fi
      ;;
    current)
      local missing=""
      for field in schema_version record_id patient_id status created_at; do
        if ! jq -e ".$field" "$file" >/dev/null 2>&1; then
          missing="$missing $field"
        fi
      done
      if [ -n "$missing" ]; then
        log_error "  Missing required current fields:$missing"
        return 1
      fi
      ;;
    *)
      log_error "  Cannot validate unknown format"
      return 1
      ;;
  esac

  # General JSON validity
  if ! jq empty "$file" 2>/dev/null; then
    log_error "  Invalid JSON"
    return 1
  fi

  return 0
}

# ── Transform a record ───────────────────────────────────────────────────────

transform_record() {
  local file="$1"
  local format="$2"

  local now
  now=$(date -u +"%Y-%m-%dT%H:%M:%SZ")

  case "$format" in
    legacy_v1)
      # Map legacy v1 fields to current schema
      jq -n \
        --arg rid "migrated-$(date +%s)-$RANDOM" \
        --arg pid "$(jq -r '.patient_id // "unknown"' "$file")" \
        --arg rt "$(jq -r '.record_type' "$file")" \
        --arg pn "$(jq -r '.patient_name' "$file")" \
        --arg dt "$(jq -r '.date // empty' "$file")" \
        --arg now "$now" \
        '{
          schema_version: "4",
          record_id: $rid,
          patient_id: $pid,
          record_type: $rt,
          patient_name: $pn,
          status: "migrated",
          created_at: $now,
          migrated_from: "legacy_v1",
          original_date: $dt,
          metadata: {}
        }'
      ;;
    legacy_v2)
      # Map legacy v2 fields — metadata already exists, promote it
      jq -n \
        --arg rid "migrated-$(date +%s)-$RANDOM" \
        --arg pid "$(jq -r '.patient_id' "$file")" \
        --arg rt "$(jq -r '.metadata.record_type // "unknown"' "$file")" \
        --arg now "$now" \
        --slurpfile meta <(jq '.metadata // {}' "$file") \
        '{
          schema_version: "4",
          record_id: $rid,
          patient_id: $pid,
          record_type: $rt,
          status: "migrated",
          created_at: $now,
          migrated_from: "legacy_v2",
          metadata: $meta[0]
        }'
      ;;
    current)
      # Already in current format — pass through
      cat "$file"
      ;;
  esac
}

# ── Main ─────────────────────────────────────────────────────────────────────

echo ""
echo "======================================="
echo "  Legacy Medical Record Migrator"
echo "======================================="
echo ""
log_info "Input directory:  $INPUT_DIR"
log_info "Output directory: $OUTPUT_DIR"
log_info "Dry run:          $DRY_RUN"
echo ""

if [ ! -d "$INPUT_DIR" ]; then
  log_error "Input directory does not exist: $INPUT_DIR"
  echo "  Create it and place legacy JSON files there, or use --input DIR."
  exit 1
fi

if [ "$DRY_RUN" = false ]; then
  mkdir -p "$OUTPUT_DIR"
fi

# Initialize summary report
{
  echo "Migration Summary Report"
  echo "========================"
  echo "Date: $(date -u +"%Y-%m-%d %H:%M:%S UTC")"
  echo "Input:  $INPUT_DIR"
  echo "Output: $OUTPUT_DIR"
  echo "Dry run: $DRY_RUN"
  echo ""
} > "$SUMMARY_FILE"

# Process each JSON file
shopt -s nullglob
for file in "$INPUT_DIR"/*.json; do
  TOTAL=$((TOTAL + 1))
  filename="$(basename "$file")"
  log_info "Processing: $filename"

  # Detect format
  FORMAT=$(detect_format "$file")
  case "$FORMAT" in
    legacy_v1) LEGACY_V1=$((LEGACY_V1 + 1)); log_info "  Format: legacy_v1" ;;
    legacy_v2) LEGACY_V2=$((LEGACY_V2 + 1)); log_info "  Format: legacy_v2" ;;
    current)   CURRENT=$((CURRENT + 1));     log_info "  Format: current" ;;
    unknown)   UNKNOWN=$((UNKNOWN + 1));     log_warn "  Format: UNKNOWN — skipping"; SKIPPED=$((SKIPPED + 1)); continue ;;
  esac

  # Validate
  if ! validate_record "$file" "$FORMAT"; then
    FAILED=$((FAILED + 1))
    echo "  FAIL: $filename (validation error)" >> "$SUMMARY_FILE"
    continue
  fi
  log_ok "  Validation passed"

  # Transform (even in dry-run to validate the transform works)
  OUTPUT=$(transform_record "$file" "$FORMAT" 2>&1) || {
    log_error "  Transform failed"
    FAILED=$((FAILED + 1))
    echo "  FAIL: $filename (transform error)" >> "$SUMMARY_FILE"
    continue
  }

  # Validate output JSON
  if ! echo "$OUTPUT" | jq empty 2>/dev/null; then
    log_error "  Output is invalid JSON"
    FAILED=$((FAILED + 1))
    echo "  FAIL: $filename (output JSON invalid)" >> "$SUMMARY_FILE"
    continue
  fi

  # Write output
  if [ "$DRY_RUN" = false ]; then
    OUTPUT_FILE="$OUTPUT_DIR/migrated_${filename}"
    echo "$OUTPUT" | jq '.' > "$OUTPUT_FILE"
    log_ok "  Written: $(basename "$OUTPUT_FILE")"
  else
    log_ok "  Dry-run: transform succeeded (not writing)"
  fi

  MIGRATED=$((MIGRATED + 1))
  echo "  OK: $filename ($FORMAT)" >> "$SUMMARY_FILE"
done
shopt -u nullglob

# Write summary
cat >> "$SUMMARY_FILE" <<EOF

Results
-------
Total files scanned:     $TOTAL
  Legacy v1 detected:    $LEGACY_V1
  Legacy v2 detected:    $LEGACY_V2
  Current format:        $CURRENT
  Unknown format:        $UNKNOWN
Successfully migrated:   $MIGRATED
Failed:                  $FAILED
Skipped:                 $SKIPPED
EOF

echo ""
echo "======================================="
echo "  Migration Complete"
echo "======================================="
echo ""
log_info "Total files scanned:   $TOTAL"
log_ok   "Successfully migrated: $MIGRATED"
if [ "$FAILED" -gt 0 ]; then
  log_error "Failed:                $FAILED"
fi
if [ "$SKIPPED" -gt 0 ]; then
  log_warn "Skipped:               $SKIPPED"
fi
log_info "Summary report: $SUMMARY_FILE"
echo ""

if [ "$FAILED" -gt 0 ]; then
  exit 1
fi

exit 0
