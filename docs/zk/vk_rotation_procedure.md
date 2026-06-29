# ZKP Registry: Verification Key (VK) Rotation & Rollback Procedure

This document outlines the design and operational procedures for rotating and rolling back verifying keys (VK) in the `zkp_registry` contract without losing the validity of historical proofs.

---

## Architecture Overview

To ensure continuous verification of historical proofs while supporting key rotations (due to circuit updates, security enhancements, or parameter updates), the contract maintains a history of all valid verification key hashes for each circuit identifier (`circuit_id`).

### Storage Layout Extensions

A new persistent storage key `DataKey::CircuitVkHistory(String)` maps a `circuit_id` to a list of historical verification key hashes:

```rust
pub enum DataKey {
    // ...
    ZKPCircuitParams(String),
    CircuitVkHistory(String), // Tracks all active and historical vk_hashes
}
```

- **Current Active Key**: The active `vk_hash` is stored in the `ZKPCircuitParams` struct.
- **Historical Keys**: Every verification key that has ever been registered or rotated into the circuit is stored in the `CircuitVkHistory` vector.
- **Verification Hook**: Proof submission via `submit_zkp` verifies that the proof's `vk_hash` matches either the current active `vk_hash` or any key in the `CircuitVkHistory` list.

---

## Procedures

### 1. Migrating Existing Circuits

For circuits registered before key rotation was supported, a one-time migration must be executed to initialize the history list with the circuit's initial verification key.

#### Entrypoint
```rust
pub fn migrate_vk_rotation(env: Env, admin: Address, circuit_id: String) -> Result<(), Error>
```
- **Access Control**: Admin-only.
- **Action**: Loads the existing `ZKPCircuitParams`, constructs the initial history list containing the active `vk_hash`, and saves it to persistent storage.

---

### 2. Rotating a Verification Key

To upgrade the verification key of an existing circuit, the admin performs a key rotation.

#### Entrypoint
```rust
pub fn rotate_vk(
    env: Env,
    admin: Address,
    circuit_id: String,
    new_vk_hash: BytesN<32>,
    new_pk_hash: BytesN<32>,
) -> Result<(), Error>
```
- **Access Control**: Admin-only.
- **Action**:
  1. Loads `ZKPCircuitParams` and `CircuitVkHistory` (initializes history if migration wasn't run).
  2. appends the `new_vk_hash` to the history vector (if not already present).
  3. Updates the active `vk_hash` and `pk_hash` in `ZKPCircuitParams` to the new values.
  4. Publishes a `vk_rot` event.

---

### 3. Rolling Back to a Previous Key

If an issue is detected with a newly rotated key, the admin can rollback the active verification key to any previously registered key in the circuit's history.

#### Entrypoint
```rust
pub fn rollback_vk(
    env: Env,
    admin: Address,
    circuit_id: String,
    target_vk_hash: BytesN<32>,
) -> Result<(), Error>
```
- **Access Control**: Admin-only.
- **Action**:
  1. Ensures the `target_vk_hash` exists in the `CircuitVkHistory` list for the circuit.
  2. Restores the active `vk_hash` in `ZKPCircuitParams` to `target_vk_hash`.
  3. Publishes a `vk_roll` event.
  4. *Note*: The rolled-back key still remains in history, so any proofs submitted under it during its active period remain valid.

---

## Verification Logic (Proof Submission)

When submitting a proof via `submit_zkp`, the contract checks the provided `vk_hash`:

```rust
// 1. Check if it matches the current active key
let mut vk_valid = params.vk_hash == vk_hash;

// 2. If not, check historical keys
if !vk_valid {
    let history_key = DataKey::CircuitVkHistory(circuit_id.clone());
    if let Some(history) = env.storage().persistent().get::<_, Vec<BytesN<32>>>(&history_key) {
        for i in 0..history.len() {
            if history.get(i).unwrap() == vk_hash {
                vk_valid = true;
                break;
            }
        }
    }
}

if !vk_valid {
    return Err(Error::InvalidProof);
}
```

This dual check preserves the validation of historical proofs while immediately allowing newer proofs to use the updated active verification keys.
