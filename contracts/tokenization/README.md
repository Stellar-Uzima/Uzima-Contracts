# Tokenization

Soroban contract that records tokenized assets and tracks fractional ownership
of each asset's supply across holders.

| | |
| --- | --- |
| Crate | `contracts/tokenization` |
| Entry points | `initialize`, `transfer_admin`, `set_treasury_address`, `pause`, `unpause`, `register_asset`, `mint`, `burn`, `transfer`, plus queries |
| Tests | `src/test.rs` (behaviour), `src/overflow_tests.rs` (arithmetic + layout markers) |

## Roles

| Role | Capabilities |
| --- | --- |
| Admin | `register_asset`, `pause`, `unpause`, `transfer_admin`, `set_treasury_address` |
| Issuer | `mint`, `burn` on assets it registered |
| Holder | `transfer` units it holds |

## Integer overflow policy (#1765)

Root `Cargo.toml` sets `overflow-checks = true` for `[profile.release]`, so an
unguarded `+` or `-` on a `u128` traps and reverts the whole transaction — which
is both expensive and a denial-of-service vector if an attacker can pick the
operand. Every arithmetic site therefore goes through `src/checked_math.rs`:

| Helper | Failure |
| --- | --- |
| `checked_add(a, b)` | `Error::ArithmeticOverflow` |
| `checked_sub(a, b)` | `Error::ArithmeticUnderflow` |
| `checked_mul(a, b)` | `Error::ArithmeticOverflow` |

Two errors are raised before the helper is reached, because they mean something
more specific than "the arithmetic was pathological":

- `Error::InsufficientBalance` — `burn`/`transfer` were asked to debit more than
  the holder holds.
- `Error::ArithmeticOverflow` — `mint` would push the running supply past
  `u128::MAX`.

`src/overflow_tests.rs` drives each of these to a real failure and asserts the
resulting state is unchanged.

## Storage gap and upgrade safety (#1766)

Two markers keep future WASM upgrades from silently reinterpreting live ledger
entries:

- **`StorageVersion`** — an instance-storage key stamped with
  `STORAGE_SCHEMA_VERSION` (`1`) at `initialize`. Upgrade tooling compares the
  on-chain value to the compiled-in value; `0` means the deployment predates the
  marker and every migration step must run first. Exposed as `storage_version()`.
- **`StorageGap(u32)`** — `STORAGE_GAP_SLOTS` (`16`) reserved key slots. The
  reservation is **layout-only**: no ledger entry is written for a gap slot, so
  the reserve costs no storage rent, but a future upgrade may claim
  `StorageGap(0)..StorageGap(16)` as ordinary persistent keys without colliding
  with anything live today. Exposed as `storage_gap_slots()` and
  `is_reserved_gap_slot(slot)`.

Adding, removing, or reordering a `DataKey` variant or an `Asset` field requires
bumping `STORAGE_SCHEMA_VERSION` and a migration step.

## Deployment (#1767)

```bash
./scripts/deploy_tokenization.sh testnet alice
```

Builds the wasm, uploads a fresh instance, runs `initialize`, asserts that
`storage_version` came back as `1`, and writes
`deployments/<network>_tokenization.json` so
`scripts/rollback_deployment.sh` and `scripts/deployment_status.sh` can find it.

## Static analysis in CI (#1768)

`.github/workflows/tokenization-slither.yml` runs on every PR that touches this
crate:

- **`slither` job** — installs `slither-analyzer` and runs it over any Solidity
  sources in the tree. The repository currently contains none (the contract is
  Soroban/Rust, and Slither only understands EVM bytecode), so the step reports
  *skipped* rather than passing silently. If a Solidity component is ever added
  to the deployment path it is covered with no workflow change.
- **`clippy-audit` job** — `cargo clippy -p tokenization --all-targets -D
  warnings` plus `cargo audit`. This is the half that actually gates the merge
  for a Rust contract.