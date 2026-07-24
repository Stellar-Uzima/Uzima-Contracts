# DID Key Rotation and Verification Flow Audit

This document audits the cryptographic key rotation and DID verification flows
for correctness and recoverability across all Uzima identity contracts.

## Scope

Contracts audited:
- `contracts/identity_registry` — DID registration and verification
- `contracts/zkp_registry` — ZKP credential registry
- `contracts/credential_registry` — Healthcare credential management
- `contracts/cross_chain_identity` — Cross-chain DID bridge

---

## Key Rotation Flow

### Current Implementation

Key rotation follows a **propose → cooldown → execute** pattern:

```
1. DID owner calls rotate_key(new_key)
   → Stores (old_key, new_key, rotate_at_ledger) as pending rotation
   → Emits KeyRotationProposed event

2. After ROTATION_DELAY_LEDGERS, owner calls confirm_rotation()
   → Replaces active key with new_key
   → Archives old_key for audit trail (not deleted)
   → Emits KeyRotated event

3. At any point before confirmation, cancel_rotation() aborts
```

### Rotation Delay

| Environment | Delay | Rationale |
|-------------|-------|-----------|
| Production | 8,640 ledgers (~12 hours) | Compromise detection window |
| Testnet | 720 ledgers (~1 hour) | Faster testing cycle |
| Emergency rotation | 0 (with guardian approval) | Break-glass only |

---

## DID Verification Security Controls

### Verification Checks

Every `verify_identity()` call validates:

1. **DID exists** — identity must be registered
2. **DID not revoked** — check revocation list
3. **Role matches** — claimed role must match registered role
4. **Key not expired** — key rotation expiry (if set) must not have passed
5. **Issuer signature** — for credential-based verification, issuer sig valid

### Known Gaps (Findings)

| ID | Finding | Severity | Status | Mitigation |
|----|---------|----------|--------|------------|
| F1 | No key expiry enforced by default | Medium | Open | Add `key_expires_at` field to identity records |
| F2 | Revocation check skipped on cross-chain bridge | High | Open | Add revocation lookup in cross_chain_identity |
| F3 | Old keys not archived after rotation | Low | Mitigated | Archive implemented in confirm_rotation() |
| F4 | No rate limit on verify_identity() calls | Low | Open | Apply RateLimiter::conservative() |

---

## Recoverability

### Scenario: DID owner loses key

1. Owner contacts guardian (designated at DID registration time)
2. Guardian submits `guardian_propose_key_recovery(did, new_key)`
3. After `ROTATION_DELAY_LEDGERS`, execute `confirm_recovery()`
4. Old key archived, new key active

### Scenario: DID compromised (key stolen)

1. Any observer (including contract monitoring) detects anomalous access
2. Guardian calls `emergency_revoke(did)` — immediately revokes DID
3. Owner can register a new DID with a fresh key after identity re-verification
4. All prior access logs preserved in audit trail

### Scenario: Guardian key lost

- Guardian must be changed via DAO governance vote (timelock + quorum)
- Minimum timelock: 48 hours for guardian replacement

---

## Recommended Fixes

### Fix F2: Add revocation check in cross_chain_identity

```rust
// In cross_chain_identity/src/lib.rs, before any cross-chain operation:
pub fn verify_cross_chain_identity(
    env: &Env,
    identity: &Address,
    role: &Symbol,
    identity_registry_id: &Address,
) -> Result<(), CrossChainError> {
    // 1. Check revocation first
    let revoked: bool = env.invoke_contract(
        identity_registry_id,
        &symbol_short!("is_revoked"),
        soroban_sdk::vec![env, identity.into_val(env)],
    );
    if revoked {
        return Err(CrossChainError::IdentityRevoked);
    }
    // 2. Verify role
    // ... existing verification ...
    Ok(())
}
```

### Fix F1: Add key_expires_at to identity records

```rust
// In identity_registry/src/lib.rs:
#[contracttype]
pub struct IdentityRecord {
    pub owner: Address,
    pub role: Symbol,
    pub metadata: String,
    pub registered_at: u64,
    pub key_expires_at: Option<u64>,  // NEW: optional key expiry
}
```

---

## Test Coverage

| Test | Location | Status |
|------|----------|--------|
| Key rotation happy path | `identity_registry/src/test.rs` | ✅ |
| Rotation before cooldown rejected | `identity_registry/src/test.rs` | ✅ |
| Revoked DID verification fails | `identity_registry/src/test.rs` | ✅ |
| Cross-chain revocation propagation | `cross_chain_identity/src/lib.rs` | ⚠️ Missing |
| Guardian recovery flow | `identity_registry/src/test.rs` | ⚠️ Missing |
