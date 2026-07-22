# Medical Records Policy Engine

This document describes the centralized policy engine for the `medical_records` contract, introduced in issue #1096.

## Overview

The policy engine (`contracts/medical_records/src/policy.rs`) consolidates access control, consent verification, encryption enforcement, and lifecycle invariant checks into a single, well-documented decision point.

Prior to this change, policy checks were scattered across 8+ distinct patterns in `lib.rs`, making it difficult to reason about the full set of invariants enforced for each operation. The centralized engine eliminates this complexity.

## Architecture

```
┌─────────────────────────────────────────────────────────┐
│                     Policy Engine                        │
│                                                         │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌────────┐ │
│  │  System   │  │ Consent  │  │Encryption│  │Lifecycle│ │
│  │  Checks   │  │  Checks  │  │  Checks  │  │ Checks │ │
│  └─────┬────┘  └────┬─────┘  └────┬─────┘  └───┬────┘ │
│        │            │             │             │       │
│        └────────────┴─────────────┴─────────────┘       │
│                         │                               │
│                   PolicyDecision                        │
│                  Allowed | Denied                       │
└─────────────────────────────────────────────────────────┘
```

## Invariant Categories

| Category | What is Checked | Error Range |
|----------|----------------|-------------|
| **System** | Contract initialized, not paused | 300-301 |
| **Authentication** | Soroban auth signature present | 100 |
| **Authorization** | RBAC role, granular permission, delegation grants | 100-199 |
| **Consent** | Patient consent via external consent contract | 100 |
| **Encryption** | Encryption-required flag, PQ envelope compliance | 600-699 |
| **Lifecycle** | Record existence, retention, patient forgotten status | 100-199, 300-399 |
| **Rate Limiting** | Per-role, per-operation call frequency | 300 |

## Lifecycle Policies

### Create Record (`check_create_record_policy`)

Checks applied before creating a new medical record:
1. Contract is initialized and not paused
2. Patient has not been marked as "forgotten" under regulatory compliance

### Read Record (`check_read_record_policy`)

Checks applied before reading a medical record:
1. Contract is initialized and not paused
2. Patient has not been forgotten
3. Patient has granted consent to the requesting provider

### Create Encrypted Record (`check_create_encrypted_record_policy`)

Checks applied before creating an encrypted medical record:
1. Contract is initialized and not paused
2. Crypto registry contract is configured
3. Patient has not been forgotten
4. At least one key envelope is addressed to the patient
5. Post-quantum envelope compliance (if PQ requirement is enabled)

### Read Encrypted Record (`check_read_encrypted_record_policy`)

Checks applied before reading an encrypted medical record:
1. Contract is initialized and not paused
2. Patient has not been forgotten
3. Caller has a decryption envelope for the record

### Update Record (`check_update_record_policy`)

Checks applied before updating a medical record:
1. Contract is initialized and not paused
2. Patient has not been forgotten
3. Patient has granted consent

### Delete Record (`check_delete_record_policy`)

Checks applied before deleting a medical record:
1. Contract is initialized and not paused
2. Patient has granted consent

### Emergency Access (`check_emergency_access_policy`)

Checks applied for emergency record access:
1. Contract is initialized and not paused
2. Patient has not been forgotten
3. Emergency grant is active
4. Emergency grant has not expired

### Cross-Chain Sync (`check_cross_chain_sync_policy`)

Checks applied for cross-chain synchronization:
1. Contract is initialized and not paused
2. Cross-chain is enabled
3. Cross-chain contracts are configured

## Structured Error Responses

Every policy violation returns a `PolicyViolation` containing:

```rust
pub struct PolicyViolation {
    pub category: PolicyCategory,  // High-level category
    pub error: Error,              // Contract error code
    pub message: String,           // Human-readable explanation
}
```

This enables:
- CI to surface structured reports on policy failures
- PR comments with actionable error details
- Programmatic error handling by downstream contracts

## Usage by Adjacent Contracts

The policy module is publicly exported from the `medical_records` crate. Adjacent contracts can import and reuse the policy types and decision patterns:

```rust
use medical_records::policy::{
    PolicyDecision, PolicyViolation, PolicyCategory,
    require_initialized, require_not_paused, check_consent,
};
```

## Migration Guide

The policy engine is a **non-breaking addition**. Existing code paths continue to work. New and refactored code should route through the policy engine:

```rust
// Before (scattered checks):
caller.require_auth();
Self::require_initialized(&env)?;
Self::require_not_paused(&env)?;
// ... inline consent check ...

// After (centralized policy):
let decision = policy::check_read_record_policy(&policy::ReadRecordPolicy {
    env: &env,
    caller: &caller,
    patient: &record.patient_id,
});
if let PolicyDecision::Denied(violation) = decision {
    return Err(violation.into_error());
}
```
