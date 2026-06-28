# Emergency Access Override Contract

A Soroban contract providing emergency data access by secured mult-signature override, for cases where patient consent cannot be obtained immediately.

## Overview

Emergency override contract ensures strict controls when doctors need patient records in life-critical situations.

- Multi-signature authority required (n-of-m trusted entities)
- Time-limited emergency access with automatic expiry
- Full event auditing for every override action
- Access revocation through admin control

### Objectives

- `grant_emergency_access(patient, provider)` under t-of-n approver consent
- `check_emergency_access(patient, provider)` status reflecting expiry
- `revoke_emergency_access(patient, provider)` by admin

### Acceptance Criteria

- Access expires automatically using timestamp logic
- Only authorized trusted entities can approve
- All actions logged with events

## Contract Core

### `initialize(env, admin, approvers, threshold)`
- Initializes contract with admin and trusted approver list
- `threshold` defines required approvals (2 of 3 example)
- Stores approvers and threshold into persistent storage
- Emits `INIT`

### `grant_emergency_access(env, approver, patient, provider, duration_seconds)`
- Auth: `approver.require_auth()`
- Must be one of trusted approvers
- First approval collects signers
- When approvals >= threshold, access is granted and expiry is set
- Emits `APPR` for each approval, `GRANT` when threshold reached

### `check_emergency_access(env, patient, provider)`
- Check if granted and not expired using current ledger time
- Emits `CHECK`

### `revoke_emergency_access(env, admin, patient, provider)`
- Admin only action
- Clears approved status + expires entry
- Emits `REVOKE`

### `get_emergency_access_record(env, patient, provider)`
- Inspection helper returning full record

### `get_admin(env)`
- Returns admin

## Data Model

### `EmergencyAccessRecord`
- `patient`, `provider`
- `requested_duration`, `granted_at`, `expiry_at`
- `approved`, `approvers` list

### `DataKey`
- `Initialized`, `Admin`, `ApprovalThreshold`
- `TrustedApprover(Address)`
- `EmergencyAccess(patient, provider)`

## Errors

- `NotInitialized` (1)
- `AlreadyInitialized` (2)
- `NotAuthorized` (3)
- `InvalidThreshold` (4)
- `InvalidDuration` (5)
- `RecordNotFound` (6)

## Events

- `INIT` (admin)
- `APPR` (patient, provider, approver, timestamp)
- `GRANT` (patient, provider, expiry_at, granted_at)
- `DUPA` duplicate approval attempt (patient, provider, approver, timestamp)
- `CHECK` (patient, provider, has_access, timestamp)
- `REVOKE` (patient, provider, timestamp)

## Security and Compliance

- `require_auth()` on all state-changing calls
- Multi-sig ensures no unilateral emergency override
- Duration safety prevents indefinite access
- Admin revocation path for post-event rollback
- Logged events satisfy audit and forensics

## Testing

Tests cover:
- initialization success/fail
- approval threshold behavior
- duplicate approvals no-effect
- unauthorized approver reject
- grant + explicit access check
- record retrieval and approval state
- admin revoke

Run:

```sh
cd contracts/emergency_access_override
cargo test --lib test::tests
```

## Future Enhancements

- request/approval IDs for concurrent establishment
- provider-initiated emergency request tracking
- optional patient notification hooks
- rate limits and usage SLA tracking
- integration with patient consent module for automatic post-crisis cleanup
