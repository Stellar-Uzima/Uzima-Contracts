#!/usr/bin/env bash
# Regenerate the "Current Storage Leaderboard" table in README.md from
# scripts/measure_storage.sh's PR-comment output (reports/storage_pr_comment.txt),
# which already contains a top-5-by-cost markdown table.
#
# Usage:
#   ./scripts/measure_storage.sh                       # generates reports/storage_pr_comment.txt
#   ./scripts/update_readme_storage_leaderboard.sh      # splices it into README.md
#
# Exit codes:
#   0 — README.md updated (or already up to date)
#   1 — reports/storage_pr_comment.txt missing, or README.md has no
#       STORAGE_LEADERBOARD markers to splice into

set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
PR_COMMENT_FILE="$ROOT_DIR/reports/storage_pr_comment.txt"
README_FILE="$ROOT_DIR/README.md"
START_MARKER="<!-- STORAGE_LEADERBOARD_START -->"
END_MARKER="<!-- STORAGE_LEADERBOARD_END -->"

if [[ ! -f "$PR_COMMENT_FILE" ]]; then
  echo "FATAL: $PR_COMMENT_FILE not found — run ./scripts/measure_storage.sh first." >&2
  exit 1
fi

if ! grep -qF "$START_MARKER" "$README_FILE" || ! grep -qF "$END_MARKER" "$README_FILE"; then
  echo "FATAL: README.md is missing the STORAGE_LEADERBOARD markers." >&2
  exit 1
fi

# Extract just the markdown table (header + separator + data rows) from the
# PR-comment file, dropping its own "### Storage Budget Measurement" title
# and any trailing prose (violation warnings, footnote).
table="$(awk '/^\| # \|/{flag=1} flag{print} /^\|---/{next} flag && /^$/{exit}' "$PR_COMMENT_FILE")"

if [[ -z "$table" ]]; then
  echo "FATAL: could not find a markdown table in $PR_COMMENT_FILE" >&2
  exit 1
fi

# Re-insert the separator row (awk skipped it above to detect table end).
header_line="$(head -1 <<<"$table")"
data_lines="$(tail -n +2 <<<"$table")"
new_block="$START_MARKER
$header_line
|---|----------|----------|-------------|-----------------|--------|
$data_lines
$END_MARKER"

tmp_file="$(mktemp)"
awk -v start="$START_MARKER" -v end="$END_MARKER" -v block="$new_block" '
  $0 == start { print block; skipping = 1; next }
  $0 == end { skipping = 0; next }
  skipping { next }
  { print }
' "$README_FILE" > "$tmp_file"

mv "$tmp_file" "$README_FILE"
echo "README.md storage leaderboard updated from $PR_COMMENT_FILE."
