# Requirements Document

## Introduction

This feature introduces a shared, repository-wide contract security policy for the Uzima Contracts workspace. The policy formalises and structurally enforces the security expectations that currently exist only as descriptive checklists and documentation. It covers five pillars: authentication-before-state-change, checked arithmetic, bounded input handling, explicit error handling, and upgrade safety. Enforcement is delivered through three complementary mechanisms: a canonical policy document referenced from contributing guides, a Clippy-based static-analysis lint pass integrated into CI, and a new-contract template that wires all required patterns by default. The existing `common_auth` crate becomes the single mandated implementation path for authorization helpers across the workspace.

---

## Glossary

- **Contract_Security_Policy**: The repository-wide normative document (`docs/CONTRACT_SECURITY_POLICY.md`) that defines all mandatory security invariants for Uzima contracts.
- **Policy_Linter**: The shell script (`scripts/check_security_policy.sh`) and its companion Clippy flags that detect security-policy violations in contract source code.
- **Common_Auth**: The shared Rust crate at `contracts/common_auth/` providing `require_admin!`, `require_admin_custom!`, `check_admin`, and `is_admin` helpers.
- **Contract_Template**: The skeleton contract at `contracts/contract_template/` that new contracts are expected to copy as their starting point.
- **CI**: The GitHub Actions continuous-integration pipeline defined in `.github/workflows/ci.yml` and related workflow files.
- **PR_Template**: The pull-request template at `.github/PULL_REQUEST_TEMPLATE.md` that reviewers and authors fill in before merging.
- **Security_Checklist_Workflow**: The GitHub Actions workflow at `.github/workflows/security_checklist.yml` that posts the security checklist comment on every new PR.
- **Violation_Report**: The machine-readable output produced by the Policy_Linter, listing each detected violation with the file path, 1-based line number, 1-based column number, rule identifier, and remediation hint.
- **Allowlist**: The file `scripts/allowlists/security_policy.txt` that records explicit, justified exceptions to individual policy rules.
- **Exception_Justification**: A code comment of the form `// SECURITY-EXCEPTION: <rule-id> — <reason>` (max 200 characters for the reason field) placed on the line immediately above the allowlisted expression or statement. The `<rule-id>` must exactly match the rule identifier of the suppressed violation.
- **Suppression_Log entry**: A record in the Violation_Report that captures the rule identifier, file path, line number, and the verbatim Exception_Justification comment text for each suppressed violation.

---

## Requirements

### Requirement 1: Repository-Wide Contract Security Policy Document

**User Story:** As a contract author, I want a single authoritative document that states every mandatory security invariant, so that I know exactly what rules I must follow without consulting multiple scattered guides.

#### Acceptance Criteria

1. THE Contract_Security_Policy SHALL exist at `docs/CONTRACT_SECURITY_POLICY.md` and define at least one mandatory rule for each of the five policy pillars: authentication order, checked arithmetic, bounded inputs, explicit error handling, and upgrade safety. Each rule SHALL carry a unique rule identifier, a violation description, and a remediation guidance section.
2. THE Contract_Security_Policy SHALL assign a unique rule identifier (e.g., `AUTH-001`, `ARITH-001`) to each mandatory rule so that violations and allowlist entries can reference rules unambiguously.
3. THE Contract_Security_Policy SHALL reference `contracts/common_auth/` as the single mandated implementation path for authorization helpers and SHALL provide a migration guide that includes: (a) a before/after code example showing conversion from an ad-hoc check to a `common_auth` macro, (b) the steps to add `common_auth` as a workspace dependency, and (c) a reference to `docs/AUTH_PATTERNS.md` for further detail.
4. IF `CONTRIBUTING.md` exists in the repository root, THEN it SHALL contain a direct hyperlink to `docs/CONTRACT_SECURITY_POLICY.md` so that authors encounter the policy before writing contract code.
5. THE Contract_Security_Policy SHALL document the Allowlist mechanism by specifying: (a) the Exception_Justification comment format, (b) the Allowlist file location and line format, (c) the steps to add a new entry (write the comment, add the allowlist line, open a follow-up issue, request security-designated reviewer approval), and (d) the policy on stale entries.
6. THE Contract_Security_Policy SHALL state that every Allowlist entry must include the rule identifier, the file path, a rationale, and a reference to a follow-up tracking issue.

---

### Requirement 2: Policy Linter — Authentication Order

**User Story:** As a security reviewer, I want automated detection of state-mutating functions that perform role checks or storage writes before calling `require_auth()`, so that authentication-bypass patterns are flagged before code is merged.

#### Acceptance Criteria

1. WHEN the Policy_Linter is executed against a contract source file, THE Policy_Linter SHALL emit a Violation_Report entry with rule identifier `AUTH-001` for every `pub fn` that invokes `env.storage()` accessor methods before `require_auth()` is called on every `Address` parameter involved in that storage operation.
2. WHEN the Policy_Linter is executed against a contract source file, THE Policy_Linter SHALL emit a Violation_Report entry with rule identifier `AUTH-002` for every `pub fn` that accepts an `Address` parameter, invokes `env.storage()` mutating methods, and never calls `require_auth()` on that address anywhere in the function body.
3. WHEN a function is listed in the Allowlist with a valid Exception_Justification comment whose rule-id matches `AUTH-001` or `AUTH-002` and whose reason field is non-empty, THE Policy_Linter SHALL suppress the corresponding violation for that function and SHALL add a Suppression_Log entry to the Violation_Report.
4. IF the Policy_Linter completes a full analysis run of all target files with zero unsuppressed violations, THEN the Policy_Linter SHALL exit with code 0.
5. IF the Policy_Linter completes a run with one or more unsuppressed violations, THEN the Policy_Linter SHALL exit with a non-zero code and write the Violation_Report to `reports/security_policy_violations.txt`.
6. IF the Policy_Linter encounters a source file that cannot be parsed, THEN the Policy_Linter SHALL emit a parse-error entry in the Violation_Report for that file, continue analysis of remaining files, and exit with a non-zero code.

---

### Requirement 3: Policy Linter — Checked Arithmetic

**User Story:** As a security reviewer, I want automated detection of raw arithmetic operators on integer types in contract code, so that potential overflow/underflow vulnerabilities are caught at review time rather than in production.

#### Acceptance Criteria

1. WHEN the Policy_Linter is executed against a contract source file, THE Policy_Linter SHALL emit a Violation_Report entry with rule identifier `ARITH-001` for every use of a raw `+`, `-`, `*`, `+=`, `-=`, or `*=` operator applied to variables of type `i128`, `u64`, `u32`, or `i64` outside of `#[cfg(test)]` modules, `#[test]`-annotated functions, and files under `tests/`.
2. WHEN the Policy_Linter is executed against a contract source file, THE Policy_Linter SHALL emit a Violation_Report entry with rule identifier `ARITH-002` for every division expression on `i128`, `u64`, `u32`, or `i64` variables that does not have an explicit zero-divisor guard (a conditional check or a `checked_div` call) within the same function scope before the division expression.
3. THE Policy_Linter SHALL emit `ARITH-001` and `ARITH-002` violations based solely on the presence of those operators in the source code, independent of the value of `overflow-checks` in any `Cargo.toml`.
4. WHEN a raw arithmetic expression carries a valid Exception_Justification comment whose rule-id is `ARITH-001` or `ARITH-002` and whose reason field is non-empty, THE Policy_Linter SHALL suppress the violation for that specific occurrence and SHALL add a Suppression_Log entry to the Violation_Report.

---

### Requirement 4: Policy Linter — Bounded Input Handling

**User Story:** As a security reviewer, I want automated detection of `String` and `Vec` parameters that are consumed by a function without any length or size bound check, so that unbounded input vulnerabilities are caught systematically.

#### Acceptance Criteria

1. WHEN the Policy_Linter is executed against a contract source file, THE Policy_Linter SHALL emit a Violation_Report entry with rule identifier `INPUT-001` for every `pub fn` outside of `#[cfg(test)]` scope that accepts a parameter of type `String` and does not invoke `.len()`, a comparison against a named `const` or integer literal bound, or a `validate_string_length` call on that parameter within the function body before the first `env.storage()` write in that same function body.
2. WHEN the Policy_Linter is executed against a contract source file, THE Policy_Linter SHALL emit a Violation_Report entry with rule identifier `INPUT-002` for every `pub fn` outside of `#[cfg(test)]` scope that accepts a parameter of type `Vec<_>` and does not invoke `.len()`, a comparison against a named `const` or integer literal bound, or a `validate_vector_size` call on that parameter within the function body before the first `env.storage()` write in that same function body.
3. WHEN an `INPUT-001` or `INPUT-002` violation is suppressed via the Allowlist with a valid Exception_Justification comment, THE Policy_Linter SHALL add a Suppression_Log entry to the Violation_Report including the verbatim Exception_Justification text.

---

### Requirement 5: Policy Linter — Explicit Error Handling

**User Story:** As a security reviewer, I want automated detection of `unwrap()`, `expect()`, and raw `panic!` calls in non-test contract code, so that abrupt transaction aborts with opaque error messages are eliminated from production paths.

#### Acceptance Criteria

1. WHEN the Policy_Linter is executed against a contract source file, THE Policy_Linter SHALL emit a Violation_Report entry with rule identifier `ERR-001` for every call to `.unwrap()` (including `Option::unwrap` and `Result::unwrap`) that appears outside `#[cfg(test)]` modules, `#[test]`-annotated functions, and files under `tests/`.
2. WHEN the Policy_Linter is executed against a contract source file, THE Policy_Linter SHALL emit a Violation_Report entry with rule identifier `ERR-002` for every call to `.expect(…)` (including `Option::expect` and `Result::expect`) that appears outside `#[cfg(test)]` modules, `#[test]`-annotated functions, and files under `tests/`.
3. WHEN the Policy_Linter is executed against a contract source file, THE Policy_Linter SHALL emit a Violation_Report entry with rule identifier `ERR-003` for every invocation of `panic!(…)` that appears outside `#[cfg(test)]` modules, `#[test]`-annotated functions, and files under `tests/`.
4. IF the workspace `clippy.toml` contains `allow-panic-in-tests = true`, `allow-unwrap-in-tests = true`, and `allow-expect-in-tests = true`, THEN the Policy_Linter SHALL produce zero ERR-* entries for call sites inside `#[cfg(test)]` modules, `#[test]`-annotated functions, and files under `tests/`. IF one or more of those keys are absent or `false`, THEN the Policy_Linter SHALL emit a configuration-error entry in the Violation_Report identifying the missing or false key(s) and exit with a non-zero code.
5. WHERE a `panic!`, `.unwrap()`, or `.expect()` call carries a valid Exception_Justification comment whose rule-id is `ERR-001`, `ERR-002`, or `ERR-003` and whose reason field is non-empty, THE Policy_Linter SHALL suppress the violation for that occurrence and SHALL add a Suppression_Log entry to the Violation_Report.

---

### Requirement 6: Policy Linter — Upgrade Safety

**User Story:** As a security reviewer, I want automated detection of upgrade-related functions that lack admin authentication, so that unauthorized contract upgrades are structurally prevented.

#### Acceptance Criteria

1. WHEN the Policy_Linter is executed against a contract source file, THE Policy_Linter SHALL emit a Violation_Report entry with rule identifier `UPG-001` for every function named `upgrade`, `migrate`, or `set_admin` that contains at least one `env.storage()` write call and does not call `require_auth()` on an `Address` parameter before that first storage write.
2. WHEN the Policy_Linter is executed against a contract source file, THE Policy_Linter SHALL emit a Violation_Report entry with rule identifier `UPG-002` for every function named `upgrade` or `migrate` that does not contain an `env.events().publish(…)` call appearing after the last `env.storage()` write statement in the function body.
3. WHEN a `UPG-001` or `UPG-002` violation is suppressed via the Allowlist, THE Policy_Linter SHALL treat the allowlist entry as invalid and still emit the violation UNLESS the Exception_Justification comment at the suppressed code location includes the rule-id, a non-empty reason, and an explicit reference to `docs/CONTRACT_UPGRADE_SAFETY.md`.
4. THE Contract_Security_Policy SHALL state that UPG allowlist entries must be approved by a security-designated reviewer as defined by the repository CODEOWNERS file, and that approvals recorded only in external audit tickets or security review notes do not satisfy this requirement unless also reflected in `docs/CONTRACT_UPGRADE_SAFETY.md`.

---

### Requirement 7: CI Integration of the Policy Linter

**User Story:** As a team lead, I want the Policy Linter to run automatically on every pull request that touches contract source files, so that policy violations block merge before they reach the main branch.

#### Acceptance Criteria

1. WHEN a pull request modifies any file matching `contracts/*/src/*.rs`, THE CI SHALL execute the Policy_Linter as a dedicated job named `security-policy` within `.github/workflows/ci.yml`.
2. IF the Policy_Linter exits with a non-zero code, THEN the CI `security-policy` job SHALL fail, blocking the pull request from merging.
3. IF the Policy_Linter exits with a non-zero code and the triggering event is a pull request, THEN the CI SHALL post the contents of `reports/security_policy_violations.txt` as a PR comment. IF the report file does not exist, the CI SHALL post a comment stating that the linter exited non-zero but produced no report file.
4. IF the Policy_Linter exits with code 0, THEN the CI `security-policy` job SHALL pass.
5. IF a pull request body contains the token `[skip-security-policy]`, THEN the CI SHALL emit a `::warning` annotation identifying the skip token but SHALL NOT fail the `security-policy` job, consistent with the escape-hatch pattern used by the existing `sdk-bindings` job.
6. THE CI `security-policy` job SHALL upload `reports/security_policy_violations.txt` as a workflow artifact named `security-policy-report` with a retention period of 30 days, regardless of whether the job passed or failed. IF the report file does not exist, the upload step SHALL be skipped without failing the job.

---

### Requirement 8: Common Auth Adoption as Default Pattern

**User Story:** As a contract author, I want a clear, documented, and tooling-enforced standard for writing authorization checks, so that I do not accidentally implement a weaker ad-hoc alternative.

#### Acceptance Criteria

1. THE Contract_Security_Policy SHALL designate `require_admin!` and `require_admin_custom!` from `common_auth` as the mandatory implementation path for admin-role checks in all new contracts.
2. THE Contract_Template SHALL declare `common_auth` as a workspace dependency in its `Cargo.toml` and SHALL demonstrate use of `require_admin!` in at least one example public function so that authors copying the template inherit the correct pattern by default.
3. WHEN the Policy_Linter is executed against a contract source file, THE Policy_Linter SHALL emit a Violation_Report entry with rule identifier `AUTH-003` for every occurrence of a manual inline admin check pattern (e.g., `if caller != admin`, `if admin != caller`) that bypasses `common_auth` helpers. AUTH-003 violations SHALL NOT be suppressible via the Allowlist mechanism.
4. THE `docs/AUTH_PATTERNS.md` document SHALL include a section titled "Using common_auth" that: (a) lists the signatures of `require_admin!` and `require_admin_custom!`, (b) shows a complete before/after migration example converting a manual inline check to a `common_auth` macro call, and (c) links to `docs/CONTRACT_SECURITY_POLICY.md`.
5. IF `CONTRIBUTING.md` is updated, THEN it SHALL reference `docs/AUTH_PATTERNS.md` so that new contributors encounter the preferred auth implementation before writing their first contract function.

---

### Requirement 9: Security Review Guidance in PR Template and Security Checklist Workflow

**User Story:** As a reviewer, I want the PR template and automated checklist to reference the Contract Security Policy and direct authors to the Policy Linter report, so that reviews are grounded in the formal policy rather than each reviewer's personal recall.

#### Acceptance Criteria

1. THE PR_Template SHALL include a hyperlink to `docs/CONTRACT_SECURITY_POLICY.md` within the security section so that the policy is one click away during every review.
2. WHEN the Security_Checklist_Workflow posts its comment on a new pull request, THE Security_Checklist_Workflow comment SHALL include a note directing reviewers to check the `security-policy-report` workflow artifact for the automated Violation_Report.
3. THE PR_Template SHALL add a checkbox item under the "Build & Deployment" section with the text: "The `security-policy` CI job has passed (or a `[skip-security-policy]` justification is included above)."
4. THE PR_Template checklist SHALL preserve the existing section numbering and checkbox style so that reviewers are not confused by structural changes.

---

### Requirement 10: Exception Handling and Allowlist Integrity

**User Story:** As a contract author with a legitimate edge case, I want a documented, auditable process for recording a policy exception, so that exceptions are explicit and reviewable rather than silent.

#### Acceptance Criteria

1. THE Allowlist file at `scripts/allowlists/security_policy.txt` SHALL exist and each non-comment line SHALL follow the format `<rule-id> <file-path> <justification-issue-ref>` so that each entry is machine-parseable by the Policy_Linter.
2. WHEN the Policy_Linter reads the Allowlist and a listed file path does not exist in the repository at the time of the linter run, THE Policy_Linter SHALL emit a warning entry in the Violation_Report identifying the stale entry. THE Policy_Linter SHALL NOT automatically modify the Allowlist file; cleanup of stale entries SHALL be performed manually.
3. IF an Allowlist entry exists for a file and rule-id but the corresponding `// SECURITY-EXCEPTION: <rule-id> — <reason>` comment is absent or the rule-id in the comment does not match, THEN the Policy_Linter SHALL treat the allowlist entry as invalid and SHALL still emit the violation as active (non-suppressed).
4. THE Contract_Security_Policy SHALL state that Allowlist entries must be reviewed and approved by at least one security-designated reviewer (as identified in the repository CODEOWNERS file) before merge.
5. WHEN the Policy_Linter produces a Violation_Report that contains both active (non-suppressed) violations and suppressed violations, THE Violation_Report SHALL include a distinct "Suppressed Violations" section listing each Suppression_Log entry. WHEN all detected violations are suppressed and there are no active violations, THE Policy_Linter SHALL exit with code 0 and SHALL NOT write a Violation_Report file.
