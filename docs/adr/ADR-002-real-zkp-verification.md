# ADR-002: Real ZKP Verification

**Status:** Accepted
**Date:** 2025-06-29

## Context

The repository contains a `zkp_registry` contract and related test suites for
range proof behavior. Governance and healthcare workflows rely on privacy claims
being verifiable rather than simulated.

Simulated verification is useful during early scaffolding, but it creates a
misleading security boundary: callers may believe a cryptographic proof was
validated when only placeholder logic ran. That is especially risky in a
healthcare-oriented protocol, where privacy claims and access decisions need a
clear audit trail.

Issue #829, "Replace Simulated ZKP Verification with Real Cryptographic
Verification", is the decision point this ADR records.

## Decision

ZKP verification paths must use real cryptographic verification instead of
simulation or test-only acceptance logic.

The production contract boundary should distinguish three cases:

- valid proof accepted
- invalid proof rejected with a typed error
- malformed or unsupported proof rejected before it can affect state

Tests should cover positive and negative proof paths, including range proof
negative cases, so future changes cannot silently reintroduce permissive
placeholder behavior.

## Consequences

Security expectations become clearer for off-chain clients and auditors: a
successful verification result means the proof was actually checked.

The implementation becomes more complex than simulated logic. Developers need to
maintain deterministic tests, fixtures, and negative cases so proof validation
remains reviewable.

Contract size, cost, and dependency constraints must be monitored. If a proof
system cannot be supported within Soroban limits, that constraint should be
captured in a new ADR rather than bypassed with simulation.

## References

- `contracts/zkp_registry/`
- `contracts/zkp_registry/tests/range_proof_negatives_tests.rs`
- `contracts/zkp_registry/tests/range_proof_property_tests.rs`
- Related issue: #829
