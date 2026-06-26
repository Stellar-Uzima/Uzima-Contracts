# Error Codes Registry

> Last updated: 2026-06-20
> Source of truth for shared error policy, migration status, and reserved unique ranges.

## Policy

- `common_error` owns the shared low range: `0-99`
- New contract-specific registries should use unique, non-overlapping ranges at `1000+`
- Legacy contracts that still use shared category bands (`100-899`) are tracked below until migrated
- Legacy contracts that still use sequential local codes (`1-99`) are tracked below until migrated

## Audit Summary

Audit date: `2026-06-20`

- State-mutating public APIs returning `bool` or `Result<bool, _>` are still present in multiple contracts; result-first compatibility shims were added in this pass for `anomaly_detection`, `drug_discovery`, `patient_risk_stratification`, `health_check`, `genomic_data`, and `zk_verifier`
- Non-test `unwrap()` remains present in several legacy contracts outside the migrated set; CI now guards against introducing new production `unwrap()` calls without updating the allowlist
- Shared `common_error` is now used directly by at least 5 contracts: `anomaly_detection`, `drug_discovery`, `patient_risk_stratification`, `health_check`, and `genomic_data`

## Contract Range Registry

Only rows with status `unique-range` are enforced for overlap by `scripts/check_error_codes.sh`.

| Contract | Status | Reserved Range(s) | Notes |
|---|---|---:|---|
| `common_error` | `unique-range` | `0-99` | Shared reusable contract-agnostic errors |
| `medical_records` | `unique-range` | `1000-1899` | Already migrated to contract-unique numbering |
| `rbac` | `unique-range` | `2000-2099` | Uses `common_error` for shared low-code compatibility helpers |
| `appointment_booking_escrow` | `legacy-shared-category` | `100-599` | Still uses shared category bands |
| `audit` | `legacy-shared-category` | `100-499` | Still uses shared category bands |
| `clinical_nlp` | `legacy-shared-category` | `100-899` | Still uses shared category bands |
| `code_ownership` | `legacy-sequential` | `1-99` | Pending migration |
| `contract_template` | `legacy-sequential` | `1-99` | Pending migration |
| `cross_chain_bridge` | `legacy-shared-category` | `100-899` | Still uses shared category bands |
| `deprecation_framework` | `legacy-sequential` | `1-99` | Pending migration |
| `emergency_access_override` | `legacy-shared-category` | `100-499` | Still uses shared category bands |
| `escrow` | `legacy-shared-category` | `100-899` | Still uses shared category bands |
| `governor` | `legacy-shared-category` | `200-599` | Still uses shared category bands |
| `healthcare_payment` | `legacy-shared-category` | `100-899` | Still uses shared category bands |
| `identity_registry` | `legacy-shared-category` | `100-699` | Still uses shared category bands |
| `iot_device_management` | `legacy-shared-category` | `100-899` | Still uses shared category bands |
| `medical_record_hash_registry` | `legacy-shared-category` | `100-799` | Still uses shared category bands |
| `notification_system` | `legacy-shared-category` | `100-599` | Still uses shared category bands |
| `patient_consent_management` | `legacy-shared-category` | `100-499` | Still uses shared category bands |
| `runtime_validation` | `legacy-sequential` | `1-99` | Pending migration |
| `timelock` | `legacy-shared-category` | `100-799` | Still uses shared category bands |
| `token_sale` | `legacy-sequential` | `1-99` | Pending migration |
| `upgrade_manager` | `legacy-shared-category` | `100-499` | Still uses shared category bands |
| `meta_tx_forwarder` | `legacy-sequential` | `1-10` | Inline in lib.rs; defines errors 1–10 (InvalidSignature, InvalidNonce, RequestExpired, ExecutionFailed, Unauthorized, AlreadyInitialized, OwnerNotSet, BatchLengthMismatch, PubKeyNotRegistered, InvalidFeePercentage) |
| `zk_verifier` | `legacy-shared-category` | `100-699` | Still uses shared category bands |

## Migrated Result-Based Entry Points

These contracts now expose canonical `try_*` state-mutating functions that return `Result<(), ContractError>` while preserving the previous public entrypoints as compatibility wrappers:

- `anomaly_detection`
- `drug_discovery`
- `patient_risk_stratification`
- `health_check`
- `genomic_data`
- `zk_verifier`

## Follow-Up Migration Queue

- Move all `legacy-sequential` contracts off `1-99`
- Move all `legacy-shared-category` contracts onto unique `1000+` contract ranges
- Remove compatibility `bool` wrappers after downstream callers have migrated to `try_*` entrypoints
