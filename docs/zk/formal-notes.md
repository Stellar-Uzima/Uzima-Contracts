# Formal Notes

## Core Invariants
1. Access Safety:
   When `zk_enforced = true`, no privileged record read is returned unless:
   - ACL predicate is true, and
   - valid non-expired ZK grant exists.

2. Grant Integrity:
   ZK grant exists only after successful `submit_zk_access_proof`.

3. Nullifier Uniqueness:
   A nullifier cannot be accepted more than once.

4. Root Consistency:
   Submitted `credential_root` must equal current active issuer root and must not be revoked.

5. Commitment Binding:
   Submitted `record_commitment` must match on-chain record commitment for `record_id`.

## State Transition Safety Properties
1. `submit_zk_access_proof`:
   - Pre: initialized, not paused, valid inputs, nullifier unused.
   - Post success: nullifier marked used; grant persisted with expiration.
   - Post failure: no new grant; no nullifier consumption.

2. `get_record` and related read paths:
   - Pre: ACL pass required.
   - If `zk_enforced`: must also observe a valid grant.
   - Post: emits access event only when both checks pass.

3. Admin mutation functions:
   - Require auth and admin role.

## Suggested Property-Based Tests
1. Nullifier replay:
   For any valid proof envelope with nullifier `n`, second submission with same `n` always fails.

2. Root mismatch:
   For any generated public inputs where `credential_root != active_root`, submission fails.

3. Commitment mismatch:
   For any generated public inputs where `record_commitment != stored_commitment(record_id)`, submission fails.

4. Enforcement toggle:
   If `zk_enforced=false`, ACL-only access behavior remains unchanged.
   If `zk_enforced=true`, ACL pass without grant must fail.

5. Grant expiration:
   Any grant with `expires_at <= now` is invalid.

