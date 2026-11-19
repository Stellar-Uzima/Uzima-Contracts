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
| `fractional_ownership` | `unique-range` | `3000-3099` | Share ledger, fixed-price listings and payout pool (#1785-#1788) |
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

### Newly registered contracts (issue #1949)

| `aml` | `legacy-sequential` | `1-3` | Registry entry added from source error enum (issue #1949) |
| `anomaly_detection` | `legacy-sequential` | `1-10` | Registry entry added from source error enum (issue #1949) |
| `anomaly_detector` | `legacy-sequential` | `1-14` | Registry entry added from source error enum (issue #1949) |
| `bridge_dispute_mediation` | `legacy-sequential` | `1-9` | Registry entry added from source error enum (issue #1949) |
| `clinical_trial` | `legacy-sequential` | `1-9` | Registry entry added from source error enum (issue #1949) |
| `contract_usage_analytics` | `legacy-sequential` | `1-4` | Registry entry added from source error enum (issue #1949) |
| `credential_notifications` | `legacy-sequential` | `1-4` | Registry entry added from source error enum (issue #1949) |
| `credential_registry` | `legacy-sequential` | `1-9` | Registry entry added from source error enum (issue #1949) |
| `cross_chain_access` | `legacy-sequential` | `1-23` | Registry entry added from source error enum (issue #1949) |
| `cross_chain_enhancements` | `legacy-sequential` | `1-11` | Registry entry added from source error enum (issue #1949) |
| `cross_chain_identity` | `legacy-sequential` | `1-18` | Registry entry added from source error enum (issue #1949) |
| `crypto_registry` | `legacy-sequential` | `1-7` | Registry entry added from source error enum (issue #1949) |
| `dicomweb_services` | `legacy-sequential` | `1-15` | Registry entry added from source error enum (issue #1949) |
| `differential_privacy` | `legacy-sequential` | `1-11` | Registry entry added from source error enum (issue #1949) |
| `digital_twin` | `legacy-sequential` | `1-19` | Registry entry added from source error enum (issue #1949) |
| `dispute_resolution` | `legacy-sequential` | `1-9` | Registry entry added from source error enum (issue #1949) |
| `drug_discovery` | `legacy-sequential` | `1-9` | Registry entry added from source error enum (issue #1949) |
| `emr_integration` | `legacy-sequential` | `1-31` | Registry entry added from source error enum (issue #1949) |
| `explainable_ai` | `legacy-sequential` | `1-6` | Registry entry added from source error enum (issue #1949) |
| `failover_detector` | `legacy-sequential` | `1-10` | Registry entry added from source error enum (issue #1949) |
| `federated_learning` | `legacy-sequential` | `1-24` | Registry entry added from source error enum (issue #1949) |
| `fhir_integration` | `legacy-sequential` | `1-21` | Registry entry added from source error enum (issue #1949) |
| `fido2_authenticator` | `legacy-sequential` | `1-8` | Registry entry added from source error enum (issue #1949) |
| `forensics` | `legacy-sequential` | `1-4` | Registry entry added from source error enum (issue #1949) |
| `healthcare_analytics_dashboard` | `legacy-sequential` | `1-16` | Registry entry added from source error enum (issue #1949) |
| `healthcare_compliance` | `legacy-sequential` | `1-26` | Registry entry added from source error enum (issue #1949) |
| `healthcare_compliance_automation` | `legacy-sequential` | `1-9` | Registry entry added from source error enum (issue #1949) |
| `healthcare_data_conversion` | `legacy-sequential` | `1-17` | Registry entry added from source error enum (issue #1949) |
| `healthcare_data_marketplace` | `legacy-sequential` | `1-15` | Registry entry added from source error enum (issue #1949) |
| `healthcare_reputation` | `legacy-sequential` | `1-16` | Registry entry added from source error enum (issue #1949) |
| `homomorphic_registry` | `legacy-sequential` | `1-14` | Registry entry added from source error enum (issue #1949) |
| `ihe_integration` | `legacy-sequential` | `1-25` | Registry entry added from source error enum (issue #1949) |
| `medical_imaging` | `legacy-sequential` | `1-23` | Registry entry added from source error enum (issue #1949) |
| `medical_imaging_ai` | `legacy-sequential` | `1-18` | Registry entry added from source error enum (issue #1949) |
| `medical_record_backup` | `legacy-sequential` | `1-18` | Registry entry added from source error enum (issue #1949) |
| `medical_record_search` | `legacy-sequential` | `1-12` | Registry entry added from source error enum (issue #1949) |
| `medication_management` | `legacy-sequential` | `1-11` | Registry entry added from source error enum (issue #1949) |
| `mpc_manager` | `legacy-sequential` | `1-15` | Registry entry added from source error enum (issue #1949) |
| `multi_region_orchestrator` | `legacy-sequential` | `1-10` | Registry entry added from source error enum (issue #1949) |
| `patient_gamification` | `legacy-sequential` | `1-19` | Registry entry added from source error enum (issue #1949) |
| `patient_risk_stratification` | `legacy-sequential` | `1-9` | Registry entry added from source error enum (issue #1949) |
| `payment_router` | `legacy-sequential` | `1-16` | Registry entry added from source error enum (issue #1949) |
| `pharma_supply_chain` | `legacy-sequential` | `1-9` | Registry entry added from source error enum (issue #1949) |
| `property_registry` | `legacy-sequential` | `1-11` | Registry entry added from source error enum (issue #1949) |
| `provider_directory` | `legacy-sequential` | `1-9` | Registry entry added from source error enum (issue #1949) |
| `public_health_surveillance` | `legacy-sequential` | `1-15` | Registry entry added from source error enum (issue #1949) |
| `regional_node_manager` | `legacy-sequential` | `1-10` | Registry entry added from source error enum (issue #1949) |
| `regulatory_compliance` | `legacy-sequential` | `1-5` | Registry entry added from source error enum (issue #1949) |
| `rental_yield` | `legacy-sequential` | `1-9` | Registry entry added from source error enum (issue #1949) |
| `reputation` | `legacy-sequential` | `1-4` | Registry entry added from source error enum (issue #1949) |
| `reputation_access_control` | `legacy-sequential` | `1-9` | Registry entry added from source error enum (issue #1949) |
| `reputation_integration` | `legacy-sequential` | `1-8` | Registry entry added from source error enum (issue #1949) |
| `spending-categories` | `legacy-shared-category` | `100-450` | Registry entry added from source error enum (issue #1949) |
| `spending-limits` | `legacy-shared-category` | `100-500` | Registry entry added from source error enum (issue #1949) |
| `spending-rules` | `legacy-shared-category` | `100-600` | Registry entry added from source error enum (issue #1949) |
| `storage_cleanup` | `legacy-sequential` | `1-5` | Registry entry added from source error enum (issue #1949) |
| `storage_migration` | `legacy-sequential` | `1-7` | Registry entry added from source error enum (issue #1949) |
| `sync_manager` | `legacy-sequential` | `1-13` | Registry entry added from source error enum (issue #1949) |
| `tokenization` | `legacy-sequential` | `1-14` | Registry entry added from source error enum (issue #1949) |
| `treasury_controller` | `legacy-sequential` | `1-15` | Registry entry added from source error enum (issue #1949) |
| `zkp_registry` | `legacy-shared-category` | `1-612` | Registry entry added from source error enum (issue #1949) |
