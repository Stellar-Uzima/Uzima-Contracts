# Property Registry

Soroban contract: on-chain registry of property records — legal owner of record,
recorded valuation, and fractional ownership shares in basis points.

| | |
| --- | --- |
| Crate | `contracts/property_registry` |
| Entry points | `initialize`, `transfer_admin`, `set_registrar`, `pause`, `unpause`, `register_property`, `update_valuation`, `transfer_property`, `record_ownership`, plus queries |
| Tests | `src/test.rs` (behaviour), `src/overflow_tests.rs` (arithmetic + layout markers) |

## Roles

| Role | Capabilities |
| --- | --- |
| Admin | Rotate admin and registrar, pause/unpause |
| Registrar | Register properties, revalue, record ownership shares |
| Owner of record | Transfer the property, authorize share movements out of their position |

## Share accounting

Shares are denominated in basis points, `BASIS_POINTS_SCALE = 10_000` == 100%.
Every mutation moves both legs of the split together:

- `record_ownership(id, holder, amount)` debits the owner of record and credits
  `holder` in the same call, so the total across all holders is always exactly
  `BASIS_POINTS_SCALE`.
- `transfer_property(id, new_owner)` credits the successor **before** debiting
  the incumbent, and refuses with `ShareTotalExceeded` if that would push the
  successor past 100%. A rejected call therefore leaves the split untouched.

`holder_count` on the record tracks distinct holders, and a repeat transfer to an
existing holder does not double-count them.

## Integer overflow policy (#1781)

Root `Cargo.toml` sets `overflow-checks = true` for `[profile.release]`, so an
unguarded `+` or `-` on a `u128` traps and reverts the whole transaction. Every
arithmetic site goes through `src/checked_math.rs`:

| Helper | Failure |
| --- | --- |
| `checked_add(a, b)` | `Error::ArithmeticOverflow` |
| `checked_sub(a, b)` | `Error::ArithmeticUnderflow` |
| `checked_mul(a, b)` | `Error::ArithmeticOverflow` |

Two errors are raised ahead of the helper, because they mean something more
specific than "the arithmetic was pathological":

- `Error::ShareTotalExceeded` — the result would push shares past 100%.
- `Error::OwnershipExceedsShare` — more was requested than the owner of record
  currently holds.

`bps_to_percent` converts basis points to whole percent, rounding down, so
callers never do the division themselves and get it wrong.

`src/overflow_tests.rs` drives each path to a real failure and asserts the
resulting split is unchanged.

## Storage gap and upgrade safety (#1782)

- **`StorageVersion`** — an instance-storage key stamped with
  `STORAGE_SCHEMA_VERSION` (`1`) at `initialize`. Upgrade tooling compares
  on-chain vs compiled-in values; `0` means the deployment predates the marker
  and every migration step must run first. Exposed as `storage_version()`.
- **`StorageGap(u32)`** — 16 reserved key slots. The reservation is
  **layout-only**: no ledger entry is written for a gap slot, so it costs no
  storage rent, but a future upgrade may claim `StorageGap(0)..StorageGap(16)`
  as ordinary persistent keys without colliding with anything live today.
  Exposed as `storage_gap_slots()` and `is_reserved_gap_slot(slot)`, with tests
  walking the boundary of the range.

Adding, removing, or reordering a `DataKey` variant or a `Property` field
requires bumping `STORAGE_SCHEMA_VERSION` and a migration step.

## Deployment (#1783)

```bash
./scripts/deploy_property_registry.sh testnet alice
```

Builds the wasm, uploads a fresh instance, runs `initialize`, asserts that
`storage_version` came back as `1`, and writes
`deployments/<network>_property_registry.json` in the shape
`scripts/rollback_deployment.sh` and `scripts/deployment_status.sh` consume.

## Static analysis in CI (#1784)

`.github/workflows/property-registry-slither.yml` runs on every PR that touches
this crate:

- **`slither` job** — installs `slither-analyzer` and runs it over any Solidity
  sources in the tree. The repository currently contains none (the contract is
  Soroban/Rust, and Slither only understands EVM bytecode), so the step reports
  *skipped* rather than passing silently. If a Solidity component is ever added
  to the deployment path it is covered with no workflow change.
- **`clippy-audit` job** — `cargo clippy -p property_registry --all-targets -D
  warnings` plus `cargo audit`. This is the half that actually gates the merge
  for a Rust contract.