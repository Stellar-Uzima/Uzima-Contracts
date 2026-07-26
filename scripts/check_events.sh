#!/usr/bin/env bash
# Audits contracts/*/src/lib.rs for state-changing pub fn that do not emit
# a Soroban event via env.events().publish(...).
#
# Enforcement: SECURITY_CHECKLIST item 5 — every state-changing operation must
# emit a corresponding event.  Legacy functions that predate this requirement
# are listed in scripts/allowlists/event_emission.txt.
#
# Exit codes:
#   0 — all new state-changing pub fns emit events (allowlisted ones are skipped)
#   1 — one or more violations found

set -euo pipefail

ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
CONTRACTS_DIR="$ROOT_DIR/contracts"
ALLOWLIST_FILE="$ROOT_DIR/scripts/allowlists/event_emission.txt"

# ---------------------------------------------------------------------------
# Read-only function prefixes — functions whose names begin with any of these
# are skipped (they do not mutate state and need no event).
# ---------------------------------------------------------------------------
READONLY_REGEX='^(get_|is_|has_|query_|view_)'

# ---------------------------------------------------------------------------
# AWK program: parses a single lib.rs file and prints the name of every
# `pub fn` at 4-space indent whose body does not contain `.events()`.
#
# Algorithm:
#   • A line matching /^    pub fn [a-z_][a-z0-9_]*/ marks a new entrypoint.
#   • Brace depth tracking (naive, suitable for no_std Soroban code which
#     has no format!("{}", …) or other string-embedded braces) finds the end
#     of each function body.
#   • `.events()` anywhere in the body sets the "has_event" flag.
#   • At body-close (depth → 0) or at the next `pub fn` header (safety
#     fallback), the function is emitted if the flag was never set.
# ---------------------------------------------------------------------------
AWK_PROG='
BEGIN {
    in_fn    = 0
    fn_name  = ""
    has_ev   = 0
    depth    = 0
    started  = 0
}

# New pub fn entrypoint (exactly 4 spaces indent).
/^    pub fn [a-z_][a-z0-9_]*/ {
    # Flush previous tracked function if brace counting left it open.
    if (in_fn && fn_name != "" && started && !has_ev) {
        print fn_name
    }
    rest = $0
    sub(/^.*pub fn /, "", rest)
    sub(/[^a-z0-9_].*$/, "", rest)
    fn_name = rest
    has_ev  = 0
    depth   = 0
    started = 0
    in_fn   = 1
}

# Inside a function: track event emission and brace depth.
in_fn {
    if (index($0, ".events()") > 0) has_ev = 1

    n = length($0)
    for (i = 1; i <= n; i++) {
        c = substr($0, i, 1)
        if (c == "{") {
            depth++
            started = 1
        } else if (c == "}" && started) {
            depth--
            if (depth == 0) {
                if (!has_ev) print fn_name
                in_fn   = 0
                fn_name = ""
                has_ev  = 0
                started = 0
                break
            }
        }
    }
}

END {
    if (in_fn && fn_name != "" && started && !has_ev) print fn_name
}
'

# ---------------------------------------------------------------------------
# Load the allowlist into an associative array for O(1) lookup.
# ---------------------------------------------------------------------------
declare -A ALLOWLIST=()
if [[ -f "$ALLOWLIST_FILE" ]]; then
    while IFS= read -r line; do
        # Skip blank lines and comments
        [[ -z "$line" || "$line" == \#* ]] && continue
        ALLOWLIST["$line"]=1
    done < "$ALLOWLIST_FILE"
fi

# ---------------------------------------------------------------------------
# Main scan loop
# ---------------------------------------------------------------------------
violations=0
checked=0
skipped_readonly=0
skipped_allowlisted=0

while IFS= read -r lib_file; do
    contract="$(basename "$(dirname "$(dirname "$lib_file")")")"

    while IFS= read -r fn_name; do
        [[ -z "$fn_name" ]] && continue

        # Filter read-only functions by name prefix.
        if echo "$fn_name" | grep -qE "$READONLY_REGEX"; then
            skipped_readonly=$((skipped_readonly + 1))
            continue
        fi

        checked=$((checked + 1))
        key="${contract}::${fn_name}"

        # Skip legacy functions in the allowlist.
        if [[ -v "ALLOWLIST[$key]" ]]; then
            skipped_allowlisted=$((skipped_allowlisted + 1))
            continue
        fi

        echo "FAIL [missing event]: ${lib_file}  fn ${fn_name}"
        violations=$((violations + 1))
    done < <(awk "$AWK_PROG" "$lib_file")
done < <(find "$CONTRACTS_DIR" -path "*/src/lib.rs" -type f | sort)

# ---------------------------------------------------------------------------
# Report
# ---------------------------------------------------------------------------
echo
echo "Event emission audit results:"
echo "  Checked (state-changing, non-allowlisted): ${checked}"
echo "  Allowlisted (legacy — pending refactor):   ${skipped_allowlisted}"
echo "  Skipped (read-only prefix):                ${skipped_readonly}"
echo

if (( violations > 0 )); then
    echo "FAIL: ${violations} state-changing pub fn(s) found without event emission."
    echo
    echo "Fix options:"
    echo "  1. Add  env.events().publish((...), data)  inside the function body."
    echo "  2. Add  contract::function_name  to ${ALLOWLIST_FILE}"
    echo "     only if the function is a legacy one pending a separate refactor PR."
    exit 1
fi

echo "OK: all non-allowlisted state-changing pub fn(s) emit events."
