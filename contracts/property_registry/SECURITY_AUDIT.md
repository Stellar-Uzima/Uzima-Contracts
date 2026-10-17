# Access Control Audit — `property_registry`

Scope: issue **#1779** (Audit access control in PropertyRegistry).

Reviewed surface: every entry point on `PropertyRegistryContract` that changes
or reads privileged state, plus the role bookkeeping in `src/access_control.rs`.

## Trust model

| Actor | Trust level | Why |
| --- | --- | --- |
| `DEFAULT_ADMIN_ROLE` holder | Fully trusted | Rotates admin and registrar, grants and revokes roles, pauses. Chooses the initial registrar, so it can appoint an address that can amend the register. Inherent to administering a registry; called out rather than prevented. |
| `REGISTRAR` holder | Trusted for amendments | Registers properties, revalues, records ownership share changes — but cannot move a property without the owner of record also authorizing. |
| Owner of record | Trusted for their own property | Transfers the property, and authorizes debits from their own position. |
| Anyone | Untrusted | Read-only queries. |

The three gates are deliberately distinct. `REGISTRAR` says *who may amend the
register*; *owner of record* says *whose position may be debited*. Neither alone
is sufficient for `record_ownership`, and collapsing them would be the single
most damaging change available to this contract — a compromised registrar could
otherwise strip every property in the registry.

## Findings

### F1 — The registrar slot was the whole access model *(fixed in this PR)*

Every registrar-gated entry point read `DataKey::Registrar` and authorized *that*
address. There was no role index, so the slot was the entire access model and
nothing else could be expressed — in particular there was no way to add a second
registrar, or to withdraw one without rotating the slot out from under in-flight
operations.

**Fix.** `REGISTRAR_ROLE` now gates every amendment. `set_registrar` rotates the
slot of record for readability and backward compatibility, while the role is
what is actually enforced; `require_registrar_role` checks both, so a rotation
that updated only the slot cannot silently do nothing. This is pinned by
`test_registrar_slot_without_the_role_cannot_amend_the_register`.

### F2 — Authority was resolved from a single mirrored slot *(fixed in this PR)*

The admin slot was both the authorization target and the record of who is in
charge, so the two concerns could not be checked independently.

**Fix.** Authority is derived from `DEFAULT_ADMIN_ROLE` membership.
`transfer_admin` grants the role to the successor **before** revoking it from the
incumbent, so there is never an instant with zero admins. `require_admin_role`
authenticates the mirror and then re-checks the role index, so the two cannot
silently disagree — that double check is deliberate.

### F3 — Last-admin lockout *(fixed in this PR)*

Nothing stopped an admin from revoking its own `DEFAULT_ADMIN_ROLE`, which would
leave the contract permanently unadministrable: no pause, no registrar rotation,
no upgrade path.

**Fix.** `revoke_role` returns `CannotRevokeLastAdmin` when the target is the last
holder. Pinned by `test_last_admin_cannot_be_revoked` and the positive case in
`test_second_admin_allows_the_first_to_step_down`.

### F4 — `renounce_role` could silently orphan the member index *(fixed in this PR)*

Role membership is tracked in instance storage under `DataKey::RoleMembers` so
`role_members()` stays a single read. A naive `renounce_role` that cleared only
the `(account, role)` flag would leave a stale address in the index, which is
worse than no index: `role_members()` would report an address that no longer
holds the role, and the `CannotRevokeLastAdmin` guard would count it and so
permit a revoke that should have been refused.

**Fix.** `renounce_role` clears the flag and the index entry together, and clears
the index outright rather than filtering, because renouncement is per-address and
the index is rebuilt on the next `grant_role`. Pinned by
`test_renounce_clears_every_role` and the `role_members(...).len() == 0`
assertion in `src/invariant_tests.rs`.

### F5 — `holder_count` drift *(fixed in this PR)*

The share transitions that add or remove a holder had to maintain
`Property.holder_count`, which is a denormalised count the registrar set is
trusted to keep honest. A repeat transfer to an existing holder must not
double-count them.

**Fix.** Both transitions are driven off the share read, not off the argument:
a holder is added only when the pre-existing share was zero and removed only when
the moved share was non-zero. `invariant_holder_count_tracks_distinct_holders`
asserts the count across a repeat transfer.

### F6 — Pause is admin-only and does not block queries *(accepted, no change)*

`pause` blocks `register_property`, `update_valuation`, `transfer_property` and
`record_ownership` but not the read-only queries. That is intentional: a paused
contract that refuses queries makes it impossible to inspect what the incident
even was. Noted here so it is a recorded decision rather than an oversight.

## Coverage map

| Entry point | Gate | Negative test |
| --- | --- | --- |
| `initialize` | one-shot, deployer auth | `test_initialize_can_only_run_once`, `test_initialize_is_gated_by_the_deployer` |
| `grant_role` | `DEFAULT_ADMIN_ROLE` | `test_a_registrar_without_the_role_cannot_escalate_itself` |
| `revoke_role` | `DEFAULT_ADMIN_ROLE` + last-admin guard | `test_last_admin_cannot_be_revoked`, `test_revoke_removes_the_role_and_stops_amendments` |
| `renounce_role` | caller auth | `test_renounce_clears_every_role` |
| `transfer_admin` | `DEFAULT_ADMIN_ROLE`, grant-before-revoke | `test_transfer_admin_rotates_the_role_not_just_the_mirror` |
| `set_registrar` | `DEFAULT_ADMIN_ROLE` | — (same gate, covered transitively) |
| `pause` / `unpause` | `DEFAULT_ADMIN_ROLE` | `test_pause_blocks_every_state_change_regardless_of_role` |
| `register_property` | `REGISTRAR` **and** registrar slot | `test_register_property_is_gated_by_the_registrar`, `test_registrar_slot_without_the_role_cannot_amend_the_register` |
| `update_valuation` | `REGISTRAR` **and** registrar slot | `test_update_valuation_is_gated_by_the_registrar` |
| `record_ownership` | `REGISTRAR` **and** owner of record | `test_record_ownership_needs_both_the_role_and_the_owner` |
| `transfer_property` | owner of record | `test_transfer_property_is_gated_by_the_owner_of_record` |

Tests that assert *which* address had to authorize use `env.auths()` rather than
relying on `mock_all_auths()` side effects, because the question the audit needs
answered is "whose signature was demanded", not "did the call succeed".

## Residual risk

- The default admin chooses the initial registrar and can grant `REGISTRAR` to
  any address, so the admin key is effectively a register-amendment key. This is
  by design — the admin is fully trusted — but a deployment that wants a
  separation of duties should grant `REGISTRAR` to a dedicated operations address
  and leave the admin key cold.
- `holder_count` is denormalised state the registrar set can still only affect
  through the two transitions above, so it cannot be set arbitrarily — but it is
  an index, not an authority. Nothing in the contract reads it to make a
  decision.
- There is no timelock or multi-sig on `grant_role` / `revoke_role`. If a
  compromised admin key is in the threat model, governance should sit upstream of
  this contract rather than inside it.