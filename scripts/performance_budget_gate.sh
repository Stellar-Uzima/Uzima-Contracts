#!/usr/bin/env bash
#
# Performance budget and benchmark gate for Soroban contracts (Issue #1086).
#
# Unifies the three resource dimensions that decide whether a contract can be
# deployed and operated affordably:
#
#   1. WASM size          (from the wasm32 release build)
#   2. Storage entries    (from scripts/measure_storage.sh)
#   3. CPU instructions   (from scripts/measure_storage.sh benchmarks)
#
# against the versioned budget in scripts/performance_budgets.json.
#
# The gate enforces BOTH absolute thresholds and relative regressions, so a
# contract cannot silently drift toward a hard Soroban limit, and necessary
# feature work is not penalised for a small, in-budget increase.
#
# Contracts already over a hard limit when the budget was recorded are
# "grandfathered": they are reported as warnings, and only fail the build if
# they regress further. This keeps the gate actionable instead of permanently
# red — see docs/PERFORMANCE_BUDGETS.md.
#
# Outputs:
#   reports/performance_budget_report.md    — markdown report (PR comment)
#   reports/performance_budget_result.json  — machine-readable result
#   .performance_budget_trends.json         — appended trend history
#
# Usage:
#   bash scripts/performance_budget_gate.sh --check    # CI gate
#   bash scripts/performance_budget_gate.sh --update   # re-record budgets
#   bash scripts/performance_budget_gate.sh --trend    # trend view
#
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

BUDGET_FILE="${BUDGET_FILE:-$ROOT_DIR/scripts/performance_budgets.json}"
CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-$ROOT_DIR/target}"
WASM_DIR="${WASM_DIR:-$CARGO_TARGET_DIR/wasm32-unknown-unknown/release}"
REPORT_DIR="${REPORT_DIR:-$ROOT_DIR/reports}"
STORAGE_SUMMARY="${STORAGE_SUMMARY:-$REPORT_DIR/storage_summary.json}"
REPORT_MD="${REPORT_MD:-$REPORT_DIR/performance_budget_report.md}"
RESULT_JSON="${RESULT_JSON:-$REPORT_DIR/performance_budget_result.json}"
TREND_FILE="${TREND_FILE:-$ROOT_DIR/.performance_budget_trends.json}"
TREND_KEEP="${TREND_KEEP:-50}"

# Set to 1 to report violations without failing the build.
WARN_ONLY="${WARN_ONLY:-0}"

RED='\033[0;31m'; YELLOW='\033[1;33m'; GREEN='\033[0;32m'; BLUE='\033[0;34m'; NC='\033[0m'
log() { echo -e "$1"; }

# ── Dependencies ───────────────────────────────────────────────────────────

require_deps() {
  local missing=()
  command -v jq >/dev/null 2>&1 || missing+=("jq")
  command -v awk >/dev/null 2>&1 || missing+=("awk")
  if [[ ${#missing[@]} -gt 0 ]]; then
    log "${RED}Missing required dependencies: ${missing[*]}${NC}"
    log "${YELLOW}Install with: sudo apt-get install -y ${missing[*]}${NC}"
    exit 1
  fi
  if [[ ! -f "$BUDGET_FILE" ]]; then
    log "${RED}Budget file not found: $BUDGET_FILE${NC}"
    log "${YELLOW}Create it with: bash scripts/performance_budget_gate.sh --update${NC}"
    exit 1
  fi
}

human_bytes() {
  local b="$1"
  if [[ "$b" -ge 1024 ]]; then echo "$((b / 1024)) KB"; else echo "${b} B"; fi
}

# Percentage change from $1 (base) to $2 (current), one decimal place.
pct_change() {
  local base="$1" cur="$2"
  if [[ "$base" -eq 0 ]]; then
    # No meaningful baseline: report 0 rather than a divide-by-zero.
    echo "0.0"
    return
  fi
  awk -v b="$base" -v c="$cur" 'BEGIN { printf "%.1f", (c - b) * 100 / b }'
}

# True when $1 > $2, comparing as decimals.
gt() { awk -v a="$1" -v b="$2" 'BEGIN { exit !(a > b) }'; }

is_excluded() {
  jq -e --arg n "$1" '.excluded | index($n)' "$BUDGET_FILE" >/dev/null 2>&1
}

# ── Measurement collection ─────────────────────────────────────────────────

# Emit "<contract> <bytes>" for each built wasm that is not excluded.
collect_wasm_sizes() {
  local f name
  [[ -d "$WASM_DIR" ]] || return 0
  shopt -s nullglob
  for f in "$WASM_DIR"/*.wasm; do
    name="$(basename "$f" .wasm)"
    is_excluded "$name" && continue
    printf '%s %s\n' "$name" "$(wc -c < "$f")"
  done
  shopt -u nullglob
}

# Look up a field for a contract in the storage summary produced by
# scripts/measure_storage.sh. Echoes an empty string when unavailable, so the
# gate degrades gracefully to size-only when benchmarks have not been run.
storage_field() {
  local contract="$1" field="$2"
  [[ -f "$STORAGE_SUMMARY" ]] || { echo ""; return; }
  jq -r --arg c "$contract" --arg f "$field" \
    'map(select(.contract == $c)) | .[0][$f] // empty' \
    "$STORAGE_SUMMARY" 2>/dev/null || echo ""
}

# ── Budget evaluation ──────────────────────────────────────────────────────

# Evaluate one metric for one contract.
#
#   $1 contract   $2 metric label   $3 baseline   $4 current
#   $5 warn limit $6 hard limit     $7 tolerance% $8 min delta  $9 grandfathered
#
# Sets EVAL_STATUS / EVAL_DETAIL and returns 1 when the metric fails the gate.
EVAL_STATUS=""
EVAL_DETAIL=""
evaluate_metric() {
  local contract="$1" label="$2" base="$3" cur="$4"
  local warn_limit="$5" hard_limit="$6" tol="$7" min_delta="$8" grandfathered="$9"
  local delta pct
  EVAL_STATUS=":white_check_mark: ok"
  EVAL_DETAIL=""

  # No measurement for this dimension (benchmarks absent) — skip, do not fail.
  if [[ -z "$cur" || "$cur" == "null" ]]; then
    EVAL_STATUS=":grey_question: not measured"
    return 0
  fi

  # New contract with no recorded budget: only the hard limit applies.
  if [[ -z "$base" || "$base" == "null" ]]; then
    if [[ "$cur" -gt "$hard_limit" ]]; then
      EVAL_STATUS=":x: new contract over hard limit"
      EVAL_DETAIL="${label} ${cur} exceeds hard limit ${hard_limit}"
      return 1
    fi
    EVAL_STATUS=":new: new contract"
    return 0
  fi

  delta=$((cur - base))
  pct="$(pct_change "$base" "$cur")"

  # Relative regression, ignoring changes below the noise floor.
  if gt "$pct" "$tol" && [[ "$delta" -gt "$min_delta" ]]; then
    EVAL_STATUS=":x: +${pct}% (budget +${tol}%)"
    EVAL_DETAIL="${label} grew ${pct}% (${base} -> ${cur}), over the ${tol}% budget"
    return 1
  fi

  # Newly crossing a hard limit is always a failure.
  if [[ "$cur" -gt "$hard_limit" && "$base" -le "$hard_limit" ]]; then
    EVAL_STATUS=":x: crossed hard limit"
    EVAL_DETAIL="${label} crossed the hard limit ${hard_limit} (now ${cur})"
    return 1
  fi

  # Already over a hard limit when the budget was recorded.
  if [[ "$cur" -gt "$hard_limit" ]]; then
    if [[ "$grandfathered" == "true" ]]; then
      EVAL_STATUS=":warning: over hard limit (grandfathered)"
      return 0
    fi
    EVAL_STATUS=":x: over hard limit"
    EVAL_DETAIL="${label} ${cur} exceeds hard limit ${hard_limit}"
    return 1
  fi

  # Newly crossing the warning threshold is surfaced but does not fail.
  if [[ "$cur" -gt "$warn_limit" && "$base" -le "$warn_limit" ]]; then
    EVAL_STATUS=":warning: crossed warn threshold"
    return 0
  fi
  if [[ "$cur" -gt "$warn_limit" ]]; then
    EVAL_STATUS=":warning: over warn threshold"
    return 0
  fi

  [[ "$delta" -lt 0 ]] && EVAL_STATUS=":arrow_down: ${pct}%"
  return 0
}

# Actionable guidance, keyed on which dimension actually regressed.
recommendations_for() {
  case "$1" in
    wasm_size)
      cat <<'EOF'
- Run `cargo bloat --release --target wasm32-unknown-unknown -n 30` to find the largest functions.
- Check for newly added dependencies or enabled features (`cargo tree -d` for duplicates).
- Prefer `soroban_sdk::String`/`Bytes` over formatting helpers that pull in `core::fmt`.
- Consider `contract_optimizer/` passes, or splitting the contract along a storage boundary.
EOF
      ;;
    storage_entries)
      cat <<'EOF'
- Collapse related keys into a single struct value instead of one entry per field.
- Confirm temporary data uses temporary storage, not persistent, so it does not accrue rent.
- Batch writes: a single `set` of an aggregate is cheaper than N per-field writes.
- Review whether historical/audit rows can be event-emitted instead of stored.
EOF
      ;;
    cpu_instructions)
      cat <<'EOF'
- Look for loops whose bound grows with stored data; cap or paginate them.
- Hoist repeated `storage().get()` calls out of loops into a local.
- Avoid re-serialising large structs on hot paths.
- Re-run `cargo test bench_storage_ -- --nocapture` locally to confirm the delta.
EOF
      ;;
    *) echo "- Review the change against docs/PERFORMANCE_BUDGETS.md." ;;
  esac
}

# ── Commands ───────────────────────────────────────────────────────────────

cmd_update() {
  require_deps
  [[ -d "$WASM_DIR" ]] || {
    log "${RED}WASM dir not found: $WASM_DIR — build first (cargo build --workspace --target wasm32-unknown-unknown --release)${NC}"
    exit 1
  }

  local contracts="{}" name bytes entries cpu tier over
  local hard_size
  hard_size="$(jq -r '.limits.max_wasm_size_bytes' "$BUDGET_FILE")"

  while read -r name bytes; do
    entries="$(storage_field "$name" "estimated_entries")"
    cpu="$(storage_field "$name" "max_cpu")"
    # Preserve the tier if the contract is already tracked.
    tier="$(jq -r --arg n "$name" '.contracts[$n].tier // "standard"' "$BUDGET_FILE")"
    over=false
    [[ "$bytes" -gt "$hard_size" ]] && over=true

    contracts="$(jq \
      --arg n "$name" --argjson b "$bytes" --arg t "$tier" --argjson o "$over" \
      --arg e "${entries:-}" --arg c "${cpu:-}" '
        .[$n] = ({ tier: $t, wasm_size: $b, grandfathered_over_size_limit: $o }
          + (if $e == "" then {} else { storage_entries: ($e | tonumber) } end)
          + (if $c == "" then {} else { cpu_instructions: ($c | tonumber) } end))
      ' <<<"$contracts")"
  done < <(collect_wasm_sizes)

  if [[ "$contracts" == "{}" ]]; then
    log "${RED}No wasm files found in $WASM_DIR${NC}"
    exit 1
  fi

  local tmp
  tmp="$(mktemp)"
  jq --argjson c "$contracts" '.contracts = $c' "$BUDGET_FILE" > "$tmp"
  mv "$tmp" "$BUDGET_FILE"
  log "${GREEN}Recorded budgets for $(jq '.contracts | length' "$BUDGET_FILE") contract(s) in $BUDGET_FILE${NC}"
}

cmd_check() {
  require_deps
  mkdir -p "$REPORT_DIR"

  if [[ ! -d "$WASM_DIR" ]]; then
    log "${RED}WASM dir not found: $WASM_DIR${NC}"
    exit 1
  fi

  local warn_size hard_size warn_entries hard_entries warn_cpu hard_cpu
  local tol_size tol_entries tol_cpu min_size_delta min_cpu_delta
  warn_size="$(jq -r '.limits.warn_wasm_size_bytes' "$BUDGET_FILE")"
  hard_size="$(jq -r '.limits.max_wasm_size_bytes' "$BUDGET_FILE")"
  warn_entries="$(jq -r '.limits.warn_storage_entries' "$BUDGET_FILE")"
  hard_entries="$(jq -r '.limits.max_storage_entries' "$BUDGET_FILE")"
  warn_cpu="$(jq -r '.limits.warn_cpu_instructions' "$BUDGET_FILE")"
  hard_cpu="$(jq -r '.limits.max_cpu_instructions' "$BUDGET_FILE")"
  tol_size="$(jq -r '.regression.wasm_size_pct' "$BUDGET_FILE")"
  tol_entries="$(jq -r '.regression.storage_entries_pct' "$BUDGET_FILE")"
  tol_cpu="$(jq -r '.regression.cpu_instructions_pct' "$BUDGET_FILE")"
  min_size_delta="$(jq -r '.regression.min_wasm_delta_bytes' "$BUDGET_FILE")"
  min_cpu_delta="$(jq -r '.regression.min_cpu_delta_instructions' "$BUDGET_FILE")"

  local fail=0 warns=0 tracked=0 untracked=0
  local rows="" failed_dims="" results="[]"

  local name bytes base_size entries base_entries cpu base_cpu grand
  local s_status e_status c_status row_fail

  while read -r name bytes; do
    base_size="$(jq -r --arg n "$name" '.contracts[$n].wasm_size // empty' "$BUDGET_FILE")"
    if [[ -z "$base_size" ]]; then
      untracked=$((untracked + 1))
    else
      tracked=$((tracked + 1))
    fi
    grand="$(jq -r --arg n "$name" '.contracts[$n].grandfathered_over_size_limit // false' "$BUDGET_FILE")"
    base_entries="$(jq -r --arg n "$name" '.contracts[$n].storage_entries // empty' "$BUDGET_FILE")"
    base_cpu="$(jq -r --arg n "$name" '.contracts[$n].cpu_instructions // empty' "$BUDGET_FILE")"
    entries="$(storage_field "$name" "estimated_entries")"
    cpu="$(storage_field "$name" "max_cpu")"
    row_fail=0

    if evaluate_metric "$name" "wasm_size" "$base_size" "$bytes" \
        "$warn_size" "$hard_size" "$tol_size" "$min_size_delta" "$grand"; then
      s_status="$EVAL_STATUS"
    else
      s_status="$EVAL_STATUS"; row_fail=1
      failed_dims+="wasm_size"$'\n'
      log "${RED}FAIL ${name}: ${EVAL_DETAIL}${NC}"
    fi

    if evaluate_metric "$name" "storage_entries" "$base_entries" "$entries" \
        "$warn_entries" "$hard_entries" "$tol_entries" "0" "false"; then
      e_status="$EVAL_STATUS"
    else
      e_status="$EVAL_STATUS"; row_fail=1
      failed_dims+="storage_entries"$'\n'
      log "${RED}FAIL ${name}: ${EVAL_DETAIL}${NC}"
    fi

    if evaluate_metric "$name" "cpu_instructions" "$base_cpu" "$cpu" \
        "$warn_cpu" "$hard_cpu" "$tol_cpu" "$min_cpu_delta" "false"; then
      c_status="$EVAL_STATUS"
    else
      c_status="$EVAL_STATUS"; row_fail=1
      failed_dims+="cpu_instructions"$'\n'
      log "${RED}FAIL ${name}: ${EVAL_DETAIL}${NC}"
    fi

    [[ "$row_fail" -eq 1 ]] && fail=1
    case "$s_status$e_status$c_status" in *":warning:"*) warns=$((warns + 1));; esac

    # Only tracked contracts and failures go in the table, to keep the PR
    # comment readable across ~80 contracts.
    if [[ -n "$base_size" || "$row_fail" -eq 1 ]]; then
      rows+="| \`${name}\` | $(human_bytes "$bytes") | ${s_status} | ${entries:-–} | ${e_status} | ${cpu:-–} | ${c_status} |"$'\n'
    fi

    results="$(jq --arg n "$name" --argjson b "$bytes" \
      --arg e "${entries:-}" --arg c "${cpu:-}" --argjson f "$row_fail" '
      . += [{ contract: $n, wasm_size: $b, failed: ($f == 1) }
        + (if $e == "" then {} else { storage_entries: ($e | tonumber) } end)
        + (if $c == "" then {} else { cpu_instructions: ($c | tonumber) } end)]
    ' <<<"$results")"
  done < <(collect_wasm_sizes)

  if [[ "$tracked" -eq 0 && "$untracked" -eq 0 ]]; then
    log "${YELLOW}No wasm artifacts found in $WASM_DIR — nothing to gate.${NC}"
    echo "### Performance Budget Report" > "$REPORT_MD"
    echo "" >> "$REPORT_MD"
    echo "No WASM artifacts were found, so no budgets were evaluated." >> "$REPORT_MD"
    return 0
  fi

  local bench_note=""
  [[ -f "$STORAGE_SUMMARY" ]] || bench_note=$'\n> **Note:** `reports/storage_summary.json` was not found, so storage-entry and instruction budgets were not evaluated. Run `bash scripts/measure_storage.sh` to enable the full gate.\n'

  {
    echo "### Performance Budget Report"
    echo
    echo "Unified budget gate for **WASM size**, **storage entries** and **CPU instructions** (Issue #1086)."
    echo "Budgets: \`scripts/performance_budgets.json\` · Docs: \`docs/PERFORMANCE_BUDGETS.md\`"
    echo "$bench_note"
    echo "Gate: fails on a relative regression over budget (size +${tol_size}%, entries +${tol_entries}%, CPU +${tol_cpu}%) or on newly crossing a hard limit. Contracts already over a hard limit are grandfathered (:warning:)."
    echo
    echo "| Contract | Size | Size status | Entries | Entry status | CPU | CPU status |"
    echo "|---|---|---|---|---|---|---|"
    printf '%s' "$rows"
    echo
    echo "**Tracked:** ${tracked} · **Untracked built contracts:** ${untracked} · **Warnings:** ${warns}"
    echo
    if [[ "$fail" -eq 1 ]]; then
      echo "#### ❌ Performance budget gate failed"
      echo
      echo "Recommended actions:"
      echo
      local dim
      while read -r dim; do
        [[ -z "$dim" ]] && continue
        echo "**${dim}**"
        recommendations_for "$dim"
        echo
      done < <(printf '%s' "$failed_dims" | sort -u)
      echo "If the increase is intentional and reviewed, re-record the budget with:"
      echo
      echo '```bash'
      echo 'bash scripts/performance_budget_gate.sh --update'
      echo '```'
    else
      echo "#### ✅ All contracts within their performance budgets"
    fi
  } | tee "$REPORT_MD"

  jq -n --argjson r "$results" --argjson fail "$fail" \
    --argjson tracked "$tracked" --argjson warns "$warns" \
    '{ passed: ($fail == 0), tracked: $tracked, warnings: $warns, contracts: $r }' \
    > "$RESULT_JSON"

  record_trend "$results"

  if [[ "$fail" -eq 1 && "$WARN_ONLY" != "1" ]]; then
    return 1
  fi
  if [[ "$fail" -eq 1 ]]; then
    log "${YELLOW}WARN_ONLY=1 set — violations reported but not failing the build.${NC}"
  fi
  return 0
}

# Append the current measurement set to the trend history, keeping the most
# recent $TREND_KEEP samples so the file does not grow without bound.
record_trend() {
  local results="$1" ts tmp
  ts="$(date -u +%Y-%m-%dT%H:%M:%SZ)"
  [[ -f "$TREND_FILE" ]] || echo '{"samples":[]}' > "$TREND_FILE"
  tmp="$(mktemp)"
  jq --argjson r "$results" --arg ts "$ts" --argjson keep "$TREND_KEEP" \
    '.samples += [{ timestamp: $ts, contracts: $r }] | .samples |= (.[-$keep:])' \
    "$TREND_FILE" > "$tmp" && mv "$tmp" "$TREND_FILE"
}

cmd_trend() {
  require_deps
  if [[ ! -f "$TREND_FILE" ]]; then
    log "${YELLOW}No trend history yet at $TREND_FILE — run --check first.${NC}"
    return 0
  fi
  local count
  count="$(jq '.samples | length' "$TREND_FILE")"
  log "${BLUE}=== Performance Budget Trend (${count} sample(s)) ===${NC}"
  if [[ "$count" -lt 2 ]]; then
    log "${YELLOW}Need at least 2 samples to show a trend.${NC}"
    return 0
  fi
  # Compare the oldest retained sample against the newest, per contract.
  jq -r '
    (.samples[0].contracts   | map({ (.contract): .wasm_size }) | add) as $first |
    (.samples[-1].contracts  | map({ (.contract): .wasm_size }) | add) as $last  |
    ($last | keys_unsorted[]) as $k |
    select($first[$k] != null and $last[$k] != null and $first[$k] != $last[$k]) |
    "\($k)\t\($first[$k])\t\($last[$k])\t\(((($last[$k] - $first[$k]) * 1000 / $first[$k]) | floor) / 10)"
  ' "$TREND_FILE" | sort -k4 -rn | while IFS=$'\t' read -r name a b pct; do
    if gt "$pct" "0"; then
      log "${YELLOW}  ${name}: ${a} -> ${b} bytes (+${pct}%)${NC}"
    else
      log "${GREEN}  ${name}: ${a} -> ${b} bytes (${pct}%)${NC}"
    fi
  done
  log "${BLUE}Full history: $TREND_FILE${NC}"
}

show_help() {
  cat <<EOF
Usage: $0 [--check|--update|--trend|--help]

Unified performance budget gate for Soroban contracts (Issue #1086).
Evaluates WASM size, storage entries and CPU instructions against
scripts/performance_budgets.json.

Options:
  --check    Evaluate the current build against the budgets. Exits 1 on a
             regression over budget or a newly crossed hard limit. Writes
             reports/performance_budget_report.md and
             reports/performance_budget_result.json.
  --update   Re-record budgets from the current build (after an intended,
             reviewed change).
  --trend    Show how tracked contracts have moved across retained samples.
  --help     Show this message.

Environment:
  WASM_DIR          Built .wasm directory (default: target/wasm32-unknown-unknown/release)
  STORAGE_SUMMARY   measure_storage.sh JSON summary (default: reports/storage_summary.json)
  BUDGET_FILE       Budget file (default: scripts/performance_budgets.json)
  WARN_ONLY=1       Report violations without failing the build.

Requirements: jq, awk
EOF
}

case "${1:---check}" in
  --help|-h) show_help ;;
  --update)  cmd_update ;;
  --trend)   cmd_trend ;;
  --check)   if cmd_check; then exit 0; else exit 1; fi ;;
  *) log "${RED}Unknown option: $1${NC}"; show_help; exit 1 ;;
esac
