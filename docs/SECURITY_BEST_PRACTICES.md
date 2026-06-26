# Security Best Practices Guide

This guide covers secure smart contract development patterns for Uzima Contracts on Soroban/Stellar. Follow these practices when writing, reviewing, or auditing contracts.

---

## 1. Access Control

### Always authenticate before authorizing

Every state-mutating function must call `require_auth()` on the caller before checking roles:

```rust
pub fn admin_action(env: Env, caller: Address) -> Result<(), Error> {
    caller.require_auth();           // 1. Authenticate (Soroban verifies signature)
    Self::require_admin(&env, &caller)?; // 2. Authorize (check role)
    // ... perform action
    Ok(())
}
```

Never skip `require_auth()` — without it, any address can impersonate the caller.

### Enforce single initialization

Use a guard that returns an error (not a panic) on re-initialization:

```rust
pub fn initialize(env: Env, admin: Address) -> Result<(), Error> {
    if env.storage().instance().has(&DataKey::Admin) {
        return Err(Error::AlreadyInitialized);
    }
    admin.require_auth();
    env.storage().instance().set(&DataKey::Admin, &admin);
    env.events().publish(("initialized",), admin);
    Ok(())
}
```

### Principle of least privilege

- Admin functions should only be callable by the stored admin address.
- Validator/operator functions should only be callable by registered validators.
- Read-only functions need no authentication.

---

## 2. Common Vulnerabilities and Defenses

### Integer Overflow / Underflow

Soroban runs in `no_std` with Rust's overflow checks disabled in release mode. Unchecked arithmetic can lead to silent overflows, producing incorrect results that can be exploited. To prevent this, all arithmetic operations on token amounts, counters, or any other critical value **must** use checked operations.

**Guideline: Always use `checked_*` methods for arithmetic and return a typed `Error` on overflow.**

This approach ensures that any overflow is explicitly handled as an error condition rather than causing a panic or producing an incorrect value.

#### ✅ Correct: Using `checked_add` with `ok_or`

This is the required pattern for all arithmetic that can potentially overflow. It converts the `Option` returned by `checked_add` into a `Result`, propagating a clear `Error::Overflow` if the operation fails.

```rust
// ✅ Safe and explicit — returns Err(Error::Overflow) on overflow
let new_count = count.checked_add(1).ok_or(Error::Overflow)?;
```

#### ❌ Incorrect: Using `saturating_*` methods

Saturating arithmetic should be avoided. While it prevents panics, it can silently mask overflow conditions by clamping the value at its maximum. This can lead to unexpected behavior and hide critical bugs.

```rust
// ❌ Unsafe — silently fails by clamping at the type's maximum value
let new_count = count.saturating_add(1);
```

#### ❌ Incorrect: Using raw arithmetic operators

Raw operators (`+`, `-`, `*`, `/`) will panic in debug builds but will silently wrap (overflow) in release builds. This is the most dangerous option and **must not** be used in production code.

```rust
// ❌ Unsafe — panics in debug, wraps in release
let new_count = count + 1;
```

By consistently applying the `checked_*` pattern, we ensure that our contracts are robust against integer overflow vulnerabilities and provide clear, actionable error information to callers.

### Reentrancy

Soroban contracts are single-threaded and do not support mid-execution callbacks in the same way EVM does. However, cross-contract calls can still cause unexpected state if not handled carefully:

- Write state **before** making cross-contract calls (checks-effects-interactions pattern).
- Validate return values from cross-contract calls.

### Replay Attacks (Triple-Check Pattern)

For cross-chain messages, enforce all three protections — **nonce uniqueness**, **expiration**, and
**chain binding** — in a single call. The project provides a shared `replay_protection` library
(`libs/replay_protection/`) used by all cross-chain contracts:

```rust
use replay_protection::{verify_replay_protection, ChainId};

fn submit_cross_chain_message(
    env: &Env,
    message_hash: &BytesN<32>,
    sender_key: &BytesN<32>,
    nonce: u64,
    timestamp: u64,
    ttl_secs: u64,
    source_chain: &ChainId,
    expected_chain: &ChainId,
) -> Result<(), ReplayError> {
    // Single call enforces all three guards
    verify_replay_protection(
        env, message_hash, sender_key,
        nonce, timestamp, ttl_secs,
        source_chain, expected_chain,
    )
}
```

For confirm/execute stages where nonce and chain were already checked at submission, use
the lighter `check_message_expired` helper instead.

### Denial of Service via Unbounded Loops

Avoid iterating over unbounded collections in a single transaction. Use pagination or batch limits:

```rust
// ❌ Unbounded — can exceed gas/instruction limits
for i in 0..total_alerts { ... }

// ✅ Bounded — caller controls batch size
let limit = count.min(MAX_BATCH_SIZE);
for i in 0..limit { ... }
```

---

## 3. Secure Coding Practices

### Validate all inputs

Check ranges, lengths, and invariants at function entry:

```rust
if threshold_bps == 0 || threshold_bps >= 10_000 {
    return Err(Error::InvalidThreshold);
}
if feature_count == 0 || feature_count > MAX_FEATURES {
    return Err(Error::InvalidFeatureCount);
}
```

### Use typed storage keys

Avoid string-based storage keys that can collide. Use a typed `DataKey` enum:

```rust
#[contracttype]
pub enum DataKey {
    Admin,
    Message(BytesN<32>),
    Nonce(String),
}
```

### Emit events for all state changes

Every state mutation should emit an event for off-chain auditability:

```rust
env.events().publish(
    (Symbol::new(&env, "RecordCreated"),),
    (record_id, actor, timestamp),
);
```

### Avoid `panic!` in production code

Use `Result<T, Error>` instead of `panic!`. Panics abort the transaction with an opaque error; typed errors give callers actionable information:

```rust
// ❌
if already_initialized { panic!("Already initialized"); }

// ✅
if already_initialized { return Err(Error::AlreadyInitialized); }
```

---

## 4. Cryptographic Guidance

- **Signatures**: Use `BytesN<64>` for Ed25519 signatures. Verify via Soroban's host functions, not custom implementations.
- **Hashes**: Use `BytesN<32>` for SHA-256 / Keccak-256 outputs. Do not truncate hashes.
- **Keys**: Never store private keys on-chain. Store only public keys or addresses.
- **Randomness**: Soroban does not provide on-chain randomness. Use commit-reveal schemes or oracle-provided randomness for any randomness requirement.
- **Encryption**: Encrypt sensitive data off-chain before storing on-chain. Store only ciphertext and the encryption key reference (not the key itself).

---

## 5. State Management Security

### Persistent vs. Instance vs. Temporary storage

| Type | Survives ledger close | Use for |
|---|---|---|
| `persistent` | Yes (with TTL) | Records, balances, long-lived state |
| `instance` | Yes (contract lifetime) | Admin, config, flags |
| `temporary` | No | Confirmations, short-lived locks |

Use `temporary` storage for data that should not persist (e.g., in-flight confirmations) to avoid state bloat.

### TTL management

Persistent storage entries expire. Extend TTLs for critical data:

```rust
env.storage().persistent().extend_ttl(&key, MIN_TTL, MAX_TTL);
```

---

## 6. Security Review Checklist

Before submitting a PR, verify:

- [ ] All admin/privileged functions call `require_auth()` and check roles
- [ ] No `panic!` or `unwrap()` in non-test code
- [ ] All arithmetic uses `checked_*` or `saturating_*`
- [ ] Nonces enforced for replay-sensitive operations
- [ ] Events emitted for all state changes
- [ ] Input validation at function entry
- [ ] No unbounded loops over storage
- [ ] Typed `DataKey` enum used (no raw string keys)
- [ ] Sensitive data encrypted before storage

---

## 7. Incident Response

If a vulnerability is discovered:

1. **Do not disclose publicly** until a fix is deployed.
2. Pause the affected contract immediately: `pause(env, admin)`.
3. Follow the incident postmortem process: `docs/INCIDENT_POSTMORTEM_GUIDELINES.md`.
4. Deploy a fix and verify via `scripts/verify_deployment.sh`.
5. Unpause and notify stakeholders.

For critical vulnerabilities, contact the security team directly before any public disclosure.

## 8. Deterministic Build Verification

Auditors review and sign a specific WASM binary. If the artifact that is
deployed (or later rebuilt) differs from the one that was audited, the entire
security model breaks: auditors signed one binary while users run another. To
prevent this, every release records the SHA-256 of each built `.wasm` and CI
verifies fresh builds against that audited record.

### Record hashes at release time

Build the release artifacts from the **pinned** toolchain (`rust-toolchain.toml`,
currently Rust 1.92.0) so output is reproducible, then record the hashes and
attach the auditor's signing key:

```sh
make dist                                   # builds .wasm into dist/
./scripts/verify_deployment.sh record mainnet v1.0.0 <auditor_pubkey>
git add deployments/mainnet/v1.0.0/hashes.txt
```

This writes `deployments/<network>/<release>/hashes.txt` containing the
toolchain version, a `Signed-by:` line with the auditor's pubkey, and one
`<sha256>  <artifact>` line per contract.

### Verify before/after deployment and in CI

```sh
make dist
./scripts/verify_deployment.sh compare mainnet v1.0.0
```

`compare` rebuilds, hashes, and diffs against the recorded set. It **exits
non-zero on any mismatch**, so it fails CI if a deployed/built artifact drifts
from the audited record. When no record exists yet for the target network and
release it is a safe no-op, so CI stays green until a release is recorded.

The CI `Build (wasm32)` job runs `compare` automatically for the current ref
against both `testnet` and `mainnet` records.

> **Tip:** keep `deployments/<network>/<release>/hashes.txt` immutable once an
> auditor has signed it. A new audited build means a new release directory, not
> an edit to an existing one.

---

## 10. Fuzz Testing for Byte-Input Functions

### Background

Several contracts in this repository accept untrusted byte sequences:
cross-chain messages, ZK proof blobs, encrypted payloads, and XDR-encoded
state. Without regular fuzzing, edge cases in parse/validate code can remain
hidden for years until exploited.

`SECURITY_CHECKLIST.md §9` requires fuzz tests for every `pub fn` accepting
`Bytes`, `Vec<u8>`, or raw message types.

### Architecture

Soroban contracts target `wasm32-unknown-unknown`, which does not support
libFuzzer. Fuzz targets are therefore compiled for `x86_64-unknown-linux-gnu`
(the CI host) using `soroban-sdk` testutils. **proptest** provides the primary
fuzzing engine — it generates shrinkable, coverage-guided property tests that
automatically minimize failing inputs.

```
tests/fuzz/
  zk_verifier/          # verify_proof(Bytes), compute_proof_hash(Bytes)
  zkp_registry/         # import_state(Bytes), submit_zkp(proof_data: Bytes),
                        #   verify_range_proof(RangeProof), ciphertext parsing
  cross_chain_bridge/   # validate_chain_address(String), confirm_message(BytesN<64>)
  meta_tx_forwarder/    # execute(ForwardRequest, signature: BytesN<64>)
```

Each directory is a standalone Rust crate (`[workspace]` in its own
`Cargo.toml`) excluded from the workspace `wasm32` build.

### Functions audited for byte-input risk

| Contract | Function | Byte-input parameter |
|----------|----------|---------------------|
| `zk_verifier` | `verify_proof` | `proof: Bytes` |
| `zk_verifier` | `compute_proof_hash` | `proof: Bytes` |
| `zkp_registry` | `submit_zkp` | `proof_data: Bytes` |
| `zkp_registry` | `create_range_proof` | `encrypted_value: Bytes`, `proof_data: Bytes` |
| `zkp_registry` | `create_credential_proof` | `encrypted_expiration: Bytes` |
| `zkp_registry` | `import_state` | `state_bytes: Bytes` (XDR deserialization) |
| `zkp_registry` | `verify_range_proof` | `RangeProof.proof_data`, `RangeProof.encrypted_value` |
| `cross_chain_bridge` | `validate_chain_address` | `address: String` |
| `cross_chain_bridge` | `confirm_message` | `signature: BytesN<64>` |
| `cross_chain_bridge` | `submit_proof` | `signature: BytesN<64>` |
| `meta_tx_forwarder` | `execute` | `signature: BytesN<64>` |
| `meta_tx_forwarder` | `execute_batch` | `signatures: Vec<BytesN<64>>` |

### Key invariants tested

1. **No panics**: every function that accepts arbitrary bytes must return `Ok`
   or a typed `Err`. Host traps caught by `try_*` client methods count as Err.
2. **`verify_proof` returns `false` without a valid attestation**: arbitrary
   bytes must never yield `Ok(true)` from an unattested proof.
3. **`compute_proof_hash` is total**: SHA-256 over any byte sequence must
   always succeed.
4. **`import_state` handles malformed XDR**: arbitrary bytes must return
   `InvalidInput`, never trap.
5. **Expired requests are rejected before signature verification**: avoids
   unnecessary crypto work on stale requests.

### Running fuzz tests locally

```sh
# zk_verifier (quick smoke test — default 500 cases)
cd tests/fuzz/zk_verifier && cargo test

# Long-duration session (adjust PROPTEST_CASES for time budget)
PROPTEST_CASES=10000 cargo test --release

# Specific target
cargo test verify_proof_unattested_returns_false

# All four targets in parallel
for target in zk_verifier zkp_registry cross_chain_bridge meta_tx_forwarder; do
  (cd tests/fuzz/$target && PROPTEST_CASES=5000 cargo test --release) &
done
wait
```

### CI schedule

`.github/workflows/fuzz.yml` runs nightly at 02:30 UTC with
`PROPTEST_CASES=5000` and a 5-minute (`timeout 300`) budget per target.
On any test failure (non-zero, non-timeout exit):

1. The full proptest log (including shrunk failing input) is uploaded as a
   workflow artifact retained for 30 days.
2. A GitHub issue is automatically filed with the label `security,fuzz-crash`
   and the shrunk reproducer embedded in the body.

A `timeout 124` exit (normal expiry after 5 minutes with no failures) is
treated as **success** — the fuzzer ran its budget and found nothing.

### Adding a new fuzz target

1. Identify any new `pub fn` that accepts `Bytes`, `Vec<u8>`, or raw message
   structs containing byte fields.
2. Add a proptest property to the appropriate `tests/fuzz/<contract>/tests/fuzz.rs`.
3. Update the table above.
4. Run locally and confirm the seed corpus passes before pushing.