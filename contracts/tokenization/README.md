# Tokenization

Soroban contract that records tokenized assets and tracks fractional ownership
of each asset's supply across holders.

| | |
| --- | --- |
| Crate | `contracts/tokenization` |
| Entry points | `initialize`, `transfer_admin`, `set_treasury_address`, `pause`, `unpause`, `register_asset`, `mint`, `burn`, `transfer`, plus queries |
| Modules | `validation` (input bounds), `events` (audit trail), `storage`, `types` |

## Roles

| Role | Capabilities |
| --- | --- |
| Admin | `register_asset`, `pause`, `unpause`, `transfer_admin`, `set_treasury_address` |
| Issuer | `mint`, `burn` on assets it registered |
| Holder | `transfer` units it holds |

## Typed errors (#1760)

Every rejection path has its own `Error` variant. Before this change a client
could not tell "the amount was zero" from "the balance was short" from "the
supply cap was reached" without inspecting the failing call; now it matches on a
stable `u32` discriminant.

| Discriminant | Variant | Raised when |
| --- | --- | --- |
| 6 | `InvalidAssetId` | empty, `< 3`, `> 64` bytes, or a byte outside `[A-Za-z0-9_-]` |
| 7 | `DecimalsOutOfRange` | outside the Stellar Asset Convention `0..=18` |
| 8 | `InvalidMetadataUri` | empty or `> 512` bytes |
| 9 | `ZeroAmount` | a mint, burn or transfer for `amount == 0` |
| 10 | `TransferToSelf` | `from == to` on a transfer |
| 11 | `InsufficientBalance` | debit exceeds the holder's balance |
| 12 | `SupplyCapExceeded` | mint would push supply past the registered cap |
| 13 | `ArithmeticOverflow` | a `u128` add or multiply would wrap |
| 14 | `ArithmeticUnderflow` | a `u128` subtract would wrap past zero |

Discriminants are part of the contract interface: existing variants are never
renumbered and new ones are only appended. `test_error_variants_map_to_stable_discriminants`
pins them so an accidental reorder fails the build rather than a client.

`register_asset` gained a `supply_cap` parameter that is now stored on the
record, which is what makes `SupplyCapExceeded` enforceable on-chain instead of
being an off-chain promise.

## Input validation (#1759)

`src/validation.rs` runs **before** any `require_auth`. That ordering matters:
if auth ran first, the error a caller receives would depend on who holds
authority, and a malformed call could be used to probe the admin set at the cost
of only the rejected invocation. Validating first means a bad argument is
rejected uniformly, for everyone, without consuming an authorization.

The `asset_id` alphabet is ASCII alphanumerics plus `-` and `_`. That covers the
`HOSP-2026-001` style identifiers used across the platform while keeping control
characters and bidi overrides out of a value that ends up rendered in dashboards
and explorers.

## Events (#1757)

Every state-changing entry point publishes exactly one event, carrying the
complete before/after delta, so contract state is reconstructible from the event
stream alone without replaying transactions.

| Topic 0 | Topic 1 | Emitted by | Payload |
| --- | --- | --- | --- |
| `init` | — | `initialize` | `admin`, `treasury` |
| `adm_xfer` | — | `transfer_admin` | `previous`, `current` |
| `treasury` | — | `set_treasury_address` | `treasury` |
| `pause` | — | `pause` / `unpause` | resulting `paused` bool |
| `asset` | `reg` | `register_asset` | `asset_id`, `issuer`, `supply_cap`, `decimals`, `ts` |
| `asset` | `mint` | `mint` | `asset_id`, `to`, `amount`, `new_supply`, `new_balance`, `ts` |
| `asset` | `burn` | `burn` | `asset_id`, `from`, `amount`, `new_supply`, `new_balance`, `ts` |
| `asset` | `xfer` | `transfer` | `asset_id`, `from`, `to`, `amount`, `from_balance`, `to_balance`, `ts` |

Topics stay within the 9-byte `symbol_short!` limit. Rejected calls emit
nothing, which `test_rejected_calls_emit_no_events` asserts.

## Gas (#1758)

Each entry point reads every ledger key at most once and writes back only what
changed.

- `mint` / `burn` — a single `asset_of` read serves the existence check, the cap
  check and the write-back. The old code re-read the record after mutating it.
- `transfer` — does **not** rewrite the asset record at all, because it does not
  touch supply. Two balance writes, no third.
- `register_asset` — `asset_count` is read and written once; the record is
  constructed inline rather than being read-modify-written.
- Arithmetic is computed once into locals (`new_supply`, `new_balance`) and
  written once, rather than reading back what was just stored.
- Queries avoid the `unwrap_or(0)` + branch that the write paths use, so a read
  that misses costs one ledger read instead of a read plus a write.

## Tests

`src/test.rs` covers initialization, the happy path, and every error path, plus
event counts per topic pair and the `u32`-discriminant pinning. Run with
`cargo test -p tokenization`.