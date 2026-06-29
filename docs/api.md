# Contract API Stability Guarantees

This document defines the API stability guarantees for Uzima smart contracts. It is intended for developers, integrators, and operators who rely on contract entrypoints, event schemas, and upgrade paths.

## Purpose

Uzima Contracts uses contract APIs as the foundation for secure, auditable healthcare workflows. This page explains:

- Stability levels for contract APIs
- Breaking change policies
- Deprecation and migration support
- How contract API stability is classified across the repository

## Stability Levels

### Stable API

A Stable API is a public contract entrypoint, event schema, or data model that is guaranteed to remain compatible across patch and minor releases.

Guarantees:

- Patch releases may only include bug fixes, performance improvements, documentation updates, and non-breaking enhancements.
- Minor releases may add new optional functionality or extend existing parameters without breaking existing callers.
- Breaking changes require a major version bump and advance notice.
- Stable APIs are the recommended integration surface for production clients.

### Deprecated API

A Deprecated API is still available but will be removed in a future major release.

Guidelines:

- Deprecation must be documented clearly with replacement guidance.
- Deprecated APIs remain functional for at least one full major release cycle.
- Clients should migrate to the replacement API before the removal release.
- Deprecated functions, events, or data schemas should include a migration path in docs.

### Unstable API

An Unstable API is experimental or under active development.

Characteristics:

- Subject to change without prior deprecation.
- May change in patch, minor, or major releases.
- Should be used with caution and typically only by early adopters or internal integrations.
- Unstable APIs are not guaranteed to have backward compatibility.

## Breaking Change Policy

Uzima Contracts follows a predictable policy for breaking changes:

1.  **Major version bump for breaking changes**
    - Any change that alters public function signatures, event formats, or storage semantics in a way that invalidates existing clients is treated as a major breaking change.

2.  **Deprecation before removal**
    - Stable APIs are deprecated before being removed.
    - Deprecated APIs remain available for at least one major release.

3.  **Compatibility-first patch/minor releases**
    - Patch releases: bug fixes, documentation, and non-breaking adjustments only.
    - Minor releases: additive functionality and optional schema extensions only.

4.  **Migration support**
    - Contracts should expose version or compatibility metadata where feasible.
    - Migration paths and upgrade procedures must be documented in `docs/upgradeability.md`, `docs/migration.md`, and this file.

## API Versioning and Compatibility

When a contract needs explicit API version tracking, the recommended pattern is:

- `API_VERSION`: current public API version for the contract.
- `MIN_COMPATIBLE_VERSION`: minimum supported client version.
- `get_api_version()`: returns the current contract API version.
- `is_compatible(client_version)`: returns whether a client version can safely call the contract.

### Recommended contract version methods

```rust
pub const API_VERSION: u32 = 1;
pub const MIN_COMPATIBLE_VERSION: u32 = 1;

pub fn get_api_version(env: Env) -> u32 {
    API_VERSION
}

pub fn is_compatible(env: Env, client_version: u32) -> bool {
    client_version >= MIN_COMPATIBLE_VERSION && client_version <= API_VERSION
}
```

Contracts should store version metadata in persistent storage when upgradeable data structures are present.

## Migration Support

The repository already includes a migration strategy for contract upgrades in `docs/migration.md`.

Key migration support considerations:

- Preserve existing storage where possible and apply deterministic migration logic.
- Use contract-level upgrade functions such as `upgrade` or `migrate_data` when available.
- Test migrations explicitly with old-format data and confirm that new contract logic is compatible.
- Document any required client-side changes for upgraded APIs.

## API Classification

The following classification is the baseline for the repository. All contracts should be evaluated against these categories and documented accordingly.

| Contract or Module Family | Stability Level | Notes |
| --- | --- | --- |
| `contracts/medical_records` | Stable | Core patient record storage and access APIs. |
| `contracts/identity_registry` | Stable | Identity and public key management APIs. |
| `contracts/rbac` | Stable | Role-based access control APIs. |
| `contracts/patient_consent_management` | Stable | Patient consent and authorization APIs. |
| `contracts/health_data_access_logging` | Stable | Audit logging and access tracking APIs. |
| `contracts/governor` | Stable | Governance proposal and voting APIs. |
| `contracts/upgradeability` | Stable | Contract upgrade and migration APIs. |
| `contracts/audit_forensics` | Stable | Audit trail and forensic analysis APIs. |
| `contracts/cross_chain_bridge` | Unstable | Cross-chain interoperability APIs are evolving and may change. |
| `contracts/multi_region_orchestrator` | Unstable | Multi-region orchestration is experimental and subject to change. |
| `contracts/differential_privacy` | Unstable | Advanced privacy features are still under active development. |
| `contracts/federated_learning` | Unstable | Emerging machine learning integration APIs. |
| `contracts/medical_imaging_ai` | Unstable | AI-assisted imaging features are experimental. |

### How to classify a new API

For any new contract or entrypoint, classify the API at the time of introduction:

- If it is a public production integration surface and is expected to remain compatible, mark it as **Stable**.
- If it is temporary, exploratory, or may be replaced, mark it as **Unstable**.
- If it is an existing stable API that is being superseded, mark it as **Deprecated** and document the replacement.

## Documentation and Code Requirements

All contract APIs should include the following documentation elements:

- Stability level label in the contract README or public API docs.
- Deprecation notices for any removed or replaced entrypoints.
- Migration instructions for breaking changes.
- Cross-references to `docs/upgradeability.md`, `docs/migration.md`, and `docs/api.md`.

### Suggested documentation pattern

```md
### API Stability
- Status: Stable
- Last reviewed: 2026-04-25
- Compatibility: patch/minor-safe; major-breaking only
```

### Suggested code comment pattern

```rust
/// `set_patient_record` is stable for minor/patch releases.
/// Deprecated: use `update_patient_record` in the next major release.
pub fn set_patient_record(...) { ... }
```

## Enforcement

To keep contract APIs reliable:

- Update this document whenever a new contract is added.
- Review `docs/api.md` during contract design and release planning.
- Include API stability status in PR descriptions for contracts.
- Keep deprecated APIs documented with removal timelines and migration paths.

## Related Documents

- [Developer Guide](./DEVELOPER_GUIDE.md)
- [Contract upgradeability](./upgradeability.md)
- [Data migration system](./migration.md)
- [Implementation guide for versioning](./XHRISTIN3_IMPLEMENTATION_GUIDE.md)
