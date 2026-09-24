# Admin / Recovery Path Hardening — Gap Analysis

Tracks: #1572 "Harden admin/recovery paths across auth & treasury
contracts"

## Policy (what should happen)

`docs/SECURITY_BEST_PRACTICES.md` and `docs/THREAT_MODEL.md` describe
admin-recovery flows (who can trigger recovery, what thresholds/delay
apply) as security-critical paths that need explicit negative-path test
coverage — not just "does the happy path work."

## Contracts with an admin-recovery-shaped surface

Found by grepping for `recover`/`restore`/`override`/`emergency_*`
functions taking a `caller: Address` (i.e. a privileged action gated by
who's calling):

| Contract | Function(s) | Negative/unauthorized test present? |
|---|---|---|
| `emergency_access_override` | `admin_recovery.rs` recovery flow | **Yes** — `src/test.rs` has explicit unauthorized-caller cases |
| `treasury_controller` | `emergency_halt`, resume operations | **Yes** — `test_emergency_halt_unauthorized`, `test_resume_operations_unauthorized` in `src/test.rs` |
| `identity_registry` | `rotate_key`, `revoke_verification_method` | Yes (has unauthorized-pattern tests) |
| `mfa` | recovery codes / device recovery | Yes (has unauthorized-pattern tests) |
| `healthcare_payment` | payment recovery path | Yes (has unauthorized-pattern tests) |
| `cross_chain_bridge` | bridge recovery path | Yes (has unauthorized-pattern tests) |
| `medical_record_backup` | `approve_restore`, `execute_restore` | **No test specifically named/targeted at unauthorized-caller rejection** for either function |
| `failover_detector` | recovery/failover logic | **No test file exists at all** — already tracked separately as **#1536** ("Add unit + property test coverage for failover_detector"), assigned to a different contributor. Not duplicated here. |

So, beyond the contract the issue explicitly calls out
(`emergency_access_override`, already covered), the real gap is narrower
than "everywhere": **`medical_record_backup`** is the one contract with a
genuine, uncovered admin-recovery authorization gap that isn't already
tracked by another issue.

## Why a fix wasn't written here

Writing a correct negative test for `medical_record_backup::approve_restore`
/`execute_restore` requires understanding the full `RestoreRequest`
lifecycle (who can request, who approves, multi-party thresholds if any)
well enough to construct a valid non-caller scenario without breaking the
existing happy-path tests — that needs a careful read of the contract's
state machine and its own reviewed PR, not a guess bundled into an
audit-scoped change that can't be compiled to verify.

## What landed instead

- This gap analysis.
- `scripts/check_admin_recovery_test_coverage.sh` — an informational check
  that greps each contract for privileged recovery-shaped functions and
  flags ones with no nearby unauthorized/negative test, wired into
  `.github/workflows/security-checks.yml`.

## Follow-ups (file as separate issues)

1. Add `test_approve_restore_unauthorized` / `test_execute_restore_unauthorized`
   to `contracts/medical_record_backup/src/test.rs`.
2. Track `failover_detector` coverage under #1536 (already filed).
3. Once `medical_record_backup` is covered, re-run the check script and
   consider flipping it to `--strict` (non-zero exit) for new contracts.
