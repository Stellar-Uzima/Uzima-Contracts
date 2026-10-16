# Property Registry

Soroban contract: on-chain registry of property records — legal owner of record,
recorded valuation, and fractional ownership shares in basis points.

| | |
| --- | --- |
| Crate | `contracts/property_registry` |
| Access control | `src/access_control.rs`, audited in [`SECURITY_AUDIT.md`](SECURITY_AUDIT.md) |
| Tests | `src/test.rs` (behaviour + access control), `src/invariant_tests.rs` (state-machine invariants), `tests/flows.rs` (end-to-end through the public interface) |

## Roles

| Role | Capabilities |
| --- | --- |
| `DEFAULT_ADMIN_ROLE` | Rotate admin and registrar, grant and revoke roles, pause/unpause |
| `REGISTRAR` | Register properties, revalue, record ownership shares |
| Owner of record | Transfer the property, authorize debits from their own position |

The three gates are deliberately distinct: `REGISTRAR` says *who may amend the
register*, *owner of record* says *whose position may be debited*. Neither alone
is sufficient for `record_ownership`.

## OpenZeppelin primitives (#1777)

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
| `DEFAULT_ADMIN_ROLE`, `REGISTRAR_ROLE` | `DEFAULT_ADMIN_ROLE`, custom role constants |
| `has_role` / `require_role` | `has_role` / `only_role` |
| `require_admin` / `require_registrar` | modifier-style guards |
| `grant_role` / `revoke_role` / `renounce_role` | `grant_role` / `revoke_role` / `renounce_role` |
| `role_members` | `get_role_members` (enumeration variant) |

Swapping the implementation for the real crate later is a `use` change, not a
rewrite.

Two deviations from stock OpenZeppelin, both deliberate:

1. **Last-admin protection.** `revoke_role` refuses to remove `DEFAULT_ADMIN_ROLE`
   from its last holder (`CannotRevokeLastAdmin`). Stock OZ lets an admin
   renounce itself into a bricked contract; on a registry that would mean no
   pause, no registrar rotation and no upgrade path.
2. **Grant before revoke.** `transfer_admin` grants the successor the role before
   revoking the incumbent, so there is never an instant with zero admins.

The registrar slot (`DataKey::Registrar`, surfaced as `registrar()`) is retained
alongside the role and is checked by `require_registrar_role`. It records *who the
registry was configured with*; the role records *who may currently amend*. Both
are enforced so a rotation that updates only the slot cannot silently do nothing.

## Integration tests (#1778)

`tests/flows.rs` lives outside the crate and reaches the contract only through
what `lib.rs` re-exports, so it doubles as a public-surface regression check: if
a flow there stops compiling, the exported API has regressed even though the
unit tests still pass. Four end-to-end scenarios — register lifecycle with share
reconciliation, admin handover, rejection paths asserting the register is
untouched, registrar-role withdrawal, and pause.

## Access control audit (#1779)

[`SECURITY_AUDIT.md`](SECURITY_AUDIT.md) records the trust model, six findings
and their fixes, an entry-point-to-negative-test coverage map, and the residual
risk that remains accepted. Summary:

- **F1** The registrar slot was the whole access model; there was no way to add a
  second registrar or withdraw one without rotating the slot. `REGISTRAR_ROLE`
  now gates every amendment.
- **F2** Authority was a single mirrored slot, so the authorization target and the
  record of who is in charge could not be checked independently. Authority is
  now derived from role membership and cross-checked.
- **F3** Nothing stopped an admin revoking its own admin role and bricking the
  contract. Now refused.
- **F4** `renounce_role` could orphan the role-member index, which would have
  defeated the F3 guard. Both are cleared together.
- **F5** `holder_count` could drift if the transitions were driven off the
  argument rather than the share read. Both are driven off the share.
- **F6** Pause does not block queries — accepted, and recorded as a decision.

Tests that assert *which* address had to authorize use `env.auths()` rather than
relying on `mock_all_auths()`, because the audit question is "whose signature was
demanded", not "did the call succeed".

## Invariants (#1780)

`src/invariant_tests.rs` asserts properties that must hold for **any** path
through the state machine, rather than for documented flows:

- the ownership split always totals exactly 100%
- no single share can exceed 100%
- the owner of record is always a holder
- `holder_count` tracks distinct holders and a repeat transfer never double-counts
- property count is monotone and re-registration is rejected
- revaluation never disturbs the ownership split
- rejected calls leave the register byte-identical
- exactly one admin exists after any transfer
- withdrawing `REGISTRAR` halts amendments; restoring it resumes from the ledger
- pause is total: no role, however privileged, can move state while it holds

## Running

```bash
cargo test -p property_registry               # unit + invariant tests
cargo test -p property_registry --test flows  # end-to-end flows
cargo clippy -p property_registry --all-targets -- -D warnings
```