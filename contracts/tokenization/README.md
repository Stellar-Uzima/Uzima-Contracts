# Tokenization

Soroban contract that records tokenized assets and tracks fractional ownership
of each asset's supply across holders.

| | |
| --- | --- |
| Crate | `contracts/tokenization` |
| Access control | `src/access_control.rs`, audited in [`SECURITY_AUDIT.md`](SECURITY_AUDIT.md) |
| Tests | `src/test.rs` (behaviour + access control), `src/invariant_tests.rs` (state-machine invariants), `tests/flows.rs` (end-to-end through the public interface) |

## Roles

| Role | Capabilities |
| --- | --- |
| `DEFAULT_ADMIN_ROLE` | Register assets, rotate admin, grant and revoke roles, pause/unpause |
| `ISSUER` | Mint and burn, and only on assets whose recorded `issuer` is the caller |
| Holder | Transfer units it holds |

## OpenZeppelin primitives (#1761)

`src/access_control.rs` implements role-based access control modelled on
OpenZeppelin's `Ownable` + `AccessControl`, deliberately **in-tree** rather than
as a dependency. The workspace pins `soroban-sdk = "=21.7.7"` and carries no
OpenZeppelin Soroban crate; adding one changes the dependency graph for every
contract in the workspace, and OpenZeppelin's Soroban contracts are still
pre-1.0. That is a decision for maintainers, not a side effect of a hardening
PR.

The module is written to be drop-in replaceable:

| This module | OpenZeppelin equivalent |
| --- | --- |
| `DEFAULT_ADMIN_ROLE`, `ISSUER_ROLE` | `DEFAULT_ADMIN_ROLE`, custom role constants |
| `has_role` / `require_role` | `has_role` / `only_role` |
| `grant_role` / `revoke_role` / `renounce_role` | `grant_role` / `revoke_role` / `renounce_role` |
| `role_members` | `get_role_members` (enumeration variant) |

Swapping the implementation for the real crate later is a `use` change, not a
rewrite.

Two deviations from stock OpenZeppelin, both deliberate:

1. **Last-admin protection.** `revoke_role` refuses to remove `DEFAULT_ADMIN_ROLE`
   from its last holder (`CannotRevokeLastAdmin`). Stock OZ lets an admin renounce
   itself into a bricked contract; on a registry that would mean no pause and no
   upgrade path.
2. **Grant before revoke.** `transfer_admin` grants the successor the role before
   revoking the incumbent, so there is never an instant with zero admins.

## Integration tests (#1762)

`tests/flows.rs` lives outside the crate and reaches the contract only through
what `lib.rs` re-exports, so it doubles as a public-surface regression check:
if a flow there stops compiling, the exported API has regressed even though the
unit tests still pass. Four end-to-end scenarios — issuance lifecycle with
accounting reconciliation, admin handover, rejection paths asserting the ledger
is untouched, issuer-role withdrawal, and pause.

## Access control audit (#1763)

[`SECURITY_AUDIT.md`](SECURITY_AUDIT.md) records the trust model, five findings
and their fixes, an entry-point-to-negative-test coverage map, and the residual
risk that remains accepted. Summary:

- **F1** Supply creation was gated only by asset ownership; `ISSUER_ROLE` now
  gates it, so that power is separately auditable and revocable.
- **F2** Authority was resolved from a single mirrored slot; it is now derived
  from role membership and cross-checked, so the two cannot drift.
- **F3** Nothing stopped an admin from revoking its own admin role and bricking
  the contract; now refused.
- **F4** `renounce_role` could orphan the role-member index, which would have
  defeated the F3 guard; both are cleared together.
- **F5** Pause does not block queries — accepted, and recorded as a decision.

Tests that assert *which* address had to authorize use `env.auths()` rather than
relying on `mock_all_auths()`, because the audit question is "whose signature was
demanded", not "did the call succeed".

## Invariants (#1764)

`src/invariant_tests.rs` asserts properties that must hold for **any** path
through the state machine, rather than for documented flows:

- balances of tracked holders sum to reported supply
- supply moves only by the explicit mint/burn amount; transfers are value-preserving
- no balance can exceed reported supply
- rejected calls leave state byte-identical
- the asset record and the `total_supply` query can never disagree
- asset count is monotone and re-registration is rejected
- exactly one admin exists after any transfer
- withdrawing `ISSUER_ROLE` halts all supply creation; restoring it resumes from
  the ledger, not from zero
- pause is total: no role, however privileged, can move state while it holds

## Running

```bash
cargo test -p tokenization              # unit + invariant tests
cargo test -p tokenization --test flows # end-to-end flows
cargo clippy -p tokenization --all-targets -- -D warnings
```