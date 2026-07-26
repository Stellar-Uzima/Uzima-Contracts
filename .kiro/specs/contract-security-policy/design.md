# Design Document — Shared Contract Security Policy

## Overview

This design introduces a structural, code-level security enforcement layer for the Uzima Contracts workspace. It builds directly on existing infrastructure: the `common_auth` crate, the `scripts/allowlists/` pattern established by `event_emission.txt` and `production_unwraps.txt`, and the CI job structure in `.github/workflows/ci.yml`. Nothing is invented from scratch; each new artifact either extends or mirrors a pattern that already exists in the repository.

The three deliverables are:

1. **`scripts/check_security_policy.sh`** — the Policy Linter, a bash script using `rg` (ripgrep) following the same approach as `check_events.sh` and `check_no_production_unwraps.sh`.
2. **Documentation** — `docs/CONTRACT_SECURITY_POLICY.md`, `docs/AUTH_PATTERNS.md`, `docs/CONTRACT_UPGRADE_SAFETY.md`, and updates to `CONTRIBUTING.md`.
3. **CI/PR integration** — a new `security-policy` job in `ci.yml`, an updated security checklist workflow, an updated PR template, and the `scripts/allowlists/security_policy.txt` allowlist file.

---

## Architecture

### Component Map

```
┌─────────────────────────────────────────────────────────────┐
│  Pull Request opens / contract file changed                 │
└────────────────────┬────────────────────────────────────────┘
                     │
          ┌──────────▼──────────┐
          │  CI: security-policy │   (.github/workflows/ci.yml)
          │  job                 │
          └──────────┬──────────┘
                     │ runs
          ┌──────────▼──────────────────┐
          │  scripts/check_security_     │
          │  policy.sh                   │
          │                              │
          │  Reads:                      │
          │  • contracts/*/src/**/*.rs   │
          │  • scripts/allowlists/       │
          │    security_policy.txt       │
          │                              │
          │  Writes (on violations):     │
          │  • reports/security_policy_  │
          │    violations.txt            │
          └──────────┬───────────────────┘
                     │
         ┌───────────┴───────────────┐
         │                           │
   exit 0 (pass)             exit 1 (fail)
         │                           │
  CI job passes           CI posts PR comment
                          CI fails job
                          Artifact uploaded
```

### Rule-to-Script Mapping

Each linter rule is a self-contained function in `check_security_policy.sh`. Rules share infrastructure: the allowlist reader, the violation recorder, and the report writer.

| Rule ID   | Detection mechanism |
|-----------|-------------------|
| AUTH-001  | `rg` for `env.storage()` calls before the first `require_auth` call in a `pub fn` body (awk state machine) |
| AUTH-002  | `rg`/awk: `pub fn` with `Address` param + `env.storage()` write but no `require_auth` anywhere in body |
| AUTH-003  | `rg` pattern: `if \w+ != \w+` or `if \w+ == \w+` adjacent to `return Err` (manual inline admin check) |
| ARITH-001 | `rg` for raw `+`, `-`, `*`, `+=`, `-=`, `*=` on typed integer lines, excluding `#[cfg(test)]` and `tests/` |
| ARITH-002 | `rg` for `/` or `/=` operators on integer lines without a preceding `checked_div` or guard |
| INPUT-001 | awk: `pub fn` with `String` param, `env.storage()` write, no `.len()` / bound comparison / `validate_string_length` before the write |
| INPUT-002 | awk: same but `Vec<` param and `validate_vector_size` |
| ERR-001   | `rg` for `.unwrap()` outside test scope (supersedes `check_no_production_unwraps.sh` output) |
| ERR-002   | `rg` for `.expect(` outside test scope |
| ERR-003   | `rg` for `panic!(` outside test scope |
| UPG-001   | awk: functions named `upgrade`/`migrate`/`set_admin` with storage write but no `require_auth` first |
| UPG-002   | awk: `upgrade`/`migrate` without `env.events().publish` after last storage write |

---

## Detailed Component Design

### 1. `scripts/check_security_policy.sh`

**Structure** (mirrors `check_events.sh`):

```
check_security_policy.sh
├── Configuration constants (ROOT_DIR, CONTRACTS_DIR, ALLOWLIST_FILE, REPORT_FILE)
├── Allowlist loader          — parse security_policy.txt into ALLOWLIST[] map
├── Clippy.toml validator     — verify allow-panic-in-tests etc. are set
├── Per-rule detection functions
│   ├── check_auth_order()    — AUTH-001, AUTH-002
│   ├── check_auth_pattern()  — AUTH-003
│   ├── check_arithmetic()    — ARITH-001, ARITH-002
│   ├── check_input_bounds()  — INPUT-001, INPUT-002
│   ├── check_error_handling()— ERR-001, ERR-002, ERR-003
│   └── check_upgrade_safety()— UPG-001, UPG-002
├── Violation recorder        — record_violation(rule_id, file, line, col, msg)
├── Suppression logic         — try_suppress(rule_id, file, line) returns 0/1
├── Report writer             — write_report() serialises active + suppressed sections
└── Main driver               — iterate contracts/*/src/**/*.rs, invoke rule functions
```

**Allowlist format** (`scripts/allowlists/security_policy.txt`):

```
# Format: <rule-id> <relative-file-path> <issue-ref>
# Example:
AUTH-001 contracts/patient_records/src/lib.rs #1042
ARITH-001 contracts/token/src/lib.rs #1051
```

**Exception_Justification comment** (required at the suppressed code location):

```rust
// SECURITY-EXCEPTION: AUTH-001 — caller is a system oracle with no interactive auth path, tracked in #1042
```

The rule-id in the comment must exactly match the allowlist rule-id. Reason field must be non-empty. Max 200 characters for the reason field.

**Violation Report format** (`reports/security_policy_violations.txt`):

```
SECURITY_POLICY_VIOLATIONS=3
SECURITY_POLICY_SUPPRESSED=1

== Active Violations ==
[AUTH-001] contracts/foo/src/lib.rs:42:5 — storage write before require_auth(); use require_auth() first
[ERR-001] contracts/bar/src/lib.rs:17:13 — .unwrap() in production code; use ? or match
[ARITH-001] contracts/baz/src/lib.rs:88:22 — raw + on i128; use .checked_add()

== Suppressed Violations ==
[ARITH-001] contracts/token/src/lib.rs:55:10 — SUPPRESSED: // SECURITY-EXCEPTION: ARITH-001 — oracle-guaranteed range, #1051
```

**Exit codes:**
- `0` — full analysis complete, zero active violations
- `1` — one or more active violations, or parse/config error

### 2. `scripts/allowlists/security_policy.txt`

Initial file with comment-only content (no suppressions at launch). The linter warns on stale paths (file no longer exists) but does not auto-modify the file.

### 3. `docs/CONTRACT_SECURITY_POLICY.md`

Top-level policy document. Sections:

1. Purpose and scope
2. Five pillars with rule table (rule ID, name, description, remediation)
3. Mandatory use of `common_auth` with migration guide
4. Allowlist / exception process
5. Upgrade safety exceptions (reference to `CONTRACT_UPGRADE_SAFETY.md`)
6. Reference to `AUTH_PATTERNS.md` for implementation guidance

### 4. `docs/AUTH_PATTERNS.md`

Implementation guide. Key section: **"Using common_auth"**:
- Macro signatures for `require_admin!` and `require_admin_custom!`
- Before/after migration example
- Link back to `CONTRACT_SECURITY_POLICY.md`

### 5. `docs/CONTRACT_UPGRADE_SAFETY.md`

Upgrade exception registry. Structure:
- Auditor-approved exception table (rule, contract, rationale, issue ref, reviewer)
- Requirement: UPG allowlist entries must appear here before the allowlist entry is valid

### 6. `contracts/contract_template/`

New skeleton contract directory. Includes:
- `Cargo.toml` with `common_auth = { workspace = true }`
- `src/lib.rs` with `require_admin!()` macro invoked in at least one example `pub fn`
- Inline comments pointing to `docs/CONTRACT_SECURITY_POLICY.md`

### 7. `.github/workflows/ci.yml` — new `security-policy` job

Appended to the existing job list. Follows the `sdk-bindings` escape-hatch pattern for `[skip-security-policy]`.

```yaml
security-policy:
  name: Security Policy Lint
  runs-on: ubuntu-latest
  permissions:
    contents: read
    pull-requests: write
  steps:
    - uses: actions/checkout@v4
    - name: Check for [skip-security-policy] token
      id: skip_check
      env:
        PR_BODY: ${{ github.event.pull_request.body }}
      run: |
        if [ "${{ github.event_name }}" = "pull_request" ] && \
           printf '%s' "$PR_BODY" | grep -qF '[skip-security-policy]'; then
          echo "skip=true" >> "$GITHUB_OUTPUT"
        else
          echo "skip=false" >> "$GITHUB_OUTPUT"
        fi
    - name: Run security policy linter
      id: linter
      run: bash ./scripts/check_security_policy.sh
      continue-on-error: true   # capture exit code; enforce below
    - name: Emit skip warning
      if: steps.skip_check.outputs.skip == 'true'
      run: echo "::warning::Security policy lint skipped via [skip-security-policy] token"
    - name: Fail on violations (unless skipped)
      if: steps.linter.outcome == 'failure' && steps.skip_check.outputs.skip != 'true'
      run: exit 1
    - name: Post violation report as PR comment
      if: >
        github.event_name == 'pull_request' &&
        steps.linter.outcome == 'failure' &&
        steps.skip_check.outputs.skip != 'true'
      continue-on-error: true
      uses: actions/github-script@v7
      with:
        script: |
          const fs = require('fs');
          const path = 'reports/security_policy_violations.txt';
          const body = fs.existsSync(path)
            ? fs.readFileSync(path, 'utf8')
            : '⚠️ Security policy linter exited non-zero but produced no report file.';
          github.rest.issues.createComment({
            issue_number: context.issue.number,
            owner: context.repo.owner,
            repo: context.repo.repo,
            body: '### 🛡️ Security Policy Violations\n\n```\n' + body + '\n```'
          });
    - name: Upload security policy report
      if: always()
      uses: actions/upload-artifact@v4
      with:
        name: security-policy-report
        path: reports/security_policy_violations.txt
        retention-days: 30
        if-no-files-found: ignore
```

### 8. `.github/workflows/security_checklist.yml` — updated comment

Add a line to the existing checklist comment body pointing reviewers to the `security-policy-report` artifact.

### 9. `.github/PULL_REQUEST_TEMPLATE.md` — updated checklist

Add one checkbox to the existing **Section 8 (Build & Deployment)**:

```markdown
- [ ] The `security-policy` CI job has passed (or a `[skip-security-policy]` justification is included above).
```

Add a hyperlink to `docs/CONTRACT_SECURITY_POLICY.md` in the security checklist header.

### 10. `CONTRIBUTING.md` — updated references

Add hyperlinks to `docs/CONTRACT_SECURITY_POLICY.md` and `docs/AUTH_PATTERNS.md` in the contract development section.

---

## Data Models

### Allowlist Entry (parsed from `security_policy.txt`)

```
ALLOWLIST[<rule-id>:<relative-path>] = <issue-ref>
```

Used as a two-level key to look up whether a given (rule, file) pair has been allowlisted. The linter then checks for the matching `SECURITY-EXCEPTION` comment at or near the flagged line.

### Violation Record (in-memory during linter run)

```bash
# Bash associative array entry
VIOLATIONS+=("${RULE_ID}|${REL_FILE}|${LINE}|${COL}|${MESSAGE}")
SUPPRESSED+=("${RULE_ID}|${REL_FILE}|${LINE}|${COMMENT_TEXT}")
```

### Report Token (machine-parseable, in `violations.txt`)

```
SECURITY_POLICY_VIOLATIONS=<N>
SECURITY_POLICY_SUPPRESSED=<M>
```

Follows the same `KEY=VALUE` token pattern used by `sdk_bindings_drift.txt` and `doc_coverage.txt`, enabling future CI script parsing.

---

## Integration Points

| Existing artifact | Change type | Reason |
|---|---|---|
| `scripts/check_no_production_unwraps.sh` | Superseded for ERR-001 | Policy linter covers `.unwrap()` with full context (allowlist, suppression log); the old script remains for now to avoid breakage but its output overlaps ERR-001. Both can run in parallel; a follow-up can deprecate the old script once the new one is proven stable. |
| `scripts/allowlists/` directory | Extended | New file `security_policy.txt` added alongside the existing `event_emission.txt` and `production_unwraps.txt`. |
| `reports/` directory | Extended | New file `security_policy_violations.txt` written by the linter. |
| `.github/workflows/ci.yml` | Appended | New `security-policy` job added; no existing jobs modified. |
| `.github/workflows/security_checklist.yml` | Comment body updated | One sentence added directing reviewers to the artifact. |
| `.github/PULL_REQUEST_TEMPLATE.md` | Section 8 updated | One checkbox added; header gets a policy link. |
| `CONTRIBUTING.md` | References added | Two hyperlinks in the contract development section. |
| `contracts/common_auth/` | Unchanged | API is stable and sufficient; no modifications needed. |
| `Cargo.toml` [workspace.dependencies] | Already present | `common_auth = { path = "contracts/common_auth" }` already declared. |

---

## File Inventory

New files created by this feature:

```
scripts/check_security_policy.sh
scripts/allowlists/security_policy.txt
docs/CONTRACT_SECURITY_POLICY.md
docs/AUTH_PATTERNS.md
docs/CONTRACT_UPGRADE_SAFETY.md
contracts/contract_template/Cargo.toml
contracts/contract_template/src/lib.rs
```

Modified files:

```
.github/workflows/ci.yml
.github/workflows/security_checklist.yml
.github/PULL_REQUEST_TEMPLATE.md
CONTRIBUTING.md
```

---

## Error Handling and Edge Cases

- **Unparseable source file**: linter records a parse-error entry and continues; exits non-zero after full scan.
- **`reports/` directory absent**: linter creates it with `mkdir -p` before writing the report.
- **Missing `clippy.toml` keys**: linter emits a configuration-error entry and exits non-zero.
- **Stale allowlist entry** (file no longer exists): linter emits a `WARN` entry in the report; does not auto-modify the allowlist.
- **AUTH-003 violations**: not suppressible — these always appear as active violations regardless of allowlist entries.
- **`[skip-security-policy]` in PR body**: CI emits `::warning` annotation; linter itself still runs and produces its report (artifact still uploaded); only the fail-on-violations step is skipped.

---

## Security Considerations

- The Policy Linter is a bash + ripgrep script with no network access; it reads only files in the workspace.
- The allowlist file is subject to CODEOWNERS review (see `docs/CONTRACT_SECURITY_POLICY.md` for the required approval process).
- AUTH-003 is intentionally non-suppressible to prevent silent acceptance of ad-hoc admin checks; any migration must go through `common_auth`.
- UPG exceptions require documentation in `docs/CONTRACT_UPGRADE_SAFETY.md` specifically; external audit tickets alone are not sufficient.
