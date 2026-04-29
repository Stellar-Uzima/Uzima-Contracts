# Security Best Practices

## Reentrancy Protection

All payment and escrow contracts follow the **Checks-Effects-Interactions (CEI)** pattern:

1. **Checks** — Validate inputs and state
2. **Effects** — Update contract state and persist to storage
3. **Interactions** — Make external calls (token transfers)

```rust
// ✅ Correct: state updated BEFORE transfer
claim.status = ClaimStatus::Paid;
env.storage().persistent().set(&DataKey::Claim(claim_id), &claim);
token_client.transfer(...); // external call last

// ❌ Wrong: transfer before state update
token_client.transfer(...); // external call first
claim.status = ClaimStatus::Paid; // state update too late
```

The `escrow` contract additionally uses a reentrancy guard via temporary storage.

## Authentication

Always call `address.require_auth()` before any state-modifying operation:

```rust
pub fn confirm_appointment(env: Env, provider: Address, ...) {
    provider.require_auth(); // Must be first
    // ...
}
```

## Input Validation

Validate all inputs before modifying state:

- Amounts must be > 0
- Addresses must not be equal when they represent different roles
- Status transitions must follow the defined state machine

## Access Control

Use the `rbac` contract for role-based access. Never hardcode role checks inline across multiple contracts.

## Encryption

All sensitive medical data must be encrypted **client-side** before submission. The contract stores only encrypted blobs and hashes.

## Audit Trail

All data access must be logged to the `audit` contract. Use the `health_data_access_logging` contract for HIPAA-compliant access logs.

## See Also

- [docs/SECURITY_BEST_PRACTICES.md](../../SECURITY_BEST_PRACTICES.md)
- [docs/MASTER_THREAT_MODEL.md](../../MASTER_THREAT_MODEL.md)
- [docs/CRYPTOGRAPHIC_SECURITY_MODEL.md](../../CRYPTOGRAPHIC_SECURITY_MODEL.md)
