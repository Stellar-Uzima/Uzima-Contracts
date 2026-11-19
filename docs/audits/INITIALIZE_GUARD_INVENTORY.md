# Initialize Guard Inventory

Resolution: #1896  
Source scanned: `Stellar-Uzima/Uzima-Contracts` at `d6152d14db1e620d29608917343155d616d53b19`.

This inventory lists every Rust `pub fn initialize` / `pub fn init` found in the repository. Each entry is classified as using the shared `governance_commons` init guard, delegating to a helper that uses it, using a documented local-guard waiver, or being a non-deployed documentation/example occurrence.

## Summary

| Classification | Count |
| --- | ---: |
| Shared guard | 74 |
| Delegated guard | 3 |
| Waiver: local guard | 34 |
| Documentation/example | 6 |
| WAIVER REQUIRED | 0 |
| **Total** | **117** |

No unclassified `WAIVER REQUIRED` entries remain. Local-guard waivers all reject re-initialization before mutating admin/config state, either by returning `AlreadyInitialized` or panicking on an existing admin/initialized flag. Delegated entries call helper modules that use the shared guard.

## Full inventory

| Location | Function | Classification | Note |
| --- | --- | --- | --- |
| `contract_optimizer/src/complexity.rs:636` | `init` | Documentation/example | Not a deployed contract entry point |
| `contracts/ai_analytics/src/admin.rs:6` | `initialize` | Shared guard | Uses governance_commons init guard |
| `contracts/ai_analytics/src/lib.rs:30` | `initialize` | Delegated guard | Delegates to helper module that uses shared guard |
| `contracts/aml/src/lib.rs:46` | `initialize` | Shared guard | Uses governance_commons init guard |
| `contracts/anomaly_detection/src/batch.rs:109` | `initialize` | Waiver: local guard | Rejects re-initialization with contract-local check |
| `contracts/anomaly_detection/src/batch.rs:1111` | `initialize` | Waiver: local guard | Rejects re-initialization with contract-local check |
| `contracts/anomaly_detection/src/lib.rs:125` | `initialize` | Waiver: local guard | Rejects re-initialization with contract-local check |
| `contracts/anomaly_detection/src/payload.rs:23` | `initialize` | Waiver: local guard | Rejects re-initialization with contract-local check |
| `contracts/anomaly_detector/src/lib.rs:232` | `initialize` | Waiver: local guard | Rejects re-initialization with contract-local check |
| `contracts/appointment_booking_escrow/src/lib.rs:83` | `initialize` | Shared guard | Uses governance_commons init guard |
| `contracts/audit/src/bud.rs:62` | `initialize` | Waiver: local guard | Rejects re-initialization with contract-local check |
| `contracts/audit/src/lib.rs:41` | `initialize` | Waiver: local guard | Rejects re-initialization with contract-local check |
| `contracts/audit/src/vec.rs:28` | `initialize` | Waiver: local guard | Rejects re-initialization with contract-local check |
| `contracts/audit_forensics/src/lib.rs:184` | `initialize` | Shared guard | Uses governance_commons init guard |
| `contracts/bridge_dispute_mediation/src/lib.rs:258` | `initialize` | Waiver: local guard | Rejects re-initialization with contract-local check |
| `contracts/clinical_decision_support/src/lib.rs:62` | `initialize` | Shared guard | Uses governance_commons init guard |
| `contracts/clinical_nlp/src/lib.rs:67` | `initialize` | Waiver: local guard | Rejects re-initialization with contract-local check |
| `contracts/clinical_trial/src/lib.rs:185` | `initialize` | Shared guard | Uses governance_commons init guard |
| `contracts/code_ownership/src/lib.rs:23` | `initialize` | Shared guard | Uses governance_commons init guard |
| `contracts/contract_monitoring/src/lib.rs:119` | `initialize` | Shared guard | Uses governance_commons init guard |
| `contracts/contract_monitoring/src/lib.rs:828` | `initialize` | Documentation/example | Commented example in contract source |
| `contracts/contract_template/src/lib.rs:71` | `initialize` | Waiver: local guard | Rejects re-initialization with contract-local check |
| `contracts/contract_usage_analytics/src/lib.rs:76` | `initialize` | Shared guard | Uses governance_commons init guard |
| `contracts/contract_verification/src/lib.rs:119` | `initialize` | Shared guard | Uses governance_commons init guard |
| `contracts/credential_notifications/src/lib.rs:57` | `initialize` | Shared guard | Uses governance_commons init guard |
| `contracts/credential_registry/src/lib.rs:104` | `initialize` | Shared guard | Uses governance_commons init guard |
| `contracts/cross_chain_access/src/lib.rs:277` | `initialize` | Waiver: local guard | Rejects re-initialization with contract-local check |
| `contracts/cross_chain_bridge/src/lib.rs:458` | `initialize` | Shared guard | Uses governance_commons init guard |
| `contracts/cross_chain_enhancements/src/lib.rs:137` | `initialize` | Waiver: local guard | Rejects re-initialization with contract-local check |
| `contracts/cross_chain_identity/src/lib.rs:202` | `initialize` | Waiver: local guard | Rejects re-initialization with contract-local check |
| `contracts/crypto_registry/src/lib.rs:155` | `initialize` | Shared guard | Uses governance_commons init guard |
| `contracts/deprecation_framework/src/lib.rs:25` | `initialize` | Shared guard | Uses governance_commons init guard |
| `contracts/dicomweb_services/src/lib.rs:269` | `initialize` | Waiver: local guard | Rejects re-initialization with contract-local check |
| `contracts/differential_privacy/src/lib.rs:129` | `initialize` | Shared guard | Uses governance_commons init guard |
| `contracts/digital_twin/src/lib.rs:289` | `initialize` | Shared guard | Uses governance_commons init guard |
| `contracts/dispute_resolution/src/lib.rs:126` | `initialize` | Shared guard | Uses governance_commons init guard |
| `contracts/drug_discovery/src/lib.rs:200` | `initialize` | Waiver: local guard | Rejects re-initialization with contract-local check |
| `contracts/emergency_access_override/src/lib.rs:145` | `initialize` | Shared guard | Uses governance_commons init guard |
| `contracts/emr_integration/src/lib.rs:381` | `initialize` | Shared guard | Uses governance_commons init guard |
| `contracts/escrow/src/lib.rs:199` | `initialize` | Shared guard | Uses governance_commons init guard |
| `contracts/explainable_ai/src/lib.rs:164` | `initialize` | Shared guard | Uses governance_commons init guard |
| `contracts/failover_detector/src/lib.rs:148` | `initialize` | Shared guard | Uses governance_commons init guard |
| `contracts/federated_learning/src/lib.rs:276` | `initialize` | Shared guard | Uses governance_commons init guard |
| `contracts/fhir_integration/src/lib.rs:284` | `initialize` | Shared guard | Uses governance_commons init guard |
| `contracts/fido2_authenticator/src/lib.rs:325` | `initialize` | Waiver: local guard | Rejects re-initialization with contract-local check |
| `contracts/forensics/src/lib.rs:49` | `initialize` | Shared guard | Uses governance_commons init guard |
| `contracts/fractional_ownership/src/lib.rs:89` | `initialize` | Waiver: local guard | Rejects re-initialization with contract-local check |
| `contracts/genomic_data/src/lib.rs:308` | `initialize` | Waiver: local guard | Rejects re-initialization with contract-local check |
| `contracts/governor/src/lib.rs:141` | `initialize` | Shared guard | Uses governance_commons init guard |
| `contracts/health_check/src/lib.rs:109` | `initialize` | Waiver: local guard | Rejects re-initialization with contract-local check |
| `contracts/health_data_access_logging/src/lib.rs:31` | `initialize` | Shared guard | Uses governance_commons init guard |
| `contracts/healthcare_analytics_dashboard/src/lib.rs:323` | `initialize` | Shared guard | Uses governance_commons init guard |
| `contracts/healthcare_compliance/src/lib.rs:362` | `initialize` | Shared guard | Uses governance_commons init guard |
| `contracts/healthcare_compliance_automation/src/lib.rs:135` | `initialize` | Shared guard | Uses governance_commons init guard |
| `contracts/healthcare_data_conversion/src/lib.rs:195` | `initialize` | Shared guard | Uses governance_commons init guard |
| `contracts/healthcare_data_marketplace/src/lib.rs:188` | `initialize` | Waiver: local guard | Rejects re-initialization with contract-local check |
| `contracts/healthcare_oracle_network/src/admin.rs:6` | `initialize` | Shared guard | Uses governance_commons init guard |
| `contracts/healthcare_oracle_network/src/lib.rs:32` | `initialize` | Delegated guard | Delegates to helper module that uses shared guard |
| `contracts/healthcare_payment/src/lib.rs:472` | `initialize` | Shared guard | Uses governance_commons init guard |
| `contracts/healthcare_reputation/src/lib.rs:221` | `initialize` | Waiver: local guard | Rejects re-initialization with contract-local check |
| `contracts/homomorphic_registry/src/lib.rs:183` | `initialize` | Waiver: local guard | Rejects re-initialization with contract-local check |
| `contracts/identity_registry/src/lib.rs:358` | `initialize` | Shared guard | Uses governance_commons init guard |
| `contracts/ihe_integration/src/lib.rs:501` | `initialize` | Shared guard | Uses governance_commons init guard |
| `contracts/iot_device_management/src/lib.rs:230` | `initialize` | Shared guard | Uses governance_commons init guard |
| `contracts/medical_consent_nft/src/lib.rs:230` | `initialize` | Waiver: local guard | Rejects re-initialization with contract-local check |
| `contracts/medical_imaging/src/lib.rs:361` | `initialize` | Shared guard | Uses governance_commons init guard |
| `contracts/medical_imaging_ai/src/lib.rs:225` | `initialize` | Shared guard | Uses governance_commons init guard |
| `contracts/medical_record_backup/src/lib.rs:313` | `initialize` | Shared guard | Uses governance_commons init guard |
| `contracts/medical_record_hash_registry/src/lib.rs:48` | `initialize` | Shared guard | Uses governance_commons init guard |
| `contracts/medical_record_search/src/lib.rs:213` | `initialize` | Shared guard | Uses governance_commons init guard |
| `contracts/medical_records/src/lib.rs:1257` | `initialize` | Waiver: local guard | Rejects re-initialization with contract-local check |
| `contracts/medication_management/src/lib.rs:269` | `initialize` | Shared guard | Uses governance_commons init guard |
| `contracts/mental_health_support/src/lib.rs:142` | `initialize` | Shared guard | Uses governance_commons init guard |
| `contracts/meta_tx_forwarder/src/lib.rs:200` | `initialize` | Shared guard | Uses governance_commons init guard |
| `contracts/mfa/src/lib.rs:30` | `initialize` | Shared guard | Uses governance_commons init guard |
| `contracts/mpc_manager/src/lib.rs:207` | `initialize` | Shared guard | Uses governance_commons init guard |
| `contracts/multi_region_orchestrator/src/lib.rs:165` | `initialize` | Shared guard | Uses governance_commons init guard |
| `contracts/notification_system/src/lib.rs:101` | `initialize` | Shared guard | Uses governance_commons init guard |
| `contracts/patient_consent_management/src/lib.rs:142` | `initialize` | Waiver: local guard | Rejects re-initialization with contract-local check |
| `contracts/patient_gamification/src/lib.rs:249` | `initialize` | Shared guard | Uses governance_commons init guard |
| `contracts/patient_portal/src/lib.rs:140` | `initialize` | Shared guard | Uses governance_commons init guard |
| `contracts/patient_risk_stratification/src/lib.rs:139` | `initialize` | Waiver: local guard | Rejects re-initialization with contract-local check |
| `contracts/pharma_supply_chain/src/lib.rs:172` | `initialize` | Waiver: local guard | Rejects re-initialization with contract-local check |
| `contracts/predictive_analytics/src/config.rs:8` | `initialize` | Shared guard | Uses governance_commons init guard |
| `contracts/predictive_analytics/src/lib.rs:27` | `initialize` | Delegated guard | Delegates to helper module that uses shared guard |
| `contracts/property_registry/src/contract.rs:77` | `initialize` | Shared guard | Uses governance_commons init guard |
| `contracts/provider_directory/src/lib.rs:101` | `initialize` | Shared guard | Uses governance_commons init guard |
| `contracts/public_health_surveillance/src/lib.rs:401` | `initialize` | Shared guard | Uses governance_commons init guard |
| `contracts/rbac/src/lib.rs:79` | `initialize` | Shared guard | Uses governance_commons init guard |
| `contracts/regional_node_manager/src/lib.rs:134` | `initialize` | Shared guard | Uses governance_commons init guard |
| `contracts/regulatory_compliance/src/lib.rs:71` | `initialize` | Shared guard | Uses governance_commons init guard |
| `contracts/remote_patient_monitoring/src/lib.rs:61` | `initialize` | Shared guard | Uses governance_commons init guard |
| `contracts/rental_yield/src/lib.rs:37` | `initialize` | Waiver: local guard | Rejects re-initialization with contract-local check |
| `contracts/reputation/src/lib.rs:35` | `initialize` | Shared guard | Uses governance_commons init guard |
| `contracts/reputation_access_control/src/lib.rs:133` | `initialize` | Waiver: local guard | Rejects re-initialization with contract-local check |
| `contracts/reputation_integration/src/lib.rs:97` | `initialize` | Waiver: local guard | Rejects re-initialization with contract-local check |
| `contracts/runtime_validation/src/lib.rs:25` | `initialize` | Shared guard | Uses governance_commons init guard |
| `contracts/secure_enclave/src/lib.rs:70` | `initialize` | Shared guard | Uses governance_commons init guard |
| `contracts/spending-categories/src/lib.rs:23` | `initialize` | Shared guard | Uses governance_commons init guard |
| `contracts/spending-limits/src/lib.rs:23` | `initialize` | Shared guard | Uses governance_commons init guard |
| `contracts/spending-rules/src/lib.rs:27` | `initialize` | Shared guard | Uses governance_commons init guard |
| `contracts/storage_cleanup/src/lib.rs:93` | `initialize` | Shared guard | Uses governance_commons init guard |
| `contracts/storage_migration/src/lib.rs:47` | `initialize` | Waiver: local guard | Rejects re-initialization with contract-local check |
| `contracts/sync_manager/src/lib.rs:175` | `initialize` | Shared guard | Uses governance_commons init guard |
| `contracts/sync_manager/src/lib.rs:937` | `initialize` | Waiver: local guard | Rejects re-initialization with contract-local check |
| `contracts/telemedicine/src/lib.rs:431` | `initialize` | Shared guard | Uses governance_commons init guard |
| `contracts/timelock/src/lib.rs:38` | `initialize` | Shared guard | Uses governance_commons init guard |
| `contracts/token_sale/src/contract.rs:21` | `initialize` | Shared guard | Uses governance_commons init guard |
| `contracts/tokenization/src/contract.rs:27` | `initialize` | Shared guard | Uses governance_commons init guard |
| `contracts/treasury_controller/src/lib.rs:156` | `initialize` | Shared guard | Uses governance_commons init guard |
| `contracts/upgrade_manager/src/lib.rs:63` | `initialize` | Shared guard | Uses governance_commons init guard |
| `contracts/zk_verifier/src/lib.rs:66` | `initialize` | Waiver: local guard | Rejects re-initialization with contract-local check |
| `contracts/zkp_registry/src/lib.rs:490` | `initialize` | Shared guard | Uses governance_commons init guard |
| `libs/governance_commons/src/init_guard.rs:45` | `initialize` | Documentation/example | Not a deployed contract entry point |
| `libs/governance_commons/src/init_guard.rs:57` | `initialize` | Documentation/example | Not a deployed contract entry point |
| `libs/validation_utils/src/storage_layout.rs:261` | `initialize` | Documentation/example | Not a deployed contract entry point |
| `mobile-sdk/core/src/client/UzimaClientTypes.rs:188` | `initialize` | Documentation/example | Not a deployed contract entry point |
