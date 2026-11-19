#!/usr/bin/env python3
"""Rewrite the README "Current Storage Leaderboard" table from measurement output.

Reads reports/storage_summary.json (written by scripts/measure_storage.sh) and
replaces the block between the STORAGE_LEADERBOARD_START / _END markers in
README.md with the top-N contracts. Nothing is estimated here: if the input is
missing or empty the script exits non-zero and leaves README.md untouched.

Usage:
  ./scripts/measure_storage.sh
  python3 scripts/generate_storage_leaderboard.py [--top 5] [--by cost|size]
"""
import argparse
import json
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
SUMMARY = ROOT / "reports" / "storage_summary.json"
README = ROOT / "README.md"
START = "<!-- STORAGE_LEADERBOARD_START -->"
END = "<!-- STORAGE_LEADERBOARD_END -->"


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    ap.add_argument("--top", type=int, default=5)
    ap.add_argument("--by", choices=("cost", "size"), default="cost")
    args = ap.parse_args()

    if not SUMMARY.is_file():
        print(f"FATAL: {SUMMARY} not found - build the wasm contracts and run "
              "./scripts/measure_storage.sh first.", file=sys.stderr)
        return 1
    try:
        rows = json.loads(SUMMARY.read_text())
    except json.JSONDecodeError as e:
        print(f"FATAL: {SUMMARY} is not valid JSON: {e}", file=sys.stderr)
        return 1
    if not rows:
        print(f"FATAL: {SUMMARY} contains no contracts.", file=sys.stderr)
        return 1

    key = (lambda r: (r["ledger_cost"], r["wasm_size"])) if args.by == "cost" \
        else (lambda r: (r["wasm_size"], r["ledger_cost"]))
    rows = sorted(rows, key=key, reverse=True)[: args.top]

    lines = [START,
             "| # | Contract | Size (B) | Est. Entries | Est. Cost (XLM) | Status |",
             "|---|----------|----------|-------------|-----------------|--------|"]
    for i, r in enumerate(rows, 1):
        ok = r["size_status"] == "OK" and r["entry_status"] == "OK"
        status = ":white_check_mark:" if ok else ":warning:"
        lines.append(f"| {i} | {r['contract']} | {r['wasm_size']} | "
                     f"{r['estimated_entries']} | {r['ledger_cost']} | {status} |")
    lines.append(END)

    text = README.read_text()
    s, e = text.find(START), text.find(END)
    if s == -1 or e == -1 or e < s:
        print("FATAL: README.md is missing the STORAGE_LEADERBOARD markers.",
              file=sys.stderr)
        return 1
    README.write_text(text[:s] + "\n".join(lines) + text[e + len(END):])
    print(f"README.md leaderboard updated from {SUMMARY} (top {len(rows)} by {args.by}).")
    return 0


if __name__ == "__main__":
    sys.exit(main())
