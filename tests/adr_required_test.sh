#!/usr/bin/env bash
#
# Tests for scripts/check_adr_required.py (Issue #1651).
#
# Self-contained: builds a sandbox with synthetic ADRs and feeds the script
# unified diffs, so it needs no cargo build and runs in seconds.
#
# Usage: bash tests/adr_required_test.sh
#
set -uo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
GATE="$ROOT_DIR/scripts/check_adr_required.py"

PASS=0
FAIL=0

ok()   { echo "  ok   - $1"; PASS=$((PASS + 1)); }
nope() { echo "  FAIL - $1"; FAIL=$((FAIL + 1)); }

expect_exit() {
  local want="$1" got="$2" what="$3"
  if [[ "$got" -eq "$want" ]]; then ok "$what (exit $got)"; else nope "$what (want exit $want, got $got)"; fi
}

expect_contains() {
  local file="$1" needle="$2" what="$3"
  if grep -qF -- "$needle" "$file"; then ok "$what"; else nope "$what (missing: $needle)"; fi
}

VALID_ADR='# ADR-001: Adopt a decision-record process

**Status:** Accepted
**Date:** 2026-09-30
'

# A sandbox with one valid, indexed ADR and no other files.
setup() {
  SANDBOX="$(mktemp -d)"
  mkdir -p "$SANDBOX/docs/adr"
  printf '%s' "$VALID_ADR" > "$SANDBOX/docs/adr/ADR-001-decision-record-process.md"
  cat > "$SANDBOX/docs/adr/ADR-PROCESS.md" <<'EOF'
# Architecture Decision Record (ADR) Process

## Existing ADRs

| ADR | Title | Status |
|-----|-------|--------|
| ADR-001 | Adopt a decision-record process | Accepted |
EOF
  cat > "$SANDBOX/noop.patch" <<'EOF'
diff --git a/docs/notes.md b/docs/notes.md
--- a/docs/notes.md
+++ b/docs/notes.md
@@ -1 +1 @@
-old
+new
EOF
}

teardown() { [[ -n "${SANDBOX:-}" ]] && rm -rf "$SANDBOX"; }

run_gate() {
  python3 "$GATE" --root "$SANDBOX" --diff-file "$1" >"$SANDBOX/out.txt" 2>&1
}

# ── 0. The real repository passes as-is ────────────────────────────────────
echo "live repository"
python3 "$GATE" --root "$ROOT_DIR" >/tmp/adr_live.txt 2>&1
expect_exit 0 $? "the committed ADRs and index are consistent"
rm -f /tmp/adr_live.txt

# ── 1. No breaking change: no ADR required ─────────────────────────────────
echo "clean change"
setup
run_gate "$SANDBOX/noop.patch"
expect_exit 0 $? "a docs-only change passes"
teardown

# ── 2. Breaking ABI without an ADR fails ───────────────────────────────────
echo "breaking ABI, no ADR"
setup
cat > "$SANDBOX/abi.patch" <<'EOF'
diff --git a/contracts/payments/src/lib.rs b/contracts/payments/src/lib.rs
--- a/contracts/payments/src/lib.rs
+++ b/contracts/payments/src/lib.rs
@@ -1,3 +1,3 @@
 #[contractimpl]
-    pub fn transfer(to: Address, amount: i128) {
+    pub fn transfer(to: Address) {
EOF
run_gate "$SANDBOX/abi.patch"
expect_exit 1 $? "re-signing an exported function fails without an ADR"
expect_contains "$SANDBOX/out.txt" "breaking ABI change" "names the breaking ABI change"
expect_contains "$SANDBOX/out.txt" "no ADR was added" "explains how to fix it"
teardown

# ── 3. Same change with an ADR passes ──────────────────────────────────────
echo "breaking ABI, ADR present"
setup
cat > "$SANDBOX/abi_with_adr.patch" <<'EOF'
diff --git a/contracts/payments/src/lib.rs b/contracts/payments/src/lib.rs
--- a/contracts/payments/src/lib.rs
+++ b/contracts/payments/src/lib.rs
@@ -1,3 +1,3 @@
 #[contractimpl]
-    pub fn transfer(to: Address, amount: i128) {
+    pub fn transfer(to: Address) {
diff --git a/docs/adr/ADR-008-transfer-signature.md b/docs/adr/ADR-008-transfer-signature.md
new file mode 100644
--- /dev/null
+++ b/docs/adr/ADR-008-transfer-signature.md
@@ -0,0 +1,3 @@
+# ADR-008: Transfer signature
+
+**Status:** Proposed
EOF
run_gate "$SANDBOX/abi_with_adr.patch"
expect_exit 0 $? "an ADR in the change set clears the requirement"
expect_contains "$SANDBOX/out.txt" "ADR-008" "reports the ADR that was found"
teardown

# ── 4. Storage-layout change without an ADR fails (diff fallback) ──────────
echo "storage change, no ADR"
setup
cat > "$SANDBOX/storage.patch" <<'EOF'
diff --git a/contracts/records/src/lib.rs b/contracts/records/src/lib.rs
--- a/contracts/records/src/lib.rs
+++ b/contracts/records/src/lib.rs
@@ -1,6 +1,5 @@
 #[contracttype]
 enum DataKey {
-    PatientRecords,
     Admin,
 }
EOF
run_gate "$SANDBOX/storage.patch"
expect_exit 1 $? "removing a DataKey variant fails without an ADR"
expect_contains "$SANDBOX/out.txt" "storage-layout change" "names the storage change"
expect_contains "$SANDBOX/out.txt" "PatientRecords" "names the removed variant"
teardown

# ── 5. A maintainer waiver is honoured and printed ─────────────────────────
echo "allow-breaking waiver"
setup
run_waiver() {
  python3 "$GATE" --root "$SANDBOX" --diff-file "$1" --allow-breaking \
    >"$SANDBOX/out.txt" 2>&1
}
cat > "$SANDBOX/abi.patch" <<'EOF'
diff --git a/contracts/payments/src/lib.rs b/contracts/payments/src/lib.rs
--- a/contracts/payments/src/lib.rs
+++ b/contracts/payments/src/lib.rs
@@ -1,3 +1,3 @@
 #[contractimpl]
-    pub fn transfer(to: Address, amount: i128) {
+    pub fn transfer(to: Address) {
EOF
run_waiver "$SANDBOX/abi.patch"
expect_exit 0 $? "--allow-breaking clears the requirement"
expect_contains "$SANDBOX/out.txt" "WARNING" "prints the waiver so it is visible"
teardown

# ── 6. ADR hygiene: a missing Status fails ─────────────────────────────────
echo "ADR hygiene"
setup
cat > "$SANDBOX/docs/adr/ADR-002-no-status.md" <<'EOF'
# ADR-002: Missing status

**Date:** 2026-09-30
EOF
printf '| ADR-002 | Missing status | Proposed |\n' >> "$SANDBOX/docs/adr/ADR-PROCESS.md"
run_gate "$SANDBOX/noop.patch"
expect_exit 1 $? "an ADR without **Status:** fails"
expect_contains "$SANDBOX/out.txt" "missing '**Status:**'" "names the missing field"
teardown

# ── 7. Index sync: an unindexed ADR fails ──────────────────────────────────
echo "index sync"
setup
cat > "$SANDBOX/docs/adr/ADR-003-unindexed.md" <<'EOF'
# ADR-003: Unindexed

**Status:** Accepted
**Date:** 2026-09-30
EOF
run_gate "$SANDBOX/noop.patch"
expect_exit 1 $? "an ADR missing from the index fails"
expect_contains "$SANDBOX/out.txt" "missing from the ADR-PROCESS.md index" "points at the index"
teardown

echo
echo "passed: $PASS, failed: $FAIL"
[[ "$FAIL" -eq 0 ]]
