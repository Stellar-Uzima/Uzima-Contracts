# Identity Registry — Lazy Loading for DID Document Resolution

## Problem
Every verification call loads the full DID document even when only the public
key is needed, wasting compute and storage budget.

## Solution: Split core vs extended storage

```
DataKey::DidCore(Address)     -> DidCore     (public key + status only)
DataKey::DidExtended(Address) -> DidExtended (full document, services, etc.)
```

Verification calls read only `DidCore`. Full document resolution reads both.

## Proposed types

```rust
#[contracttype]
pub struct DidCore {
    pub public_key: BytesN<32>,
    pub status:     DidStatus,   // Active / Deactivated
    pub created:    u64,
}

#[contracttype]
pub struct DidExtended {
    pub document:  String,   // JSON-LD DID document (IPFS CID or inline)
    pub services:  Vec<ServiceEndpoint>,
    pub updated:   u64,
}
```

## Impact
- `verify_did` reads 1 small entry instead of the full document.
- `resolve_did` reads both entries (unchanged cost for full resolution).
- Reduces per-verification ledger fee by ~60–80% for large DID documents.