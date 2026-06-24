#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
export CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-/tmp/uzima-contracts-target}"

CONTRACTS=("$@")
if [ ${#CONTRACTS[@]} -eq 0 ]; then
  CONTRACTS=(cross_chain_bridge governor)
fi

run_contract() {
  local contract="$1"
  local manifest=""
  case "$contract" in
    medical_records)
      echo "medical_records is excluded from the workspace in Cargo.toml and does not compile standalone today; skipping." >&2
      echo "Run this target after the deferred medical_records compile debt is resolved." >&2
      return 0
      ;;
    cross_chain_bridge)
      manifest="$ROOT_DIR/contracts/cross_chain_bridge/Cargo.toml"
      ;;
    governor)
      manifest="$ROOT_DIR/contracts/governor/Cargo.toml"
      ;;
    *)
      echo "Unknown contract: $contract" >&2
      return 1
      ;;
  esac

  echo
  echo "=== $contract ==="
  local output
  if ! output=$(cargo test --manifest-path "$manifest" bench_storage_ -- --nocapture 2>&1); then
    printf '%s\n' "$output"
    return 1
  fi
  printf '%s\n' "$output"

  echo
  echo "Summary ($contract)"
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
      printf("  %-48s before=%-10s after=%-10s saved=%-10s reduction=%s%%\n", name, before, after, saved, reduction)
    }
  '
}

for contract in "${CONTRACTS[@]}"; do
  run_contract "$contract"
done
