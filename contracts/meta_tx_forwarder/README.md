# Meta-Transaction Forwarder (ERC-2771 Compatible for Soroban)

## Overview

The Meta-Transaction Forwarder enables **gasless transactions** for the Uzima
healthcare platform. Patients and doctors can interact with smart contracts
without holding native XLM for gas fees — a relayer submits the transaction
on their behalf after verifying an Ed25519 signature they produced off-chain.

This implementation:

* Implements an **ERC-2771-compatible flow**, adapted for Soroban's typed
  `(Symbol, Vec<Val>)` invocation model — the original sender is prepended as
  the first positional argument of every forwarded call.
* Verifies real **Ed25519 signatures** via the Soroban host's
  `env.crypto().ed25519_verify` primitive.
* Tracks per-user, monotonically-increasing **nonces** to prevent replay
  attacks.
* Enforces **deadlines** to prevent stale / front-run requests.
* Supports a registered **relayer set** with optional batched execution.
* Emits typed events for `init`, `reg_key`, `reg_relay`, `deact_rel`, `fwd`,
  and benchmarks direct vs. relayed CPU/memory costs in unit tests.

## Architecture

```
        User (off-chain)
         │
         ▼  signs XDR with Ed25519
        Relayer service (off-chain)
         │
         ▼  calls forwarder.execute(relayer, request, signature)
        MetaTxForwarder (Soroban)
         │  1. require_active_relayer(relayer)
         │  2. verify_signature(request, signature=ed25519_verify(pub_key, msg, sig))
         │  3. verify_nonce(request.from, request.nonce)
         │  4. require_deadline(request.deadline)
         │  5. forward_call(target, fn, [from, ...args])
         │  6. increment_nonce(request.from)
         ▼
        Target contract
         │  receives `from` as argument 0
         ▼
        End state mutation / event emission
```

Every forwarded call's first positional argument is `request.from` (the
original signer). Target contracts can either:

1. Branch on `env.invoker() == ERC2771ContextImpl::get_trusted_forwarder(&env)`
   and treat `from` as the authoritative sender, **or**
2. Accept `from` unconditionally as their first argument and trust the
   forwarder to populate it (most straightforward).

## Data Structures

### `ForwardRequest`

```rust
pub struct ForwardRequest {
    pub from: Address,        // Original sender
    pub to: Address,          // Target contract
    pub value: i128,          // Informational (Soroban token transfers use the token client)
    pub gas: u32,             // Informational gas-limit hint
    pub nonce: u64,           // Per-user nonce for replay protection
    pub deadline: u64,        // Unix-timestamp expiry
    pub data: Bytes,          // Reserved opaque bytes; unused by verifier
    pub target_fn: Symbol,    // Soroban contract function to invoke on `to`
    pub target_args: Vec<Val> // Arguments passed to `target_fn` (forwarder auto-prepends `from`)
}
```

### Signed payload byte layout

The contract signs and verifies exactly this byte sequence:

```
DOMAIN_PREFIX  (16 bytes)        = "UZM-MTX-v1\0\0\0\0\0\0"
TRUSTED_FORWARDER_ADDRESS_XDR     = env.current_contract_address().to_xdr(env)
FORWARD_REQUEST_BODY_XDR          = request.to_xdr(env)
```

The full message is the concatenation of these three slices. The user's
Ed25519 keypair must produce a 64-byte signature over this exact buffer.

> **Determinism guarantee.** Both the contract (`lib::verify_signature`) and
> the off-chain signer must produce the bytes in identically the same order
> with no additional padding, encoding, or hashing step.

### `RelayerConfig`

```rust
pub struct RelayerConfig {
    pub address: Address,
    pub is_active: bool,
    pub fee_percentage: u32,  // Fee in basis points (e.g., 100 = 1%)
}
```

## Core Functions

### `initialize(owner, fee_collector, min_relayer_stake) -> Result<(), Error>`

Initializes the forwarder with the owner and (informational) fee-collector.
Caller must be `owner`. Callable exactly once.

### `register_user_pub_key(user, pub_key: BytesN<32>) -> Result<(), Error>`

One-time registration binding a user's address to a 32-byte Ed25519 public
key. The user must `require_auth()`. Re-registration overwrites the previous
key.

### `get_user_pub_key(user) -> Option<BytesN<32>>`

Returns the registered public key, or `None` if `user` has not registered.

### `execute(relayer, request, signature: BytesN<64>) -> Result<Bytes, Error>`

Executes a meta-transaction. **`relayer.require_auth()`** is enforced. The
relayer must be in the registered-active set. The full execution order is:

1. `require_active_relayer`
2. `now <= request.deadline` (else `RequestExpired`)
3. `get_nonce(from) == request.nonce` (else `InvalidNonce`)
4. `ed25519_verify(pub_key, payload, signature)` (trap on mismatch)
5. `forward_call` invokes `request.target_fn` on `request.to` with
   `[from, ...request.target_args]`
6. `increment_nonce(from)`

Returns the XDR-encoded `Val` returned by the target contract.

### `execute_open(relayer, request, signature) -> Result<Bytes, Error>`

Same as `execute` but does **not** require a registered-active relayer.
Authentication is still enforced via `relayer.require_auth()`. Intended for
admin tooling and self-relaying; production code paths should use `execute`.

### `execute_batch(relayer, requests, signatures) -> Result<Vec<Bytes>, Error>`

Batched execution. All requests are processed sequentially. On first failure,
the already-completed requests have already mutated state and advanced
nonces; subsequent requests are not executed. Returns a vector of XDR-encoded
target return values, one per request.

### Relayer management

```rust
pub fn register_relayer(env, owner, relayer, fee_percentage) -> Result<(), Error>
pub fn deactivate_relayer(env, owner, relayer) -> Result<(), Error>
pub fn is_relayer(env, relayer) -> bool
pub fn get_relayer_config(env, relayer) -> Option<RelayerConfig>
pub fn get_nonce(env, user) -> u64
pub fn get_trusted_forwarder(env) -> Address
pub fn domain_separator(env) -> Bytes
```

## Integration with Target Contracts

Target contracts should accept the original sender as their first positional
argument and (optionally) check that the trusted forwarder is the immediate
caller:

```rust
use soroban_sdk::{contract, contractimpl, Address, Env};
use meta_tx_forwarder::erc2771_context::{ERC2771ContextImpl};

#[contract]
pub struct MyContract;

#[contractimpl]
impl MyContract {
    pub fn init(env: Env, admin: Address, forwarder: Address) {
        admin.require_auth();
        ERC2771ContextImpl::set_trusted_forwarder(&env, forwarder);
    }

    /// Called directly OR via the meta-tx forwarder.
    /// `from` is the original signer; for direct calls, it's `env.invoker()`.
    pub fn my_action(env: Env, from: Address, x: u32) -> u32 {
        // Authz: in a meta-tx flow, `from` has already authorized off-chain
        // via the Ed25519 signature — *do not* re-call `require_auth()`,
        // because only the relayer signs the inner transaction otherwise.
        // For direct calls, fall back to the standard Soroban auth model.
        let trusted = ERC2771ContextImpl::is_invoker_trusted(&env);
        if !trusted {
            from.require_auth();
        }
        x * 2
    }
}
```

A convenience helper for tests:

```rust
use meta_tx_forwarder::erc2771_context;

let msg_args: soroban_sdk::Vec<soroban_sdk::Val> = soroban_sdk::vec![&env, from.into_val(&env), 42u32.into_val(&env)];
let sender = erc2771_context::ERC2771ContextImpl::msg_sender_from_data(&env, &msg_args);
let rest   = erc2771_context::ERC2771ContextImpl::msg_data(&env, &msg_args);
```

## Security Considerations

### Signature Verification

* Signatures use the host-native `env.crypto().ed25519_verify`. Cryptographic
  verification traps on mismatch — there is no soft-failure return path.
* The signed payload includes a 16-byte domain separator (`"UZM-MTX-v1\0\0\0\0\0\0"`)
  and the forwarder's own contract address, preventing cross-forwarder and
  cross-version replay.

### Nonce Management

* Each user has a monotonic `u64` nonce. Off-chain signers must query
  `get_nonce(addr)` immediately before producing a request — there is no
  "future reservation" mechanism. The next request must be exactly
  `current + 1`.

### Deadline Enforcement

* `deadline < ledger.now` is rejected with `RequestExpired`. A typical
  deadline is current-time-plus-one hour to give the relayer ample time to
  submit and the user time to detect a loss of UX.

### Trusted Forwarder Pattern

* The forwarder stores its own address under `DataKey::TrustedForwarder`.
  Target contracts should read it via
  `ERC2771ContextImpl::get_trusted_forwarder(&env)` and verify `env.invoker()
  == trusted` before trusting a forwarded `from`.

### Relayer Authorization

* Only registered-active relayers can submit via `execute`. The fee-collector
  and minimum stake are recorded but informational; relays operate off-chain
  and the on-chain guarantees do not depend on enforced economics.

## Relayer Economics

This contract does not move tokens. Relayer fees are settled off-chain in
the user's choice of payment rail. The on-chain side:

* `RelayerConfig::fee_percentage` (basis points, up to 10 000) is the
  relayer's published rate.
* `fee_collector` is the destination the **user** pays in the off-chain
  arrangement.

## Deployment Guide

### 1. Build

```bash
cargo build --target wasm32-unknown-unknown --release \
    -p meta_tx_forwarder
```

### 2. Deploy

```bash
soroban contract deploy \
  --wasm target/wasm32-unknown-unknown/release/meta_tx_forwarder.wasm \
  --network testnet
```

### 3. Initialize

```bash
soroban contract invoke \
  --id <FORWARDER_CONTRACT_ID> \
  --network testnet \
  -- initialize \
  --owner <OWNER_ADDRESS> \
  --fee_collector <FEE_COLLECTOR_ADDRESS> \
  --min_relayer_stake 1000000000
```

### 4. Register a Relayer

```bash
soroban contract invoke \
  --id <FORWARDER_CONTRACT_ID> \
  --network testnet \
  -- register_relayer \
  --owner <OWNER_ADDRESS> \
  --relayer <RELAYER_ADDRESS> \
  --fee_percentage 100
```

### 5. Have your user register their public key (one-time)

```bash
soroban contract invoke \
  --id <FORWARDER_CONTRACT_ID> \
  --network testnet \
  -- register_user_pub_key \
  --user <USER_ADDRESS> \
  --pub_key <32_BYTE_HEX_PUBLIC_KEY>
```

### 6. Have your target contract accept the forwarded sender

Add a function whose first argument is `from: Address` and store the
trusted forwarder address via `ERC2771ContextImpl::set_trusted_forwarder`.

## Testing

```bash
cargo test -p meta_tx_forwarder                 # unit tests inside the crate
cargo test -p uzima-tests -- meta_tx            # integration + benchmarks
```

The integration tests use real `ed25519_dalek` key pairs. The benchmarks
print a `[BENCH] relayed-vs-direct CPU factor: …` line that exposes the
extra cost of the verification + dispatch loop relative to a direct
authenticated call.

## Usage Example (patient grants record access to a doctor, gasless)

### Off-chain: patient constructs and signs the request

```typescript
import { Keypair, Address, nativeToScVal, scvalToBigInt } from "@stellar/stellar-sdk";
import { Buffer } from "buffer";

// 1. Build the request
const now = Math.floor(Date.now() / 1000);
const request = {
  from: patientAddress,                 // user's on-chain Address
  to: targetContractAddress,            // contract to invoke
  value: 0n,
  gas: 100000,
  nonce: await getCurrentNonce(forwarder, patientAddress),  // read on-chain
  deadline: BigInt(now + 3600),         // +1h safety margin
  data: Buffer.alloc(0),                // reserved / unused
  targetFn: "grant_access",             // function name on `to`
  targetArgs: [nativeToScVal(doctorAddress), nativeToScVal(60 * 60 * 24)], // args after `from`
};

// 2. Serialize to canonical XDR using the same layout as `ForwardRequest::to_xdr`
const reqXdr = scvalToXdr(request);    // contracttype layout on Soroban 21.7.7

// 3. Prepend the domain separator and trusted forwarder address
const domainSep = Buffer.from("UZM-MTX-v1\0\0\0\0\0\0", "utf8");        // 16 bytes
const trustedFwdXdr = scvalToXdr(forwarderAddress);
const payload = Buffer.concat([domainSep, trustedFwdXdr, reqXdr]);

// 4. Sign with the patient's Ed25519 key
const signer = Keypair.fromSecret(patientSecret);
const signature = signer.sign(payload);  // 64 raw bytes

// 5. Hand (request, signature, signerPublicKey) to the relayer
await relayer.submit({ request, signature });
```

### On-chain: relayer invokes the forwarder

```bash
soroban contract invoke \
  --id <FORWARDER_CONTRACT_ID> \
  --network testnet -- \
  execute \
  --relayer <RELAYER_ADDRESS> \
  --request '{"from":"...","to":"...","value":0,"gas":100000,"nonce":0,"deadline":...,"data":"","target_fn":"grant_access","target_args":[...]}' \
  --signature <64_BYTE_HEX_SIG>
```

On success, the patient's nonce on the forwarder advances by one, and the
target contract's `grant_access(from=patientAddress, doctorAddr, ttl)`
executes with the forwarder as `env.invoker()` and `from` set to
`patientAddress`. The patient pays only the (off-chain) relay fee.

## Events

| Topic              | Shape                                          |
| ------------------ | ---------------------------------------------- |
| `init`             | `(owner, fee_collector, min_relayer_stake)`    |
| `reg_key`          | `(user, pub_key_bytes)`                        |
| `reg_relay`        | `(relayer, fee_percentage)`                    |
| `deact_rel`        | `(relayer,)`                                   |
| `fwd`              | `(relayer, from, to, nonce)`                   |

All events can be queried from the Soroban RPC `getEvents` endpoint and
indexed by topic.

## Benchmarks

Run `cargo test -p uzima-tests -- meta_tx_benchmarks`. The headline test
prints:

```
[BENCH]            direct_target_call cpu=    … insns  mem=    … bytes  wall=    … µs
[BENCH] relayed_call_via_forwarder cpu=    … insns  mem=    … bytes  wall=    … µs
[BENCH] relayed-vs-direct CPU factor:  …  (extra cost of meta-tx)
```

The factor is essentially the cost of an Ed25519 signature verification plus
a nonce read/write and a single `env.invoke_contract` dispatch, plus the
forwarder's own signature prepending.

## Future Enhancements

* [ ] EIP-712-style typed-domain separation (currently the contract uses a
      hard-coded `"UZM-MTX-v1"` prefix).
* [ ] Per-user key rotation (currently registration overwrites).
* [ ] On-chain fee settlement in stable tokens.
* [ ] Relayer reputation tracking (analogous to the `reputation` contract).

## References

* [ERC-2771: Secure Protocol for Native Meta Transactions](https://eips.ethereum.org/EIPS/eip-2771)
* [OpenZeppelin MinimalForwarder](https://docs.openzeppelin.com/contracts/4.x/api/metatx)
* [Soroban Documentation](https://soroban.stellar.org/docs)
* [Ed25519-Dalek](https://docs.rs/ed25519-dalek/latest/ed25519_dalek/) — used
  for off-chain signing.

## License

MIT © 2025 Stellar Uzima Contributors
