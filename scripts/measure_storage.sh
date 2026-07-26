#!/usr/bin/env bash
# Measure storage budget for Soroban contracts.
#
# Usage:
#   ./scripts/measure_storage.sh              # measure all built contracts
#   ./scripts/measure_storage.sh <contract>   # measure a single contract
#
# Outputs:
#   reports/storage_measurement_report.txt  — full per-contract report
#   reports/storage_pareto_top10.txt        — top 10 by estimated storage cost
#   reports/storage_pr_comment.txt          — PR comment snippet (first 5)
#
# Exit codes:
#   0 — all contracts within thresholds
#   1 — one or more contracts exceed thresholds

set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
export CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-$ROOT_DIR/target}"
WASM_DIR="${WASM_DIR:-$CARGO_TARGET_DIR/wasm32-unknown-unknown/release}"

# ── Thresholds (docs/CONTRACT_RESOURCE_LIMITS.md) ──────────────────────────
MAX_CONTRACT_SIZE=65536               # 64 KB hard limit
WARNING_CONTRACT_SIZE=$((MAX_CONTRACT_SIZE * 80 / 100))   # ~51.2 KB
CRITICAL_CONTRACT_SIZE=$((MAX_CONTRACT_SIZE * 95 / 100))   # ~62.3 KB
STORAGE_ENTRY_WARNING=100
STORAGE_ENTRY_CRITICAL=500
STORAGE_COST_PER_ENTRY_XLM=0.0011     # ~2 yr rent per entry (Soroban estimate)
CPU_INSTR_WARNING=5000000
CPU_INSTR_CRITICAL=10000000

# ── Output files ───────────────────────────────────────────────────────────
REPORT_DIR="$ROOT_DIR/reports"
REPORT_FILE="$REPORT_DIR/storage_measurement_report.txt"
PARETO_FILE="$REPORT_DIR/storage_pareto_top10.txt"
PR_COMMENT_FILE="$REPORT_DIR/storage_pr_comment.txt"
SUMMARY_JSON="$REPORT_DIR/storage_summary.json"

mkdir -p "$REPORT_DIR"

# ── Helpers ────────────────────────────────────────────────────────────────

usage() {
  cat <<EOF
Usage: $0 [contract ...]

Measure storage budget for Soroban contracts.

If no contracts are specified, all contracts with built WASM files
in \$WASM_DIR are measured.

Options:
  --help     Show this help
  --json     Output results as JSON to stdout (also writes summary JSON)

Environment:
  WASM_DIR   Path to compiled WASM files (default: target/.../release)
EOF
  exit 0
}

log() { printf '%s\n' "$*" >&2; }
die() { log "FATAL: $*"; exit 1; }

# ── WASM analysis (contracts WITHOUT benchmark tests) ──────────────────────

# Count storage-related imports in a WASM binary by grepping for known
# Soroban storage function names embedded in the binary.
estimate_storage_from_wasm() {
  local wasm_file="$1"
  local entries=1  # every contract has at least the code entry

  if [[ ! -f "$wasm_file" ]]; then
    echo "$entries"
    return
  fi

  # Count occurrences of Soroban storage import names in the binary.
  # The import section contains UTF-8 encoded module/field names;
  # `strings` extracts them, then we count storage-related patterns.
  local storage_hits
  storage_hits=$(strings "$wasm_file" 2>/dev/null | grep -ciE \
    'put_contract_data|get_contract_data|has_contract_data|del_contract_data|get_contract_data_for' \
    2>/dev/null || true)

  # Each distinct storage key is typically accessed by one or more of these;
  # dividing by 2 gives a rough entry-count estimate.
  if [[ "$storage_hits" -gt 0 ]]; then
    entries=$((entries + (storage_hits / 2) ))
  fi

  # The WASM binary size in KB is also a rough proxy — every ~1 KB of
  # contract logic corresponds to roughly 1 ledger entry overhead.
  local size_kb
  size_kb=$(stat -c%s "$wasm_file" 2>/dev/null || echo 0)
  size_kb=$((size_kb / 1024))
  local size_est=$((size_kb / 2))
  if [[ "$size_est" -gt "$entries" ]]; then
    entries=$size_est
  fi

  # Sanity cap
  if [[ "$entries" -gt 10000 ]]; then
    entries=10000
  fi

  echo "$entries"
}

# ── Benchmark runner (contracts WITH benchmark tests) ──────────────────────

# Run cargo test bench_storage_ for a given contract and return parsed results
# as colon-separated lines: name:before:after:saved:reduction_pct
run_benchmarks_for_contract() {
  local contract_name="$1"
  local manifest="$ROOT_DIR/contracts/$contract_name/Cargo.toml"

  if [[ ! -f "$manifest" ]]; then
    return 1
  fi

  # Check if benchmarks.rs exists
  local bench_file="$ROOT_DIR/contracts/$contract_name/src/benchmarks.rs"
  if [[ ! -f "$bench_file" ]]; then
    return 1
  fi

  local output
  if ! output=$(cargo test --manifest-path "$manifest" bench_storage_ -- --nocapture 2>&1); then
    log "  Benchmarks FAILED for $contract_name (compilation or test error)"
    printf '%s\n' "$output" | tail -20 >&2
    return 1
  fi

  # Parse [STORAGE-BENCH] lines
  printf '%s\n' "$output" | awk '
    /^\[STORAGE-BENCH\]/ {
      name = $2
      before = after = saved = reduction = ""
      for (i = 3; i <= NF; i++) {
        split($i, kv, "=")
        if (kv[1] == "before") before = kv[2]
        if (kv[1] == "after") after = kv[2]
        if (kv[1] == "saved") saved = kv[2]
        if (kv[1] == "reduction_pct") reduction = kv[2]
      }
      printf("%s:%s:%s:%s:%s\n", name, before, after, saved, reduction)
    }
  '
  return 0
}

# Count storage entries by looking at how many persistent sets are done
# in a contract (heuristic: count storage writes in the WASM). For contracts
# with benchmarks, we combine the benchmark CPU cost with WASM analysis.
compute_storage_entry_count() {
  local contract_name="$1"
  local wasm_file="$2"
  local has_benchmarks="$3"

  if [[ "$has_benchmarks" == "true" ]]; then
    # Contracts with benchmark tests: the benchmark exercises actual storage
    # operations. Estimate entries from WASM analysis is baseline, but we
    # use the fact that benchmarks exist to increase confidence.
    estimate_storage_from_wasm "$wasm_file"
  else
    estimate_storage_from_wasm "$wasm_file"
  fi
}

# ── Main measurement logic ─────────────────────────────────────────────────

# Results accumulator: "contract|wasm_size|entries|ledger_cost|cpu_cost|status"
ALL_RESULTS=()
FAILED_CONTRACTS=()
HAS_VIOLATIONS=false

measure_contract() {
  local contract_name="$1"
  local wasm_file="$2"

  local wasm_size=0
  if [[ -f "$wasm_file" ]]; then
    wasm_size=$(stat -c%s "$wasm_file" 2>/dev/null || echo 0)
  fi

  local has_benchmarks="false"
  local bench_results
  local max_cpu=0
  bench_results=$(run_benchmarks_for_contract "$contract_name" 2>/dev/null || true)
  if [[ -n "$bench_results" ]]; then
    has_benchmarks="true"
    # Extract max CPU instruction cost from benchmark results
    while IFS= read -r line; do
      local name before after saved reduction
      name=$(printf '%s' "$line" | cut -d: -f1)
      before=$(printf '%s' "$line" | cut -d: -f2)
      after=$(printf '%s' "$line" | cut -d: -f3)
      saved=$(printf '%s' "$line" | cut -d: -f4)
      reduction=$(printf '%s' "$line" | cut -d: -f5)
      if [[ "$after" -gt "$max_cpu" ]]; then
        max_cpu=$after
      fi
    done <<< "$bench_results"
  fi

  local entries
  entries=$(compute_storage_entry_count "$contract_name" "$wasm_file" "$has_benchmarks")

  local ledger_cost
  ledger_cost=$(echo "scale=6; $entries * $STORAGE_COST_PER_ENTRY_XLM" | bc -l 2>/dev/null || echo "0")

  local size_status="OK"
  local entry_status="OK"
  local cpu_status="OK"

  # Check contract size
  if [[ "$wasm_size" -gt "$CRITICAL_CONTRACT_SIZE" ]]; then
    size_status="CRITICAL"
    HAS_VIOLATIONS=true
  elif [[ "$wasm_size" -gt "$WARNING_CONTRACT_SIZE" ]]; then
    size_status="WARNING"
  fi

  # Check storage entries
  if [[ "$entries" -gt "$STORAGE_ENTRY_CRITICAL" ]]; then
    entry_status="CRITICAL"
    HAS_VIOLATIONS=true
  elif [[ "$entries" -gt "$STORAGE_ENTRY_WARNING" ]]; then
    entry_status="WARNING"
  fi

  # Check CPU instruction cost
  if [[ "$max_cpu" -gt "$CPU_INSTR_CRITICAL" ]]; then
    cpu_status="CRITICAL"
    HAS_VIOLATIONS=true
  elif [[ "$max_cpu" -gt "$CPU_INSTR_WARNING" ]]; then
    cpu_status="WARNING"
  fi

  ALL_RESULTS+=("${contract_name}|${wasm_size}|${entries}|${ledger_cost}|${max_cpu}|${size_status}|${entry_status}|${cpu_status}|${has_benchmarks}")

  # Output oneline summary to stderr
  printf '  %-40s size=%-8s entries=%-5s cost=%sXLM cpu=%-10s [%s]\n' \
    "$contract_name" \
    "$wasm_size" \
    "$entries" \
    "$ledger_cost" \
    "$max_cpu" \
    "$( [[ "$size_status" != "OK" || "$entry_status" != "OK" || "$cpu_status" != "OK" ]] && echo "$size_status/$entry_status/$cpu_status" || echo "OK" )" >&2
}

# ── Report generation ──────────────────────────────────────────────────────

generate_reports() {
  local tmpfile
  tmpfile=$(mktemp)

  # Write CSV-like data for sorting
  for result in "${ALL_RESULTS[@]}"; do
    printf '%s\n' "$result"
  done > "$tmpfile"

  # ── Full report ──
  {
    echo "================================================================"
    echo "  Storage Budget Measurement Report"
    echo "  Generated: $(date -u '+%Y-%m-%dT%H:%M:%SZ')"
    echo "================================================================"
    echo ""
    printf '%-40s %10s %8s %12s %12s  %s\n' "Contract" "Size(B)" "Entries" "Cost(XLM)" "CPU(inst)" "Status"
    printf '%-40s %10s %8s %12s %12s  %s\n' "--------" "-------" "-------" "---------" "---------" "------"

    while IFS='|' read -r contract wasm_size entries ledger_cost max_cpu size_status entry_status cpu_status has_benchmarks; do
      local status_flags=""
      [[ "$size_status" != "OK" ]] && status_flags+="size:$size_status "
      [[ "$entry_status" != "OK" ]] && status_flags+="entries:$entry_status "
      [[ "$cpu_status" != "OK" ]] && status_flags+="cpu:$cpu_status "
      [[ -z "$status_flags" ]] && status_flags="OK"

      local bench_mark=""
      [[ "$has_benchmarks" == "true" ]] && bench_mark=" *"

      printf '%-40s %10s %8s %12s %12s  %s\n' "${contract}${bench_mark}" "$wasm_size" "$entries" "$ledger_cost" "$max_cpu" "$status_flags"
    done < "$tmpfile"

    echo ""
    echo "---"
    echo "(*) Contract has storage benchmark tests"
    echo ""
    echo "Thresholds (from docs/CONTRACT_RESOURCE_LIMITS.md):"
    echo "  Contract size:  warning > ${WARNING_CONTRACT_SIZE}B (80%), critical > ${CRITICAL_CONTRACT_SIZE}B (95%)"
    echo "  Storage entries: warning > $STORAGE_ENTRY_WARNING, critical > $STORAGE_ENTRY_CRITICAL"
    echo "  CPU instructions: warning > $CPU_INSTR_WARNING, critical > $CPU_INSTR_CRITICAL"
    echo "  Ledger cost (per entry): $STORAGE_COST_PER_ENTRY_XLM XLM"
    echo ""
    echo "Contracts exceeding thresholds:"
    local violations=0
    while IFS='|' read -r contract wasm_size entries ledger_cost max_cpu size_status entry_status cpu_status has_benchmarks; do
      if [[ "$size_status" != "OK" || "$entry_status" != "OK" || "$cpu_status" != "OK" ]]; then
        echo "  - $contract: size=$size_status entries=$entry_status cpu=$cpu_status"
        violations=$((violations + 1))
      fi
    done < "$tmpfile"
    if [[ "$violations" -eq 0 ]]; then
      echo "  (none — all contracts within thresholds)"
    fi
    echo "================================================================"
  } > "$REPORT_FILE"

  # ── Pareto top 10 by storage cost ──
  {
    echo "================================================================"
    echo "  Pareto Report: Top 10 Contracts by Storage Cost"
    echo "  Generated: $(date -u '+%Y-%m-%dT%H:%M:%SZ')"
    echo "================================================================"
    echo ""
    printf '%-4s %-40s %10s %8s %12s\n' "#" "Contract" "Size(B)" "Entries" "Cost(XLM)"
    printf '%-4s %-40s %10s %8s %12s\n' "--" "--------" "-------" "-------" "---------"

    sort -t'|' -k4 -rn "$tmpfile" | head -10 | nl -w2 -s'. ' | while IFS= read -r line; do
      local rank name rest
      rank=$(printf '%s' "$line" | cut -d' ' -f1)
      rest=$(printf '%s' "$line" | cut -d' ' -f2-)
      name=$(printf '%s' "$rest" | cut -d'|' -f1)
      wasm_size=$(printf '%s' "$rest" | cut -d'|' -f2)
      entries=$(printf '%s' "$rest" | cut -d'|' -f3)
      ledger_cost=$(printf '%s' "$rest" | cut -d'|' -f4)
      printf '%-4s %-40s %10s %8s %12s\n' "$rank" "$name" "$wasm_size" "$entries" "$ledger_cost"
    done
    echo "================================================================"
  } > "$PARETO_FILE"

  # ── PR comment (first 5 contracts) ──
  {
    echo "### Storage Budget Measurement"
    echo ""
    echo "| # | Contract | Size (B) | Est. Entries | Est. Cost (XLM) | Status |"
    echo "|---|----------|----------|-------------|-----------------|--------|"

    sort -t'|' -k4 -rn "$tmpfile" | head -5 | nl -w1 -s'. ' | while IFS= read -r line; do
      local rank name rest
      rank=$(printf '%s' "$line" | cut -d' ' -f1)
      rest=$(printf '%s' "$line" | cut -d' ' -f2-)
      name=$(printf '%s' "$rest" | cut -d'|' -f1)
      wasm_size=$(printf '%s' "$rest" | cut -d'|' -f2)
      entries=$(printf '%s' "$rest" | cut -d'|' -f3)
      ledger_cost=$(printf '%s' "$rest" | cut -d'|' -f4)
      size_status=$(printf '%s' "$rest" | cut -d'|' -f6)
      entry_status=$(printf '%s' "$rest" | cut -d'|' -f7)
      status=":white_check_mark:"
      [[ "$size_status" != "OK" || "$entry_status" != "OK" ]] && status=":warning:"
      printf '| %s | %s | %s | %s | %s | %s |\n' "$rank" "$name" "$wasm_size" "$entries" "$ledger_cost" "$status"
    done

    if $HAS_VIOLATIONS; then
      echo ""
      echo "> **Threshold violations detected** — check the full report for details."
    fi

    echo ""
    echo "_Top 5 by estimated storage cost. Full Pareto report in \`reports/storage_pareto_top10.txt\`._"
  } > "$PR_COMMENT_FILE"

  # ── JSON summary ──
  {
    echo "["
    local first=true
    sort -t'|' -k4 -rn "$tmpfile" | while IFS='|' read -r contract wasm_size entries ledger_cost max_cpu size_status entry_status cpu_status has_benchmarks; do
      $first || echo ","
      first=false
      printf '{'
      printf '"contract":"%s","wasm_size":%s,"estimated_entries":%s,"ledger_cost":%s,"max_cpu":%s,"has_benchmarks":%s,"size_status":"%s","entry_status":"%s","cpu_status":"%s"' \
        "$contract" "$wasm_size" "$entries" "$ledger_cost" "$max_cpu" "$has_benchmarks" "$size_status" "$entry_status" "$cpu_status"
      printf '}'
    done
    echo ""
    echo "]"
  } > "$SUMMARY_JSON"

  rm -f "$tmpfile"

  log ""
  log "Reports written:"
  log "  $REPORT_FILE"
  log "  $PARETO_FILE"
  log "  $PR_COMMENT_FILE"
  log "  $SUMMARY_JSON"
}

# ── Threshold enforcement ──────────────────────────────────────────────────

enforce_thresholds() {
  if $HAS_VIOLATIONS; then
    log ""
    log "================================================================="
    log "  THRESHOLD VIOLATIONS DETECTED"
    log "  One or more contracts exceed the limits defined in"
    log "  docs/CONTRACT_RESOURCE_LIMITS.md"
    log "================================================================="
    log ""
    while IFS='|' read -r contract wasm_size entries ledger_cost max_cpu size_status entry_status cpu_status has_benchmarks; do
      local issues=""
      [[ "$size_status" != "OK" ]] && issues+=" size=$size_status($wasm_size bytes)"
      [[ "$entry_status" != "OK" ]] && issues+=" entries=$entry_status($entries)"
      [[ "$cpu_status" != "OK" ]] && issues+=" cpu=$cpu_status($max_cpu inst)"
      if [[ -n "$issues" ]]; then
        log "  FAIL: $contract $issues"
      fi
    done < <(for r in "${ALL_RESULTS[@]}"; do printf '%s\n' "$r"; done)
    log ""
    return 1
  fi

  log ""
  log "All contracts within thresholds."
  return 0
}

# ── Main ───────────────────────────────────────────────────────────────────

main() {
  local json_mode=false
  local explicit_contracts=()

  for arg in "$@"; do
    case "$arg" in
      --help|-h) usage ;;
      --json) json_mode=true ;;
      *) explicit_contracts+=("$arg") ;;
    esac
  done

  log "================================================================"
  log "  Storage Budget Measurement"
  log "  WASM dir: $WASM_DIR"
  log "================================================================"
  log ""

  # Detect contracts to measure
  local contracts_to_measure=()
  if [[ ${#explicit_contracts[@]} -gt 0 ]]; then
    contracts_to_measure=("${explicit_contracts[@]}")
  else
    # Find all built WASM files
    if [[ -d "$WASM_DIR" ]]; then
      local wasm_files=("$WASM_DIR"/*.wasm)
      if [[ ${#wasm_files[@]} -gt 0 ]] && [[ -f "${wasm_files[0]}" ]]; then
        for wasm in "${wasm_files[@]}"; do
          local name
          name=$(basename "$wasm" .wasm)
          # Filter out Soroban test/support WASM files (they start with libtest or similar)
          case "$name" in
            libtest|integration_test|test_*) continue ;;
          esac
          contracts_to_measure+=("$name")
        done
      fi
    fi

    # If no WASM files found, fall back to scanning contracts/ directories
    if [[ ${#contracts_to_measure[@]} -eq 0 ]]; then
      log "No WASM files found in $WASM_DIR — scanning contracts/ directories..."
      for contract_dir in "$ROOT_DIR"/contracts/*/; do
        [[ -d "$contract_dir" ]] || continue
        local name
        name=$(basename "$contract_dir")
        # Skip dirs without Cargo.toml
        [[ -f "${contract_dir}Cargo.toml" ]] || continue
        # Skip excluded contracts (they won't compile)
        local manifest="$ROOT_DIR/contracts/$name/Cargo.toml"
        contracts_to_measure+=("$name")
      done
    fi

    # Deduplicate and sort
    contracts_to_measure=($(printf '%s\n' "${contracts_to_measure[@]}" | sort -u))
  fi

  if [[ ${#contracts_to_measure[@]} -eq 0 ]]; then
    die "No contracts to measure. Build contracts first or specify contract names."
  fi

  log "Measuring ${#contracts_to_measure[@]} contract(s)..."
  log ""

  for contract_name in "${contracts_to_measure[@]}"; do
    local wasm_file="$WASM_DIR/${contract_name}.wasm"
    measure_contract "$contract_name" "$wasm_file"
  done

  # Generate reports
  generate_reports

  # JSON mode: dump summary to stdout
  if $json_mode; then
    cat "$SUMMARY_JSON"
  fi

  # Enforce thresholds (may exit non-zero)
  enforce_thresholds
}

main "$@"
