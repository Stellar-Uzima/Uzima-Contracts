#!/usr/bin/env bash
# Audits contracts/*/src/lib.rs for state-changing pub fn that do not emit
# a Soroban event via env.events().publish(...).
#
# Enforcement: SECURITY_CHECKLIST item 5 — every state-changing operation must
# emit a corresponding event. Legacy functions that predate this requirement
# are listed in scripts/allowlists/event_emission.txt with mandatory issue references.
#
# Exit codes:
#   0 — all new state-changing pub fns emit events (allowlisted ones are skipped)
#   1 — one or more violations found

set -euo pipefail

ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
ALLOWLIST_FILE="${ALLOWLIST_FILE:-${1:-$ROOT_DIR/scripts/allowlists/event_emission.txt}}"
CONTRACTS_DIR="${CONTRACTS_DIR:-${2:-$ROOT_DIR/contracts}}"

# ---------------------------------------------------------------------------
# Read-only function prefixes — functions whose names begin with any of these
# are skipped (they do not mutate state and need no event).
# ---------------------------------------------------------------------------
READONLY_REGEX='^(get_|is_|has_|query_|view_)'

# ---------------------------------------------------------------------------
# Allowlist and Contract Audit via AWK (Portable across Bash 3.2 / macOS / Linux)
# ---------------------------------------------------------------------------
# shellcheck disable=SC2016
AWK_AUDIT_PROG='
BEGIN {
    allowlist_errors = 0
    missing_events = 0
    checked = 0
    skipped_readonly = 0
    skipped_allowlisted = 0

    in_fn = 0
    fn_name = ""
    has_ev = 0
    depth = 0
    started = 0
    current_contract = ""
    current_file = ""
}

# ---------------------------------------------------------------------------
# Pass 1: Parse allowlist file
# ---------------------------------------------------------------------------
FILENAME == allowlist_path {
    line = $0
    # Strip leading/trailing whitespace
    gsub(/^[[:space:]]+|[[:space:]]+$/, "", line)

    # Skip blank lines and full-line comments
    if (line == "" || substr(line, 1, 1) == "#") {
        next
    }

    # Check for issue reference after comment delimiter "#"
    hash_idx = index(line, "#")
    if (hash_idx == 0) {
        print "FAIL [allowlist format]: " FILENAME ":" FNR " \"" line "\" missing required issue reference (e.g., contract::func # #1234)"
        allowlist_errors++
        next
    }

    entry_key = substr(line, 1, hash_idx - 1)
    issue_ref = substr(line, hash_idx + 1)
    gsub(/^[[:space:]]+|[[:space:]]+$/, "", entry_key)
    gsub(/^[[:space:]]+|[[:space:]]+$/, "", issue_ref)

    # Validate entry key format: contract::function
    if (entry_key !~ /^[a-zA-Z0-9_\-]+::[a-zA-Z0-9_]+$/) {
        print "FAIL [allowlist format]: " FILENAME ":" FNR " invalid entry key \"" entry_key "\""
        allowlist_errors++
        next
    }

    # Validate issue reference: must contain an issue number or reference
    if (issue_ref !~ /[0-9]+/ && issue_ref !~ /#[0-9]+/ && issue_ref !~ /[Ii]ssue/ && issue_ref !~ /ISSUE-[0-9]+/) {
        print "FAIL [allowlist format]: " FILENAME ":" FNR " \"" line "\" missing valid issue reference after #"
        allowlist_errors++
        next
    }

    allowlist[entry_key] = issue_ref
    next
}

# ---------------------------------------------------------------------------
# Pass 2: Audit contract lib.rs files
# ---------------------------------------------------------------------------
FNR == 1 && FILENAME != allowlist_path {
    # Flush any previous file tracking
    if (in_fn && fn_name != "" && started && !has_ev) {
        record_fn(current_contract, fn_name, current_file)
    }
    in_fn = 0
    fn_name = ""
    has_ev = 0
    depth = 0
    started = 0
    current_file = FILENAME

    # Extract contract directory name from path (e.g. contracts/<name>/src/lib.rs)
    n = split(FILENAME, path_parts, "/")
    current_contract = (n >= 3) ? path_parts[n - 2] : "unknown"
}

# Function to record and check an identified pub fn
function record_fn(contract, name, file) {
    if (name ~ readonly_pattern) {
        skipped_readonly++
        return
    }

    checked++
    key = contract "::" name
    if (key in allowlist) {
        skipped_allowlisted++
    } else {
        print "FAIL [missing event]: " file "  fn " name
        missing_events++
    }
}

# New pub fn entrypoint (exactly 4 spaces indent).
/^    pub fn [a-z_][a-z0-9_]*/ {
    if (in_fn && fn_name != "" && started && !has_ev) {
        record_fn(current_contract, fn_name, current_file)
    }
    rest = $0
    sub(/^.*pub fn /, "", rest)
    sub(/[^a-z0-9_].*$/, "", rest)
    fn_name = rest
    has_ev = 0
    depth = 0
    started = 0
    in_fn = 1
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
                if (!has_ev) {
                    record_fn(current_contract, fn_name, current_file)
                }
                in_fn = 0
                fn_name = ""
                has_ev = 0
                started = 0
                break
            }
        }
    }
}

END {
    if (in_fn && fn_name != "" && started && !has_ev) {
        record_fn(current_contract, fn_name, current_file)
    }

    total_violations = allowlist_errors + missing_events

    print ""
    print "Event emission audit results:"
    print "  Checked (state-changing, non-allowlisted): " checked
    print "  Allowlisted (legacy — pending refactor):   " skipped_allowlisted
    print "  Skipped (read-only prefix):                " skipped_readonly
    if (allowlist_errors > 0) {
        print "  Allowlist format errors:                   " allowlist_errors
    }
    print ""

    if (total_violations > 0) {
        if (missing_events > 0) {
            print "FAIL: " missing_events " state-changing pub fn(s) found without event emission."
        }
        if (allowlist_errors > 0) {
            print "FAIL: " allowlist_errors " allowlist entry(ies) lack a valid issue reference."
        }
        print ""
        print "Fix options:"
        print "  1. Add  env.events().publish((...), data)  inside the function body."
        print "  2. Add  contract::function_name # #issue_num  to " allowlist_path
        print "     only if the function is a legacy one pending a separate refactor PR."
        exit 1
    }

    print "OK: all non-allowlisted state-changing pub fn(s) emit events and allowlist format is valid."
}
'

# Find all contract lib.rs files
contract_files=()
while IFS= read -r f; do
    [[ -n "$f" ]] && contract_files+=("$f")
done < <(find "$CONTRACTS_DIR" -path "*/src/lib.rs" -type f | sort)

if [[ ${#contract_files[@]} -eq 0 ]]; then
    echo "No contract lib.rs files found in $CONTRACTS_DIR"
    exit 1
fi

# Run AWK audit
awk -v allowlist_path="$ALLOWLIST_FILE" \
    -v readonly_pattern="$READONLY_REGEX" \
    "$AWK_AUDIT_PROG" \
    "$ALLOWLIST_FILE" \
    "${contract_files[@]}"
