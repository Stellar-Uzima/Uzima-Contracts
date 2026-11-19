# Roadmap

This roadmap is derived only from [`docs/TECHNICAL_DEBT.md`](docs/TECHNICAL_DEBT.md)
and [`docs/ONRAMP_OWNERSHIP.md`](docs/ONRAMP_OWNERSHIP.md). Those documents are the
source of truth; update them first, then this file. Target quarters are the ones
recorded in the technical-debt catalog. Items with no recorded quarter are listed
as unscheduled rather than guessed.

## Technical debt (from the catalog)

| Target quarter | ID | Module | Item | Status |
|---|---|---|---|---|
| Q3 2024 | TD-001 | `zkp_registry` | Replace simulated ZKP verification in `verify_zkp_internal` with real cryptographic verification | Open |
| Q3 2024 | TD-002 | `zkp_registry` | Implement proper expiration decryption and validation in `create_credential_proof` | Open |
| Q3 2024 | TD-003 | `zkp_registry` | Replace simulated range proof verification in `verify_range_proof_internal` | Resolved |
| Q4 2024 | TD-004 | `zkp_registry` | Replace simulated recursive proof verification in `verify_recursive_proof_internal` | Open |

TD-001, TD-002 and TD-004 are High impact / High priority security items and are
past their recorded target quarters. Re-scheduling them is a decision for the
next quarterly debt review (see below); this roadmap does not assign new dates.

## On-ramp of excluded contracts (issue #828)

The ownership map lists every crate in the root `Cargo.toml` `workspace.exclude`
list with an owner, assignee, blocker and status. It records no target quarter, only
the target "#828 on-ramp".

- Every row currently has owner `@Stellar-Uzima/maintainers`, assignee `unassigned`.
- Most rows are `deferred`; blockers are either the workspace-resolution problem
  for `soroban-sdk = { workspace = true }` or a triage note in `Cargo.toml`.
- Cleanup surfaced by the map: `contracts/src` is a `stale-entry` (remove from
  `exclude`); `contracts/governance_integration_tests` and `contracts/storage-snapshot`
  are `not-a-crate`.
- Next step per the process: assign an owner per contract, then move rows to
  `in-progress` and `reintegration-ready` as described in
  [`docs/ONRAMP_OWNERSHIP.md`](docs/ONRAMP_OWNERSHIP.md).

## Review cadence

- Technical debt: reviewed at the start of each quarter (Jan, Apr, Jul, Oct).
- On-ramp map: reviewed at least quarterly in the issue #828 triage pass.
