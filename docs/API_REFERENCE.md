# Uzima Contracts — API Reference

> Auto-generated from contract source code. Do not edit manually.

- **API version**: `1.0.0`
- **Generated**: `2026-07-23T14:22:33.938Z`
- **Contracts documented**: 102

## Table of Contents

- [access_control](#access-control)
- [ai_analytics](#ai-analytics)
- [aml](#aml)
- [anomaly_detection](#anomaly-detection)
- [anomaly_detector](#anomaly-detector)
- [appointment_booking_escrow](#appointment-booking-escrow)
- [audit](#audit)
- [audit_forensics](#audit-forensics)
- [clinical_decision_support](#clinical-decision-support)
- [clinical_nlp](#clinical-nlp)
- [clinical_trial](#clinical-trial)
- [code_ownership](#code-ownership)
- [common_error](#common-error)
- [contract_behavior_fuzzing](#contract-behavior-fuzzing)
- [contract_monitoring](#contract-monitoring)
- [contract_template](#contract-template)
- [contract_usage_analytics](#contract-usage-analytics)
- [contract_verification](#contract-verification)
- [credential_notifications](#credential-notifications)
- [credential_registry](#credential-registry)
- [cross_chain_access](#cross-chain-access)
- [cross_chain_bridge](#cross-chain-bridge)
- [cross_chain_enhancements](#cross-chain-enhancements)
- [cross_chain_identity](#cross-chain-identity)
- [crypto_registry](#crypto-registry)
- [deprecation_framework](#deprecation-framework)
- [dicomweb_services](#dicomweb-services)
- [differential_privacy](#differential-privacy)
- [digital_twin](#digital-twin)
- [dispute_resolution](#dispute-resolution)
- [drug_discovery](#drug-discovery)
- [emergency_access_override](#emergency-access-override)
- [emr_integration](#emr-integration)
- [escrow](#escrow)
- [explainable_ai](#explainable-ai)
- [failover_detector](#failover-detector)
- [federated_learning](#federated-learning)
- [fhir_integration](#fhir-integration)
- [fido2_authenticator](#fido2-authenticator)
- [forensics](#forensics)
- [fp_math](#fp-math)
- [genomic_data](#genomic-data)
- [governor](#governor)
- [health_check](#health-check)
- [health_data_access_logging](#health-data-access-logging)
- [healthcare_analytics_dashboard](#healthcare-analytics-dashboard)
- [healthcare_compliance](#healthcare-compliance)
- [healthcare_compliance_automation](#healthcare-compliance-automation)
- [healthcare_data_conversion](#healthcare-data-conversion)
- [healthcare_data_marketplace](#healthcare-data-marketplace)
- [healthcare_oracle_network](#healthcare-oracle-network)
- [healthcare_payment](#healthcare-payment)
- [healthcare_reputation](#healthcare-reputation)
- [homomorphic_registry](#homomorphic-registry)
- [identity_registry](#identity-registry)
- [ihe_integration](#ihe-integration)
- [iot_device_management](#iot-device-management)
- [load_testing](#load-testing)
- [medical_consent_nft](#medical-consent-nft)
- [medical_imaging](#medical-imaging)
- [medical_imaging_ai](#medical-imaging-ai)
- [medical_record_backup](#medical-record-backup)
- [medical_record_hash_registry](#medical-record-hash-registry)
- [medical_record_search](#medical-record-search)
- [medical_records](#medical-records)
- [medication_management](#medication-management)
- [mental_health_support](#mental-health-support)
- [meta_tx_forwarder](#meta-tx-forwarder)
- [mfa](#mfa)
- [mpc_manager](#mpc-manager)
- [multi_region_orchestrator](#multi-region-orchestrator)
- [notification_system](#notification-system)
- [patient_consent_management](#patient-consent-management)
- [patient_gamification](#patient-gamification)
- [patient_portal](#patient-portal)
- [patient_risk_stratification](#patient-risk-stratification)
- [payment_router](#payment-router)
- [pharma_supply_chain](#pharma-supply-chain)
- [predictive_analytics](#predictive-analytics)
- [provider_directory](#provider-directory)
- [public_health_surveillance](#public-health-surveillance)
- [rbac](#rbac)
- [regional_node_manager](#regional-node-manager)
- [regulatory_compliance](#regulatory-compliance)
- [remote_patient_monitoring](#remote-patient-monitoring)
- [reputation](#reputation)
- [reputation_access_control](#reputation-access-control)
- [reputation_integration](#reputation-integration)
- [runtime_validation](#runtime-validation)
- [sanitization](#sanitization)
- [secure_enclave](#secure-enclave)
- [storage_cleanup](#storage-cleanup)
- [sut_token](#sut-token)
- [sync_manager](#sync-manager)
- [telemedicine](#telemedicine)
- [timelock](#timelock)
- [token_sale](#token-sale)
- [treasury_controller](#treasury-controller)
- [upgrade_manager](#upgrade-manager)
- [upgradeability](#upgradeability)
- [zk_verifier](#zk-verifier)
- [zkp_registry](#zkp-registry)

---

## access_control

### Functions

| Function | Parameters | Returns | Description |
|---|---|---|---|
| `init` | `env: &Env, admin: &Address` | `()` | Store the initial admin.  Must be called once during contract `initialize`. |
| `get_admin` | `env: &Env` | `Address` | Return the stored admin address. |
| `require_admin` | `env: &Env` | `Result<(), AccessError>` | Require that the transaction was authorised by the admin. |
| `transfer_admin` | `env: &Env, new_admin: &Address` | `Result<(), AccessError>` | Transfer admin rights to a new address. |
| `grant_role` | `env: &Env, address: &Address, role: Role` | `Result<(), AccessError>` | Assign `role` to `address`.  Caller must be admin. |
| `revoke_role` | `env: &Env, address: &Address` | `Result<(), AccessError>` | Remove the role from `address`.  Caller must be admin. |
| `get_role` | `env: &Env, address: &Address` | `Option<Role>` | Return the role assigned to `address`, if any. |
| `has_role` | `env: &Env, address: &Address, role: Role` | `bool` | Return `true` when `address` holds exactly `role`. |
| `require_role` | `env: &Env, address: &Address, role: Role` | `Result<(), AccessError>` | Require that `address` holds `role`, otherwise return `Unauthorized`. |
| `grant_permission` | `env: &Env, address: &Address, permission: Permission` | `Result<(), AccessError>` | Grant a fine-grained `permission` to `address`.  Caller must be admin. |
| `revoke_permission` | `env: &Env, address: &Address, permission: Permission` | `Result<(), AccessError>` | Revoke a fine-grained `permission` from `address`.  Caller must be admin. |
| `has_permission` | `env: &Env, address: &Address, permission: Permission` | `bool` | Return `true` when `address` has been explicitly granted `permission`. |

### Types

#### `enum DataKey`

| Variant | Value | Description |
|---|---|---|
| `Admin` | — | — |
| `Role(Address)` | — | — |
| `Permission(Address, u32)` | — | — |

#### `enum Role`

| Variant | Value | Description |
|---|---|---|
| `Admin` | 0 | — |
| `Doctor` | 1 | — |
| `Patient` | 2 | — |
| `Staff` | 3 | — |
| `Insurer` | 4 | — |
| `Researcher` | 5 | — |
| `Auditor` | 6 | — |
| `Service` | 7 | — |

#### `enum Permission`

| Variant | Value | Description |
|---|---|---|
| `ManageUsers` | 1 | — |
| `ManageSystem` | 2 | — |
| `CreateRecord` | 10 | — |
| `ReadRecord` | 11 | — |
| `UpdateRecord` | 12 | — |
| `DeleteRecord` | 13 | — |
| `ReadConfidential` | 20 | — |
| `GrantAccess` | 30 | — |
| `RevokeAccess` | 31 | — |

#### `enum AccessError`

| Variant | Value | Description |
|---|---|---|
| `Unauthorized` | 1 | — |
| `NotInitialized` | 2 | — |
| `AlreadyInitialized` | 3 | — |
| `InvalidRole` | 4 | — |

---

## ai_analytics

### Functions

| Function | Parameters | Returns | Description |
|---|---|---|---|
| `initialize` | `env: Env, admin: Address` | `Result<bool, Error>` | — |
| `start_round` | `env: Env, caller: Address, base_model_id: BytesN<32>, min_participants: u32, dp_epsilon: u32` | `Result<u64, Error>` | — |
| `submit_update` | `env: Env, participant: Address, round_id: u64, update_hash: BytesN<32>, num_samples: u32` | `Result<bool, Error>` | — |
| `finalize_round` | `env: Env, caller: Address, round_id: u64, new_model_id: BytesN<32>, description: String, metrics_ref: String, fairness_report_ref: String` | `Result<bool, Error>` | — |
| `get_round` | `env: Env, round_id: u64` | `Option<FederatedRound>` | — |
| `get_model` | `env: Env, model_id: BytesN<32>` | `Option<ModelMetadata>` | — |

### Examples

#### `test_initialize`

```rust
let (_env, client, _admin) = setup();

    // Verify contract is initialized (no panic = success)
```

#### `test_double_initialize`

```rust
let (env, client, _admin) = setup();
    let admin2 = Address::generate(&env);

    let result = client.mock_all_auths().try_initialize(&admin2);
    assert!(result.is_err());
```

#### `test_start_round`

```rust
let (env, client, admin) = setup();
    let base_model = BytesN::from_array(&env, &[1u8; 32]);

    let round_id = client
        .mock_all_auths()
        .start_round(&admin, &base_model, &2, &1);
    assert!(round_id > 0);

    let round: FederatedRound = client.get_round(&round_id).unwrap();
```

---

## aml

### Functions

| Function | Parameters | Returns | Description |
|---|---|---|---|
| `initialize` | `env: Env, admin: Address` | `Result<(), Error>` | Initialize AML with admin |
| `configure_rule` | `env: Env, admin: Address, id: u32, name: String, description: String, threshold: i128, risk_contribution: u32` | `Result<(), Error>` | Configure an AML rule |
| `monitor_transaction` | `env: Env, user: Address, amount: i128, _target: Option<Address>` | `RiskLevel` | Monitor a transaction and update risk profile |
| `is_compliant` | `env: Env, user: Address` | `bool` | Check if a user is compliant with platform AML policy |
| `update_user_status` | `env: Env, admin: Address, user: Address, is_blacklisted: bool` | `Result<(), Error>` | Update blacklist status for a user. |
| `set_user_status` | `env: Env, admin: Address, user: Address, is_blacklisted: bool` | `Result<(), Error>` | Blacklist or whitelist an address manually by admin. |
| `report_incident` | `env: Env, admin: Address, subject: Address, summary: String, evidence: String` | `Result<u64, Error>` | Generate an AML compliance report for regulatory use |
| `register_deprecated_functions` | `env: Env, admin: Address` | `Result<(), upgradeability::UpgradeError>` | Register AML deprecated entrypoints for upgrade and migration tracking. |
| `get_deprecated_functions` | `env: Env` | `Vec<upgradeability::DeprecatedFunction>` | Return tracked deprecated AML entrypoints. |
| `upgrade` | `env: Env, admin: Address, new_wasm_hash: BytesN<32>, new_version: u32` | `Result<(), upgradeability::UpgradeError>` | Upgrade the AML contract and register deprecated entrypoints atomically. |
| `validate_upgrade` | `env: Env, new_wasm_hash: BytesN<32>` | `Result<upgradeability::UpgradeValidation, upgradeability::UpgradeError>` | Validate a proposed AML upgrade before execution. |

### Types

#### `enum Error`

| Variant | Value | Description |
|---|---|---|
| `AlreadyInitialized` | 1 | — |
| `NotInitialized` | 2 | — |
| `NotAuthorized` | 3 | — |

### Examples

#### `test_aml_lifecycle`

```rust
let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let contract_id = env.register_contract(None, AntiMoneyLaundering);
    let client = AntiMoneyLaunderingClient::new(&env, &contract_id);

    // 1. Initialize
    client.initialize(&admin);
```

#### `test_validate_upgrade_reports_initialized_state`

```rust
let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let contract_id = env.register_contract(None, AntiMoneyLaundering);
    let client = AntiMoneyLaunderingClient::new(&env, &contract_id);

    client.initialize(&admin);
```

---

## anomaly_detection

### Functions

| Function | Parameters | Returns | Description |
|---|---|---|---|
| `initialize` | `env: Env, admin: Address, detector: Address, threshold_bps: u32` | `Result<(), Error>` | — |
| `update_config` | `env: Env, caller: Address, new_detector: Option<Address>, new_threshold: Option<u32>, new_sensitivity: Option<u32>, enabled: Option<bool>` | `Result<(), Error>` | — |
| `set_audit_forensics` | `env: Env, admin: Address, forensics: Address` | `Result<(), Error>` | — |
| `detect_anomaly` | `env: Env, caller: Address, record_id: u64, patient: Address, score_bps: u32, severity: u32, metadata: String, explanation_ref: String` | `Result<u64, Error>` | — |
| `get_anomaly_record` | `env: Env, anomaly_id: u64` | `Option<AnomalyRecord>` | — |
| `get_config` | `env: Env` | `Option<AnomalyDetectionConfig>` | — |
| `get_stats` | `env: Env` | `DetectionStats` | — |
| `get_anomaly_count_for_patient` | `env: Env, patient: Address` | `u64` | — |
| `whitelist_detector` | `env: Env, caller: Address, detector_addr: Address` | `Result<bool, Error>` | — |
| `is_whitelisted_detector` | `env: Env, detector_addr: Address` | `bool` | — |
| `create_alert` | `env: Env, caller: Address, anomaly_id: u64` | `Result<u64, Error>` | Promote an anomaly record to an active alert for investigation tracking. |
| `acknowledge_alert` | `env: Env, caller: Address, alert_id: u64` | `Result<bool, Error>` | Acknowledge an alert (marks it as under review). |
| `resolve_alert` | `env: Env, caller: Address, alert_id: u64, notes: String` | `Result<bool, Error>` | Resolve an alert after investigation. |
| `mark_false_positive` | `env: Env, caller: Address, alert_id: u64` | `Result<bool, Error>` | Mark alert as false positive. Feeds adaptive threshold learning. |
| `submit_feedback` | `env: Env, caller: Address, anomaly_id: u64, confirmed: bool` | `Result<bool, Error>` | Submit feedback on a detection. Adaptive threshold learning: - `confirmed = true`  → lower threshold by 50 bps (catch more) - `confirmed = false` → raise threshold by 50 bps (reduce noise) |
| `get_alert` | `env: Env, alert_id: u64` | `Option<AnomalyAlert>` | — |
| `get_alert_count` | `env: Env` | `u64` | — |
| `get_suggestion` | `error: Error` | `Symbol` | — |

### Types

#### `enum AlertStatus`

| Variant | Value | Description |
|---|---|---|
| `Active` | — | — |
| `Acknowledged` | — | — |
| `Resolved` | — | — |
| `FalsePositive` | — | — |

#### `struct AnomalyAlert`

| Field | Type | Description |
|---|---|---|
| `alert_id` | `u64` | — |
| `anomaly_id` | `u64` | — |
| `patient` | `Address` | — |
| `score_bps` | `u32` | — |
| `severity` | `u32` | — |
| `status` | `AlertStatus` | — |
| `created_at` | `u64` | — |
| `updated_at` | `u64` | — |
| `resolution_notes` | `String` | — |

#### `struct AnomalyDetectionConfig`

| Field | Type | Description |
|---|---|---|
| `admin` | `Address` | — |
| `detector` | `Address` | — |
| `threshold_bps` | `u32` | — |
| `sensitivity` | `u32` | — |
| `enabled` | `bool` | — |

#### `struct AnomalyRecord`

| Field | Type | Description |
|---|---|---|
| `record_id` | `u64` | — |
| `patient` | `Address` | — |
| `detector_address` | `Address` | — |
| `score_bps` | `u32` | — |
| `severity` | `u32` | — |
| `detected_at` | `u64` | — |
| `metadata` | `String` | — |
| `explanation_ref` | `String` | — |

#### `struct DetectionStats`

| Field | Type | Description |
|---|---|---|
| `total_anomalies` | `u64` | — |
| `high_severity_count` | `u64` | — |
| `last_detection_at` | `u64` | — |

#### `enum DataKey`

| Variant | Value | Description |
|---|---|---|
| `Config` | — | — |
| `AnomalyRecord(u64)` | — | — |
| `AnomalyCountByPatient(Address)` | — | — |
| `Stats` | — | — |
| `Whitelist(Address)` | — | — |
| `Alert(u64)` | — | — |
| `AlertCount` | — | — |
| `FeedbackCount` | — | — |
| `AuditForensicsContract` | — | — |

#### `enum Error`

| Variant | Value | Description |
|---|---|---|
| `NotAuthorized` | 1 | — |
| `ConfigNotSet` | 2 | — |
| `Disabled` | 3 | — |
| `InvalidScore` | 4 | — |
| `InvalidSeverity` | 5 | — |
| `RecordNotFound` | 6 | — |
| `NotWhitelisted` | 7 | — |
| `AlertNotFound` | 8 | — |
| `AlertAlreadyResolved` | 9 | — |
| `AlreadyInitialized` | 10 | — |

---

## anomaly_detector

### Functions

| Function | Parameters | Returns | Description |
|---|---|---|---|
| `initialize` | `env: Env, admin: Address` | `Result<bool, Error>` | — |
| `add_validator` | `env: Env, caller: Address, validator: Address` | `Result<bool, Error>` | — |
| `remove_validator` | `env: Env, caller: Address, validator: Address` | `Result<bool, Error>` | — |
| `pause` | `env: Env, caller: Address` | `Result<bool, Error>` | — |
| `unpause` | `env: Env, caller: Address` | `Result<bool, Error>` | — |
| `update_threshold` | `env: Env, caller: Address, model_id: BytesN<32>, threshold_bps: u32` | `Result<bool, Error>` | Update the anomaly detection threshold for a model (admin only). `threshold_bps` must be in range 1–9999 (basis points). |
| `clear_alerts` | `env: Env, caller: Address, count: u64` | `Result<u64, Error>` | Clear active alerts up to `count` (admin only). Pass 0 to clear all. Marks each active alert as Resolved and emits a ClearAlerts event. |
| `register_model` | `env: Env, caller: Address, model_id: BytesN<32>, name: String, feature_count: u32, weights: Vec<u32>, threshold_bps: u32` | `Result<bool, Error>` | Register an ML model with its initial feature weights. `weights` must have exactly `feature_count` elements, each 0-10000 bps. |
| `update_model_weight` | `env: Env, caller: Address, model_id: BytesN<32>, feature_index: u32, delta: u32, increase: bool` | `Result<bool, Error>` | Adjust a single feature weight (used by adaptive learning pipeline). `increase = true` adds `delta`; `increase = false` subtracts. |
| `run_inference` | `env: Env, caller: Address, patient: Address, model_id: BytesN<32>, features: Vec<u32>, feature_names: Vec<String>, metadata: String` | `Result<DetectionResult, Error>` | Run on-chain ML inference over a feature vector. Score = weighted average of normalized features (0-10000 bps). Returns explainability-ready `DetectionResult`. |
| `detect_prescription_anomaly` | `env: Env, caller: Address, patient: Address, drug_count: u32, high_risk_count: u32, unique_pharmacies: u32, time_window_hours: u32, metadata: String` | `Result<DetectionResult, Error>` | Detect prescription anomaly patterns.  Scoring (weighted average, threshold = 5000 bps): - `high_risk_ratio` (40%): high_risk_count / drug_count - `drug_rate_score` (35%): prescriptions per hour, normalized - `pharmacy_dispersion` (25%): distinct pharmacy count, normalized |
| `detect_access_anomaly` | `env: Env, caller: Address, patient: Address, access_count: u32, time_window_secs: u32, is_after_hours: bool, distinct_record_types: u32, metadata: String` | `Result<DetectionResult, Error>` | Detect access behavior anomalies.  Scoring (threshold = 5000 bps): - `access_count` (45%): absolute access count (30+ → max score) - `after_hours` (35%): 8000 bps if is_after_hours, else 0 - `record_type_diversity` (20%): distinct record types accessed |
| `create_alert` | `env: Env, caller: Address, patient: Address, model_id: BytesN<32>, result: DetectionResult, metadata: String` | `Result<u64, Error>` | Create a real-time alert from a `DetectionResult`. Returns the new alert_id. |
| `create_alert_batch` | `env: Env, caller: Address, alerts: Vec<AlertInput>` | `Result<Vec<u64>, Error>` | Create multiple alerts in a single atomic call.  All-or-nothing semantics: if any alert fails, the entire batch is rejected and no alerts are persisted.  ## Limits - Max 50 alerts per batch. |
| `acknowledge_alert` | `env: Env, caller: Address, alert_id: u64` | `Result<bool, Error>` | Acknowledge an active alert (marks as reviewed, does not close). |
| `resolve_alert` | `env: Env, caller: Address, alert_id: u64, resolution_notes: String` | `Result<bool, Error>` | Resolve an alert after investigation. Accepted from Active or Acknowledged state. |
| `mark_false_positive` | `env: Env, caller: Address, alert_id: u64` | `Result<bool, Error>` | Mark an alert as false positive, automatically feeding adaptive learning. |
| `submit_feedback` | `env: Env, caller: Address, alert_id: u64, model_id: BytesN<32>, confirmed: bool` | `Result<u64, Error>` | Submit feedback confirming or refuting an alert.  - `confirmed = true`: real anomaly → lower model threshold by LEARNING_RATE (more sensitive) - `confirmed = false`: false positive → raise threshold by LEARNING_RATE (less noisy)  Learning rate: 50 bps (0.5%) per feedback signal. |
| `submit_federated_update` | `env: Env, participant: Address, round_id: u64, update_hash: BytesN<32>, num_samples: u32` | `Result<bool, Error>` | Submit a privacy-preserving model update for a federated learning round. The `update_hash` commits to gradient updates without exposing patient data. Duplicate submissions per (round_id, participant) are rejected. |
| `get_alert` | `env: Env, alert_id: u64` | `Option<Alert>` | — |
| `get_model` | `env: Env, model_id: BytesN<32>` | `Option<AnomalyModel>` | — |
| `get_model_weights` | `env: Env, model_id: BytesN<32>` | `Option<Vec<u32>>` | — |
| `get_patient_profile` | `env: Env, patient: Address` | `Option<PatientRiskProfile>` | — |
| `get_alert_count` | `env: Env` | `u64` | — |
| `get_feedback` | `env: Env, feedback_id: u64` | `Option<ModelFeedback>` | — |
| `get_federated_update` | `env: Env, round_id: u64, participant: Address` | `Option<FederatedUpdate>` | — |
| `is_paused` | `env: Env` | `bool` | — |
| `is_validator` | `env: Env, addr: Address` | `bool` | — |

### Types

#### `enum AlertLevel`

| Variant | Value | Description |
|---|---|---|
| `Low` | — | — |
| `Medium` | — | — |
| `High` | — | — |
| `Critical` | — | — |

#### `enum AlertStatus`

| Variant | Value | Description |
|---|---|---|
| `Active` | — | — |
| `Acknowledged` | — | — |
| `Resolved` | — | — |
| `FalsePositive` | — | — |

#### `enum HealthcarePatternType`

| Variant | Value | Description |
|---|---|---|
| `BulkRecordAccess` | — | — |
| `UnusualTimeAccess` | — | — |
| `PrescriptionAnomaly` | — | — |
| `UnauthorizedSpecialtyAccess` | — | — |
| `RapidSequentialAccess` | — | — |
| `SuspiciousExport` | — | — |
| `MlScored` | — | — |

#### `struct FeatureContribution`

| Field | Type | Description |
|---|---|---|
| `feature_index` | `u32` | — |
| `feature_name` | `String` | — |
| `feature_value` | `u32` | — |
| `weight` | `u32` | — |
| `contribution` | `u32` | — |

#### `struct DetectionResult`

| Field | Type | Description |
|---|---|---|
| `anomaly_score` | `u32` | — |
| `is_anomalous` | `bool` | — |
| `confidence` | `u32` | — |
| `alert_level` | `AlertLevel` | — |
| `pattern_type` | `HealthcarePatternType` | — |
| `top_features` | `Vec<FeatureContribution>` | — |
| `explanation_summary` | `String` | — |
| `detected_at` | `u64` | — |

#### `struct AnomalyModel`

| Field | Type | Description |
|---|---|---|
| `model_id` | `BytesN<32>` | — |
| `name` | `String` | — |
| `feature_count` | `u32` | — |
| `threshold_bps` | `u32` | — |
| `version` | `u32` | — |
| `total_inferences` | `u64` | — |
| `confirmed_anomalies` | `u64` | — |
| `false_positives` | `u64` | — |
| `created_at` | `u64` | — |
| `updated_at` | `u64` | — |

#### `struct AlertInput`

| Field | Type | Description |
|---|---|---|
| `patient` | `Address` | — |
| `model_id` | `BytesN<32>` | — |
| `result` | `DetectionResult` | — |
| `metadata` | `String` | — |

#### `struct Alert`

| Field | Type | Description |
|---|---|---|
| `alert_id` | `u64` | — |
| `patient` | `Address` | — |
| `triggered_by` | `Address` | — |
| `model_id` | `BytesN<32>` | — |
| `anomaly_score` | `u32` | — |
| `alert_level` | `AlertLevel` | — |
| `status` | `AlertStatus` | — |
| `pattern_type` | `HealthcarePatternType` | — |
| `explanation_summary` | `String` | — |
| `metadata` | `String` | — |
| `created_at` | `u64` | — |
| `updated_at` | `u64` | — |

#### `struct ModelFeedback`

| Field | Type | Description |
|---|---|---|
| `feedback_id` | `u64` | — |
| `alert_id` | `u64` | — |
| `model_id` | `BytesN<32>` | — |
| `submitted_by` | `Address` | — |
| `confirmed` | `bool` | — |
| `submitted_at` | `u64` | — |

#### `struct FederatedUpdate`

| Field | Type | Description |
|---|---|---|
| `round_id` | `u64` | — |
| `participant` | `Address` | — |
| `update_hash` | `BytesN<32>` | — |
| `num_samples` | `u32` | — |
| `submitted_at` | `u64` | — |

#### `struct PatientRiskProfile`

| Field | Type | Description |
|---|---|---|
| `patient` | `Address` | — |
| `rolling_risk_score` | `u32` | — |
| `total_alerts` | `u64` | — |
| `active_alerts` | `u64` | — |
| `false_positive_count` | `u64` | — |
| `last_alert_at` | `u64` | — |

#### `enum DataKey`

| Variant | Value | Description |
|---|---|---|
| `Admin` | — | — |
| `Paused` | — | — |
| `AlertCount` | — | — |
| `FeedbackCount` | — | — |
| `ModelWeights(BytesN<32>)` | — | — |
| `Model(BytesN<32>)` | — | — |
| `Alert(u64)` | — | — |
| `Feedback(u64)` | — | — |
| `FederatedUpdate(u64, Address)` | — | — |
| `PatientProfile(Address)` | — | — |
| `Validator(Address)` | — | — |

#### `enum Error`

| Variant | Value | Description |
|---|---|---|
| `AlreadyInitialized` | 1 | — |
| `NotInitialized` | 2 | — |
| `NotAuthorized` | 3 | — |
| `ContractPaused` | 4 | — |
| `ModelNotFound` | 5 | — |
| `AlertNotFound` | 6 | — |
| `FeatureCountMismatch` | 7 | — |
| `InvalidWeight` | 8 | — |
| `InvalidThreshold` | 9 | — |
| `AlertAlreadyResolved` | 10 | — |
| `DuplicateFederatedUpdate` | 11 | — |
| `InvalidFeatureCount` | 12 | — |
| `InvalidScore` | 13 | — |
| `BatchTooLarge` | 14 | — |

### Examples

#### `test_initialize`

```rust
let env = Env::default();
    let (client, _admin) = setup(&env);
    assert!(!client.is_paused());
    assert_eq!(client.get_alert_count(), 0);
```

#### `test_initialize_twice_fails`

```rust
let env = Env::default();
    let (client, admin) = setup(&env);
    env.mock_all_auths();
    let result = client.try_initialize(&admin);
    assert_eq!(result, Err(Ok(Error::AlreadyInitialized)));
```

#### `test_add_and_remove_validator`

```rust
let env = Env::default();
    let (client, admin) = setup(&env);
    let validator = Address::generate(&env);

    env.mock_all_auths();
    assert!(!client.is_validator(&validator));

    client.add_validator(&admin, &validator);
    assert!(client.is_validator(&validator));
```

---

## appointment_booking_escrow

### Functions

| Function | Parameters | Returns | Description |
|---|---|---|---|
| `initialize` | `env: Env, admin: Address, _token: Address` | `Result<(), Error>` | Initialize the contract with an admin and token address |
| `book_appointment` | `env: Env, patient: Address, provider: Address, amount: i128, token: Address` | `Result<u64, Error>` | Book an appointment with payment locked in escrow Transfers `amount` from patient to contract and creates an appointment escrow |
| `confirm_appointment` | `env: Env, provider: Address, appointment_id: u64` | `Result<(), Error>` | Confirm appointment completion and release funds to provider Only the provider can confirm the appointment |
| `refund_appointment` | `env: Env, patient: Address, appointment_id: u64` | `Result<(), Error>` | Refund appointment if canceled Only the patient can request a refund Can only be done if appointment is still in Booked state (not Confirmed/Refunded) |
| `mark_no_show` | `env: Env, provider: Address, appointment_id: u64` | `Result<(), Error>` | Mark an appointment as a no-show (provider only). Only callable by the appointment's provider. No funds are released. |
| `send_reminder` | `env: Env, caller: Address, appointment_id: u64` | `Result<(), Error>` | Send an appointment reminder (provider or admin only). Records the timestamp when the reminder was last sent. |
| `get_appointment` | `env: Env, appointment_id: u64` | `Option<AppointmentEscrow>` | Get appointment details |
| `get_patient_appointments` | `env: Env, patient: Address` | `Vec<u64>` | Get all appointments for a patient |
| `get_provider_appointments` | `env: Env, provider: Address` | `Vec<u64>` | Get all appointments for a provider |
| `get_appointment_status` | `env: Env, appointment_id: u64` | `Result<AppointmentStatus, Error>` | Get appointment status |
| `get_escrow_balance` | `env: Env` | `i128` | Get escrow balance (should be equal to sum of all booked but not confirmed/refunded appointments) |
| `get_admin` | `env: Env` | `Result<Address, Error>` | Get the current admin |
| `health_check` | `env: Env` | `ContractHealth` | Get comprehensive health check |
| `set_paused` | `env: Env, admin: Address, paused: bool` | `Result<(), Error>` | Set pause status (admin only) |
| `is_paused` | `env: Env` | `bool` | Check if contract is paused |

### Types

#### `enum AppointmentStatus`

| Variant | Value | Description |
|---|---|---|
| `Booked` | 0 | — |
| `Confirmed` | 1 | — |
| `Refunded` | 2 | — |
| `Completed` | 3 | — |
| `NoShow` | 4 | — |

#### `struct AppointmentEscrow`

| Field | Type | Description |
|---|---|---|
| `appointment_id` | `u64` | — |
| `patient` | `Address` | — |
| `provider` | `Address` | — |
| `amount` | `i128` | — |
| `token` | `Address` | — |
| `booked_at` | `u64` | — |
| `scheduled_time` | `u64` | — |
| `confirmed_at` | `u64` | — |
| `refunded_at` | `u64` | — |
| `reminder_sent_at` | `u64` | — |
| `no_show_marked_at` | `u64` | — |
| `status` | `AppointmentStatus` | — |
| `funds_released` | `bool` | — |

#### `enum DataKey`

| Variant | Value | Description |
|---|---|---|
| `Initialized` | — | — |
| `Admin` | — | — |
| `AppointmentCounter` | — | — |
| `Appointment(u64)` | — | — |
| `PatientAppointments(Address),  
    ProviderAppointments(Address), 
    Paused,
    LastActivity,
    TotalOperations,
    FailedOperations,
    Version,` | — | — |

#### `struct ContractHealth`

| Field | Type | Description |
|---|---|---|
| `version` | `String` | — |
| `is_paused` | `bool` | — |
| `storage_usage` | `u64` | — |
| `last_activity` | `u64` | — |
| `total_operations` | `u64` | — |
| `failed_operations` | `u64` | — |
| `success_rate` | `u32` | — |
| `total_appointments` | `u64` | — |
| `active_escrow_balance` | `i128` | — |

### Error Codes

| Variant | Code | Description |
|---|---|---|
| `Unauthorized` | 100 | — |
| `OnlyPatientCanRefund` | 110 | — |
| `OnlyProviderCanConfirm` | 111 | — |
| `InvalidAmount` | 205 | — |
| `InvalidPatient` | 210 | — |
| `InvalidProvider` | 211 | — |
| `NotInitialized` | 300 | — |
| `AlreadyInitialized` | 301 | — |
| `InvalidState` | 304 | — |
| `AppointmentNotFound` | 410 | — |
| `AppointmentAlreadyConfirmed` | 411 | — |
| `AppointmentAlreadyRefunded` | 412 | — |
| `AppointmentNoShow` | 413 | — |
| `InsufficientFunds` | 500 | — |
| `TokenTransferFailed` | 501 | — |
| `DoubleWithdrawal` | 505 | — |

### Examples

#### `test_initialize`

```rust
let (_env, client, admin, token_id) = setup();
        client.initialize(&admin, &token_id);
    }

    #[test]
    fn test_initialize_twice_fails() {
        let (_env, client, admin, token_id) = setup();
        client.initialize(&admin, &token_id);
        let result = client.try_initialize(&admin, &token_id);
```

---

## audit

### Functions

| Function | Parameters | Returns | Description |
|---|---|---|---|
| `initialize` | `env: Env, admin: Address, config: AuditConfig` | `()` | Initialize the contract with an admin address and audit configuration. |
| `log_event` | `env: Env, actor: Address, action: ActionType, target: BytesN<32>, result: OperationResult, metadata: Map<String, String>` | `u64` | Record a structured AuditLog entry. |
| `log_data_access` | `env: Env, actor: Address, target: BytesN<32>, result: OperationResult, metadata: Map<String, String>` | `u64` | Convenience: log a data access event. |
| `log_permission_change` | `env: Env, actor: Address, action: ActionType, target: BytesN<32>, result: OperationResult, metadata: Map<String, String>` | `u64` | Convenience: log a permission change. |
| `log_auth_attempt` | `env: Env, actor: Address, action: ActionType, target: BytesN<32>, result: OperationResult, metadata: Map<String, String>` | `u64` | Convenience: log an authentication attempt. |
| `log_cross_chain_transfer` | `env: Env, actor: Address, action: ActionType, target: BytesN<32>, result: OperationResult, metadata: Map<String, String>` | `u64` | Convenience: log a cross-chain transfer event. |
| `get_log` | `env: Env, id: u64` | `AuditLog` | Fetch a single AuditLog by ID. |
| `get_logs_by_actor` | `env: Env, caller: Address, actor: Address` | `Vec<AuditLog>` | Fetch all logs for a given actor (requires admin or granted access). |
| `get_logs_by_action` | `env: Env, caller: Address, action: ActionType` | `Vec<AuditLog>` | Fetch all logs for a given ActionType (requires log access). |
| `get_logs_by_timeframe` | `env: Env, caller: Address, start: u64, end: u64` | `Vec<AuditLog>` | Fetch logs within a timestamp range (requires log access). |
| `grant_log_access` | `env: Env, admin: Address, reader: Address` | `Result<(), Error>` | Grant log-read access to an address (admin only). |
| `revoke_log_access` | `env: Env, admin: Address, reader: Address` | `Result<(), Error>` | Revoke log-read access (admin only). |
| `has_log_access` | `env: Env, reader: Address` | `bool` | Check whether an address has log-read access. |
| `set_retention_policy` | `env: Env, admin: Address, policy: RetentionPolicy` | `Result<(), Error>` | Update the retention policy (admin only). |
| `get_retention_policy` | `env: Env` | `RetentionPolicy` | Read the current retention policy. |
| `verify_retention` | `env: Env, log_id: u64` | `bool` | Verify that a log entry satisfies the retention policy. Returns true if the log is within the required retention window. |
| `export_logs` | `env: Env, caller: Address, start_id: u64, end_id: u64` | `ExportBundle` | Export a range of AuditLog entries as a signed bundle (requires log access). The bundle includes an integrity hash over all exported entries. |
| `get_log_rolling_hash` | `env: Env` | `BytesN<32>` | Returns the stored rolling hash of the AuditLog chain. |
| `verify_log_integrity` | `env: Env` | `BytesN<32>` | Recomputes the rolling hash from scratch and returns it. Compare with `get_log_rolling_hash` to detect tampering. |
| `is_log_tampered` | `env: Env, expected: BytesN<32>` | `bool` | Returns true if the AuditLog chain has been tampered with. |
| `verify_integrity` | `env: Env` | `BytesN<32>` | Returns the stored rolling hash (legacy alias kept for compatibility). |
| `generate_summary` | `env: Env, start: u64, end: u64` | `AuditSummary` | Compliance analytics summary over AuditLog entries. |

### Error Codes

| Variant | Code | Description |
|---|---|---|
| `Unauthorized` | 100 | — |
| `NotInitialized` | 300 | — |
| `AlreadyInitialized` | 301 | — |
| `RecordNotFound` | 403 | — |

### Examples

#### `test_log_event_data_access`

```rust
let env = Env::default();
    let (client, _admin) = setup(&env);

    let actor = Address::generate(&env);
    let target = dummy_target(&env);

    let id = client.log_event(
        &actor,
        &ActionType::DataRead,
```

#### `test_log_event_permission_change`

```rust
let env = Env::default();
    let (client, _admin) = setup(&env);

    let actor = Address::generate(&env);
    let id = client.log_event(
        &actor,
        &ActionType::PermissionGrant,
        &dummy_target(&env),
        &OperationResult::Success,
```

#### `test_log_event_record_modification`

```rust
let env = Env::default();
    let (client, _admin) = setup(&env);

    let actor = Address::generate(&env);
    let id = client.log_event(
        &actor,
        &ActionType::RecordUpdate,
        &dummy_target(&env),
        &OperationResult::Success,
```

---

## audit_forensics

### Functions

| Function | Parameters | Returns | Description |
|---|---|---|---|
| `initialize` | `env: Env, admin: Address` | `()` | — |
| `configure_audit_rule` | `env: Env, admin: Address, name: String, applies_to_language: String, severity_bps: u32, pattern_ref: String, remediation: String` | `u64` | — |
| `log_event` | `env: Env, actor: Address, action: AuditAction, record_id: Option<u64>, details_hash: BytesN<32>, metadata: Map<String, String>` | `u64` | — |
| `run_automated_audit` | `env: Env, caller: Address, contract_hash: BytesN<32>, language: String, analysis_mode: String, rule_ids: Vec<u64>, ml_confidence_bps: u32` | `u64` | — |
| `record_formal_verification` | `env: Env, admin: Address, execution_id: u64, property_name: String, proved: bool, proof_ref: String` | `bool` | — |
| `get_execution` | `env: Env, execution_id: u64` | `Option<AnalysisExecution>` | — |
| `get_finding` | `env: Env, finding_id: u64` | `Option<VulnerabilityFinding>` | — |
| `get_findings_by_execution` | `env: Env, execution_id: u64` | `Vec<VulnerabilityFinding>` | — |
| `get_formal_verification` | `env: Env, execution_id: u64` | `Option<FormalVerificationSummary>` | — |
| `generate_remediation_plan` | `env: Env, execution_id: u64` | `Vec<String>` | — |
| `analyze_timeline` | `env: Env, record_id: u64` | `Vec<AuditEntry>` | — |
| `investigate_user` | `env: Env, user: Address` | `Vec<AuditEntry>` | — |
| `generate_compliance_report` | `env: Env, start_time: u64, end_time: u64` | `Map<AuditAction, u32>` | — |
| `set_alert_threshold` | `env: Env, admin: Address, action: AuditAction, threshold: u32` | `()` | — |
| `compress_logs` | `env: Env, admin: Address, before_timestamp: u64` | `BytesN<32>` | — |
| `archive_logs` | `env: Env, admin: Address, archive_ref: String` | `()` | — |
| `sync_audit_cross_chain` | `env: Env, admin: Address, target_chain: String, audit_root: BytesN<32>` | `()` | — |
| `share_audit_with_regulator` | `env: Env, admin: Address, regulator: Address, filter_start: u64, filter_end: u64, proof_ref: String` | `()` | — |

### Types

#### `enum AuditAction`

| Variant | Value | Description |
|---|---|---|
| `RecordAccess` | — | — |
| `RecordUpdate` | — | — |
| `RecordDelete` | — | — |
| `PermissionGrant` | — | — |
| `PermissionRevoke` | — | — |
| `RecordCreated` | — | — |
| `AnomalyDetected` | — | — |
| `ComplianceReportGenerated` | — | — |
| `AlertTriggered` | — | — |

#### `struct AuditEntry`

| Field | Type | Description |
|---|---|---|
| `id` | `u64` | — |
| `timestamp` | `u64` | — |
| `actor` | `Address` | — |
| `action` | `AuditAction` | — |
| `record_id` | `Option<u64>` | — |
| `details_hash` | `BytesN<32>` | — |
| `metadata` | `Map<String, String>` | — |

#### `struct ForensicReport`

| Field | Type | Description |
|---|---|---|
| `target_id` | `u64` | — |
| `entries` | `Vec<AuditEntry>` | — |
| `summary` | `String` | — |
| `detected_anomalies` | `Vec<u64>` | — |

#### `struct AuditRule`

| Field | Type | Description |
|---|---|---|
| `id` | `u64` | — |
| `name` | `String` | — |
| `applies_to_language` | `String` | — |
| `severity_bps` | `u32` | — |
| `enabled` | `bool` | — |
| `pattern_ref` | `String` | — |
| `remediation` | `String` | — |

#### `struct VulnerabilityFinding`

| Field | Type | Description |
|---|---|---|
| `id` | `u64` | — |
| `execution_id` | `u64` | — |
| `rule_id` | `u64` | — |
| `contract_hash` | `BytesN<32>` | — |
| `title` | `String` | — |
| `severity_bps` | `u32` | — |
| `confidence_bps` | `u32` | — |
| `language` | `String` | — |
| `analysis_mode` | `String` | — |
| `remediation` | `String` | — |
| `detected_at` | `u64` | — |

#### `struct AnalysisExecution`

| Field | Type | Description |
|---|---|---|
| `id` | `u64` | — |
| `contract_hash` | `BytesN<32>` | — |
| `language` | `String` | — |
| `analysis_mode` | `String` | — |
| `finding_ids` | `Vec<u64>` | — |
| `started_at` | `u64` | — |
| `completed_at` | `u64` | — |
| `duration_minutes` | `u32` | — |
| `passed` | `bool` | — |

#### `struct FormalVerificationSummary`

| Field | Type | Description |
|---|---|---|
| `execution_id` | `u64` | — |
| `property_name` | `String` | — |
| `proved` | `bool` | — |
| `proof_ref` | `String` | — |
| `checked_at` | `u64` | — |

#### `enum DataKey`

| Variant | Value | Description |
|---|---|---|
| `Admin` | — | — |
| `NextAuditId` | — | — |
| `AuditEntry(u64)` | — | — |
| `UserAudits(Address)` | — | — |
| `RecordAudits(u64)` | — | — |
| `AlertThresholds(Symbol)` | — | — |
| `NextRuleId` | — | — |
| `Rule(u64)` | — | — |
| `NextExecutionId` | — | — |
| `Execution(u64)` | — | — |
| `NextFindingId` | — | — |
| `Finding(u64)` | — | — |
| `FindingsByExecution(u64)` | — | — |
| `FormalSummary(u64)` | — | — |

### Examples

#### `test_audit_flow`

```rust
let env = Env::default();
        let contract_id = env.register_contract(None, AuditForensicsContract);
        let client = AuditForensicsContractClient::new(&env, &contract_id);

        let admin = Address::generate(&env);
        client.initialize(&admin);

        let doctor = Address::generate(&env);
        let record_id = 101u64;
```

---

## clinical_decision_support

### Functions

| Function | Parameters | Returns | Description |
|---|---|---|---|
| `initialize` | `env: Env, admin: Address, oracle: Address, medical_records: Address` | `()` | Initialize the CDSS contract with necessary integration addresses. |
| `check_drug_interactions` | `env: Env, patient_id: String, new_medication_code: String, current_medications: Vec<String>` | `Vec<Recommendation>` | Checks for drug-drug interactions in real-time. Returns a list of alerts if interactions are found. |
| `get_treatment_recommendation` | `env: Env, patient_id: String, condition_codes: Vec<String>` | `Vec<Recommendation>` | Provides personalized treatment recommendations based on patient conditions and guidelines. |
| `optimize_pathway` | `env: Env, patient_id: String, current_pathway_step: u32, vitals_trend: i32, // -1: Declining, 0: Stable, 1: Improving` | `Recommendation` | Optimizes clinical pathways by suggesting adjustments based on real-time data. |
| `record_outcome` | `env: Env, condition_code: String, was_successful: bool` | `()` | Records clinical outcomes to enable continuous learning for the CDSS AI. |
| `update_guideline` | `env: Env, oracle: Address, guideline: ClinicalGuideline` | `()` | Administrative function to update medical guidelines from the Oracle. |
| `set_interaction` | `env: Env, admin: Address, drug_a: String, drug_b: String, severity: String` | `()` | Administrative function to update the drug interaction database. |

### Types

#### `enum RecommendationType`

| Variant | Value | Description |
|---|---|---|
| `DrugInteraction` | 0 | — |
| `TreatmentOptimization` | 1 | — |
| `PathwayAdjustment` | 2 | — |
| `PreventativeCare` | 3 | — |

#### `struct FHIRCode`

| Field | Type | Description |
|---|---|---|
| `system` | `String` | — |
| `code` | `String` | — |
| `display` | `String` | — |

#### `struct Recommendation`

| Field | Type | Description |
|---|---|---|
| `rec_id` | `String` | — |
| `patient_id` | `String` | — |
| `rec_type` | `RecommendationType` | — |
| `content` | `String` | — |
| `confidence_score` | `u32` | — |
| `urgency` | `u32` | — |
| `1` | `High` | — |
| `2` | `Critical pub timestamp: u64` | — |

#### `struct ClinicalGuideline`

| Field | Type | Description |
|---|---|---|
| `condition_code` | `String` | — |
| `recommended_action` | `String` | — |
| `evidence_level` | `String` | — |
| `min_confidence` | `u32` | — |

#### `enum DataKey`

| Variant | Value | Description |
|---|---|---|
| `Admin` | — | — |
| `Oracle` | — | — |
| `MedicalRecordsContract` | — | — |
| `Guideline(String)` | — | — |
| `Interaction(Vec<String>)` | — | — |
| `Outcome(String)` | — | — |

---

## clinical_nlp

### Functions

| Function | Parameters | Returns | Description |
|---|---|---|---|
| `initialize` | `env: Env, admin: Address` | `Result<(), Error>` | — |
| `process_clinical_note` | `env: Env, note_text: String, note_id: BytesN<32>, patient_id: Address, record_id: BytesN<32>, language: u32` | `Result<NLPResult, Error>` | — |
| `extract_entities` | `env: Env, text: String` | `Result<Vec<ExtractedEntity>, Error>` | — |
| `analyze_sentiment` | `env: Env, text: String` | `Result<SentimentResult, Error>` | — |
| `generate_coding_suggestions` | `env: Env, text: String, max_suggestions: u32` | `Result<Vec<icd_cpt_codes::CodingSuggestion>, Error>` | — |
| `get_processing_stats` | `env: Env` | `Result<ProcessingStats, Error>` | — |
| `update_config` | `env: Env, admin: Address, config: NLPConfig` | `Result<(), Error>` | — |
| `process_batch` | `env: Env, request: BatchProcessingRequest` | `Result<BatchProcessingResult, Error>` | — |
| `version` | `_env: Env` | `u32` | — |
| `is_initialized` | `env: Env` | `bool` | — |

### Types

#### `struct ProcessingStats`

| Field | Type | Description |
|---|---|---|
| `total_notes_processed` | `u64` | — |
| `total_processing_time_ms` | `u64` | — |
| `average_accuracy_bps` | `u32` | — |
| `entities_extracted` | `u64` | — |
| `concepts_extracted` | `u64` | — |
| `coding_suggestions_generated` | `u64` | — |
| `phi_detections` | `u64` | — |
| `last_updated` | `u64` | — |

#### `struct BatchProcessingRequest`

| Field | Type | Description |
|---|---|---|
| `batch_id` | `BytesN<32>` | — |
| `notes` | `Vec<String>` | — |
| `patient_ids` | `Vec<Address>` | — |
| `record_ids` | `Vec<BytesN<32>>` | — |
| `language` | `u32` | — |

#### `struct BatchProcessingResult`

| Field | Type | Description |
|---|---|---|
| `batch_id` | `BytesN<32>` | — |
| `results` | `Vec<NLPResult>` | — |
| `total_processing_time_ms` | `u64` | — |
| `success_count` | `u32` | — |
| `failure_count` | `u32` | — |
| `average_accuracy_bps` | `u32` | — |

### Error Codes

| Variant | Code | Description |
|---|---|---|
| `Unauthorized` | 100 | — |
| `InsufficientPermissions` | 101 | — |
| `HIPAAComplianceViolation` | 104 | — |
| `RecordAccessDenied` | 112 | — |
| `InputTooLong` | 201 | — |
| `BatchTooLarge` | 208 | — |
| `EmptyClinicalNote` | 209 | — |
| `InvalidLanguageCode` | 212 | — |
| `InvalidEncoding` | 213 | — |
| `NotInitialized` | 300 | — |
| `AlreadyInitialized` | 301 | — |
| `ContractPaused` | 302 | — |
| `RateLimitExceeded` | 307 | — |
| `Timeout` | 308 | — |
| `InvalidConfiguration` | 310 | — |
| `RecordNotFound` | 403 | — |
| `IntegrationFailed` | 704 | — |
| `ExternalContractNotSet` | 705 | — |
| `NLPEngineNotInitialized` | 800 | — |
| `EntityExtractionFailed` | 801 | — |
| `ConceptExtractionFailed` | 802 | — |
| `SentimentAnalysisFailed` | 803 | — |
| `CodingSuggestionFailed` | 804 | — |
| `TokenizationFailed` | 805 | — |
| `LanguageDetectionFailed` | 806 | — |
| `MedicalTermNotFound` | 807 | — |
| `InvalidMedicalTerm` | 808 | — |
| `TermDatabaseNotLoaded` | 809 | — |
| `ICD10CodeNotFound` | 810 | — |
| `CPTCodeNotFound` | 811 | — |
| `InvalidCodeFormat` | 812 | — |
| `CodeMappingFailed` | 813 | — |

### Examples

#### `test_initialize`

```rust
let env = Env::default();
    let contract_id = env.register_contract(None, ClinicalNLP);
    let client = ClinicalNLPClient::new(&env, &contract_id);

    let admin = Address::random(&env);

    // Initialize contract
    let result = client.initialize(&admin);
    assert!(result.is_ok());
```

#### `test_process_clinical_note`

```rust
let env = Env::default();
    let contract_id = env.register_contract(None, ClinicalNLP);
    let client = ClinicalNLPClient::new(&env, &contract_id);

    let admin = Address::random(&env);

    // Initialize contract
    client.initialize(&admin).unwrap();
```

#### `test_extract_entities`

```rust
let env = Env::default();
    let contract_id = env.register_contract(None, ClinicalNLP);
    let client = ClinicalNLPClient::new(&env, &contract_id);

    let admin = Address::random(&env);

    // Initialize contract
    client.initialize(&admin).unwrap();
```

---

## clinical_trial

### Functions

| Function | Parameters | Returns | Description |
|---|---|---|---|
| `initialize` | `env: Env, admin: Address` | `()` | — |
| `create_protocol` | `env: Env, proposer: Address, title: String, metadata_ref: String, max_participants: u64` | `Result<u64, Error>` | — |
| `get_protocol` | `env: Env, id: u64` | `Option<Protocol>` | — |
| `register_site` | `env: Env, registrar: Address, name: String` | `Result<u64, Error>` | — |
| `recruit_patient` | `env: Env, site: Address, patient: Address, protocol_id: u64` | `Result<(), Error>` | — |
| `record_consent` | `env: Env, patient: Address, protocol_id: u64, consent_ref: String` | `Result<u64, Error>` | — |
| `report_adverse_event` | `env: Env, reporter: Address, patient: Address, protocol_id: u64, site_id: u64, severity: u32, description_ref: String` | `Result<u64, Error>` | — |
| `get_trial_status` | `env: Env, protocol_id: u64` | `Result<(u64, u64, u64), Error>` | — |
| `enroll_participant` | `env: Env, site: Address, participant: Address, protocol_id: u64` | `Result<(), Error>` | Enroll a participant in a clinical trial.  Enforces the `max_participants` cap: if the trial is already at capacity this returns `Err(Error::TrialFull)`.  When the last available slot is filled a `TrialCapacityReached` event is emitted in addition to the standard `ParticipantEnrolled` event. |
| `has_consent` | `env: Env, patient: Address, protocol_id: u64` | `bool` | — |

### Types

#### `enum Error`

| Variant | Value | Description |
|---|---|---|
| `ProtocolNotFound` | 1 | — |
| `TrialFull` | 2 | — |
| `InvalidTitle` | 3 | — |
| `InvalidMetadataRef` | 4 | — |
| `InvalidName` | 5 | — |
| `InvalidConsentRef` | 6 | — |
| `InvalidMaxParticipants` | 7 | — |
| `InvalidDescriptionRef` | 8 | — |
| `InvalidSeverity` | 9 | — |

#### `struct Protocol`

| Field | Type | Description |
|---|---|---|
| `id` | `u64` | — |
| `title` | `String` | — |
| `version` | `u32` | — |
| `sponsor` | `Address` | — |
| `created_at` | `u64` | — |
| `active` | `bool` | — |
| `metadata_ref` | `String` | — |
| `max_participants` | `u64` | — |
| `current_participants` | `u64` | — |

#### `struct Site`

| Field | Type | Description |
|---|---|---|
| `id` | `u64` | — |
| `address` | `Address` | — |
| `name` | `String` | — |
| `active` | `bool` | — |

#### `struct Consent`

| Field | Type | Description |
|---|---|---|
| `patient` | `Address` | — |
| `protocol_id` | `u64` | — |
| `version` | `u32` | — |
| `timestamp` | `u64` | — |
| `consent_ref` | `String` | — |

#### `struct AdverseEvent`

| Field | Type | Description |
|---|---|---|
| `id` | `u64` | — |
| `patient` | `Address` | — |
| `protocol_id` | `u64` | — |
| `site_id` | `u64` | — |
| `description_ref` | `String` | — |
| `timestamp` | `u64` | — |
| `severity` | `u32` | — |

#### `enum DataKey`

| Variant | Value | Description |
|---|---|---|
| `Initialized` | — | — |
| `Protocol(u64)` | — | — |
| `ProtocolNextId` | — | — |
| `Site(u64)` | — | — |
| `SiteNextId` | — | — |
| `ConsentCount` | — | — |
| `Consent(u64)` | — | — |
| `AdverseEventNextId` | — | — |
| `AdverseEvent(u64)` | — | — |
| `ParticipantRecords(Address)` | — | — |
| `ProtocolEnrollmentCount(u64)` | — | — |

---

## code_ownership

### Functions

| Function | Parameters | Returns | Description |
|---|---|---|---|
| `initialize` | `env: Env, admin: Address` | `Result<(), Error>` | Initialize the code ownership tracking system |
| `register_module` | `env: Env, admin: Address, module_id: String, module_name: String, primary_owner: Address, secondary_owners: Vec<Address>, expertise_areas: Vec<String>` | `Result<(), Error>` | Register a module with ownership information |
| `update_module_ownership` | `env: Env, admin: Address, module_id: String, new_primary_owner: Address, new_secondary_owners: Vec<Address>` | `Result<(), Error>` | Update module ownership |
| `configure_review_route` | `env: Env, admin: Address, module_id: String, required_reviewers: u32, escalation_threshold: u32, escalation_owner: Address` | `Result<(), Error>` | Configure review routing for a module |
| `get_module_ownership` | `env: Env, module_id: String` | `Result<ModuleOwnership, Error>` | Get module ownership information |
| `get_review_route` | `env: Env, module_id: String` | `Result<ReviewRoute, Error>` | Get review routing for a module |
| `get_expertise_matrix` | `env: Env` | `OwnershipMatrix` | Get expertise matrix for all modules |
| `is_module_owner` | `env: Env, module_id: String, address: Address` | `Result<bool, Error>` | Check if an address is an owner of a module |
| `get_owned_modules` | `env: Env, owner: Address` | `Vec<String>` | Get all modules owned by an address |

### Error Codes

| Variant | Code | Description |
|---|---|---|
| `NotInitialized` | 1 | — |
| `AlreadyInitialized` | 2 | — |
| `NotAuthorized` | 3 | — |
| `ModuleNotFound` | 4 | — |
| `ModuleAlreadyExists` | 5 | — |
| `ReviewRouteNotFound` | 6 | — |
| `InvalidOwnerCount` | 7 | — |

### Examples

#### `test_initialize`

```rust
let env = Env::default();
        env.mock_all_auths();

        let admin = Address::generate(&env);
        let contract_id = env.register_contract(None, CodeOwnership);
        let client = CodeOwnershipClient::new(&env, &contract_id);

        client.initialize(&admin);
    }
```

---

## common_error

### Functions

| Function | Parameters | Returns | Description |
|---|---|---|---|
| `is_common_error_code` | `code: u32` | `bool` | — |
| `get_suggestion` | `error: CommonError` | `Symbol` | — |

### Types

#### `enum CommonError`

| Variant | Value | Description |
|---|---|---|
| `Unknown` | 0 | — |
| `Unauthorized` | 1 | — |
| `NotInitialized` | 2 | — |
| `AlreadyInitialized` | 3 | — |
| `ContractPaused` | 4 | — |
| `DeadlineExceeded` | 5 | — |
| `RateLimitExceeded` | 6 | — |
| `InsufficientFunds` | 7 | — |
| `InvalidInput` | 8 | — |
| `InvalidState` | 9 | — |
| `NotFound` | 10 | — |
| `AccessDenied` | 11 | — |
| `Timeout` | 12 | — |
| `InvalidArgument` | 13 | — |
| `ExternalContractNotSet` | 14 | — |
| `InvalidData` | 15 | — |
| `InvalidPayload` | 16 | — |
| `DuplicateSubmission` | 17 | — |
| `UnauthorizedCaller` | 18 | — |

---

## contract_behavior_fuzzing

### Functions

| Function | Parameters | Returns | Description |
|---|---|---|---|
| `execute_sequence` | `harness: &mut H, operations: &[H::Operation]` | `Result<SequenceReport<H::Operation>, CrashReport<H::Operation>> where H: BehaviorHarness,` | — |
| `run_regressions` | `cases: &[RegressionCase<H::Operation>], mut make_harness: F` | `()` | — |

### Types

#### `struct OperationOutcome`

| Field | Type | Description |
|---|---|---|
| `expected_event_delta` | `usize` | — |

---

## contract_monitoring

### Functions

| Function | Parameters | Returns | Description |
|---|---|---|---|
| `initialize` | `env: Env, admin: Address, alert_config: AlertConfig` | `Result<(), MonitoringError>` | Initialise the monitoring contract. |
| `record_call` | `env: Env, caller: Address, function_name: String, gas_used: u64` | `Result<(), MonitoringError>` | Record a successful function call.  `caller` – the address that invoked the function. `function_name` – name of the function called. `gas_used` – estimated gas consumed (pass 0 if unknown). |
| `record_error` | `env: Env, function_name: String` | `Result<(), MonitoringError>` | Record a failed function call / error. |
| `update_storage_count` | `env: Env, count: u32` | `Result<(), MonitoringError>` | Update storage-entry count (call after writes to tracked contracts). |
| `update_alert_config` | `env: Env, config: AlertConfig` | `Result<(), MonitoringError>` | Update alert thresholds (admin only). |
| `get_dashboard` | `env: Env` | `Result<DashboardSnapshot, MonitoringError>` | Return a full dashboard snapshot. |
| `get_function_stats` | `env: Env, function_name: String` | `Result<FunctionStats, MonitoringError>` | Return per-function statistics. |

### Types

#### `struct AlertConfig`

| Field | Type | Description |
|---|---|---|
| `max_error_rate_pct` | `u32` | — |
| `max_gas_per_window` | `u64` | — |
| `max_storage_entries` | `u32` | — |

#### `struct FunctionStats`

| Field | Type | Description |
|---|---|---|
| `call_count` | `u64` | — |
| `error_count` | `u64` | — |
| `total_gas` | `u64` | — |
| `last_called_at` | `u64` | — |

#### `struct DashboardSnapshot`

| Field | Type | Description |
|---|---|---|
| `total_calls` | `u64` | — |
| `total_errors` | `u64` | — |
| `error_rate_pct` | `u32` | — |
| `total_gas_used` | `u64` | — |
| `active_users` | `u32` | — |
| `storage_entries` | `u32` | — |
| `snapshot_at` | `u64` | — |
| `alert_active` | `bool` | — |

#### `enum DataKey`

| Variant | Value | Description |
|---|---|---|
| `Admin` | — | — |
| `AlertConfig` | — | — |
| `TotalCalls` | — | — |
| `TotalErrors` | — | — |
| `TotalGas` | — | — |
| `ActiveUsers` | — | — |
| `StorageEntries` | — | — |
| `FnStats(String)` | — | — |
| `SeenUser(Address)` | — | — |

#### `enum MonitoringError`

| Variant | Value | Description |
|---|---|---|
| `NotInitialized` | 1 | — |
| `AlreadyInitialized` | 2 | — |
| `Unauthorized` | 3 | — |

---

## contract_template

### Functions

| Function | Parameters | Returns | Description |
|---|---|---|---|
| `initialize` | `env: Env, admin: Address` | `Result<(), Error>` | Initialize the contract. Can only be called once.  # Auth No auth required — the deployer becomes the admin. |
| `transfer_admin` | `env: Env, new_admin: Address` | `Result<(), Error>` | Transfer admin rights to a new address.  # Auth Requires auth from the **current** admin. |
| `update_data` | `env: Env, caller: Address, data: String` | `Result<(), Error>` | Update the contract's stored data.  # Auth Requires auth from the admin. |
| `get_admin` | `env: &Env` | `Result<Address, Error>` | Return the current admin address. |
| `get_data` | `env: Env` | `Option<ContractData>` | Return the stored data, if any. |

### Error Codes

| Variant | Code | Description |
|---|---|---|
| `NotInitialized` | 1 | — |
| `AlreadyInitialized` | 2 | — |
| `Unauthorized` | 3 | — |
| `InputTooLong` | 4 | — |

### Examples

#### `test_initialize`

```rust
let (_, _, client) = setup();
    let admin2 = Address::generate(&client.env);
    assert_eq!(
        client.try_initialize(&admin2),
        Err(Ok(Error::AlreadyInitialized))
    );
```

#### `test_update_data_as_admin`

```rust
let (env, admin, client) = setup();
    let data = String::from_str(&env, "hello");
    assert!(client.try_update_data(&admin, &data).is_ok());
    let stored = client.get_data().unwrap();
    assert_eq!(stored.value, data);
```

#### `test_update_data_unauthorized`

```rust
let (env, _, client) = setup();
    let other = Address::generate(&env);
    let data = String::from_str(&env, "hack");
    assert_eq!(
        client.try_update_data(&other, &data),
        Err(Ok(Error::Unauthorized))
    );
```

---

## contract_usage_analytics

### Functions

| Function | Parameters | Returns | Description |
|---|---|---|---|
| `initialize` | `env: Env, admin: Address` | `Result<(), Error>` | — |
| `record_call` | `env: Env, function_name: String, user: Address, cpu_usage: u64, ram_usage: u64, success: bool, latency_ms: u64` | `Result<(), Error>` | — |
| `take_snapshot` | `env: Env` | `Result<UsageSnapshot, Error>` | — |
| `get_function_metrics` | `env: Env, function_name: String` | `Option<FunctionMetric>` | — |
| `get_user_metrics` | `env: Env, user: Address` | `Option<UserMetric>` | — |
| `get_all_functions` | `env: Env` | `Vec<String>` | — |
| `get_snapshots` | `env: Env` | `Vec<UsageSnapshot>` | — |

### Types

#### `struct FunctionMetric`

| Field | Type | Description |
|---|---|---|
| `name` | `String` | — |
| `call_count` | `u64` | — |
| `total_cpu_usage` | `u64` | — |
| `total_ram_usage` | `u64` | — |
| `error_count` | `u64` | — |
| `avg_latency_ms` | `u64` | — |
| `last_called` | `u64` | — |

#### `struct UserMetric`

| Field | Type | Description |
|---|---|---|
| `user` | `Address` | — |
| `total_calls` | `u64` | — |
| `last_active` | `u64` | — |

#### `struct UsageSnapshot`

| Field | Type | Description |
|---|---|---|
| `timestamp` | `u64` | — |
| `total_calls` | `u64` | — |
| `active_users` | `u32` | — |
| `error_rate_bps` | `u32` | — |

#### `enum DataKey`

| Variant | Value | Description |
|---|---|---|
| `Admin` | — | — |
| `FunctionMetric(String)` | — | — |
| `UserMetric(Address)` | — | — |
| `Snapshots` | — | — |
| `AllFunctions` | — | — |
| `ActiveUsers(u64)` | — | — |

#### `enum Error`

| Variant | Value | Description |
|---|---|---|
| `NotAuthorized` | 1 | — |
| `AlreadyInitialized` | 2 | — |
| `NotInitialized` | 3 | — |
| `InvalidInput` | 4 | — |

---

## contract_verification

### Functions

| Function | Parameters | Returns | Description |
|---|---|---|---|
| `initialize` | `env: Env, admin: Address` | `Result<(), VerificationError>` | Initialise the verification registry with an admin address. |
| `publish_metadata` | `env: Env, name: String, version: String, source_url: String, license: String, description: String` | `Result<(), VerificationError>` | Publish contract metadata.  Must be called by the admin.  Emits a `(VERIFY, META)` event that block explorers can index. |
| `publish_build_info` | `env: Env, rust_version: String, sdk_version: String, build_profile: String, wasm_hash: BytesN<32>, commit_sha: String` | `Result<(), VerificationError>` | Publish build reproducibility information. |
| `publish_abi` | `env: Env, entries: Vec<AbiEntry>` | `Result<(), VerificationError>` | Publish the ABI for all public functions. |
| `mark_verified` | `env: Env` | `Result<(), VerificationError>` | Mark the contract as fully verified (metadata + build + ABI all present). |
| `get_metadata` | `env: Env` | `Result<ContractMetadata, VerificationError>` | — |
| `get_build_info` | `env: Env` | `Result<BuildInfo, VerificationError>` | — |
| `get_abi` | `env: Env` | `Result<Vec<AbiEntry>, VerificationError>` | — |
| `is_verified` | `env: Env` | `bool` | — |

### Types

#### `struct ContractMetadata`

| Field | Type | Description |
|---|---|---|
| `name` | `String` | — |
| `version` | `String` | — |
| `source_url` | `String` | — |
| `license` | `String` | — |
| `description` | `String` | — |
| `published_at` | `u64` | — |
| `publisher` | `Address` | — |

#### `struct BuildInfo`

| Field | Type | Description |
|---|---|---|
| `rust_version` | `String` | — |
| `sdk_version` | `String` | — |
| `build_profile` | `String` | — |
| `wasm_hash` | `BytesN<32>` | — |
| `commit_sha` | `String` | — |

#### `struct AbiEntry`

| Field | Type | Description |
|---|---|---|
| `name` | `String` | — |
| `params` | `String` | — |
| `returns` | `String` | — |
| `doc` | `String` | — |

#### `enum DataKey`

| Variant | Value | Description |
|---|---|---|
| `Admin` | — | — |
| `Metadata` | — | — |
| `BuildInfo` | — | — |
| `AbiEntries` | — | — |
| `IsVerified` | — | — |

#### `enum VerificationError`

| Variant | Value | Description |
|---|---|---|
| `NotInitialized` | 1 | — |
| `AlreadyInitialized` | 2 | — |
| `Unauthorized` | 3 | — |
| `MetadataNotFound` | 4 | — |

---

## credential_notifications

### Functions

| Function | Parameters | Returns | Description |
|---|---|---|---|
| `initialize` | `env: Env, admin: Address` | `Result<(), Error>` | Initialize the contract with an admin address. Can only be called once. |
| `add_notifier` | `env: Env, caller: Address, notifier: Address` | `Result<(), Error>` | Grant notification permission to an address. Admin only. |
| `remove_notifier` | `env: Env, caller: Address, notifier: Address` | `Result<(), Error>` | Revoke notification permission from an address. Admin only. |
| `send_notification` | `env: Env, caller: Address, recipient: Address, credential_id: String, message: String` | `Result<(), Error>` | Send a credential notification. Only authorized notifiers or admin may call this. |
| `is_notifier` | `env: Env, notifier: Address` | `bool` | Check whether an address is an authorized notifier. |
| `get_admin` | `env: Env` | `Result<Address, Error>` | Return the current admin address. |

### Types

#### `enum DataKey`

| Variant | Value | Description |
|---|---|---|
| `Notifier(Address)` | — | — |

#### `enum Error`

| Variant | Value | Description |
|---|---|---|
| `AlreadyInitialized` | 1 | — |
| `NotInitialized` | 2 | — |
| `Unauthorized` | 3 | — |
| `NotifierNotFound` | 4 | — |

---

## credential_registry

### Functions

| Function | Parameters | Returns | Description |
|---|---|---|---|
| `initialize` | `env: Env, admin: Address` | `Result<(), Error>` | — |
| `set_issuer_admin` | `env: Env, caller: Address, issuer: Address, issuer_admin: Address` | `Result<bool, Error>` | — |
| `get_issuer_admin` | `env: Env, issuer: Address` | `Option<Address>` | — |
| `set_credential_root` | `env: Env, caller: Address, issuer: Address, root: BytesN<32>, metadata_hash: BytesN<32>, expiry: u64, signature: BytesN<64>` | `Result<u32, Error>` | — |
| `revoke_root` | `env: Env, caller: Address, issuer: Address, version: u32` | `Result<bool, Error>` | — |
| `set_revocation_root` | `env: Env, caller: Address, issuer: Address, revocation_root: BytesN<32>` | `Result<bool, Error>` | — |
| `get_active_root` | `env: Env, issuer: Address` | `Option<BytesN<32>>` | — |
| `get_active_version` | `env: Env, issuer: Address` | `u32` | — |
| `get_root` | `env: Env, issuer: Address, version: u32` | `Option<CredentialRootRecord>` | — |
| `get_revocation_root` | `env: Env, issuer: Address` | `Option<BytesN<32>>` | — |
| `is_root_revoked` | `env: Env, issuer: Address, root: BytesN<32>` | `bool` | — |
| `batch_set_credential_roots` | `env: Env, caller: Address, issuer: Address, roots: soroban_sdk::Vec<BytesN<32>>, metadata_hashes: soroban_sdk::Vec<BytesN<32>>, expiries: soroban_sdk::Vec<u64>, signatures: soroban_sdk::Vec<BytesN<64>>` | `Result<soroban_sdk::Vec<u32>, Error>` | — |
| `has_active_root` | `env: Env, issuer: Address` | `bool` | — |

### Types

#### `struct CredentialRootRecord`

| Field | Type | Description |
|---|---|---|
| `version` | `u32` | — |
| `root` | `BytesN<32>` | — |
| `metadata_hash` | `BytesN<32>` | — |
| `updated_at` | `u64` | — |
| `expiry` | `u64` | — |
| `signature` | `BytesN<64>` | — |
| `revoked` | `bool` | — |

#### `enum DataKey`

| Variant | Value | Description |
|---|---|---|
| `Initialized` | — | — |
| `Admin` | — | — |
| `IssuerAdmin(Address)` | — | — |
| `ActiveVersion(Address)` | — | — |
| `ActiveRoot(Address)` | — | — |
| `RootRecord(Address, u32)` | — | — |
| `RevocationRoot(Address)` | — | — |
| `RootToVersion(Address, BytesN<32>)` | — | — |

#### `enum Error`

| Variant | Value | Description |
|---|---|---|
| `AlreadyInitialized` | 1 | — |
| `NotInitialized` | 2 | — |
| `NotAuthorized` | 3 | — |
| `IssuerNotFound` | 4 | — |
| `RootVersionNotFound` | 5 | — |
| `InvalidCredentialId` | 6 | — |
| `InvalidExpiry` | 7 | — |
| `InvalidMetadata` | 8 | — |
| `InvalidSignature` | 9 | — |

---

## cross_chain_access

### Functions

| Function | Parameters | Returns | Description |
|---|---|---|---|
| `initialize` | `env: Env, admin: Address, bridge_contract: Address, identity_contract: Address` | `Result<bool, Error>` | — |
| `grant_access` | `env: Env, grantor: Address, grantee_chain: ChainId, grantee_address: String, permission_level: PermissionLevel, record_scope: AccessScope, duration: u64, conditions: Vec<AccessCondition>` | `Result<u64, Error>` | — |
| `revoke_access` | `env: Env, caller: Address, grant_id: u64` | `Result<bool, Error>` | — |
| `update_grant_conditions` | `env: Env, caller: Address, grant_id: u64, new_conditions: Vec<AccessCondition>` | `Result<bool, Error>` | — |
| `extend_grant` | `env: Env, caller: Address, grant_id: u64, additional_duration: u64` | `Result<bool, Error>` | — |
| `request_access` | `env: Env, requester_chain: ChainId, requester_address: String, patient: Address, requested_records: Vec<u64>, purpose: String, is_emergency: bool` | `Result<u64, Error>` | — |
| `process_request` | `env: Env, caller: Address, request_id: u64, approve: bool` | `Result<bool, Error>` | — |
| `create_delegation` | `env: Env, delegator: Address, delegate: Address, delegate_chain: ChainId, delegate_address: String, can_grant: bool, can_revoke: bool, can_manage_emergency: bool, duration: u64` | `Result<bool, Error>` | Create access management delegation BUG FIX: Each (delegator, delegate) pair is stored under a unique key |
| `revoke_delegation` | `env: Env, delegator: Address, delegate: Address` | `Result<bool, Error>` | — |
| `configure_emergency` | `env: Env, patient: Address, is_enabled: bool, auto_approve_duration: u64, required_attestations: u32, trusted_providers: Vec<String>` | `Result<bool, Error>` | Configure emergency access settings per patient BUG FIX: Each patient's config stored under unique key — was "emerg_key" |
| `log_access` | `env: Env, accessor_chain: ChainId, accessor_address: String, patient: Address, record_id: u64, action: AccessAction, ip_hash: BytesN<32>, success: bool` | `Result<u64, Error>` | — |
| `initiate_access_swap` | `env: Env, initiator: Address, counterpart_chain: ChainId, counterpart_address: String, offered_grant_id: u64, requested_permission: PermissionLevel, requested_scope: AccessScope, hash_lock: BytesN<32>, timelock_duration: u64` | `Result<u64, Error>` | Propose an atomic access swap: offer a grant in exchange for cross-chain access |
| `accept_access_swap` | `env: Env, acceptor: Address, swap_id: u64, offered_grant_id: u64, // Grant the counterpart is offering in return` | `Result<bool, Error>` | Accept a swap proposal: counterpart provides a grant in return |
| `finalize_access_swap` | `env: Env, caller: Address, swap_id: u64, secret: BytesN<32>, // Pre-image of hash_lock` | `Result<bool, Error>` | Finalize an accepted swap: atomically activates both sides of the exchange |
| `cancel_access_swap` | `env: Env, caller: Address, swap_id: u64` | `Result<bool, Error>` | Cancel a proposed swap (only initiator or after timelock expiry) |
| `verify_access` | `env: Env, accessor_chain: ChainId, accessor_address: String, patient: Address, record_id: u64, required_permission: PermissionLevel` | `bool` | — |
| `get_grant` | `env: Env, grant_id: u64` | `Option<AccessGrant>` | — |
| `get_request` | `env: Env, request_id: u64` | `Option<AccessRequest>` | — |
| `get_delegation` | `env: Env, delegator: Address, delegate: Address` | `Option<Delegation>` | — |
| `get_emergency_config` | `env: Env, patient: Address` | `Option<EmergencyConfig>` | — |
| `get_audit_entry` | `env: Env, entry_id: u64` | `Option<AuditEntry>` | — |
| `get_swap` | `env: Env, swap_id: u64` | `Option<SwapProposal>` | — |
| `is_paused` | `env: Env` | `bool` | — |
| `pause` | `env: Env, caller: Address` | `Result<bool, Error>` | — |
| `unpause` | `env: Env, caller: Address` | `Result<bool, Error>` | — |

### Types

#### `enum PermissionLevel`

| Variant | Value | Description |
|---|---|---|
| `None` | — | — |
| `Read` | — | — |
| `ReadConfidential` | — | — |
| `Write` | — | — |
| `Admin` | — | — |

#### `enum ChainId`

| Variant | Value | Description |
|---|---|---|
| `None` | — | — |
| `Stellar` | — | — |
| `Ethereum` | — | — |
| `Polygon` | — | — |
| `Avalanche` | — | — |
| `BinanceSmartChain` | — | — |
| `Arbitrum` | — | — |
| `Optimism` | — | — |
| `Custom(u32)` | — | — |

#### `struct AccessGrant`

| Field | Type | Description |
|---|---|---|
| `grant_id` | `u64` | — |
| `grantor` | `Address` | — |
| `grantee_chain` | `ChainId` | — |
| `grantee_address` | `String` | — |
| `permission_level` | `PermissionLevel` | — |
| `record_scope` | `AccessScope` | — |
| `granted_at` | `u64` | — |
| `expires_at` | `u64` | — |
| `is_active` | `bool` | — |
| `conditions` | `Vec<AccessCondition>` | — |

#### `enum AccessScope`

| Variant | Value | Description |
|---|---|---|
| `AllRecords` | — | — |
| `SpecificRecords(Vec<u64>)` | — | — |
| `CategoryBased(String)` | — | — |
| `TimeRanged(u64, u64)` | — | — |

#### `enum AccessCondition`

| Variant | Value | Description |
|---|---|---|
| `EmergencyOnly` | — | — |
| `RequireConsent` | — | — |
| `AuditRequired` | — | — |
| `SingleUse` | — | — |
| `TimeRestricted(u64, u64)` | — | — |

#### `struct AccessRequest`

| Field | Type | Description |
|---|---|---|
| `request_id` | `u64` | — |
| `requester_chain` | `ChainId` | — |
| `requester_address` | `String` | — |
| `patient` | `Address` | — |
| `requested_records` | `Vec<u64>` | — |
| `purpose` | `String` | — |
| `is_emergency` | `bool` | — |
| `created_at` | `u64` | — |
| `status` | `RequestStatus` | — |
| `decision_by` | `Option<Address>` | — |
| `decision_at` | `Option<u64>` | — |

#### `enum RequestStatus`

| Variant | Value | Description |
|---|---|---|
| `Pending` | — | — |
| `Approved` | — | — |
| `Rejected` | — | — |
| `Expired` | — | — |
| `Revoked` | — | — |

#### `struct AuditEntry`

| Field | Type | Description |
|---|---|---|
| `entry_id` | `u64` | — |
| `accessor_chain` | `ChainId` | — |
| `accessor_address` | `String` | — |
| `patient` | `Address` | — |
| `record_id` | `u64` | — |
| `action` | `AccessAction` | — |
| `timestamp` | `u64` | — |
| `ip_hash` | `BytesN<32>` | — |
| `success` | `bool` | — |

#### `enum AccessAction`

| Variant | Value | Description |
|---|---|---|
| `View` | — | — |
| `Download` | — | — |
| `Share` | — | — |
| `Export` | — | — |
| `EmergencyAccess` | — | — |

#### `struct Delegation`

| Field | Type | Description |
|---|---|---|
| `delegator` | `Address` | — |
| `delegate` | `Address` | — |
| `delegate_chain` | `ChainId` | — |
| `delegate_address` | `String` | — |
| `can_grant` | `bool` | — |
| `can_revoke` | `bool` | — |
| `can_manage_emergency` | `bool` | — |
| `created_at` | `u64` | — |
| `expires_at` | `u64` | — |
| `is_active` | `bool` | — |

#### `struct EmergencyConfig`

| Field | Type | Description |
|---|---|---|
| `patient` | `Address` | — |
| `is_enabled` | `bool` | — |
| `auto_approve_duration` | `u64` | — |
| `required_attestations` | `u32` | — |
| `trusted_providers` | `Vec<String>` | — |

#### `struct SwapProposal`

| Field | Type | Description |
|---|---|---|
| `swap_id` | `u64` | — |
| `initiator` | `Address` | — |
| `counterpart_chain` | `ChainId` | — |
| `counterpart_address` | `String` | — |
| `offered_grant_id` | `u64` | — |
| `requested_permission` | `PermissionLevel` | — |
| `requested_scope` | `AccessScope` | — |
| `hash_lock` | `BytesN<32>` | — |
| `timelock` | `u64` | — |
| `created_at` | `u64` | — |
| `status` | `SwapStatus` | — |
| `accepted_grant_id` | `u64` | — |

#### `enum SwapStatus`

| Variant | Value | Description |
|---|---|---|
| `Proposed` | — | — |
| `Accepted` | — | — |
| `Completed` | — | — |
| `Cancelled` | — | — |
| `Expired` | — | — |

#### `enum DataKey`

| Variant | Value | Description |
|---|---|---|
| `Admin` | — | — |
| `Bridge` | — | — |
| `Identity` | — | — |
| `Paused` | — | — |
| `GrantCount` | — | — |
| `RequestCount` | — | — |
| `AuditCount` | — | — |
| `SwapCount` | — | — |
| `Grants` | — | — |
| `Requests` | — | — |
| `AuditLog` | — | — |
| `Delegation(Address, Address)` | — | — |
| `EmergencyConfig(Address)` | — | — |
| `Swap(u64)` | — | — |

#### `enum Error`

| Variant | Value | Description |
|---|---|---|
| `NotAuthorized` | 1 | — |
| `ContractPaused` | 2 | — |
| `AlreadyInitialized` | 3 | — |
| `GrantNotFound` | 4 | — |
| `GrantExpired` | 5 | — |
| `GrantRevoked` | 6 | — |
| `RequestNotFound` | 7 | — |
| `RequestExpired` | 8 | — |
| `RequestAlreadyProcessed` | 9 | — |
| `DelegationNotFound` | 10 | — |
| `DelegationExpired` | 11 | — |
| `InsufficientPermissions` | 12 | — |
| `EmergencyNotEnabled` | 13 | — |
| `EmergencyNotAuthorized` | 14 | — |
| `InvalidScope` | 15 | — |
| `InvalidCondition` | 16 | — |
| `AuditRequired` | 17 | — |
| `SingleUseConsumed` | 18 | — |
| `TimeRestrictionViolated` | 19 | — |
| `Overflow` | 20 | — |
| `SwapNotFound` | 21 | — |
| `SwapExpired` | 22 | — |
| `SwapAlreadyProcessed` | 23 | — |

### Examples

#### `test_initialize`

```rust
let env = Env::default();
    let (client, admin, bridge, identity) = create_contract(&env);

    initialize_contract(&env, &client, &admin, &bridge, &identity);

    assert!(!client.is_paused());
```

#### `test_initialize_twice_fails`

```rust
let env = Env::default();
    let (client, admin, bridge, identity) = create_contract(&env);

    env.mock_all_auths();
    client.initialize(&admin, &bridge, &identity);

    let result = client.try_initialize(&admin, &bridge, &identity);
    assert_eq!(result, Err(Ok(Error::AlreadyInitialized)));
```

#### `test_grant_access`

```rust
let env = Env::default();
    let (client, admin, bridge, identity) = create_contract(&env);
    initialize_contract(&env, &client, &admin, &bridge, &identity);

    let patient = Address::generate(&env);
    let grantee_address = String::from_str(&env, "0x1234567890abcdef1234567890abcdef12345678");

    env.mock_all_auths();
```

---

## cross_chain_bridge

### Functions

| Function | Parameters | Returns | Description |
|---|---|---|---|
| `initialize` | `env: Env, admin: Address, medical_contract: Address, identity_contract: Address, access_contract: Address` | `Result<bool, Error>` | Initialize the bridge contract |
| `add_validator` | `env: Env, caller: Address, validator_address: Address, public_key: BytesN<32>, initial_stake: i128` | `Result<bool, Error>` | — |
| `deactivate_validator` | `env: Env, caller: Address, validator_address: Address` | `Result<bool, Error>` | — |
| `add_supported_chain` | `env: Env, caller: Address, chain: ChainId` | `Result<bool, Error>` | — |
| `set_min_confirmations` | `env: Env, caller: Address, min_confirmations: u32` | `Result<bool, Error>` | — |
| `pause` | `env: Env, caller: Address` | `Result<bool, Error>` | — |
| `unpause` | `env: Env, caller: Address` | `Result<bool, Error>` | — |
| `submit_message` | `env: Env, validator: Address, request: SubmitMessageRequest` | `Result<BytesN<32>, Error>` | — |
| `submit_message_batch` | `env: Env, validator: Address, requests: Vec<SubmitMessageRequest>` | `Result<Vec<BytesN<32>>, Error>` | Submit multiple cross-chain messages in a single call. Validates the validator once and processes each request in order. Returns the list of message IDs for successfully submitted messages. If any request fails, the error is returned immediately (partial batch may have been stored). |
| `confirm_message` | `env: Env, validator: Address, message_id: BytesN<32>, signature: BytesN<64>, nonce: u64` | `Result<bool, Error>` | Confirm a cross-chain message (validator attestation) BUG FIX: Confirmations now stored per message_id (was using shared "conf_key") |
| `execute_message` | `env: Env, caller: Address, message_id: BytesN<32>` | `Result<bool, Error>` | — |
| `fail_message` | `env: Env, caller: Address, message_id: BytesN<32>, reason: String` | `Result<bool, Error>` | Mark a message as failed and emit a failure event (validator only). This enables callers to detect failures and trigger refunds or retries. |
| `retry_message` | `env: Env, caller: Address, message_id: BytesN<32>, attempt: u32` | `Result<bool, Error>` | Retry a failed message with exponential backoff enforcement. The caller must wait at least `base_delay * 2^attempt` seconds since the original message timestamp before retrying. Resets the message status to Pending so validators can re-confirm it. |
| `initiate_atomic_tx` | `env: Env, caller: Address, tx_id: BytesN<32>, message_ids: Vec<BytesN<32>>` | `Result<BytesN<32>, Error>` | — |
| `prepare_atomic_tx` | `env: Env, validator: Address, tx_id: BytesN<32>, signature: BytesN<64>, nonce: u64` | `Result<bool, Error>` | — |
| `commit_atomic_tx` | `env: Env, caller: Address, tx_id: BytesN<32>` | `Result<bool, Error>` | — |
| `abort_atomic_tx` | `env: Env, caller: Address, tx_id: BytesN<32>` | `Result<bool, Error>` | — |
| `register_record_ref` | `env: Env, caller: Address, local_record_id: u64, external_chain: ChainId, external_record_id: String` | `Result<bool, Error>` | Register a cross-chain record reference BUG FIX: Each (record_id, chain) pair gets a unique storage key |
| `update_sync_status` | `env: Env, validator: Address, local_record_id: u64, external_chain: ChainId, status: SyncStatus, signature: BytesN<64>, nonce: u64` | `Result<bool, Error>` | Update sync status — validators attest to sync completion |
| `register_oracle` | `env: Env, caller: Address, oracle_address: Address, public_key: BytesN<32>, supported_chains: Vec<ChainId>` | `Result<bool, Error>` | Register an oracle node for cross-chain data validation |
| `deactivate_oracle` | `env: Env, caller: Address, oracle_address: Address` | `Result<bool, Error>` | Deactivate an oracle node |
| `submit_oracle_report` | `env: Env, oracle: Address, chain: ChainId, data_hash: BytesN<32>, data: String, block_height: u64, signature: BytesN<64>` | `Result<u64, Error>` | Submit a data report from an oracle node |
| `aggregate_oracle_data` | `env: Env, caller: Address, chain: ChainId, report_ids: Vec<u64>, consensus_hash: BytesN<32>, signature: BytesN<64>, nonce: u64` | `Result<bool, Error>` | Aggregate oracle reports to reach consensus for a chain |
| `submit_proof` | `env: Env, validator: Address, proof_id: BytesN<32>, source_chain: ChainId, record_hash: BytesN<32>, block_hash: BytesN<32>, merkle_root: BytesN<32>, prover: String, signature: BytesN<64>, nonce: u64` | `Result<BytesN<32>, Error>` | Submit a cryptographic proof for an external chain record |
| `verify_cross_chain_proof` | `env: Env, validator_address: Address, signature: BytesN<64>, nonce: u64, proof_id: BytesN<32>` | `Result<bool, Error>` | Verify a submitted cross-chain proof (additional validator attestation) |
| `validate_chain_address` | `_env: Env, chain: ChainId, address: String` | `bool` | Validate a chain address format (length + prefix check) Returns true if the address matches expected format for the given chain. |
| `get_chain_address_length` | `_env: Env, chain: ChainId` | `u32` | Get expected address length for a chain |
| `sync_cross_chain_event` | `env: Env, validator: Address, source_chain: ChainId, dest_chain: ChainId, event_type: CrossChainEventType, payload_hash: BytesN<32>, block_height: u64, signature: BytesN<64>, nonce: u64` | `Result<u64, Error>` | Submit a cross-chain event for synchronization |
| `process_sync_event` | `env: Env, validator: Address, event_id: u64, status: EventSyncStatus, signature: BytesN<64>, nonce: u64` | `Result<bool, Error>` | Mark a cross-chain event as processed/synced |
| `create_operation` | `env: Env, caller: Address, op_id: BytesN<32>, op_type: OperationType, refund_address: Address` | `Result<BytesN<32>, Error>` | Create a new cross-chain operation with timeout |
| `check_timeout` | `env: Env, op_id: BytesN<32>` | `Result<(), Error>` | Check if an operation has timed out and trigger refund if needed |
| `extend_timeout` | `env: Env, caller: Address, op_id: BytesN<32>, additional_time: u64` | `Result<bool, Error>` | Extend the deadline for an operation |
| `update_operation_status` | `env: Env, caller: Address, op_id: BytesN<32>, status: OperationStatus` | `Result<bool, Error>` | Update operation status |
| `get_operation` | `env: Env, op_id: BytesN<32>` | `Result<CrossChainOp, Error>` | Get operation details |
| `initiate_rollback` | `env: Env, caller: Address, op_id: BytesN<32>, op_type: RollbackOpType, original_state: String, reason: String` | `Result<BytesN<32>, Error>` | Initiate an emergency rollback for a failed cross-chain operation |
| `execute_rollback` | `env: Env, caller: Address, op_id: BytesN<32>` | `Result<bool, Error>` | Execute a rollback — marks the associated operation as failed/rolled back |
| `cancel_rollback` | `env: Env, caller: Address, op_id: BytesN<32>` | `Result<bool, Error>` | Cancel a pending rollback |
| `get_message` | `env: Env, message_id: BytesN<32>` | `Option<CrossChainMessage>` | — |
| `get_atomic_tx` | `env: Env, tx_id: BytesN<32>` | `Option<AtomicTransaction>` | — |
| `get_record_ref` | `env: Env, local_record_id: u64, external_chain: ChainId` | `Option<CrossChainRecordRef>` | — |
| `get_validator` | `env: Env, validator_address: Address` | `Option<Validator>` | — |
| `get_oracle_node` | `env: Env, oracle_address: Address` | `Option<OracleNode>` | — |
| `get_oracle_report` | `env: Env, report_id: u64` | `Option<OracleReport>` | — |
| `get_aggregated_oracle` | `env: Env, chain: ChainId` | `Option<AggregatedOracleData>` | — |
| `get_proof` | `env: Env, proof_id: BytesN<32>` | `Option<CrossChainProof>` | — |
| `get_rollback` | `env: Env, op_id: BytesN<32>` | `Option<RollbackRecord>` | — |
| `get_sync_event` | `env: Env, event_id: u64` | `Option<CrossChainEvent>` | — |
| `get_supported_chains` | `env: Env` | `Vec<ChainId>` | — |
| `is_paused` | `env: Env` | `bool` | — |
| `get_message_count` | `env: Env` | `u64` | — |
| `get_oracle_count` | `env: Env` | `u64` | — |
| `get_event_count` | `env: Env` | `u64` | — |
| `get_rollback_count` | `env: Env` | `u64` | — |
| `add_relayer` | `env: Env, admin: Address, relayer: Address` | `Result<(), Error>` | Add an authorized relayer (admin only). |
| `remove_relayer` | `env: Env, admin: Address, relayer: Address` | `Result<(), Error>` | Remove an authorized relayer (admin only). |
| `get_default_timeout_internal` | `_env: Env, op_type: OperationType` | `u64` | — |

### Types

#### `struct SubmitMessageRequest`

| Field | Type | Description |
|---|---|---|
| `message_id` | `BytesN<32>` | — |
| `source_chain` | `ChainId` | — |
| `dest_chain` | `ChainId` | — |
| `sender` | `String` | — |
| `recipient` | `Address` | — |
| `payload_type` | `MessageType` | — |
| `payload` | `String` | — |
| `nonce` | `u64` | — |
| `signature` | `BytesN<64>` | — |
| `v_signature` | `BytesN<64>` | — |
| `v_nonce` | `u64` | — |

#### `enum MessageStatus`

| Variant | Value | Description |
|---|---|---|
| `Pending` | — | — |
| `Verified` | — | — |
| `Executed` | — | — |
| `Failed` | — | — |
| `Expired` | — | — |

#### `enum ChainId`

| Variant | Value | Description |
|---|---|---|
| `Stellar` | — | — |
| `Ethereum` | — | — |
| `Polygon` | — | — |
| `Avalanche` | — | — |
| `BinanceSmartChain` | — | — |
| `Arbitrum` | — | — |
| `Optimism` | — | — |
| `Custom(u32)` | — | — |

#### `struct CrossChainMessage`

| Field | Type | Description |
|---|---|---|
| `message_id` | `BytesN<32>` | — |
| `source_chain` | `ChainId` | — |
| `dest_chain` | `ChainId` | — |
| `sender` | `String` | — |
| `recipient` | `Address` | — |
| `payload_type` | `MessageType` | — |
| `payload` | `String` | — |
| `nonce` | `u64` | — |
| `timestamp` | `u64` | — |
| `status` | `MessageStatus` | — |
| `signature` | `BytesN<64>` | — |

#### `enum MessageType`

| Variant | Value | Description |
|---|---|---|
| `RecordRequest` | — | — |
| `RecordResponse` | — | — |
| `IdentityVerify` | — | — |
| `IdentityConfirm` | — | — |
| `AccessGrant` | — | — |
| `AccessRevoke` | — | — |
| `RecordSync` | — | — |
| `EmergencyAccess` | — | — |

#### `struct Validator`

| Field | Type | Description |
|---|---|---|
| `address` | `Address` | — |
| `public_key` | `BytesN<32>` | — |
| `is_active` | `bool` | — |
| `stake` | `i128` | — |
| `confirmed_messages` | `u64` | — |

#### `struct CrossChainRecordRef`

| Field | Type | Description |
|---|---|---|
| `local_record_id` | `u64` | — |
| `external_chain` | `ChainId` | — |
| `external_record_id` | `String` | — |
| `sync_status` | `SyncStatus` | — |
| `last_sync` | `u64` | — |

#### `enum SyncStatus`

| Variant | Value | Description |
|---|---|---|
| `Synced` | — | — |
| `PendingSync` | — | — |
| `SyncFailed` | — | — |
| `Outdated` | — | — |

#### `struct AtomicTransaction`

| Field | Type | Description |
|---|---|---|
| `tx_id` | `BytesN<32>` | — |
| `messages` | `Vec<BytesN<32>>` | — |
| `status` | `AtomicTxStatus` | — |
| `created_at` | `u64` | — |
| `timeout` | `u64` | — |
| `confirmations` | `Vec<Address>` | — |

#### `enum AtomicTxStatus`

| Variant | Value | Description |
|---|---|---|
| `Initiated` | — | — |
| `Prepared` | — | — |
| `Committed` | — | — |
| `Aborted` | — | — |
| `Expired` | — | — |

#### `struct OracleNode`

| Field | Type | Description |
|---|---|---|
| `address` | `Address` | — |
| `public_key` | `BytesN<32>` | — |
| `supported_chains` | `Vec<ChainId>` | — |
| `is_active` | `bool` | — |
| `reputation` | `u32` | — |
| `total_reports` | `u64` | — |

#### `struct OracleReport`

| Field | Type | Description |
|---|---|---|
| `report_id` | `u64` | — |
| `oracle` | `Address` | — |
| `chain` | `ChainId` | — |
| `data_hash` | `BytesN<32>` | — |
| `data` | `String` | — |
| `block_height` | `u64` | — |
| `timestamp` | `u64` | — |
| `signature` | `BytesN<64>` | — |
| `status` | `OracleStatus` | — |

#### `enum OracleStatus`

| Variant | Value | Description |
|---|---|---|
| `Submitted` | — | — |
| `Validated` | — | — |
| `Rejected` | — | — |
| `Aggregated` | — | — |

#### `struct AggregatedOracleData`

| Field | Type | Description |
|---|---|---|
| `chain` | `ChainId` | — |
| `consensus_hash` | `BytesN<32>` | — |
| `report_count` | `u32` | — |
| `consensus_threshold` | `u32` | — |
| `aggregated_at` | `u64` | — |
| `is_finalized` | `bool` | — |

#### `struct CrossChainProof`

| Field | Type | Description |
|---|---|---|
| `proof_id` | `BytesN<32>` | — |
| `source_chain` | `ChainId` | — |
| `record_hash` | `BytesN<32>` | — |
| `block_hash` | `BytesN<32>` | — |
| `merkle_root` | `BytesN<32>` | — |
| `timestamp` | `u64` | — |
| `prover` | `String` | — |
| `verifier_count` | `u32` | — |
| `verified` | `bool` | — |

#### `struct RollbackRecord`

| Field | Type | Description |
|---|---|---|
| `op_id` | `BytesN<32>` | — |
| `op_type` | `RollbackOpType` | — |
| `original_state` | `String` | — |
| `triggered_by` | `Address` | — |
| `triggered_at` | `u64` | — |
| `status` | `RollbackStatus` | — |
| `reason` | `String` | — |
| `completed_at` | `u64` | — |

#### `enum RollbackOpType`

| Variant | Value | Description |
|---|---|---|
| `MessageRollback` | — | — |
| `AtomicTxRollback` | — | — |
| `RecordSyncRollback` | — | — |

#### `enum RollbackStatus`

| Variant | Value | Description |
|---|---|---|
| `Initiated` | — | — |
| `InProgress` | — | — |
| `Completed` | — | — |
| `Failed` | — | — |

#### `struct CrossChainOp`

| Field | Type | Description |
|---|---|---|
| `id` | `BytesN<32>` | — |
| `deadline` | `u64` | — |
| `refund_address` | `Address` | — |
| `op_type` | `OperationType` | — |
| `status` | `OperationStatus` | — |
| `created_at` | `u64` | — |
| `extended_count` | `u32` | — |

#### `enum OperationType`

| Variant | Value | Description |
|---|---|---|
| `TokenTransfer` | — | — |
| `MessagePassing` | — | — |
| `Verification` | — | — |
| `AtomicSwap` | — | — |
| `RecordSync` | — | — |

#### `enum OperationStatus`

| Variant | Value | Description |
|---|---|---|
| `Pending` | — | — |
| `InProgress` | — | — |
| `Completed` | — | — |
| `Failed` | — | — |
| `Refunded` | — | — |
| `Extended` | — | — |

#### `struct CrossChainEvent`

| Field | Type | Description |
|---|---|---|
| `event_id` | `u64` | — |
| `source_chain` | `ChainId` | — |
| `dest_chain` | `ChainId` | — |
| `event_type` | `CrossChainEventType` | — |
| `payload_hash` | `BytesN<32>` | — |
| `block_height` | `u64` | — |
| `timestamp` | `u64` | — |
| `sync_status` | `EventSyncStatus` | — |

#### `enum CrossChainEventType`

| Variant | Value | Description |
|---|---|---|
| `RecordCreated` | — | — |
| `RecordUpdated` | — | — |
| `AccessGranted` | — | — |
| `AccessRevoked` | — | — |
| `IdentityVerified` | — | — |
| `EmergencyTriggered` | — | — |

#### `enum EventSyncStatus`

| Variant | Value | Description |
|---|---|---|
| `Pending` | — | — |
| `Synced` | — | — |
| `Failed` | — | — |

#### `enum DataKey`

| Variant | Value | Description |
|---|---|---|
| `ValidatorNonce(BytesN<32>)` | — | — |
| `Admin` | — | — |
| `MedicalContract` | — | — |
| `IdentityContract` | — | — |
| `AccessContract` | — | — |
| `Paused` | — | — |
| `MessageCount` | — | — |
| `MinConfirmations` | — | — |
| `SupportedChains` | — | — |
| `OracleCount` | — | — |
| `RollbackCount` | — | — |
| `EventCount` | — | — |
| `OpCount` | — | — |
| `Nonce(String)` | — | — |
| `Validator(Address)` | — | — |
| `Message(BytesN<32>)` | — | — |
| `RecordRef(u64, ChainId)` | — | — |
| `AtomicTx(BytesN<32>)` | — | — |
| `OracleNode(Address)` | — | — |
| `OracleReport(u64)` | — | — |
| `AggregatedOracle(ChainId)` | — | — |
| `Proof(BytesN<32>)` | — | — |
| `Rollback(BytesN<32>)` | — | — |
| `Event(u64)` | — | — |
| `CrossChainOp(BytesN<32>)` | — | — |
| `Confirmations(BytesN<32>)` | — | — |
| `AuthorizedRelayer(Address)` | — | — |

### Error Codes

| Variant | Code | Description |
|---|---|---|
| `Unauthorized` | 100 | — |
| `UnauthorizedRelayer` | 101 | — |
| `InsufficientConfirmations` | 120 | — |
| `InsufficientOracleReports` | 121 | — |
| `DuplicateOracleReport` | 122 | — |
| `InvalidSignature` | 207 | — |
| `InvalidMessage` | 280 | — |
| `InvalidNonce` | 281 | — |
| `InvalidPayload` | 282 | — |
| `InvalidAddress` | 290 | — |
| `BatchTooLarge` | 283 | — |
| `AlreadyInitialized` | 301 | — |
| `ContractPaused` | 302 | — |
| `Overflow` | 580 | — |
| `MessageNotFound` | 480 | — |
| `MessageExpired` | 481 | — |
| `MessageAlreadyProcessed` | 482 | — |
| `AtomicTxNotFound` | 486 | — |
| `AtomicTxExpired` | 487 | — |
| `AtomicTxAlreadyProcessed` | 488 | — |
| `RecordRefNotFound` | 489 | — |
| `RollbackNotFound` | 490 | — |
| `RollbackAlreadyProcessed` | 491 | — |
| `EventNotFound` | 492 | — |
| `ValidatorNotFound` | 483 | — |
| `ValidatorNotActive` | 484 | — |
| `DuplicateConfirmation` | 485 | — |
| `ProofNotFound` | 610 | — |
| `ProofAlreadyVerified` | 611 | — |
| `InvalidChain` | 703 | — |
| `ChainNotSupported` | 720 | — |
| `OracleNotFound` | 721 | — |
| `OracleNotActive` | 722 | — |
| `OperationNotFound` | 800 | — |
| `OperationExpired` | 801 | — |
| `OperationAlreadyCompleted` | 802 | — |
| `MaxExtensionsReached` | 803 | — |
| `RefundFailed` | 804 | — |

### Examples

#### `test_initialize`

```rust
let env = Env::default();
    let (client, admin, medical, identity, access) = create_contract(&env);

    initialize_contract(&env, &client, &admin, &medical, &identity, &access);

    assert!(!client.is_paused());
    assert_eq!(client.get_message_count(), 0);
```

#### `test_initialize_twice_fails`

```rust
let env = Env::default();
    let (client, admin, medical, identity, access) = create_contract(&env);

    env.mock_all_auths();
    client.initialize(&admin, &medical, &identity, &access);

    let result = client.try_initialize(&admin, &medical, &identity, &access);
    assert_eq!(result, Err(Ok(Error::AlreadyInitialized)));
```

#### `test_add_validator`

```rust
let env = Env::default();
    let (client, admin, medical, identity, access) = create_contract(&env);
    initialize_contract(&env, &client, &admin, &medical, &identity, &access);

    let validator = Address::generate(&env);
    let public_key = BytesN::from_array(&env, &[3u8; 32]);

    env.mock_all_auths();
    let result = client.add_validator(&admin, &validator, &public_key, &1000);
```

---

## cross_chain_enhancements

### Functions

| Function | Parameters | Returns | Description |
|---|---|---|---|
| `initialize` | `env: Env, admin: Address` | `Result<(), Error>` | — |
| `submit_zk_ownership_proof` | `env: Env, prover: Address, record_id: u64, chain: ChainId, proof_data: BytesN<64>, statement_hash: BytesN<32>` | `Result<BytesN<32>, Error>` | Submit a zero-knowledge proof of data ownership This proves ownership of a medical record without revealing its contents |
| `verify_zk_ownership_proof` | `env: Env, verifier: Address, proof_id: BytesN<32>` | `Result<bool, Error>` | Verify a zero-knowledge ownership proof |
| `create_data_integrity_proof` | `env: Env, caller: Address, data_hash: BytesN<32>, merkle_root: BytesN<32>, merkle_path: Vec<BytesN<32>>, leaf_index: u32, chain_id: ChainId` | `Result<BytesN<32>, Error>` | Create a data integrity proof using Merkle tree |
| `check_replay_protection` | `env: Env, message_hash: BytesN<32>, source_chain: ChainId` | `Result<bool, Error>` | Check for replay attacks by tracking seen messages. Uses the shared `replay_protection` library for expiration checks and chain-ID conversion for chain binding. |
| `set_rate_limit` | `env: Env, admin: Address, address: Address, daily_limit: u64` | `Result<(), Error>` | Set rate limit for an address |
| `check_rate_limit` | `env: Env, caller: Address, amount: u64` | `Result<bool, Error>` | Check and update rate limit for an operation |
| `get_zk_proof` | `env: Env, proof_id: BytesN<32>` | `Option<ZKOwnershipProof>` | Get ZK proof status |
| `get_integrity_proof` | `env: Env, proof_id: BytesN<32>` | `Option<ZKDataIntegrityProof>` | Get integrity proof |

### Types

#### `struct ZKOwnershipProof`

| Field | Type | Description |
|---|---|---|
| `proof_id` | `BytesN<32>` | — |
| `record_id` | `u64` | — |
| `owner` | `Address` | — |
| `chain` | `ChainId` | — |
| `proof_data` | `BytesN<64>` | — |
| `statement_hash` | `BytesN<32>` | — |
| `verified` | `bool` | — |
| `verified_at` | `Option<u64>` | — |
| `verifier` | `Option<Address>` | — |

#### `struct ZKDataIntegrityProof`

| Field | Type | Description |
|---|---|---|
| `proof_id` | `BytesN<32>` | — |
| `data_hash` | `BytesN<32>` | — |
| `merkle_root` | `BytesN<32>` | — |
| `merkle_path` | `Vec<BytesN<32>>` | — |
| `leaf_index` | `u32` | — |
| `proven_at` | `u64` | — |
| `chain_id` | `ChainId` | — |

#### `enum ChainId`

| Variant | Value | Description |
|---|---|---|
| `Stellar` | — | — |
| `Ethereum` | — | — |
| `Polygon` | — | — |
| `Avalanche` | — | — |
| `BinanceSmartChain` | — | — |
| `Arbitrum` | — | — |
| `Optimism` | — | — |
| `Custom(u32)` | — | — |

#### `struct ReplayProtection`

| Field | Type | Description |
|---|---|---|
| `message_hash` | `BytesN<32>` | — |
| `source_chain` | `ChainId` | — |
| `seen_at` | `u64` | — |
| `expires_at` | `u64` | — |

#### `struct RateLimit`

| Field | Type | Description |
|---|---|---|
| `address` | `Address` | — |
| `daily_limit` | `u64` | — |
| `used_today` | `u64` | — |
| `last_reset` | `u64` | — |
| `is_active` | `bool` | — |

#### `enum DataKey`

| Variant | Value | Description |
|---|---|---|
| `ZKProof(BytesN<32>)` | — | — |
| `IntegrityProof(BytesN<32>)` | — | — |
| `SeenMessage(BytesN<32>)` | — | — |
| `RateLimit(Address)` | — | — |
| `Admin` | — | — |
| `Initialized` | — | — |
| `ZKCounter` | — | — |
| `IntegrityCounter` | — | — |

#### `enum Error`

| Variant | Value | Description |
|---|---|---|
| `NotAuthorized` | 1 | — |
| `NotInitialized` | 2 | — |
| `AlreadyInitialized` | 3 | — |
| `InvalidProof` | 4 | — |
| `ProofAlreadyVerified` | 5 | — |
| `ProofNotFound` | 6 | — |
| `ReplayDetected` | 7 | — |
| `RateLimitExceeded` | 8 | — |
| `ArithmeticOverflow` | 9 | — |
| `InvalidMerklePath` | 10 | — |
| `ExpiredMessage` | 11 | — |

---

## cross_chain_identity

### Functions

| Function | Parameters | Returns | Description |
|---|---|---|---|
| `initialize` | `env: Env, admin: Address, bridge_contract: Address` | `Result<bool, Error>` | — |
| `add_validator` | `env: Env, caller: Address, validator_address: Address, name: String, public_key: BytesN<32>` | `Result<bool, Error>` | — |
| `deactivate_validator` | `env: Env, caller: Address, validator_address: Address` | `Result<bool, Error>` | — |
| `update_trust_score` | `env: Env, caller: Address, validator_address: Address, trust_score: u32` | `Result<bool, Error>` | — |
| `set_min_attestations` | `env: Env, caller: Address, min_attestations: u32` | `Result<bool, Error>` | — |
| `pause` | `env: Env, caller: Address` | `Result<bool, Error>` | — |
| `unpause` | `env: Env, caller: Address` | `Result<bool, Error>` | — |
| `request_verification` | `env: Env, stellar_address: Address, external_chain: ChainId, external_address: String, proof: BytesN<64>` | `Result<u64, Error>` | — |
| `attest_verification` | `env: Env, validator: Address, request_id: u64, is_valid: bool, signature: BytesN<64>` | `Result<bool, Error>` | Validator attests to a verification request BUG FIX: each attestation stored per (request_id, validator) — was "att_key" |
| `revoke_identity` | `env: Env, caller: Address, stellar_address: Address, external_chain: ChainId` | `Result<bool, Error>` | — |
| `initiate_sync` | `env: Env, stellar_address: Address, source_chain: ChainId, dest_chain: ChainId` | `Result<u64, Error>` | — |
| `update_sync_status` | `env: Env, validator: Address, sync_id: u64, status: SyncStatus, proof: BytesN<32>` | `Result<bool, Error>` | — |
| `get_identity` | `env: Env, stellar_address: Address, external_chain: ChainId` | `Option<CrossChainIdentity>` | Get identity by Stellar address and external chain BUG FIX: each (stellar_address, chain) has a unique storage entry |
| `verify_identity` | `env: Env, stellar_address: Address, external_chain: ChainId` | `bool` | — |
| `get_request` | `env: Env, request_id: u64` | `Option<VerificationRequest>` | — |
| `get_sync` | `env: Env, sync_id: u64` | `Option<IdentitySync>` | — |
| `get_validator` | `env: Env, validator_address: Address` | `Option<IdentityValidator>` | — |
| `get_attestation` | `env: Env, request_id: u64, validator: Address` | `Option<Attestation>` | — |
| `is_paused` | `env: Env` | `bool` | — |

### Types

#### `enum VerificationStatus`

| Variant | Value | Description |
|---|---|---|
| `Unverified` | — | — |
| `Pending` | — | — |
| `Verified` | — | — |
| `Revoked` | — | — |
| `Expired` | — | — |

#### `enum ChainId`

| Variant | Value | Description |
|---|---|---|
| `Stellar` | — | — |
| `Ethereum` | — | — |
| `Polygon` | — | — |
| `Avalanche` | — | — |
| `BinanceSmartChain` | — | — |
| `Arbitrum` | — | — |
| `Optimism` | — | — |
| `Custom(u32)` | — | — |

#### `struct CrossChainIdentity`

| Field | Type | Description |
|---|---|---|
| `stellar_address` | `Address` | — |
| `external_chain` | `ChainId` | — |
| `external_address` | `String` | — |
| `verification_status` | `VerificationStatus` | — |
| `verified_at` | `u64` | — |
| `expires_at` | `u64` | — |
| `attestations` | `u32` | — |
| `metadata_hash` | `BytesN<32>` | — |

#### `struct VerificationRequest`

| Field | Type | Description |
|---|---|---|
| `request_id` | `u64` | — |
| `stellar_address` | `Address` | — |
| `external_chain` | `ChainId` | — |
| `external_address` | `String` | — |
| `proof` | `BytesN<64>` | — |
| `created_at` | `u64` | — |
| `status` | `RequestStatus` | — |
| `validator_attestations` | `Vec<Address>` | — |

#### `enum RequestStatus`

| Variant | Value | Description |
|---|---|---|
| `Pending` | — | — |
| `Approved` | — | — |
| `Rejected` | — | — |
| `Expired` | — | — |

#### `struct Attestation`

| Field | Type | Description |
|---|---|---|
| `validator` | `Address` | — |
| `stellar_address` | `Address` | — |
| `external_chain` | `ChainId` | — |
| `attested_at` | `u64` | — |
| `is_valid` | `bool` | — |
| `signature` | `BytesN<64>` | — |

#### `struct IdentityValidator`

| Field | Type | Description |
|---|---|---|
| `address` | `Address` | — |
| `name` | `String` | — |
| `public_key` | `BytesN<32>` | — |
| `is_active` | `bool` | — |
| `trust_score` | `u32` | — |
| `total_attestations` | `u64` | — |

#### `struct IdentitySync`

| Field | Type | Description |
|---|---|---|
| `stellar_address` | `Address` | — |
| `source_chain` | `ChainId` | — |
| `dest_chain` | `ChainId` | — |
| `sync_timestamp` | `u64` | — |
| `sync_status` | `SyncStatus` | — |
| `sync_proof` | `BytesN<32>` | — |

#### `enum SyncStatus`

| Variant | Value | Description |
|---|---|---|
| `Initiated` | — | — |
| `InProgress` | — | — |
| `Completed` | — | — |
| `Failed` | — | — |

#### `enum DataKey`

| Variant | Value | Description |
|---|---|---|
| `Admin` | — | — |
| `Bridge` | — | — |
| `Paused` | — | — |
| `RequestCount` | — | — |
| `SyncCount` | — | — |
| `MinAttestations` | — | — |
| `IdentityTtl` | — | — |
| `Validator(Address)` | — | — |
| `Request(u64)` | — | — |
| `Identity(Address, ChainId)` | — | — |
| `Attestation(u64, Address)` | — | — |
| `Sync(u64)` | — | — |

#### `enum Error`

| Variant | Value | Description |
|---|---|---|
| `NotAuthorized` | 1 | — |
| `ContractPaused` | 2 | — |
| `AlreadyInitialized` | 3 | — |
| `IdentityNotFound` | 4 | — |
| `IdentityAlreadyExists` | 5 | — |
| `IdentityExpired` | 6 | — |
| `IdentityRevoked` | 7 | — |
| `RequestNotFound` | 8 | — |
| `RequestExpired` | 9 | — |
| `RequestAlreadyProcessed` | 10 | — |
| `ValidatorNotFound` | 11 | — |
| `ValidatorNotActive` | 12 | — |
| `DuplicateAttestation` | 13 | — |
| `InsufficientAttestations` | 14 | — |
| `InvalidProof` | 15 | — |
| `InvalidChain` | 16 | — |
| `SyncNotFound` | 17 | — |
| `SyncFailed` | 18 | — |

### Examples

#### `test_initialize`

```rust
let env = Env::default();
    let (client, admin, bridge) = create_contract(&env);

    initialize_contract(&env, &client, &admin, &bridge);

    assert!(!client.is_paused());
```

#### `test_initialize_twice_fails`

```rust
let env = Env::default();
    let (client, admin, bridge) = create_contract(&env);

    env.mock_all_auths();
    client.initialize(&admin, &bridge);

    let result = client.try_initialize(&admin, &bridge);
    assert_eq!(result, Err(Ok(Error::AlreadyInitialized)));
```

#### `test_add_validator`

```rust
let env = Env::default();
    let (client, admin, bridge) = create_contract(&env);
    initialize_contract(&env, &client, &admin, &bridge);

    let validator = Address::generate(&env);
    let name = String::from_str(&env, "Validator1");
    let public_key = generate_public_key(&env);

    env.mock_all_auths();
```

---

## crypto_registry

### Functions

| Function | Parameters | Returns | Description |
|---|---|---|---|
| `initialize` | `env: Env, admin: Address` | `Result<(), Error>` | Initialize the registry with an admin address for policy upgrades. Key registration/rotation is always self-authorized by the account. |
| `register_key_bundle` | `env: Env, owner: Address, encryption_key: PublicKey, pq_encryption_key: PublicKey, has_pq_encryption_key: bool, signing_key: PublicKey, has_signing_key: bool` | `Result<u32, Error>` | Register (or rotate) the caller's key bundle.  Returns the newly assigned version. |
| `revoke_key_bundle` | `env: Env, owner: Address, version: u32` | `Result<(), Error>` | Revoke a specific key bundle version. |
| `get_current_version` | `env: Env, owner: Address` | `Result<u32, Error>` | — |
| `get_current_key_bundle` | `env: Env, owner: Address` | `Result<Option<KeyBundle>, Error>` | — |
| `get_key_bundle` | `env: Env, owner: Address, version: u32` | `Result<Option<KeyBundle>, Error>` | — |
| `rotate_key` | `env: Env, owner: Address, new_encryption_key: PublicKey, new_pq_encryption_key: PublicKey, has_pq_encryption_key: bool, new_signing_key: PublicKey, has_signing_key: bool` | `Result<u32, Error>` | Rotate a specific key bundle for an owner with automatic old-key invalidation. This implements the envelope encryption pattern: the new key bundle replaces the old one atomically, and the old KEK is revoked so it cannot be used for future encryption operations. |
| `get_all_key_versions` | `env: Env, owner: Address` | `Result<Vec<u32>, Error>` | Get all key bundle versions for an owner (including revoked ones). |

### Types

#### `enum KeyAlgorithm`

| Variant | Value | Description |
|---|---|---|
| `X25519` | — | — |
| `Ed25519` | — | — |
| `Secp256k1` | — | — |
| `Kyber768` | — | — |
| `Kyber1024` | — | — |
| `Dilithium2` | — | — |
| `Dilithium3` | — | — |
| `Dilithium5` | — | — |
| `Falcon512` | — | — |
| `Falcon1024` | — | — |
| `XMSS` | — | — |
| `SphincsPlus` | — | — |
| `McEliece348864` | — | — |
| `McEliece460896` | — | — |
| `McEliece6688128` | — | — |
| `McEliece6960119` | — | — |
| `McEliece8192128` | — | — |
| `Rainbow` | — | — |
| `GeMSS` | — | — |
| `HkdfSha3` | — | — |
| `Custom(u32)` | — | — |

#### `struct PublicKey`

| Field | Type | Description |
|---|---|---|
| `algorithm` | `KeyAlgorithm` | — |
| `key` | `Bytes` | — |

#### `struct KeyBundle`

| Field | Type | Description |
|---|---|---|
| `version` | `u32` | — |
| `created_at` | `u64` | — |
| `revoked` | `bool` | — |
| `encryption_key` | `PublicKey` | — |
| `has_pq_encryption_key` | `bool` | — |
| `pq_encryption_key` | `PublicKey` | — |
| `has_signing_key` | `bool` | — |
| `signing_key` | `PublicKey` | — |
| `bundle_id` | `BytesN<32>` | — |

#### `enum DataKey`

| Variant | Value | Description |
|---|---|---|
| `Initialized` | — | — |
| `Admin` | — | — |
| `CurrentVersion(Address)` | — | — |
| `Bundle(Address, u32)` | — | — |

#### `enum Error`

| Variant | Value | Description |
|---|---|---|
| `AlreadyInitialized` | 1 | — |
| `NotInitialized` | 2 | — |
| `NotAuthorized` | 3 | — |
| `InvalidKey` | 4 | — |
| `KeyNotFound` | 5 | — |
| `KeyAlreadyRevoked` | 6 | — |
| `InvalidKeyLength` | 7 | — |

### Examples

#### `key_bundle_registration_and_rotation`

```rust
let env = Env::default();
    env.mock_all_auths();

    let (client, _id) = register_contract(&env);
    let admin = soroban_sdk::Address::generate(&env);
    client.initialize(&admin);

    let alice = soroban_sdk::Address::generate(&env);
    let enc_key = PublicKey {
```

#### `revoke_bundle_marks_revoked`

```rust
let env = Env::default();
    env.mock_all_auths();

    let (client, _id) = register_contract(&env);
    let admin = soroban_sdk::Address::generate(&env);
    client.initialize(&admin);

    let alice = soroban_sdk::Address::generate(&env);
    let enc_key = PublicKey {
```

#### `post_quantum_key_registration`

```rust
let env = Env::default();
    env.mock_all_auths();

    let (client, _id) = register_contract(&env);
    let admin = soroban_sdk::Address::generate(&env);
    client.initialize(&admin);

    let alice = soroban_sdk::Address::generate(&env);
```

---

## deprecation_framework

### Functions

| Function | Parameters | Returns | Description |
|---|---|---|---|
| `initialize` | `env: Env, admin: Address` | `Result<(), Error>` | Initialize the deprecation framework |
| `mark_for_deprecation` | `env: Env, admin: Address, contract_id: String, contract_name: String, reason: String, replacement_contract: Option<String>` | `Result<(), Error>` | Mark a contract for deprecation |
| `set_sunset_timeline` | `env: Env, admin: Address, contract_id: String, announcement_date: u64, support_end_date: u64, removal_date: u64` | `Result<(), Error>` | Set sunset timeline for a contract |
| `add_migration_guide` | `env: Env, admin: Address, contract_id: String, guide_title: String, guide_content: String, code_examples: Vec<String>` | `Result<(), Error>` | Add migration guidance |
| `update_deprecation_phase` | `env: Env, admin: Address, contract_id: String, new_phase: DeprecationPhase` | `Result<(), Error>` | Update deprecation phase |
| `publish_user_communication` | `env: Env, admin: Address, contract_id: String, message: String, communication_type: String, // "email", "notification", "announcement"` | `Result<u64, Error>` | Publish user communication |
| `create_removal_checklist` | `env: Env, admin: Address, contract_id: String, checklist_items: Vec<String>` | `Result<(), Error>` | Create removal checklist |
| `mark_checklist_item_complete` | `env: Env, admin: Address, contract_id: String, item_index: u32` | `Result<(), Error>` | Mark checklist item as complete |
| `get_deprecation_status` | `env: Env, contract_id: String` | `Result<DeprecationStatus, Error>` | Get deprecation status |
| `get_sunset_timeline` | `env: Env, contract_id: String` | `Result<SunsetTimeline, Error>` | Get sunset timeline |
| `get_migration_guide` | `env: Env, contract_id: String` | `Result<MigrationGuide, Error>` | Get migration guide |
| `is_deprecated` | `env: Env, contract_id: String` | `bool` | Check if contract is deprecated |

### Error Codes

| Variant | Code | Description |
|---|---|---|
| `NotInitialized` | 1 | — |
| `AlreadyInitialized` | 2 | — |
| `NotAuthorized` | 3 | — |
| `ContractNotFound` | 4 | — |
| `ContractAlreadyDeprecated` | 5 | — |
| `InvalidTimeline` | 6 | — |
| `InvalidPhaseTransition` | 7 | — |
| `TimelineNotFound` | 8 | — |
| `GuideNotFound` | 9 | — |
| `ChecklistNotFound` | 10 | — |
| `InvalidChecklistIndex` | 11 | — |

### Examples

#### `test_initialize`

```rust
let env = Env::default();
        env.mock_all_auths();

        let admin = Address::generate(&env);
        let contract_id = env.register_contract(None, DeprecationFramework);
        let client = DeprecationFrameworkClient::new(&env, &contract_id);

        client.initialize(&admin);
    }
```

---

## dicomweb_services

### Functions

| Function | Parameters | Returns | Description |
|---|---|---|---|
| `initialize` | `env: Env, admin: Address, medical_imaging_contract: Address` | `Result<bool, Error>` | — |
| `set_paused` | `env: Env, caller: Address, paused: bool` | `Result<bool, Error>` | — |
| `qido_search_studies` | `env: Env, caller: Address, params: DicomwebQueryParams` | `Result<Vec<DicomwebStudy>, Error>` | — |
| `qido_search_series` | `env: Env, caller: Address, study_instance_uid: String, params: DicomwebQueryParams` | `Result<Vec<DicomwebSeries>, Error>` | — |
| `qido_search_instances` | `env: Env, caller: Address, study_instance_uid: String, series_instance_uid: String, params: DicomwebQueryParams` | `Result<Vec<DicomwebInstance>, Error>` | — |
| `wado_retrieve_study` | `env: Env, caller: Address, study_instance_uid: String` | `Result<Vec<DicomwebInstance>, Error>` | — |
| `wado_retrieve_series` | `env: Env, caller: Address, study_instance_uid: String, series_instance_uid: String` | `Result<Vec<DicomwebInstance>, Error>` | — |
| `wado_retrieve_instance` | `env: Env, caller: Address, study_instance_uid: String, series_instance_uid: String, sop_instance_uid: String` | `Result<DicomwebInstance, Error>` | — |
| `wado_retrieve_bulk_data` | `env: Env, caller: Address, sop_instance_uid: String` | `Result<DicomwebBulkData, Error>` | — |
| `wado_retrieve_bulk_data_batch` | `env: Env, caller: Address, sop_instance_uids: Vec<String>` | `Result<Vec<DicomwebBulkData>, Error>` | — |
| `stow_store_instance` | `env: Env, caller: Address, request: StowRequest` | `Result<StowResponse, Error>` | — |
| `stow_store_batch` | `env: Env, caller: Address, requests: Vec<StowRequest>` | `Result<Vec<StowResponse>, Error>` | — |
| `cache_set` | `env: Env, caller: Address, key: BytesN<32>, data: Bytes` | `Result<bool, Error>` | — |
| `cache_get` | `env: Env, key: BytesN<32>` | `Result<Bytes, Error>` | — |
| `cache_invalidate` | `env: Env, caller: Address, key: BytesN<32>` | `Result<bool, Error>` | — |
| `get_study` | `env: Env, study_instance_uid: String` | `Option<DicomwebStudy>` | — |
| `get_series` | `env: Env, study_instance_uid: String, series_instance_uid: String` | `Option<DicomwebSeries>` | — |
| `get_instance` | `env: Env, study_instance_uid: String, series_instance_uid: String, sop_instance_uid: String` | `Option<DicomwebInstance>` | — |
| `get_instance_by_sop` | `env: Env, sop_instance_uid: String` | `Option<DicomwebInstance>` | — |
| `list_studies` | `env: Env` | `Vec<String>` | — |
| `get_concurrency_stats` | `env: Env` | `ConcurrencyTracker` | — |
| `initialize` | `env: Env, admin: Address, medical_imaging_contract: Address` | `Result<bool, Error>` | — |
| `set_paused` | `env: Env, caller: Address, paused: bool` | `Result<bool, Error>` | — |
| `qido_search_studies` | `env: Env, caller: Address, params: DicomwebQueryParams` | `Result<Vec<DicomwebStudy>, Error>` | — |
| `qido_search_series` | `env: Env, caller: Address, study_instance_uid: String, params: DicomwebQueryParams` | `Result<Vec<DicomwebSeries>, Error>` | — |
| `qido_search_instances` | `env: Env, caller: Address, study_instance_uid: String, series_instance_uid: String, params: DicomwebQueryParams` | `Result<Vec<DicomwebInstance>, Error>` | — |
| `wado_retrieve_study` | `env: Env, caller: Address, study_instance_uid: String` | `Result<Vec<DicomwebInstance>, Error>` | — |
| `wado_retrieve_series` | `env: Env, caller: Address, study_instance_uid: String, series_instance_uid: String` | `Result<Vec<DicomwebInstance>, Error>` | — |
| `wado_retrieve_instance` | `env: Env, caller: Address, study_instance_uid: String, series_instance_uid: String, sop_instance_uid: String` | `Result<DicomwebInstance, Error>` | — |
| `wado_retrieve_bulk_data` | `env: Env, caller: Address, sop_instance_uid: String` | `Result<DicomwebBulkData, Error>` | — |
| `wado_retrieve_bulk_data_batch` | `env: Env, caller: Address, sop_instance_uids: Vec<String>` | `Result<Vec<DicomwebBulkData>, Error>` | — |
| `stow_store_instance` | `env: Env, caller: Address, request: StowRequest` | `Result<StowResponse, Error>` | — |
| `stow_store_batch` | `env: Env, caller: Address, requests: Vec<StowRequest>` | `Result<Vec<StowResponse>, Error>` | — |
| `cache_set` | `env: Env, caller: Address, key: BytesN<32>, data: Bytes` | `Result<bool, Error>` | — |
| `cache_get` | `env: Env, key: BytesN<32>` | `Result<Bytes, Error>` | — |
| `cache_invalidate` | `env: Env, caller: Address, key: BytesN<32>` | `Result<bool, Error>` | — |
| `get_study` | `env: Env, study_instance_uid: String` | `Option<DicomwebStudy>` | — |
| `get_series` | `env: Env, study_instance_uid: String, series_instance_uid: String` | `Option<DicomwebSeries>` | — |
| `get_instance` | `env: Env, study_instance_uid: String, series_instance_uid: String, sop_instance_uid: String` | `Option<DicomwebInstance>` | — |
| `get_instance_by_sop` | `env: Env, sop_instance_uid: String` | `Option<DicomwebInstance>` | — |
| `list_studies` | `env: Env` | `Vec<String>` | — |
| `get_concurrency_stats` | `env: Env` | `ConcurrencyTracker` | — |

### Types

#### `enum DicomwebServiceType`

| Variant | Value | Description |
|---|---|---|
| `Qido` | — | — |
| `Wado` | — | — |
| `Stow` | — | — |

#### `enum QueryLevel`

| Variant | Value | Description |
|---|---|---|
| `Study` | — | — |
| `Series` | — | — |
| `Instance` | — | — |

#### `enum TransferSyntax`

| Variant | Value | Description |
|---|---|---|
| `ExplicitVrLittleEndian` | — | — |
| `ImplicitVrLittleEndian` | — | — |
| `Jpeg2000Lossless` | — | — |
| `Jpeg2000Lossy` | — | — |
| `JpegBaseline` | — | — |
| `JpegLossless` | — | — |
| `RleLossless` | — | — |
| `Custom(u32)` | — | — |

#### `struct DicomJsonAttribute`

| Field | Type | Description |
|---|---|---|
| `tag` | `Symbol` | — |
| `vr` | `String` | — |
| `value` | `Vec<String>` | — |

#### `struct DicomJsonObject`

| Field | Type | Description |
|---|---|---|
| `attributes` | `Map<Symbol, DicomJsonAttribute>` | — |

#### `struct DicomwebQueryParams`

| Field | Type | Description |
|---|---|---|
| `study_instance_uid` | `Option<String>` | — |
| `series_instance_uid` | `Option<String>` | — |
| `sop_instance_uid` | `Option<String>` | — |
| `patient_id` | `Option<String>` | — |
| `patient_name` | `Option<String>` | — |
| `modality` | `Option<String>` | — |
| `study_date_from` | `Option<u64>` | — |
| `study_date_to` | `Option<u64>` | — |
| `body_part` | `Option<String>` | — |
| `limit` | `u32` | — |
| `offset` | `u32` | — |

#### `struct DicomwebStudy`

| Field | Type | Description |
|---|---|---|
| `study_instance_uid` | `String` | — |
| `patient_id` | `String` | — |
| `patient_name` | `String` | — |
| `study_date` | `u64` | — |
| `study_description` | `String` | — |
| `modalities_in_study` | `Vec<String>` | — |
| `number_of_series` | `u32` | — |
| `number_of_instances` | `u32` | — |
| `json_metadata` | `DicomJsonObject` | — |

#### `struct DicomwebSeries`

| Field | Type | Description |
|---|---|---|
| `study_instance_uid` | `String` | — |
| `series_instance_uid` | `String` | — |
| `modality` | `String` | — |
| `series_description` | `String` | — |
| `body_part` | `String` | — |
| `number_of_instances` | `u32` | — |
| `json_metadata` | `DicomJsonObject` | — |

#### `struct DicomwebInstance`

| Field | Type | Description |
|---|---|---|
| `study_instance_uid` | `String` | — |
| `series_instance_uid` | `String` | — |
| `sop_instance_uid` | `String` | — |
| `sop_class_uid` | `String` | — |
| `instance_number` | `u32` | — |
| `rows` | `u32` | — |
| `columns` | `u32` | — |
| `bits_allocated` | `u32` | — |
| `transfer_syntax` | `TransferSyntax` | — |
| `json_metadata` | `DicomJsonObject` | — |

#### `struct DicomwebBulkData`

| Field | Type | Description |
|---|---|---|
| `sop_instance_uid` | `String` | — |
| `data_reference` | `String` | — |
| `data_hash` | `BytesN<32>` | — |
| `size_bytes` | `u64` | — |
| `transfer_syntax` | `TransferSyntax` | — |
| `retrieved_at` | `u64` | — |

#### `struct StowRequest`

| Field | Type | Description |
|---|---|---|
| `study_instance_uid` | `String` | — |
| `series_instance_uid` | `String` | — |
| `sop_instance_uid` | `String` | — |
| `sop_class_uid` | `String` | — |
| `transfer_syntax` | `TransferSyntax` | — |
| `data_reference` | `String` | — |
| `data_hash` | `BytesN<32>` | — |
| `size_bytes` | `u64` | — |
| `json_metadata` | `DicomJsonObject` | — |

#### `struct StowResponse`

| Field | Type | Description |
|---|---|---|
| `sop_instance_uid` | `String` | — |
| `success` | `bool` | — |
| `error_message` | `Option<String>` | — |
| `stored_at` | `u64` | — |

#### `struct CacheEntry`

| Field | Type | Description |
|---|---|---|
| `key` | `BytesN<32>` | — |
| `data` | `Bytes` | — |
| `created_at` | `u64` | — |
| `expires_at` | `u64` | — |
| `hit_count` | `u32` | — |

#### `struct ConcurrencyTracker`

| Field | Type | Description |
|---|---|---|
| `active_requests` | `u32` | — |
| `total_requests` | `u64` | — |
| `last_reset` | `u64` | — |

#### `enum DataKey`

| Variant | Value | Description |
|---|---|---|
| `Admin` | — | — |
| `Paused` | — | — |
| `MedicalImagingContract` | — | — |
| `Study(String)` | — | — |
| `StudyIds` | — | — |
| `Series(String, String)` | — | — |
| `Instance(String, String, String)` | — | — |
| `InstanceBySop(String)` | — | — |
| `BulkData(String)` | — | — |
| `Cache(BytesN<32>)` | — | — |
| `Concurrency` | — | — |
| `QueryIndex(String)` | — | — |
| `MetadataIndex(String)` | — | — |
| `TransferSyntaxIndex(String)` | — | — |

#### `enum Error`

| Variant | Value | Description |
|---|---|---|
| `AlreadyInitialized` | 1 | — |
| `NotInitialized` | 2 | — |
| `NotAuthorized` | 3 | — |
| `ContractPaused` | 4 | — |
| `InvalidInput` | 5 | — |
| `StudyNotFound` | 6 | — |
| `SeriesNotFound` | 7 | — |
| `InstanceNotFound` | 8 | — |
| `BulkDataNotFound` | 9 | — |
| `CacheMiss` | 10 | — |
| `ConcurrencyLimitExceeded` | 11 | — |
| `InvalidTransferSyntax` | 12 | — |
| `InvalidDicomJson` | 13 | — |
| `StorageError` | 14 | — |
| `QueryError` | 15 | — |

#### `enum QueryLevel`

| Variant | Value | Description |
|---|---|---|
| `Study` | — | — |
| `Series` | — | — |
| `Instance` | — | — |

#### `enum TransferSyntax`

| Variant | Value | Description |
|---|---|---|
| `ExplicitVrLittleEndian` | — | — |
| `ImplicitVrLittleEndian` | — | — |
| `Jpeg2000Lossless` | — | — |
| `Jpeg2000Lossy` | — | — |
| `JpegBaseline` | — | — |
| `JpegLossless` | — | — |
| `RleLossless` | — | — |
| `Custom(u32)` | — | — |

#### `struct DicomJsonAttribute`

| Field | Type | Description |
|---|---|---|
| `tag` | `Symbol` | — |
| `vr` | `String` | — |
| `value` | `Vec<String>` | — |

#### `struct DicomJsonObject`

| Field | Type | Description |
|---|---|---|
| `attributes` | `Map<Symbol, DicomJsonAttribute>` | — |

#### `struct DicomwebQueryParams`

| Field | Type | Description |
|---|---|---|
| `study_instance_uid` | `Option<String>` | — |
| `series_instance_uid` | `Option<String>` | — |
| `sop_instance_uid` | `Option<String>` | — |
| `patient_id` | `Option<String>` | — |
| `patient_name` | `Option<String>` | — |
| `modality` | `Option<String>` | — |
| `study_date_from` | `Option<u64>` | — |
| `study_date_to` | `Option<u64>` | — |
| `body_part` | `Option<String>` | — |
| `limit` | `u32` | — |
| `offset` | `u32` | — |

#### `struct DicomwebStudy`

| Field | Type | Description |
|---|---|---|
| `study_instance_uid` | `String` | — |
| `patient_id` | `String` | — |
| `patient_name` | `String` | — |
| `study_date` | `u64` | — |
| `study_description` | `String` | — |
| `modalities_in_study` | `Vec<String>` | — |
| `number_of_series` | `u32` | — |
| `number_of_instances` | `u32` | — |
| `json_metadata` | `DicomJsonObject` | — |

#### `struct DicomwebSeries`

| Field | Type | Description |
|---|---|---|
| `study_instance_uid` | `String` | — |
| `series_instance_uid` | `String` | — |
| `modality` | `String` | — |
| `series_description` | `String` | — |
| `body_part` | `String` | — |
| `number_of_instances` | `u32` | — |
| `json_metadata` | `DicomJsonObject` | — |

#### `struct DicomwebInstance`

| Field | Type | Description |
|---|---|---|
| `study_instance_uid` | `String` | — |
| `series_instance_uid` | `String` | — |
| `sop_instance_uid` | `String` | — |
| `sop_class_uid` | `String` | — |
| `instance_number` | `u32` | — |
| `rows` | `u32` | — |
| `columns` | `u32` | — |
| `bits_allocated` | `u32` | — |
| `transfer_syntax` | `TransferSyntax` | — |
| `json_metadata` | `DicomJsonObject` | — |

#### `struct DicomwebBulkData`

| Field | Type | Description |
|---|---|---|
| `sop_instance_uid` | `String` | — |
| `data_reference` | `String` | — |
| `data_hash` | `BytesN<32>` | — |
| `size_bytes` | `u64` | — |
| `transfer_syntax` | `TransferSyntax` | — |
| `retrieved_at` | `u64` | — |

#### `struct StowRequest`

| Field | Type | Description |
|---|---|---|
| `study_instance_uid` | `String` | — |
| `series_instance_uid` | `String` | — |
| `sop_instance_uid` | `String` | — |
| `sop_class_uid` | `String` | — |
| `transfer_syntax` | `TransferSyntax` | — |
| `data_reference` | `String` | — |
| `data_hash` | `BytesN<32>` | — |
| `size_bytes` | `u64` | — |
| `json_metadata` | `DicomJsonObject` | — |

#### `struct StowResponse`

| Field | Type | Description |
|---|---|---|
| `sop_instance_uid` | `String` | — |
| `success` | `bool` | — |
| `error_message` | `Option<String>` | — |
| `stored_at` | `u64` | — |

#### `struct CacheEntry`

| Field | Type | Description |
|---|---|---|
| `key` | `BytesN<32>` | — |
| `data` | `Bytes` | — |
| `created_at` | `u64` | — |
| `expires_at` | `u64` | — |
| `hit_count` | `u32` | — |

#### `struct ConcurrencyTracker`

| Field | Type | Description |
|---|---|---|
| `active_requests` | `u32` | — |
| `total_requests` | `u64` | — |
| `last_reset` | `u64` | — |

#### `enum DataKey`

| Variant | Value | Description |
|---|---|---|
| `Admin` | — | — |
| `Paused` | — | — |
| `MedicalImagingContract` | — | — |
| `Study(String)` | — | — |
| `StudyIds` | — | — |
| `Series(String, String)` | — | — |
| `Instance(String, String, String)` | — | — |
| `InstanceBySop(String)` | — | — |
| `BulkData(String)` | — | — |
| `Cache(BytesN<32>)` | — | — |
| `Concurrency` | — | — |
| `QueryIndex(String)` | — | — |
| `MetadataIndex(String)` | — | — |
| `TransferSyntaxIndex(String)` | — | — |

#### `enum Error`

| Variant | Value | Description |
|---|---|---|
| `AlreadyInitialized` | 1 | — |
| `NotInitialized` | 2 | — |
| `NotAuthorized` | 3 | — |
| `ContractPaused` | 4 | — |
| `InvalidInput` | 5 | — |
| `StudyNotFound` | 6 | — |
| `SeriesNotFound` | 7 | — |
| `InstanceNotFound` | 8 | — |
| `BulkDataNotFound` | 9 | — |
| `CacheMiss` | 10 | — |
| `ConcurrencyLimitExceeded` | 11 | — |
| `InvalidTransferSyntax` | 12 | — |
| `InvalidDicomJson` | 13 | — |
| `StorageError` | 14 | — |
| `QueryError` | 15 | — |

### Examples

#### `test_initialize`

```rust
let env = Env::default();
    let contract_id = env.register_contract(None, DicomwebServicesContract);
    let client = DicomwebServicesContractClient::new(&env, &contract_id);

    let admin = Address::random(&env);
    let medical_imaging = Address::random(&env);

    let result = client.initialize(&admin, &medical_imaging);
    assert!(result.is_ok());
```

#### `test_initialize_already_initialized`

```rust
let env = Env::default();
    let contract_id = env.register_contract(None, DicomwebServicesContract);
    let client = DicomwebServicesContractClient::new(&env, &contract_id);

    let admin = Address::random(&env);
    let medical_imaging = Address::random(&env);

    client.initialize(&admin, &medical_imaging).unwrap();
```

#### `test_set_paused`

```rust
let env = Env::default();
    let contract_id = env.register_contract(None, DicomwebServicesContract);
    let client = DicomwebServicesContractClient::new(&env, &contract_id);

    let admin = Address::random(&env);
    let medical_imaging = Address::random(&env);

    client.initialize(&admin, &medical_imaging).unwrap();
```

---

## differential_privacy

### Functions

| Function | Parameters | Returns | Description |
|---|---|---|---|
| `initialize` | `env: Env, admin: Address` | `Result<(), Error>` | — |
| `create_budget` | `env: Env, admin: Address, owner: Address, epsilon_total: u64` | `Result<BytesN<32>, Error>` | Create a new privacy budget |
| `add_laplace_noise` | `env: Env, caller: Address, budget_id: BytesN<32>, query_id: BytesN<32>, data_type: DataType, true_value: i64, sensitivity: u64` | `Result<PrivacyQuery, Error>` | Add Laplace noise for ε-differential privacy |
| `add_gaussian_noise` | `env: Env, caller: Address, budget_id: BytesN<32>, query_id: BytesN<32>, data_type: DataType, true_value: i64, sensitivity: u64` | `Result<PrivacyQuery, Error>` | Add Gaussian noise for differential privacy |
| `get_remaining_budget` | `env: Env, budget_id: BytesN<32>` | `Result<u64, Error>` | Get remaining budget |
| `get_query` | `env: Env, query_id: BytesN<32>` | `Option<PrivacyQuery>` | Get query by ID |
| `deactivate_budget` | `env: Env, admin: Address, budget_id: BytesN<32>` | `Result<(), Error>` | Deactivate a privacy budget |

### Types

#### `enum NoiseMechanism`

| Variant | Value | Description |
|---|---|---|
| `Laplace` | — | — |
| `Gaussian` | — | — |

#### `enum DataType`

| Variant | Value | Description |
|---|---|---|
| `Numerical` | — | — |
| `Categorical` | — | — |
| `Count` | — | — |

#### `struct PrivacyBudget`

| Field | Type | Description |
|---|---|---|
| `budget_id` | `BytesN<32>` | — |
| `owner` | `Address` | — |
| `epsilon_remaining` | `u64` | — |
| `is_active` | `bool` | — |

#### `struct PrivacyQuery`

| Field | Type | Description |
|---|---|---|
| `query_id` | `BytesN<32>` | — |
| `budget_id` | `BytesN<32>` | — |
| `data_type` | `DataType` | — |
| `mechanism` | `NoiseMechanism` | — |
| `true_result` | `i64` | — |
| `noisy_result` | `i64` | — |
| `epsilon_cost` | `u64` | — |
| `timestamp` | `u64` | — |

#### `enum DataKey`

| Variant | Value | Description |
|---|---|---|
| `Initialized` | — | — |
| `Admin` | — | — |
| `Budget(BytesN<32>)` | — | — |
| `Query(BytesN<32>)` | — | — |
| `BudgetCounter` | — | — |
| `QueryCounter` | — | — |

#### `enum Error`

| Variant | Value | Description |
|---|---|---|
| `AlreadyInitialized` | 1 | — |
| `NotInitialized` | 2 | — |
| `NotAuthorized` | 3 | — |
| `BudgetNotFound` | 4 | — |
| `BudgetExhausted` | 5 | — |
| `BudgetNotActive` | 6 | — |
| `QueryNotFound` | 7 | — |
| `InvalidSensitivity` | 8 | — |
| `InsufficientBudget` | 9 | — |
| `InvalidInput` | 10 | — |
| `ArithmeticOverflow` | 11 | — |

### Examples

#### `test_initialize_sets_admin`

```rust
let (_env, client, admin) = setup();

    // Double initialization should fail
    let result = client.try_initialize(&admin);
    assert!(result.is_err());
```

#### `test_initialize_unauthorized`

```rust
let env = Env::default();
    env.mock_all_auths();
    let id = Address::generate(&env);
    env.register_contract(&id, DifferentialPrivacyContract);
    let client = DifferentialPrivacyContractClient::new(&env, &id);
    let admin = Address::generate(&env);
    let other = Address::generate(&env);

    client.initialize(&admin);
```

#### `test_create_budget`

```rust
let (env, client, admin) = setup();
    let data_owner = Address::generate(&env);

    let budget_id = client.create_budget(&admin, &data_owner, &100);
    assert!(!budget_id.is_empty());

    let remaining = client.get_remaining_budget(&budget_id);
    assert_eq!(remaining, 100);
```

---

## digital_twin

### Functions

| Function | Parameters | Returns | Description |
|---|---|---|---|
| `initialize` | `env: Env, admin: Address` | `Result<bool, Error>` | — |
| `set_medical_records_contract` | `env: Env, admin: Address, contract_id: Address` | `Result<bool, Error>` | — |
| `set_genomic_data_contract` | `env: Env, admin: Address, contract_id: Address` | `Result<bool, Error>` | — |
| `create_digital_twin` | `env: Env, patient: Address, initial_data_sources: Vec<DataSource>, model_types: Vec<ModelType>, sync_frequency: u32` | `Result<u64, Error>` | — |
| `update_digital_twin_status` | `env: Env, admin: Address, twin_id: u64, new_status: TwinStatus` | `Result<bool, Error>` | — |
| `add_data_stream` | `env: Env, patient: Address, twin_id: u64, source: DataSource, data_type: DataType, provider: Address, stream_ref: String, update_frequency: u32` | `Result<u64, Error>` | — |
| `add_data_point` | `env: Env, provider: Address, stream_id: u64, value: String, confidence: u32, metadata: Map<String, String>` | `Result<bool, Error>` | — |
| `add_predictive_model` | `env: Env, admin: Address, twin_id: u64, model_type: ModelType, model_ref: String` | `Result<u64, Error>` | — |
| `generate_prediction` | `env: Env, model_id: u64, input_data: String, prediction_type: String` | `Result<u64, Error>` | — |
| `create_simulation` | `env: Env, twin_id: u64, simulation_type: SimulationType, parameters: Map<String, String>, created_by: Address` | `Result<u64, Error>` | — |
| `complete_simulation` | `env: Env, simulation_id: u64, results: Map<String, String>, confidence: u32` | `Result<bool, Error>` | — |
| `create_research_snapshot` | `env: Env, researcher: Address, twin_id: u64, data_types: Vec<DataType>, privacy_level: u32, anonymization_method: String, duration_hours: u32` | `Result<u64, Error>` | — |
| `sync_with_medical_records` | `env: Env, twin_id: u64` | `Result<SyncStatus, Error>` | — |
| `get_digital_twin` | `env: Env, twin_id: u64` | `Result<DigitalTwinProfile, Error>` | — |
| `get_twin_by_patient` | `env: Env, patient: Address` | `Result<u64, Error>` | — |
| `get_data_stream` | `env: Env, stream_id: u64` | `Result<DataStream, Error>` | — |
| `get_data_points` | `env: Env, stream_id: u64, limit: u32` | `Result<Vec<DataPoint>, Error>` | — |
| `get_predictive_model` | `env: Env, model_id: u64` | `Result<PredictiveModel, Error>` | — |
| `get_prediction` | `env: Env, prediction_id: u64` | `Result<Prediction, Error>` | — |
| `get_simulation` | `env: Env, simulation_id: u64` | `Result<Simulation, Error>` | — |
| `get_research_snapshot` | `env: Env, snapshot_id: u64` | `Result<ResearchSnapshot, Error>` | — |
| `get_global_stats` | `env: Env` | `Result<Map<String, u64>, Error>` | — |

### Types

#### `enum TwinStatus`

| Variant | Value | Description |
|---|---|---|
| `Initializing` | — | — |
| `Active` | — | — |
| `Syncing` | — | — |
| `Simulation` | — | — |
| `Archived` | — | — |

#### `enum DataSource`

| Variant | Value | Description |
|---|---|---|
| `MedicalRecords` | — | — |
| `GenomicData` | — | — |
| `Wearables` | — | — |
| `EMR` | — | — |
| `LabResults` | — | — |
| `Imaging` | — | — |
| `PatientReported` | — | — |
| `External` | — | — |

#### `enum DataType`

| Variant | Value | Description |
|---|---|---|
| `VitalSigns` | — | — |
| `LabResults` | — | — |
| `Genomic` | — | — |
| `Imaging` | — | — |
| `Medications` | — | — |
| `Procedures` | — | — |
| `Symptoms` | — | — |
| `Activity` | — | — |
| `Sleep` | — | — |
| `Nutrition` | — | — |
| `Environmental` | — | — |
| `Social` | — | — |

#### `enum ModelType`

| Variant | Value | Description |
|---|---|---|
| `Predictive` | — | — |
| `Simulation` | — | — |
| `RiskAssessment` | — | — |
| `TreatmentResponse` | — | — |
| `DiseaseProgression` | — | — |
| `Wellness` | — | — |

#### `enum SimulationType`

| Variant | Value | Description |
|---|---|---|
| `Treatment` | — | — |
| `Lifestyle` | — | — |
| `Environmental` | — | — |
| `Medication` | — | — |
| `Surgical` | — | — |
| `Preventive` | — | — |

#### `struct DigitalTwinProfile`

| Field | Type | Description |
|---|---|---|
| `twin_id` | `u64` | — |
| `patient_id` | `Address` | — |
| `created_at` | `u64` | — |
| `updated_at` | `u64` | — |
| `status` | `TwinStatus` | — |
| `accuracy_score` | `u32` | — |
| `completeness_score` | `u32` | — |
| `sync_frequency` | `u32` | — |
| `last_sync` | `u64` | — |
| `data_sources` | `Vec<DataSource>` | — |
| `model_types` | `Vec<ModelType>` | — |
| `consent_version` | `u32` | — |
| `research_consent` | `bool` | — |

#### `struct DataStream`

| Field | Type | Description |
|---|---|---|
| `stream_id` | `u64` | — |
| `twin_id` | `u64` | — |
| `source` | `DataSource` | — |
| `data_type` | `DataType` | — |
| `provider` | `Address` | — |
| `stream_ref` | `String` | — |
| `last_update` | `u64` | — |
| `update_frequency` | `u32` | — |
| `quality_score` | `u32` | — |
| `is_active` | `bool` | — |
| `encryption_key_id` | `Option<BytesN<32>>` | — |

#### `struct DataPoint`

| Field | Type | Description |
|---|---|---|
| `timestamp` | `u64` | — |
| `value` | `String` | — |
| `confidence` | `u32` | — |
| `source_id` | `u64` | — |
| `verification_hash` | `BytesN<32>` | — |
| `metadata` | `Map<String, String>` | — |

#### `struct PredictiveModel`

| Field | Type | Description |
|---|---|---|
| `model_id` | `u64` | — |
| `twin_id` | `u64` | — |
| `model_type` | `ModelType` | — |
| `model_ref` | `String` | — |
| `version` | `u32` | — |
| `accuracy` | `u32` | — |
| `last_trained` | `u64` | — |
| `training_data_points` | `u32` | — |
| `validation_score` | `u32` | — |
| `is_active` | `bool` | — |

#### `struct Prediction`

| Field | Type | Description |
|---|---|---|
| `prediction_id` | `u64` | — |
| `model_id` | `u64` | — |
| `timestamp` | `u64` | — |
| `prediction_type` | `String` | — |
| `confidence` | `u32` | — |
| `result` | `String` | — |
| `input_data_hash` | `BytesN<32>` | — |
| `explanation_ref` | `Option<String>` | — |
| `risk_level` | `u32` | — |

#### `struct Simulation`

| Field | Type | Description |
|---|---|---|
| `simulation_id` | `u64` | — |
| `twin_id` | `u64` | — |
| `simulation_type` | `SimulationType` | — |
| `parameters` | `Map<String, String>` | — |
| `start_time` | `u64` | — |
| `end_time` | `u64` | — |
| `results` | `Map<String, String>` | — |
| `confidence` | `u32` | — |
| `created_by` | `Address` | — |
| `is_complete` | `bool` | — |

#### `struct SyncStatus`

| Field | Type | Description |
|---|---|---|
| `twin_id` | `u64` | — |
| `source` | `DataSource` | — |
| `last_sync` | `u64` | — |
| `sync_success` | `bool` | — |
| `records_synced` | `u32` | — |
| `errors` | `Vec<String>` | — |
| `accuracy_delta` | `i32` | — |

#### `struct ResearchSnapshot`

| Field | Type | Description |
|---|---|---|
| `snapshot_id` | `u64` | — |
| `twin_id` | `u64` | — |
| `researcher` | `Address` | — |
| `created_at` | `u64` | — |
| `expires_at` | `u64` | — |
| `data_types` | `Vec<DataType>` | — |
| `privacy_level` | `u32` | — |
| `anonymization_method` | `String` | — |
| `snapshot_hash` | `BytesN<32>` | — |
| `access_count` | `u32` | — |

#### `enum DataKey`

| Variant | Value | Description |
|---|---|---|
| `Admin` | — | — |
| `MedicalRecordsContract` | — | — |
| `GenomicDataContract` | — | — |
| `NextTwinId` | — | — |
| `Twin(u64)` | — | — |
| `TwinByPatient(Address)` | — | — |
| `TwinDataStreams(u64)` | — | — |
| `DataStream(u64)` | — | — |
| `StreamDataPoints(u64)` | — | — |
| `NextStreamId,
    NextModelId,
    PredictiveModel(u64),
    TwinModels(u64),
    NextPredictionId,
    Prediction(u64),
    ModelPredictions(u64),
    NextSimulationId,
    Simulation(u64),
    TwinSimulations(u64),
    SyncStatus(u64, DataSource),
    NextSnapshotId,
    ResearchSnapshot(u64),
    TwinSnapshots(u64),
    AccuracyMetrics(u64),
    GlobalStats,` | — | — |

#### `enum Error`

| Variant | Value | Description |
|---|---|---|
| `NotAuthorized` | 1 | — |
| `NotInitialized` | 2 | — |
| `AlreadyInitialized` | 3 | — |
| `TwinNotFound` | 4 | — |
| `InvalidStatus` | 5 | — |
| `DataStreamNotFound` | 6 | — |
| `ModelNotFound` | 7 | — |
| `SimulationNotFound` | 8 | — |
| `InvalidParameter` | 9 | — |
| `InsufficientAccuracy` | 10 | — |
| `SyncInProgress` | 11 | — |
| `ResearchAccessDenied` | 12 | — |
| `SnapshotExpired` | 13 | — |
| `DuplicateDataStream` | 14 | — |
| `ModelNotActive` | 15 | — |
| `SimulationInvalid` | 16 | — |
| `PrivacyLevelInsufficient` | 17 | — |
| `ConsentRequired` | 18 | — |
| `ContractNotSet` | 19 | — |

---

## dispute_resolution

### Functions

| Function | Parameters | Returns | Description |
|---|---|---|---|
| `initialize` | `env: Env, arbiters: Vec<Address>` | `()` | — |
| `dispute` | `env: Env, proposal_id: u64, challenger: Address` | `()` | — |
| `resolve` | `env: Env, proposal_id: u64, arbiter: Address, valid_proposal: bool` | `Result<(), Error>` | — |
| `is_disputed` | `env: Env, proposal_id: u64` | `bool` | — |

### Types

#### `enum Error`

| Variant | Value | Description |
|---|---|---|
| `NotInitialized` | 1 | — |
| `NotArbiter` | 2 | — |
| `DisputeNotFound` | 3 | — |

---

## drug_discovery

### Functions

| Function | Parameters | Returns | Description |
|---|---|---|---|
| `initialize` | `env: Env, admin: Address, analyzer: Address, predictor: Address` | `Result<(), Error>` | — |
| `configure_integrations` | `env: Env, caller: Address, genomic_contract: Option<Address>, clinical_trial_contract: Option<Address>, large_scale_mode: Option<bool>, quantum_enabled: Option<bool>` | `Result<(), Error>` | — |
| `register_molecule` | `env: Env, caller: Address, canonical_smiles: String, inchi_key: String, molecular_weight_milli: u32, h_bond_donors: u32, h_bond_acceptors: u32, rotatable_bonds: u32, fingerprint: Vec<u32>, database_refs: Vec<String>` | `Result<u64, Error>` | — |
| `analyze_molecular_structure` | `env: Env, caller: Address, molecule_id: u64` | `Result<StructureAnalysis, Error>` | — |
| `predict_drug_target_interaction` | `env: Env, caller: Address, molecule_id: u64, target_gene: String, binding_affinity_pico: u64, model_ref: String` | `Result<u64, Error>` | — |
| `predict_adverse_effects` | `env: Env, caller: Address, molecule_id: u64, effect_code: String, cohort_ref: String, severity_bps: u32` | `Result<u64, Error>` | — |
| `optimize_clinical_trial_matching` | `env: Env, caller: Address, molecule_id: u64, protocol_id: u64, genomic_record_id: Option<u64>` | `Result<u64, Error>` | — |
| `request_quantum_simulation` | `env: Env, caller: Address, molecule_id: u64, target_gene: String, algorithm: String, depth: u32, shots: u32` | `Result<u64, Error>` | — |
| `run_screening_campaign` | `env: Env, caller: Address, target_gene: String, candidate_molecule_ids: Vec<u64>, analysis_time_hours: u32, used_quantum: bool` | `Result<u64, Error>` | — |
| `get_config` | `env: Env` | `Result<PlatformConfig, Error>` | — |
| `get_molecule` | `env: Env, molecule_id: u64` | `Result<MolecularStructure, Error>` | — |
| `get_campaign_report` | `env: Env, campaign_id: u64` | `Result<ScreeningCampaignReport, Error>` | — |
| `get_suggestion` | `error: Error` | `Symbol` | — |

### Types

#### `struct PlatformConfig`

| Field | Type | Description |
|---|---|---|
| `admin` | `Address` | — |
| `analyzer` | `Address` | — |
| `predictor` | `Address` | — |
| `genomic_contract` | `Option<Address>` | — |
| `clinical_trial_contract` | `Option<Address>` | — |
| `large_scale_mode` | `bool` | — |
| `quantum_enabled` | `bool` | — |
| `min_candidate_accuracy_bps` | `u32` | — |
| `max_analysis_time_hours` | `u32` | — |

#### `struct MolecularStructure`

| Field | Type | Description |
|---|---|---|
| `molecule_id` | `u64` | — |
| `canonical_smiles` | `String` | — |
| `inchi_key` | `String` | — |
| `molecular_weight_milli` | `u32` | — |
| `h_bond_donors` | `u32` | — |
| `h_bond_acceptors` | `u32` | — |
| `rotatable_bonds` | `u32` | — |
| `fingerprint` | `Vec<u32>` | — |
| `database_refs` | `Vec<String>` | — |
| `created_at` | `u64` | — |

#### `struct StructureAnalysis`

| Field | Type | Description |
|---|---|---|
| `molecule_id` | `u64` | — |
| `lipinski_violations` | `u32` | — |
| `qed_score_bps` | `u32` | — |
| `synthetic_accessibility_bps` | `u32` | — |
| `novelty_score_bps` | `u32` | — |
| `created_at` | `u64` | — |

#### `struct DrugTargetPrediction`

| Field | Type | Description |
|---|---|---|
| `prediction_id` | `u64` | — |
| `molecule_id` | `u64` | — |
| `target_gene` | `String` | — |
| `binding_affinity_pico` | `u64` | — |
| `confidence_bps` | `u32` | — |
| `model_ref` | `String` | — |
| `created_at` | `u64` | — |

#### `struct AdverseEffectPrediction`

| Field | Type | Description |
|---|---|---|
| `adverse_id` | `u64` | — |
| `molecule_id` | `u64` | — |
| `effect_code` | `String` | — |
| `severity_bps` | `u32` | — |
| `probability_bps` | `u32` | — |
| `cohort_ref` | `String` | — |
| `created_at` | `u64` | — |

#### `struct TrialMatchResult`

| Field | Type | Description |
|---|---|---|
| `match_id` | `u64` | — |
| `molecule_id` | `u64` | — |
| `protocol_id` | `u64` | — |
| `fit_score_bps` | `u32` | — |
| `expected_enrollment_days` | `u32` | — |
| `matched_at` | `u64` | — |

#### `struct QuantumSimulationRequest`

| Field | Type | Description |
|---|---|---|
| `simulation_id` | `u64` | — |
| `molecule_id` | `u64` | — |
| `target_gene` | `String` | — |
| `algorithm` | `String` | — |
| `depth` | `u32` | — |
| `shots` | `u32` | — |
| `queued_at` | `u64` | — |

#### `struct ScreeningCampaignReport`

| Field | Type | Description |
|---|---|---|
| `campaign_id` | `u64` | — |
| `target_gene` | `String` | — |
| `screened_candidates` | `u32` | — |
| `identified_candidates` | `u32` | — |
| `candidate_accuracy_bps` | `u32` | — |
| `analysis_time_hours` | `u32` | — |
| `used_quantum` | `bool` | — |
| `completed_at` | `u64` | — |

#### `enum DataKey`

| Variant | Value | Description |
|---|---|---|
| `Config` | — | — |
| `Molecule(u64)` | — | — |
| `MoleculeCount` | — | — |
| `Analysis(u64)` | — | — |
| `Prediction(u64)` | — | — |
| `PredictionCount` | — | — |
| `AdversePrediction(u64)` | — | — |
| `AdverseCount` | — | — |
| `TrialMatch(u64)` | — | — |
| `MatchCount` | — | — |
| `QuantumRequest(u64)` | — | — |
| `QuantumCount` | — | — |
| `CampaignReport(u64)` | — | — |
| `CampaignCount` | — | — |

#### `enum Error`

| Variant | Value | Description |
|---|---|---|
| `NotAuthorized` | 1 | — |
| `NotInitialized` | 2 | — |
| `InvalidInput` | 3 | — |
| `MoleculeNotFound` | 4 | — |
| `PredictionNotFound` | 5 | — |
| `BenchmarkNotMet` | 6 | — |
| `IntegrationMissing` | 7 | — |
| `QuantumDisabled` | 8 | — |
| `AlreadyInitialized` | 9 | — |

#### `struct TrialProtocolView`

| Field | Type | Description |
|---|---|---|
| `id` | `u64` | — |
| `title` | `String` | — |
| `version` | `u32` | — |
| `sponsor` | `Address` | — |
| `created_at` | `u64` | — |
| `active` | `bool` | — |
| `metadata_ref` | `String` | — |

### Examples

#### `test_molecular_analysis_prediction_and_adverse_effects`

```rust
let env = Env::default();
    let (client, _admin, analyzer, predictor) = setup(&env);

    let fingerprint = Vec::from_array(&env, [13u32, 37u32, 101u32, 211u32]);
    let db_refs = Vec::from_array(
        &env,
        [
            String::from_str(&env, "pubchem:2244"),
            String::from_str(&env, "chembl:25"),
```

#### `test_screening_campaign_benchmark_enforcement`

```rust
let env = Env::default();
    let (client, _admin, analyzer, predictor) = setup(&env);

    let fp = Vec::from_array(&env, [1u32, 2u32, 3u32]);
    let refs = Vec::from_array(&env, [String::from_str(&env, "db:a")]);

    let m1 = client.register_molecule(
        &analyzer,
        &String::from_str(&env, "NCC(=O)O"),
```

#### `test_quantum_simulation_guardrails`

```rust
let env = Env::default();
    let (client, admin, analyzer, predictor) = setup(&env);

    let fp = Vec::from_array(&env, [7u32, 8u32, 9u32]);
    let refs = Vec::from_array(&env, [String::from_str(&env, "db:b")]);
    let molecule_id = client.register_molecule(
        &analyzer,
        &String::from_str(&env, "C1=CC=CC=C1"),
        &String::from_str(&env, "UHOVQNZJYSORNB-UHFFFAOYSA-N"),
```

---

## emergency_access_override

### Functions

| Function | Parameters | Returns | Description |
|---|---|---|---|
| `initialize` | `env: Env, admin: Address, approvers: Vec<Address>, threshold: u32` | `Result<(), Error>` | — |
| `grant_emergency_access` | `env: Env, approver: Address, patient: Address, provider: Address, duration_seconds: u64` | `Result<bool, Error>` | — |
| `reset_circuit_breaker` | `env: Env, admin: Address` | `Result<(), Error>` | Reset the circuit breaker. Only callable by admin after investigation. |
| `update_cooldown_period` | `env: Env, admin: Address, new_period_seconds: u64` | `Result<(), Error>` | Update the cooldown period. Only callable by admin (governance-gated). |
| `get_cooldown_period` | `env: Env` | `u64` | Get the current cooldown period in seconds. |
| `check_emergency_access` | `env: Env, patient: Address, provider: Address` | `Result<bool, Error>` | — |
| `revoke_emergency_access` | `env: Env, admin: Address, patient: Address, provider: Address` | `Result<(), Error>` | — |
| `get_emergency_access_record` | `env: Env, patient: Address, provider: Address` | `Option<EmergencyAccessRecord>` | — |
| `get_admin` | `env: Env` | `Result<Address, Error>` | — |
| `health_check` | `env: Env` | `bool` | On-chain health check endpoint. Returns true if the contract is initialized and operational. |
| `configure_multisig` | `env: Env, admin: Address, approvers: Vec<Address>, required_approvals: u32, expiry_seconds: u64` | `()` | Governance sets the approver set and required M. |
| `request_emergency_access` | `env: Env, requester: Address, patient_id: Symbol, reason: Symbol` | `u64` | Any party creates a pending emergency access request. |
| `approve_emergency_access` | `env: Env, approver: Address, request_id: u64` | `Result<bool, Error>` | An approver signs off on a pending request. Access is granted automatically once M approvals are collected. |
| `get_emergency_request` | `env: Env, request_id: u64` | `Option<EmergencyRequest>` | Read a request's current state. |

### Types

#### `struct EmergencyAccessRecord`

| Field | Type | Description |
|---|---|---|
| `patient` | `Address` | — |
| `provider` | `Address` | — |
| `requested_duration` | `u64` | — |
| `granted_at` | `u64` | — |
| `expiry_at` | `u64` | — |
| `approved` | `bool` | — |
| `approvers` | `Vec<Address>` | — |

#### `enum DataKey`

| Variant | Value | Description |
|---|---|---|
| `Initialized` | — | — |
| `Admin` | — | — |
| `ApprovalThreshold` | — | — |
| `TrustedApprovers` | — | — |
| `EmergencyAccess(Address, Address)` | — | — |
| `Cooldown(Address)` | — | — |
| `CooldownPeriod,                    
    GlobalGrantCount,                  
    GlobalGrantWindowStart,            
    CircuitBreakerTripped,` | — | — |

#### `struct EmergencyRequest`

| Field | Type | Description |
|---|---|---|
| `patient_id` | `Symbol` | — |
| `reason` | `Symbol` | — |
| `requester` | `Address` | — |
| `approvals` | `Vec<Address>` | — |
| `created_at` | `u64` | — |
| `granted` | `bool` | — |

#### `enum EmergencyKey`

| Variant | Value | Description |
|---|---|---|
| `Request(u64)` | — | — |
| `Config` | — | — |
| `RequestCounter` | — | — |

#### `struct MultiSigConfig`

| Field | Type | Description |
|---|---|---|
| `approvers` | `Vec<Address>` | — |
| `required_approvals` | `u32` | — |
| `expiry_seconds` | `u64` | — |

### Error Codes

| Variant | Code | Description |
|---|---|---|
| `Unauthorized` | 100 | — |
| `NotInitialized` | 300 | — |
| `AlreadyInitialized` | 301 | — |
| `InvalidThreshold` | 230 | — |
| `InvalidDuration` | 231 | — |
| `RecordNotFound` | 403 | — |
| `RateLimitExceeded` | 429 | — |

### Examples

#### `test_initialize`

```rust
let (_env, client, admin, _, _, _, approvers) = setup();
        client.initialize(&admin, &approvers, &2);
    }

    #[test]
    fn test_initialize_threshold_invalid() {
        let (_env, client, admin, _, _, _, approvers) = setup();
        let result = client.try_initialize(&admin, &approvers, &0);
        assert_eq!(result, Err(Ok(Error::InvalidThreshold)));
```

---

## emr_integration

### Functions

| Function | Parameters | Returns | Description |
|---|---|---|---|
| `initialize` | `env: Env, admin: Address, fhir_contract: Address` | `Result<bool, Error>` | — |
| `register_emr_system` | `env: Env, admin: Address, system_id: String, vendor_name: String, vendor_contact: String, system_version: String, supported_standards: Vec<String>, api_endpoints: Vec<String>` | `Result<bool, Error>` | — |
| `get_emr_system` | `env: Env, system_id: String` | `Result<EMRSystem, Error>` | — |
| `initiate_onboarding` | `env: Env, provider: Address, onboarding_id: String, provider_id: String, provider_name: String, provider_email: String, facility_name: String, npi: String, emr_system_id: String, compliance_checklist: Vec<String>` | `Result<bool, Error>` | — |
| `complete_onboarding` | `env: Env, admin: Address, onboarding_id: String, verification_id: String, license_number: String, license_state: String, license_expiration: String, board_certifications: Vec<String>, malpractice_insurance: String, background_check_id: String` | `Result<bool, Error>` | — |
| `get_onboarding_status` | `env: Env, onboarding_id: String` | `Result<ProviderOnboarding, Error>` | — |
| `get_provider_verification` | `env: Env, verification_id: String` | `Result<ProviderVerification, Error>` | — |
| `register_network_node` | `env: Env, admin: Address, node: NetworkNode` | `Result<bool, Error>` | — |
| `get_network_node` | `env: Env, node_id: String` | `Result<NetworkNode, Error>` | — |
| `register_interop_agreement` | `env: Env, admin: Address, agreement: InteroperabilityAgreement` | `Result<bool, Error>` | — |
| `get_interop_agreement` | `env: Env, agreement_id: String` | `Result<InteroperabilityAgreement, Error>` | — |
| `record_interop_test` | `env: Env, tester: Address, test: InteroperabilityTest` | `Result<bool, Error>` | — |
| `get_interop_test` | `env: Env, test_id: String` | `Result<InteroperabilityTest, Error>` | — |
| `parse_message` | `env: Env, sender: Address, message_id: String, source_system_id: String, encoding: CharacterEncoding, transport: TransportProtocol, content_type: String, payload: String` | `Result<HealthcareMessage, Error>` | — |
| `generate_message` | `env: Env, sender: Address, message_id: String, source_system_id: String, standard: MessagingStandard, version: String, message_type: String, encoding: CharacterEncoding, transport: TransportProtocol, content_type: String, metadata: Map<String, String>` | `Result<HealthcareMessage, Error>` | — |
| `transform_message` | `env: Env, sender: Address, transform_id: String, source_message_id: String, target_message_id: String, target_standard: MessagingStandard, target_version: String, target_message_type: String, target_encoding: CharacterEncoding, target_transport: TransportProtocol, target_content_type: String` | `Result<MessageTransformation, Error>` | — |
| `validate_message` | `env: Env, sender: Address, report_id: String, message_id: String` | `Result<MessageValidationReport, Error>` | — |
| `wrap_transport_payload` | `env: Env, message_id: String` | `Result<String, Error>` | — |
| `benchmark_message_processing` | `env: Env, benchmark_id: String, message_type: String, encoding: CharacterEncoding, transport: TransportProtocol, batch_size: u32` | `Result<ThroughputBenchmark, Error>` | — |
| `get_message` | `env: Env, message_id: String` | `Result<HealthcareMessage, Error>` | — |
| `get_validation_report` | `env: Env, report_id: String` | `Result<MessageValidationReport, Error>` | — |
| `get_transformation` | `env: Env, transform_id: String` | `Result<MessageTransformation, Error>` | — |
| `get_supported_message_types` | `env: Env` | `Vec<String>` | — |
| `pause` | `env: Env, admin: Address` | `Result<bool, Error>` | — |
| `resume` | `env: Env, admin: Address` | `Result<bool, Error>` | — |

### Types

#### `enum EMRStatus`

| Variant | Value | Description |
|---|---|---|
| `Active` | — | — |
| `Inactive` | — | — |
| `Suspended` | — | — |
| `Decommissioned` | — | — |

#### `enum IntegrationStatus`

| Variant | Value | Description |
|---|---|---|
| `Pending` | — | — |
| `InProgress` | — | — |
| `Completed` | — | — |
| `Failed` | — | — |
| `Paused` | — | — |

#### `struct EMRSystem`

| Field | Type | Description |
|---|---|---|
| `system_id` | `String` | — |
| `vendor_name` | `String` | — |
| `vendor_contact` | `String` | — |
| `system_version` | `String` | — |
| `supported_standards` | `Vec<String>` | — |
| `api_endpoints` | `Vec<String>` | — |
| `status` | `EMRStatus` | — |
| `last_activity` | `u64` | — |
| `integration_date` | `u64` | — |

#### `struct ProviderOnboarding`

| Field | Type | Description |
|---|---|---|
| `onboarding_id` | `String` | — |
| `provider_id` | `String` | — |
| `provider_name` | `String` | — |
| `provider_email` | `String` | — |
| `facility_name` | `String` | — |
| `npi` | `String` | — |
| `emr_system_id` | `String` | — |
| `status` | `IntegrationStatus` | — |
| `created_at` | `u64` | — |
| `completed_at` | `u64` | — |
| `verification_document_hash` | `BytesN<32>` | — |
| `compliance_checklist` | `Vec<String>` | — |
| `notes` | `String` | — |

#### `struct ProviderVerification`

| Field | Type | Description |
|---|---|---|
| `verification_id` | `String` | — |
| `provider_id` | `String` | — |
| `verified_by` | `Address` | — |
| `verification_timestamp` | `u64` | — |
| `license_number` | `String` | — |
| `license_state` | `String` | — |
| `license_expiration` | `String` | — |
| `board_certification` | `Vec<String>` | — |
| `malpractice_insurance` | `String` | — |
| `background_check_id` | `String` | — |
| `verification_status` | `String` | — |

#### `struct NetworkNode`

| Field | Type | Description |
|---|---|---|
| `node_id` | `String` | — |
| `provider_id` | `String` | — |
| `node_type` | `String` | — |
| `network_name` | `String` | — |
| `geographic_region` | `String` | — |
| `specialties` | `Vec<String>` | — |
| `bed_capacity` | `u32` | — |
| `operating_hours` | `String` | — |
| `emergency_services` | `bool` | — |
| `telemedicine_enabled` | `bool` | — |
| `coordinates` | `String` | — |
| `connectivity_score` | `u32` | — |

#### `struct InteroperabilityAgreement`

| Field | Type | Description |
|---|---|---|
| `agreement_id` | `String` | — |
| `initiating_provider` | `String` | — |
| `receiving_provider` | `String` | — |
| `effective_date` | `String` | — |
| `expiration_date` | `String` | — |
| `supported_data_types` | `Vec<String>` | — |
| `access_level` | `String` | — |
| `audit_requirement` | `String` | — |
| `data_encryption` | `String` | — |
| `status` | `String` | — |

#### `struct InteroperabilityTest`

| Field | Type | Description |
|---|---|---|
| `test_id` | `String` | — |
| `test_date` | `u64` | — |
| `provider_a` | `String` | — |
| `provider_b` | `String` | — |
| `test_type` | `String` | — |
| `result_status` | `String` | — |
| `success_rate` | `u32` | — |
| `data_exchanged` | `u64` | — |
| `latency_ms` | `u32` | — |
| `error_details` | `String` | — |
| `tester_address` | `Address` | — |

#### `enum MessagingStandard`

| Variant | Value | Description |
|---|---|---|
| `HL7v2` | — | — |
| `HL7v3` | — | — |
| `CDA` | — | — |

#### `enum TransportProtocol`

| Variant | Value | Description |
|---|---|---|
| `MLLP` | — | — |
| `HTTP` | — | — |

#### `enum CharacterEncoding`

| Variant | Value | Description |
|---|---|---|
| `UTF8` | — | — |
| `UTF16` | — | — |
| `ASCII` | — | — |
| `ISO88591` | — | — |

#### `enum ValidationSeverity`

| Variant | Value | Description |
|---|---|---|
| `Info` | — | — |
| `Warning` | — | — |
| `Critical` | — | — |

#### `struct HealthcareMessage`

| Field | Type | Description |
|---|---|---|
| `message_id` | `String` | — |
| `source_system_id` | `String` | — |
| `standard` | `MessagingStandard` | — |
| `version` | `String` | — |
| `message_type` | `String` | — |
| `control_id` | `String` | — |
| `content_type` | `String` | — |
| `encoding` | `CharacterEncoding` | — |
| `transport` | `TransportProtocol` | — |
| `segment_count` | `u32` | — |
| `field_count` | `u32` | — |
| `metadata` | `Map<String, String>` | — |
| `raw_payload` | `String` | — |
| `created_at` | `u64` | — |

#### `struct ValidationIssue`

| Field | Type | Description |
|---|---|---|
| `code` | `String` | — |
| `severity` | `ValidationSeverity` | — |
| `message` | `String` | — |
| `location` | `String` | — |

#### `struct MessageValidationReport`

| Field | Type | Description |
|---|---|---|
| `report_id` | `String` | — |
| `message_id` | `String` | — |
| `is_valid` | `bool` | — |
| `issues` | `Vec<ValidationIssue>` | — |
| `validated_at` | `u64` | — |

#### `struct MessageTransformation`

| Field | Type | Description |
|---|---|---|
| `transform_id` | `String` | — |
| `source_message_id` | `String` | — |
| `target_message_id` | `String` | — |
| `source_standard` | `MessagingStandard` | — |
| `target_standard` | `MessagingStandard` | — |
| `target_message_type` | `String` | — |
| `status` | `String` | — |
| `notes` | `String` | — |
| `transformed_at` | `u64` | — |

#### `struct ThroughputBenchmark`

| Field | Type | Description |
|---|---|---|
| `benchmark_id` | `String` | — |
| `batch_size` | `u32` | — |
| `message_type` | `String` | — |
| `encoding` | `CharacterEncoding` | — |
| `transport` | `TransportProtocol` | — |
| `elapsed_ms` | `u32` | — |
| `messages_per_second` | `u32` | — |

#### `enum Error`

| Variant | Value | Description |
|---|---|---|
| `NotAuthorized` | 1 | — |
| `ContractPaused` | 2 | — |
| `EMRSystemNotFound` | 3 | — |
| `EMRSystemAlreadyExists` | 4 | — |
| `OnboardingNotFound` | 5 | — |
| `OnboardingAlreadyExists` | 6 | — |
| `VerificationNotFound` | 7 | — |
| `NetworkNodeNotFound` | 8 | — |
| `AgreementNotFound` | 9 | — |
| `TestNotFound` | 10 | — |
| `InvalidStatus` | 11 | — |
| `InvalidEMRSystem` | 12 | — |
| `ProviderNotFound` | 13 | — |
| `InvalidNPI` | 14 | — |
| `InvalidLicenseNumber` | 15 | — |
| `LicenseExpired` | 16 | — |
| `InvalidAgreement` | 17 | — |
| `AgreementNotActive` | 18 | — |
| `TestFailed` | 19 | — |
| `InvalidTestType` | 20 | — |
| `DuplicateTest` | 21 | — |
| `FHIRContractNotSet` | 22 | — |
| `OperationFailed` | 23 | — |
| `UnsupportedMessageFormat` | 24 | — |
| `MessageParseFailed` | 25 | — |
| `UnsupportedMessageType` | 26 | — |
| `InvalidMessagePayload` | 27 | — |
| `MessageNotFound` | 28 | — |
| `ValidationReportNotFound` | 29 | — |
| `TransformationNotFound` | 30 | — |
| `UnsupportedEncoding` | 31 | — |

### Examples

#### `initialize_and_generate_hl7_v2_message`

```rust
let env = Env::default();
    let (client, _admin, _) = setup(&env);

    let generated = client.mock_all_auths().generate_message(
        &Address::generate(&env),
        &String::from_str(&env, "msg-1"),
        &String::from_str(&env, "epic-prod"),
        &MessagingStandard::HL7v2,
        &String::from_str(&env, "2.5.1"),
```

#### `parse_hl7_v2_message_extracts_header_fields`

```rust
let env = Env::default();
    let (client, _admin, _) = setup(&env);

    let payload = String::from_str(
        &env,
        "MSH|^~\\&|Uzima|Main|EMR|Receiving|20260328090000||ORU^R01|CTRL-99|P|2.5.1||||||UTF-8\rPID|1||PAT-001||DOE^JANE\rOBX|1|TX|NOTE||All good",
    );

    let parsed = client.mock_all_auths().parse_message(
```

#### `supports_hl7_v3_and_cda_documents`

```rust
let env = Env::default();
    let (client, _admin, _) = setup(&env);

    let v3 = client.mock_all_auths().generate_message(
        &Address::generate(&env),
        &String::from_str(&env, "msg-v3"),
        &String::from_str(&env, "epic-prod"),
        &MessagingStandard::HL7v3,
        &String::from_str(&env, "3.0"),
```

---

## escrow

### Functions

| Function | Parameters | Returns | Description |
|---|---|---|---|
| `initialize` | `env: Env, admin: Address` | `Result<(), Error>` | — |
| `set_fee_config` | `env: Env, caller: Address, fee_receiver: Address, platform_fee_bps: u32` | `Result<(), Error>` | — |
| `get_fee_config` | `env: Env` | `Option<FeeConfig>` | — |
| `create_escrow` | `env: Env, order_id: u64, payer: Address, payee: Address, amount: i128, token: Address` | `Result<bool, Error>` | — |
| `mark_disputed` | `env: Env, caller: Address, order_id: u64` | `Result<(), Error>` | — |
| `approve_release` | `env: Env, order_id: u64, approver: Address` | `Result<(), Error>` | — |
| `release_escrow` | `env: Env, order_id: u64` | `Result<bool, Error>` | — |
| `refund_escrow` | `env: Env, order_id: u64, reason: String` | `Result<bool, Error>` | — |
| `get_escrow` | `env: Env, order_id: u64` | `Option<Escrow>` | — |
| `get_credit` | `env: Env, addr: Address` | `i128` | — |
| `withdraw` | `env: Env, caller: Address, token: Address, to: Address` | `Result<i128, Error>` | — |
| `get_total_volume` | `env: Env` | `i128` | — |
| `get_total_escrows` | `env: Env` | `u64` | — |
| `get_settled_rate` | `env: Env` | `u32` | — |
| `get_refund_rate` | `env: Env` | `u32` | — |
| `get_dispute_rate` | `env: Env` | `u32` | — |
| `get_active_escrows_count` | `env: Env` | `u64` | — |
| `get_stats_summary` | `env: Env` | `PlatformStats` | — |
| `get_platform_health_score` | `env: Env` | `u32` | — |
| `get_token_volume` | `env: Env, _token: Address` | `i128` | — |
| `get_donor_reputation` | `env: Env, _donor: Address` | `u32` | — |
| `get_daily_stats` | `env: Env, day_id: u64` | `Option<DailyStats>` | — |
| `export_summary` | `env: Env, format: String` | `ExportMetadata` | — |

### Types

#### `enum EscrowStatus`

| Variant | Value | Description |
|---|---|---|
| `Pending` | 0 | — |
| `Active` | 1 | — |
| `Settled` | 2 | — |
| `Refunded` | 3 | — |
| `Disputed` | 4 | — |

#### `struct Escrow`

| Field | Type | Description |
|---|---|---|
| `order_id` | `u64` | — |
| `payer` | `Address` | — |
| `payee` | `Address` | — |
| `amount` | `i128` | — |
| `token` | `Address` | — |
| `status` | `EscrowStatus` | — |
| `approvals` | `Vec<Address>` | — |
| `reason` | `String` | — |

#### `struct PlatformStats`

| Field | Type | Description |
|---|---|---|
| `total_volume` | `i128` | — |
| `total_escrows` | `u64` | — |
| `settled_count` | `u64` | — |
| `refunded_count` | `u64` | — |
| `disputed_count` | `u64` | — |
| `active_count` | `u64` | — |

#### `struct DailyStats`

| Field | Type | Description |
|---|---|---|
| `day_id` | `u64` | — |
| `volume` | `i128` | — |
| `count` | `u32` | — |

#### `struct ExportMetadata`

| Field | Type | Description |
|---|---|---|
| `format` | `String` | — |
| `checksum` | `BytesN<32>` | — |
| `timestamp` | `u64` | — |

#### `struct FeeConfig`

| Field | Type | Description |
|---|---|---|
| `platform_fee_bps` | `u32` | — |
| `fee_receiver` | `Address` | — |

### Error Codes

| Variant | Code | Description |
|---|---|---|
| `Unauthorized` | 100 | — |
| `NotAdmin` | 102 | — |
| `InsufficientApprovals` | 120 | — |
| `InvalidAmount` | 205 | — |
| `InvalidFeeBps` | 260 | — |
| `FeeNotSet` | 380 | — |
| `ReentrancyGuard` | 381 | — |
| `InvalidStateTransition` | 382 | — |
| `EscrowExists` | 480 | — |
| `EscrowNotFound` | 481 | — |
| `AlreadySettled` | 482 | — |
| `NoBasisToRefund` | 560 | — |
| `NoCredit` | 561 | — |
| `Overflow` | 562 | — |

---

## explainable_ai

### Functions

| Function | Parameters | Returns | Description |
|---|---|---|---|
| `initialize` | `env: Env, admin: Address` | `bool` | — |
| `request_explanation` | `env: Env, caller: Address, ai_insight_id: u64` | `u64` | — |
| `fulfill_explanation_request` | `env: Env, caller: Address, request_id: u64, model_id: BytesN<32>, explanation_method: String, feature_importance: Vec<FeatureImportance>, primary_factors: Vec<String>, confidence_impact: u32, explanation_ref: String` | `Result<bool, Error>` | — |
| `get_explanation_request` | `env: Env, request_id: u64` | `Option<ExplanationRequest>` | — |
| `get_explanation` | `env: Env, explanation_id: u64` | `Option<ExplanationMetadata>` | — |
| `get_explanations_for_patient` | `env: Env, _patient: Address, _page: u32, _page_size: u32` | `Vec<ExplanationMetadata>` | — |
| `submit_bias_audit` | `env: Env, caller: Address, model_id: BytesN<32>, audit_summary: String, recommendations: Vec<String>` | `Result<u64, Error>` | — |
| `get_bias_audit` | `env: Env, model_id: BytesN<32>` | `Option<BiasAuditResult>` | — |
| `run_fairness_metrics` | `env: Env, caller: Address, _model_id: BytesN<32>, _protected_attribute: String, _privileged_group: String, _unprivileged_group: String` | `Result<(u32, u32, u32), Error>` | — |
| `generate_shap_explanation` | `env: Env, caller: Address, model_id: BytesN<32>, prediction_id: u64, base_value: i128, prediction: i128, feature_names: Vec<String>, feature_values: Vec<i128>, method: String` | `Result<u64, Error>` | Generate SHAP explanation for a prediction |
| `get_shap_explanation` | `env: Env, shap_id: u64` | `Option<ShapExplanation>` | Get SHAP explanation by ID |
| `generate_counterfactual` | `env: Env, caller: Address, original_prediction: i128, target_prediction: i128, current_features: Vec<(String, i128` | `()` | Generate counterfactual explanation |
| `get_counterfactual` | `env: Env, cf_id: u64` | `Option<CounterfactualExplanation>` | Get counterfactual explanation by ID |

### Types

#### `struct ShapValue`

| Field | Type | Description |
|---|---|---|
| `feature_name` | `String` | — |
| `shap_value` | `i128` | — |
| `absolute_shap` | `u128` | — |
| `feature_value` | `i128` | — |
| `baseline_value` | `i128` | — |

#### `struct ShapExplanation`

| Field | Type | Description |
|---|---|---|
| `explanation_id` | `u64` | — |
| `model_id` | `BytesN<32>` | — |
| `patient` | `Address` | — |
| `prediction_id` | `u64` | — |
| `base_value` | `i128` | — |
| `prediction` | `i128` | — |
| `shap_values` | `Vec<ShapValue>` | — |
| `method` | `String` | — |
| `computation_time_ms` | `u64` | — |
| `created_at` | `u64` | — |

#### `struct CounterfactualExplanation`

| Field | Type | Description |
|---|---|---|
| `cf_id` | `u64` | — |
| `original_prediction` | `i128` | — |
| `target_prediction` | `i128` | — |
| `minimal_changes` | `Vec<FeatureChange>` | — |
| `feasibility_score` | `u32` | — |
| `proximity_distance` | `u128` | — |
| `created_at` | `u64` | — |

#### `struct FeatureChange`

| Field | Type | Description |
|---|---|---|
| `feature_name` | `String` | — |
| `original_value` | `i128` | — |
| `counterfactual_value` | `i128` | — |
| `change_magnitude` | `u128` | — |

#### `struct ExplanationRequest`

| Field | Type | Description |
|---|---|---|
| `request_id` | `u64` | — |
| `patient` | `Address` | — |
| `ai_insight_id` | `u64` | — |
| `requested_at` | `u64` | — |
| `fulfilled_at` | `Option<u64>` | — |
| `explanation_ref` | `Option<String>` | — |
| `status` | `ExplanationStatus` | — |

#### `struct FeatureImportance`

| Field | Type | Description |
|---|---|---|
| `feature_name` | `String` | — |
| `importance_bps` | `u32` | — |
| `normalized_value` | `u32` | — |

#### `struct ExplanationMetadata`

| Field | Type | Description |
|---|---|---|
| `insight_id` | `u64` | — |
| `model_id` | `BytesN<32>` | — |
| `patient` | `Address` | — |
| `explanation_method` | `String` | — |
| `feature_importance` | `Vec<FeatureImportance>` | — |
| `primary_factors` | `Vec<String>` | — |
| `confidence_impact` | `u32` | — |
| `created_at` | `u64` | — |
| `explanation_ref` | `String` | — |

#### `enum ExplanationStatus`

| Variant | Value | Description |
|---|---|---|
| `Pending` | — | — |
| `Processing` | — | — |
| `Completed` | — | — |
| `Failed` | — | — |

#### `struct BiasAuditResult`

| Field | Type | Description |
|---|---|---|
| `model_id` | `BytesN<32>` | — |
| `audit_date` | `u64` | — |
| `demographic_fairness_metrics` | `Map<String, u32>` | — |
| `equalized_odds` | `bool, pub calibration_by_group: Map<String` | — |
| `audit_summary` | `String, pub recommendations: Vec<String>,` | — |

#### `enum DataKey`

| Variant | Value | Description |
|---|---|---|
| `Admin` | — | — |
| `Request(u64)` | — | — |
| `Explanation(u64)` | — | — |
| `BiasAudit(BytesN<32>)` | — | — |
| `RequestCounter` | — | — |
| `ExplanationCounter` | — | — |
| `AuditCounter` | — | — |
| `ShapExplanation(u64)` | — | — |
| `ShapCounter` | — | — |
| `Counterfactual(u64)` | — | — |
| `CfCounter` | — | — |

#### `enum Error`

| Variant | Value | Description |
|---|---|---|
| `NotAuthorized` | 1 | — |
| `RequestNotFound` | 2 | — |
| `ExplanationNotFound` | 3 | — |
| `InvalidImportance` | 4 | — |
| `AuditNotFound` | 5 | — |
| `InvalidBPSValue` | 6 | — |

---

## failover_detector

### Functions

| Function | Parameters | Returns | Description |
|---|---|---|---|
| `initialize` | `env: Env, admin: Address` | `Result<(), Error>` | — |
| `assign_role` | `env: Env, caller: Address, user: Address, role_mask: u32` | `Result<(), Error>` | — |
| `detect_node_failure` | `env: Env, caller: Address, node_id: u32, reason: FailoverReason, severity_level: u32` | `Result<u64, Error>` | — |
| `get_detections` | `env: Env` | `Vec<FailoverDetection>` | — |
| `get_node_metrics` | `env: Env, node_id: u32` | `Option<NodeFailureMetric>` | — |
| `create_failover_plan` | `env: Env, caller: Address, source_node_id: u32, target_nodes: Vec<u32>` | `Result<u64, Error>` | — |
| `execute_failover` | `env: Env, caller: Address, detection_id: u64, target_node_id: u32` | `Result<u64, Error>` | — |
| `get_failover_executions` | `env: Env` | `Vec<FailoverExecution>` | — |
| `get_failover_plans` | `env: Env` | `Vec<FailoverPlan>` | — |
| `mark_recovery_success` | `env: Env, caller: Address, node_id: u32` | `Result<(), Error>` | — |
| `deactivate_failover_plan` | `env: Env, caller: Address, plan_id: u64` | `Result<(), Error>` | — |

### Types

#### `enum FailoverReason`

| Variant | Value | Description |
|---|---|---|
| `NodeFailure` | 0 | — |
| `HeartbeatTimeout` | 1 | — |
| `HighLatency` | 2 | — |
| `ResourceExhaustion` | 3 | — |
| `DataInconsistency` | 4 | — |
| `ManualTrigger` | 5 | — |

#### `enum FailoverState`

| Variant | Value | Description |
|---|---|---|
| `Pending` | 0 | — |
| `InProgress` | 1 | — |
| `Completed` | 2 | — |
| `RolledBack` | 3 | — |
| `Failed` | 4 | — |

#### `struct FailoverDetection`

| Field | Type | Description |
|---|---|---|
| `detection_id` | `u64` | — |
| `source_node_id` | `u32` | — |
| `detected_at` | `u64` | — |
| `reason` | `FailoverReason` | — |
| `severity_level` | `u32` | — |
| `consecutive_failures` | `u32` | — |
| `is_critical` | `bool` | — |

#### `struct FailoverExecution`

| Field | Type | Description |
|---|---|---|
| `execution_id` | `u64` | — |
| `detection_id` | `u64` | — |
| `source_node_id` | `u32` | — |
| `target_node_id` | `u32` | — |
| `initiated_at` | `u64` | — |
| `completed_at` | `u64` | — |
| `state` | `FailoverState` | — |
| `rto_ms` | `u64` | — |

#### `struct NodeFailureMetric`

| Field | Type | Description |
|---|---|---|
| `node_id` | `u32` | — |
| `consecutive_failures` | `u32` | — |
| `last_failure_at` | `u64` | — |
| `total_failures` | `u64` | — |
| `recovery_attempts` | `u32` | — |
| `last_successful_recovery` | `u64` | — |

#### `struct FailoverPlan`

| Field | Type | Description |
|---|---|---|
| `plan_id` | `u64` | — |
| `source_node_id` | `u32` | — |
| `target_nodes` | `Vec<u32>` | — |
| `priority_order` | `Vec<u32>` | — |
| `created_at` | `u64` | — |
| `is_active` | `bool` | — |

#### `enum Error`

| Variant | Value | Description |
|---|---|---|
| `AlreadyInitialized` | 1 | — |
| `NotInitialized` | 2 | — |
| `NotAuthorized` | 3 | — |
| `InvalidInput` | 4 | — |
| `NodeNotFound` | 5 | — |
| `FailoverNotFound` | 6 | — |
| `NoAvailableTargets` | 7 | — |
| `FailoverInProgress` | 8 | — |
| `MaxFailuresReached` | 9 | — |
| `RecoveryFailed` | 10 | — |

---

## federated_learning

### Functions

| Function | Parameters | Returns | Description |
|---|---|---|---|
| `initialize` | `env: Env, admin: Address, coordinator: Address` | `Result<bool, Error>` | — |
| `register_institution` | `env: Env, admin: Address, institution: Address, name: String, credential_hash: BytesN<32>, framework_preference: Framework` | `Result<bool, Error>` | — |
| `start_round` | `env: Env, admin: Address, base_model_id: BytesN<32>, cfg: RoundConfig` | `Result<u64, Error>` | — |
| `submit_update` | `env: Env, institution: Address, round_id: u64, gradient_hash: BytesN<32>, quality_metrics: Map<String, u32>, privacy_proof: BytesN<32>` | `Result<bool, Error>` | — |
| `begin_aggregation` | `env: Env, coordinator: Address, round_id: u64` | `Result<bool, Error>` | — |
| `finalize_round` | `env: Env, coordinator: Address, round_id: u64, out: ModelOutput` | `Result<bool, Error>` | — |
| `get_institution` | `env: Env, institution: Address` | `Option<Institution>` | — |
| `get_round` | `env: Env, round_id: u64` | `Option<FederatedRound>` | — |
| `get_model` | `env: Env, model_id: BytesN<32>` | `Option<ModelMetadata>` | — |
| `get_privacy_metrics` | `env: Env, round_id: u64` | `Option<PrivacyMetrics>` | — |
| `get_attack_detection` | `env: Env, round_id: u64` | `Option<AttackDetection>` | — |
| `get_communication_metrics` | `env: Env, round_id: u64` | `Option<CommunicationMetrics>` | — |
| `get_contribution_verification` | `env: Env, round_id: u64, institution: Address` | `Option<ContributionVerification>` | — |
| `update_communication_metrics` | `env: Env, coordinator: Address, round_id: u64, metrics: CommunicationMetrics` | `Result<bool, Error>` | — |
| `blacklist_institution` | `env: Env, admin: Address, institution: Address, reason: String` | `Result<bool, Error>` | — |

### Types

#### `enum ModelType`

| Variant | Value | Description |
|---|---|---|
| `CNN` | — | — |
| `RNN` | — | — |
| `Transformer` | — | — |
| `FeedForward` | — | — |
| `GNN` | — | — |
| `Hybrid` | — | — |

#### `enum Framework`

| Variant | Value | Description |
|---|---|---|
| `TensorFlow` | — | — |
| `PyTorch` | — | — |
| `JAX` | — | — |
| `Custom` | — | — |

#### `enum AggregationMethod`

| Variant | Value | Description |
|---|---|---|
| `FedAvg` | — | — |
| `FedProx` | — | — |
| `SecureAgg` | — | — |
| `Krum` | — | — |
| `MultiKrum` | — | — |
| `TrimmedMean` | — | — |

#### `enum RoundStatus`

| Variant | Value | Description |
|---|---|---|
| `Open` | — | — |
| `Aggregating` | — | — |
| `Finalized` | — | — |
| `Failed` | — | — |
| `Verification` | — | — |

#### `enum InstitutionStatus`

| Variant | Value | Description |
|---|---|---|
| `Active` | — | — |
| `Suspended` | — | — |
| `Blacklisted` | — | — |
| `UnderReview` | — | — |

#### `struct Institution`

| Field | Type | Description |
|---|---|---|
| `id` | `Address` | — |
| `name` | `String` | — |
| `credential_hash` | `BytesN<32>` | — |
| `reputation_score` | `u32` | — |
| `total_contributions` | `u32` | — |
| `reward_balance` | `i128` | — |
| `status` | `InstitutionStatus` | — |
| `registered_at` | `u64` | — |
| `last_contribution` | `u64` | — |
| `privacy_budget_used` | `u32` | — |
| `contribution_quality_score` | `u32` | — |
| `framework_preference` | `Framework` | — |

#### `struct RoundConfig`

| Field | Type | Description |
|---|---|---|
| `model_type` | `ModelType` | — |
| `framework` | `Framework` | — |
| `aggregation_method` | `AggregationMethod` | — |
| `min_participants` | `u32` | — |
| `max_participants` | `u32` | — |
| `dp_epsilon` | `u32` | — |
| `dp_delta` | `u32` | — |
| `reward_per_participant` | `i128` | — |
| `duration_seconds` | `u64` | — |
| `verification_threshold` | `u32` | — |
| `poisoning_detection_threshold` | `u32` | — |
| `communication_budget` | `u32` | — |

#### `struct FederatedRound`

| Field | Type | Description |
|---|---|---|
| `id` | `u64` | — |
| `base_model_id` | `BytesN<32>` | — |
| `model_type` | `ModelType` | — |
| `framework` | `Framework` | — |
| `aggregation_method` | `AggregationMethod` | — |
| `min_participants` | `u32` | — |
| `max_participants` | `u32` | — |
| `reward_per_participant` | `i128` | — |
| `total_updates` | `u32` | — |
| `status` | `RoundStatus` | — |
| `started_at` | `u64` | — |
| `deadline` | `u64` | — |
| `finalized_at` | `u64` | — |
| `aggregated_model_id` | `BytesN<32>` | — |
| `verification_score` | `u32` | — |
| `poisoning_detected` | `bool` | — |
| `communication_overhead` | `u32` | — |

#### `struct ModelOutput`

| Field | Type | Description |
|---|---|---|
| `model_id` | `BytesN<32>` | — |
| `description` | `String` | — |
| `weights_ref` | `String` | — |
| `global_accuracy` | `u32` | — |
| `validation_score` | `u32` | — |
| `version` | `u32` | — |
| `convergence_metrics` | `Map<String, u32>` | — |
| `privacy_loss` | `u32` | — |
| `communication_cost` | `u32` | — |

#### `struct ModelMetadata`

| Field | Type | Description |
|---|---|---|
| `model_id` | `BytesN<32>` | — |
| `round_id` | `u64` | — |
| `model_type` | `ModelType` | — |
| `framework` | `Framework` | — |
| `num_contributors` | `u32` | — |
| `validation_score` | `u32` | — |
| `version` | `u32` | — |
| `aggregation_method` | `AggregationMethod` | — |
| `privacy_guarantee` | `u32` | — |
| `robustness_score` | `u32` | — |

#### `struct ContributionVerification`

| Field | Type | Description |
|---|---|---|
| `institution` | `Address` | — |
| `round_id` | `u64` | — |
| `gradient_hash` | `BytesN<32>` | — |
| `quality_score` | `u32` | — |
| `similarity_score` | `u32` | — |
| `privacy_compliance` | `bool` | — |
| `anomaly_detected` | `bool` | — |
| `contribution_weight` | `u32` | — |
| `verification_timestamp` | `u64` | — |

#### `struct PrivacyMetrics`

| Field | Type | Description |
|---|---|---|
| `epsilon_used` | `u32` | — |
| `delta_used` | `u32` | — |
| `noise_scale` | `u32` | — |
| `clipping_bound` | `u32` | — |
| `privacy_budget_remaining` | `u32` | — |
| `cumulative_privacy_loss` | `u32` | — |

#### `struct AttackDetection`

| Field | Type | Description |
|---|---|---|
| `round_id` | `u64` | — |
| `detected_attacks` | `Vec<String>` | — |
| `suspicious_participants` | `Vec<Address>` | — |
| `attack_confidence` | `u32` | — |
| `mitigation_applied` | `bool` | — |

#### `struct CommunicationMetrics`

| Field | Type | Description |
|---|---|---|
| `total_bytes_sent` | `u32` | — |
| `total_bytes_received` | `u32` | — |
| `compression_ratio` | `u32` | — |
| `latency_ms` | `u32` | — |
| `protocol_efficiency` | `u32` | — |

#### `enum DataKey`

| Variant | Value | Description |
|---|---|---|
| `Admin` | — | — |
| `Coordinator` | — | — |
| `RoundCounter` | — | — |
| `Institution(Address)` | — | — |
| `Round(u64)` | — | — |
| `RoundParticipants(u64)` | — | — |
| `UpdateSubmitted(u64, Address)` | — | — |
| `Model(BytesN<32>)` | — | — |
| `ContributionVerification(u64, Address)` | — | — |
| `PrivacyMetrics(u64)` | — | — |
| `AttackDetection(u64)` | — | — |
| `CommunicationMetrics(u64)` | — | — |

#### `enum Error`

| Variant | Value | Description |
|---|---|---|
| `NotAuthorized` | 1 | — |
| `AlreadyInitialized` | 2 | — |
| `RoundNotFound` | 3 | — |
| `RoundNotOpen` | 4 | — |
| `RoundNotAggregating` | 5 | — |
| `RoundFinalized` | 6 | — |
| `NotEnoughParticipants` | 7 | — |
| `TooManyParticipants` | 8 | — |
| `DuplicateUpdate` | 9 | — |
| `InvalidDPParam` | 10 | — |
| `InstitutionNotFound` | 11 | — |
| `InstitutionNotActive` | 12 | — |
| `InstitutionAlreadyRegistered` | 13 | — |
| `LowReputation` | 14 | — |
| `InvalidParameter` | 15 | — |
| `DeadlineExceeded` | 16 | — |
| `ValidationFailed` | 17 | — |
| `PrivacyBudgetExceeded` | 18 | — |
| `PoisoningAttackDetected` | 19 | — |
| `CommunicationBudgetExceeded` | 20 | — |
| `VerificationFailed` | 21 | — |
| `FrameworkNotSupported` | 22 | — |
| `ContributionQualityLow` | 23 | — |
| `Overflow` | 24 | — |

---

## fhir_integration

### Functions

| Function | Parameters | Returns | Description |
|---|---|---|---|
| `initialize` | `env: Env, admin: Address, medical_records_contract: Address` | `Result<bool, Error>` | Initialize the FHIR integration contract |
| `register_provider` | `env: Env, admin: Address, provider_id: String, name: String, facility_type: String, npi: String, tax_id: String, address: String, contact_point: String, emr_system: String, fhir_endpoint: String` | `Result<bool, Error>` | Register a healthcare provider with EMR system |
| `verify_provider` | `env: Env, admin: Address, provider_id: String, credential_id: BytesN<32>` | `Result<bool, Error>` | Verify a healthcare provider (onboarding completion) |
| `get_provider` | `env: Env, provider_id: String` | `Result<HealthcareProvider, Error>` | Get provider information |
| `configure_emr` | `env: Env, admin: Address, provider_id: String, fhir_version: String, supported_resources: Vec<FHIRResourceType>, authentication_type: String, oauth_endpoint: String, data_format: String, batch_size: u32, retry_policy: String` | `Result<bool, Error>` | Configure EMR system for a provider |
| `store_observation` | `env: Env, provider: Address, observation: FHIRObservation` | `Result<bool, Error>` | Store an observation (vital signs, lab results, etc.) Validates FHIR R4 required fields before storing. |
| `get_observation` | `env: Env, observation_id: String` | `Result<FHIRObservation, Error>` | Get observation by identifier |
| `store_condition` | `env: Env, provider: Address, condition: FHIRCondition` | `Result<bool, Error>` | Store a condition (diagnosis) Validates FHIR R4 required fields before storing. |
| `get_condition` | `env: Env, condition_id: String` | `Result<FHIRCondition, Error>` | Get condition by identifier |
| `store_medication` | `env: Env, provider: Address, medication: FHIRMedicationStatement` | `Result<bool, Error>` | Store medication statement |
| `get_medication` | `env: Env, medication_id: String` | `Result<FHIRMedicationStatement, Error>` | Get medication statement by identifier |
| `store_procedure` | `env: Env, provider: Address, procedure: FHIRProcedure` | `Result<bool, Error>` | Store procedure |
| `get_procedure` | `env: Env, procedure_id: String` | `Result<FHIRProcedure, Error>` | Get procedure by identifier |
| `store_allergy` | `env: Env, provider: Address, allergy: FHIRAllergyIntolerance` | `Result<bool, Error>` | Store allergy intolerance |
| `get_allergy` | `env: Env, allergy_id: String` | `Result<FHIRAllergyIntolerance, Error>` | Get allergy intolerance by identifier |
| `register_data_mapping` | `env: Env, admin: Address, mapping: DataMapping` | `Result<bool, Error>` | Register data mapping for format conversion |
| `get_data_mapping` | `env: Env, source_system: String, source_field: String` | `Result<DataMapping, Error>` | Get data mapping |
| `pause` | `env: Env, admin: Address` | `Result<bool, Error>` | Pause contract operations (emergency) |
| `resume` | `env: Env, admin: Address` | `Result<bool, Error>` | Resume contract operations |
| `export_patient_data` | `env: Env, patient: Address, format: ExportFormat, _medical_records_contract: Address` | `Result<BytesN<32>, Error>` | Export patient data in a standard format (FHIR Bundle, HL7 v2, or CDA). Only the patient themselves can request their own export. Rate-limited: max 1 export per 24 hours per patient. |
| `configure_export` | `env: Env, admin: Address, max_exports_per_day: u32, export_size_limit_bytes: u32` | `Result<bool, Error>` | Configure export limits (admin only). |
| `get_export_config` | `env: Env` | `Option<ExportConfig>` | Get export configuration. |

### Types

#### `enum FHIRResourceType`

| Variant | Value | Description |
|---|---|---|
| `Patient` | 0 | — |
| `Observation` | 1 | — |
| `Condition` | 2 | — |
| `MedicationStatement` | 3 | — |
| `Procedure` | 4 | — |
| `AllergyIntolerance` | 5 | — |
| `CareTeam` | 6 | — |
| `Encounter` | 7 | — |
| `DiagnosticReport` | 8 | — |
| `Immunization` | 9 | — |
| `DocumentReference` | 10 | — |

#### `enum CodingSystem`

| Variant | Value | Description |
|---|---|---|
| `ICD10` | — | — |
| `ICD9` | — | — |
| `CPT` | — | — |
| `SNOMEDCT` | — | — |
| `LOINC` | — | — |
| `RxNorm` | — | — |
| `Custom` | — | — |

#### `struct FHIRCode`

| Field | Type | Description |
|---|---|---|
| `system` | `CodingSystem` | — |
| `code` | `String` | — |
| `display` | `String` | — |

#### `struct FHIRIdentifier`

| Field | Type | Description |
|---|---|---|
| `system` | `String` | — |
| `urn` | `mrn:hospital-a" pub value: String` | — |
| `use_type` | `String` | — |

#### `struct FHIRPatient`

| Field | Type | Description |
|---|---|---|
| `identifiers` | `Vec<FHIRIdentifier>` | — |
| `given_name` | `String` | — |
| `family_name` | `String` | — |
| `birth_date` | `String` | — |
| `gender` | `String` | — |
| `contact_point` | `String` | — |
| `address` | `String` | — |
| `communication` | `Vec<String>` | — |
| `marital_status` | `String` | — |

#### `struct FHIRObservation`

| Field | Type | Description |
|---|---|---|
| `identifier` | `String` | — |
| `status` | `String` | — |
| `category` | `FHIRCode` | — |
| `code` | `FHIRCode` | — |
| `subject_reference` | `String` | — |
| `effective_datetime` | `String` | — |
| `value_quantity_value` | `i64` | — |
| `value_quantity_unit` | `String` | — |
| `interpretation` | `Vec<FHIRCode>` | — |
| `reference_range` | `String` | — |

#### `struct FHIRCondition`

| Field | Type | Description |
|---|---|---|
| `identifier` | `String` | — |
| `clinical_status` | `String` | — |
| `code` | `FHIRCode` | — |
| `subject_reference` | `String` | — |
| `onset_date_time` | `String` | — |
| `recorded_date` | `String` | — |
| `severity` | `Vec<FHIRCode>` | — |

#### `struct FHIRMedicationStatement`

| Field | Type | Description |
|---|---|---|
| `identifier` | `String` | — |
| `status` | `String` | — |
| `medication_code` | `FHIRCode` | — |
| `subject_reference` | `String` | — |
| `effective_period_start` | `String` | — |
| `effective_period_end` | `String` | — |
| `dosage` | `String` | — |
| `reason_code` | `Vec<FHIRCode>` | — |

#### `struct FHIRProcedure`

| Field | Type | Description |
|---|---|---|
| `identifier` | `String` | — |
| `status` | `String` | — |
| `code` | `FHIRCode` | — |
| `subject_reference` | `String` | — |
| `performed_date_time` | `String` | — |
| `performer` | `Vec<String>` | — |
| `reason_code` | `Vec<FHIRCode>` | — |

#### `struct FHIRAllergyIntolerance`

| Field | Type | Description |
|---|---|---|
| `identifier` | `String` | — |
| `clinical_status` | `String` | — |
| `verification_status` | `String` | — |
| `substance_code` | `FHIRCode` | — |
| `patient_reference` | `String` | — |
| `recorded_date` | `String` | — |
| `manifestation` | `Vec<FHIRCode>` | — |
| `severity` | `String` | — |

#### `struct FHIRBundle`

| Field | Type | Description |
|---|---|---|
| `bundle_id` | `String` | — |
| `timestamp` | `u64` | — |
| `bundle_type` | `String` | — |
| `total` | `u32` | — |

#### `struct HealthcareProvider`

| Field | Type | Description |
|---|---|---|
| `provider_id` | `String` | — |
| `name` | `String` | — |
| `facility_type` | `String` | — |
| `npi` | `String` | — |
| `tax_id` | `String` | — |
| `address` | `String` | — |
| `contact_point` | `String` | — |
| `emr_system` | `String` | — |
| `fhir_endpoint` | `String` | — |
| `is_verified` | `bool` | — |
| `verification_timestamp` | `u64` | — |
| `credential_id` | `BytesN<32>` | — |

#### `struct EMRConfiguration`

| Field | Type | Description |
|---|---|---|
| `provider_id` | `String` | — |
| `fhir_version` | `String` | — |
| `supported_resources` | `Vec<FHIRResourceType>` | — |
| `authentication_type` | `String` | — |
| `oauth_endpoint` | `String` | — |
| `data_format` | `String` | — |
| `batch_size` | `u32` | — |
| `retry_policy` | `String` | — |

#### `struct DataMapping`

| Field | Type | Description |
|---|---|---|
| `source_system` | `String` | — |
| `source_field` | `String` | — |
| `target_system` | `String` | — |
| `target_field` | `String` | — |
| `transformation_rule` | `String` | — |
| `status` | `String` | — |

#### `enum Error`

| Variant | Value | Description |
|---|---|---|
| `NotAuthorized` | 1 | — |
| `ContractPaused` | 2 | — |
| `ProviderNotFound` | 3 | — |
| `ProviderAlreadyExists` | 4 | — |
| `ObservationNotFound` | 5 | — |
| `ConditionNotFound` | 6 | — |
| `InvalidFHIRData` | 7 | — |
| `EMRConfigNotSet` | 8 | — |
| `InvalidResourceType` | 9 | — |
| `MappingNotFound` | 10 | — |
| `ProviderNotVerified` | 11 | — |
| `InvalidNPI` | 12 | — |
| `InvalidTaxId` | 13 | — |
| `BundleNotFound` | 14 | — |
| `InvalidDataFormat` | 15 | — |
| `ProviderAlreadyVerified` | 16 | — |
| `MedicalRecordsContractNotSet` | 17 | — |
| `OperationFailed` | 18 | — |
| `InvalidBundleType` | 19 | — |
| `DataMappingFailed` | 20 | — |

#### `enum ExportFormat`

| Variant | Value | Description |
|---|---|---|
| `FHIRBundle` | — | — |
| `HL7v2` | — | — |
| `CDA` | — | — |

#### `struct ExportConfig`

| Field | Type | Description |
|---|---|---|
| `max_exports_per_day` | `u32` | — |
| `export_size_limit_bytes` | `u32` | — |

### Examples

#### `test_initialize`

```rust
let env = Env::default();
    env.mock_all_auths();

    let (client, admin) = setup_initialized(&env);

    // Double initialization should fail
    let medical_records = Address::generate(&env);
    let result = client.try_initialize(&admin, &medical_records);
    assert!(result.is_err());
```

#### `test_register_and_get_provider`

```rust
let env = Env::default();
    env.mock_all_auths();

    let (client, admin) = setup_initialized(&env);

    let provider_id = String::from_str(&env, "PROV-001");
    let name = String::from_str(&env, "St. Mary Hospital");
    let facility_type = String::from_str(&env, "hospital");
    let npi = String::from_str(&env, "1234567890");
```

#### `test_register_duplicate_provider`

```rust
let env = Env::default();
    env.mock_all_auths();

    let (client, admin) = setup_initialized(&env);

    let provider_id = String::from_str(&env, "PROV-DUP");
    let npi = String::from_str(&env, "1234567890");
    let tax_id = String::from_str(&env, "12-3456789");
```

---

## fido2_authenticator

### Functions

| Function | Parameters | Returns | Description |
|---|---|---|---|
| `initialize` | `env: Env, admin: Address, rp_id_hash: BytesN<32>` | `Result<(), Error>` | Initializes the contract.  Must be called exactly once.  * `admin`      — address authorised to call administrative functions. * `rp_id_hash` — SHA-256 of the relying party identifier string (e.g., `sha256(b"uzima.health")`). |
| `set_identity_registry` | `env: Env, caller: Address, contract_id: Address` | `Result<(), Error>` | Configures the identity registry contract address. When set, `register_device` will bind new credentials to the caller's DID. |
| `set_zk_verifier` | `env: Env, caller: Address, contract_id: Address` | `Result<(), Error>` | Configures the ZK verifier contract used for ES256 (P-256) assertions. |
| `issue_registration_challenge` | `env: Env, user: Address` | `Result<BytesN<32>, Error>` | Issues a registration challenge for `user`.  The 32-byte challenge must be embedded in `clientDataJSON.challenge` during the FIDO2 attestation ceremony.  Valid for 5 minutes. |
| `register_device` | `env: Env, user: Address, credential_id_hash: BytesN<32>, public_key: Bytes, algorithm: PublicKeyAlgorithm, device_name: String, attachment: AuthenticatorAttachment, transports: Vec<AuthenticatorTransport>, initial_sign_count: u32, aaguid: BytesN<16>, backup_eligible: bool` | `Result<u32, Error>` | Completes device registration after the FIDO2 attestation ceremony.  Attestation statement verification is performed off-chain by a trusted relayer before calling this function.  The contract validates: - A non-expired challenge was issued for `user`. - The public key size matches the declared algorithm. - The credential has not been registered before. - `MAX_DEVICES` has not been reached.  When the identity registry is configured the credential is also bound to the user's DID document as a FIDO2 verification method.  Returns the zero-based device index. |
| `issue_auth_challenge` | `env: Env, user: Address` | `Result<BytesN<32>, Error>` | Issues a one-time authentication challenge for `user`. |
| `verify_ed25519_assertion` | `env: Env, user: Address, credential_id_hash: BytesN<32>, authenticator_data: Bytes, client_data_hash: BytesN<32>, signature: BytesN<64>, new_sign_count: u32` | `Result<AssertionResult, Error>` | Verifies a FIDO2 assertion signed with Ed25519 (EdDSA).  The signed payload per FIDO2 Level 2 spec is: `authenticatorData || SHA-256(clientDataJSON)`  # Arguments * `user`               — authenticating user address. * `credential_id_hash` — SHA-256 of the credential ID. * `authenticator_data` — raw `authenticatorData` bytes (≥ 37 bytes). * `client_data_hash`   — `SHA-256(clientDataJSON)`. * `signature`          — 64-byte Ed25519 signature. * `new_sign_count`     — monotonic counter value from the authenticator.  The transaction is aborted (host trap) if the Ed25519 signature is invalid. |
| `verify_zk_assertion` | `env: Env, user: Address, credential_id_hash: BytesN<32>, nullifier: BytesN<32>, commitment: BytesN<32>, proof: Bytes, new_sign_count: u32, vk_version: u32` | `Result<AssertionResult, Error>` | Verifies a FIDO2 assertion for a P-256 (ES256) credential using a ZK proof.  Because Soroban does not natively support P-256 ECDSA verification, the caller submits a ZK proof generated by a trusted off-chain prover that attests to a valid P-256 signature over `authenticatorData || clientDataHash`.  The proof also enables privacy-preserving authentication: the `nullifier` and `commitment` allow proving key ownership without disclosing the exact device on every authentication.  # Arguments * `credential_id_hash` — identifies which registered P-256 device is used. * `nullifier`          — unique value preventing proof replay. * `commitment`         — public commitment included in the ZK circuit. * `proof`              — ZK proof bytes forwarded to the verifier contract. * `new_sign_count`     — monotonic counter value from the authenticator. * `vk_version`         — verifying key version for the ZK circuit. |
| `revoke_device` | `env: Env, caller: Address, user: Address, credential_id_hash: BytesN<32>, reason: String` | `Result<(), Error>` | Revokes a device, preventing it from being used for future authentications.  Both the device owner (`user`) and the contract admin may revoke devices. A `RevocationRecord` is appended to the user's audit log. |
| `update_device_name` | `env: Env, user: Address, credential_id_hash: BytesN<32>, new_name: String` | `Result<(), Error>` | Updates the user-assigned friendly name of a registered device. |
| `list_devices` | `env: Env, caller: Address, user: Address` | `Result<Vec<Fido2Device>, Error>` | Returns all devices registered for `user` (active and revoked).  Only the user or the admin may call this function. |
| `get_device_count` | `env: Env, user: Address` | `u32` | Returns the total device count (active + revoked) for `user`. |
| `get_active_device_count` | `env: Env, user: Address` | `u32` | Returns the number of active (non-revoked) devices for `user`. |
| `is_device_registered` | `env: Env, user: Address, credential_id_hash: BytesN<32>` | `bool` | Returns `true` if `credential_id_hash` is registered and active for `user`. |
| `get_revocation_history` | `env: Env, caller: Address, user: Address` | `Result<Vec<RevocationRecord>, Error>` | Returns the full revocation audit history for `user`.  Only the user or the admin may call this function. |

### Types

#### `enum Error`

| Variant | Value | Description |
|---|---|---|
| `AlreadyInitialized` | 1 | — |
| `NotInitialized` | 2 | — |
| `NotAuthorized` | 3 | — |
| `DeviceNotFound` | 4 | — |
| `DeviceAlreadyRegistered` | 5 | — |
| `MaxDevicesReached` | 6 | — |
| `DeviceInactive` | 7 | — |
| `InvalidPublicKey` | 8 | — |
| `InvalidSignature` | 9 | — |
| `InvalidAuthenticatorData` | 10 | — |
| `ChallengeExpired = 11,
    
    NoChallengeIssued = 12,
    
    SignCountRegression = 13,
    InvalidDeviceName = 14,
    InvalidCredentialIdHash = 15,
    
    ZkVerifierNotSet = 16,
    
    NullifierAlreadyUsed = 17,
    
    RpIdMismatch = 18,
    
    UserPresenceNotVerified = 19,
    InvalidRevocationReason = 20,
    
    AlgorithmKeyMismatch = 21,` | — | — |

#### `enum PublicKeyAlgorithm`

| Variant | Value | Description |
|---|---|---|
| `EdDSA` | — | — |
| `ES256` | — | — |

#### `enum AuthenticatorTransport`

| Variant | Value | Description |
|---|---|---|
| `Usb` | — | — |
| `Nfc` | — | — |
| `Ble` | — | — |
| `Face ID` | — | — |
| `Windows Hello.
    Internal` | — | — |
| `Hybrid` | — | — |

#### `enum AuthenticatorAttachment`

| Variant | Value | Description |
|---|---|---|
| `Platform` | — | — |
| `CrossPlatform` | — | — |

#### `struct Fido2Device`

| Field | Type | Description |
|---|---|---|
| `credential_id_hash` | `BytesN<32>` | — |
| `public_key` | `Bytes` | — |
| `algorithm` | `PublicKeyAlgorithm` | — |
| `sign_count` | `u32` | — |
| `aaguid` | `BytesN<16>` | — |
| `device_name` | `String` | — |
| `attachment` | `AuthenticatorAttachment` | — |
| `transports` | `Vec<AuthenticatorTransport>` | — |
| `created_at` | `u64` | — |
| `last_used_at` | `u64` | — |
| `is_active` | `bool` | — |
| `backup_eligible` | `bool` | — |
| `backup_state` | `bool` | — |

#### `struct PendingChallenge`

| Field | Type | Description |
|---|---|---|
| `challenge` | `BytesN<32>` | — |
| `created_at` | `u64` | — |
| `expires_at` | `u64` | — |

#### `struct AssertionResult`

| Field | Type | Description |
|---|---|---|
| `credential_id_hash` | `BytesN<32>` | — |
| `new_sign_count` | `u32` | — |
| `device_name` | `String` | — |
| `attachment` | `AuthenticatorAttachment` | — |
| `verified_at` | `u64` | — |

#### `struct RevocationRecord`

| Field | Type | Description |
|---|---|---|
| `credential_id_hash` | `BytesN<32>` | — |
| `device_name` | `String` | — |
| `revoked_at` | `u64` | — |
| `revoked_by` | `Address` | — |
| `reason` | `String` | — |

#### `enum DataKey`

| Variant | Value | Description |
|---|---|---|
| `Admin` | — | — |
| `Initialized` | — | — |
| `IdentityRegistry` | — | — |
| `ZkVerifier` | — | — |
| `RpIdHash` | — | — |
| `UserDevices(Address)` | — | — |
| `PendingChallenge(Address)` | — | — |
| `UsedNullifier(BytesN<32>)` | — | — |
| `RevocationHistory(Address)` | — | — |

---

## forensics

### Functions

| Function | Parameters | Returns | Description |
|---|---|---|---|
| `initialize` | `env: Env, admin: Address` | `Result<(), Error>` | Initialize with administrator |
| `collect_evidence` | `env: Env, actor: Address, activity: ActivityType, location: BytesN<32>, evidence_data: Bytes, threat: ThreatLevel` | `u64` | Log a forensic event and collect evidence |
| `analyze_pattern` | `env: Env, pattern_id: String` | `PatternAnalysis` | Analyze activity patterns for potential threats |
| `detect_suspicious` | `env: Env, actor: Address, threshold: u32` | `bool` | Detect suspicious activity using adaptive algorithms (simplified) |
| `generate_report` | `env: Env, admin: Address, start: u64, end: u64, evidence_ids: Vec<u64>, findings: String` | `Result<u64, Error>` | Generate an immutable forensic report |
| `update_investigation` | `env: Env, admin: Address, report_id: u64, status: String` | `Result<bool, Error>` | Update an investigation status |
| `blacklist_actor` | `env: Env, admin: Address, actor_to_blacklist: Address` | `Result<(), Error>` | Blacklist a suspicious address after forensic evidence |

### Types

#### `enum Error`

| Variant | Value | Description |
|---|---|---|
| `AlreadyInitialized` | 1 | — |
| `NotInitialized` | 2 | — |
| `Unauthorized` | 3 | — |
| `ReportNotFound` | 4 | — |

### Examples

#### `test_forensics_lifecycle`

```rust
let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let contract_id = env.register_contract(None, OnChainForensics);
    let client = OnChainForensicsClient::new(&env, &contract_id);

    // 1. Initialize
    client.initialize(&admin);
```

#### `test_double_initialization_returns_error`

```rust
let env = Env::default();
    let admin = Address::generate(&env);
    let contract_id = env.register_contract(None, OnChainForensics);
    let client = OnChainForensicsClient::new(&env, &contract_id);

    client.initialize(&admin);
    let result = client.try_initialize(&admin);
    assert!(result.is_err());
```

---

## fp_math

### Functions

| Function | Parameters | Returns | Description |
|---|---|---|---|
| `mul_bps` | `amount: i128, bps: u32` | `Option<i128>` | Multiply `amount` by basis points (1 bps = 0.01%) using floor division.  Floor rounding ensures fees are never rounded up — the fee taker always receives ≤ the exact fractional amount. Callers can reconstruct the complementary side as `amount - fee` to guarantee `fee + remainder == amount`.  Returns `None` if the intermediate `amount * bps` overflows `i128`. |
| `mul_bps_round_half_up` | `amount: i128, bps: u32` | `Option<i128>` | Multiply `amount` by basis points with round-half-up rounding.  Returns `None` on overflow. |
| `tokens_for_payment` | `payment: u128, price_per_token: u128, token_decimals: u32` | `Option<u128>` | Calculate tokens to allocate for a payment: `tokens = payment * 10^token_decimals / price_per_token`  Returns `None` on overflow or if `price_per_token` is zero. |

---

## genomic_data

### Functions

| Function | Parameters | Returns | Description |
|---|---|---|---|
| `initialize` | `env: Env, admin: Address` | `Result<(), CommonError>` | — |
| `set_zk_verifier` | `env: Env, admin: Address, contract_id: Address` | `Result<(), CommonError>` | — |
| `add_record` | `env: Env, patient: Address, uploader: Address, format: GenomicFormat, compression: Compression, data_ref: String, data_hash: BytesN<32>, ciphertext_hash: BytesN<32>, tags: Vec<String>, envelopes: Vec<KeyEnvelope>, consent_id: Option<Bytes>` | `u64` | — |
| `get_record_header` | `env: Env, caller: Address, id: u64` | `Option<GenomicRecordHeader>` | — |
| `grant_consent` | `env: Env, patient: Address, record_id: u64, grantee: Address, scope: String, expires_at: u64` | `bool` | — |
| `revoke_consent` | `env: Env, patient: Address, record_id: u64, grantee: Address` | `Result<(), CommonError>` | — |
| `grant_research_consent` | `env: Env, patient: Address, record_id: u64, grantee: Address, category: GenomicConsentCategory, expires_at: u64` | `bool` | — |
| `revoke_research_consent` | `env: Env, patient: Address, record_id: u64, grantee: Address, category: GenomicConsentCategory` | `bool` | — |
| `get_record_header_for_research` | `env: Env, requester: Address, record_id: u64, category: GenomicConsentCategory` | `Option<GenomicRecordHeader>` | — |
| `verify_and_grant_access` | `env: Env, patient: Address, record_id: u64, requester: Address, vk_version: u32, public_inputs_hash: BytesN<32>, proof: Bytes, pseudonym: BytesN<32>, expires_at: u64` | `bool` | — |
| `add_gene_disease_assoc` | `env: Env, curator: Address, record_id: u64, gene: String, disease_code: String, score_bps: u32, method: String` | `u64` | — |
| `add_drug_response` | `env: Env, caller: Address, record_id: u64, drug: String, genotype_marker: String, effect: String, recommendation: String` | `u64` | — |
| `set_ancestry_profile` | `env: Env, caller: Address, record_id: u64, components: Vec<PopulationShare>, method: String` | `bool` | — |
| `create_listing` | `env: Env, seller: Address, record_id: u64, price: i128, currency: Address, escrow: Option<Address>` | `u64` | — |
| `purchase_listing` | `env: Env, buyer: Address, listing_id: u64` | `Result<(), CommonError>` | — |
| `report_breach` | `env: Env, reporter: Address, record_id: Option<u64>, severity_bps: u32, message: String` | `u64` | — |
| `upgrade` | `env: Env, caller: Address, new_wasm_hash: BytesN<32>, new_version: u32` | `Result<(), upgradeability::UpgradeError>` | — |
| `validate_upgrade` | `env: Env, new_wasm_hash: BytesN<32>` | `Result<upgradeability::UpgradeValidation, upgradeability::UpgradeError>` | — |

### Types

#### `enum GenomicFormat`

| Variant | Value | Description |
|---|---|---|
| `Fasta` | — | — |
| `Vcf` | — | — |
| `Bam` | — | — |

#### `enum Compression`

| Variant | Value | Description |
|---|---|---|
| `None` | — | — |
| `Gzip` | — | — |
| `Zstd` | — | — |

#### `enum EnvelopeAlgorithm`

| Variant | Value | Description |
|---|---|---|
| `X25519` | — | — |
| `Kyber768` | — | — |
| `HybridX25519Kyber768` | — | — |

#### `struct KeyEnvelope`

| Field | Type | Description |
|---|---|---|
| `recipient` | `Address` | — |
| `key_version` | `u32` | — |
| `algorithm` | `EnvelopeAlgorithm` | — |
| `wrapped_key` | `Bytes` | — |
| `pq_wrapped_key` | `Option<Bytes>` | — |

#### `struct GenomicRecordHeader`

| Field | Type | Description |
|---|---|---|
| `id` | `u64` | — |
| `patient` | `Address` | — |
| `uploader` | `Address` | — |
| `format` | `GenomicFormat` | — |
| `compression` | `Compression` | — |
| `created_at` | `u64` | — |
| `data_ref` | `String` | — |
| `data_hash` | `BytesN<32>` | — |
| `ciphertext_hash` | `BytesN<32>` | — |

#### `struct GenomicRecord`

| Field | Type | Description |
|---|---|---|
| `header` | `GenomicRecordHeader` | — |
| `tags` | `Vec<String>` | — |
| `envelopes` | `Vec<KeyEnvelope>` | — |
| `consent_id` | `Option<Bytes>` | — |

#### `struct PrivacyGrant`

| Field | Type | Description |
|---|---|---|
| `record_id` | `u64` | — |
| `requester` | `Address` | — |
| `expires_at` | `u64` | — |
| `pseudonym` | `BytesN<32>` | — |
| `vk_version` | `u32` | — |

#### `struct GeneDiseaseAssoc`

| Field | Type | Description |
|---|---|---|
| `record_id` | `u64` | — |
| `gene` | `String` | — |
| `disease_code` | `String` | — |
| `score_bps` | `u32` | — |
| `method` | `String` | — |
| `created_at` | `u64` | — |
| `curator` | `Address` | — |

#### `struct DrugResponse`

| Field | Type | Description |
|---|---|---|
| `record_id` | `u64` | — |
| `drug` | `String` | — |
| `genotype_marker` | `String` | — |
| `effect` | `String` | — |
| `recommendation` | `String` | — |
| `created_at` | `u64` | — |

#### `struct PopulationShare`

| Field | Type | Description |
|---|---|---|
| `label` | `String` | — |
| `bps` | `u32` | — |

#### `struct AncestryProfile`

| Field | Type | Description |
|---|---|---|
| `record_id` | `u64` | — |
| `components` | `Vec<PopulationShare>` | — |
| `method` | `String` | — |
| `created_at` | `u64` | — |

#### `struct ConsentEntry`

| Field | Type | Description |
|---|---|---|
| `record_id` | `u64` | — |
| `patient` | `Address` | — |
| `grantee` | `Address` | — |
| `scope` | `String` | — |
| `expires_at` | `u64` | — |
| `active` | `bool` | — |

#### `enum GenomicConsentCategory`

| Variant | Value | Description |
|---|---|---|
| `GeneralResearch` | — | — |
| `DiseaseSpecific(String)` | — | — |
| `CommercialResearch` | — | — |
| `InternationalTransfer` | — | — |

#### `struct ResearchConsentEntry`

| Field | Type | Description |
|---|---|---|
| `record_id` | `u64` | — |
| `patient` | `Address` | — |
| `grantee` | `Address` | — |
| `category` | `GenomicConsentCategory` | — |
| `expires_at` | `u64` | — |
| `active` | `bool` | — |

#### `struct GenomicConsentAudit`

| Field | Type | Description |
|---|---|---|
| `record_id` | `u64` | — |
| `requester` | `Address` | — |
| `category` | `GenomicConsentCategory` | — |
| `granted` | `bool` | — |
| `timestamp` | `u64` | — |

#### `struct ResearchWithdrawalNotification`

| Field | Type | Description |
|---|---|---|
| `record_id` | `u64` | — |
| `category` | `GenomicConsentCategory` | — |
| `notified_project` | `Address` | — |
| `revoked_grantee` | `Address` | — |
| `timestamp` | `u64` | — |

#### `enum ListingStatus`

| Variant | Value | Description |
|---|---|---|
| `Active` | — | — |
| `Purchased` | — | — |
| `Cancelled` | — | — |

#### `struct Listing`

| Field | Type | Description |
|---|---|---|
| `listing_id` | `u64` | — |
| `record_id` | `u64` | — |
| `seller` | `Address` | — |
| `price` | `i128` | — |
| `currency` | `Address` | — |
| `escrow` | `Option<Address>` | — |
| `buyer` | `Option<Address>` | — |
| `status` | `ListingStatus` | — |
| `created_at` | `u64` | — |

#### `struct BreachEvent`

| Field | Type | Description |
|---|---|---|
| `id` | `u64` | — |
| `reporter` | `Address` | — |
| `record_id` | `Option<u64>` | — |
| `severity_bps` | `u32` | — |
| `message` | `String` | — |
| `created_at` | `u64` | — |

#### `enum LogLevel`

| Variant | Value | Description |
|---|---|---|
| `Info` | — | — |
| `Warning` | — | — |
| `Err` | — | — |

#### `struct StructuredLog`

| Field | Type | Description |
|---|---|---|
| `timestamp` | `u64` | — |
| `level` | `LogLevel` | — |
| `operation` | `String` | — |
| `actor` | `Option<Address>` | — |
| `record_id` | `Option<u64>` | — |
| `message` | `String` | — |

#### `struct RateLimitConfig`

| Field | Type | Description |
|---|---|---|
| `max_calls` | `u32` | — |
| `window_secs` | `u64` | — |

#### `struct RateLimitEntry`

| Field | Type | Description |
|---|---|---|
| `count` | `u32` | — |
| `window_start` | `u64` | — |

#### `enum DataKey`

| Variant | Value | Description |
|---|---|---|
| `Initialized` | — | — |
| `NextId` | — | — |
| `ListingNextId` | — | — |
| `ZkVerifierContract` | — | — |
| `Record(u64)` | — | — |
| `RecordHeader(u64)` | — | — |
| `PatientRecords(Address)` | — | — |
| `Consent(u64, Address)` | — | — |
| `ResearchConsent(u64, Address, GenomicConsentCategory)` | — | — |
| `ActiveResearchProjects(u64, GenomicConsentCategory)` | — | — |
| `AssocCount(u64)` | — | — |
| `Assoc(u64, u64)` | — | — |
| `DrugRespCount(u64)` | — | — |
| `DrugResp(u64, u64)` | — | — |
| `Ancestry(u64)` | — | — |
| `Listing(u64)` | — | — |
| `RecordListings(u64)` | — | — |
| `BreachCount` | — | — |
| `Breach(u64)` | — | — |
| `RateLimitCfg(u32)` | — | — |
| `RateLimit(Address, u32)` | — | — |

### Examples

#### `test_initialize_and_add_record`

```rust
let env = Env::default();
    env.mock_all_auths();
    let (client, _id) = setup(&env);
    let admin = Address::generate(&env);
    client.initialize(&admin);

    let patient = Address::generate(&env);
    let uploader = Address::generate(&env);
    let data_ref = String::from_str(&env, "ipfs://QmData");
```

#### `test_consent_and_access`

```rust
let env = Env::default();
    env.mock_all_auths();
    let (client, _id) = setup(&env);
    let admin = Address::generate(&env);
    client.initialize(&admin);

    let patient = Address::generate(&env);
    let uploader = Address::generate(&env);
    let rid = client.add_record(
```

#### `test_research_consent_category_access_and_withdrawal_notification`

```rust
let env = Env::default();
    env.mock_all_auths();
    let (client, _id) = setup(&env);
    let admin = Address::generate(&env);
    client.initialize(&admin);

    let patient = Address::generate(&env);
    let uploader = Address::generate(&env);
    let researcher = Address::generate(&env);
```

---

## governor

### Functions

| Function | Parameters | Returns | Description |
|---|---|---|---|
| `initialize` | `env: Env, token: Address, timelock: Address, voting_delay: u64, voting_period: u64, quorum_bps: u32, proposal_threshold: i128, reputation_contract: Option<Address>, dispute_contract: Option<Address>` | `Result<(), Error>` | — |
| `propose` | `env: Env, proposer: Address, description_hash: Bytes, execution_data: Bytes` | `Result<u64, Error>` | — |
| `cast_vote` | `env: Env, proposal_id: u64, voter: Address, support: u32` | `Result<(), Error>` | — |
| `state` | `env: Env, proposal_id: u64` | `Result<u32, Error>` | — |
| `queue` | `env: Env, proposal_id: u64` | `Result<(), Error>` | — |
| `execute` | `env: Env, proposal_id: u64` | `Result<(), Error>` | — |
| `balance_of` | `env: Env, user: Address` | `i128` | — |
| `set_bal` | `env: Env, user: Address, amount: i128` | `()` | — |

### Types

#### `struct GovernorConfig`

| Field | Type | Description |
|---|---|---|
| `voting_delay` | `u64` | — |
| `voting_period` | `u64` | — |
| `quorum_bps` | `u32` | — |
| `timelock` | `Address` | — |
| `token` | `Address` | — |
| `rep_contract` | `Option<Address>` | — |
| `dispute_contract` | `Option<Address>` | — |
| `prop_threshold` | `i128` | — |

#### `struct Proposal`

| Field | Type | Description |
|---|---|---|
| `id` | `u64` | — |
| `proposer` | `Address` | — |
| `desc_hash` | `Bytes` | — |
| `start_time` | `u64` | — |
| `end_time` | `u64` | — |
| `for_votes` | `i128` | — |
| `against_votes` | `i128` | — |
| `abstain_votes` | `i128` | — |
| `canceled` | `bool` | — |
| `queued` | `bool` | — |
| `executed` | `bool` | — |
| `exec_data` | `Bytes` | — |

### Error Codes

| Variant | Code | Description |
|---|---|---|
| `NotInitialized` | 300 | — |
| `AlreadyInitialized` | 301 | — |
| `InvalidState` | 304 | — |
| `VotingClosed` | 370 | — |
| `AlreadyVoted` | 371 | — |
| `NotQueued` | 372 | — |
| `ProposalDisputed` | 373 | — |
| `ProposalNotFound` | 450 | — |
| `ProposalNotSuccessful` | 451 | — |
| `AlreadyExecuted` | 452 | — |
| `ProposalThresholdNotMet` | 530 | — |
| `NoVotingPower` | 531 | — |
| `Overflow` | 580 | — |
| `InvalidVoteType` | 280 | — |

---

## health_check

### Functions

| Function | Parameters | Returns | Description |
|---|---|---|---|
| `initialize` | `env: Env` | `Result<(), CommonError>` | Initialize the health check contract |
| `health_check` | `env: Env` | `ContractHealth` | Get comprehensive health check |
| `get_monitoring_metrics` | `env: Env` | `MonitoringMetrics` | Get detailed monitoring metrics |
| `get_gas_metrics` | `env: Env` | `GasMetrics` | Get gas usage metrics |
| `get_error_metrics` | `env: Env` | `ErrorMetrics` | Get error rate metrics |
| `record_operation` | `env: Env, gas_used: u64, success: bool` | `()` | Record an operation |
| `record_error` | `env: Env, error_code: u32` | `()` | Record an error |
| `set_paused` | `env: Env, paused: bool` | `()` | Set pause status |
| `is_paused` | `env: &Env` | `bool` | Check if contract is paused |
| `storage_usage` | `env: &Env` | `u64` | Get storage usage estimate |
| `last_activity` | `env: &Env` | `u64` | Get last activity timestamp |
| `reset_recent_errors` | `env: Env` | `()` | Reset recent errors (for hourly cleanup) |
| `check_alert_thresholds` | `env: Env` | `Vec<String>` | Get alert thresholds status |

### Types

#### `struct ContractHealth`

| Field | Type | Description |
|---|---|---|
| `version` | `String` | — |
| `is_paused` | `bool` | — |
| `storage_usage` | `u64` | — |
| `last_activity` | `u64` | — |
| `total_operations` | `u64` | — |
| `failed_operations` | `u64` | — |
| `success_rate` | `u32` | — |

#### `struct MonitoringMetrics`

| Field | Type | Description |
|---|---|---|
| `version` | `String` | — |
| `is_paused` | `bool` | — |
| `storage_usage` | `u64` | — |
| `last_activity` | `u64` | — |
| `error_count` | `u64` | — |
| `avg_gas_usage` | `u64` | — |
| `peak_gas_usage` | `u64` | — |
| `total_operations` | `u64` | — |

#### `struct GasMetrics`

| Field | Type | Description |
|---|---|---|
| `current_usage` | `u64` | — |
| `average_usage` | `u64` | — |
| `peak_usage` | `u64` | — |
| `total_consumed` | `u64` | — |
| `operation_count` | `u64` | — |

#### `struct ErrorMetrics`

| Field | Type | Description |
|---|---|---|
| `total_errors` | `u64` | — |
| `recent_errors` | `u64` | — |
| `error_rate` | `u32` | — |
| `last_error_time` | `u64` | — |
| `common_error_code` | `u32` | — |

#### `enum DataKey`

| Variant | Value | Description |
|---|---|---|
| `Initialized` | — | — |
| `Paused` | — | — |
| `Admin` | — | — |
| `LastActivity` | — | — |
| `TotalOperations` | — | — |
| `FailedOperations` | — | — |
| `TotalGasUsed` | — | — |
| `PeakGasUsage` | — | — |
| `TotalErrors` | — | — |
| `RecentErrors` | — | — |
| `LastErrorTime` | — | — |
| `CommonErrorCode` | — | — |

### Examples

#### `test_initialize`

```rust
let env = Env::default();
    let contract_id = env.register_contract(None, HealthCheckContract);
    let client = HealthCheckContractClient::new(&env, &contract_id);

    client.initialize();
    let result = client.try_initialize();
    assert!(result.is_err()); // Second init should fail
```

#### `test_health_check`

```rust
let env = Env::default();
    let contract_id = env.register_contract(None, HealthCheckContract);
    let client = HealthCheckContractClient::new(&env, &contract_id);

    client.initialize();

    let health = client.health_check();
    assert_eq!(health.version, String::from_str(&env, "1.0.0"));
    assert!(!health.is_paused);
```

#### `test_record_operation`

```rust
let env = Env::default();
    let contract_id = env.register_contract(None, HealthCheckContract);
    let client = HealthCheckContractClient::new(&env, &contract_id);

    client.initialize();

    // Record successful operation
    client.record_operation(&1000, &true);
```

---

## health_data_access_logging

### Functions

| Function | Parameters | Returns | Description |
|---|---|---|---|
| `initialize` | `env: Env, admin: Address, config: LoggingConfig` | `()` | Initialize the health data access logging contract  # Arguments * `env` - The contract environment * `admin` - The admin address (usually the contract deployer) * `config` - Logging configuration  # Panics Panics if already initialized |
| `log_access` | `env: Env, patient_id: Address, accessor_address: Address, access_type: String, metadata: Map<String, String>` | `u64` | Log an access to patient health data  This function records when someone accesses a patient's health data. Every access is logged with immutable records to create an audit trail.  # Arguments * `env` - The contract environment * `patient_id` - The address/ID of the patient whose data was accessed * `accessor_address` - The address of whoever accessed the data * `access_type` - Type of access (e.g., "read", "write", "export") * `metadata` - Optional metadata about the access (reason, context, etc.)  # Returns The ID of the created access log entry  # Panics Panics if contract is not initialized |
| `get_access_logs` | `env: Env, patient_id: Address` | `Vec<AccessLogEntry>` | Retrieve all access logs for a specific patient  Returns a vector of all access log entries for the specified patient. Caller must be either the patient themselves or have authorization.  # Arguments * `env` - The contract environment * `patient_id` - The patient whose logs to retrieve  # Returns Vector of AccessLogEntry items |
| `get_access_logs_in_range` | `env: Env, patient_id: Address, start_timestamp: u64, end_timestamp: u64` | `Vec<AccessLogEntry>` | Retrieve access logs for a patient within a specific time range  # Arguments * `env` - The contract environment * `patient_id` - The patient whose logs to retrieve * `start_timestamp` - Start of time range (inclusive) * `end_timestamp` - End of time range (inclusive)  # Returns Vector of AccessLogEntry items within the range |
| `get_logs_by_accessor` | `env: Env, patient_id: Address, accessor: Address` | `Vec<AccessLogEntry>` | Retrieve access logs by a specific accessor for a patient  # Arguments * `env` - The contract environment * `patient_id` - The patient whose logs to retrieve * `accessor` - Filter logs to only this accessor  # Returns Vector of AccessLogEntry items from the specified accessor |
| `get_latest_access_logs` | `env: Env, patient_id: Address, limit: u32` | `Vec<AccessLogEntry>` | Retrieve the most recent N access logs for a patient  # Arguments * `env` - The contract environment * `patient_id` - The patient whose logs to retrieve * `limit` - Maximum number of logs to return  # Returns Vector of up to N most recent AccessLogEntry items |
| `get_access_log_summary` | `env: Env, patient_id: Address` | `types::AccessLogSummary` | Get summary statistics for a patient's access logs  # Arguments * `env` - The contract environment * `patient_id` - The patient whose summary to retrieve  # Returns AccessLogSummary with statistics and integrity hash |
| `get_unique_accessors_count` | `env: Env, patient_id: Address` | `u32` | Get the count of unique accessors for a patient  # Arguments * `env` - The contract environment * `patient_id` - The patient to query  # Returns Number of unique addresses that have accessed this patient's data |
| `get_unique_accessors` | `env: Env, patient_id: Address` | `Vec<Address>` | Get all unique accessors for a patient  # Arguments * `env` - The contract environment * `patient_id` - The patient to query  # Returns Vector of all unique accessor addresses |
| `verify_logs_integrity` | `env: Env` | `soroban_sdk::BytesN<32>` | Verify the integrity of the access logs using the rolling hash  Returns the current rolling hash which can be compared against expected values to detect tampering.  # Arguments * `env` - The contract environment  # Returns The rolling hash of all access logs |
| `update_config` | `env: Env, config: LoggingConfig` | `()` | Update the logging configuration (admin only)  # Arguments * `env` - The contract environment * `config` - New logging configuration |
| `get_config` | `env: Env` | `LoggingConfig` | Get the current logging configuration  # Returns The current LoggingConfig |

### Examples

#### `test_initialize`

```rust
let env = create_test_env();
        let admin = Address::random(&env);
        let config = LoggingConfig {
            max_logs_per_patient: 1000,
            allow_public_queries: false,
            retention_period: 0,
        };

        HealthDataAccessLogging::initialize(env.clone(), admin.clone(), config.clone());
```

---

## healthcare_analytics_dashboard

### Functions

| Function | Parameters | Returns | Description |
|---|---|---|---|
| `initialize` | `env: Env, admin: Address, min_cohort_size: u32, noise_bps: u32` | `Result<bool, Error>` | — |
| `set_collector` | `env: Env, caller: Address, collector: Address, enabled: bool` | `Result<bool, Error>` | — |
| `configure_ai_analytics` | `env: Env, caller: Address, ai_analytics_contract: Address` | `Result<bool, Error>` | — |
| `register_data_lake_connection` | `env: Env, caller: Address, provider: String, bucket_uri: String, query_engine: String, supports_parquet: bool, supports_orc: bool, max_dataset_size_tb: u64` | `Result<u64, Error>` | — |
| `sync_export_to_data_lake` | `env: Env, caller: Address, export_id: u64, connection_id: u64, dataset_name: String, file_format: String, partition_key: String, index_ref: String, governance_tag: String, upstream_ref: String, estimated_size_tb: u64` | `Result<u64, Error>` | — |
| `optimize_query_profile` | `env: Env, caller: Address, connection_id: u64, dataset_name: String, projected_scan_mb: u64, partition_pruning_bps: u32` | `Result<QueryOptimizationProfile, Error>` | — |
| `record_medical_metric` | `env: Env, caller: Address, metric_name: String, period_id: u64, metric_value_bps: u32, cohort_size: u32` | `Result<bool, Error>` | — |
| `record_system_snapshot` | `env: Env, caller: Address, active_users: u32, tx_count: u32, error_count: u32, latency_p95_ms: u32, uptime_bps: u32` | `Result<bool, Error>` | — |
| `create_report_template` | `env: Env, caller: Address, name: String, metric_filters: Vec<String>, include_compliance: bool, include_performance: bool, output_format: String` | `Result<u64, Error>` | — |
| `schedule_report` | `env: Env, caller: Address, template_id: u64, cadence_seconds: u64, next_run_at: u64` | `Result<u64, Error>` | — |
| `run_scheduled_report` | `env: Env, caller: Address, schedule_id: u64, data_ref: String, checksum: BytesN<32>` | `Result<u64, Error>` | — |
| `upsert_compliance_summary` | `env: Env, caller: Address, period_id: u64, passed: bool, violation_count: u32, audit_event_count: u32, severity_bps: u32, report_ref: String` | `Result<bool, Error>` | — |
| `sync_ai_round` | `env: Env, caller: Address, round_id: u64` | `Result<AiRoundInsight, Error>` | — |
| `get_config` | `env: Env` | `Result<DashboardConfig, Error>` | — |
| `get_metric_aggregate` | `env: Env, metric_name: String, period_id: u64` | `Result<MetricAggregate, Error>` | — |
| `get_latest_snapshot` | `env: Env` | `Option<DashboardSnapshot>` | — |
| `get_performance_kpi` | `env: Env` | `Option<PerformanceKpi>` | — |
| `get_report_template` | `env: Env, template_id: u64` | `Option<ReportTemplate>` | — |
| `get_report_schedule` | `env: Env, schedule_id: u64` | `Option<ReportSchedule>` | — |
| `get_compliance_summary` | `env: Env, period_id: u64` | `Option<ComplianceSummary>` | — |
| `get_export_record` | `env: Env, export_id: u64` | `Option<ExportRecord>` | — |
| `get_data_lake_connection` | `env: Env, connection_id: u64` | `Option<DataLakeConnection>` | — |
| `get_data_lake_partition` | `env: Env, partition_id: u64` | `Option<DataLakePartition>` | — |
| `get_lineage_record` | `env: Env, export_id: u64` | `Option<LineageRecord>` | — |
| `get_query_optimization` | `env: Env, connection_id: u64, dataset_name: String` | `Option<QueryOptimizationProfile>` | — |
| `get_ai_round_insight` | `env: Env, round_id: u64` | `Option<AiRoundInsight>` | — |
| `set_differential_privacy_contract` | `env: Env, caller: Address, dp_contract: Address` | `Result<bool, Error>` | Configure the differential privacy contract address (admin only). |
| `get_differential_privacy_contract` | `env: Env` | `Option<Address>` | Get the configured differential privacy contract address. |
| `apply_differential_privacy_noise` | `env: Env, caller: Address, budget_id: BytesN<32>, query_id: BytesN<32>, data_type: u32, true_value: i64, sensitivity: u64` | `Result<u64, Error>` | Apply Laplace noise via the configured differential privacy contract. Delegates to the DP contract's `add_laplace_noise` and returns remaining budget. |
| `get_visualization_series` | `env: Env, metric_name: String, max_points: u32` | `Vec<VisualizationPoint>` | — |

### Types

#### `struct DashboardConfig`

| Field | Type | Description |
|---|---|---|
| `admin` | `Address` | — |
| `min_cohort_size` | `u32` | — |
| `noise_bps` | `u32` | — |
| `realtime_enabled` | `bool` | — |

#### `struct MetricAggregate`

| Field | Type | Description |
|---|---|---|
| `metric_name` | `String` | — |
| `period_id` | `u64` | — |
| `total_value_bps` | `u64` | — |
| `count` | `u32` | — |
| `min_value_bps` | `u32` | — |
| `max_value_bps` | `u32` | — |
| `avg_value_bps` | `u32` | — |
| `last_updated` | `u64` | — |

#### `struct DashboardSnapshot`

| Field | Type | Description |
|---|---|---|
| `active_users` | `u32` | — |
| `tx_count` | `u32` | — |
| `error_count` | `u32` | — |
| `latency_p95_ms` | `u32` | — |
| `uptime_bps` | `u32` | — |
| `timestamp` | `u64` | — |

#### `struct PerformanceKpi`

| Field | Type | Description |
|---|---|---|
| `total_snapshots` | `u32` | — |
| `avg_latency_p95_ms` | `u32` | — |
| `avg_uptime_bps` | `u32` | — |
| `avg_error_rate_bps` | `u32` | — |
| `last_updated` | `u64` | — |

#### `struct ReportTemplate`

| Field | Type | Description |
|---|---|---|
| `id` | `u64` | — |
| `name` | `String` | — |
| `metric_filters` | `Vec<String>` | — |
| `include_compliance` | `bool` | — |
| `include_performance` | `bool` | — |
| `output_format` | `String` | — |
| `created_at` | `u64` | — |

#### `struct ReportSchedule`

| Field | Type | Description |
|---|---|---|
| `id` | `u64` | — |
| `template_id` | `u64` | — |
| `cadence_seconds` | `u64` | — |
| `next_run_at` | `u64` | — |
| `last_run_at` | `u64` | — |
| `enabled` | `bool` | — |

#### `struct ComplianceSummary`

| Field | Type | Description |
|---|---|---|
| `period_id` | `u64` | — |
| `total_checks` | `u32` | — |
| `passed_checks` | `u32` | — |
| `total_violations` | `u32` | — |
| `total_audit_events` | `u32` | — |
| `severity_bps` | `u32` | — |
| `generated_at` | `u64` | — |
| `latest_report_ref` | `String` | — |

#### `struct VisualizationPoint`

| Field | Type | Description |
|---|---|---|
| `period_id` | `u64` | — |
| `avg_value_bps` | `u32` | — |
| `sample_count` | `u32` | — |

#### `struct ExportRecord`

| Field | Type | Description |
|---|---|---|
| `export_id` | `u64` | — |
| `template_id` | `u64` | — |
| `output_format` | `String` | — |
| `data_ref` | `String` | — |
| `checksum` | `BytesN<32>` | — |
| `generated_at` | `u64` | — |

#### `struct DataLakeConnection`

| Field | Type | Description |
|---|---|---|
| `id` | `u64` | — |
| `provider` | `String` | — |
| `bucket_uri` | `String` | — |
| `query_engine` | `String` | — |
| `supports_parquet` | `bool` | — |
| `supports_orc` | `bool` | — |
| `partitioning_enabled` | `bool` | — |
| `lineage_enabled` | `bool` | — |
| `encryption_at_rest` | `bool` | — |
| `encryption_in_transit` | `bool` | — |
| `max_dataset_size_tb` | `u64` | — |
| `active` | `bool` | — |

#### `struct DataLakePartition`

| Field | Type | Description |
|---|---|---|
| `id` | `u64` | — |
| `connection_id` | `u64` | — |
| `export_id` | `u64` | — |
| `dataset_name` | `String` | — |
| `file_format` | `String` | — |
| `partition_key` | `String` | — |
| `index_ref` | `String` | — |
| `estimated_size_tb` | `u64` | — |
| `created_at` | `u64` | — |

#### `struct LineageRecord`

| Field | Type | Description |
|---|---|---|
| `export_id` | `u64` | — |
| `connection_id` | `u64` | — |
| `dataset_name` | `String` | — |
| `upstream_ref` | `String` | — |
| `governance_tag` | `String` | — |
| `query_engine` | `String` | — |
| `file_format` | `String` | — |
| `recorded_at` | `u64` | — |

#### `struct QueryOptimizationProfile`

| Field | Type | Description |
|---|---|---|
| `connection_id` | `u64` | — |
| `dataset_name` | `String` | — |
| `projected_scan_mb` | `u64` | — |
| `partition_pruning_bps` | `u32` | — |
| `performance_score_bps` | `u32` | — |
| `optimized_at` | `u64` | — |

#### `struct AiRoundInsight`

| Field | Type | Description |
|---|---|---|
| `round_id` | `u64` | — |
| `min_participants` | `u32` | — |
| `total_updates` | `u32` | — |
| `dp_epsilon` | `u32` | — |
| `is_finalized` | `bool` | — |
| `started_at` | `u64` | — |
| `finalized_at` | `u64` | — |
| `participation_bps` | `u32` | — |

#### `struct AiFederatedRound`

| Field | Type | Description |
|---|---|---|
| `id` | `u64` | — |
| `base_model_id` | `BytesN<32>` | — |
| `min_participants` | `u32` | — |
| `dp_epsilon` | `u32` | — |
| `started_at` | `u64` | — |
| `finalized_at` | `u64` | — |
| `total_updates` | `u32` | — |
| `is_finalized` | `bool` | — |

#### `enum DataKey`

| Variant | Value | Description |
|---|---|---|
| `Config` | — | — |
| `Collector(Address)` | — | — |
| `Metric(String, u64)` | — | — |
| `MetricPeriods(String)` | — | — |
| `LatestSnapshot` | — | — |
| `PerformanceKpi` | — | — |
| `TemplateCounter` | — | — |
| `Template(u64)` | — | — |
| `ScheduleCounter` | — | — |
| `Schedule(u64)` | — | — |
| `Compliance(u64)` | — | — |
| `ExportCounter` | — | — |
| `Export(u64)` | — | — |
| `DataLakeConnectionCounter` | — | — |
| `DataLakeConnection(u64)` | — | — |
| `DataLakePartitionCounter` | — | — |
| `DataLakePartition(u64)` | — | — |
| `LineageRecord(u64)` | — | — |
| `QueryOptimization(u64, String)` | — | — |
| `AiContract` | — | — |
| `AiInsight(u64)` | — | — |
| `DifferentialPrivacyContract` | — | — |

#### `enum Error`

| Variant | Value | Description |
|---|---|---|
| `NotAuthorized` | 1 | — |
| `AlreadyInitialized` | 2 | — |
| `NotInitialized` | 3 | — |
| `InvalidInput` | 4 | — |
| `PrivacyThresholdNotMet` | 5 | — |
| `MetricNotFound` | 6 | — |
| `TemplateNotFound` | 7 | — |
| `ScheduleNotFound` | 8 | — |
| `ComplianceNotFound` | 9 | — |
| `AiAnalyticsNotConfigured` | 10 | — |
| `AiRoundNotFound` | 11 | — |
| `DataLakeNotFound` | 12 | — |
| `ExportNotFound` | 13 | — |
| `UnsupportedDataLakeProvider` | 14 | — |

---

## healthcare_compliance

### Functions

| Function | Parameters | Returns | Description |
|---|---|---|---|
| `initialize` | `env: Env, admin: Address` | `Result<(), Error>` | Initialize the compliance contract |
| `health_check` | `env: Env` | `(Symbol, u32, u64)` | Perform a health check on the contract |
| `update_config` | `env: Env, admin: Address, config: ComplianceConfig` | `Result<(), Error>` | Update compliance configuration |
| `get_config` | `env: Env` | `Result<ComplianceConfig, Error>` | Get current compliance configuration |
| `grant_consent` | `env: Env, patient: Address, consent: ConsentRecord` | `Result<(), Error>` | Grant patient consent for data processing |
| `revoke_consent` | `env: Env, patient: Address, consent_id: String, reason: String` | `Result<(), Error>` | Revoke patient consent |
| `has_valid_consent` | `env: Env, patient: Address, purpose: String, data_category: String` | `Result<bool, Error>` | Check if patient has valid consent for specific purpose |
| `log_audit_event` | `env: Env, actor: Address, action: AuditEventType, resource_type: FHIRResourceType, resource_id: String, patient_id: String, details: String, framework: ComplianceFramework, hipaa_category: u32, gdpr_category: u32` | `Result<(), Error>` | Log audit event for compliance tracking |
| `get_audit_logs` | `env: Env, user: Address, limit: u32` | `Result<Vec<AuditLogEntry>, Error>` | Get audit logs for a specific user |
| `report_breach` | `env: Env, reporter: Address, breach: BreachReport` | `Result<(), Error>` | Report data breach |
| `get_compliance_metrics` | `env: Env` | `Result<ComplianceMetrics, Error>` | Get compliance dashboard metrics |
| `register_retention_record` | `env: Env, actor: Address, record_id: String, data_type: DataType, owner: Address` | `Result<(), Error>` | Register a data record for policy-based retention tracking. |
| `set_retention_policy` | `env: Env, admin: Address, policy: RetentionPolicy` | `Result<(), Error>` | Set or update a retention policy for a specific data type. |
| `get_retention_policy` | `env: Env, data_type: DataType` | `Result<RetentionPolicy, Error>` | Retrieve retention policy for a data class. |
| `request_data_deletion` | `env: Env, requester: Address, record_id: String` | `Result<(), Error>` | GDPR "right to be forgotten" handler. |
| `enforce_retention` | `env: Env` | `Result<u32, Error>` | Automated retention sweep that deletes all expired records. |
| `get_deletion_audit` | `env: Env` | `Vec<DeletionAuditEntry>` | Get all deletion audit entries. |
| `pause` | `env: Env, admin: Address` | `Result<(), Error>` | Pause contract operations (emergency) |
| `resume` | `env: Env, admin: Address` | `Result<(), Error>` | Resume contract operations |
| `submit_compliance_report` | `env: Env, reporter: Address, report_id: String, report_hash: BytesN<32>, uri: String` | `Result<(), Error>` | Submit a compliance report (on-chain evidence stamping) |
| `get_compliance_report` | `env: Env, report_id: String` | `Result<ReportRecord, Error>` | Retrieve a stamped compliance report |

### Types

#### `enum ComplianceFramework`

| Variant | Value | Description |
|---|---|---|
| `HIPAA` | — | — |
| `GDPR` | — | — |
| `HL7FHIR` | — | — |
| `SOX` | — | — |
| `HITECH` | — | — |

#### `enum HIPAACategory`

| Variant | Value | Description |
|---|---|---|
| `Treatment` | — | — |
| `Payment` | — | — |
| `HealthcareOperations` | — | — |
| `Research` | — | — |
| `PublicHealth` | — | — |
| `Emergency` | — | — |
| `Marketing` | — | — |

#### `enum GDPRProcessingCategory`

| Variant | Value | Description |
|---|---|---|
| `Consent` | — | — |
| `Contract` | — | — |
| `LegalObligation` | — | — |
| `VitalInterest` | — | — |
| `PublicTask` | — | — |
| `LegitimateInterest` | — | — |

#### `enum FHIRResourceType`

| Variant | Value | Description |
|---|---|---|
| `Patient` | — | — |
| `Observation` | — | — |
| `Condition` | — | — |
| `Medication` | — | — |
| `AllergyIntolerance` | — | — |
| `Procedure` | — | — |
| `DiagnosticReport` | — | — |
| `DocumentReference` | — | — |
| `Consent` | — | — |
| `AuditEvent` | — | — |

#### `enum ConsentStatus`

| Variant | Value | Description |
|---|---|---|
| `Draft` | — | — |
| `Proposed` | — | — |
| `Active` | — | — |
| `Rejected` | — | — |
| `Inactive` | — | — |
| `EnteredInError` | — | — |

#### `enum AuditEventType`

| Variant | Value | Description |
|---|---|---|
| `Create` | — | — |
| `Read` | — | — |
| `Update` | — | — |
| `Delete` | — | — |
| `Execute` | — | — |
| `Consent` | — | — |
| `Access` | — | — |
| `Disclosure` | — | — |
| `Breach` | — | — |

#### `enum BreachSeverity`

| Variant | Value | Description |
|---|---|---|
| `Low` | — | — |
| `Moderate` | — | — |
| `High` | — | — |
| `Critical` | — | — |

#### `enum ViolationType`

| Variant | Value | Description |
|---|---|---|
| `UnauthorizedAccess` | — | — |
| `DataBreach` | — | — |
| `ConsentViolation` | — | — |
| `AuditFailure` | — | — |
| `RetentionViolation` | — | — |
| `DisclosureViolation` | — | — |
| `ProcessingViolation` | — | — |

#### `enum DataType`

| Variant | Value | Description |
|---|---|---|
| `MedicalRecords` | — | — |
| `AuditLogs` | — | — |
| `TemporaryData` | — | — |
| `UserPreferences` | — | — |

#### `struct RetentionPolicy`

| Field | Type | Description |
|---|---|---|
| `data_type` | `DataType` | — |
| `retention_period` | `u64` | — |
| `auto_delete` | `bool` | — |

#### `struct RetentionRecord`

| Field | Type | Description |
|---|---|---|
| `record_id` | `String` | — |
| `data_type` | `DataType` | — |
| `owner` | `Address` | — |
| `created_at` | `u64` | — |
| `legal_hold` | `bool` | — |
| `deleted` | `bool` | — |
| `deleted_at` | `u64` | — |

#### `struct DeletionAuditEntry`

| Field | Type | Description |
|---|---|---|
| `record_id` | `String` | — |
| `data_type` | `DataType` | — |
| `deleted_at` | `u64` | — |
| `deleted_by` | `Address` | — |
| `reason` | `String` | — |

#### `struct ConsentRecord`

| Field | Type | Description |
|---|---|---|
| `consent_id` | `String` | — |
| `patient` | `Address` | — |
| `data_controller` | `Address` | — |
| `data_processor` | `Address` | — |
| `purpose` | `String` | — |
| `data_categories` | `Vec<String>` | — |
| `processing_categories` | `Vec<GDPRProcessingCategory>` | — |
| `status` | `ConsentStatus` | — |
| `granted_at` | `u64` | — |
| `expires_at` | `u64` | — |
| `revoked_at` | `u64` | — |
| `revocation_reason` | `String` | — |
| `signature` | `BytesN<64>` | — |

#### `struct AuditLogEntry`

| Field | Type | Description |
|---|---|---|
| `log_id` | `String` | — |
| `timestamp` | `u64` | — |
| `actor` | `Address` | — |
| `action` | `AuditEventType` | — |
| `resource_type` | `FHIRResourceType` | — |
| `resource_id` | `String` | — |
| `patient_id` | `String` | — |
| `success` | `bool` | — |
| `details` | `String` | — |
| `ip_address` | `String` | — |
| `user_agent` | `String` | — |
| `compliance_framework` | `ComplianceFramework` | — |
| `hipaa_category` | `u32` | — |
| `gdpr_category` | `u32` | — |

#### `struct BreachReport`

| Field | Type | Description |
|---|---|---|
| `report_id` | `String` | — |
| `timestamp` | `u64` | — |
| `reporter` | `Address` | — |
| `severity` | `BreachSeverity` | — |
| `affected_records` | `u32` | — |
| `affected_patients` | `Vec<Address>` | — |
| `breach_type` | `String` | — |
| `description` | `String` | — |
| `mitigation_steps` | `Vec<String>` | — |
| `notified_authorities` | `bool` | — |
| `notified_patients` | `bool` | — |
| `resolution_status` | `String` | — |

#### `struct ViolationReport`

| Field | Type | Description |
|---|---|---|
| `violation_id` | `String` | — |
| `timestamp` | `u64` | — |
| `reporter` | `Address` | — |
| `violation_type` | `ViolationType` | — |
| `affected_resource` | `String` | — |
| `actor` | `Address` | — |
| `details` | `String` | — |
| `evidence` | `Vec<String>` | — |
| `resolved` | `bool` | — |
| `resolution_notes` | `String` | — |
| `penalty_amount` | `i128` | — |

#### `struct ReportRecord`

| Field | Type | Description |
|---|---|---|
| `report_id` | `String` | — |
| `reporter` | `Address` | — |
| `timestamp` | `u64` | — |
| `report_hash` | `BytesN<32>` | — |
| `uri` | `String` | — |

#### `struct ComplianceMetrics`

| Field | Type | Description |
|---|---|---|
| `total_audits` | `u32` | — |
| `successful_audits` | `u32` | — |
| `failed_audits` | `u32` | — |
| `total_consents` | `u32` | — |
| `active_consents` | `u32` | — |
| `revoked_consents` | `u32` | — |
| `total_breaches` | `u32` | — |
| `resolved_breaches` | `u32` | — |
| `pending_violations` | `u32` | — |
| `compliance_score` | `u32` | — |
| `last_audit_timestamp` | `u64` | — |

#### `struct ComplianceConfig`

| Field | Type | Description |
|---|---|---|
| `hipaa_enabled` | `bool` | — |
| `gdpr_enabled` | `bool` | — |
| `hl7_fhir_enabled` | `bool` | — |
| `audit_logging_enabled` | `bool` | — |
| `breach_notification_enabled` | `bool` | — |
| `auto_consent_expiration` | `bool` | — |
| `default_retention_days` | `u32` | — |
| `admin_addresses` | `Vec<Address>` | — |
| `compliance_officers` | `Vec<Address>` | — |

#### `enum Error`

| Variant | Value | Description |
|---|---|---|
| `NotAuthorized` | 1 | — |
| `ContractPaused` | 2 | — |
| `ConsentNotFound` | 3 | — |
| `ConsentAlreadyExists` | 4 | — |
| `InvalidConsentStatus` | 5 | — |
| `ConsentExpired` | 6 | — |
| `AuditLogNotFound` | 7 | — |
| `BreachReportNotFound` | 8 | — |
| `ViolationNotFound` | 9 | — |
| `InvalidFramework` | 10 | — |
| `InvalidResourceType` | 11 | — |
| `DataBreachAlreadyReported` | 12 | — |
| `ViolationAlreadyExists` | 13 | — |
| `InvalidSignature` | 14 | — |
| `RetentionPolicyNotFound` | 15 | — |
| `ComplianceConfigNotSet` | 16 | — |
| `InsufficientPermissions` | 17 | — |
| `DataPurgeFailed` | 18 | — |
| `NotificationFailed` | 19 | — |
| `InvalidPatientAddress` | 20 | — |
| `ReportAlreadyExists` | 21 | — |
| `ReportNotFound` | 22 | — |
| `RecordAlreadyExists` | 23 | — |
| `RetentionRecordNotFound` | 24 | — |
| `RecordNotDeletable` | 25 | — |
| `LegalHoldActive` | 26 | — |

### Examples

#### `test_submit_and_get_compliance_report`

```rust
let env = Env::default();
    let (client, _admin) = setup_contract(&env);

    let reporter = Address::generate(&env);
    let report_id = String::from_str(&env, "report-1");
    let report_hash = BytesN::from_array(&env, &[1u8; 32]);
    let uri = String::from_str(&env, "ipfs://report-1");

    let r = client.submit_compliance_report(&reporter, &report_id, &report_hash, &uri);
```

#### `test_default_retention_policies_exist`

```rust
let env = Env::default();
    let (client, _admin) = setup_contract(&env);

    let med = client.get_retention_policy(&DataType::MedicalRecords).expect("policy");
    let audit = client.get_retention_policy(&DataType::AuditLogs).expect("policy");
    let temp = client
        .get_retention_policy(&DataType::TemporaryData)
        .expect("policy");
    let pref = client
```

#### `test_enforce_retention_deletes_expired_temporary_data`

```rust
let env = Env::default();
    let (client, _admin) = setup_contract(&env);
    let actor = Address::generate(&env);
    let owner = Address::generate(&env);
    let record_id = String::from_str(&env, "tmp-1");

    env.ledger().with_mut(|li| li.timestamp = 1);
    client.register_retention_record(
        &actor,
```

---

## healthcare_compliance_automation

### Functions

| Function | Parameters | Returns | Description |
|---|---|---|---|
| `initialize` | `env: Env, admin: Address, frameworks: Vec<String>` | `()` | — |
| `add_framework` | `env: Env, admin: Address, framework: String` | `()` | — |
| `get_supported_frameworks` | `env: Env` | `FrameworkList` | — |

### Types

#### `struct FrameworkList`

| Field | Type | Description |
|---|---|---|
| `frameworks` | `Vec<String>` | — |

---

## healthcare_data_conversion

### Functions

| Function | Parameters | Returns | Description |
|---|---|---|---|
| `initialize` | `env: Env, admin: Address` | `Result<bool, Error>` | Initialize the healthcare data conversion contract |
| `register_conversion_rule` | `env: Env, admin: Address, rule: ConversionRule` | `Result<bool, Error>` | Register a conversion rule |
| `get_conversion_rule` | `env: Env, rule_id: String` | `Result<ConversionRule, Error>` | Get conversion rule |
| `register_coding_mapping` | `env: Env, admin: Address, mapping: CodingMapping` | `Result<bool, Error>` | Register healthcare coding mapping (e.g., ICD9 to ICD10) |
| `get_coding_mapping` | `env: Env, mapping_id: String` | `Result<CodingMapping, Error>` | Get coding mapping |
| `find_coding_mapping` | `env: Env, _source_system: String, _target_system: String, _source_code: String` | `Result<CodingMapping, Error>` | Get coding mapping by source and target codes |
| `register_format_specification` | `env: Env, admin: Address, spec: FormatSpecification` | `Result<bool, Error>` | Register format specification |
| `get_format_specification` | `env: Env, format: DataFormat` | `Result<FormatSpecification, Error>` | Get format specification |
| `validate_conversion` | `env: Env, validator: Address, source_format: DataFormat, target_format: DataFormat, _source_data_hash: BytesN<32>` | `Result<ValidationResult, Error>` | Validate data format conversion compatibility |
| `record_conversion` | `env: Env, requester: Address, source_format: DataFormat, target_format: DataFormat, source_data_hash: BytesN<32>, target_data_hash: BytesN<32>` | `Result<u64, Error>` | Record a data conversion request |
| `get_conversion_request` | `env: Env, request_id: u64` | `Result<ConversionRequest, Error>` | Get conversion request details |
| `record_lossy_conversion_warning` | `env: Env, admin: Address, warning: LossyConversionWarning` | `Result<bool, Error>` | Record lossy conversion warning |
| `get_lossy_conversion_warning` | `env: Env, warning_id: String` | `Result<LossyConversionWarning, Error>` | Get lossy conversion warning |
| `pause` | `env: Env, admin: Address` | `Result<bool, Error>` | Pause contract operations |
| `resume` | `env: Env, admin: Address` | `Result<bool, Error>` | Resume contract operations |

### Types

#### `enum DataFormat`

| Variant | Value | Description |
|---|---|---|
| `FHIRJSON` | 0 | — |
| `FHIRXML` | 1 | — |
| `HL7v2` | 2 | — |
| `CDA` | 3 | — |
| `HL7v3` | 4 | — |
| `CCD` | 5 | — |
| `C32` | 6 | — |
| `PDF` | 7 | — |
| `CSV` | 8 | — |

#### `enum FieldType`

| Variant | Value | Description |
|---|---|---|
| `String` | — | — |
| `Integer` | — | — |
| `Decimal` | — | — |
| `DateTime` | — | — |
| `Boolean` | — | — |
| `Code` | — | — |
| `Array` | — | — |
| `Object` | — | — |

#### `struct ConversionRule`

| Field | Type | Description |
|---|---|---|
| `rule_id` | `String` | — |
| `source_format` | `DataFormat` | — |
| `target_format` | `DataFormat` | — |
| `source_path` | `String` | — |
| `target_path` | `String` | — |
| `transformation_type` | `String` | — |
| `field_type` | `FieldType` | — |
| `mapping_table_ref` | `String` | — |
| `validation_rules` | `Vec<String>` | — |
| `is_active` | `bool` | — |

#### `struct CodingMapping`

| Field | Type | Description |
|---|---|---|
| `mapping_id` | `String` | — |
| `source_code_system` | `String` | — |
| `target_code_system` | `String` | — |
| `source_code` | `String` | — |
| `target_code` | `String` | — |
| `source_description` | `String` | — |
| `target_description` | `String` | — |
| `confidence_score` | `u32` | — |
| `backward_mapping` | `Option<String>` | — |
| `effective_date` | `String` | — |
| `end_date` | `String` | — |

#### `struct FormatSpecification`

| Field | Type | Description |
|---|---|---|
| `format` | `DataFormat` | — |
| `version` | `String` | — |
| `mime_type` | `String` | — |
| `encoding` | `String` | — |
| `character_set` | `String` | — |
| `supported_resources` | `Vec<String>` | — |
| `description` | `String` | — |
| `standard_url` | `String` | — |

#### `struct ConversionRequest`

| Field | Type | Description |
|---|---|---|
| `request_id` | `u64` | — |
| `source_format` | `DataFormat` | — |
| `target_format` | `DataFormat` | — |
| `source_data_hash` | `BytesN<32>` | — |
| `target_data_hash` | `BytesN<32>` | — |
| `conversion_timestamp` | `u64` | — |
| `requester` | `Address` | — |
| `status` | `String` | — |
| `error_details` | `String` | — |

#### `struct ValidationResult`

| Field | Type | Description |
|---|---|---|
| `validation_id` | `u64` | — |
| `source_format` | `DataFormat` | — |
| `target_format` | `DataFormat` | — |
| `is_valid` | `bool` | — |
| `validation_errors` | `Vec<String>` | — |
| `validation_warnings` | `Vec<String>` | — |
| `validated_at` | `u64` | — |

#### `struct LossyConversionWarning`

| Field | Type | Description |
|---|---|---|
| `warning_id` | `String` | — |
| `conversion_request_id` | `u64` | — |
| `lost_fields` | `Vec<String>` | — |
| `data_loss_percentage` | `u32` | — |
| `mitigation_recommendation` | `String` | — |

#### `enum Error`

| Variant | Value | Description |
|---|---|---|
| `NotAuthorized` | 1 | — |
| `ContractPaused` | 2 | — |
| `RuleNotFound` | 3 | — |
| `CodingMappingNotFound` | 4 | — |
| `FormatNotSupported` | 5 | — |
| `ConversionFailed` | 6 | — |
| `ValidationFailed` | 7 | — |
| `InvalidConversionRequest` | 8 | — |
| `SourceFormatNotSupported` | 9 | — |
| `TargetFormatNotSupported` | 10 | — |
| `MappingTableNotFound` | 11 | — |
| `DuplicateRule` | 12 | — |
| `IncompatibleFormats` | 13 | — |
| `DataLossWarning` | 14 | — |
| `InvalidMappingData` | 15 | — |
| `OperationFailed` | 16 | — |

### Examples

#### `test_initialize`

```rust
let env = Env::default();
    env.mock_all_auths();

    let (client, _id) = setup(&env);
    let admin = Address::generate(&env);
    assert!(client.initialize(&admin).is_ok());

    // Double initialization should fail
    let result = client.try_initialize(&admin);
```

#### `test_initialize_unauthorized`

```rust
let env = Env::default();
    env.mock_all_auths();

    let (client, _id) = setup(&env);
    let admin = Address::generate(&env);
    let other = Address::generate(&env);

    // First init as admin
    assert!(client.initialize(&admin).is_ok());
```

#### `test_register_and_get_conversion_rule`

```rust
let env = Env::default();
    env.mock_all_auths();

    let (client, _id) = setup(&env);
    let admin = Address::generate(&env);
    client.initialize(&admin);

    let rule = ConversionRule {
        rule_id: String::from_str(&env, "rule-001"),
```

---

## healthcare_data_marketplace

### Functions

| Function | Parameters | Returns | Description |
|---|---|---|---|
| `initialize` | `env: Env, admin: Address, payment_router: Address, escrow_contract: Address, treasury: Address, settlement_window_secs: u64` | `Result<(), Error>` | — |
| `register_provider` | `env: Env, provider: Address` | `Result<(), Error>` | — |
| `set_provider_status` | `env: Env, admin: Address, provider: Address, active: bool` | `Result<(), Error>` | — |
| `create_listing` | `env: Env, provider: Address, payload: ListingPayload` | `Result<u64, Error>` | — |
| `reserve_purchase` | `env: Env, buyer: Address, listing_id: u64` | `Result<u64, Error>` | — |
| `initiate_transaction` | `env: Env, buyer: Address, intent_id: u64` | `Result<u64, Error>` | — |
| `finalize_settlement` | `env: Env, settler: Address, intent_id: u64` | `Result<(i128, i128, i128), Error>` | — |
| `cancel_listing` | `env: Env, actor: Address, listing_id: u64` | `Result<(), Error>` | — |
| `get_provider_count` | `env: Env` | `u64` | — |
| `get_provider` | `env: Env, provider: Address` | `Option<ProviderProfile>` | — |
| `get_listing` | `env: Env, listing_id: u64` | `Option<Listing>` | — |
| `get_intent` | `env: Env, intent_id: u64` | `Option<PurchaseIntent>` | — |
| `purchase_access_tier` | `env: Env, buyer: Address, tier: AccessTier, sut_token: Address, treasury: Address` | `()` | Purchase an access tier by transferring SUT tokens to treasury. Basic is free; Standard costs 100 SUT; Premium costs 1000 SUT. |
| `get_effective_tier` | `env: Env, caller: Address` | `AccessTier` | Get a caller's current tier. Returns Basic if grant is missing or expired. |
| `query_data` | `env: Env, caller: Address` | `Symbol` | Query data — returns granularity matching the caller's tier. Basic: aggregated only. Standard: anonymized records. Premium: full de-identified. |

### Types

#### `enum DataFormat`

| Variant | Value | Description |
|---|---|---|
| `FhirJson` | — | — |
| `Hl7` | — | — |
| `Dicom` | — | — |
| `Csv` | — | — |
| `Parquet` | — | — |
| `Custom` | — | — |

#### `enum AnonymizationLevel`

| Variant | Value | Description |
|---|---|---|
| `KAnonymity` | — | — |
| `DifferentialPrivacy` | — | — |
| `Synthetic` | — | — |

#### `enum ListingStatus`

| Variant | Value | Description |
|---|---|---|
| `Active` | — | — |
| `Reserved` | — | — |
| `Settled` | — | — |
| `Cancelled` | — | — |

#### `struct QualityMetrics`

| Field | Type | Description |
|---|---|---|
| `completeness_bps` | `u32` | — |
| `consistency_bps` | `u32` | — |
| `timeliness_bps` | `u32` | — |
| `validity_bps` | `u32` | — |

#### `struct RoyaltyPolicy`

| Field | Type | Description |
|---|---|---|
| `provider_bps` | `u32` | — |
| `curator_bps` | `u32` | — |
| `platform_bps` | `u32` | — |

#### `struct Config`

| Field | Type | Description |
|---|---|---|
| `admin` | `Address` | — |
| `payment_router` | `Address` | — |
| `escrow_contract` | `Address` | — |
| `treasury` | `Address` | — |
| `settlement_window_secs` | `u64` | — |

#### `struct ProviderProfile`

| Field | Type | Description |
|---|---|---|
| `provider` | `Address` | — |
| `active` | `bool` | — |
| `listings_count` | `u64` | — |
| `reputation_bps` | `u32` | — |

#### `struct Listing`

| Field | Type | Description |
|---|---|---|
| `id` | `u64` | — |
| `provider` | `Address` | — |
| `data_ref` | `String` | — |
| `data_hash` | `BytesN<32>` | — |
| `format` | `DataFormat` | — |
| `anonymization` | `AnonymizationLevel` | — |
| `min_k` | `u32` | — |
| `dp_epsilon_milli` | `u32` | — |
| `quality` | `QualityMetrics` | — |
| `royalty` | `RoyaltyPolicy` | — |
| `price` | `i128` | — |
| `token` | `Address` | — |
| `created_at` | `u64` | — |
| `status` | `ListingStatus` | — |

#### `struct ListingPayload`

| Field | Type | Description |
|---|---|---|
| `data_ref` | `String` | — |
| `data_hash` | `BytesN<32>` | — |
| `format` | `DataFormat` | — |
| `anonymization` | `AnonymizationLevel` | — |
| `min_k` | `u32` | — |
| `dp_epsilon_milli` | `u32` | — |
| `quality` | `QualityMetrics` | — |
| `royalty` | `RoyaltyPolicy` | — |
| `price` | `i128` | — |
| `token` | `Address` | — |

#### `struct PurchaseIntent`

| Field | Type | Description |
|---|---|---|
| `id` | `u64` | — |
| `listing_id` | `u64` | — |
| `buyer` | `Address` | — |
| `amount` | `i128` | — |
| `created_at` | `u64` | — |
| `escrow_order_id` | `Option<u64>` | — |
| `settled` | `bool` | — |

#### `enum DataKey`

| Variant | Value | Description |
|---|---|---|
| `Config` | — | — |
| `ProviderCount` | — | — |
| `Provider(Address)` | — | — |
| `NextListingId` | — | — |
| `Listing(u64)` | — | — |
| `NextIntentId` | — | — |
| `Intent(u64)` | — | — |
| `NextEscrowOrderId` | — | — |

#### `enum Error`

| Variant | Value | Description |
|---|---|---|
| `AlreadyInitialized` | 1 | — |
| `NotInitialized` | 2 | — |
| `Unauthorized` | 3 | — |
| `ProviderNotActive` | 4 | — |
| `ProviderExists` | 5 | — |
| `ListingNotFound` | 6 | — |
| `InvalidPricing` | 7 | — |
| `InvalidQuality` | 8 | — |
| `InvalidRoyalty` | 9 | — |
| `InvalidAnonymization` | 10 | — |
| `InvalidSettlementWindow` | 11 | — |
| `InvalidStatus` | 12 | — |
| `IntentNotFound` | 13 | — |
| `EscrowNotLinked` | 14 | — |
| `SettlementTimeout` | 15 | — |

#### `enum AccessTier`

| Variant | Value | Description |
|---|---|---|
| `Basic` | — | — |
| `Standard` | — | — |
| `Premium` | — | — |

#### `struct TierGrant`

| Field | Type | Description |
|---|---|---|
| `tier` | `AccessTier` | — |
| `granted_at` | `u32` | — |
| `expires_at` | `u32` | — |

#### `enum TierKey`

| Variant | Value | Description |
|---|---|---|
| `Grant(Address)` | — | — |

### Examples

#### `test_create_listing_requires_valid_anonymization_and_quality`

```rust
let env = Env::default();
    env.mock_all_auths();
    let (client, _) = setup(&env);
    let admin = Address::generate(&env);
    let payment_router = Address::generate(&env);
    let escrow = Address::generate(&env);
    let treasury = Address::generate(&env);
    client.initialize(&admin, &payment_router, &escrow, &treasury, &300u64);
```

#### `test_provider_counter_increments`

```rust
let env = Env::default();
    env.mock_all_auths();
    let (client, _) = setup(&env);
    let admin = Address::generate(&env);
    client.initialize(
        &admin,
        &Address::generate(&env),
        &Address::generate(&env),
        &Address::generate(&env),
```

#### `test_settlement_timeout_enforced_under_five_minutes`

```rust
let env = Env::default();
    env.mock_all_auths();
    let (client, _) = setup(&env);
    let admin = Address::generate(&env);
    let payment_router = env.register_contract(None, MockPaymentRouter {});
    let escrow = env.register_contract(None, MockEscrow {});
    client.initialize(
        &admin,
        &payment_router,
```

---

## healthcare_oracle_network

### Functions

| Function | Parameters | Returns | Description |
|---|---|---|---|
| `initialize` | `env: Env, admin: Address, arbiters: Vec<Address>, min_submissions: u32` | `Result<(), Error>` | — |
| `register_oracle` | `env: Env, operator: Address, endpoint: String, source_type: SourceType` | `Result<(), Error>` | — |
| `verify_oracle` | `env: Env, admin: Address, operator: Address, verified: bool, active: bool` | `Result<(), Error>` | — |
| `update_oracle_endpoint` | `env: Env, operator: Address, endpoint: String` | `Result<(), Error>` | — |
| `update_config` | `env: Env, admin: Address, min_submissions: u32, min_reputation: i128, max_drug_price_minor: i128, max_availability_units: u32` | `Result<(), Error>` | — |
| `add_arbiter` | `env: Env, admin: Address, arbiter: Address` | `Result<(), Error>` | — |
| `submit_drug_price` | `env: Env, operator: Address, feed_id: String, ndc_code: String, currency: String, price_minor: i128, availability_units: u32, observed_at: u64` | `Result<u64, Error>` | — |
| `submit_clinical_trial` | `env: Env, operator: Address, trial_id: String, phase: u32, enrolled: u32, success_rate_bps: u32, adverse_event_rate_bps: u32, result_hash: String, published_at: u64` | `Result<u64, Error>` | — |
| `submit_regulatory_update` | `env: Env, operator: Address, regulation_id: String, authority: RegulatoryAuthority, status: RegulatoryStatus, title: String, details_hash: String, effective_at: u64` | `Result<u64, Error>` | — |
| `submit_treatment_outcome` | `env: Env, operator: Address, outcome_id: String, condition_code: String, treatment_code: String, improvement_rate_bps: u32, readmission_rate_bps: u32, mortality_rate_bps: u32, sample_size: u32, reported_at: u64` | `Result<u64, Error>` | — |
| `finalize_feed` | `env: Env, kind: FeedKind, feed_id: String` | `Result<ConsensusRecord, Error>` | — |
| `raise_dispute` | `env: Env, challenger: Address, kind: FeedKind, feed_id: String, reason: String` | `Result<u64, Error>` | — |
| `resolve_dispute` | `env: Env, resolver: Address, dispute_id: u64, valid_dispute: bool, ruling: String, penalized_oracle: Option<Address>` | `Result<(), Error>` | — |
| `get_consensus` | `env: Env, kind: FeedKind, feed_id: String` | `Option<ConsensusRecord>` | — |
| `report_oracle_misbehavior` | `env: Env, reporter: Address, reported_oracle: Address, kind: FeedKind, feed_id: String, reason: String` | `Result<(), Error>` | — |
| `get_oracle` | `env: Env, operator: Address` | `Option<OracleNode>` | — |
| `fetch_external_payload` | `env: Env, provider: Address, feed_id: String` | `Result<FeedPayload, Error>` | — |
| `get_dispute` | `env: Env, dispute_id: u64` | `Option<Dispute>` | — |
| `get_config` | `env: Env` | `Option<Config>` | — |

### Examples

#### `test_oracle_must_be_verified_before_submission`

```rust
let env = Env::default();
    let (client, _admin, _arbiter) = setup_contract(&env, 1);

    let oracle = Address::generate(&env);
    let endpoint = String::from_str(&env, "https://oracle.example");
    let feed_id = String::from_str(&env, "NDC:0002-8215-01:US");
    let ndc = String::from_str(&env, "0002-8215-01");
    let currency = String::from_str(&env, "USD");
```

#### `test_drug_feed_consensus_and_weighted_aggregation`

```rust
let env = Env::default();
    let (client, admin, _arbiter) = setup_contract(&env, 2);

    let oracle_1 = Address::generate(&env);
    let oracle_2 = Address::generate(&env);
    register_and_verify_oracle(&env, &client, &admin, &oracle_1, "https://o1.example");
    register_and_verify_oracle(&env, &client, &admin, &oracle_2, "https://o2.example");

    let feed_id = String::from_str(&env, "NDC:55513-1234-1:KE");
```

#### `test_clinical_trial_and_regulatory_feeds`

```rust
let env = Env::default();
    let (client, admin, _arbiter) = setup_contract(&env, 1);

    let oracle = Address::generate(&env);
    register_and_verify_oracle(&env, &client, &admin, &oracle, "https://clinical.example");

    let trial_id = String::from_str(&env, "NCT-2026-001");
    let hash_a = String::from_str(&env, "sha256:trial-a");
    client.submit_clinical_trial(
```

---

## healthcare_payment

### Functions

| Function | Parameters | Returns | Description |
|---|---|---|---|
| `initialize` | `env: Env, admin: Address, payment_router: Address, escrow_contract: Address, treasury: Address, token: Address, aml_contract: Address, rbac_contract: Address` | `Result<(), Error>` | — |
| `register_insurance_provider` | `env: Env, caller: Address, name: String, payer_code: String, supports_edi_837: bool, supports_edi_834: bool` | `Result<u64, Error>` | — |
| `register_coverage_policy` | `env: Env, caller: Address, patient: Address, insurance_provider_id: u64, policy_external_id: String, member_id: String, group_number: String, deductible_total: i128, copay_amount: i128, coinsurance_bps: u32` | `Result<u64, Error>` | — |
| `verify_insurance_eligibility` | `env: Env, caller: Address, policy_id: u64, service_id: String, coverage_bps: u32, provider_ref: String` | `Result<u64, Error>` | — |
| `submit_claim` | `env: Env, patient: Address, provider: Address, service_id: String, amount: i128, policy_id: String, preauth_id: Option<u64>` | `Result<u64, Error>` | — |
| `submit_insurance_claim` | `env: Env, caller: Address, claim_id: u64, coverage_policy_id: u64, payer_ref: String, transaction_code: String` | `Result<bool, Error>` | — |
| `sync_coverage_enrollment` | `env: Env, caller: Address, coverage_policy_id: u64, enrollment_ref: String, transaction_code: String` | `Result<u64, Error>` | — |
| `verify_claim` | `env: Env, claim_id: u64, verifier: Address` | `Result<(), Error>` | — |
| `approve_claim` | `env: Env, claim_id: u64, approver: Address` | `Result<(), Error>` | — |
| `reject_claim` | `env: Env, claim_id: u64, rejector: Address, reason: String` | `Result<(), Error>` | — |
| `process_eob` | `env: Env, caller: Address, claim_id: u64, coverage_policy_id: u64, allowed_amount: i128, insurer_paid: i128, deductible_applied: i128, adjudication_notes: String, edi_transaction: String` | `Result<bool, Error>` | — |
| `process_payment` | `env: Env, claim_id: u64` | `Result<(), Error>` | — |
| `batch_process_payments` | `env: Env, claim_ids: Vec<u64>` | `Result<Vec<u64>, Error>` | Process multiple approved claims in one call. Reads Config and creates TokenClient once. |
| `escrow_claim` | `env: Env, claim_id: u64` | `Result<(), Error>` | — |
| `request_preauth` | `env: Env, patient: Address, provider: Address, service_id: String, estimated_cost: i128` | `Result<u64, Error>` | — |
| `approve_preauth` | `env: Env, preauth_id: u64, approver: Address` | `Result<(), Error>` | — |
| `report_fraud` | `env: Env, claim_id: u64, reporter: Address, reason: String` | `Result<(), Error>` | — |
| `create_payment_plan` | `env: Env, patient: Address, provider: Address, total_amount: i128, installment_amount: i128, frequency: u64` | `Result<u64, Error>` | — |
| `pay_installment` | `env: Env, plan_id: u64` | `Result<(), Error>` | — |
| `get_coverage_policy` | `env: Env, coverage_policy_id: u64` | `Result<CoveragePolicy, Error>` | — |
| `get_eligibility_check` | `env: Env, eligibility_id: u64` | `Result<EligibilityCheck, Error>` | — |
| `get_claim_submission` | `env: Env, claim_id: u64` | `Result<ClaimSubmission, Error>` | — |
| `get_coverage_enrollment` | `env: Env, enrollment_id: u64` | `Result<CoverageEnrollment, Error>` | — |
| `get_explanation_of_benefits` | `env: Env, claim_id: u64` | `Result<ExplanationOfBenefits, Error>` | — |
| `submit_coverage_proof` | `env: Env, patient: Address, policy_id: u64, proof_hash: BytesN<32>, circuit_version: u32, proven_coverage_bps: u32, expires_at: u64, registry_proof_id: BytesN<32>` | `Result<(), Error>` | Submit a zero-knowledge proof of insurance coverage. The patient proves they have active coverage without revealing sensitive policy details on-chain. |
| `verify_coverage_with_zk` | `env: Env, caller: Address, policy_id: u64, patient: Address` | `Result<u32, Error>` | Verify insurance coverage using a previously submitted ZK proof. Returns the proven coverage basis points if the proof is valid. |
| `get_coverage_proof` | `env: Env, caller: Address, policy_id: u64, patient: Address` | `Result<CoverageProof, Error>` | Get the ZK coverage proof for a patient's policy. Get the ZK coverage proof for a patient's policy. |
| `get_coverage_proof_count` | `env: Env` | `u64` | Get the total number of coverage proofs submitted. |
| `get_patient_responsibility` | `env: Env, patient: Address` | `Option<PatientResponsibility>` | — |
| `emergency_pause` | `env: Env, caller: Address` | `Result<(), Error>` | Immediately open the circuit (emergency stop). Callable by admin or any authorized pauser. |
| `begin_recovery` | `env: Env, caller: Address` | `Result<(), Error>` | Transition circuit from Open -> HalfOpen to begin gradual recovery. Admin only. |
| `resume_operations` | `env: Env, caller: Address` | `Result<(), Error>` | Transition circuit from HalfOpen -> Closed, resetting the failure counter. Admin only. |
| `add_authorized_pauser` | `env: Env, caller: Address, pauser: Address` | `Result<(), Error>` | Grant an address the ability to trigger an emergency pause. Admin only. |
| `remove_authorized_pauser` | `env: Env, caller: Address, pauser: Address` | `Result<(), Error>` | Revoke an address's ability to trigger an emergency pause. Admin only. |
| `report_anomaly` | `env: Env, caller: Address, increment: u32` | `Result<CircuitState, Error>` | Report an anomaly. Increments the internal failure counter and automatically trips the circuit when the threshold is reached. Callable by admin or any authorized pauser (e.g. the anomaly_detection contract). |
| `set_failure_threshold` | `env: Env, caller: Address, threshold: u32` | `Result<(), Error>` | Set the failure threshold for automatic circuit tripping. Admin only. |
| `get_circuit_state` | `env: Env` | `CircuitState` | Returns the current circuit state (defaults to Closed if never set). |
| `get_circuit_breaker` | `env: Env` | `Option<CircuitBreaker>` | Returns the full circuit breaker record. |

### Types

#### `enum ClaimStatus`

| Variant | Value | Description |
|---|---|---|
| `Submitted` | 0 | — |
| `Verified` | 1 | — |
| `Approved` | 2 | — |
| `PendingAMLReview` | 3 | — |
| `Rejected` | 4 | — |
| `Paid` | 5 | — |
| `Disputed` | 6 | — |

#### `enum PreAuthStatus`

| Variant | Value | Description |
|---|---|---|
| `Pending` | 0 | — |
| `Approved` | 1 | — |
| `Denied` | 2 | — |
| `Expired` | 3 | — |

#### `enum PaymentPlanStatus`

| Variant | Value | Description |
|---|---|---|
| `Active` | 0 | — |
| `Completed` | 1 | — |
| `Defaulted` | 2 | — |
| `Cancelled` | 3 | — |

#### `enum CircuitState`

| Variant | Value | Description |
|---|---|---|
| `Closed` | — | — |
| `Open` | — | — |
| `HalfOpen` | — | — |

#### `struct CircuitBreaker`

| Field | Type | Description |
|---|---|---|
| `state` | `CircuitState` | — |
| `failure_count` | `u32` | — |
| `failure_threshold` | `u32` | — |
| `opened_at` | `u64` | — |
| `last_state_change` | `u64` | — |
| `triggered_by` | `Option<Address>` | — |

#### `enum ClaimSubmissionStatus`

| Variant | Value | Description |
|---|---|---|
| `Submitted` | 0 | — |
| `Acknowledged` | 1 | — |
| `Adjudicated` | 2 | — |

#### `struct Claim`

| Field | Type | Description |
|---|---|---|
| `id` | `u64` | — |
| `patient` | `Address` | — |
| `provider` | `Address` | — |
| `service_id` | `String` | — |
| `amount` | `i128` | — |
| `status` | `ClaimStatus` | — |
| `policy_id` | `String` | — |
| `preauth_id` | `Option<u64>` | — |
| `created_at` | `u64` | — |
| `updated_at` | `u64` | — |

#### `struct PreAuth`

| Field | Type | Description |
|---|---|---|
| `id` | `u64` | — |
| `patient` | `Address` | — |
| `provider` | `Address` | — |
| `service_id` | `String` | — |
| `estimated_cost` | `i128` | — |
| `status` | `PreAuthStatus` | — |
| `expiry` | `u64` | — |

#### `struct PaymentPlan`

| Field | Type | Description |
|---|---|---|
| `id` | `u64` | — |
| `patient` | `Address` | — |
| `provider` | `Address` | — |
| `total_amount` | `i128` | — |
| `remaining_amount` | `i128` | — |
| `installment_amount` | `i128` | — |
| `frequency` | `u64` | — |
| `next_due` | `u64` | — |
| `status` | `PaymentPlanStatus` | — |

#### `struct FraudReport`

| Field | Type | Description |
|---|---|---|
| `claim_id` | `u64` | — |
| `reporter` | `Address` | — |
| `reason` | `String` | — |
| `timestamp` | `u64` | — |

#### `struct InsuranceProvider`

| Field | Type | Description |
|---|---|---|
| `id` | `u64` | — |
| `name` | `String` | — |
| `payer_code` | `String` | — |
| `supports_edi_837` | `bool` | — |
| `supports_edi_834` | `bool` | — |
| `active` | `bool` | — |

#### `struct CoveragePolicy`

| Field | Type | Description |
|---|---|---|
| `id` | `u64` | — |
| `patient` | `Address` | — |
| `insurance_provider_id` | `u64` | — |
| `policy_external_id` | `String` | — |
| `member_id` | `String` | — |
| `group_number` | `String` | — |
| `deductible_total` | `i128` | — |
| `deductible_met` | `i128` | — |
| `copay_amount` | `i128` | — |
| `coinsurance_bps` | `u32` | — |
| `coverage_active` | `bool` | — |
| `last_verified_at` | `u64` | — |

#### `struct EligibilityCheck`

| Field | Type | Description |
|---|---|---|
| `id` | `u64` | — |
| `policy_id` | `u64` | — |
| `service_id` | `String` | — |
| `eligible` | `bool` | — |
| `coverage_bps` | `u32` | — |
| `copay_amount` | `i128` | — |
| `deductible_remaining` | `i128` | — |
| `checked_at` | `u64` | — |
| `provider_ref` | `String` | — |

#### `struct ClaimSubmission`

| Field | Type | Description |
|---|---|---|
| `claim_id` | `u64` | — |
| `policy_id` | `u64` | — |
| `submission_format` | `String` | — |
| `transaction_code` | `String` | — |
| `payer_ref` | `String` | — |
| `submitted_at` | `u64` | — |
| `status` | `ClaimSubmissionStatus` | — |

#### `struct CoverageEnrollment`

| Field | Type | Description |
|---|---|---|
| `id` | `u64` | — |
| `policy_id` | `u64` | — |
| `transaction_code` | `String` | — |
| `enrollment_ref` | `String` | — |
| `synced_at` | `u64` | — |

#### `struct ExplanationOfBenefits`

| Field | Type | Description |
|---|---|---|
| `claim_id` | `u64` | — |
| `policy_id` | `u64` | — |
| `allowed_amount` | `i128` | — |
| `insurer_paid` | `i128` | — |
| `patient_responsibility` | `i128` | — |
| `deductible_applied` | `i128` | — |
| `copay_amount` | `i128` | — |
| `adjudication_notes` | `String` | — |
| `processed_at` | `u64` | — |
| `edi_transaction` | `String` | — |

#### `struct CoverageProof`

| Field | Type | Description |
|---|---|---|
| `policy_id` | `u64` | — |
| `patient` | `Address` | — |
| `proof_hash` | `BytesN<32>` | — |
| `circuit_version` | `u32` | — |
| `is_verified` | `bool` | — |
| `proven_coverage_bps` | `u32` | — |
| `submitted_at` | `u64` | — |
| `expires_at` | `u64` | — |
| `registry_proof_id` | `BytesN<32>` | — |

#### `struct PatientResponsibility`

| Field | Type | Description |
|---|---|---|
| `patient` | `Address` | — |
| `total_copay_tracked` | `i128` | — |
| `total_deductible_tracked` | `i128` | — |
| `total_patient_responsibility` | `i128` | — |
| `last_updated` | `u64` | — |

#### `enum RbacRole`

| Variant | Value | Description |
|---|---|---|
| `Admin` | 0 | — |
| `Doctor` | 1 | — |
| `Patient` | 2 | — |
| `Staff` | 3 | — |
| `Insurer` | 4 | — |
| `Researcher` | 5 | — |
| `Auditor` | 6 | — |
| `Service` | 7 | — |

#### `enum RbacError`

| Variant | Value | Description |
|---|---|---|
| `Unauthorized` | 100 | — |
| `NotInitialized` | 300 | — |
| `AlreadyInitialized` | 301 | — |

#### `struct Config`

| Field | Type | Description |
|---|---|---|
| `admin` | `Address` | — |
| `payment_router` | `Address` | — |
| `escrow_contract` | `Address` | — |
| `treasury` | `Address` | — |
| `token` | `Address` | — |
| `aml_contract` | `Address` | — |
| `rbac_contract` | `Address` | — |

#### `enum DataKey`

| Variant | Value | Description |
|---|---|---|
| `Config` | — | — |
| `ClaimCount` | — | — |
| `Claim(u64)` | — | — |
| `PreAuthCount` | — | — |
| `PreAuth(u64)` | — | — |
| `PaymentPlanCount` | — | — |
| `PaymentPlan(u64)` | — | — |
| `FraudReport(u64)` | — | — |
| `InsuranceProviderCount` | — | — |
| `InsuranceProvider(u64)` | — | — |
| `CoveragePolicyCount` | — | — |
| `CoveragePolicy(u64)` | — | — |
| `PolicyByExternalId(String)` | — | — |
| `EligibilityCount` | — | — |
| `Eligibility(u64)` | — | — |
| `LatestEligibilityByPolicy(u64)` | — | — |
| `ClaimSubmission(u64)` | — | — |
| `CoverageEnrollmentCount` | — | — |
| `CoverageEnrollment(u64)` | — | — |
| `Eob(u64)` | — | — |
| `PatientResponsibility(Address)` | — | — |
| `CircuitBreakerState` | — | — |
| `AuthorizedPausers` | — | — |
| `Locked` | — | — |
| `CoverageProof(u64, Address)` | — | — |
| `CoverageProofCount` | — | — |

### Error Codes

| Variant | Code | Description |
|---|---|---|
| `Unauthorized` | 100 | — |
| `UnauthorizedCaller` | 101 | — |
| `NotAuthorizedPauser` | 102 | — |
| `InvalidAmount` | 205 | — |
| `InvalidSignature` | 207 | — |
| `InvalidCoverage` | 280 | — |
| `PolicyMismatch` | 281 | — |
| `NotInitialized` | 300 | — |
| `AlreadyInitialized` | 301 | — |
| `ContractPaused` | 302 | — |
| `CircuitOpen` | 303 | — |
| `InvalidStatus` | 304 | — |
| `AlreadyInState` | 305 | — |
| `DeadlineExceeded` | 306 | — |
| `ClaimNotFound` | 480 | — |
| `PreAuthNotFound` | 481 | — |
| `PaymentPlanNotFound` | 482 | — |
| `InsuranceProviderNotFound` | 483 | — |
| `CoveragePolicyNotFound` | 484 | — |
| `EligibilityCheckNotFound` | 485 | — |
| `ClaimSubmissionNotFound` | 486 | — |
| `EobNotFound` | 487 | — |
| `InsufficientFunds` | 500 | — |
| `StorageFull` | 502 | — |
| `FraudDetected` | 580 | — |
| `EscrowFailed` | 581 | — |
| `UnsupportedTransaction` | 582 | — |
| `CrossChainTimeout` | 702 | — |
| `Reentrancy` | 800 | — |

### Examples

#### `test_submit_and_approve_claim`

```rust
let (env, client, admin, provider, patient, treasury, _, token_client) =
        setup_env_and_clients();

    let claim_id = client.submit_claim(
        &patient,
        &provider,
        &String::from_str(&env, "SERVICE-123"),
        &1000i128,
        &String::from_str(&env, "POLICY-XYZ"),
```

#### `test_escrow_claim`

```rust
let (env, client, admin, provider, patient, _, _, _) = setup_env_and_clients();

    let claim_id = client.submit_claim(
        &patient,
        &provider,
        &String::from_str(&env, "SERVICE-456"),
        &2000i128,
        &String::from_str(&env, "POLICY-ABC"),
        &None,
```

#### `test_fraud_report`

```rust
let (env, client, admin, provider, patient, _, _, _) = setup_env_and_clients();

    let claim_id = client.submit_claim(
        &patient,
        &provider,
        &String::from_str(&env, "SERVICE-789"),
        &3000i128,
        &String::from_str(&env, "POLICY-DEF"),
        &None,
```

---

## healthcare_reputation

### Functions

| Function | Parameters | Returns | Description |
|---|---|---|---|
| `initialize` | `env: Env, admin: Address` | `Result<(), Error>` | — |
| `add_credential` | `env: Env, provider: Address, credential_id: BytesN<32>, credential_type: CredentialType, issuer: Address, issue_date: u64, expiration_date: u64, credential_hash: BytesN<32>` | `Result<(), Error>` | — |
| `verify_credential` | `env: Env, admin: Address, provider: Address, credential_id: BytesN<32>, verified: bool` | `Result<(), Error>` | — |
| `add_feedback` | `env: Env, provider: Address, patient: Address, rating: u32, comment: String, feedback_type: FeedbackType` | `Result<(), Error>` | — |
| `add_conduct_entry` | `env: Env, reporter: Address, provider: Address, conduct_type: ConductType, description: String, severity: u32, action_taken: String` | `Result<(), Error>` | — |
| `create_dispute` | `env: Env, challenger: Address, provider: Address, dispute_type: DisputeType, description: String, evidence: Vec<String>` | `Result<(), Error>` | — |
| `resolve_dispute` | `env: Env, admin: Address, dispute_id: BytesN<32>, approved: bool, resolution: String` | `Result<(), Error>` | — |
| `get_reputation_score` | `env: Env, provider: Address` | `Result<u32, Error>` | — |
| `get_reputation_components` | `env: Env, provider: Address` | `Result<ReputationComponents, Error>` | — |
| `check_reputation_threshold` | `env: Env, provider: Address, threshold: u32` | `Result<bool, Error>` | — |
| `get_provider_credentials` | `env: Env, provider: Address` | `Result<Vec<ProviderCredential>, Error>` | — |
| `get_provider_feedback` | `env: Env, provider: Address` | `Result<Vec<PatientFeedback>, Error>` | — |
| `check_expired_credentials` | `env: Env, provider: Address` | `Result<Vec<BytesN<32>>, Error>` | — |

### Types

#### `enum Error`

| Variant | Value | Description |
|---|---|---|
| `AlreadyInitialized` | 1 | — |
| `NotInitialized` | 2 | — |
| `NotAuthorized` | 3 | — |
| `ProviderNotFound` | 4 | — |
| `CredentialNotFound` | 5 | — |
| `InvalidCredentialType` | 6 | — |
| `CredentialExpired` | 7 | — |
| `CredentialRevoked` | 8 | — |
| `DuplicateCredential` | 9 | — |
| `InvalidRating` | 10 | — |
| `FeedbackNotFound` | 11 | — |
| `DisputeNotFound` | 12 | — |
| `InsufficientReputation` | 13 | — |
| `NotVerifiedProvider` | 14 | — |
| `InvalidConductEntry` | 15 | — |
| `ConductEntryNotFound` | 16 | — |

#### `enum CredentialType`

| Variant | Value | Description |
|---|---|---|
| `MedicalLicense` | 0 | — |
| `BoardCertification` | 1 | — |
| `Specialization` | 2 | — |
| `DEARegistration` | 3 | — |
| `StateLicense` | 4 | — |
| `HospitalPrivileges` | 5 | — |
| `InsuranceCredentials` | 6 | — |
| `ContinuingEducation` | 7 | — |

#### `struct ProviderCredential`

| Field | Type | Description |
|---|---|---|
| `credential_id` | `BytesN<32>` | — |
| `provider` | `Address` | — |
| `credential_type` | `CredentialType` | — |
| `issuer` | `Address` | — |
| `issue_date` | `u64` | — |
| `expiration_date` | `u64` | — |
| `credential_hash` | `BytesN<32>` | — |
| `is_active` | `bool` | — |
| `verification_status` | `VerificationStatus` | — |

#### `enum VerificationStatus`

| Variant | Value | Description |
|---|---|---|
| `Pending` | 0 | — |
| `Verified` | 1 | — |
| `Rejected` | 2 | — |
| `Expired` | 3 | — |
| `Revoked` | 4 | — |

#### `struct PatientFeedback`

| Field | Type | Description |
|---|---|---|
| `feedback_id` | `BytesN<32>` | — |
| `provider` | `Address` | — |
| `patient` | `Address` | — |
| `rating` | `u32` | — |
| `comment` | `String` | — |
| `timestamp` | `u64` | — |
| `is_verified` | `bool` | — |
| `feedback_type` | `FeedbackType` | — |

#### `enum FeedbackType`

| Variant | Value | Description |
|---|---|---|
| `General` | 0 | — |
| `Treatment` | 1 | — |
| `Communication` | 2 | — |
| `BedsideManner` | 3 | — |
| `WaitTime` | 4 | — |
| `Facility` | 5 | — |

#### `struct ConductEntry`

| Field | Type | Description |
|---|---|---|
| `entry_id` | `BytesN<32>` | — |
| `provider` | `Address` | — |
| `conduct_type` | `ConductType` | — |
| `description` | `String` | — |
| `severity` | `u32` | — |
| `reporter` | `Address` | — |
| `timestamp` | `u64` | — |
| `is_verified` | `bool` | — |
| `action_taken` | `String` | — |

#### `enum ConductType`

| Variant | Value | Description |
|---|---|---|
| `Positive` | 0 | — |
| `Complaint` | 1 | — |
| `Malpractice` | 2 | — |
| `EthicsViolation` | 3 | — |
| `ProfessionalAchievement` | 4 | — |
| `CommunityService` | 5 | — |

#### `struct ReputationDispute`

| Field | Type | Description |
|---|---|---|
| `dispute_id` | `BytesN<32>` | — |
| `provider` | `Address` | — |
| `challenger` | `Address` | — |
| `dispute_type` | `DisputeType` | — |
| `description` | `String` | — |
| `evidence` | `Vec<String>` | — |
| `timestamp` | `u64` | — |
| `status` | `DisputeStatus` | — |
| `resolution` | `String` | — |

#### `enum DisputeType`

| Variant | Value | Description |
|---|---|---|
| `Credential` | 0 | — |
| `Feedback` | 1 | — |
| `Conduct` | 2 | — |
| `Score` | 3 | — |

#### `enum DisputeStatus`

| Variant | Value | Description |
|---|---|---|
| `Pending` | 0 | — |
| `UnderReview` | 1 | — |
| `Resolved` | 2 | — |
| `Rejected` | 3 | — |

#### `struct ReputationComponents`

| Field | Type | Description |
|---|---|---|
| `credential_score` | `u32` | — |
| `feedback_score` | `u32` | — |
| `conduct_score` | `u32` | — |
| `experience_score` | `u32` | — |
| `total_score` | `u32` | — |

#### `enum DataKey`

| Variant | Value | Description |
|---|---|---|
| `Admin` | — | — |
| `Initialized` | — | — |
| `ProviderCredential(Address, BytesN<32>)` | — | — |
| `ProviderCredentials(Address)` | — | — |
| `PatientFeedback(BytesN<32>)` | — | — |
| `ProviderFeedback(Address)` | — | — |
| `ConductEntry(BytesN<32>)` | — | — |
| `ProviderConduct(Address)` | — | — |
| `ReputationDispute(BytesN<32>)` | — | — |
| `ProviderDisputes(Address)` | — | — |
| `ReputationScore(Address)` | — | — |
| `ReputationComponents(Address)` | — | — |
| `CredentialVerification(BytesN<32>)` | — | — |
| `ExpirationNotification(Address, u64)` | — | — |

---

## homomorphic_registry

### Functions

| Function | Parameters | Returns | Description |
|---|---|---|---|
| `initialize` | `env: Env, admin: Address` | `Result<(), Error>` | — |
| `register_key_bundle` | `env: Env, admin: Address, key_id: BytesN<32>, context_id: BytesN<32>, public_key_ref: String, eval_key_ref: String, relin_key_ref: String, galois_key_ref: String, key_hash: BytesN<32>` | `Result<(), Error>` | — |
| `get_active_key_bundle` | `env: Env, context_id: BytesN<32>` | `Result<Option<FHEKeyBundle>, Error>` | — |
| `set_performance_profile` | `env: Env, admin: Address, context_id: BytesN<32>, batching_enabled: bool, max_batch_size: u32, lazy_relinearization: bool, auto_bootstrap: bool, bootstrap_threshold: u32` | `Result<(), Error>` | — |
| `get_performance_profile` | `env: Env, context_id: BytesN<32>` | `Result<Option<PerformanceProfile>, Error>` | — |
| `encrypt_ckks_vector` | `env: Env, submitter: Address, ciphertext_id: BytesN<32>, context_id: BytesN<32>, values: Vec<i128>, scale: u32` | `Result<(), Error>` | — |
| `encrypt_bgv_vector` | `env: Env, submitter: Address, ciphertext_id: BytesN<32>, context_id: BytesN<32>, values: Vec<i128>` | `Result<(), Error>` | — |
| `fhe_add` | `env: Env, submitter: Address, output_id: BytesN<32>, left_id: BytesN<32>, right_id: BytesN<32>` | `Result<(), Error>` | — |
| `fhe_multiply` | `env: Env, submitter: Address, output_id: BytesN<32>, left_id: BytesN<32>, right_id: BytesN<32>` | `Result<(), Error>` | — |
| `bootstrap_ciphertext` | `env: Env, admin: Address, ciphertext_id: BytesN<32>` | `Result<(), Error>` | — |
| `get_ciphertext` | `env: Env, ciphertext_id: BytesN<32>` | `Result<Option<EncryptedVector>, Error>` | — |
| `encrypted_statistics` | `env: Env, submitter: Address, ciphertext_id: BytesN<32>` | `Result<EncryptedStats, Error>` | — |
| `encrypted_linear_inference` | `env: Env, submitter: Address, output_id: BytesN<32>, features_id: BytesN<32>, model_weights: Vec<i128>, bias: i128` | `Result<(), Error>` | — |
| `estimate_operation_cost` | `env: Env, context_id: BytesN<32>, multiplicative_depth: u32, slot_count: u32` | `Result<u64, Error>` | — |
| `register_context` | `env: Env, admin: Address, context_id: BytesN<32>, scheme: HEScheme, params_ref: String, params_hash: BytesN<32>` | `Result<(), Error>` | — |
| `deactivate_context` | `env: Env, admin: Address, context_id: BytesN<32>` | `Result<(), Error>` | — |
| `submit_encrypted_computation` | `env: Env, submitter: Address, computation_id: BytesN<32>, context_id: BytesN<32>, ciphertext_ref: String, ciphertext_hash: BytesN<32>, proof_ref: String, proof_hash: BytesN<32>` | `Result<(), Error>` | — |
| `get_context` | `env: Env, context_id: BytesN<32>` | `Result<Option<HEContext>, Error>` | — |
| `get_computation` | `env: Env, computation_id: BytesN<32>` | `Result<Option<EncryptedComputation>, Error>` | — |

### Types

#### `enum HEScheme`

| Variant | Value | Description |
|---|---|---|
| `Paillier` | — | — |
| `BFV` | — | — |
| `BGV` | — | — |
| `CKKS` | — | — |
| `TFHE` | — | — |
| `Custom(u32)` | — | — |

#### `struct HEContext`

| Field | Type | Description |
|---|---|---|
| `context_id` | `BytesN<32>` | — |
| `scheme` | `HEScheme` | — |
| `params_ref` | `String` | — |
| `params_hash` | `BytesN<32>` | — |
| `created_at` | `u64` | — |
| `is_active` | `bool` | — |

#### `struct EncryptedComputation`

| Field | Type | Description |
|---|---|---|
| `computation_id` | `BytesN<32>` | — |
| `context_id` | `BytesN<32>` | — |
| `submitter` | `Address` | — |
| `ciphertext_ref` | `String` | — |
| `ciphertext_hash` | `BytesN<32>` | — |
| `proof_ref` | `String` | — |
| `proof_hash` | `BytesN<32>` | — |
| `submitted_at` | `u64` | — |

#### `struct FHEKeyBundle`

| Field | Type | Description |
|---|---|---|
| `key_id` | `BytesN<32>` | — |
| `context_id` | `BytesN<32>` | — |
| `version` | `u32` | — |
| `public_key_ref` | `String` | — |
| `eval_key_ref` | `String` | — |
| `relin_key_ref` | `String` | — |
| `galois_key_ref` | `String` | — |
| `key_hash` | `BytesN<32>` | — |
| `created_at` | `u64` | — |
| `is_active` | `bool` | — |

#### `struct PerformanceProfile`

| Field | Type | Description |
|---|---|---|
| `context_id` | `BytesN<32>` | — |
| `batching_enabled` | `bool` | — |
| `max_batch_size` | `u32` | — |
| `lazy_relinearization` | `bool` | — |
| `auto_bootstrap` | `bool` | — |
| `bootstrap_threshold` | `u32` | — |
| `created_at` | `u64` | — |
| `updated_at` | `u64` | — |

#### `struct EncryptedVector`

| Field | Type | Description |
|---|---|---|
| `ciphertext_id` | `BytesN<32>` | — |
| `context_id` | `BytesN<32>` | — |
| `owner` | `Address` | — |
| `scheme` | `HEScheme` | — |
| `scale` | `u32` | — |
| `noise_budget` | `u32` | — |
| `multiplicative_depth` | `u32` | — |
| `slots` | `Vec<i128>` | — |
| `created_at` | `u64` | — |
| `last_bootstrapped_at` | `u64` | — |

#### `struct EncryptedStats`

| Field | Type | Description |
|---|---|---|
| `ciphertext_id` | `BytesN<32>` | — |
| `count` | `u32` | — |
| `sum` | `i128` | — |
| `mean_scaled` | `i128` | — |
| `variance_scaled` | `i128` | — |
| `min` | `i128` | — |
| `max` | `i128` | — |

#### `enum DataKey`

| Variant | Value | Description |
|---|---|---|
| `Initialized` | — | — |
| `Admin` | — | — |
| `Context(BytesN<32>)` | — | — |
| `Computation(BytesN<32>)` | — | — |
| `KeyBundle(BytesN<32>)` | — | — |
| `ActiveKey(BytesN<32>)` | — | — |
| `Ciphertext(BytesN<32>)` | — | — |
| `Profile(BytesN<32>)` | — | — |

#### `enum Error`

| Variant | Value | Description |
|---|---|---|
| `AlreadyInitialized` | 1 | — |
| `NotInitialized` | 2 | — |
| `NotAuthorized` | 3 | — |
| `ContextNotFound` | 4 | — |
| `ContextInactive` | 5 | — |
| `InvalidInput` | 6 | — |
| `ComputationAlreadyExists` | 7 | — |
| `CiphertextNotFound` | 8 | — |
| `CiphertextAlreadyExists` | 9 | — |
| `SchemeMismatch` | 10 | — |
| `IncompatibleDimensions` | 11 | — |
| `NoiseBudgetExhausted` | 12 | — |
| `ArithmeticOverflow` | 13 | — |
| `KeyNotFound` | 14 | — |

### Examples

#### `context_and_submission_flow`

```rust
let env = Env::default();
    env.mock_all_auths();

    let (client, _id) = setup(&env);
    let admin = Address::generate(&env);
    client.initialize(&admin);

    let ctx_id = BytesN::from_array(&env, &[7u8; 32]);
    let params_ref = String::from_str(&env, "ipfs://he-params");
```

#### `ckks_secure_stats_and_ml_inference_flow`

```rust
let env = Env::default();
    env.mock_all_auths();
    env.ledger().set_timestamp(1_000);
    let (client, _id) = setup(&env);

    let admin = Address::generate(&env);
    let analyst = Address::generate(&env);
    client.initialize(&admin);
```

#### `bgv_exact_computation_and_noise_bootstrap`

```rust
let env = Env::default();
    env.mock_all_auths();
    env.ledger().set_timestamp(2_000);
    let (client, _id) = setup(&env);

    let admin = Address::generate(&env);
    let submitter = Address::generate(&env);
    client.initialize(&admin);
```

---

## identity_registry

### Functions

| Function | Parameters | Returns | Description |
|---|---|---|---|
| `initialize` | `env: Env, owner: Address, network_id: String, rbac_contract: Address` | `Result<(), Error>` | Initialize the contract with an owner and network identifier |
| `health_check` | `env: Env` | `(Symbol, u32, u64)` | Perform a health check on the contract |
| `is_paused` | `env: Env` | `bool` | Returns true if the contract is currently paused. |
| `pause` | `env: Env, caller: Address` | `Result<bool, Error>` | — |
| `unpause` | `env: Env, caller: Address` | `Result<bool, Error>` | — |
| `initialize_legacy` | `env: Env, owner: Address, rbac_contract: Address` | `()` | Legacy initialize for backward compatibility |
| `create_did` | `env: Env, subject: Address, primary_public_key: BytesN<32>, services: Vec<ServiceEndpoint>` | `Result<String, Error>` | Create a new DID Document for a subject Only the subject can create their own DID |
| `resolve_did` | `env: Env, subject: Address` | `Result<DIDDocument, Error>` | Resolve a DID Document by subject address |
| `resolve_did_by_string` | `env: Env, did_string: String` | `Result<DIDDocument, Error>` | Resolve a DID Document by DID string |
| `update_did` | `env: Env, subject: Address, new_services: Vec<ServiceEndpoint>, new_also_known_as: Vec<String>` | `Result<(), Error>` | Update DID Document (add/modify services, also_known_as) |
| `deactivate_did` | `env: Env, subject: Address` | `Result<(), Error>` | Deactivate a DID (soft delete) |
| `add_verification_method` | `env: Env, subject: Address, method_id: String, method_type: VerificationMethodType, public_key: BytesN<32>, relationships: Vec<VerificationRelationship>` | `Result<(), Error>` | Add a new verification method to a DID |
| `rotate_key` | `env: Env, subject: Address, method_id: String, new_public_key: BytesN<32>` | `Result<(), Error>` | Rotate a verification method key |
| `revoke_verification_method` | `env: Env, subject: Address, method_id: String` | `Result<(), Error>` | Revoke/deactivate a verification method |
| `issue_credential` | `env: Env, issuer: Address, subject: Address, credential_type: CredentialType, credential_hash: BytesN<32>, credential_uri: String, expiration_date: u64` | `Result<BytesN<32>, Error>` | Issue a verifiable credential (only verifiers/issuers can do this) |
| `verify_credential` | `env: Env, credential_id: BytesN<32>` | `Result<CredentialStatus, Error>` | Verify a credential's status |
| `get_credential` | `env: Env, credential_id: BytesN<32>` | `Result<VerifiableCredential, Error>` | Get a credential by ID |
| `revoke_credential` | `env: Env, issuer: Address, credential_id: BytesN<32>, reason: String` | `Result<(), Error>` | Revoke a credential (only issuer can revoke) |
| `get_subject_credentials` | `env: Env, subject: Address` | `Vec<VerifiableCredential>` | Get all credentials for a subject |
| `has_valid_credential` | `env: Env, subject: Address, credential_type: CredentialType` | `bool` | Verify if subject has a valid credential of a specific type |
| `add_recovery_guardian` | `env: Env, subject: Address, guardian: Address, weight: u32` | `Result<(), Error>` | Add a recovery guardian |
| `remove_recovery_guardian` | `env: Env, subject: Address, guardian: Address` | `Result<(), Error>` | Remove a recovery guardian |
| `set_recovery_threshold` | `env: Env, subject: Address, threshold: u32` | `Result<(), Error>` | Set recovery threshold |
| `initiate_recovery` | `env: Env, guardian: Address, subject: Address, new_controller: Address, new_primary_key: BytesN<32>` | `Result<u64, Error>` | Initiate identity recovery |
| `approve_recovery` | `env: Env, guardian: Address, request_id: u64` | `Result<(), Error>` | Approve a recovery request |
| `execute_recovery` | `env: Env, request_id: u64` | `Result<(), Error>` | Execute recovery after timelock and threshold met |
| `cancel_recovery` | `env: Env, subject: Address` | `Result<(), Error>` | Cancel a recovery request (only subject with existing key) |
| `add_service` | `env: Env, subject: Address, service_id: String, service_type: String, endpoint: String` | `Result<(), Error>` | Add a service endpoint to DID |
| `remove_service` | `env: Env, subject: Address, service_id: String` | `Result<(), Error>` | Remove/deactivate a service endpoint |
| `add_verifier` | `env: Env, verifier: Address` | `Result<(), Error>` | Add a verifier (only owner can do this) |
| `remove_verifier` | `env: Env, verifier: Address` | `Result<(), Error>` | Remove a verifier (only owner can do this) |
| `is_verifier` | `env: Env, account: Address` | `bool` | Check if an address is a verifier |
| `get_owner` | `env: Env` | `Result<Address, Error>` | Get the contract owner |
| `register_identity_hash` | `env: Env, hash: BytesN<32>, subject: Address, meta: String` | `()` | Register an identity hash with metadata (legacy support) |
| `attest` | `env: Env, verifier: Address, subject: Address, claim_hash: BytesN<32>` | `()` | Create an attestation (legacy - only verifiers can do this) |
| `revoke_attestation` | `env: Env, verifier: Address, subject: Address, claim_hash: BytesN<32>` | `()` | Revoke an attestation (legacy) |
| `get_identity_hash` | `env: Env, subject: Address` | `Option<BytesN<32>>` | Get identity hash for a subject (legacy) |
| `get_identity_meta` | `env: Env, subject: Address` | `Option<String>` | Get identity metadata for a subject (legacy) |
| `is_attested` | `env: Env, subject: Address, claim_hash: BytesN<32>` | `bool` | Check if a specific attestation is active (legacy) |
| `get_attestations` | `env: Env, subject: Address` | `Vec<BytesN<32>>` | Get all active attestations for a subject (legacy) |
| `verify_did_authorization` | `env: Env, subject: Address, required_relationship: VerificationRelationship` | `bool` | DID-based authorization check |
| `add_fido2_device` | `env: Env, subject: Address, device_name: String, algorithm_tag: u32, public_key_hash: BytesN<32>` | `Result<(), Error>` | Registers a FIDO2 / WebAuthn authenticator device as a verification method in the subject's DID document.  Called by the `fido2_authenticator` contract after a successful device registration ceremony.  The public key is stored as a SHA-256 hash (`public_key_hash`) because DID verification methods use 32-byte keys and FIDO2 P-256 keys are 65 bytes; the hash acts as a stable, compact identifier.  # Arguments * `subject`          — DID owner; must have an active DID document. * `device_name`      — friendly name used as the verification method fragment ID. * `algorithm_tag`    — 1 = EdDSA (Ed25519), 2 = ES256 (P-256). * `public_key_hash`  — SHA-256 of the raw authenticator public key bytes.  If the subject has no DID document the call is silently ignored so that the `fido2_authenticator` registration is never blocked by DID state. |
| `deposit_stake` | `env: Env, provider: Address, amount: i128, token_address: Address` | `Result<(), Error>` | Deposit stake for a healthcare provider. |
| `withdraw_stake` | `env: Env, provider: Address` | `Result<i128, Error>` | Withdraw stake after lock period if not slashed and in good standing. |
| `slash_stake` | `env: Env, governance: Address, provider: Address, amount: i128, reason: String` | `Result<(), Error>` | Slash stake for verified misconduct (governance only). |
| `init_mock` | `_env: Env` | `()` | — |
| `has_role` | `env: Env, address: Address, role: RbacRole` | `Result<bool, RbacError>` | — |
| `assign_role` | `env: Env, address: Address, role: RbacRole` | `Result<bool, RbacError>` | — |
| `remove_role` | `env: Env, address: Address, role: RbacRole` | `Result<bool, RbacError>` | — |

### Types

#### `enum RbacRole`

| Variant | Value | Description |
|---|---|---|
| `Admin` | 0 | — |
| `Doctor` | 1 | — |
| `Patient` | 2 | — |
| `Staff` | 3 | — |
| `Insurer` | 4 | — |
| `Researcher` | 5 | — |
| `Auditor` | 6 | — |
| `Service` | 7 | — |

#### `enum RbacError`

| Variant | Value | Description |
|---|---|---|
| `Unauthorized` | 100 | — |
| `NotInitialized` | 300 | — |
| `AlreadyInitialized` | 301 | — |

#### `enum VerificationMethodType`

| Variant | Value | Description |
|---|---|---|
| `Ed25519VerificationKey2020` | — | — |
| `EcdsaSecp256k1VerifKey2019` | — | — |
| `X25519KeyAgreementKey2020` | — | — |
| `JsonWebKey2020` | — | — |
| `Fido2EdDsa2024` | — | — |
| `Fido2Es2562024` | — | — |

#### `enum VerificationRelationship`

| Variant | Value | Description |
|---|---|---|
| `Authentication` | — | — |
| `AssertionMethod` | — | — |
| `KeyAgreement` | — | — |
| `CapabilityInvocation` | — | — |
| `CapabilityDelegation` | — | — |

#### `struct VerificationMethod`

| Field | Type | Description |
|---|---|---|
| `id` | `String` | — |
| `method_type` | `VerificationMethodType` | — |
| `controller` | `Address` | — |
| `public_key` | `BytesN<32>` | — |
| `is_active` | `bool` | — |
| `created` | `u64` | — |
| `last_rotated` | `u64` | — |

#### `struct ServiceEndpoint`

| Field | Type | Description |
|---|---|---|
| `id` | `String` | — |
| `service_type` | `String` | — |
| `endpoint` | `String` | — |
| `is_active` | `bool` | — |

#### `enum DIDStatus`

| Variant | Value | Description |
|---|---|---|
| `Active` | — | — |
| `Deactivated` | — | — |
| `RecoveryPending` | — | — |

#### `struct DIDDocument`

| Field | Type | Description |
|---|---|---|
| `id` | `String` | — |
| `controller` | `Address` | — |
| `also_known_as` | `Vec<String>` | — |
| `verification_methods` | `Vec<VerificationMethod>` | — |
| `authentication` | `Vec<String>` | — |
| `assertion_method` | `Vec<String>` | — |
| `key_agreement` | `Vec<String>` | — |
| `capability_invocation` | `Vec<String>` | — |
| `capability_delegation` | `Vec<String>` | — |
| `services` | `Vec<ServiceEndpoint>` | — |
| `status` | `DIDStatus` | — |
| `created` | `u64` | — |
| `updated` | `u64` | — |
| `version` | `u32` | — |
| `previous_hash` | `BytesN<32>` | — |

#### `enum CredentialType`

| Variant | Value | Description |
|---|---|---|
| `MedicalLicense` | — | — |
| `SpecialistCertification` | — | — |
| `HospitalAffiliation` | — | — |
| `ResearchAuthorization` | — | — |
| `PatientConsent` | — | — |
| `EmergencyAccess` | — | — |
| `DataAccessPermission` | — | — |

#### `struct VerifiableCredential`

| Field | Type | Description |
|---|---|---|
| `id` | `BytesN<32>` | — |
| `credential_type` | `CredentialType` | — |
| `issuer` | `Address` | — |
| `subject` | `Address` | — |
| `issuance_date` | `u64` | — |
| `expiration_date` | `u64` | — |
| `credential_hash` | `BytesN<32>` | — |
| `credential_uri` | `String` | — |
| `is_revoked` | `bool` | — |
| `revoked_at` | `u64` | — |
| `revocation_reason` | `String` | — |

#### `enum CredentialStatus`

| Variant | Value | Description |
|---|---|---|
| `Valid` | — | — |
| `Revoked` | — | — |
| `Expired` | — | — |
| `NotFound` | — | — |

#### `struct RecoveryGuardian`

| Field | Type | Description |
|---|---|---|
| `address` | `Address` | — |
| `weight` | `u32` | — |
| `added_at` | `u64` | — |

#### `struct RecoveryRequest`

| Field | Type | Description |
|---|---|---|
| `request_id` | `u64` | — |
| `subject` | `Address` | — |
| `new_controller` | `Address` | — |
| `new_primary_key` | `BytesN<32>` | — |
| `created_at` | `u64` | — |
| `approvals` | `Vec<Address>` | — |
| `total_weight` | `u32` | — |
| `executed` | `bool` | — |

#### `struct IdentityRecord`

| Field | Type | Description |
|---|---|---|
| `hash` | `BytesN<32>` | — |
| `meta` | `String` | — |
| `registered_by` | `Address` | — |

#### `struct Attestation`

| Field | Type | Description |
|---|---|---|
| `claim_hash` | `BytesN<32>` | — |
| `verifier` | `Address` | — |
| `is_active` | `bool` | — |

#### `struct ProviderStake`

| Field | Type | Description |
|---|---|---|
| `provider` | `Address` | — |
| `token_address` | `Address` | — |
| `amount` | `i128` | — |
| `locked_until` | `u64` | — |
| `slashed` | `bool` | — |
| `deposited_at` | `u64` | — |

#### `enum DataKey`

| Variant | Value | Description |
|---|---|---|
| `Owner` | — | — |
| `Initialized` | — | — |
| `NetworkId` | — | — |
| `RbacContract` | — | — |
| `Paused` | — | — |
| `Verifier(Address)` | — | — |
| `IdentityHash(Address)` | — | — |
| `Attestation(Address, BytesN<32>)` | — | — |
| `SubjectAttestations(Address)` | — | — |
| `DIDDocument(Address)` | — | — |
| `DIDByString(String)` | — | — |
| `VerificationMethod(Address, String)` | — | — |
| `Credential(BytesN<32>)` | — | — |
| `SubjectCredentials(Address)` | — | — |
| `IssuerCredentials(Address)` | — | — |
| `CredentialsByType(Address, CredentialType)` | — | — |
| `RecoveryGuardians(Address)` | — | — |
| `RecoveryThreshold(Address)` | — | — |
| `RecoveryRequest(u64)` | — | — |
| `ActiveRecovery(Address)` | — | — |
| `RecoveryCounter` | — | — |
| `LastKeyRotation(Address)` | — | — |
| `KeyRotationCooldown` | — | — |
| `StakeInfo(Address)` | — | — |

### Error Codes

| Variant | Code | Description |
|---|---|---|
| `Unauthorized` | 100 | — |
| `NotVerifier` | 110 | — |
| `CannotRemoveOwner` | 111 | — |
| `InvalidRecoveryGuardian` | 120 | — |
| `InsufficientGuardianApprovals` | 121 | — |
| `InvalidInput` | 200 | — |
| `InputTooLong` | 201 | — |
| `InvalidVerificationMethod` | 250 | — |
| `InvalidCredentialType` | 251 | — |
| `InvalidServiceEndpoint` | 252 | — |
| `NotInitialized` | 300 | — |
| `AlreadyInitialized` | 301 | — |
| `ContractPaused` | 302 | — |
| `RecoveryNotInitiated` | 360 | — |
| `RecoveryAlreadyPending` | 361 | — |
| `RecoveryTimelockNotElapsed` | 362 | — |
| `VerificationMethodNotFound` | 450 | — |
| `CredentialNotFound` | 460 | — |
| `AttestationNotFound` | 461 | — |
| `ServiceNotFound` | 462 | — |
| `DIDNotFound` | 470 | — |
| `DIDAlreadyExists` | 471 | — |
| `DIDDeactivated` | 472 | — |
| `CredentialExpired` | 605 | — |
| `CredentialRevoked` | 606 | — |
| `KeyRotationCooldown` | 603 | — |

---

## ihe_integration

### Functions

| Function | Parameters | Returns | Description |
|---|---|---|---|
| `initialize` | `env: Env, admin: Address` | `Result<(), Error>` | — |
| `xds_register_document` | `env: Env, author: Address, entry: XDSDocumentEntry` | `Result<(), Error>` | Register a new document in the XDS registry |
| `xds_deprecate_document` | `env: Env, author: Address, document_id: String` | `Result<(), Error>` | Deprecate an existing XDS document entry |
| `xds_query_documents` | `env: Env, requester: Address, patient_id: String` | `Result<Vec<XDSDocumentEntry>, Error>` | Query XDS documents for a patient |
| `xds_retrieve_document` | `env: Env, requester: Address, document_id: String` | `Result<XDSDocumentEntry, Error>` | Retrieve a single XDS document entry |
| `xds_submit_document_set` | `env: Env, author: Address, submission_set: XDSSubmissionSet` | `Result<(), Error>` | Submit an XDS submission set (groups documents from one clinical event) |
| `pix_register_patient` | `env: Env, actor: Address, local_id: PatientIdentifier, cross_ids: Vec<PatientIdentifier>` | `Result<u64, Error>` | Register a patient identity and return the cross-reference record ID |
| `pix_query_identifiers` | `env: Env, requester: Address, patient_id: String` | `Result<Vec<PIXCrossReference>, Error>` | Query all cross-referenced identifiers for a patient |
| `pix_merge_patients` | `env: Env, actor: Address, surviving_ref_id: u64, subsumed_ref_id: u64` | `Result<(), Error>` | Merge two patient identities (PIX merge operation) |
| `pdq_register_demographics` | `env: Env, actor: Address, demographics: PatientDemographics` | `Result<(), Error>` | Register or update patient demographics |
| `pdq_query` | `env: Env, requester: Address, query_params: Map<String, String>, requesting_system: String, hl7_type: HL7MessageType, domain_filter: String` | `Result<u64, Error>` | Execute a PDQ demographics query; returns matching records |
| `pdq_get_demographics` | `env: Env, requester: Address, patient_id: String` | `Result<PatientDemographics, Error>` | Retrieve patient demographics by patient ID |
| `atna_log_event` | `env: Env, actor: Address, event_type: ATNAEventType, event_action_code: String, event_outcome: ATNAEventOutcome, source_id: String, source_type: String, active_participants: Vec<ATNAParticipant>, participant_objects: Vec<ATNAParticipantObject>, hl7_message_id: String, profile: IHEProfile` | `Result<u64, Error>` | Log an ATNA-compliant audit event (used by external actors and other profiles) |
| `atna_get_event` | `env: Env, event_id: u64` | `Result<ATNAAuditEvent, Error>` | Retrieve an ATNA audit event by ID |
| `atna_authenticate_node` | `env: Env, node: Address, node_id: String, certificate_hash: BytesN<32>` | `Result<u64, Error>` | Authenticate a node and record the ATNA authentication event |
| `xca_register_gateway` | `env: Env, admin: Address, gateway: XCAGateway` | `Result<(), Error>` | Register a cross-community gateway |
| `xca_initiate_query` | `env: Env, requester: Address, gateway_id: String, patient_id: String` | `Result<XCAGateway, Error>` | Initiate a cross-gateway query (returns gateway record for routing) |
| `mpi_register_master_patient` | `env: Env, actor: Address, global_patient_id: String, demographics: PatientDemographics, linked_ids: Vec<PatientIdentifier>, confidence_score: u32` | `Result<u64, Error>` | Register a master patient record linking multiple local identifiers |
| `mpi_find_patient` | `env: Env, requester: Address, global_patient_id: String` | `Result<MPIMasterPatient, Error>` | Find a master patient record by global patient ID |
| `xdr_send_document` | `env: Env, sender: Address, entry: XDSDocumentEntry, intended_recipient: String` | `Result<(), Error>` | Reliable document interchange — wraps XDS registration with delivery confirmation |
| `xdm_record_media_package` | `env: Env, actor: Address, package_id: String, patient_id: String, content_hash: BytesN<32>, media_type: String, document_ids: Vec<String>` | `Result<(), Error>` | Record a media interchange package (content hash stored on-chain) |
| `ct_record_time_sync` | `env: Env, actor: Address, node_id: String, reported_time: u64` | `Result<u64, Error>` | Record a time synchronization event on-chain |
| `bppc_register_consent` | `env: Env, author: Address, patient_id: String, policy_id: String, access_consent_list: Vec<String>, expiry_time: u64, document_ref: String` | `Result<u64, Error>` | Register a patient privacy consent document |
| `bppc_revoke_consent` | `env: Env, author: Address, consent_id: u64` | `Result<(), Error>` | Revoke a privacy consent |
| `bppc_verify_consent` | `env: Env, consent_id: u64` | `Result<BPPCConsent, Error>` | Verify consent is active and not expired |
| `dsg_sign_document` | `env: Env, signer: Address, document_id: String, signature_hash: BytesN<32>, signature_algorithm: String, certificate_ref: String, signature_purpose: String` | `Result<u64, Error>` | Record a digital signature for a document |
| `dsg_verify_signature` | `env: Env, signature_id: u64` | `Result<DSGSignature, Error>` | Verify a document signature by signature ID |
| `dsg_get_document_signatures` | `env: Env, document_id: String` | `Result<Vec<DSGSignature>, Error>` | Get all signatures for a document |
| `hpd_register_provider` | `env: Env, actor: Address, provider: HPDProvider` | `Result<u64, Error>` | Register a provider in the Healthcare Provider Directory |
| `hpd_get_provider` | `env: Env, provider_id: u64` | `Result<HPDProvider, Error>` | Query a provider by ID |
| `svs_register_value_set` | `env: Env, actor: Address, value_set: SVSValueSet` | `Result<u64, Error>` | Register a named value set |
| `svs_get_value_set_by_oid` | `env: Env, oid: String` | `Result<SVSValueSet, Error>` | Retrieve a value set by OID |
| `connectathon_record_test` | `env: Env, tester: Address, profile: IHEProfile, actor_name: String, test_name: String, passed: bool, notes: String` | `Result<u64, Error>` | Record the result of a Connectathon conformance test |
| `connectathon_get_profile_results` | `env: Env, profile: IHEProfile` | `Vec<ConnectathonTestResult>` | Get all Connectathon test results for a profile |
| `connectathon_is_compliant` | `env: Env, profile: IHEProfile` | `bool` | Check if a profile passes all recorded Connectathon tests |

### Types

#### `enum IHEProfile`

| Variant | Value | Description |
|---|---|---|
| `XDS` | — | — |
| `PIX` | — | — |
| `PDQ` | — | — |
| `ATNA` | — | — |
| `XCA` | — | — |
| `MPI` | — | — |
| `XDR` | — | — |
| `XDM` | — | — |
| `CT` | — | — |
| `BPPC` | — | — |
| `DSG` | — | — |
| `HPD` | — | — |
| `SVS` | — | — |

#### `enum HL7MessageType`

| Variant | Value | Description |
|---|---|---|
| `V2ADT` | — | — |
| `Discharge` | — | — |
| `Transfer
    V2ORM` | — | — |
| `V2ORU` | — | — |
| `V2MFN` | — | — |
| `V2QBP` | — | — |
| `V2RSP` | — | — |
| `V2ACK` | — | — |
| `V3ClinicalDocument` | — | — |
| `V3PatientQuery` | — | — |
| `V3PatientResponse` | — | — |
| `V3DeviceQuery` | — | — |

#### `enum DocumentStatus`

| Variant | Value | Description |
|---|---|---|
| `Approved` | — | — |
| `Deprecated` | — | — |
| `Submitted` | — | — |

#### `struct XDSDocumentEntry`

| Field | Type | Description |
|---|---|---|
| `document_id` | `String` | — |
| `patient_id` | `String` | — |
| `content_hash` | `BytesN<32>` | — |
| `document_class_code` | `String` | — |
| `document_type_code` | `String` | — |
| `format_code` | `String` | — |
| `healthcare_facility_type` | `String` | — |
| `practice_setting_code` | `String` | — |
| `creation_time` | `u64` | — |
| `author` | `Address` | — |
| `confidentiality_code` | `String` | — |
| `language_code` | `String` | — |
| `hl7_message_type` | `HL7MessageType` | — |
| `status` | `DocumentStatus` | — |
| `repository_unique_id` | `String` | — |
| `submission_set_id` | `String` | — |
| `mime_type` | `String` | — |

#### `struct XDSSubmissionSet`

| Field | Type | Description |
|---|---|---|
| `submission_set_id` | `String` | — |
| `patient_id` | `String` | — |
| `submission_time` | `u64` | — |
| `source_id` | `String` | — |
| `author` | `Address` | — |
| `content_type_code` | `String` | — |
| `document_ids` | `Vec<String>` | — |
| `intended_recipient` | `String` | — |

#### `struct PatientIdentifier`

| Field | Type | Description |
|---|---|---|
| `id_value` | `String` | — |
| `assigning_authority` | `String` | — |
| `identifier_type_code` | `String` | — |

#### `struct PIXCrossReference`

| Field | Type | Description |
|---|---|---|
| `reference_id` | `u64` | — |
| `local_id` | `PatientIdentifier` | — |
| `cross_referenced_ids` | `Vec<PatientIdentifier>` | — |
| `created_at` | `u64` | — |
| `updated_at` | `u64` | — |
| `is_merged` | `bool` | — |

#### `struct PatientDemographics`

| Field | Type | Description |
|---|---|---|
| `patient_id` | `String` | — |
| `given_name` | `String` | — |
| `family_name` | `String` | — |
| `date_of_birth` | `String` | — |
| `administrative_gender` | `String` | — |
| `street_address` | `String` | — |
| `city` | `String` | — |
| `state` | `String` | — |
| `postal_code` | `String` | — |
| `country_code` | `String` | — |
| `phone_home` | `String` | — |
| `phone_mobile` | `String` | — |
| `mother_maiden_name` | `String` | — |
| `marital_status` | `String` | — |
| `race` | `String` | — |
| `ethnicity` | `String` | — |
| `primary_language` | `String` | — |
| `last_updated` | `u64` | — |
| `assigning_authority` | `String` | — |

#### `struct PDQQuery`

| Field | Type | Description |
|---|---|---|
| `query_id` | `u64` | — |
| `query_parameters` | `Map<String, String>` | — |
| `requesting_system` | `String` | — |
| `query_time` | `u64` | — |
| `hl7_message_type` | `HL7MessageType` | — |
| `domain_filter` | `String` | — |

#### `enum ATNAEventType`

| Variant | Value | Description |
|---|---|---|
| `PatientRecordAccess` | — | — |
| `PatientRecordUpdate` | — | — |
| `UserAuthentication` | — | — |
| `NodeAuthentication` | — | — |
| `DocumentExport` | — | — |
| `DocumentImport` | — | — |
| `QueryRequest` | — | — |
| `QueryResponse` | — | — |
| `SecurityAlert` | — | — |
| `OrderMessage` | — | — |
| `ProcedureRecord` | — | — |

#### `enum ATNAEventOutcome`

| Variant | Value | Description |
|---|---|---|
| `Success` | 0 | — |
| `MinorFailure` | 4 | — |
| `SeriousFailure` | 8 | — |
| `MajorFailure` | 12 | — |

#### `struct ATNAParticipant`

| Field | Type | Description |
|---|---|---|
| `user_id` | `String` | — |
| `user_name` | `String` | — |
| `role_id_code` | `String` | — |
| `is_requestor` | `bool` | — |
| `network_access_point` | `String` | — |

#### `struct ATNAParticipantObject`

| Field | Type | Description |
|---|---|---|
| `object_id_type_code` | `String` | — |
| `object_id` | `String` | — |
| `object_type_code` | `u32` | — |
| `object_sensitivity` | `String` | — |
| `object_query` | `String` | — |

#### `struct ATNAAuditEvent`

| Field | Type | Description |
|---|---|---|
| `event_id` | `u64` | — |
| `event_type` | `ATNAEventType` | — |
| `event_action_code` | `String` | — |
| `event_date_time` | `u64` | — |
| `event_outcome` | `ATNAEventOutcome` | — |
| `source_id` | `String` | — |
| `source_type` | `String` | — |
| `active_participants` | `Vec<ATNAParticipant>` | — |
| `participant_objects` | `Vec<ATNAParticipantObject>` | — |
| `hl7_message_id` | `String` | — |
| `profile` | `IHEProfile` | — |

#### `struct XCAGateway`

| Field | Type | Description |
|---|---|---|
| `gateway_id` | `String` | — |
| `community_id` | `String` | — |
| `gateway_address` | `String` | — |
| `supported_profiles` | `Vec<IHEProfile>` | — |
| `registered_by` | `Address` | — |
| `registration_time` | `u64` | — |
| `is_active` | `bool` | — |

#### `struct MPIMasterPatient`

| Field | Type | Description |
|---|---|---|
| `master_id` | `u64` | — |
| `global_patient_id` | `String` | — |
| `linked_identifiers` | `Vec<PatientIdentifier>` | — |
| `demographics` | `PatientDemographics` | — |
| `created_at` | `u64` | — |
| `updated_at` | `u64` | — |
| `confidence_score` | `u32` | — |

#### `enum ConsentStatus`

| Variant | Value | Description |
|---|---|---|
| `Active` | — | — |
| `Revoked` | — | — |
| `Expired` | — | — |

#### `struct BPPCConsent`

| Field | Type | Description |
|---|---|---|
| `consent_id` | `u64` | — |
| `patient_id` | `String` | — |
| `policy_id` | `String` | — |
| `consent_status` | `ConsentStatus` | — |
| `access_consent_list` | `Vec<String>` | — |
| `date_of_consent` | `u64` | — |
| `expiry_time` | `u64` | — |
| `author` | `Address` | — |
| `document_ref` | `String` | — |

#### `struct DSGSignature`

| Field | Type | Description |
|---|---|---|
| `signature_id` | `u64` | — |
| `document_id` | `String` | — |
| `signer` | `Address` | — |
| `signature_hash` | `BytesN<32>` | — |
| `signature_algorithm` | `String` | — |
| `signing_time` | `u64` | — |
| `certificate_ref` | `String` | — |
| `signature_purpose` | `String` | — |
| `is_valid` | `bool` | — |

#### `enum ProviderType`

| Variant | Value | Description |
|---|---|---|
| `Individual` | — | — |
| `Organization` | — | — |
| `Department` | — | — |

#### `struct HPDProvider`

| Field | Type | Description |
|---|---|---|
| `provider_id` | `u64` | — |
| `provider_type` | `ProviderType` | — |
| `given_name` | `String` | — |
| `family_name` | `String` | — |
| `organization_name` | `String` | — |
| `specialty_code` | `String` | — |
| `license_number` | `String` | — |
| `npi` | `String` | — |
| `address` | `String` | — |
| `electronic_service_info` | `String` | — |
| `registered_by` | `Address` | — |
| `registration_time` | `u64` | — |
| `is_active` | `bool` | — |

#### `struct SVSConcept`

| Field | Type | Description |
|---|---|---|
| `code` | `String` | — |
| `code_system` | `String` | — |
| `code_system_name` | `String` | — |
| `display_name` | `String` | — |
| `level` | `u32` | — |
| `type_code` | `String` | — |

#### `struct SVSValueSet`

| Field | Type | Description |
|---|---|---|
| `value_set_id` | `u64` | — |
| `oid` | `String` | — |
| `name` | `String` | — |
| `version` | `String` | — |
| `status` | `String` | — |
| `description` | `String` | — |
| `concepts` | `Vec<SVSConcept>` | — |
| `effective_date` | `u64` | — |
| `source_url` | `String` | — |
| `registered_by` | `Address` | — |

#### `struct ConnectathonTestResult`

| Field | Type | Description |
|---|---|---|
| `test_id` | `u64` | — |
| `profile` | `IHEProfile` | — |
| `actor_name` | `String` | — |
| `test_name` | `String` | — |
| `passed` | `bool` | — |
| `tested_at` | `u64` | — |
| `tested_by` | `Address` | — |
| `notes` | `String` | — |

#### `enum DataKey`

| Variant | Value | Description |
|---|---|---|
| `Admin` | — | — |
| `NextDocumentId` | — | — |
| `NextPixRefId` | — | — |
| `NextPdqQueryId` | — | — |
| `NextAtnaEventId` | — | — |
| `NextMasterPatientId` | — | — |
| `NextConsentId` | — | — |
| `NextSignatureId` | — | — |
| `NextProviderId` | — | — |
| `NextValueSetId` | — | — |
| `NextTestResultId` | — | — |
| `XDSDocument(String)` | — | — |
| `XDSSubmissionSet(String), 
    PatientDocuments(String), 
    
    PIXCrossRef(u64),       
    PIXPatientRefs(String), 
    
    PatientDemographics(String), 
    PDQQuery(u64),               
    
    ATNAEvent(u64), 
    
    XCAGateway(String), 
    
    MPIMasterPatient(u64),  
    MPIGlobalIndex(String), 
    
    BPPCConsent(u64),        
    PatientConsents(String), 
    
    DSGSignature(u64),          
    DocumentSignatures(String), 
    
    HPDProvider(u64), 
    
    SVSValueSet(u64),         
    SVSValueSetByOid(String), 
    
    ConnectathonResult(u64),    
    ProfileTestIds(IHEProfile),` | — | — |

#### `enum Error`

| Variant | Value | Description |
|---|---|---|
| `NotInitialized` | 1 | — |
| `AlreadyInitialized` | 2 | — |
| `NotAuthorized` | 3 | — |
| `DocumentNotFound` | 4 | — |
| `DocumentAlreadyExists` | 5 | — |
| `DocumentDeprecated` | 6 | — |
| `PatientNotFound` | 7 | — |
| `CrossReferenceNotFound` | 8 | — |
| `DemographicsNotFound` | 9 | — |
| `AuditEventNotFound` | 10 | — |
| `GatewayNotFound` | 11 | — |
| `GatewayAlreadyExists` | 12 | — |
| `MasterPatientNotFound` | 13 | — |
| `ConsentNotFound` | 14 | — |
| `ConsentRevoked` | 15 | — |
| `ConsentExpired` | 16 | — |
| `SignatureNotFound` | 17 | — |
| `SignatureInvalid` | 18 | — |
| `ProviderNotFound` | 19 | — |
| `ValueSetNotFound` | 20 | — |
| `ValueSetOidExists` | 21 | — |
| `InvalidHL7Message` | 22 | — |
| `ConnectathonTestNotFound` | 23 | — |
| `EmptyPatientId` | 24 | — |
| `EmptyDocumentId` | 25 | — |

### Examples

#### `test_initialize`

```rust
let (_, _, _) = setup();
```

#### `test_double_initialize_fails`

```rust
let (env, admin, client) = setup();
    let _ = env;
    let result = client.try_initialize(&admin);
    assert!(result.is_err());
```

#### `test_xds_register_and_retrieve_document`

```rust
let (env, _, client) = setup();
    let author = Address::generate(&env);
    let entry = make_xds_entry(&env, &author);

    client.xds_register_document(&author, &entry);

    let retrieved = client.xds_retrieve_document(&author, &String::from_str(&env, "DOC-001"));
    assert_eq!(retrieved.document_id, entry.document_id);
    assert_eq!(retrieved.patient_id, entry.patient_id);
```

---

## iot_device_management

### Functions

| Function | Parameters | Returns | Description |
|---|---|---|---|
| `initialize` | `env: Env, admin: Address` | `Result<(), Error>` | — |
| `pause` | `env: Env, admin: Address` | `Result<(), Error>` | — |
| `unpause` | `env: Env, admin: Address` | `Result<(), Error>` | — |
| `set_role` | `env: Env, admin: Address, user: Address, role: Role` | `Result<(), Error>` | — |
| `get_role` | `env: Env, user: Address` | `Role` | — |
| `register_manufacturer` | `env: Env, admin: Address, manufacturer_id: BytesN<32>, name: String, certification_hash: BytesN<32>` | `Result<(), Error>` | — |
| `get_manufacturer` | `env: Env, manufacturer_id: BytesN<32>` | `Result<Manufacturer, Error>` | — |
| `deactivate_manufacturer` | `env: Env, admin: Address, manufacturer_id: BytesN<32>` | `Result<(), Error>` | — |
| `register_device` | `env: Env, operator: Address, device_id: BytesN<32>, manufacturer_id: BytesN<32>, device_type: DeviceType, model: String, serial_number: String, location: String, encryption_key_hash: BytesN<32>, metadata_ref: String` | `Result<(), Error>` | — |
| `get_device` | `env: Env, device_id: BytesN<32>` | `Result<Device, Error>` | — |
| `get_device_count` | `env: Env` | `u64` | — |
| `get_devices_by_operator` | `env: Env, operator: Address` | `Vec<BytesN<32>>` | — |
| `activate_device` | `env: Env, caller: Address, device_id: BytesN<32>` | `Result<(), Error>` | — |
| `suspend_device` | `env: Env, caller: Address, device_id: BytesN<32>` | `Result<(), Error>` | — |
| `decommission_device` | `env: Env, admin: Address, device_id: BytesN<32>` | `Result<(), Error>` | — |
| `publish_firmware` | `env: Env, caller: Address, manufacturer_id: BytesN<32>, version: u32, device_type: DeviceType, binary_hash: BytesN<32>, release_notes_ref: String, min_version: u32, size_bytes: u64` | `Result<(), Error>` | — |
| `approve_firmware` | `env: Env, admin: Address, manufacturer_id: BytesN<32>, version: u32` | `Result<(), Error>` | — |
| `reject_firmware` | `env: Env, admin: Address, manufacturer_id: BytesN<32>, version: u32` | `Result<(), Error>` | — |
| `get_firmware` | `env: Env, manufacturer_id: BytesN<32>, version: u32` | `Result<FirmwareVersion, Error>` | — |
| `get_latest_firmware_version` | `env: Env, manufacturer_id: BytesN<32>, device_type: DeviceType` | `Result<u32, Error>` | — |
| `update_device_firmware` | `env: Env, caller: Address, device_id: BytesN<32>, target_version: u32` | `Result<u64, Error>` | — |
| `submit_heartbeat` | `env: Env, caller: Address, device_id: BytesN<32>, health_status: HealthStatus, battery_pct: u32, signal_strength: u32, error_count: u32, metrics_ref: String` | `Result<(), Error>` | — |
| `get_device_heartbeats` | `env: Env, device_id: BytesN<32>` | `Result<Vec<Heartbeat>, Error>` | — |
| `get_device_uptime_bps` | `env: Env, device_id: BytesN<32>` | `Result<u32, Error>` | — |
| `get_active_device_count` | `env: Env` | `u64` | — |
| `set_heartbeat_interval` | `env: Env, admin: Address, interval_secs: u64` | `Result<(), Error>` | — |
| `create_comm_channel` | `env: Env, caller: Address, device_id: BytesN<32>, channel_id: BytesN<32>, encryption_key_hash: BytesN<32>, protocol: String` | `Result<(), Error>` | — |
| `get_comm_channel` | `env: Env, channel_id: BytesN<32>` | `Result<CommChannel, Error>` | — |
| `rotate_encryption_key` | `env: Env, caller: Address, channel_id: BytesN<32>, new_encryption_key_hash: BytesN<32>` | `Result<(), Error>` | — |
| `rotate_device_key` | `env: Env, caller: Address, device_id: BytesN<32>, new_encryption_key_hash: BytesN<32>` | `Result<(), Error>` | — |
| `set_key_rotation_interval` | `env: Env, admin: Address, interval_secs: u64` | `Result<(), Error>` | — |
| `get_devices_by_manufacturer` | `env: Env, manufacturer_id: BytesN<32>` | `Vec<BytesN<32>>` | — |
| `get_device_firmware_history` | `env: Env, device_id: BytesN<32>` | `Result<Vec<FirmwareUpdateRecord>, Error>` | — |
| `get_manufacturer_count` | `env: Env` | `u32` | — |
| `get_firmware_update_record` | `env: Env, update_id: u64` | `Result<FirmwareUpdateRecord, Error>` | — |

### Types

#### `enum DeviceStatus`

| Variant | Value | Description |
|---|---|---|
| `Provisioning` | 0 | — |
| `Active` | 1 | — |
| `Suspended` | 2 | — |
| `Maintenance` | 3 | — |
| `Decommissioned` | 4 | — |

#### `enum DeviceType`

| Variant | Value | Description |
|---|---|---|
| `VitalSignsMonitor` | 0 | — |
| `BloodPressureMonitor` | 1 | — |
| `GlucoseMonitor` | 2 | — |
| `PulseOximeter` | 3 | — |
| `ECGMonitor` | 4 | — |
| `TemperatureSensor` | 5 | — |
| `InfusionPump` | 6 | — |
| `Ventilator` | 7 | — |
| `WearableSensor` | 8 | — |
| `ImagingDevice` | 9 | — |
| `LabAnalyzer` | 10 | — |
| `Other` | 99 | — |

#### `enum FirmwareStatus`

| Variant | Value | Description |
|---|---|---|
| `Pending` | 0 | — |
| `Approved` | 1 | — |
| `Rejected` | 2 | — |
| `Deprecated` | 3 | — |

#### `enum HealthStatus`

| Variant | Value | Description |
|---|---|---|
| `Healthy` | 0 | — |
| `Degraded` | 1 | — |
| `Critical` | 2 | — |
| `Offline` | 3 | — |

#### `enum Role`

| Variant | Value | Description |
|---|---|---|
| `Admin` | 0 | — |
| `Manufacturer` | 1 | — |
| `Operator` | 2 | — |
| `Viewer` | 3 | — |

#### `struct Manufacturer`

| Field | Type | Description |
|---|---|---|
| `manufacturer_id` | `BytesN<32>` | — |
| `address` | `Address` | — |
| `name` | `String` | — |
| `certification_hash` | `BytesN<32>` | — |
| `is_active` | `bool` | — |
| `registered_at` | `u64` | — |
| `device_count` | `u32` | — |

#### `struct Device`

| Field | Type | Description |
|---|---|---|
| `device_id` | `BytesN<32>` | — |
| `manufacturer_id` | `BytesN<32>` | — |
| `device_type` | `DeviceType` | — |
| `model` | `String` | — |
| `serial_number` | `String` | — |
| `firmware_version` | `u32` | — |
| `status` | `DeviceStatus` | — |
| `operator` | `Address` | — |
| `location` | `String` | — |
| `registered_at` | `u64` | — |
| `last_heartbeat` | `u64` | — |
| `health_status` | `HealthStatus` | — |
| `uptime_start` | `u64` | — |
| `total_uptime_secs` | `u64` | — |
| `total_downtime_secs` | `u64` | — |
| `encryption_key_hash` | `BytesN<32>` | — |
| `metadata_ref` | `String` | — |

#### `struct FirmwareVersion`

| Field | Type | Description |
|---|---|---|
| `version` | `u32` | — |
| `manufacturer_id` | `BytesN<32>` | — |
| `device_type` | `DeviceType` | — |
| `binary_hash` | `BytesN<32>` | — |
| `release_notes_ref` | `String` | — |
| `status` | `FirmwareStatus` | — |
| `min_version` | `u32` | — |
| `published_at` | `u64` | — |
| `approved_by` | `Address` | — |
| `size_bytes` | `u64` | — |

#### `struct FirmwareUpdateRecord`

| Field | Type | Description |
|---|---|---|
| `update_id` | `u64` | — |
| `device_id` | `BytesN<32>` | — |
| `from_version` | `u32` | — |
| `to_version` | `u32` | — |
| `initiated_by` | `Address` | — |
| `initiated_at` | `u64` | — |
| `completed_at` | `u64` | — |
| `success` | `bool` | — |
| `error_ref` | `String` | — |

#### `struct Heartbeat`

| Field | Type | Description |
|---|---|---|
| `device_id` | `BytesN<32>` | — |
| `timestamp` | `u64` | — |
| `health_status` | `HealthStatus` | — |
| `battery_pct` | `u32` | — |
| `signal_strength` | `u32` | — |
| `error_count` | `u32` | — |
| `metrics_ref` | `String` | — |

#### `struct CommChannel`

| Field | Type | Description |
|---|---|---|
| `channel_id` | `BytesN<32>` | — |
| `device_id` | `BytesN<32>` | — |
| `encryption_key_hash` | `BytesN<32>` | — |
| `protocol` | `String` | — |
| `created_at` | `u64` | — |
| `last_rotated` | `u64` | — |
| `rotation_count` | `u32` | — |

#### `enum DataKey`

| Variant | Value | Description |
|---|---|---|
| `Initialized` | — | — |
| `Admin` | — | — |
| `Paused` | — | — |
| `UserRole(Address)` | — | — |
| `Manufacturer(BytesN<32>)` | — | — |
| `ManufacturerByAddr(Address)` | — | — |
| `ManufacturerCount` | — | — |
| `Device(BytesN<32>)` | — | — |
| `DevicesByOperator(Address)` | — | — |
| `DevicesByManufacturer(BytesN<32>)` | — | — |
| `DevicesByType(u32)` | — | — |
| `DeviceCount` | — | — |
| `ActiveDeviceCount` | — | — |
| `Firmware(BytesN<32>, u32)` | — | — |
| `LatestFirmware(BytesN<32>, u32)` | — | — |
| `FirmwareUpdateRecord(u64),
    FirmwareUpdateCount,
    DeviceFirmwareUpdates(BytesN<32>), 

    
    DeviceHeartbeats(BytesN<32>), 
    HeartbeatMinInterval,         

    
    CommChannel(BytesN<32>),   
    DeviceChannel(BytesN<32>), 
    KeyRotationMinInterval,` | — | — |

### Error Codes

| Variant | Code | Description |
|---|---|---|
| `Unauthorized` | 100 | — |
| `NotAdmin` | 102 | — |
| `NotDeviceOperator` | 115 | — |
| `NotManufacturer` | 116 | — |
| `InputTooLong` | 201 | — |
| `InputTooShort` | 202 | — |
| `InvalidDeviceType` | 240 | — |
| `InvalidFirmwareHash` | 250 | — |
| `InvalidMetricValue` | 260 | — |
| `InvalidTimestamp` | 270 | — |
| `NotInitialized` | 300 | — |
| `AlreadyInitialized` | 301 | — |
| `ContractPaused` | 302 | — |
| `NotPaused` | 303 | — |
| `DeviceNotFound` | 405 | — |
| `DeviceAlreadyRegistered` | 420 | — |
| `ManufacturerNotRegistered` | 425 | — |
| `ManufacturerAlreadyRegistered` | 426 | — |
| `FirmwareVersionNotFound` | 430 | — |
| `FirmwareAlreadyExists` | 431 | — |
| `ChannelNotFound` | 440 | — |
| `InvalidEncryptionKey` | 602 | — |
| `KeyRotationTooFrequent` | 603 | — |
| `DeviceDecommissioned` | 820 | — |
| `FirmwareNotApproved` | 821 | — |
| `HeartbeatTooFrequent` | 822 | — |
| `DeviceNotActive` | 823 | — |
| `DeviceSuspended` | 824 | — |
| `DowngradeNotAllowed` | 825 | — |
| `DeviceOffline` | 826 | — |

### Examples

#### `test_initialize`

```rust
let env = Env::default();
    let (client, admin) = setup(&env);
    client.initialize(&admin);
    // Calling initialize again should fail
    let result = client.try_initialize(&admin);
    assert_eq!(result, Err(Ok(Error::AlreadyInitialized)));
```

#### `test_pause_unpause`

```rust
let env = Env::default();
    let (client, admin) = setup(&env);
    client.initialize(&admin);
    client.pause(&admin);
    // set_role should fail when paused
    let user = Address::generate(&env);
    let result = client.try_set_role(&admin, &user, &Role::Operator);
    assert_eq!(result, Err(Ok(Error::ContractPaused)));
    client.unpause(&admin);
```

#### `test_pause_not_admin`

```rust
let env = Env::default();
    let (client, admin) = setup(&env);
    client.initialize(&admin);
    let non_admin = Address::generate(&env);
    let result = client.try_pause(&non_admin);
    assert_eq!(result, Err(Ok(Error::NotAdmin)));
```

---

## load_testing

### Functions

| Function | Parameters | Returns | Description |
|---|---|---|---|
| `run` | `env: Env, config: LoadTestConfig` | `LoadTestResult` | Execute a load-test run and persist the result.  Each "operation" is a lightweight storage read/write that exercises the contract's execution path.  Latency is measured in ledger sequence units. |
| `last_result` | `env: Env` | `Option<LoadTestResult>` | Return the result of the most recent run. |
| `run_count` | `env: Env` | `u32` | Return the total number of runs executed. |

### Types

#### `struct LoadTestConfig`

| Field | Type | Description |
|---|---|---|
| `num_requests` | `u32` | — |
| `concurrency` | `u32` | — |
| `max_avg_latency` | `u64` | — |
| `min_success_rate` | `u32` | — |

#### `struct LoadTestResult`

| Field | Type | Description |
|---|---|---|
| `total_requests` | `u32` | — |
| `successful` | `u32` | — |
| `failed` | `u32` | — |
| `success_rate` | `u32` | — |
| `min_latency` | `u64` | — |
| `max_latency` | `u64` | — |
| `avg_latency` | `u64` | — |
| `p95_latency` | `u64` | — |
| `p99_latency` | `u64` | — |
| `passed` | `bool` | — |

#### `enum DataKey`

| Variant | Value | Description |
|---|---|---|
| `LastResult` | — | — |
| `RunCount` | — | — |

---

## medical_consent_nft

### Functions

| Function | Parameters | Returns | Description |
|---|---|---|---|
| `initialize` | `env: Env, admin: Address` | `Result<(), ContractError>` | Initialize the contract with an admin |
| `add_issuer` | `env: Env, issuer: Address` | `()` | Add an authorized issuer (clinic/healthcare provider) |
| `remove_issuer` | `env: Env, issuer: Address` | `()` | Remove an authorized issuer |
| `is_issuer` | `env: Env, address: Address` | `bool` | Check if address is an authorized issuer |
| `mint_consent` | `env: Env, issuer: Address, // FIXED: Passed by caller (must be their own Address::AccountId` | `()` | Mint a new consent token - FIXED: Add issuer: Address param, require_auth on it, use for check & metadata (no env.invoker()) |
| `update_consent` | `env: Env, token_id: u64, new_metadata_uri: String` | `Result<(), ContractError>` | Update consent metadata (creates new version) |
| `revoke_consent` | `env: Env, token_id: u64` | `Result<(), ContractError>` | Revoke consent (marks as revoked, prevents transfers) - Patient authorizes via require_auth on their address from metadata |
| `transfer` | `env: Env, from: Address, to: Address, token_id: u64` | `Result<(), ContractError>` | Transfer consent token (blocked if revoked) |
| `owner_of` | `env: Env, token_id: u64` | `Address` | Get token owner |
| `get_metadata` | `env: Env, token_id: u64` | `ConsentMetadata` | Get consent metadata |
| `is_revoked` | `env: Env, token_id: u64` | `bool` | Check if consent is revoked |
| `get_history` | `env: Env, token_id: u64` | `Vec<ConsentHistoryEntry>` | Get consent history (audit trail) |
| `tokens_of_owner` | `env: Env, owner: Address` | `Vec<u64>` | Get all tokens owned by an address |
| `has_consent` | `env: Env, patient: Address, doctor: Address, consent_type: String` | `bool` | Check if doctor has valid consent for patient and type (for cross-contract access control) |
| `is_valid` | `env: Env, token_id: u64` | `bool` | Check if consent is valid (not revoked and not expired) |
| `set_granular_permissions` | `env: Env, caller: Address, token_id: u64, permissions: GranularPermissions` | `Result<(), ContractError>` | Set granular permissions for a consent token |
| `get_granular_permissions` | `env: Env, token_id: u64` | `Result<GranularPermissions, ContractError>` | Get granular permissions for a consent token |
| `has_permission` | `env: Env, token_id: u64, requester: Address, data_type: DataType, required_level: PermissionLevel` | `bool` | Check if requester has permission for specific data type |
| `set_access_controls` | `env: Env, token_id: u64, access_control: AccessControl` | `Result<(), ContractError>` | Set access controls for a consent token |
| `check_access_allowed` | `env: Env, token_id: u64, _requester: Address` | `Result<bool, ContractError>` | Check if access is allowed based on access controls |
| `record_access` | `env: Env, token_id: u64, _requester: Address` | `Result<(), ContractError>` | Record access attempt |
| `delegate_consent` | `env: Env, token_id: u64, delegate: Address, permissions: GranularPermissions, expiry_timestamp: u64` | `Result<(), ContractError>` | Delegate consent to another address |
| `revoke_delegation` | `env: Env, token_id: u64, delegate: Address` | `Result<(), ContractError>` | Revoke delegation |
| `get_delegations` | `env: Env, token_id: u64` | `Vec<Delegation>` | Get active delegations for a token |
| `set_inheritance` | `env: Env, child_token_id: u64, parent_token_id: u64, inherited_permissions: GranularPermissions` | `Result<(), ContractError>` | Set consent inheritance (child consent inherits from parent) |
| `add_emergency_authority` | `env: Env, authority: Address` | `Result<(), ContractError>` | Add emergency authority |
| `emergency_override` | `env: Env, caller: Address, token_id: u64, reason: String, duration: u64` | `Result<u64, ContractError>` | Emergency override access |
| `set_marketplace_enabled` | `env: Env, enabled: bool` | `Result<(), ContractError>` | Enable/disable marketplace |
| `list_on_marketplace` | `env: Env, token_id: u64, price: i128, data_types: Vec<DataType>, research_purpose: String, duration: u64` | `Result<(), ContractError>` | List consent on marketplace for research |
| `get_marketplace_listing` | `env: Env, token_id: u64` | `Result<MarketplaceListing, ContractError>` | Get marketplace listing |
| `purchase_marketplace_listing` | `env: Env, token_id: u64, buyer: Address` | `Result<(), ContractError>` | Purchase marketplace listing (simplified - would need payment integration) |
| `update_consent_dynamic` | `env: Env, caller: Address, token_id: u64, new_metadata_uri: String, change_summary: String` | `Result<(), ContractError>` | Enhanced dynamic consent update with version history |
| `get_version_history` | `env: Env, token_id: u64` | `Vec<VersionHistoryEntry>` | Get version history |
| `enable_dynamic_updates` | `env: Env, token_id: u64` | `Result<(), ContractError>` | Enable dynamic updates for a consent |
| `get_analytics` | `env: Env` | `AnalyticsData` | Get analytics data |
| `generate_consent_report` | `env: Env, patient: Address` | `Vec<u64>` | Generate consent report for a patient |

### Types

#### `enum DataKey`

| Variant | Value | Description |
|---|---|---|
| `Admin` | — | — |
| `Issuers` | — | — |
| `TokenCounter` | — | — |
| `TokenOwner(u64)` | — | — |
| `TokenMetadata(u64)` | — | — |
| `TokenRevoked(u64)` | — | — |
| `OwnerTokens(Address)` | — | — |
| `ConsentHistory(u64)` | — | — |
| `PatientConsents(Address)` | — | — |
| `GranularPermissions(u64)` | — | — |
| `AccessControls(u64)` | — | — |
| `ConsentDelegations(u64)` | — | — |
| `ConsentInheritance(u64)` | — | — |
| `EmergencyOverrides(u64)` | — | — |
| `MarketplaceListings(u64)` | — | — |
| `VersionHistory(u64)` | — | — |
| `AnalyticsData` | — | — |
| `EmergencyAuthorities` | — | — |
| `MarketplaceEnabled` | — | — |

#### `enum ContractError`

| Variant | Value | Description |
|---|---|---|
| `NotAuthorized` | 1 | — |
| `TokenNotFound` | 2 | — |
| `ConsentRevoked` | 3 | — |
| `AlreadyInitialized` | 4 | — |
| `NotTokenOwner` | 5 | — |
| `InvalidPermission` | 6 | — |
| `AccessDenied` | 7 | — |
| `InvalidDelegation` | 8 | — |
| `EmergencyOverrideFailed` | 9 | — |
| `MarketplaceNotEnabled` | 10 | — |
| `InvalidCondition` | 11 | — |
| `InheritanceCycle` | 12 | — |

#### `enum DataType`

| Variant | Value | Description |
|---|---|---|
| `Demographics` | — | — |
| `MedicalHistory` | — | — |
| `LabResults` | — | — |
| `Imaging` | — | — |
| `Medications` | — | — |
| `Procedures` | — | — |
| `Allergies` | — | — |
| `Research` | — | — |
| `Financial` | — | — |

#### `enum PermissionLevel`

| Variant | Value | Description |
|---|---|---|
| `None` | — | — |
| `Read` | — | — |
| `Write` | — | — |
| `Full` | — | — |

#### `struct GranularPermissions`

| Field | Type | Description |
|---|---|---|
| `permissions` | `Map<DataType, PermissionLevel>` | — |

#### `enum AccessCondition`

| Variant | Value | Description |
|---|---|---|
| `TimeWindow(u64, u64)` | — | — |
| `DayOfWeek(Vec<u32>)` | — | — |
| `TimeOfDay(u32, u32)` | — | — |
| `LocationBased(Vec<String>)` | — | — |
| `PurposeBased(Vec<String>)` | — | — |
| `EmergencyOnly` | — | — |

#### `struct AccessControl`

| Field | Type | Description |
|---|---|---|
| `conditions` | `Vec<AccessCondition>` | — |
| `max_access_count` | `u32` | — |
| `current_access_count` | `u32` | — |
| `last_access_timestamp` | `u64` | — |

#### `struct Delegation`

| Field | Type | Description |
|---|---|---|
| `delegate` | `Address` | — |
| `permissions` | `GranularPermissions` | — |
| `expiry_timestamp` | `u64` | — |
| `created_timestamp` | `u64` | — |

#### `struct Inheritance`

| Field | Type | Description |
|---|---|---|
| `parent_token_id` | `u64` | — |
| `inherited_permissions` | `GranularPermissions` | — |

#### `struct EmergencyOverride`

| Field | Type | Description |
|---|---|---|
| `override_id` | `u64` | — |
| `authorized_by` | `Address` | — |
| `reason` | `String` | — |
| `timestamp` | `u64` | — |
| `duration` | `u64` | — |
| `used` | `bool` | — |

#### `struct MarketplaceListing`

| Field | Type | Description |
|---|---|---|
| `token_id` | `u64` | — |
| `price` | `i128` | — |
| `data_types` | `Vec<DataType>` | — |
| `research_purpose` | `String` | — |
| `duration` | `u64` | — |
| `listed_by` | `Address` | — |
| `listed_timestamp` | `u64` | — |
| `active` | `bool` | — |

#### `struct VersionHistoryEntry`

| Field | Type | Description |
|---|---|---|
| `version` | `u32` | — |
| `metadata_uri` | `String` | — |
| `updated_by` | `Address` | — |
| `timestamp` | `u64` | — |
| `change_summary` | `String` | — |

#### `struct ConsentMetadata`

| Field | Type | Description |
|---|---|---|
| `metadata_uri` | `String` | — |
| `consent_type` | `String` | — |
| `issued_timestamp` | `u64` | — |
| `expiry_timestamp` | `u64` | — |
| `issuer` | `Address` | — |
| `patient` | `Address` | — |
| `version` | `u32` | — |
| `dynamic_updates_enabled` | `bool` | — |

#### `struct ConsentHistoryEntry`

| Field | Type | Description |
|---|---|---|
| `action` | `String` | — |
| `timestamp` | `u64` | — |
| `actor` | `Address` | — |
| `metadata_uri` | `String` | — |
| `details` | `String` | — |

#### `struct AnalyticsData`

| Field | Type | Description |
|---|---|---|
| `total_consents` | `u64` | — |
| `active_consents` | `u64` | — |
| `revoked_consents` | `u64` | — |
| `total_delegations` | `u64` | — |
| `total_emergency_overrides` | `u64` | — |
| `marketplace_listings` | `u64` | — |
| `total_access_count` | `u64` | — |

### Examples

#### `test_initialize_and_add_issuer`

```rust
let env = Env::default();
        let contract_id = env.register_contract(None, PatientConsentToken);
        let client = PatientConsentTokenClient::new(&env, &contract_id);

        let admin = Address::generate(&env);
        let issuer = Address::generate(&env);

        client.initialize(&admin);
        client.add_issuer(&issuer);
```

---

## medical_imaging

### Functions

| Function | Parameters | Returns | Description |
|---|---|---|---|
| `initialize` | `env: Env, admin: Address, safety_threshold_mgy: u32` | `Result<bool, Error>` | — |
| `set_paused` | `env: Env, caller: Address, paused: bool` | `Result<bool, Error>` | — |
| `assign_role` | `env: Env, caller: Address, user: Address, role_mask: u32` | `Result<bool, Error>` | — |
| `set_safety_threshold` | `env: Env, caller: Address, safety_threshold_mgy: u32` | `Result<bool, Error>` | — |
| `upload_image` | `env: Env, caller: Address, patient: Address, modality: ImagingModality, encrypted_ref: String, compression: CompressionAlgorithm, original_size_bytes: u64, compressed_size_bytes: u64, content_hash: BytesN<32>, encrypted_key_commitment: BytesN<32>, dicom: DicomMetadata` | `Result<u64, Error>` | — |
| `extract_and_index_metadata` | `env: Env, caller: Address, image_id: u64, token_hashes: Vec<BytesN<32>>, finding_hashes: Vec<BytesN<32>>` | `Result<bool, Error>` | — |
| `run_edge_detection` | `env: Env, caller: Address, image_id: u64, bins: Vec<u32>, gradient_threshold: u32, output_ref: String, output_hash: BytesN<32>, algorithm_version: u32` | `Result<ProcessingResult, Error>` | — |
| `run_segmentation` | `env: Env, caller: Address, image_id: u64, bins: Vec<u32>, lower_bound: u32, upper_bound: u32, output_ref: String, output_hash: BytesN<32>, algorithm_version: u32` | `Result<ProcessingResult, Error>` | — |
| `register_ai_model` | `env: Env, caller: Address, model_id: BytesN<32>, model_name_hash: BytesN<32>, version: u32, modality: ImagingModality` | `Result<bool, Error>` | — |
| `submit_diagnostic_assistance` | `env: Env, caller: Address, image_id: u64, model_id: BytesN<32>, condition_hash: BytesN<32>, confidence_bps: u32, explanation_ref: String, recommended_action_hash: BytesN<32>` | `Result<u64, Error>` | — |
| `grant_image_access` | `env: Env, caller: Address, image_id: u64, grantee: Address, scope: ShareScope, expires_at: u64, zk_access_commitment: BytesN<32>, watermark_hash: BytesN<32>` | `Result<bool, Error>` | — |
| `revoke_image_access` | `env: Env, caller: Address, image_id: u64, grantee: Address` | `Result<bool, Error>` | — |
| `verify_share_access` | `env: Env, image_id: u64, viewer: Address` | `Result<bool, Error>` | — |
| `verify_image_integrity` | `env: Env, caller: Address, image_id: u64, observed_hash: BytesN<32>` | `Result<bool, Error>` | — |
| `add_annotation` | `env: Env, caller: Address, image_id: u64, visibility: AnnotationVisibility, encrypted_note_ref: String, note_hash: BytesN<32>, region_hash: BytesN<32>, collaborators: Vec<Address>` | `Result<u64, Error>` | — |
| `add_annotation_reply` | `env: Env, caller: Address, annotation_id: u64, reply_hash: BytesN<32>` | `Result<bool, Error>` | — |
| `resolve_annotation` | `env: Env, caller: Address, annotation_id: u64` | `Result<bool, Error>` | — |
| `link_image_to_record` | `env: Env, caller: Address, image_id: u64, record_contract: Address, medical_record_id: u64` | `Result<bool, Error>` | — |
| `record_radiation_dose` | `env: Env, caller: Address, patient: Address, image_id: u64, modality: ImagingModality, dose_mgy: u32` | `Result<u64, Error>` | — |
| `get_image` | `env: Env, image_id: u64` | `Option<MedicalImage>` | — |
| `get_dicom` | `env: Env, image_id: u64` | `Option<DicomMetadata>` | — |
| `get_image_by_sop` | `env: Env, sop_uid_hash: BytesN<32>` | `Option<u64>` | — |
| `list_images_by_patient` | `env: Env, patient: Address` | `Vec<u64>` | — |
| `list_images_by_modality_hash` | `env: Env, modality_code_hash: BytesN<32>` | `Vec<u64>` | — |
| `list_images_by_body_part_hash` | `env: Env, body_part_hash: BytesN<32>` | `Vec<u64>` | — |
| `get_compression_ratio_bps` | `env: Env, image_id: u64` | `Result<u32, Error>` | — |
| `get_processing_result` | `env: Env, image_id: u64, kind: ProcessingKind` | `Option<ProcessingResult>` | — |
| `get_metadata_index` | `env: Env, image_id: u64` | `Option<ImageMetadataIndex>` | — |
| `get_model` | `env: Env, model_id: BytesN<32>` | `Option<AiDiagnosticModel>` | — |
| `get_diagnostic` | `env: Env, diagnosis_id: u64` | `Option<DiagnosticAssistance>` | — |
| `get_share_grant` | `env: Env, image_id: u64, grantee: Address` | `Option<ImageShareGrant>` | — |
| `get_annotation` | `env: Env, annotation_id: u64` | `Option<ImageAnnotation>` | — |
| `list_annotations_for_image` | `env: Env, image_id: u64` | `Vec<ImageAnnotation>` | — |
| `get_image_record_link` | `env: Env, image_id: u64` | `Option<ImageRecordLink>` | — |
| `get_dose_entry` | `env: Env, dose_id: u64` | `Option<RadiationDoseEntry>` | — |
| `get_dose_summary` | `env: Env, patient: Address` | `Option<DoseSummary>` | — |
| `create_study` | `env: Env, caller: Address, patient: Address, modality: ImagingModality, image_ids: Vec<u64>, required_readers: u32` | `Result<u64, Error>` | — |
| `assign_reader` | `env: Env, caller: Address, study_id: u64, reader: Address` | `Result<bool, Error>` | — |
| `assign_arbitrator` | `env: Env, caller: Address, study_id: u64, arbitrator: Address` | `Result<bool, Error>` | — |
| `link_ai_results` | `env: Env, caller: Address, study_id: u64, result_ids: Vec<u64>` | `Result<bool, Error>` | — |
| `get_study` | `env: Env, study_id: u64` | `Option<ImagingStudy>` | — |
| `get_studies_by_reader` | `env: Env, reader: Address` | `Vec<u64>` | — |
| `get_studies_by_status` | `env: Env, status: StudyStatus` | `Vec<u64>` | — |
| `get_studies_by_patient` | `env: Env, patient: Address` | `Vec<u64>` | — |
| `submit_reader_report` | `env: Env, reader: Address, study_id: u64, diagnosis_hash: BytesN<32>, findings_hash: BytesN<32>, findings_ref: String, agrees_with_ai: bool, ai_accuracy_feedback_bps: u32` | `Result<u64, Error>` | — |
| `get_reader_reports` | `env: Env, caller: Address, study_id: u64` | `Vec<ReaderReport>` | — |
| `get_my_report` | `env: Env, reader: Address, study_id: u64` | `Result<ReaderReport, Error>` | — |
| `finalize_study` | `env: Env, caller: Address, study_id: u64, _final_report_ref: String` | `Result<bool, Error>` | — |
| `amend_study` | `env: Env, caller: Address, study_id: u64, _amendment_ref: String, _reason_hash: BytesN<32>` | `Result<bool, Error>` | — |

### Types

#### `enum ImagingModality`

| Variant | Value | Description |
|---|---|---|
| `XRay` | — | — |
| `MRI` | — | — |
| `CT` | — | — |
| `Ultrasound` | — | — |
| `PET` | — | — |
| `Mammography` | — | — |
| `Custom(u32)` | — | — |

#### `enum CompressionAlgorithm`

| Variant | Value | Description |
|---|---|---|
| `None` | — | — |
| `LosslessJpeg` | — | — |
| `Jpeg2000Lossless` | — | — |
| `Rle` | — | — |
| `Deflate` | — | — |
| `Custom(u32)` | — | — |

#### `enum ProcessingKind`

| Variant | Value | Description |
|---|---|---|
| `EdgeDetection` | — | — |
| `Segmentation` | — | — |

#### `enum ShareScope`

| Variant | Value | Description |
|---|---|---|
| `ViewOnly` | — | — |
| `Diagnostics` | — | — |
| `Research` | — | — |

#### `enum AnnotationVisibility`

| Variant | Value | Description |
|---|---|---|
| `Private` | — | — |
| `CareTeam` | — | — |
| `MultiInstitution` | — | — |

#### `struct DicomMetadata`

| Field | Type | Description |
|---|---|---|
| `study_uid_hash` | `BytesN<32>` | — |
| `series_uid_hash` | `BytesN<32>` | — |
| `sop_uid_hash` | `BytesN<32>` | — |
| `modality_code_hash` | `BytesN<32>` | — |
| `body_part_hash` | `BytesN<32>` | — |
| `acquisition_timestamp` | `u64` | — |
| `rows` | `u32` | — |
| `cols` | `u32` | — |
| `bits_allocated` | `u32` | — |
| `pixel_spacing_microns` | `u32` | — |

#### `struct MedicalImage`

| Field | Type | Description |
|---|---|---|
| `image_id` | `u64` | — |
| `patient` | `Address` | — |
| `uploaded_by` | `Address` | — |
| `modality` | `ImagingModality` | — |
| `encrypted_ref` | `String` | — |
| `compression` | `CompressionAlgorithm` | — |
| `original_size_bytes` | `u64` | — |
| `compressed_size_bytes` | `u64` | — |
| `content_hash` | `BytesN<32>` | — |
| `encrypted_key_commitment` | `BytesN<32>` | — |
| `dicom_sop_uid_hash` | `BytesN<32>` | — |
| `uploaded_at` | `u64` | — |
| `integrity_verified_at` | `u64` | — |
| `tamper_detected` | `bool` | — |

#### `struct ImageMetadataIndex`

| Field | Type | Description |
|---|---|---|
| `image_id` | `u64` | — |
| `extracted_by` | `Address` | — |
| `extracted_at` | `u64` | — |
| `token_hashes` | `Vec<BytesN<32>>` | — |
| `finding_hashes` | `Vec<BytesN<32>>` | — |

#### `struct ProcessingResult`

| Field | Type | Description |
|---|---|---|
| `image_id` | `u64` | — |
| `kind` | `ProcessingKind` | — |
| `processor` | `Address` | — |
| `algorithm_version` | `u32` | — |
| `output_ref` | `String` | — |
| `output_hash` | `BytesN<32>` | — |
| `quality_score_bps` | `u32` | — |
| `created_at` | `u64` | — |

#### `struct AiDiagnosticModel`

| Field | Type | Description |
|---|---|---|
| `model_id` | `BytesN<32>` | — |
| `owner` | `Address` | — |
| `model_name_hash` | `BytesN<32>` | — |
| `version` | `u32` | — |
| `modality` | `ImagingModality` | — |
| `is_active` | `bool` | — |
| `created_at` | `u64` | — |

#### `struct DiagnosticAssistance`

| Field | Type | Description |
|---|---|---|
| `diagnosis_id` | `u64` | — |
| `image_id` | `u64` | — |
| `model_id` | `BytesN<32>` | — |
| `clinician` | `Address` | — |
| `condition_hash` | `BytesN<32>` | — |
| `confidence_bps` | `u32` | — |
| `explanation_ref` | `String` | — |
| `recommended_action_hash` | `BytesN<32>` | — |
| `created_at` | `u64` | — |

#### `struct ImageShareGrant`

| Field | Type | Description |
|---|---|---|
| `image_id` | `u64` | — |
| `patient` | `Address` | — |
| `grantee` | `Address` | — |
| `granted_by` | `Address` | — |
| `scope` | `ShareScope` | — |
| `expires_at` | `u64` | — |
| `zk_access_commitment` | `BytesN<32>` | — |
| `watermark_hash` | `BytesN<32>` | — |
| `revoked` | `bool` | — |

#### `struct ImageAnnotation`

| Field | Type | Description |
|---|---|---|
| `annotation_id` | `u64` | — |
| `image_id` | `u64` | — |
| `author` | `Address` | — |
| `visibility` | `AnnotationVisibility` | — |
| `encrypted_note_ref` | `String` | — |
| `note_hash` | `BytesN<32>` | — |
| `region_hash` | `BytesN<32>` | — |
| `collaborators` | `Vec<Address>` | — |
| `created_at` | `u64` | — |
| `resolved` | `bool` | — |
| `resolved_by` | `Option<Address>` | — |
| `replies` | `Vec<BytesN<32>>` | — |

#### `struct ImageRecordLink`

| Field | Type | Description |
|---|---|---|
| `image_id` | `u64` | — |
| `record_contract` | `Address` | — |
| `medical_record_id` | `u64` | — |
| `linked_by` | `Address` | — |
| `linked_at` | `u64` | — |

#### `struct RadiationDoseEntry`

| Field | Type | Description |
|---|---|---|
| `dose_id` | `u64` | — |
| `patient` | `Address` | — |
| `image_id` | `u64` | — |
| `modality` | `ImagingModality` | — |
| `dose_mgy` | `u32` | — |
| `warning_threshold_mgy` | `u32` | — |
| `accumulated_mgy` | `u64` | — |
| `recorded_at` | `u64` | — |
| `threshold_exceeded` | `bool` | — |

#### `struct DoseSummary`

| Field | Type | Description |
|---|---|---|
| `patient` | `Address` | — |
| `total_mgy` | `u64` | — |
| `event_count` | `u32` | — |
| `last_recorded_at` | `u64` | — |
| `safety_alerts` | `u32` | — |

#### `enum StudyStatus`

| Variant | Value | Description |
|---|---|---|
| `Pending` | — | — |
| `Assigned` | — | — |
| `InReview` | — | — |
| `PreliminaryReport` | — | — |
| `DiscrepancyReview` | — | — |
| `FinalReport` | — | — |
| `Amended` | — | — |

#### `struct ImagingStudy`

| Field | Type | Description |
|---|---|---|
| `study_id` | `u64` | — |
| `patient` | `Address` | — |
| `created_by` | `Address` | — |
| `modality` | `ImagingModality` | — |
| `image_ids` | `Vec<u64>` | — |
| `ai_result_ids` | `Vec<u64>` | — |
| `required_readers` | `u32` | — |
| `status` | `StudyStatus` | — |
| `created_at` | `u64` | — |
| `finalized_at` | `u64` | — |

#### `struct ReaderReport`

| Field | Type | Description |
|---|---|---|
| `report_id` | `u64` | — |
| `study_id` | `u64` | — |
| `reader` | `Address` | — |
| `diagnosis_hash` | `BytesN<32>` | — |
| `findings_hash` | `BytesN<32>` | — |
| `findings_ref` | `String` | — |
| `agrees_with_ai` | `bool` | — |
| `ai_accuracy_feedback_bps` | `u32` | — |
| `submitted_at` | `u64` | — |

#### `enum DataKey`

| Variant | Value | Description |
|---|---|---|
| `Roles(Address)` | — | — |
| `Image(u64)` | — | — |
| `ImageIds` | — | — |
| `Dicom(u64)` | — | — |
| `ImageByPatient(Address)` | — | — |
| `ImageByModality(BytesN<32>)` | — | — |
| `ImageByBodyPart(BytesN<32>)` | — | — |
| `SopLookup(BytesN<32>)` | — | — |
| `MetadataIndex(u64)` | — | — |
| `Processing(u64, ProcessingKind)` | — | — |
| `Model(BytesN<32>)` | — | — |
| `Diagnosis(u64)` | — | — |
| `Share(u64, Address)` | — | — |
| `Annotation(u64)` | — | — |
| `ImageAnnotations(u64)` | — | — |
| `Link(u64)` | — | — |
| `DoseEntry(u64)` | — | — |
| `DoseSummary(Address)` | — | — |
| `Study(u64)` | — | — |
| `ReaderReportEntry(u64)` | — | — |
| `StudyReports(u64)` | — | — |
| `StudyReaders(u64)` | — | — |
| `ReaderStudies(Address)` | — | — |
| `StatusStudies(u32)` | — | — |
| `PatientStudies(Address)` | — | — |
| `StudyArbitrator(u64)` | — | — |

#### `enum Error`

| Variant | Value | Description |
|---|---|---|
| `AlreadyInitialized` | 1 | — |
| `NotInitialized` | 2 | — |
| `NotAuthorized` | 3 | — |
| `ContractPaused` | 4 | — |
| `InvalidInput` | 5 | — |
| `ImageNotFound` | 6 | — |
| `ModelNotFound` | 7 | — |
| `ShareNotFound` | 8 | — |
| `ShareExpired` | 9 | — |
| `AnnotationNotFound` | 10 | — |
| `LinkNotFound` | 11 | — |
| `DuplicateDicomSop` | 12 | — |
| `IntegrityMismatch` | 13 | — |
| `StudyNotFound` | 14 | — |
| `StudyNotInExpectedStatus` | 15 | — |
| `ReaderNotAssigned` | 16 | — |
| `ReaderAlreadySubmitted` | 17 | — |
| `TooManyReaders` | 18 | — |
| `TooManyImages` | 19 | — |
| `AllReadersNotSubmitted` | 20 | — |
| `ArbitratorNotAssigned` | 21 | — |
| `InvalidStatusTransition` | 22 | — |
| `ReportsNotYetAvailable` | 23 | — |

### Examples

#### `end_to_end_imaging_flow_with_privacy_ai_and_safety`

```rust
let env = Env::default();
    env.mock_all_auths();
    let (client, admin) = setup(&env);

    let patient = Address::generate(&env);
    let tech = Address::generate(&env);
    let radiologist = Address::generate(&env);
    let physician = Address::generate(&env);
    let auditor = Address::generate(&env);
```

#### `supports_dicom_lookup_and_indexes`

```rust
let env = Env::default();
    env.mock_all_auths();
    let (client, admin) = setup(&env);

    let patient = Address::generate(&env);
    let tech = Address::generate(&env);
    client.assign_role(&admin, &tech, &1u32);

    let md = dicom(&env, 100);
```

#### `duplicate_dicom_sop_rejected`

```rust
let env = Env::default();
    env.mock_all_auths();
    let (client, admin) = setup(&env);

    let patient = Address::generate(&env);
    let tech = Address::generate(&env);
    client.assign_role(&admin, &tech, &1u32);

    let md = dicom(&env, 150);
```

---

## medical_imaging_ai

### Functions

| Function | Parameters | Returns | Description |
|---|---|---|---|
| `initialize` | `env: Env, admin: Address, default_warning_bps: u32, default_critical_bps: u32, default_min_samples: u64` | `Result<bool, Error>` | — |
| `pause` | `env: Env, admin: Address` | `Result<bool, Error>` | — |
| `unpause` | `env: Env, admin: Address` | `Result<bool, Error>` | — |
| `register_evaluator` | `env: Env, admin: Address, evaluator: Address` | `Result<bool, Error>` | — |
| `revoke_evaluator` | `env: Env, admin: Address, evaluator: Address` | `Result<bool, Error>` | — |
| `register_cnn_model` | `env: Env, caller: Address, model_id: BytesN<32>, modality: ImagingModality, input: CnnModelInput` | `Result<bool, Error>` | — |
| `update_model_status` | `env: Env, admin: Address, model_id: BytesN<32>, new_status: ModelStatus` | `Result<bool, Error>` | — |
| `submit_analysis` | `env: Env, caller: Address, image_id: u64, model_id: BytesN<32>, attestation_hash: BytesN<32>, signature: BytesN<64>, findings: Vec<Finding>, overall_confidence_bps: u32, processing_time_ms: u32` | `Result<u64, Error>` | — |
| `get_analysis` | `env: Env, result_id: u64` | `AnalysisResult` | — |
| `get_image_analyses` | `env: Env, image_id: u64` | `Vec<u64>` | — |
| `submit_segmentation` | `env: Env, caller: Address, image_id: u64, model_id: BytesN<32>, attestation_hash: BytesN<32>, signature: BytesN<64>, regions: Vec<SegmentedRegion>, processing_time_ms: u32` | `Result<u64, Error>` | — |
| `get_segmentation` | `env: Env, seg_id: u64` | `SegmentationResult` | — |
| `get_model` | `env: Env, model_id: BytesN<32>` | `CnnModelMetadata` | — |
| `is_model_active` | `env: Env, model_id: BytesN<32>` | `bool` | — |
| `record_evaluation` | `env: Env, caller: Address, result_id: u64, is_correct: bool` | `Result<ModelPerformance, Error>` | — |
| `get_performance` | `env: Env, model_id: BytesN<32>` | `ModelPerformance` | — |
| `configure_thresholds` | `env: Env, admin: Address, model_id: BytesN<32>, warning_bps: u32, critical_bps: u32, min_samples: u64, window_size: u64` | `Result<bool, Error>` | — |

### Types

#### `enum ImagingModality`

| Variant | Value | Description |
|---|---|---|
| `XRay` | — | — |
| `MRI` | — | — |
| `CT` | — | — |
| `Ultrasound` | — | — |
| `PET` | — | — |
| `Mammography` | — | — |
| `Custom(u32)` | — | — |

#### `enum ModelStatus`

| Variant | Value | Description |
|---|---|---|
| `Active` | — | — |
| `Degraded` | — | — |
| `Deactivated` | — | — |
| `Retired` | — | — |

#### `struct BoundingBox`

| Field | Type | Description |
|---|---|---|
| `x_min` | `u32` | — |
| `y_min` | `u32` | — |
| `x_max` | `u32` | — |
| `y_max` | `u32` | — |

#### `struct CnnModelInput`

| Field | Type | Description |
|---|---|---|
| `architecture_hash` | `BytesN<32>` | — |
| `version` | `u32` | — |
| `layer_count` | `u32` | — |
| `input_rows` | `u32` | — |
| `input_cols` | `u32` | — |
| `input_channels` | `u32` | — |
| `training_samples` | `u64` | — |
| `validation_accuracy_bps` | `u32` | — |
| `training_dataset_hash` | `BytesN<32>` | — |
| `signing_pubkey` | `BytesN<32>` | — |

#### `struct CnnModelMetadata`

| Field | Type | Description |
|---|---|---|
| `model_id` | `BytesN<32>` | — |
| `owner` | `Address` | — |
| `version` | `u32` | — |
| `modality` | `ImagingModality` | — |
| `architecture_hash` | `BytesN<32>` | — |
| `layer_count` | `u32` | — |
| `input_rows` | `u32` | — |
| `input_cols` | `u32` | — |
| `input_channels` | `u32` | — |
| `training_samples` | `u64` | — |
| `validation_accuracy_bps` | `u32` | — |
| `training_dataset_hash` | `BytesN<32>` | — |
| `signing_pubkey` | `BytesN<32>` | — |
| `status` | `ModelStatus` | — |
| `registered_at` | `u64` | — |
| `last_evaluated_at` | `u64` | — |

#### `struct Finding`

| Field | Type | Description |
|---|---|---|
| `finding_id` | `u32` | — |
| `condition_hash` | `BytesN<32>` | — |
| `confidence_bps` | `u32` | — |
| `severity` | `u32` | — |
| `region` | `BoundingBox` | — |
| `explanation_ref` | `String` | — |

#### `struct AnalysisResult`

| Field | Type | Description |
|---|---|---|
| `result_id` | `u64` | — |
| `image_id` | `u64` | — |
| `model_id` | `BytesN<32>` | — |
| `submitter` | `Address` | — |
| `attestation_hash` | `BytesN<32>` | — |
| `signature` | `BytesN<64>` | — |
| `findings` | `Vec<Finding>` | — |
| `overall_confidence_bps` | `u32` | — |
| `processing_time_ms` | `u32` | — |
| `created_at` | `u64` | — |

#### `struct SegmentedRegion`

| Field | Type | Description |
|---|---|---|
| `label_hash` | `BytesN<32>` | — |
| `pixel_count` | `u64` | — |
| `volume_mm3` | `u64` | — |
| `mean_intensity` | `u32` | — |
| `mask_ref` | `String` | — |
| `bounds` | `BoundingBox` | — |

#### `struct SegmentationResult`

| Field | Type | Description |
|---|---|---|
| `seg_id` | `u64` | — |
| `image_id` | `u64` | — |
| `model_id` | `BytesN<32>` | — |
| `submitter` | `Address` | — |
| `attestation_hash` | `BytesN<32>` | — |
| `signature` | `BytesN<64>` | — |
| `regions` | `Vec<SegmentedRegion>` | — |
| `processing_time_ms` | `u32` | — |
| `created_at` | `u64` | — |

#### `struct ModelPerformance`

| Field | Type | Description |
|---|---|---|
| `model_id` | `BytesN<32>` | — |
| `modality` | `ImagingModality` | — |
| `total_evaluated` | `u64` | — |
| `correct_count` | `u64` | — |
| `lifetime_accuracy_bps` | `u32` | — |
| `window_size` | `u64` | — |
| `window_correct` | `u64` | — |
| `window_total` | `u64` | — |
| `rolling_accuracy_bps` | `u32` | — |
| `avg_processing_time_ms` | `u32` | — |
| `warning_threshold_bps` | `u32` | — |
| `critical_threshold_bps` | `u32` | — |
| `min_sample_size` | `u64` | — |
| `last_updated` | `u64` | — |

#### `enum DataKey`

| Variant | Value | Description |
|---|---|---|
| `CnnModel(BytesN<32>)` | — | — |
| `AnalysisResult(u64)` | — | — |
| `SegResult(u64)` | — | — |
| `Performance(BytesN<32>)` | — | — |
| `ImageResults(u64)` | — | — |
| `ImageSegResults(u64)` | — | — |
| `Evaluator(Address)` | — | — |
| `DefaultWarningBps` | — | — |
| `DefaultCriticalBps` | — | — |
| `DefaultMinSamples` | — | — |

#### `enum Error`

| Variant | Value | Description |
|---|---|---|
| `AlreadyInitialized` | 1 | — |
| `NotInitialized` | 2 | — |
| `NotAuthorized` | 3 | — |
| `ContractPaused` | 4 | — |
| `InvalidInput` | 5 | — |
| `ModelNotFound` | 6 | — |
| `ModelNotActive` | 7 | — |
| `ModelAlreadyExists` | 8 | — |
| `ResultNotFound` | 9 | — |
| `SegmentationNotFound` | 10 | — |
| `TooManyFindings` | 11 | — |
| `TooManyRegions` | 12 | — |
| `InvalidConfidence` | 13 | — |
| `InvalidSeverity` | 14 | — |
| `InvalidThreshold` | 15 | — |
| `AttestationInvalid` | 16 | — |
| `DuplicateResult` | 17 | — |
| `InsufficientSamples` | 18 | — |

### Examples

#### `test_initialize`

```rust
let env = Env::default();
    env.mock_all_auths();
    let (_, _) = setup(&env);
```

#### `test_pause_unpause`

```rust
let env = Env::default();
    env.mock_all_auths();
    let (client, admin) = setup(&env);
    client.pause(&admin);
    client.unpause(&admin);
```

#### `test_register_and_revoke_evaluator`

```rust
let env = Env::default();
    env.mock_all_auths();
    let (client, admin) = setup(&env);
    let evaluator = Address::generate(&env);
    client.register_evaluator(&admin, &evaluator);
    client.revoke_evaluator(&admin, &evaluator);
```

---

## medical_record_backup

### Functions

| Function | Parameters | Returns | Description |
|---|---|---|---|
| `initialize` | `env: Env, admin: Address` | `Result<(), Error>` | — |
| `set_paused` | `env: Env, caller: Address, paused: bool` | `Result<bool, Error>` | — |
| `assign_role` | `env: Env, caller: Address, user: Address, role_mask: u32` | `Result<bool, Error>` | — |
| `set_policy` | `env: Env, caller: Address, policy: BackupPolicy` | `Result<bool, Error>` | — |
| `get_policy` | `env: Env` | `Result<BackupPolicy, Error>` | — |
| `register_target` | `env: Env, caller: Address, network: BackupNetwork, region: GeoRegion, endpoint_hash: BytesN<32>, encrypted_only: bool, cost_weight: u32, max_capacity_units: u64` | `Result<u32, Error>` | — |
| `set_target_active` | `env: Env, caller: Address, target_id: u32, active: bool` | `Result<bool, Error>` | — |
| `get_target` | `env: Env, target_id: u32` | `Option<BackupTarget>` | — |
| `list_targets` | `env: Env` | `Vec<BackupTarget>` | — |
| `run_scheduled_backup` | `env: Env, caller: Address, source_root: BytesN<32>, snapshot_ref: String, encryption_key_version: u32` | `Result<u64, Error>` | — |
| `run_backup_now` | `env: Env, caller: Address, source_root: BytesN<32>, snapshot_ref: String, encryption_key_version: u32` | `Result<u64, Error>` | — |
| `verify_backup_integrity` | `env: Env, caller: Address, artifact_id: u64, observed_checksum: BytesN<32>` | `Result<bool, Error>` | — |
| `request_restore` | `env: Env, caller: Address, artifact_id: u64, reason_hash: BytesN<32>` | `Result<u64, Error>` | — |
| `approve_restore` | `env: Env, caller: Address, request_id: u64` | `Result<bool, Error>` | — |
| `execute_restore` | `env: Env, caller: Address, request_id: u64` | `Result<String, Error>` | — |
| `run_recovery_test` | `env: Env, caller: Address, artifact_id: u64, validation_hash: BytesN<32>` | `Result<u64, Error>` | — |
| `optimize_and_cleanup` | `env: Env, caller: Address` | `Result<CleanupReport, Error>` | — |
| `report_target_failure` | `env: Env, caller: Address, target_id: u32, reason_hash: BytesN<32>` | `Result<bool, Error>` | — |
| `resolve_alert` | `env: Env, caller: Address, alert_id: u64` | `Result<bool, Error>` | — |
| `list_alerts` | `env: Env, open_only: bool` | `Vec<AlertEntry>` | — |
| `list_artifacts` | `env: Env, include_archived: bool` | `Vec<BackupArtifact>` | — |
| `get_artifact` | `env: Env, artifact_id: u64` | `Option<BackupArtifact>` | — |
| `get_execution` | `env: Env, execution_id: u64` | `Option<BackupExecution>` | — |
| `get_restore_request` | `env: Env, request_id: u64` | `Option<RestoreRequest>` | — |
| `get_recovery_test` | `env: Env, test_id: u64` | `Option<RecoveryTest>` | — |
| `get_health` | `env: Env` | `BackupHealth` | — |
| `get_schedule` | `env: Env` | `(u64, u64)` | — |

### Types

#### `enum BackupNetwork`

| Variant | Value | Description |
|---|---|---|
| `Stellar` | — | — |
| `Ethereum` | — | — |
| `Polygon` | — | — |
| `Arbitrum` | — | — |
| `Optimism` | — | — |
| `Avalanche` | — | — |
| `BinanceSmartChain` | — | — |
| `Ipfs` | — | — |
| `Filecoin` | — | — |
| `Arweave` | — | — |
| `AwsS3` | — | — |
| `AzureBlob` | — | — |
| `GcpStorage` | — | — |
| `Custom(u32)` | — | — |

#### `enum GeoRegion`

| Variant | Value | Description |
|---|---|---|
| `UsEast` | — | — |
| `UsWest` | — | — |
| `EuCentral` | — | — |
| `EuWest` | — | — |
| `ApSouth` | — | — |
| `ApNorth` | — | — |
| `SaEast` | — | — |
| `AfSouth` | — | — |
| `Custom(u32)` | — | — |

#### `enum BackupStatus`

| Variant | Value | Description |
|---|---|---|
| `Completed` | — | — |
| `Verified` | — | — |
| `Failed` | — | — |
| `Archived` | — | — |
| `Restored` | — | — |

#### `enum ReplicaStatus`

| Variant | Value | Description |
|---|---|---|
| `Synced` | — | — |
| `Verified` | — | — |
| `Failed` | — | — |
| `Archived` | — | — |

#### `enum AlertSeverity`

| Variant | Value | Description |
|---|---|---|
| `Low` | — | — |
| `Medium` | — | — |
| `High` | — | — |
| `Critical` | — | — |

#### `enum AlertKind`

| Variant | Value | Description |
|---|---|---|
| `BackupFailure` | — | — |
| `TargetFailure` | — | — |
| `GeoRedundancyRisk` | — | — |
| `IntegrityCheckFailed` | — | — |
| `RestoreFailure` | — | — |
| `CostThresholdExceeded` | — | — |
| `ScheduleMissed` | — | — |
| `RecoveryDrillFailed` | — | — |

#### `enum RestoreStatus`

| Variant | Value | Description |
|---|---|---|
| `Pending` | — | — |
| `Approved` | — | — |
| `Executed` | — | — |
| `Rejected` | — | — |

#### `struct BackupPolicy`

| Field | Type | Description |
|---|---|---|
| `interval_seconds` | `u64` | — |
| `retention_seconds` | `u64` | — |
| `max_active_backups` | `u32` | — |
| `min_targets_per_backup` | `u32` | — |
| `min_region_count` | `u32` | — |
| `max_total_cost_weight` | `u32` | — |
| `verify_on_write` | `bool` | — |
| `encryption_required` | `bool` | — |
| `auto_cleanup` | `bool` | — |
| `min_restore_approvals` | `u32` | — |

#### `struct BackupTarget`

| Field | Type | Description |
|---|---|---|
| `target_id` | `u32` | — |
| `network` | `BackupNetwork` | — |
| `region` | `GeoRegion` | — |
| `endpoint_hash` | `BytesN<32>` | — |
| `is_active` | `bool` | — |
| `encrypted_only` | `bool` | — |
| `cost_weight` | `u32` | — |
| `max_capacity_units` | `u64` | — |
| `failure_count` | `u32` | — |

#### `struct BackupArtifact`

| Field | Type | Description |
|---|---|---|
| `artifact_id` | `u64` | — |
| `source_root` | `BytesN<32>` | — |
| `checksum` | `BytesN<32>` | — |
| `snapshot_ref` | `String` | — |
| `encryption_key_version` | `u32` | — |
| `encrypted` | `bool` | — |
| `created_at` | `u64` | — |
| `expires_at` | `u64` | — |
| `target_ids` | `Vec<u32>` | — |
| `region_count` | `u32` | — |
| `total_cost_weight` | `u32` | — |
| `status` | `BackupStatus` | — |
| `last_verified_at` | `u64` | — |
| `last_restored_at` | `u64` | — |
| `restore_drill_passed` | `bool` | — |

#### `struct BackupReplica`

| Field | Type | Description |
|---|---|---|
| `artifact_id` | `u64` | — |
| `target_id` | `u32` | — |
| `checksum` | `BytesN<32>` | — |
| `synced_at` | `u64` | — |
| `status` | `ReplicaStatus` | — |

#### `struct BackupExecution`

| Field | Type | Description |
|---|---|---|
| `execution_id` | `u64` | — |
| `triggered_by` | `Address` | — |
| `started_at` | `u64` | — |
| `completed_at` | `u64` | — |
| `scheduled` | `bool` | — |
| `success_targets` | `u32` | — |
| `failed_targets` | `u32` | — |
| `artifact_id` | `Option<u64>` | — |
| `error_code` | `Option<u32>` | — |

#### `struct AlertEntry`

| Field | Type | Description |
|---|---|---|
| `alert_id` | `u64` | — |
| `kind` | `AlertKind` | — |
| `severity` | `AlertSeverity` | — |
| `created_at` | `u64` | — |
| `details_hash` | `BytesN<32>` | — |
| `resolved` | `bool` | — |
| `resolved_at` | `u64` | — |

#### `struct RecoveryTest`

| Field | Type | Description |
|---|---|---|
| `test_id` | `u64` | — |
| `artifact_id` | `u64` | — |
| `started_by` | `Address` | — |
| `executed_at` | `u64` | — |
| `validation_hash` | `BytesN<32>` | — |
| `passed` | `bool` | — |

#### `struct RestoreRequest`

| Field | Type | Description |
|---|---|---|
| `request_id` | `u64` | — |
| `artifact_id` | `u64` | — |
| `requested_by` | `Address` | — |
| `reason_hash` | `BytesN<32>` | — |
| `requested_at` | `u64` | — |
| `approvals` | `Vec<Address>` | — |
| `status` | `RestoreStatus` | — |
| `executed_at` | `u64` | — |

#### `struct BackupHealth`

| Field | Type | Description |
|---|---|---|
| `total_runs` | `u64` | — |
| `successful_runs` | `u64` | — |
| `failed_runs` | `u64` | — |
| `consecutive_failures` | `u32` | — |
| `last_success_at` | `u64` | — |
| `last_failure_at` | `u64` | — |
| `last_error_code` | `u32` | — |

#### `struct CleanupReport`

| Field | Type | Description |
|---|---|---|
| `archived_backups` | `u32` | — |
| `reclaimed_cost_weight` | `u32` | — |
| `remaining_active_backups` | `u32` | — |

#### `enum DataKey`

| Variant | Value | Description |
|---|---|---|
| `Policy` | — | — |
| `Roles(Address)` | — | — |
| `Target(u32)` | — | — |
| `TargetIds` | — | — |
| `Artifact(u64)` | — | — |
| `ArtifactIds` | — | — |
| `Replica(u64, u32)` | — | — |
| `Execution(u64)` | — | — |
| `Alert(u64)` | — | — |
| `AlertIds` | — | — |
| `RecoveryTest(u64)` | — | — |
| `RestoreRequest(u64)` | — | — |
| `Health` | — | — |

#### `enum Error`

| Variant | Value | Description |
|---|---|---|
| `AlreadyInitialized` | 1 | — |
| `NotInitialized` | 2 | — |
| `NotAuthorized` | 3 | — |
| `ContractPaused` | 4 | — |
| `InvalidInput` | 5 | — |
| `TargetNotFound` | 6 | — |
| `BackupNotFound` | 7 | — |
| `RestoreRequestNotFound` | 8 | — |
| `RecoveryTestNotFound` | 9 | — |
| `ScheduleNotDue` | 10 | — |
| `InsufficientTargets` | 11 | — |
| `GeoRedundancyNotMet` | 12 | — |
| `EncryptionRequired` | 13 | — |
| `IntegrityMismatch` | 14 | — |
| `RestoreNotApproved` | 15 | — |
| `AlreadyExecuted` | 16 | — |
| `DuplicateApproval` | 17 | — |
| `CostLimitExceeded` | 18 | — |

### Examples

#### `backup_run_creates_geo_redundant_artifact`

```rust
let env = Env::default();
    env.mock_all_auths();
    let (client, admin) = setup(&env);
    register_two_targets(&client, &admin, &env);

    let id = client.run_backup_now(
        &admin,
        &sample_hash(&env, 7),
        &String::from_str(&env, "ipfs://snapshot-a"),
```

#### `scheduled_backup_respects_interval`

```rust
let env = Env::default();
    env.mock_all_auths();
    let (client, admin) = setup(&env);
    register_two_targets(&client, &admin, &env);

    let policy = BackupPolicy {
        interval_seconds: 1_000,
        retention_seconds: 10_000,
        max_active_backups: 10,
```

#### `integrity_mismatch_creates_alert`

```rust
let env = Env::default();
    env.mock_all_auths();
    let (client, admin) = setup(&env);
    register_two_targets(&client, &admin, &env);

    let id = client.run_backup_now(
        &admin,
        &sample_hash(&env, 3),
        &String::from_str(&env, "ipfs://snapshot-d"),
```

---

## medical_record_hash_registry

### Functions

| Function | Parameters | Returns | Description |
|---|---|---|---|
| `initialize` | `env: Env, admin: Address` | `Result<(), Error>` | Initialize the contract with an admin |
| `store_record` | `env: Env, caller: Address, patient_id: Address, record_hash: BytesN<32>` | `Result<(), Error>` | Store a medical record hash for a patient Returns an error if: - Contract is not initialized - The same hash already exists for this patient (duplicate detection) |
| `verify_record` | `env: Env, patient_id: Address, record_hash: BytesN<32>` | `Result<bool, Error>` | Verify if a record hash exists and is valid for a patient Returns true if the record exists and is verified, false otherwise |
| `get_patient_by_hash` | `env: Env, record_hash: BytesN<32>` | `Option<Address>` | Get the patient ID associated with a specific record hash |
| `get_patient_records` | `env: Env, patient_id: Address` | `Option<PatientRecords>` | Get all records for a patient |
| `get_record_count` | `env: Env, patient_id: Address` | `u32` | Get the count of records for a patient |
| `get_admin` | `env: Env` | `Result<Address, Error>` | Get the current admin |

### Types

#### `struct RecordEntry`

| Field | Type | Description |
|---|---|---|
| `patient_id` | `Address` | — |
| `record_hash` | `BytesN<32>` | — |
| `timestamp` | `u64` | — |
| `verified` | `bool` | — |

#### `struct PatientRecords`

| Field | Type | Description |
|---|---|---|
| `records` | `Vec<RecordEntry>` | — |
| `record_count` | `u32` | — |

#### `enum DataKey`

| Variant | Value | Description |
|---|---|---|
| `Initialized` | — | — |
| `Admin` | — | — |
| `RecordStorage(Address)` | — | — |
| `HashIndex(BytesN<32>),` | — | — |

### Error Codes

| Variant | Code | Description |
|---|---|---|
| `Unauthorized` | 100 | — |
| `InvalidId` | 206 | — |
| `InvalidSignature` | 207 | — |
| `InvalidRecordHash` | 251 | — |
| `NotInitialized` | 300 | — |
| `AlreadyInitialized` | 301 | — |
| `ContractPaused` | 302 | — |
| `DeadlineExceeded` | 306 | — |
| `DuplicateRecord` | 402 | — |
| `RecordNotFound` | 403 | — |
| `InsufficientFunds` | 500 | — |
| `StorageFull` | 502 | — |
| `CrossChainTimeout` | 702 | — |

### Examples

#### `test_initialize`

```rust
let (_env, client, admin) = setup();
        client.initialize(&admin);
    }

    #[test]
    fn test_initialize_twice_fails() {
        let (_env, client, admin) = setup();
        client.initialize(&admin);
        let result = client.try_initialize(&admin);
```

---

## medical_record_search

### Functions

| Function | Parameters | Returns | Description |
|---|---|---|---|
| `initialize` | `env: Env, admin: Address` | `Result<(), Error>` | — |
| `set_paused` | `env: Env, caller: Address, paused: bool` | `Result<bool, Error>` | — |
| `assign_role` | `env: Env, caller: Address, user: Address, role_mask: u32` | `Result<bool, Error>` | — |
| `set_cache_policy` | `env: Env, caller: Address, policy: CachePolicy` | `Result<bool, Error>` | — |
| `set_ranking` | `env: Env, caller: Address, cfg: RankingConfig` | `Result<bool, Error>` | — |
| `index_record` | `env: Env, caller: Address, input: IndexInput` | `Result<bool, Error>` | — |
| `batch_index_records` | `env: Env, caller: Address, inputs: Vec<IndexInput>` | `Result<(u32, u32), Error>` | — |
| `search` | `env: Env, caller: Address, query: SearchQuery, page: u32, page_size: u32` | `Result<Vec<SearchResult>, Error>` | — |
| `get_cache_entry` | `env: Env, query_hash: BytesN<32>` | `Result<QueryCacheEntry, Error>` | — |
| `invalidate_cache` | `env: Env, caller: Address` | `Result<bool, Error>` | — |
| `get_audit` | `env: Env, caller: Address, query_id: u64` | `Result<SearchAuditEntry, Error>` | — |
| `preview_query_hash` | `env: Env, query: SearchQuery` | `BytesN<32>` | — |
| `get_indexed_entry` | `env: Env, record_id: u64` | `Result<SearchIndexEntry, Error>` | — |

### Types

#### `enum ChainId`

| Variant | Value | Description |
|---|---|---|
| `Stellar` | — | — |
| `Ethereum` | — | — |
| `Polygon` | — | — |
| `Avalanche` | — | — |
| `Arbitrum` | — | — |
| `Optimism` | — | — |
| `Custom(u32)` | — | — |

#### `struct IndexInput`

| Field | Type | Description |
|---|---|---|
| `record_id` | `u64` | — |
| `patient` | `Address` | — |
| `network` | `ChainId` | — |
| `created_at` | `u64` | — |
| `is_confidential` | `bool` | — |
| `category_hash` | `BytesN<32>` | — |
| `token_hashes` | `Vec<BytesN<32>>` | — |
| `attribute_hashes` | `Vec<BytesN<32>>` | — |
| `encrypted_ref_hash` | `BytesN<32>` | — |
| `quality_score_bps` | `u32` | — |

#### `struct SearchIndexEntry`

| Field | Type | Description |
|---|---|---|
| `record_id` | `u64` | — |
| `indexed_by` | `Address` | — |
| `patient` | `Address` | — |
| `network` | `ChainId` | — |
| `created_at` | `u64` | — |
| `is_confidential` | `bool` | — |
| `category_hash` | `BytesN<32>` | — |
| `token_hashes` | `Vec<BytesN<32>>` | — |
| `attribute_hashes` | `Vec<BytesN<32>>` | — |
| `encrypted_ref_hash` | `BytesN<32>` | — |
| `quality_score_bps` | `u32` | — |

#### `struct SearchQuery`

| Field | Type | Description |
|---|---|---|
| `required_tokens` | `Vec<BytesN<32>>` | — |
| `optional_tokens` | `Vec<BytesN<32>>` | — |
| `category_filters` | `Vec<BytesN<32>>` | — |
| `attribute_filters` | `Vec<BytesN<32>>` | — |
| `network_filters` | `Vec<ChainId>` | — |
| `patient_filter` | `Option<Address>` | — |
| `from_timestamp` | `u64` | — |
| `to_timestamp` | `u64` | — |
| `include_confidential` | `bool` | — |
| `min_quality_bps` | `u32` | — |

#### `struct SearchResult`

| Field | Type | Description |
|---|---|---|
| `record_id` | `u64` | — |
| `patient` | `Address` | — |
| `network` | `ChainId` | — |
| `created_at` | `u64` | — |
| `encrypted_ref_hash` | `BytesN<32>` | — |
| `is_confidential` | `bool` | — |
| `score_bps` | `u32` | — |

#### `struct QueryCacheEntry`

| Field | Type | Description |
|---|---|---|
| `query_hash` | `BytesN<32>` | — |
| `created_at` | `u64` | — |
| `expires_at` | `u64` | — |
| `hit_count` | `u32` | — |
| `results` | `Vec<SearchResult>` | — |

#### `struct CachePolicy`

| Field | Type | Description |
|---|---|---|
| `ttl_seconds` | `u64` | — |
| `max_entries` | `u32` | — |

#### `struct RankingConfig`

| Field | Type | Description |
|---|---|---|
| `required_weight_bps` | `u32` | — |
| `optional_weight_bps` | `u32` | — |
| `recency_weight_bps` | `u32` | — |
| `quality_weight_bps` | `u32` | — |

#### `struct SearchAuditEntry`

| Field | Type | Description |
|---|---|---|
| `query_id` | `u64` | — |
| `caller` | `Address` | — |
| `query_hash` | `BytesN<32>` | — |
| `timestamp` | `u64` | — |
| `result_count` | `u32` | — |
| `from_cache` | `bool` | — |
| `granted` | `bool` | — |

#### `enum DataKey`

| Variant | Value | Description |
|---|---|---|
| `Roles(Address)` | — | — |
| `Index(u64)` | — | — |
| `IndexedIds` | — | — |
| `TokenPosting(BytesN<32>)` | — | — |
| `CategoryPosting(BytesN<32>)` | — | — |
| `AttributePosting(BytesN<32>)` | — | — |
| `Cache(BytesN<32>)` | — | — |
| `CacheOrder` | — | — |
| `CachePolicy` | — | — |
| `Ranking` | — | — |
| `Audit(u64)` | — | — |

#### `enum Error`

| Variant | Value | Description |
|---|---|---|
| `AlreadyInitialized` | 1 | — |
| `NotInitialized` | 2 | — |
| `NotAuthorized` | 3 | — |
| `ContractPaused` | 4 | — |
| `InvalidInput` | 5 | — |
| `RecordNotIndexed` | 6 | — |
| `QueryTooLarge` | 7 | — |
| `CacheMiss` | 8 | — |

### Examples

#### `search_ranking_and_cache_work`

```rust
let env = Env::default();
    env.mock_all_auths();
    let (client, admin) = setup(&env);

    let indexer = Address::generate(&env);
    let searcher = Address::generate(&env);
    client.assign_role(
        &admin,
        &indexer,
```

#### `search_requires_role`

```rust
let env = Env::default();
    env.mock_all_auths();
    let (client, admin) = setup(&env);
    let indexer = Address::generate(&env);
    let unauthorized = Address::generate(&env);
    client.assign_role(&admin, &indexer, &ROLE_INDEXER);

    index_entry(
        &env,
```

#### `confidential_records_require_confidential_role`

```rust
let env = Env::default();
    env.mock_all_auths();
    let (client, admin) = setup(&env);
    let indexer = Address::generate(&env);
    let basic_searcher = Address::generate(&env);
    let privileged_searcher = Address::generate(&env);
    client.assign_role(&admin, &indexer, &(ROLE_INDEXER | ROLE_SEARCHER));
    client.assign_role(&admin, &basic_searcher, &ROLE_SEARCHER);
    client.assign_role(
```

---

## medical_records

### Functions

| Function | Parameters | Returns | Description |
|---|---|---|---|
| `initialize` | `env: Env, admin: Address, rbac_contract: Address` | `bool` | Initialize the contract, setting the admin and default storage values. |
| `health_check` | `env: Env` | `(Symbol, u32, u64)` | Return contract status, current version, and ledger timestamp. |
| `set_audit_forensics` | `env: Env, admin: Address, contract_id: Address` | `Result<bool, Error>` | Set the audit/forensics contract address; only callable by admin. |
| `get_audit_forensics` | `env: Env` | `Option<Address>` | Return the registered audit/forensics contract address, if set. |
| `manage_user` | `env: Env, caller: Address, user: Address, role: Role` | `Result<bool, Error>` | Register or update a user's role; only callable by admin. |
| `set_user_qkd_status` | `env: Env, admin: Address, user: Address, capable: bool` | `Result<(), Error>` | — |
| `is_user_qkd_capable` | `env: Env, user: Address` | `bool` | — |
| `deactivate_user` | `env: Env, caller: Address, user: Address` | `Result<bool, Error>` | — |
| `get_user_role` | `env: Env, user: Address` | `Result<Role, Error>` | — |
| `pause` | `env: Env, caller: Address` | `Result<bool, Error>` | — |
| `unpause` | `env: Env, caller: Address` | `Result<bool, Error>` | — |
| `grant_permission` | `env: Env, granter: Address, grantee: Address, permission: Permission, expiration: u64, // 0 = permanent is_delegatable: bool` | `Result<bool, Error>` | — |
| `revoke_permission` | `env: Env, revoker: Address, grantee: Address, permission: Permission` | `Result<bool, Error>` | — |
| `issue_access_attribute` | `env: Env, issuer: Address, user: Address, namespace: String, value: String, expires_at: u64, is_verified: bool` | `Result<bool, Error>` | — |
| `revoke_access_attribute` | `env: Env, revoker: Address, user: Address, namespace: String, value: String` | `Result<bool, Error>` | — |
| `get_user_access_attributes` | `env: Env, user: Address` | `Result<Vec<UserAccessAttribute>, Error>` | — |
| `get_access_attribute_epoch` | `env: Env, namespace: String, value: String` | `Result<u32, Error>` | — |
| `add_record` | `env: Env, caller: Address, patient: Address, diagnosis: String, treatment: String, is_confidential: bool, tags: Vec<String>, category: String, treatment_type: String, data_ref: String` | `Result<u64, Error>` | Store a new medical record; enforces RBAC and consent checks. |
| `write_record` | `env: Env, caller: Address, patient: Address, diagnosis: String, treatment: String, is_confidential: bool, tags: Vec<String>, category: String, treatment_type: String, data_ref: String, traditional_metadata: Option<TraditionalMedicineMetadata>` | `Result<u64, Error>` | Write a medical record with optional traditional medicine metadata.  This is the canonical entry-point for records that may involve traditional healing practices. When `traditional_metadata` is `Some`, the metadata is stored encrypted alongside the main record and the record ID is appended to the patient-scoped traditional-records index so it can be queried separately via `list_traditional_records`.  Calling with `traditional_metadata: None` is fully backward-compatible with the existing `add_record` behaviour. |
| `write_record_batch` | `env: Env, caller: Address, records: Vec<RecordInput>` | `Result<Vec<u64>, Error>` | Create multiple medical records in a single atomic call.  All-or-nothing semantics: if any record fails validation, the entire batch is rejected and no records are persisted.  ## Limits - Max 50 records per batch. |
| `list_traditional_records` | `env: Env, caller: Address, patient_id: Address` | `Result<Vec<u64>, Error>` | Return the record IDs of all traditional-medicine records for a patient.  Only the patient themselves, an admin, or a caller with `ReadRecord` permission may invoke this function. |
| `add_record_with_did` | `env: Env, caller: Address, patient: Address, diagnosis: String, treatment: String, is_confidential: bool, tags: Vec<String>, category: String, treatment_type: String, data_ref: String, _credential_ref: Option<String>` | `Result<u64, Error>` | — |
| `get_record` | `env: Env, caller: Address, record_id: u64` | `Result<MedicalRecord, Error>` | Retrieve a medical record by ID; enforces caller authorization and access control. |
| `get_record_with_did` | `env: Env, caller: Address, record_id: u64, purpose: String` | `Result<Option<MedicalRecord>, Error>` | — |
| `get_record_metadata` | `env: Env, record_id: u64` | `Result<RecordMetadata, Error>` | — |
| `get_history` | `env: Env, caller: Address, patient: Address, page: u32, page_size: u32` | `Result<Vec<(u64, RecordMetadata)>, Error>` | — |
| `get_record_count` | `env: Env` | `u64` | Return the total number of records stored in the contract. |
| `get_patient_record_count` | `env: Env, patient: Address` | `u64` | — |
| `get_patient_record_id` | `env: Env, patient: Address, index: u64` | `Option<u64>` | — |
| `list_records` | `env: Env, caller: Address, cursor: Option<u64>, limit: u32` | `Result<ListRecordsResult, Error>` | List medical records using cursor-based pagination. Returns up to `limit` records starting after the given cursor. `cursor` is the last record_id from a previous page (None for first page). `limit` must be between 1 and 100. |
| `set_zk_verifier_contract` | `env: Env, caller: Address, verifier: Address` | `Result<bool, Error>` | — |
| `get_zk_verifier_contract` | `env: Env` | `Option<Address>` | — |
| `set_credential_registry_contract` | `env: Env, caller: Address, registry: Address` | `Result<bool, Error>` | — |
| `get_credential_registry_contract` | `env: Env` | `Option<Address>` | — |
| `set_patient_consent_contract` | `env: Env, caller: Address, consent_contract: Address` | `Result<bool, Error>` | — |
| `get_patient_consent_contract` | `env: Env` | `Option<Address>` | — |
| `set_zk_enforced` | `env: Env, caller: Address, enforced: bool` | `Result<bool, Error>` | — |
| `is_zk_enforced` | `env: Env` | `bool` | — |
| `set_zk_grant_ttl` | `env: Env, caller: Address, ttl_secs: u64` | `Result<bool, Error>` | — |
| `get_zk_grant_ttl` | `env: Env` | `u64` | — |
| `get_record_commitment` | `env: Env, record_id: u64` | `Option<BytesN<32>>` | — |
| `has_valid_zk_access_grant` | `env: Env, requester: Address, record_id: u64` | `bool` | — |
| `submit_zk_access_proof` | `env: Env, caller: Address, record_id: u64, purpose: String, public_inputs: ZkPublicInputs, proof: Bytes` | `Result<bool, Error>` | — |
| `update_record_metadata` | `env: Env, caller: Address, record_id: u64, tags: Vec<String>, custom_fields: Map<String, String>` | `Result<(), Error>` | Updates tags and custom metadata fields for an existing record. Only the record's doctor or an admin may call this. Each update creates a versioned history entry. |
| `search_records_by_tag` | `env: Env, caller: Address, tag: String, page: u32, page_size: u32` | `Result<Vec<u64>, Error>` | Returns record IDs that are indexed under a given tag, paginated. Any authenticated user may search. |
| `export_record_metadata` | `env: Env, caller: Address, record_id: u64` | `Result<RecordMetadata, Error>` | Exports full metadata (including history) for a record. Accessible by the patient, the record's doctor, or an admin. |
| `import_record_metadata` | `env: Env, caller: Address, record_id: u64, tags: Vec<String>, custom_fields: Map<String, String>` | `Result<(), Error>` | Admin-only: imports (overwrites) tags and custom fields for a record. Useful for data migration. Creates a history entry before overwriting. |
| `set_crypto_registry` | `env: Env, caller: Address, registry: Address` | `Result<bool, Error>` | — |
| `get_crypto_registry` | `env: Env` | `Option<Address>` | — |
| `set_homomorphic_registry` | `env: Env, caller: Address, registry: Address` | `Result<bool, Error>` | — |
| `get_homomorphic_registry` | `env: Env` | `Option<Address>` | — |
| `set_mpc_manager` | `env: Env, caller: Address, manager: Address` | `Result<bool, Error>` | — |
| `get_mpc_manager` | `env: Env` | `Option<Address>` | — |
| `set_encryption_required` | `env: Env, caller: Address, required: bool` | `Result<bool, Error>` | — |
| `is_encryption_required` | `env: Env` | `bool` | — |
| `set_regulatory_compliance` | `env: Env, caller: Address, compliance: Address` | `Result<bool, Error>` | — |
| `get_regulatory_compliance` | `env: &Env` | `Option<Address>` | — |
| `set_require_pq_envelopes` | `env: Env, caller: Address, required: bool` | `Result<bool, Error>` | — |
| `is_require_pq_envelopes` | `env: Env` | `bool` | — |
| `propose_crypto_config_update` | `env: Env, caller: Address, new_crypto_registry: Option<Address>, new_homomorphic_registry: Option<Address>, new_mpc_manager: Option<Address>, encryption_required: Option<bool>, require_pq_envelopes: Option<bool>` | `Result<u64, Error>` | — |
| `approve_crypto_config_update` | `env: Env, caller: Address, proposal_id: u64` | `Result<bool, Error>` | — |
| `execute_crypto_config_update` | `env: Env, caller: Address, proposal_id: u64` | `Result<bool, Error>` | — |
| `get_crypto_config_proposal` | `env: Env, caller: Address, proposal_id: u64` | `Result<Option<CryptoConfigProposal>, Error>` | — |
| `set_quantum_threat_level` | `env: Env, admin: Address, level: u32` | `Result<(), Error>` | — |
| `get_quantum_threat_level` | `env: Env` | `u32` | — |
| `upgrade_record_to_quantum_safe` | `env: Env, caller: Address, record_id: u64, new_envelope: KeyEnvelope` | `Result<(), Error>` | Migrates a record to include a new quantum-safe envelope. Accessible by the patient or an authorized doctor. |
| `add_advanced_encrypted_record` | `env: Env, caller: Address, patient: Address, is_confidential: bool, tags: Vec<String>, category: String, treatment_type: String, advanced: AdvancedEncryptedRecordInput` | `Result<u64, Error>` | — |
| `add_encrypted_record` | `env: Env, caller: Address, patient: Address, is_confidential: bool, tags: Vec<String>, category: String, treatment_type: String, ciphertext_ref: String, ciphertext_hash: BytesN<32>, envelopes: Vec<KeyEnvelope>` | `Result<u64, Error>` | — |
| `bind_encrypted_record_abe_policy` | `env: Env, caller: Address, record_id: u64, policy_ref: String, policy_hash: BytesN<32>, access_ciphertext_ref: String, access_ciphertext_hash: BytesN<32>, required_permission: Permission, attribute_count: u32, valid_until: u64, revocation_epoch: u32` | `Result<bool, Error>` | — |
| `get_encrypted_record_header` | `env: Env, caller: Address, record_id: u64` | `Result<Option<EncryptedRecordHeader>, Error>` | — |
| `get_encrypted_record_envelope` | `env: Env, caller: Address, record_id: u64` | `Result<Option<KeyEnvelope>, Error>` | — |
| `upsert_encrypted_record_envelope` | `env: Env, caller: Address, record_id: u64, envelope: KeyEnvelope` | `Result<bool, Error>` | — |
| `get_encrypted_record_abe_policy` | `env: Env, caller: Address, record_id: u64` | `Result<Option<AbePolicyMetadata>, Error>` | — |
| `get_crypto_audit_logs` | `env: Env, caller: Address, page: u32, page_size: u32` | `Result<Vec<CryptoAuditEntry>, Error>` | — |
| `set_identity_registry` | `env: Env, caller: Address, registry: Address` | `Result<bool, Error>` | — |
| `get_identity_registry` | `env: Env` | `Option<Address>` | — |
| `set_did_auth_level` | `env: Env, caller: Address, level: DIDAuthLevel` | `Result<bool, Error>` | — |
| `get_did_auth_level` | `env: Env` | `DIDAuthLevel` | — |
| `link_did_to_user` | `env: Env, caller: Address, user: Address, did: String` | `Result<bool, Error>` | — |
| `get_user_did` | `env: Env, user: Address` | `Option<String>` | — |
| `verify_professional_credential` | `env: Env, user: Address` | `bool` | Minimal on-chain verifier used by tests: returns true iff the user is an active Doctor. |
| `set_ai_config` | `env: Env, caller: Address, ai_coordinator: Address, dp_epsilon: u32, min_participants: u32` | `Result<bool, Error>` | — |
| `get_ai_config` | `env: Env` | `Option<AIConfig>` | — |
| `submit_anomaly_score` | `env: Env, caller: Address, record_id: u64, model_id: BytesN<32>, score_bps: u32, explanation_ref: String, explanation_summary: String, model_version: String, _feature_importance: Vec<(String, u32` | `()` | — |
| `get_anomaly_score` | `env: Env, caller: Address, record_id: u64` | `Result<Option<AIInsight>, Error>` | — |
| `submit_risk_score` | `env: Env, caller: Address, patient: Address, model_id: BytesN<32>, score_bps: u32, explanation_ref: String, explanation_summary: String, model_version: String, _feature_importance: Vec<(String, u32` | `()` | — |
| `get_latest_risk_score` | `env: Env, caller: Address, patient: Address` | `Result<Option<AIInsight>, Error>` | — |
| `grant_emergency_access` | `env: Env, caller: Address, grantee: Address, duration_secs: u64, record_scope: Vec<u64>` | `Result<bool, Error>` | — |
| `has_emergency_access` | `env: Env, grantee: Address, patient: Address, record_id: u64` | `bool` | — |
| `revoke_emergency_access` | `env: Env, caller: Address, grantee: Address` | `Result<bool, Error>` | — |
| `get_patient_emergency_grants` | `env: Env, patient: Address` | `Vec<EmergencyAccess>` | — |
| `get_patient_access_logs` | `env: Env, caller: Address, patient: Address, page: u32, page_size: u32` | `Vec<AccessRequest>` | — |
| `get_access_logs` | `env: Env, page: u32, page_size: u32` | `Vec<AccessRequest>` | — |
| `propose_recovery` | `env: Env, caller: Address, token_contract: Address, to: Address, amount: i128` | `Result<u64, Error>` | — |
| `approve_recovery` | `env: Env, caller: Address, proposal_id: u64` | `Result<bool, Error>` | — |
| `execute_recovery` | `env: Env, caller: Address, proposal_id: u64` | `Result<bool, Error>` | — |
| `set_cross_chain_contracts` | `env: Env, caller: Address, bridge: Address, identity: Address, access: Address` | `Result<bool, Error>` | — |
| `set_cross_chain_enabled` | `env: Env, caller: Address, enabled: bool` | `Result<bool, Error>` | — |
| `is_cross_chain_enabled` | `env: Env` | `bool` | — |
| `register_cross_chain_ref` | `env: Env, caller: Address, record_id: u64, chain: ChainId, external_record_hash: BytesN<32>` | `Result<bool, Error>` | — |
| `update_cross_chain_sync` | `env: Env, caller: Address, record_id: u64, chain: ChainId, new_external_hash: BytesN<32>` | `Result<bool, Error>` | — |
| `get_cross_chain_ref` | `env: Env, record_id: u64, chain: ChainId` | `Option<CrossChainRecordRef>` | — |
| `get_all_cross_chain_refs` | `env: Env, record_id: u64` | `Vec<CrossChainRecordRef>` | — |
| `get_record_cross_chain` | `env: Env, caller: Address, record_id: u64, _chain: ChainId, _access_token: String` | `Result<Option<MedicalRecord>, Error>` | — |
| `upgrade` | `env: Env, caller: Address, new_wasm_hash: BytesN<32>, new_version: u32` | `Result<(), Error>` | — |
| `validate_upgrade` | `env: Env, new_wasm_hash: BytesN<32>` | `Result<upgradeability::UpgradeValidation, Error>` | — |
| `version` | `env: Env` | `u32` | — |
| `export_patient_data` | `env: Env, patient_id: Address, format: ExportFormat` | `Result<Bytes, Error>` | Export all patient data in the requested format for data portability. Only the patient themselves can request their export. Rate-limited to one export per 24 hours per patient. |
| `set_rate_limit_config` | `env: Env, admin: Address, op: u32, config: RateLimitConfig` | `Result<bool, Error>` | Configure the rate limit for a specific operation (admin only). |
| `set_rate_limit_bypass` | `env: Env, admin: Address, account: Address, bypass: bool` | `Result<bool, Error>` | Grant or revoke rate-limit bypass for an account (admin only). |
| `validate_record_quality` | `env: Env, caller: Address, record_id: u64` | `Result<ValidationReport, Error>` | Validates a stored medical record and returns a comprehensive quality report.  Performs completeness checks, format validation, consistency verification, and FHIR compliance assessment. Emits a `DataQualityValidated` event. |
| `get_field_completeness` | `env: Env, caller: Address, record_id: u64` | `Result<FieldCompleteness, Error>` | Returns field-level completeness / gap detection for a stored record. |
| `validate_record_type` | `env: Env, caller: Address, record_id: u64, record_type: MedicalRecordType` | `Result<bool, Error>` | Validates a stored record against type-specific rules. |
| `get_correction_workflow` | `env: Env, caller: Address, record_id: u64` | `Result<CorrectionWorkflow, Error>` | Returns a prioritised `CorrectionWorkflow` for a stored medical record.  The workflow maps every validation issue into an actionable `CorrectionItem` (with severity-based priority and suggested fix), counts issues by category, and sets `can_auto_fix` when only minor, non-blocking issues remain.  Callers with `ReadRecord` permission may invoke this function to build a step-by-step remediation plan without modifying the stored record. |
| `cleanse_record_data` | `env: Env, caller: Address, record_id: u64` | `Result<CleanseResult, Error>` | Auto-cleanses a stored medical record using deterministic normalization rules.  Applies safe, non-clinical transformations: - Normalises category casing to the canonical allowed value. - Removes empty `doctor_did` strings (replaces `Some("")` with `None`).  If any changes were made, the updated record is persisted and a `DataQualityValidated` event is emitted with the post-cleanse quality score. Returns a `CleanseResult` describing what (if anything) changed.  Requires `UpdateRecord` permission. |
| `initialize` | `env: Env, admin: Address, config: soroban_sdk::Val` | `()` | — |
| `has_role` | `env: Env, address: Address, role: RbacRole` | `Result<bool, RbacError>` | — |
| `assign_role` | `env: Env, address: Address, role: RbacRole` | `Result<bool, RbacError>` | — |
| `remove_role` | `env: Env, address: Address, role: RbacRole` | `Result<bool, RbacError>` | — |
| `add_record_with_traditional` | `env: Env, caller: Address, patient: Address, diagnosis: String, treatment: String, is_confidential: bool, tags: Vec<String>, category: String, treatment_type: String, data_ref: String, traditional_metadata: Option<TraditionalMedicineMetadata>` | `Result<u64, Error>` | Store a medical record with optional traditional medicine metadata. When `traditional_metadata` is provided, the record is also indexed for separate querying via `list_traditional_records`. |
| `list_traditional_records` | `env: Env, caller: Address, patient: Address` | `Result<Vec<u64>, Error>` | List traditional medicine records for a patient. Returns record IDs that have associated traditional medicine metadata. |

### Types

#### `enum ChainId`

| Variant | Value | Description |
|---|---|---|
| `Stellar` | — | — |
| `Ethereum` | — | — |
| `Polygon` | — | — |
| `Avalanche` | — | — |
| `BinanceSmartChain` | — | — |
| `Arbitrum` | — | — |
| `Optimism` | — | — |
| `Custom(u32)` | — | — |

#### `struct CrossChainRecordRef`

| Field | Type | Description |
|---|---|---|
| `local_record_id` | `u64` | — |
| `external_chain` | `ChainId` | — |
| `external_record_hash` | `BytesN<32>` | — |
| `sync_timestamp` | `u64` | — |
| `is_synced` | `bool` | — |

#### `struct RecordMetadata`

| Field | Type | Description |
|---|---|---|
| `record_id` | `u64` | — |
| `patient_id` | `Address` | — |
| `timestamp` | `u64` | — |
| `category` | `String` | — |
| `is_confidential` | `bool` | — |
| `record_hash` | `BytesN<32>` | — |
| `tags` | `Vec<String>` | — |
| `custom_fields` | `Map<String, String>` | — |
| `version` | `u32` | — |
| `history` | `Vec<RecordMetadataHistoryEntry>` | — |

#### `struct RecordMetadataHistoryEntry`

| Field | Type | Description |
|---|---|---|
| `version` | `u32` | — |
| `timestamp` | `u64` | — |
| `tags` | `Vec<String>` | — |
| `custom_fields` | `Map<String, String>` | — |

#### `enum Role`

| Variant | Value | Description |
|---|---|---|
| `Admin` | — | — |
| `Doctor` | — | — |
| `Patient` | — | — |
| `None` | — | — |

#### `enum RbacRole`

| Variant | Value | Description |
|---|---|---|
| `Admin` | 0 | — |
| `Doctor` | 1 | — |
| `Patient` | 2 | — |
| `Staff` | 3 | — |
| `Insurer` | 4 | — |
| `Researcher` | 5 | — |
| `Auditor` | 6 | — |
| `Service` | 7 | — |

#### `enum RbacError`

| Variant | Value | Description |
|---|---|---|
| `Unauthorized` | 100 | — |
| `NotInitialized` | 300 | — |
| `AlreadyInitialized` | 301 | — |

#### `enum Permission`

| Variant | Value | Description |
|---|---|---|
| `ManageUsers` | 1 | — |
| `ManageSystem` | 2 | — |
| `CreateRecord` | 10 | — |
| `ReadRecord` | 11 | — |
| `UpdateRecord` | 12 | — |
| `DeleteRecord` | 13 | — |
| `ReadConfidential` | 20 | — |
| `DelegatePermission` | 30 | — |

#### `struct PermissionGrant`

| Field | Type | Description |
|---|---|---|
| `permission` | `Permission` | — |
| `granter` | `Address` | — |
| `expires_at` | `u64` | — |
| `is_delegatable` | `bool` | — |

#### `struct UserProfile`

| Field | Type | Description |
|---|---|---|
| `role` | `Role` | — |
| `active` | `bool` | — |
| `did_reference` | `Option<String>` | — |
| `qkd_capable` | `bool` | — |

#### `enum DIDAuthLevel`

| Variant | Value | Description |
|---|---|---|
| `None` | — | — |
| `Basic` | — | — |
| `CredentialRequired` | — | — |
| `Full` | — | — |

#### `struct AccessRequest`

| Field | Type | Description |
|---|---|---|
| `requester` | `Address` | — |
| `patient` | `Address` | — |
| `record_id` | `u64` | — |
| `purpose` | `String` | — |
| `timestamp` | `u64` | — |
| `granted` | `bool` | — |

#### `struct EmergencyAccess`

| Field | Type | Description |
|---|---|---|
| `grantee` | `Address` | — |
| `patient` | `Address` | — |
| `expires_at` | `u64` | — |
| `record_scope` | `Vec<u64>` | — |
| `is_active` | `bool` | — |

#### `struct ZkPublicInputs`

| Field | Type | Description |
|---|---|---|
| `record_id` | `u64` | — |
| `record_commitment` | `BytesN<32>` | — |
| `credential_root` | `BytesN<32>` | — |
| `issuer` | `Address` | — |
| `requester_commitment` | `BytesN<32>` | — |
| `provider_commitment` | `BytesN<32>` | — |
| `claim_commitment` | `BytesN<32>` | — |
| `min_timestamp` | `u64` | — |
| `max_timestamp` | `u64` | — |
| `nullifier` | `BytesN<32>` | — |
| `pseudonym` | `BytesN<32>` | — |
| `vk_version` | `u32` | — |

#### `struct ZkAccessGrant`

| Field | Type | Description |
|---|---|---|
| `record_id` | `u64` | — |
| `requester` | `Address` | — |
| `expires_at` | `u64` | — |
| `nullifier` | `BytesN<32>` | — |
| `pseudonym` | `BytesN<32>` | — |
| `vk_version` | `u32` | — |

#### `struct ZkAuditRecord`

| Field | Type | Description |
|---|---|---|
| `record_id` | `u64` | — |
| `pseudonym` | `BytesN<32>` | — |
| `timestamp` | `u64` | — |
| `proof_verified` | `bool` | — |
| `nullifier_present` | `bool` | — |
| `nullifier` | `BytesN<32>` | — |

#### `struct MedicalRecord`

| Field | Type | Description |
|---|---|---|
| `patient_id` | `Address` | — |
| `doctor_id` | `Address` | — |
| `timestamp` | `u64` | — |
| `diagnosis` | `String` | — |
| `treatment` | `String` | — |
| `is_confidential` | `bool` | — |
| `tags` | `Vec<String>` | — |
| `category` | `String` | — |
| `treatment_type` | `String` | — |
| `data_ref` | `String` | — |
| `doctor_did` | `Option<String>` | — |

#### `struct TraditionalMedicineMetadata`

| Field | Type | Description |
|---|---|---|
| `practice_type` | `String` | — |
| `practitioner_tradition` | `String` | — |
| `remedies_used` | `String` | — |
| `cultural_context` | `String` | — |
| `language` | `String` | — |

#### `enum AIInsightType`

| Variant | Value | Description |
|---|---|---|
| `AnomalyScore` | — | — |
| `RiskScore` | — | — |

#### `struct AIInsight`

| Field | Type | Description |
|---|---|---|
| `patient` | `Address` | — |
| `record_id` | `u64` | — |
| `model_id` | `BytesN<32>` | — |
| `insight_type` | `AIInsightType` | — |
| `score_bps` | `u32` | — |
| `explanation_ref` | `String` | — |
| `explanation_summary` | `String` | — |
| `created_at` | `u64` | — |
| `model_version` | `String` | — |

#### `struct AIConfig`

| Field | Type | Description |
|---|---|---|
| `ai_coordinator` | `Address` | — |
| `dp_epsilon` | `u32` | — |
| `min_participants` | `u32` | — |

#### `struct RecoveryProposal`

| Field | Type | Description |
|---|---|---|
| `proposal_id` | `u64` | — |
| `token_contract` | `Address` | — |
| `to` | `Address` | — |
| `amount` | `i128` | — |
| `created_at` | `u64` | — |
| `executed` | `bool` | — |
| `approvals` | `Vec<Address>` | — |

#### `enum EnvelopeAlgorithm`

| Variant | Value | Description |
|---|---|---|
| `X25519` | — | — |
| `Kyber768` | — | — |
| `Kyber1024` | — | — |
| `HybridX25519Kyber768` | — | — |
| `HybridX25519Kyber1024` | — | — |
| `HybridKyberMcEliece` | — | — |
| `McEliece` | — | — |
| `Custom(u32)` | — | — |

#### `struct KeyEnvelope`

| Field | Type | Description |
|---|---|---|
| `recipient` | `Address` | — |
| `key_version` | `u32` | — |
| `algorithm` | `EnvelopeAlgorithm` | — |
| `wrapped_key` | `Bytes` | — |
| `pq_wrapped_key` | `Option<Bytes>` | — |

#### `struct EncryptedRecord`

| Field | Type | Description |
|---|---|---|
| `patient_id` | `Address` | — |
| `doctor_id` | `Address` | — |
| `timestamp` | `u64` | — |
| `is_confidential` | `bool` | — |
| `tags` | `Vec<String>` | — |
| `category` | `String` | — |
| `treatment_type` | `String` | — |
| `ciphertext_ref` | `String` | — |
| `ciphertext_hash` | `BytesN<32>` | — |
| `envelopes` | `Vec<KeyEnvelope>` | — |
| `doctor_did` | `Option<String>` | — |

#### `struct EncryptedRecordHeader`

| Field | Type | Description |
|---|---|---|
| `record_id` | `u64` | — |
| `patient_id` | `Address` | — |
| `doctor_id` | `Address` | — |
| `timestamp` | `u64` | — |
| `is_confidential` | `bool` | — |
| `tags` | `Vec<String>` | — |
| `category` | `String` | — |
| `treatment_type` | `String` | — |
| `ciphertext_ref` | `String` | — |
| `ciphertext_hash` | `BytesN<32>` | — |
| `doctor_did` | `Option<String>` | — |

#### `struct UserAccessAttribute`

| Field | Type | Description |
|---|---|---|
| `namespace` | `String` | — |
| `value` | `String` | — |
| `issued_by` | `Address` | — |
| `issued_at` | `u64` | — |
| `expires_at` | `u64` | — |
| `revoked_at` | `u64` | — |
| `epoch` | `u32` | — |
| `is_active` | `bool` | — |
| `is_verified` | `bool` | — |

#### `struct AbePolicyMetadata`

| Field | Type | Description |
|---|---|---|
| `policy_ref` | `String` | — |
| `policy_hash` | `BytesN<32>` | — |
| `access_ciphertext_ref` | `String` | — |
| `access_ciphertext_hash` | `BytesN<32>` | — |
| `required_permission` | `Permission` | — |
| `attribute_count` | `u32` | — |
| `compiled_at` | `u64` | — |
| `valid_until` | `u64` | — |
| `revocation_epoch` | `u32` | — |

#### `struct AdvancedAccessState`

| Field | Type | Description |
|---|---|---|
| `record_policies` | `Map<u64, AbePolicyMetadata>` | — |
| `user_attributes` | `Map<Address, Vec<UserAccessAttribute>>` | — |
| `attribute_epochs` | `Map<BytesN<32>, u32>` | — |

#### `struct AdvancedEncryptedRecordInput`

| Field | Type | Description |
|---|---|---|
| `ciphertext_ref` | `String` | — |
| `ciphertext_hash` | `BytesN<32>` | — |
| `envelopes` | `Vec<KeyEnvelope>` | — |
| `policy_ref` | `String` | — |
| `policy_hash` | `BytesN<32>` | — |
| `access_ciphertext_ref` | `String` | — |
| `access_ciphertext_hash` | `BytesN<32>` | — |
| `required_permission` | `Permission` | — |
| `attribute_count` | `u32` | — |
| `valid_until` | `u64` | — |
| `revocation_epoch` | `u32` | — |

#### `enum CryptoAuditAction`

| Variant | Value | Description |
|---|---|---|
| `CryptoRegistrySet` | — | — |
| `HomomorphicRegistrySet` | — | — |
| `MpcManagerSet` | — | — |
| `EncryptionRequiredSet` | — | — |
| `EncryptedRecordCreated` | — | — |
| `EnvelopeUpdated` | — | — |
| `RequirePqEnvelopesSet` | — | — |
| `CryptoConfigProposed` | — | — |
| `CryptoConfigApproved` | — | — |
| `CryptoConfigExecuted` | — | — |
| `QuantumThreatDetected` | — | — |
| `QuantumMigrationStarted` | — | — |
| `QuantumMigrationCompleted` | — | — |

#### `struct CryptoAuditEntry`

| Field | Type | Description |
|---|---|---|
| `id` | `u64` | — |
| `timestamp` | `u64` | — |
| `actor` | `Address` | — |
| `action` | `CryptoAuditAction` | — |
| `record_id` | `Option<u64>` | — |
| `details_hash` | `BytesN<32>` | — |
| `details_ref` | `Option<String>` | — |

#### `struct CryptoConfigProposal`

| Field | Type | Description |
|---|---|---|
| `proposal_id` | `u64` | — |
| `created_at` | `u64` | — |
| `executed` | `bool` | — |
| `approvals` | `Vec<Address>` | — |
| `new_crypto_registry` | `Option<Address>` | — |
| `new_homomorphic_registry` | `Option<Address>` | — |
| `new_mpc_manager` | `Option<Address>` | — |
| `encryption_required` | `Option<bool>` | — |
| `require_pq_envelopes` | `Option<bool>` | — |

#### `enum DataKey`

| Variant | Value | Description |
|---|---|---|
| `Initialized` | — | — |
| `Paused` | — | — |
| `ContractVersion` | — | — |
| `RbacContract` | — | — |
| `Users` | — | — |
| `IdentityRegistry` | — | — |
| `DidAuthLevel` | — | — |
| `UserPermissions(Address)` | — | — |
| `NextId` | — | — |
| `RecordCount` | — | — |
| `Record(u64)` | — | — |
| `RecordMeta(u64)` | — | — |
| `RecordCommitment(u64)` | — | — |
| `PatientRecords(Address)` | — | — |
| `PatientRecordCount(Address)` | — | — |
| `PatientRecord(Address, u64)` | — | — |
| `TagIndex(String)` | — | — |
| `AccessLogCount,
    AccessLog(u64),
    PatientAccessLogCount(Address),
    PatientAccessLog(Address, u64),

    
    PatientEmergencyGrants(Address),

    
    AIConfig,
    PatientRisk(Address),
    RecordAnomaly(u64),

    
    Proposal(u64),
    CryptoConfigProposal(u64),

    
    BridgeContract,
    CrossChainIdentityContract,
    CrossChainAccessContract,
    CrossChainEnabled,
    CrossChainRef(u64, ChainId),

    
    CryptoRegistry,
    HomomorphicRegistry,
    MpcManager,
    EncryptionRequired,
    RequirePqEnvelopes,

    
    EncryptedRecord(u64),
    PatientEncryptedRecords(Address),

    
    CryptoAuditCount,
    CryptoAudit(u64),

    
    AuditForensicsContract,
    
    RegulatoryCompliance,

    
    ZkVerifierContract,
    CredentialRegistryContract,
    PatientConsentContract,
    ZkEnforced,
    ZkGrantTtl,
    ZkUsedNullifier(BytesN<32>),
    ZkAccessGrant(Address, u64),
    
    RateLimitCfg(u32),        
    RateLimit(Address, u32),  
    RateLimitBypass(Address), 
    QuantumThreatLevel,       
    LastExportTime(Address),  

    
    
    TraditionalMeta(u64),
    
    PatientTraditionalRecords(Address),` | — | — |

#### `struct RecordInput`

| Field | Type | Description |
|---|---|---|
| `patient` | `Address` | — |
| `diagnosis` | `String` | — |
| `treatment` | `String` | — |
| `is_confidential` | `bool` | — |
| `tags` | `Vec<String>` | — |
| `category` | `String` | — |
| `treatment_type` | `String` | — |
| `data_ref` | `String` | — |
| `traditional_metadata` | `Option<TraditionalMedicineMetadata>` | — |

#### `struct FailureInfo`

| Field | Type | Description |
|---|---|---|
| `index` | `u32` | — |
| `error_code` | `u32` | — |

#### `struct BatchResult`

| Field | Type | Description |
|---|---|---|
| `successes` | `Vec<u64>` | — |
| `failures` | `Vec<FailureInfo>` | — |

#### `struct ListRecordsResult`

| Field | Type | Description |
|---|---|---|
| `records` | `Vec<MedicalRecord>` | — |
| `next_cursor` | `Option<u64>` | — |

#### `struct RateLimitConfig`

| Field | Type | Description |
|---|---|---|
| `doctor_max_calls` | `u32` | — |
| `patient_max_calls` | `u32` | — |
| `admin_max_calls` | `u32` | — |
| `window_secs` | `u64` | — |

#### `struct RateLimitEntry`

| Field | Type | Description |
|---|---|---|
| `count` | `u32` | — |
| `window_start` | `u64` | — |

#### `enum MedicalRecordType`

| Variant | Value | Description |
|---|---|---|
| `General` | — | — |
| `Laboratory` | — | — |
| `Prescription` | — | — |
| `Imaging` | — | — |
| `Surgical` | — | — |
| `Emergency` | — | — |

#### `struct DataQualityScore`

| Field | Type | Description |
|---|---|---|
| `overall_score` | `u32` | — |
| `completeness_score` | `u32` | — |
| `format_score` | `u32` | — |
| `consistency_score` | `u32` | — |
| `fhir_compliance_score` | `u32` | — |
| `issue_count` | `u32` | — |

#### `enum ValidationSeverity`

| Variant | Value | Description |
|---|---|---|
| `Info` | — | — |
| `Warning` | — | — |
| `ValidationErr` | — | — |
| `Critical` | — | — |

#### `struct ValidationIssue`

| Field | Type | Description |
|---|---|---|
| `severity` | `ValidationSeverity` | — |
| `field_name` | `String` | — |
| `issue_description` | `String` | — |
| `suggestion` | `String` | — |

#### `struct ValidationReport`

| Field | Type | Description |
|---|---|---|
| `record_id` | `u64` | — |
| `quality_score` | `DataQualityScore` | — |
| `issues` | `Vec<ValidationIssue>` | — |
| `is_fhir_compliant` | `bool` | — |
| `validated_at` | `u64` | — |

#### `struct FieldCompleteness`

| Field | Type | Description |
|---|---|---|
| `has_diagnosis` | `bool` | — |
| `has_treatment` | `bool` | — |
| `has_category` | `bool` | — |
| `has_treatment_type` | `bool` | — |
| `has_data_ref` | `bool` | — |
| `has_tags` | `bool` | — |
| `has_doctor_did` | `bool` | — |
| `total_fields` | `u32` | — |
| `completed_fields` | `u32` | — |

#### `enum CorrectionPriority`

| Variant | Value | Description |
|---|---|---|
| `Critical` | — | — |
| `High` | — | — |
| `Medium` | — | — |
| `Low` | — | — |

#### `enum CorrectionAction`

| Variant | Value | Description |
|---|---|---|
| `AddMissingField` | — | — |
| `FixFormat` | — | — |
| `NormalizeValue` | — | — |
| `CheckConsistency` | — | — |
| `ReviewFhirRequirement` | — | — |

#### `struct CorrectionItem`

| Field | Type | Description |
|---|---|---|
| `field_name` | `String` | — |
| `action` | `CorrectionAction` | — |
| `description` | `String` | — |
| `suggested_value` | `Option<String>` | — |
| `priority` | `CorrectionPriority` | — |

#### `struct CorrectionWorkflow`

| Field | Type | Description |
|---|---|---|
| `record_id` | `u64` | — |
| `total_issues` | `u32` | — |
| `critical_count` | `u32` | — |
| `error_count` | `u32` | — |
| `warning_count` | `u32` | — |
| `info_count` | `u32` | — |
| `corrections` | `Vec<CorrectionItem>` | — |
| `can_auto_fix` | `bool` | — |
| `workflow_created_at` | `u64` | — |

#### `struct CleanseResult`

| Field | Type | Description |
|---|---|---|
| `record` | `MedicalRecord` | — |
| `changes_made` | `Vec<String>` | — |
| `was_modified` | `bool` | — |

#### `enum ExportFormat`

| Variant | Value | Description |
|---|---|---|
| `FHIRBundle` | — | — |
| `HL7v2` | — | — |
| `CDA` | — | — |

#### `enum LogLevel`

| Variant | Value | Description |
|---|---|---|
| `Info` | — | — |
| `Warning` | — | — |
| `LogError` | — | — |

#### `struct StructuredLog`

| Field | Type | Description |
|---|---|---|
| `timestamp` | `u64` | — |
| `level` | `LogLevel` | — |
| `operation` | `String` | — |
| `actor` | `Option<Address>` | — |
| `target_id` | `Option<Address>` | — |
| `record_id` | `Option<u64>` | — |
| `message` | `String` | — |

### Error Codes

| Variant | Code | Description |
|---|---|---|
| `NotAICoordinator` | 1150 | — |
| `EmergencyAccessExpired` | 1160 | — |
| `InvalidPagination` | 1202 | — |
| `InputTooLong` | 1201 | — |
| `BatchTooLarge` | 1208 | — |
| `InvalidSignature` | 1207 | — |
| `InvalidDataRefLength` | 1250 | — |
| `InvalidDataRefCharset` | 1251 | — |
| `InvalidDiagnosisLength` | 1252 | — |
| `InvalidTreatmentLength` | 1253 | — |
| `InvalidPurposeLength` | 1254 | — |
| `InvalidTagLength` | 1255 | — |
| `InvalidModelVersionLength` | 1256 | — |
| `InvalidExplanationLength` | 1257 | — |
| `InvalidTreatmentTypeLength` | 1258 | — |
| `InvalidAddress` | 1290 | — |
| `SameAddress` | 1291 | — |
| `InvalidBatch` | 1292 | — |
| `NumberOutOfBounds` | 1293 | — |
| `InvalidCategory` | 1280 | — |
| `EmptyTreatment` | 1281 | — |
| `EmptyDiagnosis` | 1282 | — |
| `EmptyTag` | 1283 | — |
| `EmptyDataRef` | 1284 | — |
| `ProposalAlreadyExecuted` | 1320 | — |
| `TimelockNotElapsed` | 1321 | — |
| `NotEnoughApproval` | 1322 | — |
| `CryptoRegistryNotSet` | 1340 | — |
| `EncryptionRequired` | 1341 | — |
| `IdentityRegistryNotSet` | 1342 | — |
| `RecordNotFound` | 1403 | — |
| `EmergencyAccessNotFound` | 1460 | — |
| `DIDNotFound` | 1470 | — |
| `DIDNotActive` | 1471 | — |
| `RecordAlreadySynced` | 1480 | — |
| `StorageFull` | 1502 | — |
| `InvalidCredential` | 1640 | — |
| `MissingRequiredCredential` | 1641 | — |
| `CredentialExpired` | 1605 | — |
| `CredentialRevoked` | 1606 | — |
| `CrossChainAccessDenied` | 1700 | — |
| `CrossChainTimeout` | 1702 | — |
| `InvalidChain` | 1703 | — |
| `CrossChainNotEnabled` | 1710 | — |
| `CrossChainContractsNotSet` | 1711 | — |
| `AIConfigNotSet` | 1830 | — |
| `InvalidAIScore` | 1831 | — |
| `InvalidScore` | 1832 | — |
| `InvalidDPEpsilon` | 1833 | — |
| `InvalidParticipantCount` | 1834 | — |

### Examples

#### `test_add_and_get_record`

```rust
let env = Env::default();
    env.mock_all_auths();

    let (client, admin) = create_contract(&env);
    let doctor = Address::generate(&env);
    let patient = Address::generate(&env);
    let diagnosis = String::from_str(&env, "Common cold");
    let treatment = String::from_str(&env, "Rest and fluids");
    let is_confidential = false;
```

#### `test_get_record_denied_after_consent_expiry`

```rust
let env = Env::default();
    env.mock_all_auths();

    let (client, admin) = create_contract(&env);
    let consent_contract_id = env.register_contract(None, PatientConsentManagement);
    let consent_client = PatientConsentManagementClient::new(&env, &consent_contract_id);
    consent_client.initialize(&admin);
    client.set_patient_consent_contract(&admin, &consent_contract_id);
```

#### `test_empty_data_ref`

```rust
let env = Env::default();
    env.mock_all_auths();
    let (client, admin) = create_contract(&env);
    let doctor = Address::generate(&env);
    let patient = Address::generate(&env);

    client.manage_user(&admin, &doctor, &Role::Doctor);
    client.manage_user(&admin, &patient, &Role::Patient);
```

---

## medication_management

### Functions

| Function | Parameters | Returns | Description |
|---|---|---|---|
| `initialize` | `env: Env, admin: Address, pharmacist: Address, fda_oracle: Address, medical_records_contract: Address, healthcare_payment_contract: Address` | `Result<(), Error>` | — |
| `upsert_fda_medication` | `env: Env, operator: Address, medication: MedicationDefinition` | `Result<(), Error>` | — |
| `sync_fda_catalog` | `env: Env, operator: Address, medications: Vec<MedicationDefinition>` | `Result<u32, Error>` | — |
| `create_schedule` | `env: Env, patient: Address, provider: Address, request: ScheduleRequest` | `Result<u64, Error>` | — |
| `update_schedule_status` | `env: Env, schedule_id: u64, actor: Address, status: ScheduleStatus` | `Result<(), Error>` | — |
| `register_interaction` | `env: Env, operator: Address, interaction: DrugInteraction` | `Result<(), Error>` | — |
| `check_interactions` | `env: Env, medication_a: String, medication_b: String` | `Option<DrugInteraction>` | Check if two medications have a known interaction. Returns the interaction details if one exists, or None if no interaction is known. |
| `update_interaction` | `env: Env, operator: Address, interaction: DrugInteraction` | `Result<(), Error>` | Update an existing interaction record. Only admin, pharmacist, or fda_oracle may call this. |
| `resolve_interaction` | `env: Env, caller: Address, schedule_id: u64, alert_index: u32` | `Result<(), Error>` | Resolve (remove) an interaction alert for a given schedule and alert index. Only the patient, provider, or admin may call this. |
| `record_dose` | `env: Env, patient: Address, schedule_id: u64, scheduled_for: u64, status: AdherenceEventStatus, notes: String` | `Result<(), Error>` | — |
| `process_refill` | `env: Env, actor: Address, schedule_id: u64` | `Result<RefillReminder, Error>` | — |
| `trigger_auto_refill` | `env: Env, actor: Address, schedule_id: u64` | `Result<RefillReminder, Error>` | — |
| `get_schedule` | `env: Env, schedule_id: u64` | `Result<MedicationSchedule, Error>` | — |
| `get_medication` | `env: Env, medication_code: String` | `Result<MedicationDefinition, Error>` | — |
| `get_refill_status` | `env: Env, schedule_id: u64` | `Result<RefillReminder, Error>` | — |
| `get_interaction_alerts` | `env: Env, schedule_id: u64` | `Result<Vec<InteractionAlert>, Error>` | — |
| `get_patient_schedules` | `env: Env, patient: Address` | `Vec<u64>` | — |
| `generate_adherence_report` | `env: Env, schedule_id: u64` | `Result<AdherenceReport, Error>` | — |
| `get_catalog_size` | `env: Env` | `u64` | — |

### Types

#### `enum Error`

| Variant | Value | Description |
|---|---|---|
| `AlreadyInitialized` | 1 | — |
| `NotInitialized` | 2 | — |
| `Unauthorized` | 3 | — |
| `MedicationNotFound` | 4 | — |
| `ScheduleNotFound` | 5 | — |
| `InvalidData` | 6 | — |
| `RefillNotFound` | 7 | — |
| `InteractionAlreadyExists` | 8 | — |
| `DuplicateMedication` | 9 | — |
| `DoseAlreadyRecorded` | 10 | — |
| `AutoRefillDisabled` | 11 | — |

#### `enum MedicationSource`

| Variant | Value | Description |
|---|---|---|
| `Fda` | — | — |
| `ManualClinicalEntry` | — | — |
| `FhirMedicationStatement` | — | — |
| `TelemedicinePrescription` | — | — |

#### `enum Severity`

| Variant | Value | Description |
|---|---|---|
| `Low` | — | — |
| `Moderate` | — | — |
| `High` | — | — |
| `Contraindicated` | — | — |

#### `enum ScheduleStatus`

| Variant | Value | Description |
|---|---|---|
| `Active` | — | — |
| `Paused` | — | — |
| `Completed` | — | — |
| `Cancelled` | — | — |

#### `enum AdherenceEventStatus`

| Variant | Value | Description |
|---|---|---|
| `Taken` | — | — |
| `Missed` | — | — |
| `Skipped` | — | — |

#### `enum RefillStatus`

| Variant | Value | Description |
|---|---|---|
| `Monitoring` | — | — |
| `ReminderDue` | — | — |
| `Requested` | — | — |
| `Processing` | — | — |
| `Fulfilled` | — | — |

#### `enum DosingSchedule`

| Variant | Value | Description |
|---|---|---|
| `OnceDaily` | — | — |
| `TwiceDaily` | — | — |
| `ThreeTimesDaily` | — | — |
| `EveryNHours(u32)` | — | — |
| `EveryNDays(u32)` | — | — |
| `Weekly` | — | — |
| `SpecificTimes(Vec<u32>)` | — | — |

#### `struct Config`

| Field | Type | Description |
|---|---|---|
| `admin` | `Address` | — |
| `pharmacist` | `Address` | — |
| `fda_oracle` | `Address` | — |
| `medical_records_contract` | `Address` | — |
| `healthcare_payment_contract` | `Address` | — |

#### `struct MedicationDefinition`

| Field | Type | Description |
|---|---|---|
| `code` | `String` | — |
| `ndc_code` | `String` | — |
| `name` | `String` | — |
| `generic_name` | `String` | — |
| `manufacturer` | `String` | — |
| `dosage_form` | `String` | — |
| `strength` | `String` | — |
| `controlled_substance` | `bool` | — |
| `source` | `MedicationSource` | — |
| `last_fda_sync` | `u64` | — |

#### `struct MedicationSchedule`

| Field | Type | Description |
|---|---|---|
| `id` | `u64` | — |
| `patient` | `Address` | — |
| `provider` | `Address` | — |
| `medication_code` | `String` | — |
| `dosage_amount` | `String` | — |
| `schedule` | `DosingSchedule` | — |
| `start_time` | `u64` | — |
| `end_time` | `Option<u64>` | — |
| `instructions` | `String` | — |
| `status` | `ScheduleStatus` | — |
| `linked_record_id` | `Option<u64>` | — |
| `linked_claim_id` | `Option<u64>` | — |
| `prescription_ref` | `Option<String>` | — |
| `refill` | `RefillPolicy` | — |
| `adherence_baseline_bps` | `u32` | — |
| `created_at` | `u64` | — |
| `updated_at` | `u64` | — |

#### `struct ScheduleLinks`

| Field | Type | Description |
|---|---|---|
| `linked_record_id` | `Option<u64>` | — |
| `linked_claim_id` | `Option<u64>` | — |
| `prescription_ref` | `Option<String>` | — |

#### `struct ScheduleRequest`

| Field | Type | Description |
|---|---|---|
| `medication_code` | `String` | — |
| `dosage_amount` | `String` | — |
| `schedule` | `DosingSchedule` | — |
| `start_time` | `u64` | — |
| `end_time` | `Option<u64>` | — |
| `instructions` | `String` | — |
| `links` | `ScheduleLinks` | — |
| `refill` | `RefillPolicy` | — |
| `adherence_baseline_bps` | `u32` | — |

#### `struct RefillPolicy`

| Field | Type | Description |
|---|---|---|
| `enabled` | `bool` | — |
| `auto_refill` | `bool` | — |
| `total_authorized_refills` | `u32` | — |
| `refills_used` | `u32` | — |
| `reminder_window_days` | `u32` | — |
| `doses_remaining` | `u32` | — |
| `low_supply_threshold` | `u32` | — |
| `last_refill_at` | `u64` | — |

#### `struct RefillReminder`

| Field | Type | Description |
|---|---|---|
| `schedule_id` | `u64` | — |
| `patient` | `Address` | — |
| `status` | `RefillStatus` | — |
| `reminder_due_at` | `u64` | — |
| `last_notified_at` | `Option<u64>` | — |
| `next_refill_eta` | `Option<u64>` | — |
| `auto_refill_triggered_at` | `Option<u64>` | — |

#### `struct DrugInteraction`

| Field | Type | Description |
|---|---|---|
| `medication_a` | `String` | — |
| `medication_b` | `String` | — |
| `severity` | `Severity` | — |
| `advisory` | `String` | — |
| `clinical_guidance` | `String` | — |
| `source_ref` | `String` | — |
| `updated_at` | `u64` | — |

#### `struct InteractionAlert`

| Field | Type | Description |
|---|---|---|
| `schedule_id` | `u64` | — |
| `interacting_schedule_id` | `u64` | — |
| `patient` | `Address` | — |
| `medication_a` | `String` | — |
| `medication_b` | `String` | — |
| `severity` | `Severity` | — |
| `advisory` | `String` | — |
| `created_at` | `u64` | — |

#### `struct DoseEvent`

| Field | Type | Description |
|---|---|---|
| `schedule_id` | `u64` | — |
| `patient` | `Address` | — |
| `scheduled_for` | `u64` | — |
| `recorded_at` | `u64` | — |
| `status` | `AdherenceEventStatus` | — |
| `notes` | `String` | — |

#### `struct AdherenceReport`

| Field | Type | Description |
|---|---|---|
| `schedule_id` | `u64` | — |
| `patient` | `Address` | — |
| `expected_doses` | `u32` | — |
| `recorded_doses` | `u32` | — |
| `taken_doses` | `u32` | — |
| `missed_doses` | `u32` | — |
| `skipped_doses` | `u32` | — |
| `adherence_bps` | `u32` | — |
| `baseline_bps` | `u32` | — |
| `improvement_bps` | `i32` | — |
| `target_improvement_met` | `bool` | — |
| `generated_at` | `u64` | — |

#### `enum DataKey`

| Variant | Value | Description |
|---|---|---|
| `Config` | — | — |
| `MedicationCount` | — | — |
| `ScheduleCount` | — | — |
| `Medication(String)` | — | — |
| `Schedule(u64)` | — | — |
| `PatientSchedules(Address)` | — | — |
| `Interaction(String, String)` | — | — |
| `ScheduleAlerts(u64)` | — | — |
| `RefillReminder(u64)` | — | — |
| `DoseEvents(u64)` | — | — |

---

## mental_health_support

### Functions

| Function | Parameters | Returns | Description |
|---|---|---|---|
| `initialize` | `env: Env, admin: Address` | `Result<(), MentalHealthError>` | — |
| `set_integration_contracts` | `env: Env, caller: Address, telemedicine: Address, notification: Address` | `Result<(), MentalHealthError>` | — |
| `set_emergency_routing_commitment` | `env: Env, caller: Address, meta_hash: BytesN<32>` | `Result<(), MentalHealthError>` | — |
| `get_telemedicine_contract` | `env: Env` | `Result<Option<Address>, MentalHealthError>` | — |
| `get_notification_contract` | `env: Env` | `Result<Option<Address>, MentalHealthError>` | — |
| `pause` | `env: Env, caller: Address` | `Result<(), MentalHealthError>` | — |
| `unpause` | `env: Env, caller: Address` | `Result<(), MentalHealthError>` | — |
| `enroll` | `env: Env, patient: Address` | `Result<(), MentalHealthError>` | — |
| `is_enrolled` | `env: Env, patient: Address` | `Result<bool, MentalHealthError>` | — |
| `log_mood` | `env: Env, patient: Address, mood_score: u32, symptom_blob_hash: BytesN<32>` | `Result<u64, MentalHealthError>` | — |
| `get_mood` | `env: Env, id: u64` | `Result<MoodEntry, MentalHealthError>` | — |
| `book_teletherapy` | `env: Env, patient: Address, modality: TherapyModality, telemedicine_session_id: BytesN<32>, scheduled_at: u64, notes: String` | `Result<u64, MentalHealthError>` | — |
| `get_booking` | `env: Env, id: u64` | `Result<TeletherapyBooking, MentalHealthError>` | — |
| `report_crisis` | `env: Env, patient: Address, severity: CrisisSeverity, detail_hash: BytesN<32>` | `Result<u64, MentalHealthError>` | Records crisis state and enqueues for triage. Use [`Self::get_notification_contract`] from a relayer to call `notification_system` (with this contract as authorised sender). |
| `get_crisis` | `env: Env, id: u64` | `Result<CrisisIntervention, MentalHealthError>` | — |
| `create_peer_community` | `env: Env, admin: Address, name: String` | `Result<u64, MentalHealthError>` | — |
| `join_peer_community` | `env: Env, patient: Address, community_id: u64` | `Result<(), MentalHealthError>` | — |
| `list_community_members` | `env: Env, community_id: u64` | `Result<Vec<Address>, MentalHealthError>` | — |
| `open_crisis_queue` | `env: Env` | `Result<Vec<u64>, MentalHealthError>` | — |

### Types

#### `enum MentalHealthError`

| Variant | Value | Description |
|---|---|---|
| `AlreadyInitialized` | 1 | — |
| `NotInitialized` | 2 | — |
| `NotAdmin` | 3 | — |
| `Paused` | 4 | — |
| `NotEnrolled` | 5 | — |
| `CommunityNotFound` | 6 | — |
| `AlreadyMember` | 7 | — |
| `InvalidInput` | 8 | — |

#### `enum CrisisSeverity`

| Variant | Value | Description |
|---|---|---|
| `Elevated` | — | — |
| `High` | — | — |
| `Imminent` | — | — |

#### `enum TherapyModality`

| Variant | Value | Description |
|---|---|---|
| `Cbt` | — | — |
| `Dbt` | — | — |
| `Psychodynamic` | — | — |
| `Group` | — | — |
| `Family` | — | — |
| `MedicationManagement` | — | — |
| `Other` | — | — |

#### `struct MoodEntry`

| Field | Type | Description |
|---|---|---|
| `id` | `u64` | — |
| `patient` | `Address` | — |
| `recorded_at` | `u64` | — |
| `mood_score` | `u32` | — |
| `symptom_blob_hash` | `BytesN<32>` | — |

#### `struct TeletherapyBooking`

| Field | Type | Description |
|---|---|---|
| `id` | `u64` | — |
| `patient` | `Address` | — |
| `modality` | `TherapyModality` | — |
| `telemedicine_session_id` | `BytesN<32>` | — |
| `scheduled_at` | `u64` | — |
| `notes` | `String` | — |

#### `struct CrisisIntervention`

| Field | Type | Description |
|---|---|---|
| `id` | `u64` | — |
| `patient` | `Address` | — |
| `severity` | `CrisisSeverity` | — |
| `detail_hash` | `BytesN<32>` | — |
| `created_at` | `u64` | — |
| `notification_id` | `Option<u64>` | — |

#### `struct PeerCommunity`

| Field | Type | Description |
|---|---|---|
| `id` | `u64` | — |
| `name` | `String` | — |
| `created_at` | `u64` | — |

#### `enum DataKey`

| Variant | Value | Description |
|---|---|---|
| `Initialized` | — | — |
| `Admin` | — | — |
| `Paused` | — | — |
| `Telemedicine` | — | — |
| `Notification` | — | — |
| `EmergencyMetaHash` | — | — |
| `NextMoodId` | — | — |
| `Mood(u64)` | — | — |
| `NextBookingId` | — | — |
| `Booking(u64)` | — | — |
| `NextCrisisId` | — | — |
| `Crisis(u64)` | — | — |
| `NextCommunityId` | — | — |
| `Community(u64)` | — | — |
| `CommunityMembers(u64)` | — | — |
| `Enrolled(Address)` | — | — |
| `PatientMoodIds(Address)` | — | — |
| `PatientBookingIds(Address)` | — | — |
| `PatientCrisisIds(Address)` | — | — |
| `PatientCommunities(Address)` | — | — |
| `OpenCrisisQueue` | — | — |

### Examples

#### `enroll_mood_booking_community`

```rust
let env = Env::default();
    let (client, admin) = setup(&env);
    let patient = Address::generate(&env);
    client.try_enroll(&patient).unwrap().unwrap();
    assert!(client.try_is_enrolled(&patient).unwrap().unwrap());

    let mid = client
        .try_log_mood(&patient, &6u32, &BytesN::from_array(&env, &[1u8; 32]))
        .unwrap()
```

#### `crisis_is_queued`

```rust
let env = Env::default();
    let (mh, admin) = setup(&env);

    let tele = Address::generate(&env);
    let notif = Address::generate(&env);
    mh.try_set_integration_contracts(&admin, &tele, &notif)
        .unwrap()
        .unwrap();
```

---

## meta_tx_forwarder

### Functions

| Function | Parameters | Returns | Description |
|---|---|---|---|
| `initialize` | `env: Env, owner: Address, fee_collector: Address, min_relayer_stake: i128` | `Result<(), Error>` | Initialize the forwarder contract.  # Arguments * `owner` - Contract owner address * `fee_collector` - Address to receive relay fees (informational) * `min_relayer_stake` - Minimum stake required for relayers (informational) |
| `register_user_pub_key` | `env: Env, user: Address, pub_key: BytesN<32>` | `Result<(), Error>` | Register an Ed25519 public key (32 bytes) for a user.  The public key is required before the user can sign `ForwardRequest`s. One-time registration — re-registering overwrites the previous key. |
| `get_user_pub_key` | `env: Env, user: Address` | `Option<BytesN<32>>` | Returns the registered Ed25519 public key for `user`, if any. |
| `execute` | `env: Env, relayer: Address, request: ForwardRequest, signature: BytesN<64>` | `Result<Bytes, Error>` | Execute a meta-transaction on behalf of a user. |
| `execute_batch` | `env: Env, relayer: Address, requests: Vec<ForwardRequest>, signatures: Vec<BytesN<64>>` | `Result<Vec<Bytes>, Error>` | Execute multiple meta-transactions in a batch.  All requests must be valid individually; on first failure, the already-completed requests have incremented their nonces and committed state, and the rest are not executed. |
| `register_relayer` | `env: Env, owner: Address, relayer: Address, fee_percentage: u32` | `Result<(), Error>` | Register a new relayer (owner-only). |
| `deactivate_relayer` | `env: Env, owner: Address, relayer: Address` | `Result<(), Error>` | Deactivate a relayer (owner-only). |
| `get_nonce` | `env: Env, user: Address` | `u64` | Get the current nonce for a user. |
| `is_relayer` | `env: Env, relayer: Address` | `bool` | Check if an address is an active relayer. |
| `get_relayer_config` | `env: Env, relayer: Address` | `Option<RelayerConfig>` | Get relayer configuration. |
| `get_trusted_forwarder` | `env: Env` | `Address` | Get the trusted forwarder address (this contract). |
| `domain_separator` | `env: Env` | `Bytes` | Returns the canonical domain separator string used in the signed message. Exposed for off-chain clients that need to reproduce the exact prefix when constructing signatures. |

### Types

#### `enum Error`

| Variant | Value | Description |
|---|---|---|
| `InvalidSignature` | 1 | — |
| `InvalidNonce` | 2 | — |
| `RequestExpired` | 3 | — |
| `ExecutionFailed` | 4 | — |
| `Unauthorized` | 5 | — |
| `AlreadyInitialized` | 6 | — |
| `OwnerNotSet` | 7 | — |
| `BatchLengthMismatch` | 8 | — |
| `PubKeyNotRegistered` | 9 | — |
| `InvalidFeePercentage` | 10 | — |

#### `struct ForwardRequest`

| Field | Type | Description |
|---|---|---|
| `from` | `Address` | — |
| `to` | `Address` | — |
| `value` | `i128` | — |
| `gas` | `u32` | — |
| `nonce` | `u64` | — |
| `deadline` | `u64` | — |
| `target_fn` | `Symbol` | — |
| `target_args` | `Vec<Val>` | — |

#### `struct RelayerConfig`

| Field | Type | Description |
|---|---|---|
| `address` | `Address` | — |
| `is_active` | `bool` | — |
| `fee_percentage` | `u32` | — |

#### `enum DataKey`

| Variant | Value | Description |
|---|---|---|
| `Owner` | — | — |
| `Nonce(Address)` | — | — |
| `Relayer(Address)` | — | — |
| `TrustedForwarder` | — | — |
| `FeeCollector` | — | — |
| `MinRelayerStake` | — | — |
| `UserPubKey(Address)` | — | — |

#### `struct ForwardOutcome`

| Field | Type | Description |
|---|---|---|
| `result` | `Bytes` | — |
| `new_nonce` | `u64` | — |
| `executed_at` | `u64` | — |

### Examples

#### `test_initialize_success`

```rust
let env = Env::default();
    env.mock_all_auths();
    let (_fid, fwd) = install_forwarder(&env);
    let owner = Address::generate(&env);
    let fee_collector = Address::generate(&env);
    initialize(&env, &fwd, &owner, &fee_collector);
    assert_eq!(fwd.get_trusted_forwarder(), fwd.address);
```

#### `test_initialize_twice_fails`

```rust
let env = Env::default();
    env.mock_all_auths();
    let (_fid, fwd) = install_forwarder(&env);
    let owner = Address::generate(&env);
    let fee_collector = Address::generate(&env);
    initialize(&env, &fwd, &owner, &fee_collector);
    let res = fwd.try_initialize(&owner, &fee_collector, &1_000_000i128);
    assert_eq!(res, Err(Ok(Error::AlreadyInitialized)));
```

#### `test_register_user_pub_key_persists`

```rust
let env = Env::default();
    env.mock_all_auths();
    let (_fid, fwd) = install_forwarder(&env);
    let user = Address::generate(&env);
    let pk = BytesN::<32>::from_array(&env, &[7u8; 32]);
    fwd.register_user_pub_key(&user, &pk);
    let stored = fwd.get_user_pub_key(&user).expect("key stored");
    assert_eq!(stored, pk);
```

---

## mfa

### Functions

| Function | Parameters | Returns | Description |
|---|---|---|---|
| `initialize` | `env: Env, admin: Address, config: MFAConfig` | `()` | Initialize with global MFA configuration |
| `add_factor` | `env: Env, user: Address, factor: FactorType, provider: Option<Address>, metadata: String` | `u64` | Add a new authentication factor for the caller |
| `start_session` | `env: Env, user: Address, required: Vec<FactorType>` | `u64` | Initiate an authentication session requiring specific factors |
| `verify_mfa_factor` | `env: Env, user: Address, factor: FactorType, proof: Bytes` | `bool` | Verify a specific factor for an existing session |
| `is_authenticated` | `env: Env, user: Address` | `bool` | Check if the user has a valid verified MFA session |
| `initiate_recovery` | `env: Env, user: Address, _secret_hash: BytesN<32>` | `()` | Recovery mechanism for lost factors |
| `emergency_override` | `env: Env, admin: Address, target_user: Address` | `bool` | Emergency override using admin signatures (multi-sig simulation) |

### Examples

#### `test_mfa_lifecycle`

```rust
let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let contract_id = env.register_contract(None, MultiFactorAuth);
    let client = MultiFactorAuthClient::new(&env, &contract_id);

    let config = MFAConfig {
        session_ttl: 3600,
```

---

## mpc_manager

### Functions

| Function | Parameters | Returns | Description |
|---|---|---|---|
| `initialize` | `env: Env, admin: Address` | `Result<(), Error>` | — |
| `start_session` | `env: Env, initiator: Address, session_id: BytesN<32>, participants: Vec<Address>, threshold: u32, purpose: String, ttl_secs: u64, computation_type: ComputationType` | `Result<(), Error>` | — |
| `commit_share` | `env: Env, participant: Address, session_id: BytesN<32>, commitment_hash: BytesN<32>` | `Result<(), Error>` | — |
| `reveal_share` | `env: Env, participant: Address, session_id: BytesN<32>, share_ref: String, share_hash: BytesN<32>` | `Result<(), Error>` | — |
| `finalize_session` | `env: Env, initiator: Address, session_id: BytesN<32>, result_ref: String, result_hash: BytesN<32>, proof_ref: String, proof_hash: BytesN<32>` | `Result<(), Error>` | — |
| `get_session` | `env: Env, session_id: BytesN<32>` | `Result<Option<MPCSession>, Error>` | — |
| `get_commitment` | `env: Env, session_id: BytesN<32>, participant: Address` | `Result<Option<BytesN<32>>, Error>` | — |
| `get_reveal` | `env: Env, session_id: BytesN<32>, participant: Address` | `Result<Option<ShareReveal>, Error>` | — |
| `create_secret_shares` | `env: Env, participant: Address, session_id: BytesN<32>, secret: Bytes, num_shares: u32, threshold: u32` | `Result<Vec<SecretShare>, Error>` | Create Shamir's Secret Sharing shares for medical record encryption keys |
| `submit_computation_proof` | `env: Env, participant: Address, session_id: BytesN<32>, proof: ComputationProof` | `Result<(), Error>` | Submit a computation proof for verification |
| `perform_statistical_analysis` | `env: Env, participant: Address, session_id: BytesN<32>, _analysis_type: String, encrypted_data: Bytes` | `Result<BytesN<32>, Error>` | Perform privacy-preserving statistical analysis |
| `train_secure_ml_model` | `env: Env, participant: Address, session_id: BytesN<32>, model_params: Bytes, training_data: Bytes` | `Result<BytesN<32>, Error>` | Train machine learning model on encrypted data |
| `get_audit_trail` | `env: Env, session_id: BytesN<32>` | `Result<Vec<AuditEntry>, Error>` | Get audit trail for a session |
| `get_gas_stats` | `env: Env, session_id: BytesN<32>` | `Result<u64, Error>` | Get gas usage statistics for a session |

### Types

#### `struct SecretShare`

| Field | Type | Description |
|---|---|---|
| `share_id` | `u32` | — |
| `share_value` | `Bytes` | — |
| `commitment` | `BytesN<32>` | — |
| `created_at` | `u64` | — |

#### `struct ComputationProof`

| Field | Type | Description |
|---|---|---|
| `computation_type` | `String` | — |
| `input_commitment` | `BytesN<32>` | — |
| `output_hash` | `BytesN<32>` | — |
| `proof_data` | `Bytes` | — |
| `verification_key_hash` | `BytesN<32>` | — |
| `gas_used` | `u64` | — |
| `created_at` | `u64` | — |

#### `enum ComputationType`

| Variant | Value | Description |
|---|---|---|
| `StatisticalAnalysis` | — | — |
| `SecureAggregation` | — | — |
| `PrivacyPreservingML` | — | — |
| `DiagnosticAnalysis` | — | — |
| `DrugDiscovery` | — | — |

#### `struct AuditEntry`

| Field | Type | Description |
|---|---|---|
| `participant` | `Address` | — |
| `operation` | `String` | — |
| `session_id` | `BytesN<32>` | — |
| `timestamp` | `u64` | — |
| `gas_used` | `u64` | — |
| `metadata` | `Bytes` | — |

#### `enum SessionStatus`

| Variant | Value | Description |
|---|---|---|
| `Initiated` | — | — |
| `CommitPhase` | — | — |
| `RevealPhase` | — | — |
| `Finalized` | — | — |
| `Aborted` | — | — |
| `Expired` | — | — |

#### `struct ShareReveal`

| Field | Type | Description |
|---|---|---|
| `share_ref` | `String` | — |
| `share_hash` | `BytesN<32>` | — |
| `revealed_at` | `u64` | — |

#### `struct MPCSession`

| Field | Type | Description |
|---|---|---|
| `session_id` | `BytesN<32>` | — |
| `initiator` | `Address` | — |
| `participants` | `Vec<Address>` | — |
| `threshold` | `u32` | — |
| `purpose` | `String` | — |
| `created_at` | `u64` | — |
| `expires_at` | `u64` | — |
| `status` | `SessionStatus` | — |
| `commits` | `u32` | — |
| `reveals` | `u32` | — |
| `result_ref` | `String` | — |
| `result_hash` | `BytesN<32>` | — |
| `proof_ref` | `String` | — |
| `proof_hash` | `BytesN<32>` | — |
| `computation_type` | `ComputationType` | — |
| `total_gas_used` | `u64` | — |
| `audit_entries` | `u32` | — |

#### `enum DataKey`

| Variant | Value | Description |
|---|---|---|
| `Initialized` | — | — |
| `Admin` | — | — |
| `Session(BytesN<32>)` | — | — |
| `Commit(BytesN<32>, Address)` | — | — |
| `Reveal(BytesN<32>, Address)` | — | — |
| `SecretShare(BytesN<32>, Address, u32)` | — | — |
| `ComputationProof(BytesN<32>)` | — | — |
| `AuditEntry(u32)` | — | — |
| `AuditCounter` | — | — |
| `GasTracker(BytesN<32>, Address)` | — | — |

#### `enum Error`

| Variant | Value | Description |
|---|---|---|
| `AlreadyInitialized` | 1 | — |
| `NotInitialized` | 2 | — |
| `NotAuthorized` | 3 | — |
| `InvalidInput` | 4 | — |
| `SessionNotFound` | 5 | — |
| `SessionExpired` | 6 | — |
| `InvalidState` | 7 | — |
| `DuplicateCommit` | 8 | — |
| `DuplicateReveal` | 9 | — |
| `ThresholdNotMet` | 10 | — |
| `InvalidShare` | 11 | — |
| `ComputationFailed` | 12 | — |
| `ProofVerificationFailed` | 13 | — |
| `GasLimitExceeded` | 14 | — |
| `InsufficientParticipants` | 15 | — |

### Examples

#### `mpc_session_lifecycle`

```rust
let env = Env::default();
    env.mock_all_auths();

    let (client, _id) = setup(&env);
    let admin = Address::generate(&env);
    client.initialize(&admin);

    let initiator = Address::generate(&env);
    let p1 = Address::generate(&env);
```

#### `test_shamir_secret_sharing`

```rust
let env = Env::default();
    env.mock_all_auths();

    let (client, _id) = setup(&env);
    let admin = Address::generate(&env);
    client.initialize(&admin);

    let initiator = Address::generate(&env);
    let p1 = Address::generate(&env);
```

#### `test_statistical_analysis`

```rust
let env = Env::default();
    env.mock_all_auths();

    let (client, _id) = setup(&env);
    let admin = Address::generate(&env);
    client.initialize(&admin);

    let initiator = Address::generate(&env);
    let p1 = Address::generate(&env);
```

---

## multi_region_orchestrator

### Functions

| Function | Parameters | Returns | Description |
|---|---|---|---|
| `initialize` | `env: Env, admin: Address` | `Result<(), Error>` | — |
| `set_paused` | `env: Env, caller: Address, paused: bool` | `Result<(), Error>` | — |
| `assign_role` | `env: Env, caller: Address, user: Address, role_mask: u32` | `Result<(), Error>` | — |
| `register_region` | `env: Env, caller: Address, region: GeoRegion, node_id: u32, endpoint_hash: u64, is_primary: bool` | `Result<u32, Error>` | — |
| `list_regions` | `env: Env` | `Vec<RegionNode>` | — |
| `get_region_status` | `env: Env, region_id: u32` | `Option<RegionStatus>` | — |
| `update_region_status` | `env: Env, caller: Address, region_id: u32, status: RegionStatus` | `Result<(), Error>` | — |
| `trigger_failover` | `env: Env, caller: Address, source_region_id: u32, target_region_id: u32, reason: Symbol` | `Result<u64, Error>` | — |
| `get_failover_events` | `env: Env` | `Vec<FailoverEvent>` | — |
| `sync_data` | `env: Env, caller: Address, _source_region_id: u32, target_region_ids: Vec<u32>, data_hash: u64` | `Result<u64, Error>` | — |
| `get_sync_operations` | `env: Env` | `Vec<SyncOperation>` | — |
| `check_health` | `env: Env, caller: Address` | `Result<bool, Error>` | — |
| `record_uptime_metric` | `env: Env, caller: Address, start_time: u64, end_time: u64, uptime_basis_points: u32, outages: u32, total_outage_ms: u64` | `Result<(), Error>` | — |
| `get_uptime_metrics` | `env: Env` | `Vec<UptimeMetric>` | — |
| `get_current_uptime` | `env: Env` | `u32` | — |
| `set_policy` | `env: Env, caller: Address, policy: DRPolicy` | `Result<(), Error>` | — |
| `get_policy` | `env: Env` | `DRPolicy` | — |

### Types

#### `enum GeoRegion`

| Variant | Value | Description |
|---|---|---|
| `UsEast` | 0 | — |
| `UsWest` | 1 | — |
| `EuCentral` | 2 | — |
| `EuWest` | 3 | — |
| `ApSouth` | 4 | — |
| `ApNorth` | 5 | — |
| `SaEast` | 6 | — |
| `AfSouth` | 7 | — |
| `Custom` | 8 | — |

#### `enum RegionStatus`

| Variant | Value | Description |
|---|---|---|
| `Active` | 0 | — |
| `Degraded` | 1 | — |
| `Unavailable` | 2 | — |
| `RecoveryInProgress` | 3 | — |

#### `struct RegionNode`

| Field | Type | Description |
|---|---|---|
| `region` | `GeoRegion` | — |
| `node_id` | `u32` | — |
| `status` | `RegionStatus` | — |
| `endpoint_hash` | `u64` | — |
| `last_heartbeat` | `u64` | — |
| `replica_count` | `u32` | — |
| `is_primary` | `bool` | — |
| `failure_count` | `u32` | — |

#### `struct FailoverEvent`

| Field | Type | Description |
|---|---|---|
| `event_id` | `u64` | — |
| `triggered_at` | `u64` | — |
| `source_region` | `GeoRegion` | — |
| `target_region` | `GeoRegion` | — |
| `reason` | `Symbol` | — |
| `rto_ms` | `u64` | — |
| `success` | `bool` | — |

#### `struct UptimeMetric`

| Field | Type | Description |
|---|---|---|
| `start_time` | `u64` | — |
| `end_time` | `u64` | — |
| `uptime_basis_points` | `u32` | — |
| `outages` | `u32` | — |
| `total_outage_ms` | `u64` | — |

#### `struct DRPolicy`

| Field | Type | Description |
|---|---|---|
| `min_replicas_per_region` | `u32` | — |
| `max_regions` | `u32` | — |
| `failover_timeout_ms` | `u64` | — |
| `sync_interval_ms` | `u64` | — |
| `health_check_interval_ms` | `u64` | — |
| `auto_failover_enabled` | `bool` | — |
| `rto_target_ms` | `u64` | — |

#### `struct SyncOperation`

| Field | Type | Description |
|---|---|---|
| `sync_id` | `u64` | — |
| `source_region` | `GeoRegion` | — |
| `target_regions` | `Vec<GeoRegion>` | — |
| `data_hash` | `u64` | — |
| `started_at` | `u64` | — |
| `completed_at` | `u64` | — |
| `success` | `bool` | — |

#### `enum Error`

| Variant | Value | Description |
|---|---|---|
| `AlreadyInitialized` | 1 | — |
| `NotInitialized` | 2 | — |
| `NotAuthorized` | 3 | — |
| `InvalidInput` | 4 | — |
| `MaxRegionsExceeded` | 5 | — |
| `AllRegionsUnavailable` | 6 | — |
| `FailoverFailed` | 7 | — |
| `SyncFailed` | 8 | — |
| `RtoExceeded` | 9 | — |
| `InsufficientReplicas` | 10 | — |

---

## notification_system

### Functions

| Function | Parameters | Returns | Description |
|---|---|---|---|
| `initialize` | `env: Env, admin: Address` | `Result<(), Error>` | Initialise the contract. Must be called exactly once. |
| `get_admin` | `env: Env` | `Result<Address, Error>` | Returns the current admin address. |
| `add_authorized_sender` | `env: Env, caller: Address, sender: Address` | `Result<(), Error>` | Authorise `sender` to create notifications on behalf of integrated contracts. |
| `remove_authorized_sender` | `env: Env, caller: Address, sender: Address` | `Result<(), Error>` | Revoke a sender's authorisation. |
| `get_authorized_senders` | `env: Env` | `Result<Vec<Address>, Error>` | Returns the list of all currently authorised sender addresses. |
| `set_preferences` | `env: Env, user: Address, prefs: NotificationPreferences` | `Result<(), Error>` | Upsert `user`'s notification preferences. The user must sign the call. |
| `get_preferences` | `env: Env, user: Address` | `Result<Option<NotificationPreferences>, Error>` | Returns the preferences for `user`, or `None` if not configured. |
| `create_notification` | `env: Env, sender: Address, recipient: Address, notif_type: NotificationType, priority: AlertPriority, title: String, message: String, reference_id: Option<u64>, expires_at: Option<u64>` | `Result<u64, Error>` | Create a single notification for `recipient`. Caller must be the admin or an authorised sender. Returns the assigned notification ID. |
| `create_bulk_notifications` | `env: Env, sender: Address, recipients: Vec<Address>, notif_type: NotificationType, priority: AlertPriority, title: String, message: String, reference_id: Option<u64>, expires_at: Option<u64>` | `Result<Vec<u64>, Error>` | Create one notification per recipient in `recipients`. Bounded by MAX_BULK_RECIPIENTS to cap gas cost. |
| `get_notification` | `env: Env, caller: Address, notif_id: u64` | `Result<Notification, Error>` | Fetch a single notification by ID. Only the recipient or admin may view it. |
| `get_notifications` | `env: Env, caller: Address, user: Address, filter: NotificationFilter` | `Result<NotificationPage, Error>` | Paginated query over a user's notification history. Caller must be the user or admin. Results are returned newest-first; `filter.offset` skips matching records. |
| `get_unread_count` | `env: Env, user: Address` | `Result<u32, Error>` | Returns the number of unread (Pending + Delivered) notifications for a user. |
| `mark_read` | `env: Env, caller: Address, notif_id: u64` | `Result<(), Error>` | Mark a single notification as Read. Only the recipient may call this. |
| `mark_all_read` | `env: Env, caller: Address` | `Result<u32, Error>` | Mark all Pending / Delivered notifications for the caller as Read. Returns the count of newly-read notifications. |
| `archive_notification` | `env: Env, caller: Address, notif_id: u64` | `Result<(), Error>` | Archive a notification so it no longer appears in default queries. Caller must be the recipient or admin. |
| `create_alert_rule` | `env: Env, caller: Address, name: String, watches_type: u32, priority: AlertPriority, recipients: Vec<Address>` | `Result<u64, Error>` | Create a new alert rule. Only admin may call this. |
| `update_alert_rule` | `env: Env, caller: Address, rule_id: u64, is_active: bool, priority: AlertPriority, recipients: Vec<Address>` | `Result<(), Error>` | Update the active state, priority, and recipients of an existing rule. |
| `delete_alert_rule` | `env: Env, caller: Address, rule_id: u64` | `Result<(), Error>` | Permanently delete an alert rule. |
| `get_alert_rules` | `env: Env, caller: Address` | `Result<Vec<AlertRule>, Error>` | Returns all non-deleted alert rules. Admin only. |
| `trigger_alert` | `env: Env, sender: Address, rule_id: u64, reference_id: Option<u64>, custom_message: Option<String>` | `Result<Vec<u64>, Error>` | Trigger a specific alert rule: creates notifications for each of its recipients. Caller must be admin or an authorised sender. Returns the IDs of all created notifications. |
| `set_template` | `env: Env, caller: Address, template: NotificationTemplate` | `Result<(), Error>` | Upsert a localised notification template. Admin only. |
| `get_template` | `env: Env, notif_type: u32, locale: String` | `Result<NotificationTemplate, Error>` | Retrieve a template by notification type and locale. |
| `get_analytics` | `env: Env, caller: Address` | `Result<NotificationAnalytics, Error>` | Returns aggregated send/read/pending counters. Admin only. |

### Types

#### `enum DataKey`

| Variant | Value | Description |
|---|---|---|
| `Initialized` | — | — |
| `Admin` | — | — |
| `AuthorizedSenders` | — | — |
| `bounded by MAX_SENDERS

    
    SenderRate(Address)` | — | — |
| `UserPrefs(Address)` | — | — |
| `NotifCount` | — | — |
| `Notif(u64)` | — | — |
| `UserNotifIds(Address)` | — | — |
| `UserUnreadCount(Address)` | — | — |
| `AlertRuleCount` | — | — |
| `AlertRule(u64)` | — | — |
| `ActiveAlertRuleIds` | — | — |
| `Template(u32, String)` | — | — |
| `TotalSent` | — | — |
| `TotalRead` | — | — |
| `TotalPending` | — | — |
| `ByTypeSent(u32)` | — | — |
| `ByPrioritySent(u32)` | — | — |

### Error Codes

| Variant | Code | Description |
|---|---|---|
| `Unauthorized` | 100 | — |
| `SenderNotAuthorized` | 120 | — |
| `BatchTooLarge` | 208 | — |
| `RecipientsEmpty` | 209 | — |
| `TitleTooLong` | 221 | — |
| `MessageTooLong` | 222 | — |
| `NameTooLong` | 223 | — |
| `LocaleTooLong` | 224 | — |
| `InvalidNotifType` | 241 | — |
| `TooManyEnabledTypes` | 242 | — |
| `NotInitialized` | 300 | — |
| `AlreadyInitialized` | 301 | — |
| `RateLimitExceeded` | 307 | — |
| `AlreadyRead` | 330 | — |
| `AlreadyArchived` | 331 | — |
| `NotificationNotFound` | 450 | — |
| `AlertRuleNotFound` | 451 | — |
| `TemplateNotFound` | 452 | — |
| `SenderNotFound` | 453 | — |
| `MaxSendersReached` | 510 | — |
| `MaxRulesReached` | 511 | — |
| `MaxNotificationsReached` | 512 | — |
| `MaxTemplatesReached` | 513 | — |

### Examples

#### `test_initialize_stores_admin`

```rust
let env = Env::default();
    let (client, admin) = setup(&env);
    assert_eq!(client.get_admin(), admin);
```

#### `test_double_initialize_fails`

```rust
let env = Env::default();
    let (client, admin) = setup(&env);
    env.mock_all_auths();
    assert!(matches!(
        client.try_initialize(&admin),
        Err(Ok(Error::AlreadyInitialized))
    ));
```

#### `test_get_admin_before_init_fails`

```rust
let env = Env::default();
    let contract_id = Address::generate(&env);
    env.register_contract(&contract_id, NotificationContract);
    let client = NotificationContractClient::new(&env, &contract_id);
    env.mock_all_auths();
    assert!(client.try_get_admin().is_err());
```

---

## patient_consent_management

### Functions

| Function | Parameters | Returns | Description |
|---|---|---|---|
| `initialize` | `env: Env, admin: Address` | `Result<(), Error>` | — |
| `grant_consent` | `env: Env, patient: Address, provider: Address` | `Result<(), Error>` | — |
| `grant_consent_with_expiry` | `env: Env, patient: Address, provider: Address, expires_at: u64` | `Result<(), Error>` | — |
| `batch_grant_consent` | `env: Env, patient: Address, grantees: Vec<Address>` | `Result<u32, Error>` | Grant consent to multiple providers in a single transaction. |
| `revoke_consent` | `env: Env, patient: Address, provider: Address` | `Result<(), Error>` | — |
| `check_consent` | `env: Env, patient: Address, provider: Address` | `Result<bool, Error>` | — |
| `cleanup_expired_consents` | `env: Env, patient: Address` | `Result<u32, Error>` | — |
| `get_patient_consents` | `env: Env, patient: Address` | `Option<ConsentLog>` | — |
| `get_active_consent_count` | `env: Env, patient: Address` | `u32` | — |
| `verify_consent_with_audit` | `env: Env, patient: Address, provider: Address` | `Result<(bool, u64, u64), Error>` | — |
| `get_admin` | `env: Env` | `Result<Address, Error>` | — |
| `is_paused` | `env: Env` | `bool` | — |
| `pause` | `env: Env, caller: Address` | `Result<bool, Error>` | — |
| `unpause` | `env: Env, caller: Address` | `Result<bool, Error>` | — |
| `health_check` | `env: Env` | `bool` | On-chain health check endpoint. Returns true if the contract is initialized and operational. |
| `designate_proxy` | `env: Env, patient: Address, proxy_address: Address, scope: ProxyScope` | `()` | Patient designates a proxy who can act on their behalf when incapacitated. Requires the patient's own signature (invoke as patient). |
| `revoke_proxy` | `env: Env, patient: Address` | `()` | Patient revokes their currently designated proxy. |
| `get_proxy` | `env: Env, patient: Address` | `Option<ProxyRecord>` | Retrieve the proxy record for a patient, if one exists. |
| `proxy_grant_consent` | `env: Env, proxy: Address, patient: Address, grantee: Address` | `()` | Proxy grants consent on behalf of an incapacitated patient. Checks that caller is the designated proxy and scope allows it. |
| `proxy_revoke_consent` | `env: Env, proxy: Address, patient: Address, grantee: Address` | `()` | Proxy revokes consent on behalf of an incapacitated patient. |

### Types

#### `struct ConsentRecord`

| Field | Type | Description |
|---|---|---|
| `patient` | `Address` | — |
| `provider` | `Address` | — |
| `granted_at` | `u64` | — |
| `expires_at` | `u64` | — |
| `revoked_at` | `u64` | — |
| `active` | `bool` | — |

#### `struct ConsentLog`

| Field | Type | Description |
|---|---|---|
| `records` | `Vec<ConsentRecord>` | — |
| `record_count` | `u32` | — |

#### `enum DataKey`

| Variant | Value | Description |
|---|---|---|
| `Initialized` | — | — |
| `Admin` | — | — |
| `Paused` | — | — |
| `ConsentStorage(Address)` | — | — |
| `ProviderIndex(Address, Address)` | — | — |

#### `enum ProxyScope`

| Variant | Value | Description |
|---|---|---|
| `FullAuthority` | — | — |
| `EmergencyOnly` | — | — |
| `ReadOnly` | — | — |

#### `struct ProxyRecord`

| Field | Type | Description |
|---|---|---|
| `proxy_address` | `Address` | — |
| `scope` | `ProxyScope` | — |
| `designated_at` | `u64` | — |

#### `enum ProxyKey`

| Variant | Value | Description |
|---|---|---|
| `Proxy(Address)` | — | — |

### Error Codes

| Variant | Code | Description |
|---|---|---|
| `Unauthorized` | 100 | — |
| `InvalidPatient` | 210 | — |
| `InvalidProvider` | 211 | — |
| `NotInitialized` | 300 | — |
| `AlreadyInitialized` | 301 | — |
| `ContractPaused` | 302 | — |
| `ConsentNotFound` | 406 | — |
| `ConsentAlreadyExists` | 460 | — |
| `InvalidExpiry` | 470 | — |

### Examples

#### `test_initialize`

```rust
let (_env, client, admin) = setup();
        client.initialize(&admin);
    }

    #[test]
    fn test_initialize_twice_fails() {
        let (_env, client, admin) = setup();
        client.initialize(&admin);
        let result = client.try_initialize(&admin);
```

---

## patient_gamification

### Functions

| Function | Parameters | Returns | Description |
|---|---|---|---|
| `initialize` | `env: Env, admin: Address, points_per_achievement: u32, points_per_challenge: u32, points_per_streak_day: u32, max_daily_points: u32, privacy_threshold: u32` | `Result<bool, Error>` | — |
| `create_achievement` | `env: Env, caller: Address, name: String, description: String, category: String, points_reward: u32, badge_uri: String, requirement_type: String, requirement_value: u32` | `Result<u64, Error>` | — |
| `get_achievement` | `env: Env, achievement_id: u64` | `Result<Achievement, Error>` | — |
| `update_achievement_progress` | `env: Env, caller: Address, patient_id: Address, achievement_id: u64, progress: u32` | `Result<bool, Error>` | — |
| `get_patient_achievement` | `env: Env, patient_id: Address, achievement_id: u64` | `Result<PatientAchievement, Error>` | — |
| `get_patient_achievements` | `env: Env, patient_id: Address` | `Result<Vec<u64>, Error>` | — |
| `create_challenge` | `env: Env, caller: Address, name: String, description: String, challenge_type: String, metric_name: String, target_value: u32, start_time: u64, end_time: u64, points_reward: u32, max_participants: u32` | `Result<u64, Error>` | — |
| `get_challenge` | `env: Env, challenge_id: u64` | `Result<HealthChallenge, Error>` | — |
| `join_challenge` | `env: Env, patient_id: Address, challenge_id: u64` | `Result<bool, Error>` | — |
| `update_challenge_progress` | `env: Env, caller: Address, patient_id: Address, challenge_id: u64, value: u32` | `Result<bool, Error>` | — |
| `get_challenge_participant` | `env: Env, challenge_id: u64, patient_id: Address` | `Result<ChallengeParticipant, Error>` | — |
| `get_challenge_participants` | `env: Env, challenge_id: u64` | `Result<Vec<Address>, Error>` | — |
| `get_reward_points` | `env: Env, patient_id: Address` | `Result<RewardPoints, Error>` | — |
| `redeem_points` | `env: Env, patient_id: Address, points: u64` | `Result<bool, Error>` | — |
| `commit_random_bonus` | `env: Env, patient_id: Address, reveal_hash: BytesN<32>, target_ledger: u32, max_bonus_points: u32` | `Result<bool, Error>` | — |
| `reveal_random_bonus` | `env: Env, patient_id: Address, reveal: BytesN<32>` | `Result<RandomBonusOutcome, Error>` | — |
| `get_random_bonus_commitment` | `env: Env, patient_id: Address` | `Result<RandomBonusCommitment, Error>` | — |
| `create_social_profile` | `env: Env, patient_id: Address, display_name: String, bio: String, avatar_uri: String, is_public: bool, show_achievements: bool, show_challenges: bool, show_points: bool` | `Result<bool, Error>` | — |
| `get_social_profile` | `env: Env, patient_id: Address` | `Result<SocialProfile, Error>` | — |
| `update_social_profile` | `env: Env, patient_id: Address, display_name: String, bio: String, avatar_uri: String, is_public: bool, show_achievements: bool, show_challenges: bool, show_points: bool` | `Result<bool, Error>` | — |
| `get_leaderboard` | `env: Env, limit: u32` | `Result<Vec<LeaderboardEntry>, Error>` | — |
| `get_patient_rank` | `env: Env, patient_id: Address` | `Result<u32, Error>` | — |
| `record_health_metric` | `env: Env, caller: Address, patient_id: Address, metric_name: String, value: u32, unit: String, source: String` | `Result<bool, Error>` | — |
| `get_health_metric` | `env: Env, patient_id: Address, metric_name: String, timestamp: u64` | `Result<HealthMetric, Error>` | — |
| `get_health_metrics` | `env: Env, patient_id: Address, metric_name: String` | `Result<Vec<u64>, Error>` | — |
| `get_daily_streak` | `env: Env, patient_id: Address` | `Result<DailyStreak, Error>` | — |
| `update_config` | `env: Env, caller: Address, points_per_achievement: u32, points_per_challenge: u32, points_per_streak_day: u32, max_daily_points: u32, privacy_threshold: u32, enabled: bool` | `Result<bool, Error>` | — |
| `deactivate_achievement` | `env: Env, caller: Address, achievement_id: u64` | `Result<bool, Error>` | — |
| `deactivate_challenge` | `env: Env, caller: Address, challenge_id: u64` | `Result<bool, Error>` | — |
| `get_config` | `env: Env` | `Result<GamificationConfig, Error>` | — |
| `get_total_achievements` | `env: Env` | `Result<u64, Error>` | — |
| `get_total_challenges` | `env: Env` | `Result<u64, Error>` | — |

### Types

#### `struct GamificationConfig`

| Field | Type | Description |
|---|---|---|
| `admin` | `Address` | — |
| `points_per_achievement` | `u32` | — |
| `points_per_challenge` | `u32` | — |
| `points_per_streak_day` | `u32` | — |
| `max_daily_points` | `u32` | — |
| `privacy_threshold` | `u32` | — |
| `enabled` | `bool` | — |

#### `struct Achievement`

| Field | Type | Description |
|---|---|---|
| `id` | `u64` | — |
| `name` | `String` | — |
| `description` | `String` | — |
| `category` | `String` | — |
| `points_reward` | `u32` | — |
| `badge_uri` | `String` | — |
| `requirement_type` | `String` | — |
| `requirement_value` | `u32` | — |
| `is_active` | `bool` | — |
| `created_at` | `u64` | — |

#### `struct PatientAchievement`

| Field | Type | Description |
|---|---|---|
| `patient_id` | `Address` | — |
| `achievement_id` | `u64` | — |
| `earned_at` | `u64` | — |
| `progress` | `u32` | — |
| `is_completed` | `bool` | — |

#### `struct HealthChallenge`

| Field | Type | Description |
|---|---|---|
| `id` | `u64` | — |
| `name` | `String` | — |
| `description` | `String` | — |
| `challenge_type` | `String` | — |
| `metric_name` | `String` | — |
| `target_value` | `u32` | — |
| `start_time` | `u64` | — |
| `end_time` | `u64` | — |
| `points_reward` | `u32` | — |
| `max_participants` | `u32` | — |
| `current_participants` | `u32` | — |
| `is_active` | `bool` | — |
| `created_at` | `u64` | — |

#### `struct ChallengeParticipant`

| Field | Type | Description |
|---|---|---|
| `patient_id` | `Address` | — |
| `challenge_id` | `u64` | — |
| `current_value` | `u32` | — |
| `joined_at` | `u64` | — |
| `completed_at` | `Option<u64>` | — |
| `is_completed` | `bool` | — |

#### `struct RewardPoints`

| Field | Type | Description |
|---|---|---|
| `patient_id` | `Address` | — |
| `total_points` | `u64` | — |
| `available_points` | `u64` | — |
| `lifetime_points` | `u64` | — |
| `last_updated` | `u64` | — |

#### `struct RandomBonusCommitment`

| Field | Type | Description |
|---|---|---|
| `patient_id` | `Address` | — |
| `reveal_hash` | `BytesN<32>` | — |
| `target_ledger` | `u32` | — |
| `expires_at_ledger` | `u32` | — |
| `max_bonus_points` | `u32` | — |
| `committed_at` | `u64` | — |

#### `struct RandomBonusOutcome`

| Field | Type | Description |
|---|---|---|
| `patient_id` | `Address` | — |
| `random_value` | `u64` | — |
| `bonus_points` | `u32` | — |
| `target_ledger` | `u32` | — |
| `revealed_at` | `u64` | — |

#### `struct LeaderboardEntry`

| Field | Type | Description |
|---|---|---|
| `patient_id` | `Address` | — |
| `display_name` | `String` | — |
| `points` | `u64` | — |
| `achievements_count` | `u32` | — |
| `challenges_completed` | `u32` | — |
| `rank` | `u32` | — |
| `last_updated` | `u64` | — |

#### `struct SocialProfile`

| Field | Type | Description |
|---|---|---|
| `patient_id` | `Address` | — |
| `display_name` | `String` | — |
| `bio` | `String` | — |
| `avatar_uri` | `String` | — |
| `is_public` | `bool` | — |
| `show_achievements` | `bool` | — |
| `show_challenges` | `bool` | — |
| `show_points` | `bool` | — |
| `created_at` | `u64` | — |
| `last_active` | `u64` | — |

#### `struct HealthMetric`

| Field | Type | Description |
|---|---|---|
| `patient_id` | `Address` | — |
| `metric_name` | `String` | — |
| `value` | `u32` | — |
| `unit` | `String` | — |
| `recorded_at` | `u64` | — |
| `source` | `String` | — |

#### `struct DailyStreak`

| Field | Type | Description |
|---|---|---|
| `patient_id` | `Address` | — |
| `current_streak` | `u32` | — |
| `longest_streak` | `u32` | — |
| `last_activity_date` | `u64` | — |
| `total_active_days` | `u32` | — |

#### `enum DataKey`

| Variant | Value | Description |
|---|---|---|
| `Config` | — | — |
| `AchievementCounter` | — | — |
| `Achievement(u64)` | — | — |
| `PatientAchievement(Address, u64)` | — | — |
| `PatientAchievements(Address)` | — | — |
| `ChallengeCounter` | — | — |
| `Challenge(u64)` | — | — |
| `ChallengeParticipant(u64, Address)` | — | — |
| `ChallengeParticipants(u64)` | — | — |
| `RewardPoints(Address)` | — | — |
| `RandomBonusCommitment(Address)` | — | — |
| `Leaderboard` | — | — |
| `SocialProfile(Address)` | — | — |
| `HealthMetric(Address, String, u64)` | — | — |
| `HealthMetrics(Address, String)` | — | — |
| `DailyStreak(Address)` | — | — |
| `Admin(Address)` | — | — |

#### `enum Error`

| Variant | Value | Description |
|---|---|---|
| `NotAuthorized` | 1 | — |
| `AlreadyInitialized` | 2 | — |
| `NotInitialized` | 3 | — |
| `InvalidInput` | 4 | — |
| `AchievementNotFound` | 5 | — |
| `ChallengeNotFound` | 6 | — |
| `ChallengeFull` | 7 | — |
| `ChallengeEnded` | 8 | — |
| `AlreadyParticipating` | 9 | — |
| `NotParticipating` | 10 | — |
| `InsufficientPoints` | 11 | — |
| `PrivacyThresholdNotMet` | 12 | — |
| `InvalidTimeRange` | 13 | — |
| `AlreadyCompleted` | 14 | — |
| `RandomnessAlreadyCommitted` | 15 | — |
| `RandomnessCommitNotFound` | 16 | — |
| `RandomnessRevealTooEarly` | 17 | — |
| `RandomnessRevealMismatch` | 18 | — |
| `RandomnessCommitExpired` | 19 | — |

### Examples

#### `test_initialize`

```rust
let env = Env::default();
    let contract_id = env.register_contract(None, PatientGamificationContract);
    let client = PatientGamificationContractClient::new(&env, &contract_id);

    let admin = Address::random(&env);

    let result = client.initialize(
        &admin,
        &100u32,  // points_per_achievement
```

#### `test_create_achievement`

```rust
let env = Env::default();
    let contract_id = env.register_contract(None, PatientGamificationContract);
    let client = PatientGamificationContractClient::new(&env, &contract_id);

    let admin = Address::random(&env);

    client.initialize(
        &admin,
        &100u32,
```

#### `test_achievement_progress`

```rust
let env = Env::default();
    let contract_id = env.register_contract(None, PatientGamificationContract);
    let client = PatientGamificationContractClient::new(&env, &contract_id);

    let admin = Address::random(&env);
    let patient = Address::random(&env);

    client.initialize(
        &admin,
```

---

## patient_portal

### Functions

| Function | Parameters | Returns | Description |
|---|---|---|---|
| `initialize` | `env: Env, admin: Address` | `Result<(), PatientPortalError>` | — |
| `set_integration_contracts` | `env: Env, caller: Address, medical_records: Address, identity_registry: Address` | `Result<(), PatientPortalError>` | Wire `medical_records` and `identity_registry` for integrators (viewing uses those contracts). |
| `get_medical_records_contract` | `env: Env` | `Result<Option<Address>, PatientPortalError>` | — |
| `get_identity_registry_contract` | `env: Env` | `Result<Option<Address>, PatientPortalError>` | — |
| `pause` | `env: Env, caller: Address` | `Result<(), PatientPortalError>` | — |
| `unpause` | `env: Env, caller: Address` | `Result<(), PatientPortalError>` | — |
| `register` | `env: Env, patient: Address, identity_commitment: BytesN<32>, locale: String` | `Result<(), PatientPortalError>` | Patient-signed registration for the portal (pairs with secure auth in the dApp). |
| `get_profile` | `env: Env, patient: Address` | `Result<PortalProfile, PatientPortalError>` | — |
| `request_phr_export` | `env: Env, patient: Address, record_ids: Vec<u64>, manifest_hash: BytesN<32>` | `Result<u64, PatientPortalError>` | Audit trail for PHR download / export (actual ciphertext lives off-chain). |
| `get_export` | `env: Env, id: u64` | `Result<PhrExportManifest, PatientPortalError>` | — |
| `schedule_appointment` | `env: Env, patient: Address, provider: Address, start_ts: u64, end_ts: u64, telemedicine_appointment_id: BytesN<32>, notes: String` | `Result<u64, PatientPortalError>` | Book or request an appointment; link to telemedicine appointment id when available. |
| `set_appointment_status` | `env: Env, patient: Address, appointment_id: u64, status: AppointmentStatus` | `Result<(), PatientPortalError>` | — |
| `get_appointment` | `env: Env, appointment_id: u64` | `Result<PortalAppointment, PatientPortalError>` | — |
| `list_my_appointment_ids` | `env: Env, patient: Address` | `Result<Vec<u64>, PatientPortalError>` | — |
| `log_medication_event` | `env: Env, patient: Address, medication_ref: String, scheduled_for: u64, taken: bool` | `Result<u64, PatientPortalError>` | Medication adherence tracking (references prescriptions / meds off-chain or in EMR). |
| `get_adherence_event` | `env: Env, id: u64` | `Result<MedicationAdherenceEvent, PatientPortalError>` | — |
| `list_my_adherence_ids` | `env: Env, patient: Address` | `Result<Vec<u64>, PatientPortalError>` | — |

### Types

#### `enum PatientPortalError`

| Variant | Value | Description |
|---|---|---|
| `AlreadyInitialized` | 1 | — |
| `NotInitialized` | 2 | — |
| `NotAdmin` | 3 | — |
| `Paused` | 4 | — |
| `AlreadyRegistered` | 5 | — |
| `NotRegistered` | 6 | — |
| `AppointmentNotFound` | 7 | — |
| `ExportTooManyRecords` | 8 | — |
| `InvalidInput` | 9 | — |
| `NotAppointmentOwner` | 10 | — |

#### `enum AppointmentStatus`

| Variant | Value | Description |
|---|---|---|
| `Requested` | — | — |
| `Confirmed` | — | — |
| `Completed` | — | — |
| `Cancelled` | — | — |

#### `struct PortalProfile`

| Field | Type | Description |
|---|---|---|
| `patient` | `Address` | — |
| `registered_at` | `u64` | — |
| `identity_commitment` | `BytesN<32>` | — |
| `locale` | `String` | — |

#### `struct PortalAppointment`

| Field | Type | Description |
|---|---|---|
| `id` | `u64` | — |
| `patient` | `Address` | — |
| `provider` | `Address` | — |
| `start_ts` | `u64` | — |
| `end_ts` | `u64` | — |
| `status` | `AppointmentStatus` | — |
| `telemedicine_appointment_id` | `BytesN<32>` | — |
| `notes` | `String` | — |

#### `struct MedicationAdherenceEvent`

| Field | Type | Description |
|---|---|---|
| `id` | `u64` | — |
| `patient` | `Address` | — |
| `medication_ref` | `String` | — |
| `scheduled_for` | `u64` | — |
| `taken` | `bool` | — |
| `logged_at` | `u64` | — |

#### `struct PhrExportManifest`

| Field | Type | Description |
|---|---|---|
| `id` | `u64` | — |
| `patient` | `Address` | — |
| `record_ids` | `Vec<u64>` | — |
| `requested_at` | `u64` | — |
| `manifest_hash` | `BytesN<32>` | — |

#### `enum DataKey`

| Variant | Value | Description |
|---|---|---|
| `Initialized` | — | — |
| `Admin` | — | — |
| `Paused` | — | — |
| `MedicalRecords` | — | — |
| `IdentityRegistry` | — | — |
| `NextAppointmentId` | — | — |
| `Appointment(u64)` | — | — |
| `NextAdherenceId` | — | — |
| `Adherence(u64)` | — | — |
| `NextExportId` | — | — |
| `Export(u64)` | — | — |
| `Profile(Address)` | — | — |
| `PatientAppointmentIds(Address)` | — | — |
| `PatientAdherenceIds(Address)` | — | — |
| `PatientExportIds(Address)` | — | — |

### Examples

#### `register_and_integrations`

```rust
let env = Env::default();
    let (client, admin) = setup(&env);
    let med = Address::generate(&env);
    let ident = Address::generate(&env);
    client
        .try_set_integration_contracts(&admin, &med, &ident)
        .unwrap()
        .unwrap();
    assert_eq!(
```

#### `appointments_and_adherence_and_export`

```rust
let env = Env::default();
    let (client, _admin) = setup(&env);
    let patient = Address::generate(&env);
    let provider = Address::generate(&env);
    client
        .try_register(
            &patient,
            &BytesN::from_array(&env, &[0u8; 32]),
            &String::from_str(&env, "en"),
```

---

## patient_risk_stratification

### Functions

| Function | Parameters | Returns | Description |
|---|---|---|---|
| `initialize` | `env: Env, admin: Address` | `Result<(), Error>` | — |
| `register_risk_model` | `env: Env, caller: Address, model_id: BytesN<32>, model_type: RiskModelType, specialty: String, version: String, min_confidence_bps: u32, description: String` | `Result<(), Error>` | — |
| `perform_risk_assessment` | `env: Env, caller: Address, patient: Address, model_id: BytesN<32>, risk_score_bps: u32, confidence_bps: u32, prediction_horizon_days: u32, risk_factors: Vec<RiskFactor>, interventions: Vec<InterventionRecommendation>, auc_score_bps: u32` | `Result<u64, Error>` | — |
| `get_risk_assessment` | `env: Env, assessment_id: u64` | `Option<RiskAssessment>` | — |
| `get_patient_risk_profile` | `env: Env, patient: Address` | `Option<PatientRiskProfile>` | — |
| `get_risk_model` | `env: Env, model_id: BytesN<32>` | `Option<RiskModel>` | — |
| `get_patient_risk_factors` | `env: Env, patient: Address, specialty: String` | `Vec<RiskFactor>` | — |
| `get_intervention_recommendations` | `env: Env, patient: Address` | `Vec<InterventionRecommendation>` | — |
| `update_model_status` | `env: Env, caller: Address, model_id: BytesN<32>, enabled: bool` | `Result<bool, Error>` | — |
| `get_suggestion` | `error: Error` | `Symbol` | — |

### Types

#### `enum RiskModelType`

| Variant | Value | Description |
|---|---|---|
| `Readmission` | — | — |
| `Mortality` | — | — |
| `Complications` | — | — |

#### `struct RiskModel`

| Field | Type | Description |
|---|---|---|
| `model_id` | `BytesN<32>` | — |
| `model_type` | `RiskModelType` | — |
| `specialty` | `String` | — |
| `version` | `String` | — |
| `min_confidence_bps` | `u32` | — |
| `enabled` | `bool` | — |
| `description` | `String` | — |

#### `struct RiskAssessment`

| Field | Type | Description |
|---|---|---|
| `assessment_id` | `u64` | — |
| `patient` | `Address` | — |
| `model_id` | `BytesN<32>` | — |
| `risk_score_bps` | `u32` | — |
| `confidence_bps` | `u32` | — |
| `assessment_date` | `u64` | — |
| `prediction_horizon_days` | `u32` | — |
| `risk_factors` | `Vec<RiskFactor>` | — |
| `interventions` | `Vec<InterventionRecommendation>` | — |
| `specialty` | `String` | — |
| `auc_score_bps` | `u32` | — |

#### `struct RiskFactor`

| Field | Type | Description |
|---|---|---|
| `factor_name` | `String` | — |
| `contribution_bps` | `i32` | — |
| `importance_bps` | `u32` | — |
| `category` | `String` | — |
| `explanation` | `String` | — |

#### `struct InterventionRecommendation`

| Field | Type | Description |
|---|---|---|
| `intervention_type` | `String` | — |
| `priority` | `u32` | — |
| `description` | `String` | — |
| `expected_impact_bps` | `u32` | — |
| `timeframe_days` | `u32` | — |
| `resources_needed` | `Vec<String>` | — |

#### `struct PatientRiskProfile`

| Field | Type | Description |
|---|---|---|
| `patient` | `Address` | — |
| `latest_assessment_id` | `u64` | — |
| `current_risk_level` | `String` | — |
| `risk_trend` | `String` | — |
| `last_updated` | `u64` | — |
| `total_assessments` | `u32` | — |
| `specialty_profiles` | `Map<String, SpecialtyRiskSummary>` | — |

#### `struct SpecialtyRiskSummary`

| Field | Type | Description |
|---|---|---|
| `specialty` | `String` | — |
| `avg_risk_score_bps` | `u32` | — |
| `high_risk_count` | `u32` | — |
| `last_assessment_date` | `u64` | — |

#### `enum DataKey`

| Variant | Value | Description |
|---|---|---|
| `Config` | — | — |
| `RiskModel(BytesN<32>)` | — | — |
| `Assessment(u64)` | — | — |
| `PatientProfile(Address)` | — | — |
| `AssessmentCounter` | — | — |
| `ModelRegistry(RiskModelType)` | — | — |

#### `enum Error`

| Variant | Value | Description |
|---|---|---|
| `NotAuthorized` | 1 | — |
| `ConfigNotSet` | 2 | — |
| `ModelNotFound` | 3 | — |
| `InvalidScore` | 4 | — |
| `LowConfidence` | 5 | — |
| `AssessmentNotFound` | 6 | — |
| `InvalidModel` | 7 | — |
| `DuplicateModel` | 8 | — |
| `AlreadyInitialized` | 9 | — |

---

## payment_router

### Functions

| Function | Parameters | Returns | Description |
|---|---|---|---|
| `set_fee_config` | `env: Env, fee_receiver: Address, platform_fee_bps: u32` | `Result<(), Error>` | — |
| `get_fee_config` | `env: Env` | `Option<RouterFeeConfig>` | — |
| `compute_split` | `env: Env, amount: i128` | `Result<(i128, i128), Error>` | — |

### Types

#### `enum Error`

| Variant | Value | Description |
|---|---|---|
| `InvalidFeeBps` | 1 | — |
| `FeeNotSet` | 2 | — |
| `Overflow` | 3 | — |
| `InsufficientFunds` | 10 | — |
| `DeadlineExceeded` | 11 | — |
| `InvalidSignature` | 12 | — |
| `UnauthorizedCaller` | 13 | — |
| `ContractPaused` | 14 | — |
| `StorageFull` | 15 | — |
| `CrossChainTimeout` | 16 | — |

#### `struct RouterFeeConfig`

| Field | Type | Description |
|---|---|---|
| `platform_fee_bps` | `u32` | — |
| `fee_receiver` | `Address` | — |

---

## pharma_supply_chain

### Functions

| Function | Parameters | Returns | Description |
|---|---|---|---|
| `initialize` | `env: Env, admin: Address` | `Result<(), Error>` | — |
| `register_manufacturer` | `env: Env, admin: Address, operator: Address, name: String, license_number: String` | `Result<u64, Error>` | — |
| `register_medication` | `env: Env, caller: Address, manufacturer_id: u64, name: String, ndc: String, requires_cold_chain: bool, min_temp_c: i32, max_temp_c: i32, regulatory_region: String` | `Result<u64, Error>` | — |
| `create_batch` | `env: Env, caller: Address, medication_id: u64, lot_number: String, quantity: u32, auth_hash: BytesN<32>, expires_at: u64` | `Result<u64, Error>` | — |
| `verify_batch_authenticity` | `env: Env, batch_id: u64, auth_hash: BytesN<32>` | `Result<bool, Error>` | — |
| `create_shipment` | `env: Env, caller: Address, batch_id: u64, to: Address, carrier_ref: String` | `Result<u64, Error>` | — |
| `log_condition_data` | `env: Env, caller: Address, shipment_id: u64, temperature_c: i32, humidity_bps: u32, latitude_e6: i32, longitude_e6: i32` | `Result<bool, Error>` | — |
| `complete_shipment` | `env: Env, caller: Address, shipment_id: u64, verified: bool` | `Result<bool, Error>` | — |
| `run_compliance_check` | `env: Env, batch_id: u64` | `Result<bool, Error>` | — |
| `get_inventory_snapshot` | `env: Env, owner: Address` | `InventorySnapshot` | — |
| `optimize_inventory` | `env: Env, owner: Address, forecast_units: u32` | `InventoryRecommendation` | — |
| `get_batch` | `env: Env, batch_id: u64` | `Result<Batch, Error>` | — |
| `get_shipment` | `env: Env, shipment_id: u64` | `Result<Shipment, Error>` | — |

### Types

#### `enum Error`

| Variant | Value | Description |
|---|---|---|
| `AlreadyInitialized` | 1 | — |
| `NotInitialized` | 2 | — |
| `Unauthorized` | 3 | — |
| `ManufacturerNotFound` | 4 | — |
| `MedicationNotFound` | 5 | — |
| `BatchNotFound` | 6 | — |
| `ShipmentNotFound` | 7 | — |
| `InvalidInput` | 8 | — |
| `BatchAlreadyExists` | 9 | — |

#### `enum BatchStatus`

| Variant | Value | Description |
|---|---|---|
| `Manufactured` | 0 | — |
| `InTransit` | 1 | — |
| `Delivered` | 2 | — |
| `Recalled` | 3 | — |

#### `enum ShipmentStatus`

| Variant | Value | Description |
|---|---|---|
| `Created` | 0 | — |
| `InTransit` | 1 | — |
| `Delivered` | 2 | — |
| `Flagged` | 3 | — |

#### `struct Manufacturer`

| Field | Type | Description |
|---|---|---|
| `id` | `u64` | — |
| `operator` | `Address` | — |
| `name` | `String` | — |
| `license_number` | `String` | — |
| `active` | `bool` | — |

#### `struct Medication`

| Field | Type | Description |
|---|---|---|
| `id` | `u64` | — |
| `manufacturer_id` | `u64` | — |
| `name` | `String` | — |
| `ndc` | `String` | — |
| `requires_cold_chain` | `bool` | — |
| `min_temp_c` | `i32` | — |
| `max_temp_c` | `i32` | — |
| `regulatory_region` | `String` | — |

#### `struct Batch`

| Field | Type | Description |
|---|---|---|
| `id` | `u64` | — |
| `medication_id` | `u64` | — |
| `lot_number` | `String` | — |
| `quantity` | `u32` | — |
| `auth_hash` | `BytesN<32>` | — |
| `manufactured_at` | `u64` | — |
| `expires_at` | `u64` | — |
| `current_owner` | `Address` | — |
| `status` | `BatchStatus` | — |
| `compliance_ok` | `bool` | — |

#### `struct Shipment`

| Field | Type | Description |
|---|---|---|
| `id` | `u64` | — |
| `batch_id` | `u64` | — |
| `from` | `Address` | — |
| `to` | `Address` | — |
| `carrier_ref` | `String` | — |
| `status` | `ShipmentStatus` | — |
| `latest_temp_c` | `i32` | — |
| `latest_humidity_bps` | `u32` | — |
| `latitude_e6` | `i32` | — |
| `longitude_e6` | `i32` | — |
| `compliance_ok` | `bool` | — |
| `created_at` | `u64` | — |
| `delivered_at` | `u64` | — |

#### `struct InventorySnapshot`

| Field | Type | Description |
|---|---|---|
| `owner` | `Address` | — |
| `batch_count` | `u32` | — |
| `total_units` | `u32` | — |
| `cold_chain_violations` | `u32` | — |
| `last_updated` | `u64` | — |

#### `struct InventoryRecommendation`

| Field | Type | Description |
|---|---|---|
| `owner` | `Address` | — |
| `available_units` | `u32` | — |
| `forecast_units` | `u32` | — |
| `reorder_needed` | `bool` | — |
| `recommended_reorder_units` | `u32` | — |

#### `enum DataKey`

| Variant | Value | Description |
|---|---|---|
| `Admin` | — | — |
| `ManufacturerCount` | — | — |
| `Manufacturer(u64)` | — | — |
| `MedicationCount` | — | — |
| `Medication(u64)` | — | — |
| `BatchCount` | — | — |
| `Batch(u64)` | — | — |
| `BatchByLotNumber(String)` | — | — |
| `ShipmentCount` | — | — |
| `Shipment(u64)` | — | — |

---

## predictive_analytics

### Functions

| Function | Parameters | Returns | Description |
|---|---|---|---|
| `initialize` | `env: Env, admin: Address, predictor: Address, prediction_horizon_days: u32, min_confidence_bps: u32` | `bool` | — |
| `update_config` | `env: Env, caller: Address, new_predictor: Option<Address>, new_horizon: Option<u32>, new_min_confidence: Option<u32>, enabled: Option<bool>` | `Result<bool, Error>` | — |
| `make_prediction` | `env: Env, caller: Address, patient: Address, model_id: BytesN<32>, outcome_type: String, predicted_value: u32, confidence_bps: u32, features_used: Vec<String>, explanation_ref: String, risk_factors: Vec<String>` | `Result<u64, Error>` | — |
| `get_prediction` | `env: Env, prediction_id: u64` | `Option<HealthPrediction>` | — |
| `get_config` | `env: Env` | `Option<PredictionConfig>` | — |
| `get_patient_summary` | `env: Env, patient: Address` | `Option<PatientPredictionsSummary>` | — |
| `get_model_metrics` | `env: Env, model_id: BytesN<32>` | `Option<PredictionMetrics>` | — |
| `update_model_metrics` | `env: Env, caller: Address, model_id: BytesN<32>, metrics: PredictionMetrics` | `Result<bool, Error>` | — |
| `has_high_risk_prediction` | `env: Env, patient: Address` | `bool` | — |
| `whitelist_predictor` | `env: Env, caller: Address, predictor_addr: Address` | `Result<bool, Error>` | — |
| `is_whitelisted_predictor` | `env: Env, predictor_addr: Address` | `bool` | — |

### Examples

#### `test_prediction_flow`

```rust
let env = Env::default();
    let contract_id = env.register_contract(None, PredictiveAnalyticsContract);
    let client = PredictiveAnalyticsContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let predictor = Address::generate(&env);
    let patient = Address::generate(&env);

    client
```

#### `test_low_confidence_rejection`

```rust
let env = Env::default();
    let contract_id = env.register_contract(None, PredictiveAnalyticsContract);
    let client = PredictiveAnalyticsContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let predictor = Address::generate(&env);
    let patient = Address::generate(&env);

    client
```

#### `test_config_updates`

```rust
let env = Env::default();
    let contract_id = env.register_contract(None, PredictiveAnalyticsContract);
    let client = PredictiveAnalyticsContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let predictor = Address::generate(&env);

    client
        .mock_all_auths()
```

---

## provider_directory

### Functions

| Function | Parameters | Returns | Description |
|---|---|---|---|
| `initialize` | `env: Env, admin: Address` | `Result<(), Error>` | — |
| `set_rate_limit_config` | `env: Env, admin: Address, max_searches: u32, window_secs: u64` | `Result<(), Error>` | — |
| `set_institution_exemption` | `env: Env, admin: Address, institution: Address, is_exempt: bool` | `Result<(), Error>` | — |
| `search_providers` | `env: Env, caller: Address, _query: String` | `Result<Vec<Provider>, Error>` | — |

### Types

#### `enum Error`

| Variant | Value | Description |
|---|---|---|
| `NotInitialized` | 1 | — |
| `AlreadyInitialized` | 2 | — |
| `RateLimitExceeded` | 3 | — |
| `NotAuthorized` | 4 | — |

#### `enum DataKey`

| Variant | Value | Description |
|---|---|---|
| `Admin` | — | — |
| `RateLimitConfig` | — | — |
| `SearchRateLimit(Address)` | — | — |
| `ExemptInstitution(Address)` | — | — |

#### `struct RateLimitConfig`

| Field | Type | Description |
|---|---|---|
| `max_searches` | `u32` | — |
| `window_secs` | `u64` | — |

#### `struct Provider`

| Field | Type | Description |
|---|---|---|
| `id` | `Address` | — |
| `name` | `String` | — |
| `specialty` | `String` | — |

### Examples

#### `test_initialize`

```rust
let env = Env::default();
    let admin = Address::generate(&env);
    let identity_registry = Address::generate(&env);
    
    let contract_id = env.register_contract(None, ProviderDirectoryContract);
    let client = ProviderDirectoryContractClient::new(&env, &contract_id);

    client.initialize(&admin, &identity_registry);
```

#### `test_profile_management`

```rust
let env = Env::default();
    env.mock_all_auths();
    
    let admin = Address::generate(&env);
    let identity_registry = Address::generate(&env);
    let provider = Address::generate(&env);
    
    let contract_id = env.register_contract(None, ProviderDirectoryContract);
    let client = ProviderDirectoryContractClient::new(&env, &contract_id);
```

#### `test_search_by_specialty`

```rust
let env = Env::default();
    env.mock_all_auths();
    
    let admin = Address::generate(&env);
    let identity_registry = Address::generate(&env);
    let p1 = Address::generate(&env);
    let p2 = Address::generate(&env);
    
    let contract_id = env.register_contract(None, ProviderDirectoryContract);
```

---

## public_health_surveillance

### Functions

| Function | Parameters | Returns | Description |
|---|---|---|---|
| `initialize` | `env: Env, admin: Address` | `Result<(), Error>` | Initialize the public health surveillance platform |
| `report_outbreak_data` | `env: Env, provider: Address, data_id: BytesN<32>, encrypted_region: Bytes, disease_code: String, aggregated_cases: u64, time_period_start: u64, time_period_end: u64, aggregation_method: AggregationMethod, privacy_epsilon: u64, confidence_bps: u32` | `Result<(), Error>` | Report outbreak data with privacy preservation |
| `create_epidemic_model` | `env: Env, modeler: Address, model_id: BytesN<32>, disease_code: String, encrypted_scope: Bytes, model_type: String, r0_estimate: u64, incubation_days: u32, infectious_days: u32, case_fatality_bps: u32` | `Result<(), Error>` | Create epidemic model for disease prediction |
| `create_public_health_alert` | `env: Env, authority: Address, alert_type: AlertType, severity: DiseaseSeverity, encrypted_affected_regions: Bytes, message: String, recommended_actions: Vec<String>, expiration_hours: u32` | `Result<u64, Error>` | Create public health alert |
| `report_vaccination_coverage` | `env: Env, provider: Address, coverage_id: BytesN<32>, encrypted_region: Bytes, vaccine_type: String, encrypted_target_population: u64, private_vaccinated_count: u64, coverage_bps: u32, reporting_period_start: u64, reporting_period_end: u64` | `Result<(), Error>` | Report vaccination coverage with privacy preservation |
| `report_environmental_health` | `env: Env, monitoring_station: Address, env_data_id: BytesN<32>, encrypted_location: Bytes, metric_type: String, aggregated_value: u64, risk_bps: u32, measurement_period_start: u64, measurement_period_end: u64, aggregation_method: AggregationMethod, privacy_epsilon: u64` | `Result<(), Error>` | Report environmental health data |
| `report_antimicrobial_resistance` | `env: Env, testing_lab: Address, amr_data_id: BytesN<32>, encrypted_region: Bytes, pathogen_code: String, antibiotic_class: String, resistance_bps: u32, private_sample_size: u64, aggregation_method: AggregationMethod, privacy_epsilon: u64` | `Result<(), Error>` | Report antimicrobial resistance data |
| `report_social_determinants` | `env: Env, data_source: Address, sdoh_data_id: BytesN<32>, encrypted_region: Bytes, determinant_type: String, aggregated_metric: u64, impact_bps: u32, aggregation_method: AggregationMethod, privacy_epsilon: u64` | `Result<(), Error>` | Report social determinants of health data |
| `create_intervention` | `env: Env, coordinator: Address, intervention_id: BytesN<32>, intervention_type: String, encrypted_target_population: Bytes, encrypted_scope: Bytes, start_date: u64, end_date: u64, implementation_cost: u64, expected_outcomes: Vec<String>, aggregation_method: AggregationMethod` | `Result<(), Error>` | Create public health intervention |
| `create_global_collaboration` | `env: Env, lead_organization: Address, collaboration_id: BytesN<32>, participants: Vec<Address>, collaboration_type: String, data_sharing_protocol: String, exchange_method: AggregationMethod, objectives: Vec<String>, start_date: u64, end_date: u64` | `Result<(), Error>` | Create global health collaboration |
| `get_outbreak_data` | `env: Env, data_id: BytesN<32>` | `Result<OutbreakData, Error>` | Get outbreak data |
| `get_epidemic_model` | `env: Env, model_id: BytesN<32>` | `Result<EpidemicModel, Error>` | Get epidemic model |
| `get_public_health_alert` | `env: Env, alert_id: u64` | `Result<PublicHealthAlert, Error>` | Get public health alert |
| `get_vaccination_coverage` | `env: Env, coverage_id: BytesN<32>` | `Result<VaccinationCoverage, Error>` | Get vaccination coverage |
| `get_environmental_health` | `env: Env, env_data_id: BytesN<32>` | `Result<EnvironmentalHealth, Error>` | Get environmental health data |
| `get_antimicrobial_resistance` | `env: Env, amr_data_id: BytesN<32>` | `Result<AntimicrobialResistance, Error>` | Get antimicrobial resistance data |
| `get_social_determinants` | `env: Env, sdoh_data_id: BytesN<32>` | `Result<SocialHealthDeterminants, Error>` | Get social determinants of health data |
| `get_public_health_intervention` | `env: Env, intervention_id: BytesN<32>` | `Result<PublicHealthIntervention, Error>` | Get public health intervention |
| `get_global_collaboration` | `env: Env, collaboration_id: BytesN<32>` | `Result<GlobalHealthCollaboration, Error>` | Get global health collaboration |
| `get_privacy_budget` | `env: Env, user: Address` | `Result<u64, Error>` | Get privacy budget for address |

### Types

#### `enum DiseaseSeverity`

| Variant | Value | Description |
|---|---|---|
| `Low` | 1 | — |
| `Medium` | 2 | — |
| `High` | 3 | — |
| `Critical` | 4 | — |

#### `enum AlertType`

| Variant | Value | Description |
|---|---|---|
| `DiseaseOutbreak` | — | — |
| `EnvironmentalHazard` | — | — |
| `VaccineShortage` | — | — |
| `AntimicrobialResistance` | — | — |
| `SupplyChainDisruption` | — | — |
| `EmergingPathogen` | — | — |
| `SeasonalEpidemic` | — | — |

#### `enum OutbreakStatus`

| Variant | Value | Description |
|---|---|---|
| `Monitoring` | — | — |
| `Detected` | — | — |
| `Investigating` | — | — |
| `Confirmed` | — | — |
| `Contained` | — | — |
| `Resolved` | — | — |

#### `enum AggregationMethod`

| Variant | Value | Description |
|---|---|---|
| `DifferentialPrivacy` | — | — |
| `SecureMultipartyComputation` | — | — |
| `HomomorphicEncryption` | — | — |
| `ZeroKnowledgeProofs` | — | — |
| `FederatedLearning` | — | — |

#### `struct OutbreakData`

| Field | Type | Description |
|---|---|---|
| `data_id` | `BytesN<32>` | — |
| `encrypted_region` | `Bytes` | — |
| `disease_code` | `String` | — |
| `aggregated_cases` | `u64` | — |
| `time_period_start` | `u64` | — |
| `time_period_end` | `u64` | — |
| `aggregation_method` | `AggregationMethod` | — |
| `privacy_epsilon` | `u64` | — |
| `confidence_bps` | `u32` | — |
| `provider` | `Address` | — |
| `reported_at` | `u64` | — |

#### `struct EpidemicModel`

| Field | Type | Description |
|---|---|---|
| `model_id` | `BytesN<32>` | — |
| `disease_code` | `String` | — |
| `encrypted_scope` | `Bytes` | — |
| `model_type` | `String` | — |
| `r0_estimate` | `u64` | — |
| `incubation_days` | `u32` | — |
| `infectious_days` | `u32` | — |
| `case_fatality_bps` | `u32` | — |
| `encrypted_params` | `Bytes` | — |
| `prediction_horizon` | `u32` | — |
| `confidence_bps` | `u32` | — |
| `last_updated` | `u64` | — |
| `creator` | `Address` | — |

#### `struct PublicHealthAlert`

| Field | Type | Description |
|---|---|---|
| `alert_id` | `u64` | — |
| `alert_type` | `AlertType` | — |
| `severity` | `DiseaseSeverity` | — |
| `encrypted_affected_regions` | `Bytes` | — |
| `message` | `String` | — |
| `recommended_actions` | `Vec<String>` | — |
| `source` | `Address` | — |
| `created_at` | `u64` | — |
| `expires_at` | `u64` | — |
| `is_active` | `bool` | — |
| `acknowledgment_count` | `u32` | — |

#### `struct VaccinationCoverage`

| Field | Type | Description |
|---|---|---|
| `coverage_id` | `BytesN<32>` | — |
| `encrypted_region` | `Bytes` | — |
| `vaccine_type` | `String` | — |
| `encrypted_target_population` | `u64` | — |
| `private_vaccinated_count` | `u64` | — |
| `coverage_bps` | `u32` | — |
| `aggregation_method` | `AggregationMethod` | — |
| `privacy_epsilon` | `u64` | — |
| `reporting_period_start` | `u64` | — |
| `reporting_period_end` | `u64` | — |
| `provider` | `Address` | — |
| `reported_at` | `u64` | — |

#### `struct EnvironmentalHealth`

| Field | Type | Description |
|---|---|---|
| `env_data_id` | `BytesN<32>` | — |
| `encrypted_location` | `Bytes` | — |
| `metric_type` | `String` | — |
| `aggregated_value` | `u64` | — |
| `risk_bps` | `u32` | — |
| `measurement_period_start` | `u64` | — |
| `measurement_period_end` | `u64` | — |
| `aggregation_method` | `AggregationMethod` | — |
| `privacy_epsilon` | `u64` | — |
| `monitoring_station` | `Address` | — |
| `measured_at` | `u64` | — |

#### `struct AntimicrobialResistance`

| Field | Type | Description |
|---|---|---|
| `amr_data_id` | `BytesN<32>` | — |
| `encrypted_region` | `Bytes` | — |
| `pathogen_code` | `String` | — |
| `antibiotic_class` | `String` | — |
| `resistance_bps` | `u32` | — |
| `private_sample_size` | `u64` | — |
| `aggregation_method` | `AggregationMethod` | — |
| `privacy_epsilon` | `u64` | — |
| `testing_lab` | `Address` | — |
| `tested_at` | `u64` | — |

#### `struct SocialHealthDeterminants`

| Field | Type | Description |
|---|---|---|
| `sdoh_data_id` | `BytesN<32>` | — |
| `encrypted_region` | `Bytes` | — |
| `determinant_type` | `String` | — |
| `aggregated_metric` | `u64` | — |
| `impact_bps` | `u32` | — |
| `aggregation_method` | `AggregationMethod` | — |
| `privacy_epsilon` | `u64` | — |
| `data_source` | `Address` | — |
| `collected_at` | `u64` | — |

#### `struct PublicHealthIntervention`

| Field | Type | Description |
|---|---|---|
| `intervention_id` | `BytesN<32>` | — |
| `intervention_type` | `String` | — |
| `encrypted_target_population` | `Bytes` | — |
| `encrypted_scope` | `Bytes` | — |
| `start_date` | `u64` | — |
| `end_date` | `u64` | — |
| `implementation_cost` | `u64` | — |
| `expected_outcomes` | `Vec<String>` | — |
| `effectiveness_bps` | `u32` | — |
| `aggregation_method` | `AggregationMethod` | — |
| `coordinator` | `Address` | — |
| `created_at` | `u64` | — |

#### `struct GlobalHealthCollaboration`

| Field | Type | Description |
|---|---|---|
| `collaboration_id` | `BytesN<32>` | — |
| `participants` | `Vec<Address>` | — |
| `collaboration_type` | `String` | — |
| `data_sharing_protocol` | `String` | — |
| `exchange_method` | `AggregationMethod` | — |
| `objectives` | `Vec<String>` | — |
| `start_date` | `u64` | — |
| `end_date` | `u64` | — |
| `lead_organization` | `Address` | — |
| `established_at` | `u64` | — |

#### `enum DataKey`

| Variant | Value | Description |
|---|---|---|
| `Initialized` | — | — |
| `Admin` | — | — |
| `OutbreakData(BytesN<32>)` | — | — |
| `EpidemicModel(BytesN<32>)` | — | — |
| `PublicHealthAlert(u64)` | — | — |
| `VaccinationCoverage(BytesN<32>)` | — | — |
| `EnvironmentalHealth(BytesN<32>)` | — | — |
| `AntimicrobialResistance(BytesN<32>)` | — | — |
| `SocialHealthDeterminants(BytesN<32>)` | — | — |
| `PublicHealthIntervention(BytesN<32>)` | — | — |
| `GlobalHealthCollaboration(BytesN<32>)` | — | — |
| `AlertCounter` | — | — |
| `ModelCounter` | — | — |
| `CoverageCounter` | — | — |
| `InterventionCounter` | — | — |
| `CollaborationCounter` | — | — |
| `PrivacyBudget(Address)` | — | — |

#### `enum Error`

| Variant | Value | Description |
|---|---|---|
| `AlreadyInitialized` | 1 | — |
| `NotInitialized` | 2 | — |
| `NotAuthorized` | 3 | — |
| `InvalidInput` | 4 | — |
| `DataNotFound` | 5 | — |
| `InvalidAggregationMethod` | 6 | — |
| `PrivacyBudgetExceeded` | 7 | — |
| `InsufficientPrivilege` | 8 | — |
| `InvalidSeverity` | 9 | — |
| `AlertExpired` | 10 | — |
| `ModelNotFound` | 11 | — |
| `InterventionNotFound` | 12 | — |
| `CollaborationNotFound` | 13 | — |
| `InvalidTimeRange` | 14 | — |
| `InvalidRegion` | 15 | — |

### Examples

#### `test_public_health_surveillance_initialization`

```rust
let env = Env::default();
    env.mock_all_auths();

    let (client, _id) = setup(&env);
    let admin = Address::generate(&env);

    client.initialize(&admin);

    // Test that initialization works
```

#### `test_outbreak_data_reporting`

```rust
let env = Env::default();
    env.mock_all_auths();

    let (client, _id) = setup(&env);
    let admin = Address::generate(&env);
    client.initialize(&admin);

    let provider = Address::generate(&env);
    let data_id = BytesN::from_array(&env, &[1u8; 32]);
```

#### `test_epidemic_model_creation`

```rust
let env = Env::default();
    env.mock_all_auths();

    let (client, _id) = setup(&env);
    let admin = Address::generate(&env);
    client.initialize(&admin);

    let modeler = Address::generate(&env);
    let model_id = BytesN::from_array(&env, &[2u8; 32]);
```

---

## rbac

### Functions

| Function | Parameters | Returns | Description |
|---|---|---|---|
| `initialize` | `env: Env, admin: Address, config: RBACConfig` | `Result<(), Error>` | — |
| `assign_role` | `env: Env, address: Address, role: Role` | `Result<bool, Error>` | — |
| `remove_role` | `env: Env, address: Address, role: Role` | `Result<bool, Error>` | — |
| `has_role` | `env: Env, address: Address, role: Role` | `Result<bool, Error>` | — |
| `get_roles` | `env: Env, address: Address` | `Result<Vec<Role>, Error>` | — |
| `has_any_role` | `env: Env, address: Address, roles: Vec<Role>` | `Result<bool, Error>` | — |
| `has_all_roles` | `env: Env, address: Address, roles: Vec<Role>` | `Result<bool, Error>` | — |
| `get_address_roles` | `env: Env, address: Address` | `Result<types::AddressRoles, Error>` | — |
| `get_role_members` | `env: Env, role: Role` | `Result<Vec<Address>, Error>` | — |
| `get_role_member_count` | `env: Env, role: Role` | `Result<u32, Error>` | — |
| `is_admin` | `env: Env, address: Address` | `Result<bool, Error>` | — |
| `is_doctor` | `env: Env, address: Address` | `Result<bool, Error>` | — |
| `is_patient` | `env: Env, address: Address` | `Result<bool, Error>` | — |
| `is_staff` | `env: Env, address: Address` | `Result<bool, Error>` | — |
| `update_config` | `env: Env, config: RBACConfig` | `Result<(), Error>` | — |
| `get_config` | `env: Env` | `Result<RBACConfig, Error>` | — |

### Examples

#### `test_initialize`

```rust
let env = Env::default();
        let contract_id = env.register_contract(None, RBAC);
        let client = RBACClient::new(&env, &contract_id);
        let admin = Address::generate(&env);
        let config = RBACConfig {
            emit_events: true,
            max_roles_per_address: 10,
        };
```

---

## regional_node_manager

### Functions

| Function | Parameters | Returns | Description |
|---|---|---|---|
| `initialize` | `env: Env, admin: Address` | `Result<(), Error>` | — |
| `assign_role` | `env: Env, caller: Address, user: Address, role_mask: u32` | `Result<(), Error>` | — |
| `register_node` | `env: Env, caller: Address, region_name: String` | `Result<u32, Error>` | — |
| `get_node` | `env: Env, node_id: u32` | `Option<RegionalNode>` | — |
| `list_nodes` | `env: Env` | `Vec<RegionalNode>` | — |
| `update_node_metrics` | `env: Env, caller: Address, node_id: u32, cpu_usage: u32, memory_usage: u32, disk_usage: u32, replica_lag_ms: u64` | `Result<(), Error>` | — |
| `perform_health_check` | `env: Env, caller: Address, node_id: u32` | `Result<u64, Error>` | — |
| `get_health_checks` | `env: Env` | `Vec<HealthCheckResult>` | — |
| `get_recent_health_check` | `env: Env, node_id: u32` | `Option<HealthCheckResult>` | — |
| `register_replica` | `env: Env, caller: Address, replica_id: u32, node_id: u32, data_hash: u64` | `Result<(), Error>` | — |
| `update_replica_sync` | `env: Env, caller: Address, replica_id: u32, lag_ms: u64` | `Result<(), Error>` | — |
| `get_replicas_for_node` | `env: Env, node_id: u32` | `Vec<ReplicaInfo>` | — |
| `set_configuration` | `env: Env, caller: Address, config: NodeConfiguration` | `Result<(), Error>` | — |
| `get_configuration` | `env: Env` | `NodeConfiguration` | — |

### Types

#### `enum NodeStatus`

| Variant | Value | Description |
|---|---|---|
| `Healthy` | 0 | — |
| `Degraded` | 1 | — |
| `Unhealthy` | 2 | — |
| `Unreachable` | 3 | — |

#### `struct RegionalNode`

| Field | Type | Description |
|---|---|---|
| `node_id` | `u32` | — |
| `region_name` | `String` | — |
| `status` | `NodeStatus` | — |
| `cpu_usage_percent` | `u32` | — |
| `memory_usage_percent` | `u32` | — |
| `disk_usage_percent` | `u32` | — |
| `last_heartbeat` | `u64` | — |
| `replica_lag_ms` | `u64` | — |
| `total_uptime_ms` | `u64` | — |
| `failure_count` | `u32` | — |

#### `struct HealthCheckResult`

| Field | Type | Description |
|---|---|---|
| `check_id` | `u64` | — |
| `node_id` | `u32` | — |
| `checked_at` | `u64` | — |
| `status` | `NodeStatus` | — |
| `cpu_usage` | `u32` | — |
| `memory_usage` | `u32` | — |
| `disk_usage` | `u32` | — |
| `response_time_ms` | `u64` | — |

#### `struct ReplicaInfo`

| Field | Type | Description |
|---|---|---|
| `replica_id` | `u32` | — |
| `node_id` | `u32` | — |
| `data_hash` | `u64` | — |
| `last_synced` | `u64` | — |
| `lag_ms` | `u64` | — |
| `is_in_sync` | `bool` | — |

#### `struct NodeConfiguration`

| Field | Type | Description |
|---|---|---|
| `max_cpu_threshold` | `u32` | — |
| `max_memory_threshold` | `u32` | — |
| `max_disk_threshold` | `u32` | — |
| `max_replica_lag_ms` | `u64` | — |
| `heartbeat_timeout_ms` | `u64` | — |
| `health_check_interval_ms` | `u64` | — |

#### `enum Error`

| Variant | Value | Description |
|---|---|---|
| `AlreadyInitialized` | 1 | — |
| `NotInitialized` | 2 | — |
| `NotAuthorized` | 3 | — |
| `InvalidInput` | 4 | — |
| `NodeNotFound` | 5 | — |
| `HealthCheckFailed` | 6 | — |
| `ReplicaOutOfSync` | 7 | — |
| `NodeUnreachable` | 8 | — |
| `InvalidThreshold` | 9 | — |
| `DuplicateNode` | 10 | — |

---

## regulatory_compliance

### Functions

| Function | Parameters | Returns | Description |
|---|---|---|---|
| `initialize` | `env: Env, admin: Address` | `Result<(), Error>` | — |
| `set_rule` | `env: Env, framework: String, rule: ComplianceRule` | `Result<(), Error>` | — |
| `get_rule` | `env: Env, framework: String` | `Option<ComplianceRule>` | — |
| `grant_consent` | `env: Env, user: Address, action: String` | `Result<(), Error>` | — |
| `revoke_consent` | `env: Env, user: Address, action: String` | `Result<(), Error>` | — |
| `has_consent` | `env: Env, user: Address, action: String` | `bool` | — |
| `log_audit` | `env: Env, user: Address, action: String, details: String` | `()` | — |
| `get_audit_logs` | `env: Env, user: Address` | `Result<Vec<AuditLog>, Error>` | — |
| `invoke_right_to_be_forgotten` | `env: Env, user: Address` | `Result<(), Error>` | — |
| `is_forgotten` | `env: &Env, user: Address` | `bool` | — |

### Types

#### `enum Error`

| Variant | Value | Description |
|---|---|---|
| `NotInitialized` | 1 | — |
| `AlreadyInitialized` | 2 | — |
| `UserAlreadyForgotten` | 3 | — |
| `RuleNotConfigured` | 4 | — |
| `RightToBeForgottenDisabled` | 5 | — |

#### `enum DataResidency`

| Variant | Value | Description |
|---|---|---|
| `Global` | — | — |
| `EU` | — | — |
| `US` | — | — |
| `Local(String)` | — | — |

#### `struct ComplianceRule`

| Field | Type | Description |
|---|---|---|
| `require_consent` | `bool` | — |
| `right_to_be_forgotten` | `bool` | — |
| `residency` | `DataResidency` | — |
| `strict_auditing` | `bool` | — |

#### `struct AuditLog`

| Field | Type | Description |
|---|---|---|
| `action` | `String` | — |
| `actor` | `Address` | — |
| `timestamp` | `u64` | — |
| `details` | `String` | — |

#### `enum DataKey`

| Variant | Value | Description |
|---|---|---|
| `Admin` | — | — |
| `Rule(String)` | — | — |
| `Consent(Address, String)` | — | — |
| `AuditLogs(Address)` | — | — |
| `Forgotten(Address)` | — | — |

---

## remote_patient_monitoring

### Functions

| Function | Parameters | Returns | Description |
|---|---|---|---|
| `initialize` | `env: Env, admin: Address` | `()` | — |
| `register_device` | `env: Env, caller: Address, device_id: u64, device_type: u32, patient: Address, connectivity: Vec<String>` | `()` | — |
| `add_caregiver` | `env: Env, caller: Address, device_id: u64, caregiver: Address` | `()` | — |
| `submit_vital_sign` | `env: Env, caller: Address, patient: Address, device_id: u64, vital_type: String, value: i64, unit: String, quality: u32` | `()` | — |
| `set_threshold` | `env: Env, caller: Address, patient: Address, vital_type: String, min_value: i64, max_value: i64, alert_severity: u32` | `()` | — |
| `update_battery_level` | `env: Env, caller: Address, device_id: u64, battery_level: u32` | `()` | — |
| `get_device` | `env: Env, device_id: u64` | `Option<Device>` | — |
| `get_vitals` | `_env: Env, _patient: Address, _limit: u32` | `Vec<VitalSign>` | — |
| `get_alerts` | `_env: Env, _patient: Address, _limit: u32` | `Vec<Alert>` | — |
| `get_caregiver_alerts` | `_env: Env, _caregiver: Address` | `Vec<Alert>` | — |

### Types

#### `struct Device`

| Field | Type | Description |
|---|---|---|
| `id` | `u64` | — |
| `device_type` | `u32` | — |
| `1` | `HeartRateMonitor` | — |
| `2` | `GlucoseMeter` | — |
| `patient` | `Address` | — |
| `caregivers` | `Vec<Address>` | — |
| `connectivity` | `Vec<String>` | — |
| `battery_level` | `Option<u32>` | — |

#### `struct VitalSign`

| Field | Type | Description |
|---|---|---|
| `patient` | `Address` | — |
| `device_id` | `u64` | — |
| `timestamp` | `u64` | — |
| `vital_type` | `String` | — |
| `value` | `i64` | — |
| `unit` | `String` | — |
| `quality` | `u32` | — |

#### `struct Alert`

| Field | Type | Description |
|---|---|---|
| `patient` | `Address` | — |
| `alert_type` | `u32` | — |
| `1` | `DeviceOffline` | — |
| `2` | `BatteryLow` | — |
| `3` | `AbnormalReading pub message: String` | — |
| `timestamp` | `u64` | — |
| `severity` | `u32` | — |

#### `struct Threshold`

| Field | Type | Description |
|---|---|---|
| `vital_type` | `String` | — |
| `min_value` | `i64` | — |
| `max_value` | `i64` | — |
| `alert_severity` | `u32` | — |

---

## reputation

### Functions

| Function | Parameters | Returns | Description |
|---|---|---|---|
| `initialize` | `env: Env, admin: Address` | `Result<(), Error>` | — |
| `get_score` | `env: Env, user: Address` | `i128` | — |
| `mint` | `env: Env, user: Address, amount: i128` | `Result<(), Error>` | — |
| `slash` | `env: Env, user: Address, amount: i128` | `Result<(), Error>` | — |

### Types

#### `enum Error`

| Variant | Value | Description |
|---|---|---|
| `AlreadyInitialized` | 1 | — |
| `NotInitialized` | 2 | — |
| `NegativeAmount` | 3 | — |
| `InvalidAmount` | 4 | — |

---

## reputation_access_control

### Functions

| Function | Parameters | Returns | Description |
|---|---|---|---|
| `initialize` | `env: Env, admin: Address, _reputation_contract: Address` | `Result<(), Error>` | — |
| `set_access_policy` | `env: Env, admin: Address, resource_type: ResourceType, policy: AccessPolicy` | `Result<(), Error>` | — |
| `check_access` | `env: Env, provider: Address, resource_type: ResourceType, requested_access: AccessLevel` | `Result<bool, Error>` | — |
| `request_access` | `env: Env, provider: Address, resource_type: ResourceType, requested_access: AccessLevel, justification: String` | `Result<BytesN<32>, Error>` | — |
| `approve_request` | `env: Env, admin: Address, request_id: BytesN<32>` | `Result<(), Error>` | — |
| `deny_request` | `env: Env, admin: Address, request_id: BytesN<32>` | `Result<(), Error>` | — |
| `grant_emergency_access` | `env: Env, admin: Address, provider: Address, _duration_hours: u32` | `Result<(), Error>` | — |
| `revoke_emergency_access` | `env: Env, admin: Address, provider: Address` | `Result<(), Error>` | — |
| `get_provider_access_level` | `env: Env, provider: Address, resource_type: ResourceType` | `Result<AccessLevel, Error>` | — |
| `get_provider_requests` | `env: Env, provider: Address` | `Result<Vec<AccessRequest>, Error>` | — |
| `set_reputation_threshold` | `env: Env, admin: Address, resource_type: ResourceType, threshold: u32` | `Result<(), Error>` | — |

### Types

#### `enum Error`

| Variant | Value | Description |
|---|---|---|
| `AlreadyInitialized` | 1 | — |
| `NotInitialized` | 2 | — |
| `NotAuthorized` | 3 | — |
| `InsufficientReputation` | 4 | — |
| `AccessDenied` | 5 | — |
| `InvalidResource` | 6 | — |
| `PolicyNotFound` | 7 | — |
| `ProviderNotVerified` | 8 | — |
| `CredentialExpired` | 9 | — |

#### `enum ResourceType`

| Variant | Value | Description |
|---|---|---|
| `PatientRecords` | 0 | — |
| `MedicalPrescriptions` | 1 | — |
| `DiagnosticReports` | 2 | — |
| `SurgicalProcedures` | 3 | — |
| `EmergencyAccess` | 4 | — |
| `ResearchData` | 5 | — |
| `AdministrativeFunctions` | 6 | — |
| `ProviderDirectory` | 7 | — |
| `CredentialManagement` | 8 | — |

#### `enum AccessLevel`

| Variant | Value | Description |
|---|---|---|
| `None` | 0 | — |
| `Read` | 1 | — |
| `Write` | 2 | — |
| `Update` | 3 | — |
| `Delete` | 4 | — |
| `Admin` | 5 | — |

#### `enum TimeRestrictionPolicy`

| Variant | Value | Description |
|---|---|---|
| `None` | — | — |
| `Restricted(TimeRestriction)` | — | — |

#### `struct AccessPolicy`

| Field | Type | Description |
|---|---|---|
| `resource_type` | `ResourceType` | — |
| `min_reputation_score` | `u32` | — |
| `required_credentials` | `Vec<Symbol>` | — |
| `access_level` | `AccessLevel` | — |
| `time_restriction` | `TimeRestrictionPolicy` | — |
| `special_conditions` | `Vec<Symbol>` | — |

#### `struct TimeRestriction`

| Field | Type | Description |
|---|---|---|
| `start_hour` | `u32` | — |
| `end_hour` | `u32` | — |
| `allowed_days` | `Vec<u32>` | — |
| `timezone` | `String` | — |

#### `struct AccessRequest`

| Field | Type | Description |
|---|---|---|
| `request_id` | `BytesN<32>` | — |
| `provider` | `Address` | — |
| `resource_type` | `ResourceType` | — |
| `requested_access` | `AccessLevel` | — |
| `timestamp` | `u64` | — |
| `justification` | `String` | — |
| `status` | `RequestStatus` | — |

#### `enum RequestStatus`

| Variant | Value | Description |
|---|---|---|
| `Pending` | 0 | — |
| `Approved` | 1 | — |
| `Denied` | 2 | — |
| `Expired` | 3 | — |

#### `enum DataKey`

| Variant | Value | Description |
|---|---|---|
| `Admin` | — | — |
| `Initialized` | — | — |
| `AccessPolicy(ResourceType)` | — | — |
| `AccessRequest(BytesN<32>)` | — | — |
| `ProviderRequests(Address)` | — | — |
| `ProviderAccessLevel(Address, ResourceType)` | — | — |
| `ReputationThreshold(ResourceType)` | — | — |
| `EmergencyAccess(Address)` | — | — |

---

## reputation_integration

### Functions

| Function | Parameters | Returns | Description |
|---|---|---|---|
| `initialize` | `env: Env, admin: Address, base_reputation_contract: Address, healthcare_reputation_contract: Address` | `Result<(), Error>` | — |
| `sync_provider_reputation` | `env: Env, admin: Address, provider: Address` | `Result<i128, Error>` | — |
| `batch_sync_providers` | `env: Env, admin: Address, providers: Vec<Address>` | `Result<Vec<i128>, Error>` | — |
| `auto_sync_all_providers` | `env: Env, admin: Address` | `Result<u32, Error>` | — |
| `update_score_mapping` | `env: Env, admin: Address, base_weight: u32, healthcare_weight: u32, adjustment_factor: i32` | `Result<(), Error>` | — |
| `update_sync_settings` | `env: Env, admin: Address, settings: SyncSettings` | `Result<(), Error>` | — |
| `get_combined_score` | `env: Env, provider: Address` | `Result<i128, Error>` | — |
| `get_sync_history` | `env: Env, provider: Address, limit: u32` | `Result<Vec<SyncRecord>, Error>` | — |
| `trigger_credential_sync` | `env: Env, provider: Address` | `Result<(), Error>` | — |
| `trigger_feedback_sync` | `env: Env, provider: Address` | `Result<(), Error>` | — |
| `trigger_conduct_sync` | `env: Env, provider: Address` | `Result<(), Error>` | — |

### Types

#### `enum Error`

| Variant | Value | Description |
|---|---|---|
| `AlreadyInitialized` | 1 | — |
| `NotInitialized` | 2 | — |
| `NotAuthorized` | 3 | — |
| `ProviderNotFound` | 4 | — |
| `ReputationContractNotFound` | 5 | — |
| `HealthcareReputationContractNotFound` | 6 | — |
| `InvalidScoreMapping` | 7 | — |
| `SyncFailed` | 8 | — |

#### `struct ScoreMapping`

| Field | Type | Description |
|---|---|---|
| `base_reputation_weight` | `u32` | — |
| `healthcare_reputation_weight` | `u32` | — |
| `adjustment_factor` | `i32` | — |
| `last_sync_timestamp` | `u64` | — |

#### `struct SyncRecord`

| Field | Type | Description |
|---|---|---|
| `provider` | `Address` | — |
| `base_score` | `i128` | — |
| `healthcare_score` | `u32` | — |
| `combined_score` | `i128` | — |
| `timestamp` | `u64` | — |
| `sync_type` | `SyncType` | — |

#### `enum SyncType`

| Variant | Value | Description |
|---|---|---|
| `Manual` | 0 | — |
| `Automatic` | 1 | — |
| `CredentialUpdate` | 2 | — |
| `FeedbackUpdate` | 3 | — |
| `ConductUpdate` | 4 | — |

#### `enum DataKey`

| Variant | Value | Description |
|---|---|---|
| `Admin` | — | — |
| `Initialized` | — | — |
| `BaseReputationContract` | — | — |
| `HealthcareReputationContract` | — | — |
| `ScoreMapping` | — | — |
| `SyncRecord(Address, u64)` | — | — |
| `timestamp
    ProviderSyncList(Address)` | — | — |
| `LastSyncTime(Address)` | — | — |
| `SyncSettings` | — | — |

#### `struct SyncSettings`

| Field | Type | Description |
|---|---|---|
| `auto_sync_enabled` | `bool` | — |
| `sync_interval_hours` | `u32` | — |
| `sync_on_credential_change` | `bool` | — |
| `sync_on_feedback_change` | `bool` | — |
| `sync_on_conduct_change` | `bool` | — |

---

## runtime_validation

### Functions

| Function | Parameters | Returns | Description |
|---|---|---|---|
| `initialize` | `env: Env, admin: Address` | `Result<(), Error>` | Initialize the runtime validation system |
| `register_invariant` | `env: Env, admin: Address, check_id: String, description: String, severity: u32, // 1=low, 2=medium, 3=high, 4=critical` | `Result<(), Error>` | Register an invariant check |
| `register_state_check` | `env: Env, admin: Address, check_id: String, description: String, expected_state: String` | `Result<(), Error>` | Register a state consistency check |
| `register_permission_check` | `env: Env, admin: Address, check_id: String, description: String, required_role: String` | `Result<(), Error>` | Register a permission check |
| `register_resource_tracker` | `env: Env, admin: Address, tracker_id: String, resource_type: String, max_allocation: i128` | `Result<(), Error>` | Register a resource tracker |
| `report_violation` | `env: Env, reporter: Address, check_id: String, violation_type: ViolationType, details: String` | `Result<u64, Error>` | Report a validation violation |
| `verify_invariant` | `env: Env, check_id: String, current_value: i128, expected_range_min: i128, expected_range_max: i128` | `Result<bool, Error>` | Verify an invariant check |
| `verify_state_consistency` | `env: Env, check_id: String, current_state: String` | `Result<bool, Error>` | Verify state consistency |
| `verify_permission` | `env: Env, check_id: String, user_role: String` | `Result<bool, Error>` | Check permission |
| `update_resource_usage` | `env: Env, tracker_id: String, usage_delta: i128` | `Result<(), Error>` | Update resource usage |
| `get_violation_report` | `env: Env, violation_id: u64` | `Result<ValidationReport, Error>` | Get validation report |
| `get_violation_count` | `env: Env` | `u64` | Get total violations |

### Error Codes

| Variant | Code | Description |
|---|---|---|
| `NotInitialized` | 1 | — |
| `AlreadyInitialized` | 2 | — |
| `NotAuthorized` | 3 | — |
| `CheckNotFound` | 4 | — |
| `CheckAlreadyExists` | 5 | — |
| `CheckNotActive` | 6 | — |
| `InvalidSeverity` | 7 | — |
| `InvalidResourceLimit` | 8 | — |
| `ResourceLimitExceeded` | 9 | — |
| `ViolationNotFound` | 10 | — |

### Examples

#### `test_initialize`

```rust
let env = Env::default();
        env.mock_all_auths();

        let admin = Address::generate(&env);
        let contract_id = env.register_contract(None, RuntimeValidation);
        let client = RuntimeValidationClient::new(&env, &contract_id);

        client.initialize(&admin);
    }
```

---

## sanitization

### Functions

| Function | Parameters | Returns | Description |
|---|---|---|---|
| `sanitize_string` | `_env: &Env, input: &String, max_len: u32` | `Result<(), SanitizationError>` | Validates a general-purpose string: non-empty, within `max_len` bytes, no null bytes, no ASCII control characters (allows tab/LF/CR). |
| `sanitize_name` | `_env: &Env, input: &String` | `Result<(), SanitizationError>` | Validates a human name: letters (any UTF-8), digits, spaces, hyphens, apostrophes, commas, and periods only (ASCII subset). |
| `sanitize_email` | `_env: &Env, input: &String` | `Result<(), SanitizationError>` | Validates an email address: single '@', non-empty local and domain parts, domain contains at least one '.', all chars from the RFC 5321 allowed set. |
| `sanitize_id` | `_env: &Env, input: &String` | `Result<(), SanitizationError>` | Validates an identifier: alphanumeric chars, hyphens, underscores, colons, dots, and forward slashes (covers DIDs, slugs, and resource paths). |
| `sanitize_url` | `_env: &Env, input: &String` | `Result<(), SanitizationError>` | Validates a URL: printable ASCII only, length within MAX_URL_LEN. |

### Types

#### `enum SanitizationError`

| Variant | Value | Description |
|---|---|---|
| `InputTooLong` | 1 | — |
| `EmptyInput` | 2 | — |
| `NullByte` | 3 | — |
| `InvalidCharacter` | 4 | — |
| `InvalidFormat` | 5 | — |

---

## secure_enclave

### Functions

| Function | Parameters | Returns | Description |
|---|---|---|---|
| `initialize` | `env: Env, admin: Address` | `()` | — |
| `register_enclave` | `env: Env, caller: Address, node_id: BytesN<32>, provider: CloudProvider, quote: Bytes, public_key: BytesN<32>` | `()` | — |
| `verify_attestation` | `env: Env, admin: Address, node_id: BytesN<32>, is_valid: bool` | `()` | — |
| `submit_task` | `env: Env, submitter: Address, task_id: BytesN<32>, payload_hash: BytesN<32>, require_zk_proof: bool` | `()` | — |
| `assign_task` | `env: Env, admin: Address, task_id: BytesN<32>, node_id: BytesN<32>` | `()` | — |
| `complete_task` | `env: Env, node_address: Address, task_id: BytesN<32>, result: Bytes, zk_proof: Option<Bytes>` | `()` | — |
| `fallback_to_mpc` | `env: Env, admin: Address, task_id: BytesN<32>, mpc_manager_id: Address` | `()` | — |

### Types

#### `enum DataKey`

| Variant | Value | Description |
|---|---|---|
| `Admin` | — | — |
| `Node(BytesN<32>)` | — | — |
| `Task(BytesN<32>)` | — | — |
| `NodeList` | — | — |

#### `enum CloudProvider`

| Variant | Value | Description |
|---|---|---|
| `AWSNitro` | — | — |
| `IntelSGX` | — | — |
| `GCPConfidentialSpace` | — | — |

#### `enum EnclaveStatus`

| Variant | Value | Description |
|---|---|---|
| `PendingRegistration` | — | — |
| `Active` | — | — |
| `Compromised` | — | — |
| `Offline` | — | — |

#### `struct EnclaveNode`

| Field | Type | Description |
|---|---|---|
| `provider` | `CloudProvider` | — |
| `quote` | `Bytes` | — |
| `public_key` | `BytesN<32>` | — |
| `status` | `EnclaveStatus` | — |

#### `enum TaskStatus`

| Variant | Value | Description |
|---|---|---|
| `Submitted` | — | — |
| `Processing` | — | — |
| `Completed` | — | — |
| `Failed` | — | — |
| `FallbackMPC` | — | — |

#### `struct ProcessingTask`

| Field | Type | Description |
|---|---|---|
| `submitter` | `Address` | — |
| `payload_hash` | `BytesN<32>` | — |
| `status` | `TaskStatus` | — |
| `result` | `Option<Bytes>` | — |
| `assigned_node` | `Bytes` | — |
| `require_zk_proof` | `bool` | — |

### Examples

#### `test_registration_and_attestation`

```rust
let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let contract_id = env.register_contract(None, SecureEnclaveContract);
    let client = SecureEnclaveContractClient::new(&env, &contract_id);

    client.initialize(&admin);
```

#### `test_submit_and_complete_task`

```rust
let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let contract_id = env.register_contract(None, SecureEnclaveContract);
    let client = SecureEnclaveContractClient::new(&env, &contract_id);

    client.initialize(&admin);
```

---

## storage_cleanup

### Functions

| Function | Parameters | Returns | Description |
|---|---|---|---|
| `initialize` | `env: Env, admin: Address` | `Result<(), Error>` | — |
| `set_paused` | `env: Env, caller: Address, paused: bool` | `Result<(), Error>` | — |
| `set_retention_config` | `env: Env, caller: Address, config: RetentionConfig` | `Result<(), Error>` | — |
| `register_credential` | `env: Env, id: u64, expires_at: u64` | `()` | — |
| `register_audit_log` | `env: Env, id: u64, logged_at: u64` | `()` | — |
| `register_escrow` | `env: Env, id: u64, settled_at: u64` | `()` | — |
| `register_consent` | `env: Env, id: u64, revoked_at: u64` | `()` | — |
| `register_schedule` | `env: Env, id: u64, end_at: u64` | `()` | — |
| `cleanup_expired` | `env: Env, caller: Address, max_items: u32` | `Result<u32, Error>` | Clean up expired items across all categories. Returns total number of items removed. |
| `preview_cleanup` | `env: Env, max_items: u32` | `Result<u32, Error>` | Preview how many items would be cleaned without removing them. |
| `cleanup_credentials` | `env: Env, caller: Address, max_items: u32` | `Result<u32, Error>` | — |
| `cleanup_audit_logs` | `env: Env, caller: Address, max_items: u32` | `Result<u32, Error>` | — |
| `cleanup_escrows` | `env: Env, caller: Address, max_items: u32` | `Result<u32, Error>` | — |
| `cleanup_consents` | `env: Env, caller: Address, max_items: u32` | `Result<u32, Error>` | — |
| `cleanup_schedules` | `env: Env, caller: Address, max_items: u32` | `Result<u32, Error>` | — |
| `get_cleanup_log` | `env: Env` | `Vec<CleanupEntry>` | — |
| `is_paused` | `env: Env` | `bool` | — |

### Types

#### `enum DataKey`

| Variant | Value | Description |
|---|---|---|
| `Admin` | — | — |
| `Paused` | — | — |
| `CredentialIds` | — | — |
| `AuditLogIds` | — | — |
| `EscrowIds` | — | — |
| `ConsentIds` | — | — |
| `ScheduleIds` | — | — |
| `CredentialExpiry(u64)` | — | — |
| `AuditLogExpiry(u64)` | — | — |
| `EscrowSettledAt(u64)` | — | — |
| `ConsentRevokedAt(u64)` | — | — |
| `ScheduleEndAt(u64)` | — | — |
| `CleanupLog` | — | — |
| `RetentionConfig` | — | — |

#### `struct RetentionConfig`

| Field | Type | Description |
|---|---|---|
| `credential_secs` | `u64` | — |
| `audit_log_secs` | `u64` | — |
| `escrow_secs` | `u64` | — |
| `consent_secs` | `u64` | — |
| `schedule_secs` | `u64` | — |

#### `struct CleanupEntry`

| Field | Type | Description |
|---|---|---|
| `timestamp` | `u64` | — |
| `caller` | `Address` | — |
| `category` | `u32` | — |
| `count` | `u32` | — |

#### `enum Error`

| Variant | Value | Description |
|---|---|---|
| `NotInitialized` | 1 | — |
| `AlreadyInitialized` | 2 | — |
| `NotAuthorized` | 3 | — |
| `Paused` | 4 | — |
| `BatchTooLarge` | 5 | — |

---

## sut_token

### Functions

| Function | Parameters | Returns | Description |
|---|---|---|---|
| `initialize` | `env: Env, admin: Address, name: String, symbol: String, decimals: u32, supply_cap: i128` | `Result<(), Error>` | Initialize the token contract |
| `name` | `env: Env` | `Result<String, Error>` | Get token name |
| `symbol` | `env: Env` | `Result<String, Error>` | Get token symbol |
| `decimals` | `env: Env` | `Result<u32, Error>` | Get token decimals |
| `total_supply` | `env: Env` | `Result<i128, Error>` | Get total supply |
| `supply_cap` | `env: Env` | `Result<i128, Error>` | Get supply cap |
| `balance_of` | `env: Env, account: Address` | `i128` | Get balance of an address |
| `allowance` | `env: Env, owner: Address, spender: Address` | `i128` | Get allowance between owner and spender |
| `transfer` | `env: Env, from: Address, to: Address, amount: i128` | `Result<(), Error>` | Transfer tokens |
| `transfer_from` | `env: Env, spender: Address, from: Address, to: Address, amount: i128` | `Result<(), Error>` | Transfer tokens from one address to another (requires allowance) |
| `approve` | `env: Env, owner: Address, spender: Address, amount: i128` | `Result<(), Error>` | Approve spender to spend tokens |
| `mint` | `env: Env, minter: Address, to: Address, amount: i128` | `Result<(), Error>` | Mint new tokens (only by minter) |
| `burn` | `env: Env, minter: Address, from: Address, amount: i128` | `Result<(), Error>` | Burn tokens (only by minter) |
| `add_minter` | `env: Env, minter: Address` | `Result<(), Error>` | Add a new minter (only by admin) |
| `remove_minter` | `env: Env, minter: Address` | `Result<(), Error>` | Remove a minter (only by admin) |
| `is_minter` | `env: Env, address: Address` | `bool` | Check if address is a minter |
| `snapshot` | `env: Env` | `Result<u32, Error>` | Create a snapshot for voting/rewards |
| `balance_of_at` | `env: Env, account: Address, snapshot_id: u32` | `Result<i128, Error>` | Get balance at snapshot |
| `total_supply_at` | `env: Env, snapshot_id: u32` | `Result<i128, Error>` | Get total supply at snapshot |

### Types

#### `enum Error`

| Variant | Value | Description |
|---|---|---|
| `AlreadyInitialized` | 1 | — |
| `NotInitialized` | 2 | — |
| `Unauthorized` | 3 | — |
| `InsufficientBalance` | 4 | — |
| `InsufficientAllowance` | 5 | — |
| `ExceedsSupplyCap` | 6 | — |
| `InvalidAmount` | 7 | — |
| `InvalidAddress` | 8 | — |
| `SnapshotNotFound` | 9 | — |
| `Overflow` | 10 | — |
| `IndexOutOfBounds` | 11 | — |

#### `struct TokenMetadata`

| Field | Type | Description |
|---|---|---|
| `name` | `String` | — |
| `symbol` | `String` | — |
| `decimals` | `u32` | — |

#### `struct TokenInfo`

| Field | Type | Description |
|---|---|---|
| `total_supply` | `i128` | — |
| `supply_cap` | `i128` | — |
| `admin` | `Address` | — |

#### `struct Snapshot`

| Field | Type | Description |
|---|---|---|
| `block_number` | `u32` | — |
| `total_supply` | `i128` | — |

#### `struct Checkpoint`

| Field | Type | Description |
|---|---|---|
| `snapshot_id` | `u32` | — |
| `balance` | `i128` | — |

#### `enum DataKey`

| Variant | Value | Description |
|---|---|---|
| `Metadata` | — | — |
| `TokenInfo` | — | — |
| `Balance(Address)` | — | — |
| `Allowance(Address, Address)` | — | — |
| `spender
    Minter(Address)` | — | — |
| `Snapshot(u32)` | — | — |
| `SnapshotCount` | — | — |
| `UserCheckpoints(Address)` | — | — |
| `UserCheckpointCount(Address)` | — | — |

#### `struct TransferEvent`

| Field | Type | Description |
|---|---|---|
| `from` | `Address` | — |
| `to` | `Address` | — |
| `amount` | `i128` | — |

#### `struct ApprovalEvent`

| Field | Type | Description |
|---|---|---|
| `owner` | `Address` | — |
| `spender` | `Address` | — |
| `amount` | `i128` | — |

#### `struct MintEvent`

| Field | Type | Description |
|---|---|---|
| `to` | `Address` | — |
| `amount` | `i128` | — |

#### `struct BurnEvent`

| Field | Type | Description |
|---|---|---|
| `from` | `Address` | — |
| `amount` | `i128` | — |

#### `struct SnapshotEvent`

| Field | Type | Description |
|---|---|---|
| `id` | `u32` | — |
| `block_number` | `u32` | — |

### Examples

#### `test_initialize`

```rust
let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let contract_id = create_token_contract(&env);

    let (name, symbol, decimals, supply_cap) = initialize_token(&env, &contract_id, &admin);
    let client = SutTokenClient::new(&env, &contract_id);
```

#### `test_initialize_twice_fails`

```rust
let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let contract_id = create_token_contract(&env);
    let client = SutTokenClient::new(&env, &contract_id);

    let name = String::from_str(&env, "Test Token");
    let symbol = String::from_str(&env, "TEST");
```

#### `test_mint`

```rust
let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let user = Address::generate(&env);
    let contract_id = create_token_contract(&env);

    initialize_token(&env, &contract_id, &admin);
    let client = SutTokenClient::new(&env, &contract_id);
```

---

## sync_manager

### Functions

| Function | Parameters | Returns | Description |
|---|---|---|---|
| `initialize` | `env: Env, admin: Address` | `Result<(), Error>` | — |
| `assign_role` | `env: Env, caller: Address, user: Address, role_mask: u32` | `Result<(), Error>` | — |
| `initiate_sync` | `env: Env, caller: Address, source_region_id: u32, target_region_ids: Vec<u32>, data_hash: u64, consistency_level: ConsistencyLevel` | `Result<u64, Error>` | — |
| `execute_sync` | `env: Env, caller: Address, operation_id: u64` | `Result<bool, Error>` | — |
| `retry_sync` | `env: Env, caller: Address, operation_id: u64` | `Result<bool, Error>` | — |
| `get_sync_operation` | `env: Env, operation_id: u64` | `Option<SyncOperation>` | — |
| `list_sync_operations` | `env: Env` | `Vec<SyncOperation>` | — |
| `record_replication_lag` | `env: Env, caller: Address, source_region_id: u32, target_region_id: u32, lag_ms: u64` | `Result<u64, Error>` | — |
| `get_replication_lags` | `env: Env` | `Vec<ReplicationLag>` | — |
| `get_region_lag` | `env: Env, source_region_id: u32, target_region_id: u32` | `Option<ReplicationLag>` | — |
| `detect_sync_conflict` | `env: Env, caller: Address, operation_id: u64, conflicting_regions: Vec<u32>` | `Result<u64, Error>` | — |
| `resolve_conflict` | `env: Env, caller: Address, conflict_id: u64, strategy: u32` | `Result<(), Error>` | — |
| `get_conflicts` | `env: Env` | `Vec<ConflictResolution>` | — |
| `set_sync_policy` | `env: Env, caller: Address, policy: SyncPolicy` | `Result<(), Error>` | — |
| `get_sync_policy` | `env: Env` | `SyncPolicy` | — |

### Types

#### `enum SyncStatus`

| Variant | Value | Description |
|---|---|---|
| `Pending` | 0 | — |
| `InProgress` | 1 | — |
| `Completed` | 2 | — |
| `Failed` | 3 | — |
| `PartialSuccess` | 4 | — |

#### `enum ConsistencyLevel`

| Variant | Value | Description |
|---|---|---|
| `Eventual` | 0 | — |
| `Strong` | 1 | — |
| `Causal` | 2 | — |

#### `struct SyncOperation`

| Field | Type | Description |
|---|---|---|
| `operation_id` | `u64` | — |
| `source_region_id` | `u32` | — |
| `target_region_ids` | `Vec<u32>` | — |
| `data_hash` | `u64` | — |
| `initiated_at` | `u64` | — |
| `completed_at` | `u64` | — |
| `status` | `SyncStatus` | — |
| `consistency_level` | `ConsistencyLevel` | — |
| `retry_count` | `u32` | — |
| `success_count` | `u32` | — |
| `failure_count` | `u32` | — |

#### `struct SyncWindow`

| Field | Type | Description |
|---|---|---|
| `window_id` | `u64` | — |
| `region_id` | `u32` | — |
| `start_ts` | `u64` | — |
| `end_ts` | `u64` | — |
| `data_version` | `u64` | — |
| `checksum` | `u64` | — |
| `is_applied` | `bool` | — |

#### `struct ReplicationLag`

| Field | Type | Description |
|---|---|---|
| `lag_id` | `u64` | — |
| `source_region_id` | `u32` | — |
| `target_region_id` | `u32` | — |
| `lag_ms` | `u64` | — |
| `measured_at` | `u64` | — |
| `acceptable` | `bool` | — |

#### `struct SyncPolicy`

| Field | Type | Description |
|---|---|---|
| `sync_interval_ms` | `u64` | — |
| `max_lag_ms` | `u64` | — |
| `consistency_mode` | `ConsistencyLevel` | — |
| `max_retries` | `u32` | — |
| `auto_sync_enabled` | `bool` | — |
| `conflict_resolution_strategy` | `u32` | — |

#### `struct ConflictResolution`

| Field | Type | Description |
|---|---|---|
| `conflict_id` | `u64` | — |
| `operation_id` | `u64` | — |
| `source_region_id` | `u32` | — |
| `conflicting_regions` | `Vec<u32>` | — |
| `detected_at` | `u64` | — |
| `resolved` | `bool` | — |
| `resolution_strategy` | `u32` | — |

#### `enum Error`

| Variant | Value | Description |
|---|---|---|
| `AlreadyInitialized` | 1 | — |
| `NotInitialized` | 2 | — |
| `NotAuthorized` | 3 | — |
| `InvalidInput` | 4 | — |
| `SyncOperationNotFound` | 5 | — |
| `SyncFailed` | 6 | — |
| `ConflictDetected` | 7 | — |
| `MaxRetriesExceeded` | 8 | — |
| `InconsistentState` | 9 | — |
| `TargetUnavailable` | 10 | — |

---

## telemedicine

### Functions

| Function | Parameters | Returns | Description |
|---|---|---|---|
| `initialize` | `env: Env, admin: Address` | `Result<(), TelemedicineError>` | — |
| `pause` | `env: Env` | `Result<(), TelemedicineError>` | — |
| `unpause` | `env: Env` | `Result<(), TelemedicineError>` | — |
| `register_provider` | `env: &Env, provider_id: BytesN<32>, address: Address, name: String, credentials: BytesN<32>, jurisdictions: Vec<String>, specialty: String, license_expiry: u64` | `Result<(), TelemedicineError>` | — |
| `get_provider` | `env: &Env, provider_id: BytesN<32>` | `Result<Provider, TelemedicineError>` | — |
| `deactivate_provider` | `env: &Env, provider_id: BytesN<32>` | `Result<(), TelemedicineError>` | — |
| `register_patient` | `env: &Env, patient_id: BytesN<32>, address: Address, primary_care_physician: BytesN<32>, jurisdiction: String, contact_info: String, preferred_language: String` | `Result<(), TelemedicineError>` | — |
| `get_patient` | `env: &Env, patient_id: BytesN<32>` | `Result<Patient, TelemedicineError>` | — |
| `grant_consent` | `env: &Env, consent_id: BytesN<32>, patient_id: BytesN<32>, consent_type: ConsentType, scope: String, expiry: Option<u64>` | `Result<(), TelemedicineError>` | — |
| `revoke_consent` | `env: &Env, consent_id: BytesN<32>` | `Result<(), TelemedicineError>` | — |
| `has_valid_consent` | `env: &Env, patient_id: BytesN<32>, consent_type: ConsentType` | `Result<bool, TelemedicineError>` | Returns true only if the patient has at least one active, non-expired consent record of the requested type. |
| `has_recording_consent` | `env: &Env, patient_id: BytesN<32>` | `Result<(bool, u64), TelemedicineError>` | Check whether a patient has granted session recording consent. Returns (has_consent, expiry) where expiry is 0 if no consent exists. |
| `schedule_consultation` | `env: &Env, session_id: BytesN<32>, patient_id: BytesN<32>, provider_id: BytesN<32>, scheduled_time: u64, consultation_type: String, _appointment_id: BytesN<32>` | `Result<(), TelemedicineError>` | — |
| `start_consultation` | `env: &Env, session_id: BytesN<32>, caller: Address` | `Result<(), TelemedicineError>` | — |
| `complete_consultation` | `env: &Env, session_id: BytesN<32>, provider_address: Address, recording_hash: BytesN<32>, _appointment_id: BytesN<32>, quality_score: u32` | `Result<(), TelemedicineError>` | — |
| `get_consultation` | `env: &Env, session_id: BytesN<32>` | `Result<Consultation, TelemedicineError>` | — |
| `issue_prescription` | `env: &Env, prescription_id: BytesN<32>, consultation_id: BytesN<32>, patient_id: BytesN<32>, provider_id: BytesN<32>, provider_address: Address, medications: Vec<String>, valid_days: u64, pharmacy_id: String` | `Result<(), TelemedicineError>` | — |
| `get_prescription` | `env: &Env, prescription_id: BytesN<32>` | `Result<Prescription, TelemedicineError>` | — |
| `start_monitoring_session` | `env: &Env, session_id: BytesN<32>, patient_id: BytesN<32>, provider_id: BytesN<32>, _duration_hours: u32` | `Result<(), TelemedicineError>` | — |
| `end_monitoring_session` | `env: &Env, session_id: BytesN<32>` | `Result<MonitoringSession, TelemedicineError>` | — |
| `upsert_knowledge_entry` | `env: &Env, entry_id: BytesN<32>, category: String, language: String, title: String, summary: String, guidance: String, source_ref: String` | `Result<(), TelemedicineError>` | — |
| `get_knowledge_entry` | `env: &Env, entry_id: BytesN<32>` | `Result<MedicalKnowledgeEntry, TelemedicineError>` | — |
| `configure_emergency_protocol` | `env: &Env, protocol_id: BytesN<32>, emergency_contact: String, escalation_message_en: String, escalation_message_sw: String, escalation_message_fr: String, ambulance_ref: String` | `Result<(), TelemedicineError>` | — |
| `get_emergency_protocol` | `env: &Env` | `EmergencyProtocol` | — |
| `submit_chatbot_inquiry` | `env: &Env, inquiry_id: BytesN<32>, patient_id: BytesN<32>, caller: Address, message: String` | `Result<ChatbotInquiry, TelemedicineError>` | — |
| `get_chatbot_inquiry` | `env: &Env, inquiry_id: BytesN<32>` | `Result<ChatbotInquiry, TelemedicineError>` | — |
| `get_latest_patient_inquiry` | `env: &Env, patient_id: BytesN<32>` | `Result<ChatbotInquiry, TelemedicineError>` | — |
| `is_chatbot_inquiry_accurate` | `env: &Env, inquiry_id: BytesN<32>` | `Result<bool, TelemedicineError>` | — |
| `get_chatbot_response_time_ms` | `env: &Env, inquiry_id: BytesN<32>` | `Result<u32, TelemedicineError>` | — |
| `get_emergency_case` | `env: &Env, emergency_id: BytesN<32>` | `Result<EmergencyCase, TelemedicineError>` | — |
| `get_active_emergencies` | `env: &Env` | `Vec<BytesN<32>>` | — |
| `resolve_emergency_case` | `env: &Env, emergency_id: BytesN<32>` | `Result<EmergencyCase, TelemedicineError>` | — |
| `get_platform_stats` | `env: Env` | `(u64, u64, u64, u64, u64, u64)` | — |

### Types

#### `enum TelemedicineError`

| Variant | Value | Description |
|---|---|---|
| `ContractPaused` | 1 | — |
| `NotPaused` | 2 | — |
| `NotAdmin` | 3 | — |
| `ProviderAlreadyRegistered` | 4 | — |
| `ProviderNotFound` | 5 | — |
| `ProviderNotActive` | 6 | — |
| `LicenseExpired` | 7 | — |
| `PatientAlreadyRegistered` | 8 | — |
| `PatientNotFound` | 9 | — |
| `ConsentNotGiven` | 10 | — |
| `ConsultationNotFound` | 11 | — |
| `ConsultationNotScheduled` | 12 | — |
| `ConsultationNotActive` | 13 | — |
| `ConsultationAlreadyCompleted` | 14 | — |
| `PrescriptionNotFound` | 15 | — |
| `MonitoringSessionNotFound` | 16 | — |
| `AppointmentNotFound` | 17 | — |
| `DigitalTherapeuticNotFound` | 18 | — |
| `QualityAssessmentNotFound` | 19 | — |
| `EmergencyNotFound` | 20 | — |
| `EmergencyAlreadyResolved` | 21 | — |
| `InvalidJurisdiction` | 22 | — |
| `DataTransferNotApproved` | 23 | — |
| `UnsupportedLanguage` | 24 | — |
| `ChatbotInquiryNotFound` | 25 | — |
| `InvalidChatMessage` | 26 | — |
| `KnowledgeEntryAlreadyExists` | 27 | — |
| `KnowledgeEntryNotFound` | 28 | — |

#### `enum ConsentType`

| Variant | Value | Description |
|---|---|---|
| `VideoConsultation` | 0 | — |
| `RemoteMonitoring` | 1 | — |
| `DigitalTherapeutic` | 2 | — |
| `EmergencyContact` | 3 | — |
| `DataSharing` | 4 | — |
| `SessionRecording` | 5 | — |

#### `enum ConsultationStatus`

| Variant | Value | Description |
|---|---|---|
| `Scheduled` | 0 | — |
| `Active` | 1 | — |
| `Completed` | 2 | — |
| `Cancelled` | 3 | — |

#### `enum EmergencyLevel`

| Variant | Value | Description |
|---|---|---|
| `Low` | 0 | — |
| `Medium` | 1 | — |
| `High` | 2 | — |
| `Critical` | 3 | — |

#### `enum QualityRating`

| Variant | Value | Description |
|---|---|---|
| `Poor` | 0 | — |
| `Fair` | 1 | — |
| `Good` | 2 | — |
| `VeryGood` | 3 | — |
| `Excellent` | 4 | — |

#### `enum ChatIntent`

| Variant | Value | Description |
|---|---|---|
| `SymptomCheck` | 0 | — |
| `HealthEducation` | 1 | — |
| `MedicationGuidance` | 2 | — |
| `EmergencySupport` | 3 | — |
| `GeneralInquiry` | 4 | — |

#### `struct Provider`

| Field | Type | Description |
|---|---|---|
| `provider_id` | `BytesN<32>` | — |
| `address` | `Address` | — |
| `name` | `String` | — |
| `credentials` | `BytesN<32>` | — |
| `jurisdictions` | `Vec<String>` | — |
| `specialty` | `String` | — |
| `license_expiry` | `u64` | — |
| `is_active` | `bool` | — |
| `registration_date` | `u64` | — |

#### `struct Patient`

| Field | Type | Description |
|---|---|---|
| `patient_id` | `BytesN<32>` | — |
| `address` | `Address` | — |
| `primary_care_physician` | `BytesN<32>` | — |
| `monitoring_device` | `String` | — |
| `jurisdiction` | `String` | — |
| `contact_info` | `String` | — |
| `preferred_language` | `String` | — |
| `registration_date` | `u64` | — |

#### `struct ConsentRecord`

| Field | Type | Description |
|---|---|---|
| `consent_id` | `BytesN<32>` | — |
| `patient_id` | `BytesN<32>` | — |
| `consent_type` | `ConsentType` | — |
| `granted` | `bool` | — |
| `timestamp` | `u64` | — |
| `expiry` | `u64` | — |
| `scope` | `String` | — |

#### `struct Consultation`

| Field | Type | Description |
|---|---|---|
| `session_id` | `BytesN<32>` | — |
| `patient_id` | `BytesN<32>` | — |
| `provider_id` | `BytesN<32>` | — |
| `scheduled_time` | `u64` | — |
| `start_time` | `u64` | — |
| `end_time` | `u64` | — |
| `status` | `ConsultationStatus` | — |
| `recording_hash` | `BytesN<32>` | — |
| `appointment_id` | `BytesN<32>` | — |
| `consultation_type` | `String` | — |
| `quality_score` | `u32` | — |
| `recording_consent_granted_at` | `u64` | — |
| `recording_consent_expiry` | `u64` | — |

#### `struct Prescription`

| Field | Type | Description |
|---|---|---|
| `prescription_id` | `BytesN<32>` | — |
| `consultation_id` | `BytesN<32>` | — |
| `patient_id` | `BytesN<32>` | — |
| `provider_id` | `BytesN<32>` | — |
| `medications` | `Vec<String>` | — |
| `issued_date` | `u64` | — |
| `valid_days` | `u64` | — |
| `pharmacy_id` | `String` | — |
| `is_active` | `bool` | — |
| `cross_border` | `bool` | — |
| `jurisdiction` | `String` | — |

#### `struct VitalSigns`

| Field | Type | Description |
|---|---|---|
| `heart_rate` | `u32` | — |
| `blood_pressure_systolic` | `u32` | — |
| `blood_pressure_diastolic` | `u32` | — |
| `spo2` | `u32` | — |
| `temperature` | `u32` | — |
| `respiratory_rate` | `u32` | — |
| `blood_glucose` | `u32` | — |
| `device_id` | `String` | — |
| `timestamp` | `u64` | — |

#### `struct MonitoringSession`

| Field | Type | Description |
|---|---|---|
| `session_id` | `BytesN<32>` | — |
| `patient_id` | `BytesN<32>` | — |
| `provider_id` | `BytesN<32>` | — |
| `start_time` | `u64` | — |
| `end_time` | `u64` | — |
| `is_active` | `bool` | — |
| `vital_signs_count` | `u32` | — |
| `alerts_count` | `u32` | — |

#### `struct AppointmentSlot`

| Field | Type | Description |
|---|---|---|
| `appointment_id` | `BytesN<32>` | — |
| `provider_id` | `BytesN<32>` | — |
| `patient_id` | `BytesN<32>` | — |
| `start_time` | `u64` | — |
| `end_time` | `u64` | — |
| `consultation_type` | `String` | — |
| `is_confirmed` | `bool` | — |
| `telemedicine_room` | `String` | — |

#### `struct ComplianceRecord`

| Field | Type | Description |
|---|---|---|
| `record_id` | `BytesN<32>` | — |
| `consultation_id` | `BytesN<32>` | — |
| `patient_jurisdiction` | `String` | — |
| `provider_jurisdiction` | `String` | — |
| `compliance_framework` | `String` | — |
| `data_transfer_approved` | `bool` | — |
| `gdpr_compliant` | `bool` | — |
| `hipaa_compliant` | `bool` | — |
| `local_law_compliant` | `bool` | — |
| `verification_timestamp` | `u64` | — |
| `verified_by` | `Address` | — |

#### `struct DigitalTherapeutic`

| Field | Type | Description |
|---|---|---|
| `therapeutic_id` | `BytesN<32>` | — |
| `patient_id` | `BytesN<32>` | — |
| `provider_id` | `BytesN<32>` | — |
| `program_name` | `String` | — |
| `program_hash` | `BytesN<32>` | — |
| `enrollment_date` | `u64` | — |
| `completion_percentage` | `u32` | — |
| `adherence_score` | `u32` | — |
| `session_count` | `u32` | — |
| `duration_days` | `u32` | — |
| `is_active` | `bool` | — |

#### `struct QualityAssessment`

| Field | Type | Description |
|---|---|---|
| `assessment_id` | `BytesN<32>` | — |
| `consultation_id` | `BytesN<32>` | — |
| `assessor_provider` | `Address` | — |
| `technical_quality` | `QualityRating` | — |
| `clinical_quality` | `QualityRating` | — |
| `patient_satisfaction` | `u32` | — |
| `connection_quality` | `u32` | — |
| `issues` | `Vec<String>` | — |
| `assessment_date` | `u64` | — |

#### `struct EmergencyCase`

| Field | Type | Description |
|---|---|---|
| `emergency_id` | `BytesN<32>` | — |
| `patient_id` | `BytesN<32>` | — |
| `reporting_provider` | `BytesN<32>` | — |
| `responding_provider` | `BytesN<32>` | — |
| `emergency_level` | `EmergencyLevel` | — |
| `reported_symptoms` | `String` | — |
| `triage_notes_hash` | `BytesN<32>` | — |
| `triggered_at` | `u64` | — |
| `response_time` | `u64` | — |
| `resolved_at` | `u64` | — |
| `is_resolved` | `bool` | — |
| `escalated_to_physical` | `bool` | — |

#### `struct MedicalKnowledgeEntry`

| Field | Type | Description |
|---|---|---|
| `entry_id` | `BytesN<32>` | — |
| `category` | `String` | — |
| `language` | `String` | — |
| `title` | `String` | — |
| `summary` | `String` | — |
| `guidance` | `String` | — |
| `source_ref` | `String` | — |
| `content_hash` | `BytesN<32>` | — |
| `updated_at` | `u64` | — |
| `is_active` | `bool` | — |

#### `struct EmergencyProtocol`

| Field | Type | Description |
|---|---|---|
| `protocol_id` | `BytesN<32>` | — |
| `emergency_contact` | `String` | — |
| `escalation_message_en` | `String` | — |
| `escalation_message_sw` | `String` | — |
| `escalation_message_fr` | `String` | — |
| `ambulance_ref` | `String` | — |
| `updated_at` | `u64` | — |

#### `struct ChatbotInquiry`

| Field | Type | Description |
|---|---|---|
| `inquiry_id` | `BytesN<32>` | — |
| `patient_id` | `BytesN<32>` | — |
| `patient_address` | `Address` | — |
| `original_message` | `String` | — |
| `normalized_message` | `String` | — |
| `detected_language` | `String` | — |
| `intent` | `ChatIntent` | — |
| `confidence_bps` | `u32` | — |
| `triage_level` | `EmergencyLevel` | — |
| `emergency_detected` | `bool` | — |
| `escalation_required` | `bool` | — |
| `recommended_action` | `String` | — |
| `health_education` | `String` | — |
| `knowledge_source_ref` | `String` | — |
| `matched_articles` | `Vec<BytesN<32>>` | — |
| `emergency_case_id` | `BytesN<32>` | — |
| `response_time_ms` | `u32` | — |
| `created_at` | `u64` | — |

#### `enum DataKey`

| Variant | Value | Description |
|---|---|---|
| `Admin` | — | — |
| `Paused` | — | — |
| `EmergencyProtocol` | — | — |
| `KnowledgeIndex` | — | — |
| `PlatformStats` | — | — |
| `Provider(BytesN<32>)` | — | — |
| `Patient(BytesN<32>)` | — | — |
| `Consent(BytesN<32>)` | — | — |
| `PatientConsents(BytesN<32>),
    Consultation(BytesN<32>),
    Prescription(BytesN<32>),
    MonitoringSession(BytesN<32>),
    Appointment(BytesN<32>),
    ComplianceRecord(BytesN<32>),
    DigitalTherapeutic(BytesN<32>),
    QualityAssessment(BytesN<32>),
    Emergency(BytesN<32>),
    KnowledgeEntry(BytesN<32>),
    ActiveEmergencies,
    ChatbotInquiry(BytesN<32>),
    
    LatestPatientInquiry(BytesN<32>),
    ProviderSchedule(BytesN<32>),` | — | — |

### Examples

#### `test_initialize_contract`

```rust
let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, TelemedicineContract);
    let client = TelemedicineContractClient::new(&env, &contract_id);
    let admin = generate_test_address(&env);

    client.initialize(&admin);

    let (providers, patients, consultations, prescriptions, alerts, emergencies) =
```

#### `test_double_initialization`

```rust
let ctx = TestContext::new();
    let result = ctx.client.try_initialize(&ctx.admin);
    assert_err!(result, TelemedicineError::NotPaused);
```

#### `test_pause_unpause`

```rust
let ctx = TestContext::new();

    ctx.client.pause();

    let result = ctx.client.try_register_patient(
        &BytesN::from_array(&ctx.env, &[3u8; 32]),
        &ctx.patient,
        &BytesN::from_array(&ctx.env, &[4u8; 32]),
        &String::from_str(&ctx.env, "KE"),
```

---

## timelock

### Functions

| Function | Parameters | Returns | Description |
|---|---|---|---|
| `initialize` | `env: Env, admin: Address, delay_seconds: u64` | `Result<(), Error>` | — |
| `get_config` | `env: Env` | `Option<TimelockConfig>` | — |
| `queue` | `env: Env, id: u64, target: Address, call: BytesN<32>` | `Result<(), Error>` | — |
| `execute` | `env: Env, id: u64` | `Result<(), Error>` | — |

### Types

#### `struct TimelockConfig`

| Field | Type | Description |
|---|---|---|
| `admin` | `Address` | — |
| `delay_seconds` | `u64` | — |

#### `struct QueuedTx`

| Field | Type | Description |
|---|---|---|
| `target` | `Address` | — |
| `call` | `BytesN<32>` | — |
| `eta` | `u64` | — |

### Error Codes

| Variant | Code | Description |
|---|---|---|
| `Unauthorized` | 100 | — |
| `InvalidSignature` | 207 | — |
| `NotInitialized` | 300 | — |
| `AlreadyInitialized` | 301 | — |
| `ContractPaused` | 302 | — |
| `DeadlineExceeded` | 306 | — |
| `AlreadyQueued` | 375 | — |
| `NotQueued` | 372 | — |
| `NotReady` | 376 | — |
| `InsufficientFunds` | 500 | — |
| `StorageFull` | 502 | — |
| `CrossChainTimeout` | 702 | — |

---

## token_sale

### Functions

| Function | Parameters | Returns | Description |
|---|---|---|---|

### Error Codes

| Variant | Code | Description |
|---|---|---|
| `AlreadyInitialized` | 1 | — |
| `InvalidArgument` | 2 | — |
| `Overflow` | 3 | — |
| `PhaseNotFound` | 4 | — |
| `PhaseClosed` | 5 | — |
| `CapExceeded` | 6 | — |
| `NotFinalized` | 7 | — |
| `AlreadyClaimed` | 8 | — |
| `RefundsNotEnabled` | 9 | — |
| `Paused` | 10 | — |

### Examples

#### `test_token_sale_initialization`

```rust
let env = Env::default();
    env.mock_all_auths();

    let owner = Address::generate(&env);
    let treasury = Address::generate(&env);
    let (token_address, _token_client, _token_admin) = create_token_contract(&env, &owner);

    let contract_id = env.register_contract(None, TokenSaleContract);
    let client = TokenSaleContractClient::new(&env, &contract_id);
```

#### `test_add_sale_phase`

```rust
let env = Env::default();
    env.mock_all_auths();

    let owner = Address::generate(&env);
    let treasury = Address::generate(&env);
    let (token_address, _token_client, _token_admin) = create_token_contract(&env, &owner);

    let contract_id = env.register_contract(None, TokenSaleContract);
    let client = TokenSaleContractClient::new(&env, &contract_id);
```

#### `test_contribution_and_claim`

```rust
let env = Env::default();
    env.mock_all_auths();

    let owner = Address::generate(&env);
    let treasury = Address::generate(&env);
    let contributor = Address::generate(&env);

    let (sut_token_address, sut_token_client, sut_token_admin) =
        create_token_contract(&env, &owner);
```

---

## treasury_controller

### Functions

| Function | Parameters | Returns | Description |
|---|---|---|---|
| `initialize` | `env: Env, admin: Address, signers: Vec<Address>, threshold: u32, timelock_duration: u64, emergency_threshold: u32, max_withdrawal_amount: i128` | `Result<(), Error>` | Initialize the treasury controller |
| `add_supported_token` | `env: Env, token_address: Address` | `Result<(), Error>` | Add supported token for treasury operations |
| `create_proposal` | `env: Env, proposer: Address, proposal_type: ProposalType, target_address: Address, token_contract: Address, amount: i128, purpose: String, metadata: String, execution_data: Bytes` | `Result<u64, Error>` | Create a new treasury proposal |
| `approve_proposal` | `env: Env, signer: Address, proposal_id: u64` | `Result<(), Error>` | Approve a treasury proposal |
| `execute_proposal` | `env: Env, executor: Address, proposal_id: u64` | `Result<(), Error>` | Execute an approved proposal after timelock |
| `emergency_halt` | `env: Env, caller: Address` | `Result<(), Error>` | Emergency halt all treasury operations |
| `resume_operations` | `env: Env, caller: Address` | `Result<(), Error>` | Resume operations after emergency halt |
| `get_config` | `env: Env` | `Result<TreasuryConfig, Error>` | Get treasury configuration |
| `get_proposal` | `env: Env, proposal_id: u64` | `Result<TreasuryProposal, Error>` | Get proposal details |
| `get_proposal_count` | `env: Env` | `Result<u64, Error>` | Get total number of proposals |
| `is_proposal_executable` | `env: Env, proposal_id: u64` | `bool` | Check if proposal is ready for execution |
| `gnosis_get_threshold` | `env: Env` | `Result<u32, Error>` | Get threshold for Gnosis Safe compatibility |
| `gnosis_get_owners` | `env: Env` | `Result<Vec<Address>, Error>` | Get owners for Gnosis Safe compatibility |
| `governance_execute` | `env: Env, token_contract: Address, to: Address, amount: i128` | `Result<(), Error>` | Allows the Governor/Timelock (Admin) to execute transfers immediately Bypassing the multisig process. |

### Types

#### `enum Error`

| Variant | Value | Description |
|---|---|---|
| `NotInitialized` | 1 | — |
| `AlreadyInitialized` | 2 | — |
| `InvalidThreshold` | 3 | — |
| `InvalidTimelock` | 4 | — |
| `NotSigner` | 5 | — |
| `ProposalNotFound` | 6 | — |
| `NotPending` | 7 | — |
| `AlreadyApproved` | 8 | — |
| `TimelockNotExpired` | 9 | — |
| `NotApproved` | 10 | — |
| `Halted` | 11 | — |
| `NotAuthorized` | 12 | — |
| `SymbolTooLong` | 13 | — |
| `TransferFailed` | 14 | — |
| `ConfigNotFound` | 15 | — |

#### `enum ProposalType`

| Variant | Value | Description |
|---|---|---|
| `Withdrawal` | — | — |
| `ConfigChange` | — | — |
| `EmergencyHalt` | — | — |

#### `enum ProposalStatus`

| Variant | Value | Description |
|---|---|---|
| `Pending` | — | — |
| `Approved` | — | — |
| `Executed` | — | — |
| `Rejected` | — | — |
| `Expired` | — | — |

#### `struct TreasuryProposal`

| Field | Type | Description |
|---|---|---|
| `proposal_id` | `u64` | — |
| `proposal_type` | `ProposalType` | — |
| `proposer` | `Address` | — |
| `target_address` | `Address` | — |
| `token_contract` | `Address` | — |
| `amount` | `i128` | — |
| `purpose` | `String` | — |
| `metadata` | `String` | — |
| `created_at` | `u64` | — |
| `timelock_end` | `u64` | — |
| `status` | `ProposalStatus` | — |
| `approvals` | `Vec<Address>` | — |
| `rejections` | `Vec<Address>` | — |
| `execution_data` | `Bytes` | — |

#### `struct MultisigConfig`

| Field | Type | Description |
|---|---|---|
| `signers` | `Vec<Address>` | — |
| `threshold` | `u32` | — |
| `timelock_duration` | `u64` | — |
| `emergency_threshold` | `u32` | — |

#### `struct TreasuryConfig`

| Field | Type | Description |
|---|---|---|
| `admin` | `Address` | — |
| `multisig_config` | `MultisigConfig` | — |
| `max_withdrawal_amount` | `i128` | — |
| `emergency_halted` | `bool` | — |
| `supported_tokens` | `Vec<Address>` | — |

#### `struct WithdrawalRecord`

| Field | Type | Description |
|---|---|---|
| `proposal_id` | `u64` | — |
| `token_contract` | `Address` | — |
| `amount` | `i128` | — |
| `recipient` | `Address` | — |
| `purpose` | `String` | — |
| `executed_at` | `u64` | — |
| `executed_by` | `Address` | — |
| `transaction_hash` | `BytesN<32>` | — |

#### `enum DataKey`

| Variant | Value | Description |
|---|---|---|
| `Config` | — | — |
| `Proposals` | — | — |
| `ProposalCount` | — | — |
| `Withdrawals` | — | — |

### Examples

#### `test_initialize`

```rust
let (_env, _client, _admin, signers) = setup();
        assert_eq!(signers.len(), 3);
    }

    #[test]
    fn test_double_initialize() {
        let (env, client, admin, _signers) = setup();

        let result = client.try_initialize(
```

---

## upgrade_manager

### Functions

| Function | Parameters | Returns | Description |
|---|---|---|---|
| `initialize` | `env: Env, admin: Address, validators: Vec<Address>` | `Result<(), Error>` | — |
| `propose_upgrade` | `env: Env, proposer: Address, target: Address, new_wasm_hash: BytesN<32>, new_version: u32, description: Symbol, is_emergency: bool` | `Result<u64, Error>` | — |
| `approve` | `env: Env, validator: Address, proposal_id: u64` | `Result<(), Error>` | — |
| `execute` | `env: Env, proposal_id: u64` | `Result<(), Error>` | — |
| `execute_emergency` | `env: Env, proposal_id: u64` | `Result<(), Error>` | — |
| `validate_proposal` | `env: Env, proposal_id: u64` | `Result<UpgradeValidation, Error>` | — |

### Types

#### `struct UpgradeProposal`

| Field | Type | Description |
|---|---|---|
| `target` | `Address` | — |
| `new_wasm_hash` | `BytesN<32>` | — |
| `new_version` | `u32` | — |
| `description` | `Symbol` | — |
| `proposer` | `Address` | — |
| `created_at` | `u64` | — |
| `executable_at` | `u64` | — |
| `executed` | `bool` | — |
| `canceled` | `bool` | — |
| `approvals` | `Vec<Address>` | — |
| `is_emergency` | `bool` | — |

#### `struct Config`

| Field | Type | Description |
|---|---|---|
| `admin` | `Address` | — |
| `min_delay` | `u64` | — |
| `required_approvals` | `u32` | — |
| `validators` | `Vec<Address>` | — |
| `emergency_approvals` | `u32` | — |

### Error Codes

| Variant | Code | Description |
|---|---|---|
| `NotAValidator` | 110 | — |
| `NotEnoughApprovals` | 120 | — |
| `AlreadyInitialized` | 301 | — |
| `InvalidState` | 304 | — |
| `TimelockNotExpired` | 376 | — |
| `ConfigNotFound` | 390 | — |
| `ProposalNotFound` | 450 | — |
| `AlreadyApproved` | 451 | — |

### Examples

#### `test_complex_upgrade_flow`

```rust
let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let v1 = Address::generate(&env);
    let v2 = Address::generate(&env);
    let v3 = Address::generate(&env);
    let validators = Vec::from_array(&env, [v1.clone(), v2.clone(), v3.clone()]);
```

#### `test_error_codes_are_stable`

```rust
use crate::errors::Error;
    assert_eq!(Error::NotAValidator as u32, 110);
    assert_eq!(Error::NotEnoughApprovals as u32, 120);
    assert_eq!(Error::AlreadyInitialized as u32, 301);
    assert_eq!(Error::InvalidState as u32, 304);
    assert_eq!(Error::TimelockNotExpired as u32, 376);
    assert_eq!(Error::ProposalNotFound as u32, 450);
```

#### `test_get_suggestion_returns_expected_hint`

```rust
use crate::errors::{get_suggestion, Error};
    assert_eq!(
        get_suggestion(Error::NotAValidator),
        symbol_short!("CHK_AUTH")
    );
    assert_eq!(
        get_suggestion(Error::AlreadyInitialized),
        symbol_short!("ALREADY")
    );
```

---

## upgradeability

### Functions

| Function | Parameters | Returns | Description |
|---|---|---|---|
| `get_version` | `env: &Env` | `u32` | — |
| `set_version` | `env: &Env, version: u32` | `()` | — |
| `get_admin` | `env: &Env` | `Option<Address>` | — |
| `set_admin` | `env: &Env, admin: &Address` | `()` | — |
| `is_frozen` | `env: &Env` | `bool` | — |
| `freeze` | `env: &Env` | `()` | — |
| `add_history` | `env: &Env, history: UpgradeHistory` | `()` | — |
| `get_history` | `env: &Env` | `Vec<UpgradeHistory>` | — |
| `set_deprecated_functions` | `env: &Env, deprecations: &Vec<DeprecatedFunction>` | `()` | — |
| `get_deprecated_functions` | `env: &Env` | `Vec<DeprecatedFunction>` | — |
| `authorize_upgrade` | `env: &Env` | `Result<Address, UpgradeError>` | — |
| `execute_upgrade` | `env: &Env, new_wasm_hash: BytesN<32>, new_version: u32, description: Symbol` | `Result<(), UpgradeError>` | — |
| `execute_upgrade_with_deprecations` | `env: &Env, new_wasm_hash: BytesN<32>, new_version: u32, description: Symbol, deprecations: Vec<DeprecatedFunction>` | `Result<(), UpgradeError>` | — |
| `validate_upgrade` | `env: &Env, new_wasm_hash: BytesN<32>` | `Result<UpgradeValidation, UpgradeError>` | — |
| `rollback` | `env: &Env` | `Result<(), UpgradeError>` | — |
| `set_deprecated_functions` | `env: &Env, deprecations: Vec<DeprecatedFunction>` | `Result<(), UpgradeError>` | — |
| `get_deprecated_functions` | `env: &Env` | `Vec<DeprecatedFunction>` | — |
| `get_deprecated_function` | `env: &Env, function: Symbol` | `Option<DeprecatedFunction>` | — |
| `emit_deprecation_warning` | `env: &Env, function: Symbol` | `Result<(), UpgradeError>` | — |

### Types

#### `enum UpgradeError`

| Variant | Value | Description |
|---|---|---|
| `NotAuthorized` | 100 | — |
| `InvalidWasmHash` | 101 | — |
| `VersionAlreadyExists` | 102 | — |
| `MigrationFailed` | 103 | — |
| `IncompatibleVersion` | 104 | — |
| `ContractPaused` | 105 | — |
| `HistoryNotFound` | 106 | — |
| `IntegrityCheckFailed` | 107 | — |
| `DeprecatedFunctionNotTracked` | 108 | — |

#### `struct UpgradeHistory`

| Field | Type | Description |
|---|---|---|
| `wasm_hash` | `BytesN<32>` | — |
| `version` | `u32` | — |
| `upgraded_at` | `u64` | — |
| `description` | `Symbol` | — |
| `state_hash` | `BytesN<32>` | — |

#### `struct DeprecatedFunction`

| Field | Type | Description |
|---|---|---|
| `function` | `Symbol` | — |
| `since` | `String` | — |
| `replacement` | `Option<Symbol>` | — |
| `removed_in` | `Option<String>` | — |
| `note` | `String` | — |
| `migration_guide` | `Option<String>` | — |

### Examples

#### `test_deprecated_functions_are_tracked`

```rust
let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    storage::set_admin(&env, &admin);

    let deprecation = sample_deprecation(&env);
    let deprecations = Vec::from_array(&env, [deprecation.clone()]);
```

#### `test_deprecation_warning_emits_event`

```rust
let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    storage::set_admin(&env, &admin);

    let deprecations = Vec::from_array(&env, [sample_deprecation(&env)]);
    set_deprecated_functions(&env, deprecations).unwrap();
```

---

## zk_verifier

### Functions

| Function | Parameters | Returns | Description |
|---|---|---|---|
| `initialize` | `env: Env, admin: Address, default_ttl: u64` | `Result<(), Error>` | — |
| `set_default_ttl` | `env: Env, caller: Address, ttl: u64` | `Result<(), Error>` | — |
| `get_default_ttl` | `env: Env` | `u64` | — |
| `register_verifying_key` | `env: Env, caller: Address, vk_hash: BytesN<32>, circuit_id: BytesN<32>, attestor: Address, metadata_hash: BytesN<32>` | `Result<u32, Error>` | — |
| `deactivate_verifying_key` | `env: Env, caller: Address, version: u32` | `Result<bool, Error>` | — |
| `get_verifying_key` | `env: Env, version: u32` | `Option<VerifyingKeyConfig>` | — |
| `get_current_version` | `env: Env` | `u32` | — |
| `submit_attestation` | `env: Env, attestor: Address, vk_version: u32, public_inputs_hash: BytesN<32>, proof_hash: BytesN<32>, verified: bool, ttl: u64` | `Result<(), Error>` | — |
| `verify_proof` | `env: Env, vk_version: u32, public_inputs_hash: BytesN<32>, proof: Bytes` | `bool` | — |
| `get_attestation` | `env: Env, vk_version: u32, public_inputs_hash: BytesN<32>, proof_hash: BytesN<32>` | `Option<ProofAttestation>` | — |
| `compute_proof_hash` | `env: Env, proof: Bytes` | `BytesN<32>` | — |
| `mark_nullifier_used` | `env: Env, nullifier: BytesN<32>` | `Result<(), Error>` | — |
| `is_nullifier_used` | `env: Env, nullifier: BytesN<32>` | `bool` | — |

### Types

#### `struct VerifyingKeyConfig`

| Field | Type | Description |
|---|---|---|
| `version` | `u32` | — |
| `vk_hash` | `BytesN<32>` | — |
| `circuit_id` | `BytesN<32>` | — |
| `attestor` | `Address` | — |
| `metadata_hash` | `BytesN<32>` | — |
| `created_at` | `u64` | — |
| `active` | `bool` | — |

#### `struct ProofAttestation`

| Field | Type | Description |
|---|---|---|
| `vk_version` | `u32` | — |
| `public_inputs_hash` | `BytesN<32>` | — |
| `proof_hash` | `BytesN<32>` | — |
| `verified` | `bool` | — |
| `attestor` | `Address` | — |
| `created_at` | `u64` | — |
| `expires_at` | `u64` | — |

#### `struct NullifierRecord`

| Field | Type | Description |
|---|---|---|
| `nullifier` | `BytesN<32>` | — |
| `consumed_at` | `u64` | — |

#### `enum DataKey`

| Variant | Value | Description |
|---|---|---|
| `Initialized` | — | — |
| `Admin` | — | — |
| `CurrentVersion` | — | — |
| `DefaultTtl` | — | — |
| `VerifyingKey(u32)` | — | — |
| `Attestation(BytesN<32>)` | — | — |
| `Nullifier(BytesN<32>)` | — | — |

### Error Codes

| Variant | Code | Description |
|---|---|---|
| `Unauthorized` | 100 | — |
| `InvalidInput` | 200 | — |
| `NotInitialized` | 300 | — |
| `AlreadyInitialized` | 301 | — |
| `VersionNotFound` | 430 | — |
| `InvalidProof` | 600 | — |
| `VerificationFailed` | 601 | — |

---

## zkp_registry

### Functions

| Function | Parameters | Returns | Description |
|---|---|---|---|
| `initialize` | `env: Env, admin: Address` | `Result<(), Error>` | Initialize the ZKP registry |
| `configure_multisig` | `env: Env, admin: Address, config: MultiSigConfig` | `Result<(), Error>` | Configure multi-signature for admin operations |
| `create_admin_proposal` | `env: Env, signer: Address, action: AdminAction` | `Result<u64, Error>` | Create an admin proposal |
| `approve_admin_proposal` | `env: Env, signer: Address, proposal_id: u64` | `Result<(), Error>` | Approve an admin proposal |
| `execute_admin_proposal` | `env: Env, executor: Address, proposal_id: u64` | `Result<(), Error>` | Execute an admin proposal |
| `emergency_override` | `env: Env, executor: Address, proposal_id: u64` | `Result<(), Error>` | Emergency override to execute a proposal without waiting for the timelock |
| `register_circuit` | `env: Env, admin: Address, circuit_id: String, circuit_type: ZKPType, num_public_inputs: u32, num_private_inputs: u32, num_constraints: u32, security_param: u32, vk_hash: BytesN<32>, pk_hash: BytesN<32>, trusted_setup: bool` | `Result<(), Error>` | Register ZKP circuit parameters |
| `submit_zkp` | `env: Env, submitter: Address, proof_id: BytesN<32>, proof_type: ZKPType, hash_function: ZKPHashFunction, circuit_id: String, public_inputs: Vec<Bytes>, proof_data: Bytes, vk_hash: BytesN<32>, verification_gas: u64` | `Result<(), Error>` | Submit and verify a zero-knowledge proof |
| `submit_zkp_batch` | `env: Env, submitter: Address, proof_ids: Vec<BytesN<32>>, proof_types: Vec<ZKPType>, hash_functions: Vec<ZKPHashFunction>, circuit_ids: Vec<String>, public_inputs_batch: Vec<Vec<Bytes>>, proof_data_batch: Vec<Bytes>, vk_hashes: Vec<BytesN<32>>, verification_gas_batch: Vec<u64>` | `Result<Vec<bool>, Error>` | Submit and verify a batch of zero-knowledge proofs |
| `create_medical_record_proof` | `env: Env, patient: Address, record_id: u64, authenticity_proof: ZKProof, access_proof: ZKProof, metadata_hash: BytesN<32>` | `Result<(), Error>` | Create medical record authenticity proof |
| `create_range_proof` | `env: Env, prover: Address, proof_id: BytesN<32>, encrypted_value: Bytes, min_value: u64, max_value: u64, proof_data: Bytes, vk_hash: BytesN<32>, verification_gas: u64` | `Result<(), Error>` | Create range proof for age/condition verification |
| `verify_range_proof` | `env: Env, proof: RangeProof` | `Result<bool, Error>` | Verify a range proof without storing it.  Host-callable verifier that runs the same cryptographic checks as `create_range_proof` but returns the boolean verdict instead of persisting the proof. Useful for cross-contract calls where the caller only needs a verification result. |
| `create_credential_proof` | `env: Env, holder: Address, credential_type: String, issuer: Address, validity_proof: ZKProof, attribute_proof: ZKProof, encrypted_expiration: Bytes` | `Result<(), Error>` | Create credential verification proof |
| `set_issuer_salt` | `env: Env, admin: Address, issuer: Address, salt: BytesN<32>` | `Result<(), Error>` | Admin-only: publish a per-issuer XOR salt used by `decrypt_credential_expiration`. Without this, the contract falls back to `DEFAULT_ISSUER_SALT`, which is a development convenience and MUST NOT be used for production credentials. |
| `create_recursive_proof` | `env: Env, composer: Address, base_proof_id: BytesN<32>, recursive_proof: ZKProof, aggregated_vk_hash: BytesN<32>, composition_depth: u32, total_gas: u64` | `Result<(), Error>` | Create recursive zero-knowledge proof |
| `cleanup_proof` | `env: Env, submitter: Address, proof_id: BytesN<32>` | `Result<(), Error>` | Clean up a proof to manually free storage space |
| `get_verification_result` | `env: Env, proof_id: BytesN<32>` | `Result<ZKPVerificationResult, Error>` | Get ZKP verification result |
| `get_medical_record_proof` | `env: Env, patient: Address, record_id: u64` | `Result<MedicalRecordProof, Error>` | Get medical record proof |
| `get_range_proof` | `env: Env, proof_id: BytesN<32>` | `Result<RangeProof, Error>` | Get range proof |
| `get_credential_proof` | `env: Env, holder: Address, credential_type: String` | `Result<CredentialProof, Error>` | Get credential proof |
| `get_circuit_params` | `env: Env, circuit_id: String` | `Result<ZKPCircuitParams, Error>` | Get circuit parameters |
| `get_gas_stats` | `env: Env, user: Address` | `Result<u64, Error>` | Get gas usage statistics |
| `export_state` | `env: Env` | `Result<Bytes, Error>` | Export contract state for migrations |
| `import_state` | `env: Env, caller: Address, state_bytes: Bytes` | `Result<(), Error>` | Import contract state during migrations |

### Types

#### `struct MultiSigConfig`

| Field | Type | Description |
|---|---|---|
| `signers` | `Vec<Address>` | — |
| `threshold` | `u32` | — |
| `timelock_duration` | `u64` | — |

#### `enum AdminAction`

| Variant | Value | Description |
|---|---|---|
| `UpgradeContract(BytesN<32>)` | — | — |
| `UpdateParameters(String, u32)` | — | — |
| `EmergencyPause` | — | — |
| `EmergencyResume` | — | — |

#### `struct AdminProposal`

| Field | Type | Description |
|---|---|---|
| `id` | `u64` | — |
| `action` | `AdminAction` | — |
| `created_at` | `u64` | — |
| `executed` | `bool` | — |
| `approvals` | `Vec<Address>` | — |

#### `enum ZKPType`

| Variant | Value | Description |
|---|---|---|
| `SNARK` | — | — |
| `STARK` | — | — |
| `Bulletproof` | — | — |
| `PedersenCommitment` | — | — |
| `Recursive` | — | — |

#### `enum ZKPHashFunction`

| Variant | Value | Description |
|---|---|---|
| `Poseidon` | — | — |
| `MiMC` | — | — |
| `SHA256` | — | — |
| `Rescue` | — | — |

#### `struct ZKProof`

| Field | Type | Description |
|---|---|---|
| `proof_type` | `ZKPType` | — |
| `hash_function` | `ZKPHashFunction` | — |
| `circuit_id` | `String` | — |
| `public_inputs` | `Vec<Bytes>` | — |
| `proof_data` | `Bytes` | — |
| `vk_hash` | `BytesN<32>` | — |
| `verification_gas` | `u64` | — |
| `created_at` | `u64` | — |

#### `struct MedicalRecordProof`

| Field | Type | Description |
|---|---|---|
| `patient_id` | `Address` | — |
| `record_id` | `u64` | — |
| `authenticity_proof` | `ZKProof` | — |
| `access_proof` | `ZKProof` | — |
| `metadata_hash` | `BytesN<32>` | — |
| `is_verified` | `bool` | — |
| `verified_at` | `u64` | — |

#### `struct RangeProof`

| Field | Type | Description |
|---|---|---|
| `prover` | `Address` | — |
| `encrypted_value` | `Bytes` | — |
| `min_value` | `u64` | — |
| `max_value` | `u64` | — |
| `proof_data` | `Bytes` | — |
| `vk_hash` | `BytesN<32>` | — |
| `verification_gas` | `u64` | — |
| `created_at` | `u64` | — |

#### `struct CredentialProof`

| Field | Type | Description |
|---|---|---|
| `holder` | `Address` | — |
| `credential_type` | `String` | — |
| `issuer` | `Address` | — |
| `validity_proof` | `ZKProof` | — |
| `attribute_proof` | `ZKProof` | — |
| `encrypted_expiration` | `Bytes` | — |
| `is_verified` | `bool` | — |
| `verified_at` | `u64` | — |

#### `struct RecursiveProof`

| Field | Type | Description |
|---|---|---|
| `base_proof_id` | `BytesN<32>` | — |
| `recursive_proof` | `ZKProof` | — |
| `aggregated_vk_hash` | `BytesN<32>` | — |
| `composition_depth` | `u32` | — |
| `total_gas` | `u64` | — |
| `composed_at` | `u64` | — |

#### `struct ZKPCircuitParams`

| Field | Type | Description |
|---|---|---|
| `circuit_id` | `String` | — |
| `circuit_type` | `ZKPType` | — |
| `num_public_inputs` | `u32` | — |
| `num_private_inputs` | `u32` | — |
| `num_constraints` | `u32` | — |
| `security_param` | `u32` | — |
| `vk_hash` | `BytesN<32>` | — |
| `pk_hash` | `BytesN<32>` | — |
| `setup_at` | `u64` | — |
| `trusted_setup` | `bool` | — |

#### `struct ZKPVerificationResult`

| Field | Type | Description |
|---|---|---|
| `proof_id` | `BytesN<32>` | — |
| `is_valid` | `bool` | — |
| `gas_used` | `u64` | — |
| `verified_at` | `u64` | — |
| `verifier` | `Address` | — |
| `metadata` | `Bytes` | — |

#### `enum OptionalMultiSigConfig`

| Variant | Value | Description |
|---|---|---|
| `None` | — | — |
| `Some(MultiSigConfig)` | — | — |

#### `struct RegistryStateExport`

| Field | Type | Description |
|---|---|---|
| `format_version` | `u32` | — |
| `admin` | `Address` | — |
| `initialized` | `bool` | — |
| `paused` | `bool` | — |
| `multisig_config` | `OptionalMultiSigConfig` | — |
| `proposal_counter` | `u64` | — |
| `proposals` | `Vec<AdminProposal>` | — |

#### `enum DataKey`

| Variant | Value | Description |
|---|---|---|
| `Initialized` | — | — |
| `Admin` | — | — |
| `MultiSigConfig` | — | — |
| `ProposalCounter` | — | — |
| `ContractPaused` | — | — |
| `ProofCounter` | — | — |
| `AdminProposal(u64)` | — | — |
| `MedicalRecordProof(Address, u64)` | — | — |
| `RangeProof(BytesN<32>)` | — | — |
| `CredentialProof(Address, String)` | — | — |
| `RecursiveProof(BytesN<32>)` | — | — |
| `ZKPCircuitParams(String)` | — | — |
| `GasTracker(Address)` | — | — |
| `IssuerSalt(Address)` | — | — |
| `ZKProof(BytesN<32>)` | — | — |
| `VerificationResult(BytesN<32>)` | — | — |

#### `enum Error`

| Variant | Value | Description |
|---|---|---|
| `AlreadyInitialized` | 1 | — |
| `NotInitialized` | 2 | — |
| `NotAuthorized` | 3 | — |
| `InvalidProof` | 4 | — |
| `ProofNotFound` | 5 | — |
| `CircuitNotFound` | 6 | — |
| `VerificationFailed` | 7 | — |
| `GasLimitExceeded` | 8 | — |
| `InvalidInput` | 9 | — |
| `InvalidRange` | 10 | — |
| `CredentialExpired` | 11 | — |
| `InvalidCircuit` | 12 | — |
| `ProofTooLarge` | 13 | — |
| `RecursiveDepthExceeded` | 14 | — |
| `InvalidHashFunction` | 15 | — |
| `InsufficientFunds` | 20 | — |
| `DeadlineExceeded` | 21 | — |
| `InvalidSignature` | 22 | — |
| `UnauthorizedCaller` | 23 | — |
| `ContractPaused` | 24 | — |
| `StorageFull` | 25 | — |
| `CrossChainTimeout` | 26 | — |
| `InvalidSigner` | 27 | — |
| `InvalidThreshold` | 28 | — |
| `ProposalNotFound` | 29 | — |
| `AlreadyApproved` | 30 | — |
| `TimelockNotExpired` | 31 | — |
| `AlreadyExecuted` | 32 | — |
| `NotEnoughApprovals` | 33 | — |
| `MalformedProof` | 612 | — |
| `VkMismatch` | 613 | — |
| `InconsistentPublicInputCount` | 614 | — |
| `contains` | — | — |
| `or was tampered with.
    InvalidExpirationCiphertext = 615` | — | — |
| `InconsistentCommitment` | 616 | — |
| `InvalidProofFormat` | 617 | — |
| `BaseProofMissing` | 618 | — |

### Examples

#### `test_zkp_registry_initialization`

```rust
let env = Env::default();
    env.mock_all_auths();

    let (client, _id) = setup(&env);
    let admin = Address::generate(&env);

    client.initialize(&admin);

    let result = client.try_get_circuit_params(&String::from_str(&env, "test_circuit"));
```

#### `test_circuit_registration`

```rust
let env = Env::default();
    env.mock_all_auths();

    let (client, _id) = setup(&env);
    let admin = Address::generate(&env);
    client.initialize(&admin);

    let circuit_id = String::from_str(&env, "medical_authenticity");
    let vk_hash = BytesN::from_array(&env, &[1u8; 32]);
```

#### `test_zkp_submission_and_verification`

```rust
let env = Env::default();
    env.mock_all_auths();

    let (client, _id) = setup(&env);
    let admin = Address::generate(&env);
    client.initialize(&admin);

    let circuit_id = String::from_str(&env, "test_circuit");
    let vk_hash = BytesN::from_array(&env, &[1u8; 32]);
```

---

