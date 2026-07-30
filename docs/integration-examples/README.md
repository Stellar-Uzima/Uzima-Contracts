# Uzima Contract Integration Examples

Practical code examples for integrating with Uzima Soroban smart contracts
across multiple languages and SDKs.

## Contracts Covered

| Contract | Description |
|---|---|
| `medical_records` | Create, read, and manage patient medical records |
| `healthcare_payment` | Submit claims, verify eligibility, process payments |
| `patient_consent_management` | Grant and revoke patient data access consent |

## Language Guides

| Language | SDK | Guide |
|---|---|---|
| Rust | `soroban-sdk` | [Rust Examples](./rust/README.md) |
| TypeScript | `@stellar/stellar-sdk` | [TypeScript Examples](./typescript/README.md) |
| Python | `stellar-sdk` | [Python Examples](./python/README.md) |

## Prerequisites

- A Stellar network passphrase (testnet or standalone)
- A funded account for transaction fees
- Compiled `.wasm` contract binaries or deployed contract IDs
- RPC/URL endpoint (e.g., `https://soroban-testnet.stellar.org`)

## Common Patterns

All examples follow these conventions:

1. **Authentication** — The caller must authorize the transaction with `TxEnvelope` signing.
2. **Error handling** — Contract errors are caught and decoded to human-readable messages.
3. **Pagination** — List queries accept `offset` and `limit` parameters for cursor-based pagination.
4. **Input validation** — External data is validated client-side before submission.

## Network Configuration

Examples default to **testnet**. To target standalone:

```bash
export STELLAR_NETWORK_PASSPHRASE="Standalone Network ; September 2015"
export STELLAR_RPC_URL="http://localhost:8000"
```
