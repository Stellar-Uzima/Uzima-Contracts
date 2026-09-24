# security_review_check.sh — Wiring Notes & Known Limitation

Tracks: #1568 "Wire security-scan.sh and security_review_check.sh into PR
CI"

## What landed

- `.github/workflows/security-scan-pr.yml` runs both scripts on every PR
  touching `contracts/**` or `Cargo.lock`:
  - `scripts/security-scan.sh` — **blocking**. It only `exit 1`s on
    critical/high `cargo-audit` advisories (known CVEs in dependencies);
    medium findings and `cargo-geiger` unsafe-code counts are reported but
    don't fail the build (that's the script's own existing behavior, not
    something changed here).
  - `scripts/security_review_check.sh` — **non-blocking** (`|| true`), for
    the reason below.

## Why `security_review_check.sh` isn't blocking yet

While wiring it up, "Check 2: No bare unwrap in non-test code" turned out
to have a real bug: it grepped for `.unwrap()` across each contract's
`src/`, excluding only lines that literally contain the `#[cfg(test)]`
attribute — not the rest of the test module that attribute guards. Since
nearly every contract keeps its tests in a separate `src/test.rs` (or
similar) full of `.unwrap()` calls (completely normal and fine in test
code), **every contract with a test file using `.unwrap()` failed this
FAIL-level check**, which would have made this step permanently red across
the whole repo the moment it went live as a blocking gate.

Fixed the heuristic to also exclude dedicated test files (`test.rs`,
`tests.rs`, `*_test.rs`, `*_tests.rs`, anything under a `tests/`
directory) — the dominant test-organization convention across this
codebase — not just the attribute line. This should eliminate the bulk of
false positives.

**Residual limitation:** it still can't catch `.unwrap()` inside an inline
`#[cfg(test)] mod tests { ... }` block within an otherwise-production file
(a pattern this repo appears to use less often than dedicated test files,
but it does exist in places). Grep has no reliable way to know where that
block ends. A real fix needs either a Rust-aware AST-based scan, or a
per-contract convention that inline test modules aren't used.

## Recommendation

1. Let this run non-blocking for a cycle and check the aggregated output
   (or run `./scripts/security_review_check.sh` locally) for any
   remaining false positives from inline test modules.
2. If clean, flip `security-review-check` to blocking (drop the `|| true`)
   in `.github/workflows/security-scan-pr.yml`.
3. If contracts using inline `#[cfg(test)] mod tests {}` still trip it,
   either move those tests to a dedicated file (matches repo convention)
   or extend the script's exclusion logic further.
