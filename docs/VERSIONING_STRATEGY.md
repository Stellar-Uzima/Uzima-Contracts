# Versioning Strategy

This document describes the SDK versioning policy for the Uzima Contracts
workspace. It covers bump cadence, compatibility guarantees, deprecation
timeline, and the mechanisms that enforce consistency.

## Workspace Dependency Pin

Every contract crate **must** inherit `soroban-sdk` from the workspace root:

```toml
# Member crate Cargo.toml — correct
soroban-sdk.workspace = true          # bare dependency
soroban-sdk = { workspace = true }    # with extra features (e.g. testutils)
```

Hardcoded versions are **prohibited**:

```toml
# ❌ NEVER do this
soroban-sdk = "21.7.7"
soroban-sdk = { version = "=21.7.7" }
```

The single source of truth is the workspace root `Cargo.toml`:

```toml
[workspace.dependencies]
soroban-sdk = { version = "=21.7.7" }
```

A CI guard script (`scripts/check_sdk_version.sh`) enforces this rule on
every pull request. The check scans **all** `Cargo.toml` files (including
excluded/deferred contracts) and fails if any crate pins a version directly.

## SDK Bump Cadence

| Cadence | Action |
|---|---|
| **Patch** (21.7.x) | Apply immediately when released. No API breakage expected. |
| **Minor** (21.x.0) | Evaluate within 2 weeks. Review changelog for new APIs and deprecations. |
| **Major** (x.0.0) | Dedicated migration branch. Full audit of breaking changes before merge. |

Patch bumps are the common case and should be low-risk. The workspace pin
uses an **exact version** (`=21.7.7`) so that builds are fully reproducible —
no semver range drift.

## Compatibility Matrix

| soroban-sdk | stellar-sdk | Stellar network | Status |
|---|---|---|---|
| `21.7.x` | `21.x` | mainnet / testnet | **Current** |
| `22.x` | `22.x` | mainnet / testnet | Future (requires migration) |

> **Note:** The `healthcare_compliance` contract was previously pinned to
> soroban-sdk `22.0.0` — this drift was corrected in PR #855.

## Deprecation Policy

When the workspace SDK is bumped:

1. **Release notes** document the new pinned version.
2. A **migration branch** (`chore/bump-soroban-sdk-X.Y`) is created with the
   version change and any required API adjustments.
3. All member crates are compiled and tested against the new version.
4. The PR description includes `env.budget()` before/after snapshots (per
   issue #855 acceptance criteria) so reviewers can assess fee impact.
5. Excluded/deferred contracts are updated on a best-effort basis. Their
   `Cargo.toml` drift is caught by `check_sdk_version.sh` but resolution is
   not a merge blocker for SDK-bump PRs.

## Adding a New Contract

When integrating a new contract into the workspace:

1. Create `contracts/<name>/Cargo.toml` with `soroban-sdk.workspace = true`.
2. Do **not** specify a version — let the workspace pin handle it.
3. Run `./scripts/check_sdk_version.sh` locally before opening PR.
4. If the contract requires features beyond `testutils` (e.g. `alloc`),
   inherit them explicitly:

```toml
[dependencies]
soroban-sdk = { workspace = true, features = ["alloc"] }
```

## Excluded Contracts

Contracts listed in the workspace `exclude = [...]` array are deferred for
various reasons (SDK compatibility, no-Cargo, non-contract). They are still
scanned by `check_sdk_version.sh` to catch drift early, but fixing their
pin is tracked separately (see issue #828).

## CI Enforcement

```yaml
# .github/workflows/ci.yml (example snippet)
- name: Check SDK version consistency
  run: ./scripts/check_sdk_version.sh
```

The script exits with code **1** if any crate overrides the workspace pin,
failing the CI check and preventing merge.

## References

- Issue #834 — SDK 21 compatibility fixes (identified drift)
- Issue #828 — Excluded contracts audit
- Issue #855 — This PR (workspace pin + CI guard)
- [Keep a Changelog](https://keepachangelog.com/en/1.0.0/)
- [Semantic Versioning](https://semver.org/spec/v2.0.0.html)
