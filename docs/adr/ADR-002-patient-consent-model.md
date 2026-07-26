# ADR-002: Patient consent model — explicit on-chain authorisation

**Status:** Accepted  
**Date:** 2025-01-15

## Context
Medical records access requires patient consent. Options:
1. Off-chain consent stored in a database.
2. On-chain consent via `require_auth()` on every sensitive call.

## Decision
Use Soroban's `Address::require_auth()` for all patient-data operations.

## Rationale
- **Auditability** — every consent event is recorded on-chain and immutable.
- **No oracle dependency** — consent is verified by the network, not a
  centralised service that could be compromised.
- **Simplicity** — `require_auth()` is a single line; no custom ACL tables.

## Consequences
- Patients must sign transactions, requiring a Stellar wallet.
- Batch operations on behalf of a patient require multi-auth or delegated
  signing, which adds UX complexity.
- Off-chain consent caching is not possible without additional trust assumptions.