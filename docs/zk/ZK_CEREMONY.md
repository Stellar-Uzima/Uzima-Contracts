# ZK Ceremony & Verifier Key Management

## What is a Ceremony?

A ZK trusted setup ceremony generates the public parameters (verifier key)
used to verify zero-knowledge proofs. The Uzima `zkp_registry` contract
stores this key on-chain and uses it to validate all incoming proofs.

## Key Rotation Process

1. **Run a new ceremony** off-chain (e.g., using snarkjs or similar tooling).
2. **Export the new verifier key** as a 32-byte hex string.
3. **Call `update_verifier_key(admin, new_key)`** from the admin account.
   Only the admin role can perform this operation.
4. **Verify the event** — the contract emits a `verifier_key_updated` event
   that is recorded on-chain for audit purposes.
5. **Deprecate old proofs** — all proofs generated under the previous key
   are immediately rejected after rotation. Notify users to re-generate proofs.

## Security Rules

- Key rotation is **admin-only** — unauthorized attempts are rejected with an error.
- The old key is **not stored** after rotation — it is permanently replaced.
- All rotations are **auditable** via on-chain events.
- Run ceremonies in a **multi-party computation (MPC)** setup to avoid
  a single point of trust.

## Testing Key Rotation

```bash
cargo test -p zkp_registry upgrade_tests
```