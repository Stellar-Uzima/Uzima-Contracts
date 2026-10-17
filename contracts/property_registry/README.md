# Property Registry

Soroban contract: on-chain registry of property records — legal owner of record,
recorded valuation, and fractional ownership shares in basis points.

| | |
| --- | --- |
| Crate | `contracts/property_registry` |
| Entry points | `initialize`, `transfer_admin`, `set_registrar`, `pause`, `unpause`, `register_property`, `update_valuation`, `transfer_property`, `record_ownership`, plus queries |
| Modules | `validation` (input bounds), `events` (audit trail), `storage`, `types` |

## Roles

| Role | Capabilities |
| --- | --- |
| Admin | Rotate admin and registrar, pause/unpause |
| Registrar | Register properties, revalue, record ownership shares |
| Owner of record | Transfer the property, authorize share movements out of their position |

## Typed errors (#1776)

Every rejection path has its own `Error` variant. Before this change a caller
could not tell "the id was malformed" from "the currency was not alphabetic"
from "the owner does not hold that much" without inspecting the failing call;
now it matches on a stable `u32` discriminant.

| Discriminant | Variant | Raised when |
| --- | --- | --- |
| 6 | `InvalidPropertyId` | empty, `< 3`, `> 64` bytes, or a byte outside `[A-Za-z0-9_-]` |
| 7 | `InvalidCurrency` | empty, `> 8` bytes, or not alphabetic |
| 8 | `InvalidMetadataUri` | empty or `> 512` bytes |
| 9 | `ZeroValue` | a valuation or share of zero |
| 10 | `HolderIsOwner` | `record_ownership` credited the owner of record, a no-op written twice |
| 11 | `TransferToSelf` | `transfer_property` to the current owner of record |
| 12 | `ShareTotalExceeded` | the credit would push a holder past 100% |
| 13 | `OwnershipExceedsShare` | more was requested than the owner of record holds |
| 14 | `ShareTooLarge` | a share above `BASIS_POINTS_SCALE` |
| 15 | `ArithmeticOverflow` | a `u128` add or multiply would wrap |
| 16 | `ArithmeticUnderflow` | a `u128` subtract would wrap past zero |

Discriminants are part of the contract interface: existing variants are never
renumbered and new ones are only appended. `test_error_variants_map_to_stable_discriminants`
pins them so an accidental reorder fails the build rather than a client.

## Input validation (#1775)

`src/validation.rs` runs **before** any `require_auth`. That ordering is
load-bearing: if auth ran first, the error a caller receives would depend on who
holds authority, and a malformed call could be used to probe the registrar set at
the cost of only the rejected invocation. Validating first means a bad argument is
rejected uniformly, for everyone, without consuming an authorization.

Bounds: `property_id` in `3..=64` bytes over `[A-Za-z0-9_-]`; `valuation` strictly
positive; `valuation_currency` alphabetic and at most 8 bytes (so a downstream
indexer can treat it as ISO 4217 without a second parse — `is_iso_4217` is
exported for the stricter three-letter check); `metadata_uri` in `1..=512` bytes;
shares in `1..=BASIS_POINTS_SCALE`.

The `property_id` alphabet is deliberately narrow. It covers the `PROP-2026-001`
style identifiers used across the platform while keeping control characters and
bidi overrides out of a value that ends up rendered in dashboards and land
registries.

## Events (#1773)

Every state-changing entry point publishes exactly one event, carrying the full
before/after delta, so the register and the ownership split are reconstructible
from the event stream alone without replaying transactions.

| Topic 0 | Topic 1 | Emitted by | Payload |
| --- | --- | --- | --- |
| `init` | — | `initialize` | `admin`, `registrar` |
| `adm_xfer` | — | `transfer_admin` | `previous`, `current` |
| `registrar` | — | `set_registrar` | `registrar` |
| `pause` | — | `pause` / `unpause` | resulting `paused` bool |
| `prop` | `reg` | `register_property` | `property_id`, `owner`, `valuation`, `currency`, `ts` |
| `prop` | `value` | `update_valuation` | `property_id`, `previous`, `current`, `ts` |
| `prop` | `xfer` | `transfer_property` | `property_id`, `previous`, `current`, `moved_share`, `ts` |
| `prop` | `share` | `record_ownership` | `property_id`, `holder`, `amount`, `owner_share`, `holder_share`, `ts` |

Topics stay within the 9-byte `symbol_short!` limit. Rejected calls emit
nothing, which `test_rejected_calls_emit_no_events` asserts.

## Gas (#1774)

Each entry point reads every ledger key at most once and writes back only what
changed.

- `register_property` — one `ledger().timestamp()` read serves both
  `created_at` and `updated_at`; `property_count` is read and written once; the
  record is constructed inline rather than read-modify-written.
- `update_valuation` — a single `property_of` read serves the existence check
  and the write-back.
- `record_ownership` — the record is read once and both `updated_at` and
  `holder_count` are settled before a single write.
- `transfer_property` — both legs are computed into locals first, so the two
  ownership writes and the record write are all decided before any of them lands.
- Arithmetic is computed once into locals (`remaining`, `credited`) and written
  once, rather than reading back what was just stored.

## Consistent splits

`record_ownership` debits the owner of record and credits the holder in the same
call. `transfer_property` credits the successor **before** debiting the incumbent
and refuses with `ShareTotalExceeded` if that would exceed 100%. So a rejected
call leaves the ownership split untouched, and the total across all holders is
always exactly `BASIS_POINTS_SCALE`. `holder_count` tracks distinct holders and a
repeat transfer to an existing holder does not double-count them.

## Tests

`src/test.rs` covers initialization, the happy path, every error path, per-topic
event counts, the `u32`-discriminant pinning, and the validation-before-auth
ordering. `src/validation.rs` carries unit tests for each bound at its edge. Run
with `cargo test -p property_registry`.
