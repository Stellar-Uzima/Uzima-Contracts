# Property-Based Tests for ZK Access Control

This document records the property-based tests that guard the ZK access-control
invariants defined in [`formal-notes.md`](./formal-notes.md). These tests are
implemented with [`proptest`](https://github.com/proptest-rs/proptest) 1.6 and
ship alongside the deterministic unit tests so crypto regressions can be caught
both by known-corpus cases and by random fuzzing.

## Source

| Contract crate            | Test file                                                |
| ------------------------- | -------------------------------------------------------- |
| `contracts/medical_records` | `contracts/medical_records/tests/zk_property_tests.rs`   |

The tests are wired into the workspace's standard `cargo test` flow — `make test`
or `cargo test -p medical_records` will run them.

## Coverage Matrix

Each Core Invariant in `formal-notes.md` is mapped to exactly one property test:

| Invariant | Test function                                    | Failure mode asserted                          |
| --------- | ------------------------------------------------ | ---------------------------------------------- |
| 1. Access Safety (Z gate) | `prop_enforcement_toggle_preserves_expected_behavior` | toggle flips behaviour: enforced→rejected when ACL-passing lacks grant; bypassed→allowed |
| 3. Nullifier Uniqueness   | `prop_nullifier_uniqueness_replay_always_fails` | replay yields `Error::CredentialRevoked`       |
| 4. Root Consistency       | `prop_root_mismatch_always_fails`               | mismatch yields `Error::InvalidCredential`     |
| 5. Commitment Binding     | `prop_record_commitment_mismatch_always_fails`  | mismatch yields `Error::InvalidCredential`     |
| 5. Grant expiration       | `prop_grant_expiration_invalidates_after_ttl`   | past TTL yields `Error::InvalidCredential`     |

Invariant 2 (Grant Integrity — ZK grant exists only after successful
`submit_zk_access_proof`) is implicit across all property tests: each test
exercises the contract flow that creates / consumes a grant, so a regression in
the grant-creation path is observable as a failure in one of the other tests.

## Strategy Choices

`BytesN<N>` is `Env`-bound on Soroban, so the proptest strategies only yield
plain Rust primitives (`u8`, `u64`, `[u8; 32]`, `bool`). All contract-bound
types are constructed inside each test body. This keeps strategies `'static`
while still allowing us to vary the *content* of byte arrays.

For signed-bit tests (`prop_root_mismatch_always_fails`,
`prop_record_commitment_mismatch_always_fails`) we deterministically build the
"wrong" value by XOR'ing at least one byte of a stored value with a proptest
input — guaranteeing the wrong value differs from the stored value even under
degenerately small inputs without depending on `prop_assume`/filtering.

## Throttling

Each `proptest!` block uses `cases: 16` (or `8` for the toggles) with
`max_shrink_iters: 0`. The total run-time of the file is well under a few
seconds in CI, but each case still exercises a fresh Soroban `Env`, several
contract deployments, and an end-to-end ZK submission path. Crank these up
locally with the `PROPTEST_CASES` environment variable to do nightly fuzzing.

## Adding New Property Tests

When introducing a property test for a new ZK invariant:

1. Add a row to the matrix above.
2. Keep strategies `Env`-agnostic (return owned `Vec<u8>` / `[u8; 32]` / `u64`).
3. Construct `BytesN<32>` / `Address` inside the test body.
4. Use `prop_assert_eq!` against the exact `Error` variant — generic
   `prop_assert!(result.is_err())` is not strong enough to catch regressions.
5. Set `max_shrink_iters: 0` unless a counter-example is likely to be
   interesting for a human reader.
