# Technical Debt Tracking System

## Overview
This document serves as the central catalog and tracking system for technical debt across the Uzima-Contracts repository. It defines the process for identifying, assessing, prioritizing, and repaying technical debt.

## Debt Catalog

| ID | Contract/Module | Description | Type | Impact | Priority | Status | Owner | Target Date |
|---|---|---|---|---|---|---|---|---|
| TD-001 | `zkp_registry` | Replace simulated ZKP verification in `verify_zkp_internal` with actual cryptographic verification. Source still derives a gas cost from the proof type and checks it against `MAX_VERIFICATION_GAS` ("Simulate verification"). | Security | High | High | Open (verified 2026-11-19) | @Stellar-Uzima/maintainers | Q3 2024 (overdue; re-target at next quarterly review) |
| TD-002 | `zkp_registry` | Implement proper expiration decryption and validation in `create_credential_proof`. | Security | High | High | Resolved in source: `create_credential_proof` calls `decrypt_credential_expiration`, which checks ciphertext length and domain tag and rejects expired credentials (`InvalidExpirationCiphertext`, `CredentialExpired`). No fixing PR recorded. Residual: it is an XOR cipher keyed by a per-issuer salt, with a public default salt when none is set (`DEFAULT_ISSUER_SALT`). (verified 2026-11-19) | @Stellar-Uzima/maintainers | Q3 2024 (met) |
| TD-003 | `zkp_registry` | Replace simulated range proof verification in `verify_range_proof_internal` with actual verification. | Security | High | High | Resolved (PR #848) | @Stellar-Uzima/maintainers | Q3 2024 |
| TD-004 | `zkp_registry` | Replace simulated recursive proof verification in `verify_recursive_proof_internal` with actual verification. | Security | High | High | Partially resolved in source: the stub is gone; the function now requires a stored, previously verified base proof and a matching `aggregated_vk_hash` (SHA-256 binding). It does not verify the recursive proof cryptographically, and the base proof's own verification is still TD-001. No fixing PR recorded. (verified 2026-11-19) | @Stellar-Uzima/maintainers | Q4 2024 (overdue; tied to TD-001) |
| TD-005 | `scripts/wasm_size_baselines.json` | 31 deployable contracts, including `medical_records`, have no WASM size baseline entry; `check_wasm_baseline_staleness.sh` only reports the gap (non-blocking) until baselines are measured and `--strict` is enabled. Source: `docs/audits/WASM_BASELINE_STALENESS.md`. | Testing | Medium | Medium | Open | @Stellar-Uzima/maintainers | Q1 2027 (proposed) |
| TD-006 | `dicomweb_services` | DICOM conformance tests in `src/conformance_tests.rs` are TODO stubs (WADO-RS and STOW-RS retrieve/store cases are not implemented). Source: `contracts/dicomweb_services/src/conformance_tests.rs`. | Testing | Medium | Medium | Open | @Stellar-Uzima/maintainers | Q1 2027 (proposed) |
| TD-007 | Root `Cargo.toml` `workspace.exclude` | Contracts excluded from the workspace are deferred under issue #828 and tracked per contract (owner, blocker, status) in `docs/ONRAMP_OWNERSHIP.md`. | Architecture | High | Medium | Open (per-contract status in `docs/ONRAMP_OWNERSHIP.md`) | @Stellar-Uzima/maintainers | #828 on-ramp (see ownership map) |

## Assessment Matrix

### Impact Assessment
* **High**: Affects security, core functionality, or causes significant performance degradation. Needs immediate attention.
* **Medium**: Affects maintainability, causes minor performance issues, or lacks test coverage in non-critical paths.
* **Low**: Minor code quality issues, outdated documentation, or non-blocking architectural improvements.

### Priority Ranking
* **High**: Must be resolved in the next milestone or sprint.
* **Medium**: Should be resolved within the next 1-2 quarters.
* **Low**: Can be resolved when working on related components.

## Repayment Tracking Process

1. **Identification**: Any contributor can identify technical debt and propose adding it to the catalog via a Pull Request to this document.
2. **Assessment**: Maintainers review the proposed debt, assess its impact, and assign a priority.
3. **Scheduling**: High-priority items are converted into GitHub Issues and added to the current or next milestone.
4. **Repayment**: A contributor resolves the debt in a PR, referencing the TD-ID.
5. **Closure**: Once the PR is merged, the status in the catalog is updated to "Resolved", with a link to the fixing PR.

## Reporting and Reviews

* **Quarterly Reviews**: The maintainers will conduct a review of the technical debt catalog at the beginning of each quarter (Jan, Apr, Jul, Oct).
* **Review Goals**:
    * Re-assess open items and adjust priorities if necessary.
    * Identify new systemic technical debt.
    * Track the rate of debt repayment vs. accumulation.
    * Schedule repayment for the upcoming quarter.

## Types of Technical Debt

* **Architecture**: Suboptimal design choices that make the system harder to scale or modify.
* **Code Quality**: Code that is difficult to read, understand, or maintain (e.g., duplicated code, complex logic).
* **Security**: Potential vulnerabilities or missing security best practices (e.g., simulated verification instead of actual cryptography).
* **Testing**: Missing or inadequate test coverage for critical components.
* **Documentation**: Outdated, missing, or inaccurate documentation.
* **Observability**: Insufficient logging, monitoring, or tracing capabilities.