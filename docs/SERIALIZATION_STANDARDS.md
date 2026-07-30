# Serialization and XDR Handling Standards

This document defines the canonical serialization approach for all Uzima
contracts and SDKs, ensuring consistent encoding across Rust contracts,
TypeScript, and Python clients.

## Principles

1. **Use Soroban SDK types** (`Bytes`, `BytesN<N>`, `String`, `Vec`, `Map`) for
   all on-chain data — never raw `u8` slices or std collections.
2. **`#[contracttype]` for all structured data** that crosses the contract
   boundary (arguments, return values, storage values, events).
3. **XDR is the wire format** — all encoding/decoding must go through
   `soroban_sdk` which handles XDR automatically.
4. **No custom serialization** — do not implement manual byte packing.
5. **String lengths must be bounded** — always validate with `max_len` before
   storing.

## Canonical Type Mappings

| Use case | Soroban type | Notes |
|----------|-------------|-------|
| Patient/Provider address | `Address` | Always use `Address`, never `String` |
| Record identifier | `u64` | Monotonically increasing |
| Cryptographic hash (32 bytes) | `BytesN<32>` | WASM hashes, ZKP commitments |
| Arbitrary binary data | `Bytes` | Encrypted payloads, IPFS hashes |
| Short human-readable labels | `Symbol` | Max 32 chars, no spaces |
| Long human-readable strings | `String` | With validated max length |
| Timestamp / ledger sequence | `u64` / `u32` | Prefer ledger seq for on-chain |
| Enumerated state | `#[contracttype] enum` | Never raw u32 in ABI |
| Structured data | `#[contracttype] struct` | For all cross-boundary types |

## XDR Edge Cases and Known Pitfalls

### 1. `Option<T>` encoding

`Option<T>` serialises as `SCVal::Void` for `None` and `SCVal::T` for `Some(T)`.
Do **not** use sentinel values (e.g. `u64::MAX` for "no value") — use `Option`.

### 2. `Map` ordering

`soroban_sdk::Map` preserves insertion order in XDR. When comparing maps across
versions, always compare keys explicitly rather than byte-level equality.

### 3. `BytesN` vs `Bytes`

- `BytesN<N>` — fixed-length, stack-friendly, use for hashes and keys
- `Bytes` — variable-length, use for encrypted payloads

### 4. Large payloads

Soroban has a 64 KB entry size limit. For large medical record payloads:
- Store an IPFS/HTTPS content-addressed hash on-chain (`BytesN<32>` or `Bytes`)
- Store the actual data off-chain encrypted

### 5. Cross-SDK deserialization

When deserialising from TypeScript/Python SDKs:
- `Address` → Stellar strkey (G... for accounts, C... for contracts)
- `BytesN<32>` → 32-byte hex string or base64 depending on SDK
- `Symbol` → plain string, max 32 chars

## SDK Serialization Reference

### Rust (Soroban)

```rust
// Canonical: use contracttype for all structured ABI types
#[contracttype]
#[derive(Clone)]
pub struct MedicalRecord {
    pub id: u64,
    pub patient: Address,
    pub data_hash: BytesN<32>,
    pub category: Symbol,
    pub ledger: u64,
}

// Canonical: bounded string validation
pub fn validate_string_len(s: &soroban_sdk::String, max_len: u32) -> Result<(), CommonError> {
    if s.len() > max_len {
        return Err(CommonError::InvalidInput);
    }
    Ok(())
}
```

### TypeScript SDK

```typescript
// Decode a contract return value
import { scValToNative } from '@stellar/stellar-sdk';

const record = scValToNative(invokeResult);
// record.patient is a string (G... strkey)
// record.data_hash is a Buffer (32 bytes)
// record.category is a string
```

### Python SDK

```python
from stellar_sdk.soroban import SorobanServer
from stellar_sdk.xdr import SCVal

# Decode address
address = SCVal.from_xdr(raw_val).address.account_id.ed25519
```

## Validation Helpers

See `libs/validation_utils/src/string.rs` for:
- `validate_string_length()` — checks against configured max
- `validate_symbol()` — ensures Symbol constraints

See `libs/validation_utils/src/address.rs` for:
- `validate_address()` — basic address sanity checks
