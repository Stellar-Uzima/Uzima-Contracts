# deprecation_framework

Healthcare deprecation-framework contract: marks contracts for deprecation,
sets their sunset timeline, publishes migration guides and user
communications, and tracks removal checklists.

## SDK policy and workspace integration

This contract is a first-class member of the root workspace and is covered by
the workspace Soroban SDK policy:

- `soroban-sdk` resolves through the workspace dependency table
  (`soroban-sdk = { version = "=21.7.7" }`); the crate enables the `alloc`
  feature only. There is no second SDK version graph and no conflicting
  SDK declaration.
- `governance_commons` resolves through the workspace path dependency, so
  guard initialization is shared with the rest of the platform.
- The crate builds against the toolchain pinned in `rust-toolchain.toml`
  (`channel = "1.92.0"`).

CI checks the crate whenever it, the workspace manifest, or its path
dependencies change (see `.github/workflows/deprecation-framework-check.yml`):

```bash
cargo check --manifest-path contracts/deprecation_framework/Cargo.toml --lib
```

## Compatibility note (issue #1426)

`contracts/deprecation_framework` was previously listed in the root
workspace `exclude` array, so its compatibility with the pinned SDK could
silently drift and was never checked by the workspace build/test gates. As
part of issue #1426 it was removed from `exclude` and integrated as a
workspace member, with only manifest-level changes (adding `rlib` to the
crate types so the workspace test harness can build it).

The public contract types and entrypoints were not changed by this
integration:

- Public entrypoints: `initialize`, `mark_for_deprecation`,
  `set_sunset_timeline`, `add_migration_guide`,
  `update_deprecation_phase`, `publish_user_communication`,
  `create_removal_checklist`, `mark_checklist_item_complete`,
  `get_deprecation_status`, `get_sunset_timeline`,
  `get_migration_guide`, `is_deprecated`.
- Public types: `DataKey`, `DeprecationPhase`, `DeprecationStatus`,
  `MigrationGuide`, `SunsetTimeline`, `Error`.
- Wire format, storage layout, and event topics (`DEPREC:*`) are stable.

The `#![no_std]` implementation uses only Soroban-compatible data
construction (`soroban_sdk::{Address, Env, String, Vec}`, storage instance
and persistent access, ledger timestamps, and `events().publish`); no
production `std` or allocation-only APIs are used.

## Files

| File | Purpose |
|------|---------|
| `src/lib.rs` | Main contract: `#[contract]` + `#[contractimpl]` logic |
| `src/errors.rs` | `#[contracterror]` enum following Uzima conventions |
| `src/events.rs` | `DEPREC:*` event-emission helpers |
| `src/types.rs` | Shared `#[contracttype]` structs and enums |
| `src/test.rs` | Unit tests using `soroban-sdk/testutils` |
| `Cargo.toml` | Workspace-inherited dependencies, `cdylib` + `rlib` crate types |