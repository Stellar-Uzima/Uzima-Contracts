# ADR-001: Use Soroban (Stellar) over alternative smart-contract platforms

**Status:** Accepted  
**Date:** 2025-01-01

## Context
The project needed a smart-contract platform for healthcare payment processing.
Candidates considered: Ethereum/EVM, Solana, Soroban (Stellar).

## Decision
Use Soroban on the Stellar network.

## Rationale
- **Deterministic fees** — Stellar's fee model is predictable, critical for
  healthcare billing where cost certainty matters.
- **Built-in asset primitives** — SEP-41 tokens map naturally to payment flows
  without custom ERC-20 boilerplate.
- **WASM sandbox** — Soroban's WASM execution environment is auditable and
  size-constrained (64 KB), encouraging lean contracts.
- **No gas estimation surprises** — fixed resource fees simplify UX for
  non-crypto-native healthcare operators.

## Consequences
- Team must learn Rust + Soroban SDK.
- Ecosystem tooling (wallets, explorers) is smaller than Ethereum.
- WASM 64 KB limit requires careful dependency management.