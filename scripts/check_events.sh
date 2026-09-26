#!/usr/bin/env bash
# Audits contracts/*/src/**/*.rs for state-changing pub fn that do not emit
# a Soroban event via env.events().publish(...).
#
# Enforcement: SECURITY_CHECKLIST item 5 — every state-changing operation must
# emit a corresponding event.  Legacy functions that predate this requirement
# are listed in scripts/allowlists/event_emission.txt.
#
# Scope: every non-test Rust source under contracts/*/src/, not just lib.rs
# (#1587). Test modules define helper functions that never publish events by
# design, so including them would bury the real findings in noise.
#
# Exit codes:
#   0 — all state-changing pub fns emit events, and the allowlist carries no
#       obsolete entries
#   1 — one or more violations found
#   2 — the allowlist or the source tree could not be read

# NOTE: keep this script POSIX-ish enough to run under bash 3.2, which is what
# /bin/bash is on macOS. An earlier version used `declare -A` (bash 4.0+) and
# so failed to run at all on the platform many contributors develop on.

set -euo pipefail

ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
CONTRACTS_DIR="$ROOT_DIR/contracts"
# Kept as a literal relative path rather than stripping $ROOT_DIR out of
# ALLOWLIST_FILE with ${var#"$ROOT_DIR"/}: bash 3.2 cannot parse quotes nested
# inside a parameter expansion.
ALLOWLIST_REL="scripts/allowlists/event_emission.txt"
ALLOWLIST_FILE="$ROOT_DIR/$ALLOWLIST_REL"

# ---------------------------------------------------------------------------
# Read-only function prefixes — functions whose names begin with any of these
# are skipped (they do not mutate state and need no event).
# ---------------------------------------------------------------------------
READONLY_REGEX='^(get_|is_|has_|query_|view_)'

# ---------------------------------------------------------------------------
# Test sources, excluded from the audit.
#
# Matched as a shell glob against the path relative to contracts/, e.g.
# `foo/src/test.rs`. Covers `*_tests.rs`, `test.rs`, `test_*.rs` and the
# standalone harness crates, while leaving real modules such as
# `cross_chain_bridge/src/reorg_protection.rs` in scope.
# ---------------------------------------------------------------------------
TEST_GLOB_EXCLUDES=(
    '*/src/test.rs'
    '*/src/test_*.rs'
    '*/src/*_tests.rs'
    '*/test-helpers/*'
    '*/load_testing/*'
)

# ---------------------------------------------------------------------------
# AWK program: parses one Rust source file and prints every `pub fn` it finds
# as `name<TAB>has_event`, where has_event is 1 if the body contains
# `.events()`.
#
# Reporting every function rather than only the violators is what lets the
# shell below detect obsolete allowlist entries — a function that has since
# been fixed is still in the allowlist and can only be noticed if we also look
# at the ones that are already compliant.
#
# Algorithm:
#   • A line matching a `pub fn` at 0, 4 or 8 spaces of indent marks a new
#     entrypoint: 4 spaces for a free function inside a `mod`, 8 for a method
#     inside an `impl` block, 0 for a module-level helper. The previous version
#     matched 4 spaces only and so missed every impl-block method. The three
#     alternatives are spelled out rather than collapsed into /^ *pub fn/,
#     which this awk rejects with a syntax error.
#   • Brace depth tracking (naive, suitable for no_std Soroban code which
#     has no format!("{}", …) or other string-embedded braces) finds the end
#     of each function body.
#   • `.events()` anywhere in the body sets the "has_event" flag.
#   • At body-close (depth → 0) or at the next `pub fn` header (safety
#     fallback), the function is reported with its current flag.
# ---------------------------------------------------------------------------
AWK_PROG='
BEGIN {
    in_fn    = 0
    fn_name  = ""
    has_ev   = 0
    depth    = 0
    started  = 0
}

# New pub fn entrypoint: 4-space (free fn) or 8-space (impl method) indent.
(/^pub fn [a-z_][a-z0-9_]*/ || /^    pub fn [a-z_][a-z0-9_]*/ || /^        pub fn [a-z_][a-z0-9_]*/) {
    # Flush previous tracked function if brace counting left it open.
    if (in_fn && fn_name != "" && started) {
        print fn_name "\t" has_ev
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
                print fn_name "\t" has_ev
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
    if (in_fn && fn_name != "" && started) print fn_name "\t" has_ev
}
'

# ---------------------------------------------------------------------------
# List the in-scope source files: every contracts/*/src/**/*.rs that is not a
# test source. Sorted so the report is stable between runs.
# ---------------------------------------------------------------------------
list_sources() {
    find "$CONTRACTS_DIR" -path '*/src/*' -name '*.rs' -type f | sort | while read -r file; do
        rel="${file#"$CONTRACTS_DIR"/}"
        skip=0
        for pattern in "${TEST_GLOB_EXCLUDES[@]}"; do
            # shellcheck disable=SC2053  # intentional glob match on rel path
            if [[ "$rel" == $pattern ]]; then
                skip=1
                break
            fi
        done
        if (( skip == 0 )); then
            echo "$file"
        fi
    done
}

if [[ ! -d "$CONTRACTS_DIR" ]]; then
    echo "FAIL: contracts directory not found at ${CONTRACTS_DIR}" >&2
    exit 2
fi
if [[ ! -f "$ALLOWLIST_FILE" ]]; then
    echo "FAIL: allowlist not found at ${ALLOWLIST_FILE}" >&2
    exit 2
fi

# ---------------------------------------------------------------------------
# Main scan loop.
#
# The allowlist is consulted with `grep -Fxq` rather than a bash associative
# array so the script works on bash 3.2.
# ---------------------------------------------------------------------------
violations=0
checked=0
skipped_readonly=0
skipped_allowlisted=0
files_scanned=0
stale=0

# Obsolete allowlist entries accumulate as functions get refactored; collect
# them so the summary can name them.
STALE_LIST=()

# contract::fn -> 1 for every function seen, to detect obsolete entries.
SEEN_KEYS="$(
    mktemp
)"
# Keys already reported. A function name can legitimately appear in more than
# one file of the same contract (a trait method plus an inherent method), and
# the allowlist is keyed by contract::function, so each key is reported once
# and the counts below stay honest.
REPORTED_KEYS="$(
    mktemp
)"
trap 'rm -f "$SEEN_KEYS" "$REPORTED_KEYS"' EXIT

while IFS= read -r src_file; do
    files_scanned=$((files_scanned + 1))
    rel="${src_file#"$CONTRACTS_DIR"/}"
    # First path component below contracts/ is the contract name. Deriving it
    # with basename/dirname instead only works for contracts/<name>/src/*.rs
    # and silently yields "src" for a nested module such as
    # contracts/aml/src/enforcement/rule_enforcer.rs — which then mismatches
    # every allowlist key for that function.
    contract="${rel%%/*}"

    while IFS="$(printf '\t')" read -r fn_name has_ev; do
        [[ -z "$fn_name" ]] && continue

        key="${contract}::${fn_name}"
        printf '%s\n' "$key" >>"$SEEN_KEYS"

        # Process each contract::function once. A name can be defined in more
        # than one file of the same contract (a trait method plus an inherent
        # method), and the allowlist is keyed by contract::function, so the
        # second definition would only duplicate output and skew the counters.
        if grep -Fxq -- "$key" "$REPORTED_KEYS"; then
            continue
        fi
        printf '%s\n' "$key" >>"$REPORTED_KEYS"

        # Filter read-only functions by name prefix.
        if echo "$fn_name" | grep -qE "$READONLY_REGEX"; then
            skipped_readonly=$((skipped_readonly + 1))
            continue
        fi

        allowlisted=0
        if grep -Fxq -- "$key" "$ALLOWLIST_FILE"; then
            allowlisted=1
        fi

        if [[ "$has_ev" == "1" ]]; then
            # Already compliant. Still allowlisted? Then the refactor landed
            # and the entry is dead weight — report it so it can be removed.
            if (( allowlisted == 1 )); then
                STALE_LIST+=("${key}|${rel}|emits an event now")
                stale=$((stale + 1))
            fi
            continue
        fi

        # No event in the body.
        if (( allowlisted == 1 )); then
            skipped_allowlisted=$((skipped_allowlisted + 1))
            continue
        fi

        checked=$((checked + 1))
        echo "FAIL [missing event]: ${rel}  fn ${fn_name}"
        violations=$((violations + 1))
    done < <(awk "$AWK_PROG" "$src_file")
done < <(list_sources)

# ---------------------------------------------------------------------------
# Allowlist entries that name a function which no longer exists. An entry here
# is a permanent silent exemption: the function was deleted, and the entry
# keeps matching nothing while hiding the fact that the exemption is untested.
# ---------------------------------------------------------------------------
missing_entries=0
while IFS= read -r line; do
    [[ -z "$line" || "$line" == \#* ]] && continue
    if ! grep -Fxq -- "$line" "$SEEN_KEYS"; then
        echo "FAIL [stale allowlist]: ${line}  (no such function found)"
        missing_entries=$((missing_entries + 1))
    fi
done <"$ALLOWLIST_FILE"

# ---------------------------------------------------------------------------
# Report
# ---------------------------------------------------------------------------
echo
echo "Event emission audit results:"
echo "  Files scanned (contracts/*/src, non-test):       ${files_scanned}"
echo "  Checked (state-changing, non-allowlisted):      ${checked}"
echo "  Allowlisted (legacy — pending refactor):        ${skipped_allowlisted}"
echo "  Skipped (read-only prefix):                     ${skipped_readonly}"
echo "  Stale allowlist entries (function emits now):   ${stale}"
echo "  Stale allowlist entries (function not found):   ${missing_entries}"
echo

if (( missing_entries > 0 )); then
    echo "FAIL: ${missing_entries} allowlist entr(y/ies) name a function that no longer exists."
    echo "      Remove them from $ALLOWLIST_REL."
    echo
fi

if (( stale > 0 )); then
    echo "Obsolete allowlist entries — these functions now emit events, so the"
    echo "exemption is no longer needed:"
    for entry in "${STALE_LIST[@]}"; do
        key="${entry%%|*}"
        rest="${entry#*|}"
        echo "  ${key}  (${rest})"
    done
    echo
    echo "Remove them from $ALLOWLIST_REL."
    echo
fi

if (( violations > 0 )); then
    echo "FAIL: ${violations} state-changing pub fn(s) found without event emission."
    echo
    echo "Fix options:"
    echo "  1. Add  env.events().publish((...), data)  inside the function body."
    echo "  2. Add  contract::function_name  to $ALLOWLIST_REL"
    echo "     only if the function is a legacy one pending a separate refactor PR."
    exit 1
fi

if (( stale > 0 || missing_entries > 0 )); then
    echo "FAIL: the allowlist has ${stale} obsolete entr(ies); event audit itself is clean."
    exit 1
fi

echo "OK: all non-allowlisted state-changing pub fn(s) emit events."
