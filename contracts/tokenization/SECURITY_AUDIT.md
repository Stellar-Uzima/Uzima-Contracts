# Access Control Audit — `tokenization`

Scope: issue **#1763** (Audit access control in Tokenization).

Reviewed surface: every entry point on `TokenizationContract` that changes or
reads privileged state, plus the role bookkeeping in `src/access_control.rs`.

## Trust model

| Actor | Trust level | Why |
| --- | --- | --- |
| `DEFAULT_ADMIN_ROLE` holder | Fully trusted | Registers assets, rotates admin, grants and revokes roles, pauses. Can appoint itself as any asset's issuer, so it can mint — this is inherent to administering a registry and is called out rather than prevented. |
| `ISSUER` holder + recorded `Asset.issuer` | Trusted for supply | May mint and burn, and only on assets whose recorded `issuer` is that address. |
| Holder | Untrusted | May only move units it holds. |
| Anyone | Untrusted | Read-only queries. |

## Findings

### F1 — Supply creation was gated only by asset ownership *(fixed in this PR)*

The pre-refactor contract gated `mint` and `burn` on `require_admin()` plus
`asset.issuer.require_auth()`. `Asset.issuer` is chosen by the admin at
`register_asset` time and is not otherwise constrained, so this is
self-consistent — but it means the *only* thing separating "this address owns
one asset" from "this address can create supply on the contract" is the
attentiveness of whoever calls `register_asset`. There is no contract-level
switch to withdraw that power.

**Fix.** `ISSUER_ROLE` now gates supply creation, and `mint`/`burn` require both
that the caller holds the role *and* that it is the asset's recorded issuer. The
role is granted and revoked by the default admin, so supply-creation authority
is separately auditable and separately revocable.

`test_mint_requires_the_issuer_role_on_top_of_asset_ownership` and
`test_holding_the_issuer_role_is_not_enough_for_another_issuers_asset` pin both
halves.

### F2 — Authority was resolved from a single mirrored slot *(fixed in this PR)*

Every admin-gated entry point read `DataKey::Admin` and authorized *that*
address. There was no role index, so the mirror was the whole of the access
model and nothing else could be expressed.

**Fix.** The admin slot is now derived from `DEFAULT_ADMIN_ROLE` membership.
`transfer_admin` grants the role to the successor **before** revoking it from
the incumbent, so there is never a moment with zero admins. `require_admin_role`
authenticates the mirror and then re-checks the role index, so the two cannot
silently disagree — that double check is deliberate.

### F3 — Last-admin lockout *(fixed in this PR)*

Nothing stopped an admin from revoking its own `DEFAULT_ADMIN_ROLE`, which would
leave the contract permanently unadministrable: no pause, no role grants, no
re-registration, and no upgrade path.

**Fix.** `revoke_role` returns `CannotRevokeLastAdmin` when the target is the
last holder of `DEFAULT_ADMIN_ROLE`. Covered by
`test_last_admin_cannot_be_revoked` and the positive case in
`test_second_admin_allows_the_first_to_step_down`.

### F4 — `renounce_role` could silently orphan the member index *(fixed in this PR)*

Role membership is tracked in instance storage under `DataKey::RoleMembers` so
`role_members()` stays a single read. A naive `renounce_role` that cleared only
the `(account, role)` flag would leave a stale address in the index, which is
worse than no index at all: `role_members()` would report an address that no
longer holds the role, and the `CannotRevokeLastAdmin` guard would count it and
so permit a revoke that should have been refused.

**Fix.** `renounce_role` clears both the flag and the index entry, and clears
the index outright rather than filtering, because renouncement is per-address
and the index is rebuilt on the next `grant_role`. Pinned by
`test_renounce_clears_every_role` and the `role_members(...).len() == 0`
assertion in `src/invariant_tests.rs`.

### F5 — Pause was admin-only and not consulted by queries *(accepted, no change)*

`pause` blocks `register_asset`, `mint`, `burn` and `transfer` but not the
read-only queries. That is intentional: a paused contract that refuses queries
makes it impossible to inspect what the incident even was. Noted here so it is
a recorded decision rather than an oversight.

## Coverage map

| Entry point | Gate | Negative test |
| --- | --- | --- |
| `initialize` | one-shot, deployer auth | `test_initialize_can_only_run_once`, `test_initialize_is_gated_by_the_deployer` |
| `grant_role` | `DEFAULT_ADMIN_ROLE` | `test_a_non_admin_issuer_cannot_escalate_itself` |
| `revoke_role` | `DEFAULT_ADMIN_ROLE` + last-admin guard | `test_last_admin_cannot_be_revoked`, `test_revoke_removes_the_role_and_stops_minting` |
| `renounce_role` | caller auth | `test_renounce_clears_every_role` |
| `transfer_admin` | `DEFAULT_ADMIN_ROLE`, grant-before-revoke | `test_transfer_admin_rotates_the_role_not_just_the_mirror` |
| `set_treasury_address` | `DEFAULT_ADMIN_ROLE` | — (same gate, covered transitively) |
| `pause` / `unpause` | `DEFAULT_ADMIN_ROLE` | `test_pause_blocks_every_state_change_regardless_of_role` |
| `register_asset` | `DEFAULT_ADMIN_ROLE` | `test_register_asset_is_gated_by_the_admin` |
| `mint` / `burn` | `ISSUER_ROLE` **and** `Asset.issuer` | `test_mint_requires_the_issuer_role_on_top_of_asset_ownership`, `test_holding_the_issuer_role_is_not_enough_for_another_issuers_asset`, `test_mint_is_gated_by_the_assets_own_issuer` |
| `transfer` | debit-side auth | `test_transfer_is_gated_by_the_debit_side` |

Tests that assert *which* address had to authorize use `env.auths()` rather than
`mock_all_auths()` side effects, because the question the audit needs answered is
"whose signature was demanded", not "did the call succeed".

## Residual risk

- The default admin can register an asset with itself as issuer and then mint.
  This is by design — the admin is fully trusted — but it means the admin key is
  a minting key. A deployment that wants that separation should register assets
  under a dedicated issuer address and grant `ISSUER_ROLE` to it.
- There is no timelock or multi-sig on `grant_role` / `revoke_role`. If a
  compromised admin key is in the threat model, governance should sit upstream of
  this contract rather than inside it.