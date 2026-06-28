//! anomaly_detection - Healthcare smart contract on Stellar blockchain.
// Anomaly Detection Contract - Healthcare anomaly detection with proper validation
#![no_std]
#![allow(clippy::too_many_arguments)]
#![allow(clippy::arithmetic_side_effects)]
#![allow(clippy::panic)]
#![allow(dead_code)]

use common_error::{get_suggestion as common_suggestion, CommonError};
use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype, symbol_short, Address, BytesN, Env,
    IntoVal, Map, String, Symbol,
};

// ==================== Alert Lifecycle Types ====================

#[derive(Clone, PartialEq, Eq, Debug)]
#[contracttype]
pub enum AlertStatus {
    Active,
    Acknowledged,
    Resolved,
    FalsePositive,
}

/// Alert wraps an AnomalyRecord with review lifecycle state
#[derive(Clone)]
#[contracttype]
pub struct AnomalyAlert {
    pub alert_id: u64,
    pub anomaly_id: u64,
    pub patient: Address,
    pub score_bps: u32,
    pub severity: u32,
    pub status: AlertStatus,
    pub created_at: u64,
    pub updated_at: u64,
    pub resolution_notes: String,
}

#[derive(Clone)]
#[contracttype]
pub struct AnomalyDetectionConfig {
    pub admin: Address,
    pub detector: Address,
    pub threshold_bps: u32, // Threshold in basis points (0-10000)
    pub sensitivity: u32,   // Sensitivity level (1-10)
    pub enabled: bool,
}

#[derive(Clone)]
#[contracttype]
pub struct AnomalyRecord {
    pub record_id: u64,
    pub patient: Address,
    pub detector_address: Address,
    pub score_bps: u32, // Anomaly score in basis points (0-10000)
    pub severity: u32,  // Severity level (1-5)
    pub detected_at: u64,
    pub metadata: String, // JSON string with additional detection metadata
    pub explanation_ref: String, // Off-chain reference to detailed explanation (e.g. IPFS CID)
}

#[derive(Clone)]
#[contracttype]
pub struct DetectionStats {
    pub total_anomalies: u64,
    pub high_severity_count: u64,
    pub last_detection_at: u64,
}

#[derive(Clone)]
#[contracttype]
pub enum DataKey {
    Config,
    AnomalyRecord(u64),
    AnomalyCountByPatient(Address),
    Stats,
    Whitelist(Address),
    Alert(u64),
    AlertCount,
    FeedbackCount,
    AuditForensicsContract,
}

const ANOMALY_COUNTER: Symbol = symbol_short!("ANOM_CT");

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    NotAuthorized = 1,
    ConfigNotSet = 2,
    Disabled = 3,
    InvalidScore = 4,
    InvalidSeverity = 5,
    RecordNotFound = 6,
    NotWhitelisted = 7,
    AlertNotFound = 8,
    AlertAlreadyResolved = 9,
    AlreadyInitialized = 10,
}

#[contract]
pub struct AnomalyDetectionContract;

#[contractimpl]
impl AnomalyDetectionContract {
    pub fn initialize(
        env: Env,
        admin: Address,
        detector: Address,
        threshold_bps: u32,
    ) -> Result<(), Error> {
        admin.require_auth();

        if env.storage().instance().has(&DataKey::Config) {
            return Err(Error::AlreadyInitialized);
        }

        if threshold_bps > 10_000 {
            return Err(Error::InvalidScore);
        }

        let config = AnomalyDetectionConfig {
            admin,
            detector,
            threshold_bps,
            sensitivity: 5, // Default sensitivity
            enabled: true,
        };

        env.storage().instance().set(&DataKey::Config, &config);
        env.storage().instance().set(&ANOMALY_COUNTER, &0u64);
        Ok(())
    }

    fn load_config(env: &Env) -> Result<AnomalyDetectionConfig, Error> {
        env.storage()
            .instance()
            .get(&DataKey::Config)
            .ok_or(Error::ConfigNotSet)
    }

    fn ensure_admin(env: &Env, caller: &Address) -> Result<AnomalyDetectionConfig, Error> {
        let config = Self::load_config(env)?;
        if config.admin != *caller {
            return Err(Error::NotAuthorized);
        }
        Ok(config)
    }

    fn ensure_detector(env: &Env, caller: &Address) -> Result<AnomalyDetectionConfig, Error> {
        let config = Self::load_config(env)?;
        if config.detector != *caller {
            return Err(Error::NotAuthorized);
        }
        if !config.enabled {
            return Err(Error::Disabled);
        }
        Ok(config)
    }

    fn ensure_enabled(env: &Env) -> Result<AnomalyDetectionConfig, Error> {
        let config = Self::load_config(env)?;
        if !config.enabled {
            return Err(Error::Disabled);
        }
        Ok(config)
    }

    fn next_anomaly_id(env: &Env) -> u64 {
        let current: u64 = env.storage().instance().get(&ANOMALY_COUNTER).unwrap_or(0);
        let next = current + 1;
        env.storage().instance().set(&ANOMALY_COUNTER, &next);
        next
    }

    pub fn update_config(
        env: Env,
        caller: Address,
        new_detector: Option<Address>,
        new_threshold: Option<u32>,
        new_sensitivity: Option<u32>,
        enabled: Option<bool>,
    ) -> Result<(), Error> {
        caller.require_auth();
        let mut config = Self::ensure_admin(&env, &caller)?;

        if let Some(detector) = new_detector {
            config.detector = detector;
        }

        if let Some(threshold) = new_threshold {
            if threshold > 10_000 {
                return Err(Error::InvalidScore);
            }
            config.threshold_bps = threshold;
        }

        if let Some(sensitivity) = new_sensitivity {
            if sensitivity == 0 || sensitivity > 10 {
                return Err(Error::InvalidSeverity);
            }
            config.sensitivity = sensitivity;
        }

        if let Some(enable_flag) = enabled {
            config.enabled = enable_flag;
        }

        env.storage().instance().set(&DataKey::Config, &config);
        env.events().publish((symbol_short!("CfgUpdate"),), true);

        Ok(())
    }

    pub fn set_audit_forensics(
        env: Env,
        admin: Address,
        forensics: Address,
    ) -> Result<(), Error> {
        admin.require_auth();
        Self::ensure_admin(&env, &admin)?;
        env.storage()
            .instance()
            .set(&DataKey::AuditForensicsContract, &forensics);
        Ok(())
    }

    pub fn detect_anomaly(
        env: Env,
        caller: Address,
        record_id: u64,
        patient: Address,
        score_bps: u32,
        severity: u32,
        metadata: String,
        explanation_ref: String,
    ) -> Result<u64, Error> {
        caller.require_auth();

        let _config = Self::ensure_detector(&env, &caller)?;

        // Validate inputs
        if score_bps > 10_000 {
            return Err(Error::InvalidScore);
        }

        if severity == 0 || severity > 5 {
            return Err(Error::InvalidSeverity);
        }

        if explanation_ref.is_empty() {
            panic!("explanation_ref cannot be empty");
        }

        // Create anomaly record
        let anomaly_id = Self::next_anomaly_id(&env);
        let timestamp = env.ledger().timestamp();

        let anomaly_record = AnomalyRecord {
            record_id,
            patient: patient.clone(),
            detector_address: caller.clone(),
            score_bps,
            severity,
            detected_at: timestamp,
            metadata,
            explanation_ref,
        };

        env.storage()
            .instance()
            .set(&DataKey::AnomalyRecord(anomaly_id), &anomaly_record);

        // Update patient's anomaly count
        let patient_count: u64 = env
            .storage()
            .instance()
            .get(&DataKey::AnomalyCountByPatient(patient.clone()))
            .unwrap_or(0);
        env.storage().instance().set(
            &DataKey::AnomalyCountByPatient(patient),
            &(patient_count + 1),
        );

        // Update global stats
        let mut stats: DetectionStats =
            env.storage()
                .instance()
                .get(&DataKey::Stats)
                .unwrap_or(DetectionStats {
                    total_anomalies: 0,
                    high_severity_count: 0,
                    last_detection_at: 0,
                });

        stats.total_anomalies += 1;
        if severity >= 4 {
            // High severity is 4 or 5
            stats.high_severity_count += 1;
        }
        stats.last_detection_at = timestamp;

        env.storage().instance().set(&DataKey::Stats, &stats);

        // Log to Forensics System
        if let Some(forensics_id) = env
            .storage()
            .instance()
            .get::<DataKey, Address>(&DataKey::AuditForensicsContract)
        {
            #[derive(Clone, Copy, PartialEq, Eq)]
            #[soroban_sdk::contracttype]
            enum AuditAction {
                RecordAccess,
                RecordUpdate,
                RecordDelete,
                PermissionGrant,
                PermissionRevoke,
                RecordCreated,
                AnomalyDetected,
                ComplianceReportGenerated,
                AlertTriggered,
            }

            let mut meta = Map::new(&env);
            meta.set(
                String::from_str(&env, "score"),
                String::from_str(&env, "score_placeholder"),
            );

            env.invoke_contract::<u64>(
                &forensics_id,
                &symbol_short!("log_event"),
                (
                    caller,
                    AuditAction::AnomalyDetected,
                    Some(record_id),
                    BytesN::<32>::from_array(&env, &[0u8; 32]),
                    meta,
                )
                    .into_val(&env),
            );
        }

        // Emit event
        env.events().publish(
            (symbol_short!("AnomDet"),),
            (anomaly_id, record_id, score_bps, severity),
        );

        Ok(anomaly_id)
    }

    pub fn get_anomaly_record(env: Env, anomaly_id: u64) -> Option<AnomalyRecord> {
        env.storage()
            .instance()
            .get(&DataKey::AnomalyRecord(anomaly_id))
    }

    pub fn get_config(env: Env) -> Option<AnomalyDetectionConfig> {
        env.storage().instance().get(&DataKey::Config)
    }

    pub fn get_stats(env: Env) -> DetectionStats {
        env.storage()
            .instance()
            .get(&DataKey::Stats)
            .unwrap_or(DetectionStats {
                total_anomalies: 0,
                high_severity_count: 0,
                last_detection_at: 0,
            })
    }

    pub fn get_anomaly_count_for_patient(env: Env, patient: Address) -> u64 {
        env.storage()
            .instance()
            .get(&DataKey::AnomalyCountByPatient(patient))
            .unwrap_or(0)
    }

    pub fn whitelist_detector(
        env: Env,
        caller: Address,
        detector_addr: Address,
    ) -> Result<bool, Error> {
        caller.require_auth();
        let _config = Self::ensure_admin(&env, &caller)?;

        env.storage()
            .instance()
            .set(&DataKey::Whitelist(detector_addr.clone()), &true);

        env.events()
            .publish((symbol_short!("DetectWL"),), detector_addr);

        Ok(true)
    }

    pub fn is_whitelisted_detector(env: Env, detector_addr: Address) -> bool {
        env.storage()
            .instance()
            .get(&DataKey::Whitelist(detector_addr))
            .unwrap_or(false)
    }

    // ==================== Alert Lifecycle ====================

    fn next_alert_id(env: &Env) -> u64 {
        let count: u64 = env
            .storage()
            .instance()
            .get(&DataKey::AlertCount)
            .unwrap_or(0);
        let next = count.saturating_add(1);
        env.storage().instance().set(&DataKey::AlertCount, &next);
        next
    }

    /// Promote an anomaly record to an active alert for investigation tracking.
    pub fn create_alert(env: Env, caller: Address, anomaly_id: u64) -> Result<u64, Error> {
        caller.require_auth();
        let _config = Self::ensure_admin(&env, &caller)?;

        let record: AnomalyRecord = env
            .storage()
            .instance()
            .get(&DataKey::AnomalyRecord(anomaly_id))
            .ok_or(Error::RecordNotFound)?;

        let alert_id = Self::next_alert_id(&env);
        let now = env.ledger().timestamp();

        let alert = AnomalyAlert {
            alert_id,
            anomaly_id,
            patient: record.patient,
            score_bps: record.score_bps,
            severity: record.severity,
            status: AlertStatus::Active,
            created_at: now,
            updated_at: now,
            resolution_notes: String::from_str(&env, ""),
        };

        env.storage()
            .instance()
            .set(&DataKey::Alert(alert_id), &alert);

        env.events()
            .publish((symbol_short!("AlertCrt"),), (alert_id, anomaly_id));

        Ok(alert_id)
    }

    /// Acknowledge an alert (marks it as under review).
    pub fn acknowledge_alert(env: Env, caller: Address, alert_id: u64) -> Result<bool, Error> {
        caller.require_auth();
        let _config = Self::ensure_admin(&env, &caller)?;

        let mut alert: AnomalyAlert = env
            .storage()
            .instance()
            .get(&DataKey::Alert(alert_id))
            .ok_or(Error::AlertNotFound)?;

        if alert.status != AlertStatus::Active {
            return Err(Error::AlertAlreadyResolved);
        }

        alert.status = AlertStatus::Acknowledged;
        alert.updated_at = env.ledger().timestamp();
        env.storage()
            .instance()
            .set(&DataKey::Alert(alert_id), &alert);

        env.events().publish((symbol_short!("AlertAck"),), alert_id);
        Ok(true)
    }

    /// Resolve an alert after investigation.
    pub fn resolve_alert(
        env: Env,
        caller: Address,
        alert_id: u64,
        notes: String,
    ) -> Result<bool, Error> {
        caller.require_auth();
        let _config = Self::ensure_admin(&env, &caller)?;

        let mut alert: AnomalyAlert = env
            .storage()
            .instance()
            .get(&DataKey::Alert(alert_id))
            .ok_or(Error::AlertNotFound)?;

        if alert.status == AlertStatus::Resolved || alert.status == AlertStatus::FalsePositive {
            return Err(Error::AlertAlreadyResolved);
        }

        alert.status = AlertStatus::Resolved;
        alert.updated_at = env.ledger().timestamp();
        alert.resolution_notes = notes;
        env.storage()
            .instance()
            .set(&DataKey::Alert(alert_id), &alert);

        env.events().publish((symbol_short!("AlertRes"),), alert_id);
        Ok(true)
    }

    /// Mark alert as false positive. Feeds adaptive threshold learning.
    pub fn mark_false_positive(env: Env, caller: Address, alert_id: u64) -> Result<bool, Error> {
        caller.require_auth();
        let mut config = Self::ensure_admin(&env, &caller)?;

        let mut alert: AnomalyAlert = env
            .storage()
            .instance()
            .get(&DataKey::Alert(alert_id))
            .ok_or(Error::AlertNotFound)?;

        if alert.status == AlertStatus::Resolved || alert.status == AlertStatus::FalsePositive {
            return Err(Error::AlertAlreadyResolved);
        }

        alert.status = AlertStatus::FalsePositive;
        alert.updated_at = env.ledger().timestamp();
        env.storage()
            .instance()
            .set(&DataKey::Alert(alert_id), &alert);

        // Adaptive learning: false positive → raise threshold by 50 bps
        config.threshold_bps = (config.threshold_bps + 50).min(10_000);
        env.storage().instance().set(&DataKey::Config, &config);

        env.events().publish(
            (symbol_short!("FalsePos"),),
            (alert_id, config.threshold_bps),
        );
        Ok(true)
    }

    /// Submit feedback on a detection. Adaptive threshold learning:
    /// - `confirmed = true`  → lower threshold by 50 bps (catch more)
    /// - `confirmed = false` → raise threshold by 50 bps (reduce noise)
    pub fn submit_feedback(
        env: Env,
        caller: Address,
        anomaly_id: u64,
        confirmed: bool,
    ) -> Result<bool, Error> {
        caller.require_auth();
        let mut config = Self::ensure_admin(&env, &caller)?;

        // Verify the anomaly record exists
        let _record: AnomalyRecord = env
            .storage()
            .instance()
            .get(&DataKey::AnomalyRecord(anomaly_id))
            .ok_or(Error::RecordNotFound)?;

        const LEARNING_RATE: u32 = 50;
        if confirmed {
            config.threshold_bps = config.threshold_bps.saturating_sub(LEARNING_RATE);
        } else {
            config.threshold_bps = (config.threshold_bps + LEARNING_RATE).min(10_000);
        }
        env.storage().instance().set(&DataKey::Config, &config);

        env.events().publish(
            (symbol_short!("Feedback"),),
            (anomaly_id, confirmed, config.threshold_bps),
        );
        Ok(true)
    }

    pub fn get_alert(env: Env, alert_id: u64) -> Option<AnomalyAlert> {
        env.storage().instance().get(&DataKey::Alert(alert_id))
    }

    pub fn get_alert_count(env: Env) -> u64 {
        env.storage()
            .instance()
            .get(&DataKey::AlertCount)
            .unwrap_or(0)
    }
}

pub fn get_suggestion(error: Error) -> Symbol {
    match error {
        Error::NotAuthorized => common_suggestion(CommonError::Unauthorized),
        Error::ConfigNotSet => common_suggestion(CommonError::NotInitialized),
        Error::AlreadyInitialized => common_suggestion(CommonError::AlreadyInitialized),
        Error::Disabled => common_suggestion(CommonError::InvalidState),
        Error::RecordNotFound | Error::AlertNotFound => common_suggestion(CommonError::NotFound),
        Error::NotWhitelisted => common_suggestion(CommonError::AccessDenied),
        Error::InvalidScore | Error::InvalidSeverity => {
            common_suggestion(CommonError::InvalidInput)
        },
        Error::AlertAlreadyResolved => common_suggestion(CommonError::InvalidState),
    }
}

#[cfg(all(test, feature = "testutils"))]
#[allow(clippy::unwrap_used)]
mod test {
    use super::*;
    use soroban_sdk::testutils::Address as _;

    #[test]
    fn test_anomaly_detection_flow() {
        let env = Env::default();
        let contract_id = env.register_contract(None, AnomalyDetectionContract);
        let client = AnomalyDetectionContractClient::new(&env, &contract_id);

        let admin = Address::generate(&env);
        let detector = Address::generate(&env);
        let patient = Address::generate(&env);

        // Initialize contract
        client
            .mock_all_auths()
            .initialize(&admin, &detector, &7500u32);

        // Verify config
        let config = client.get_config().unwrap();
        assert_eq!(config.admin, admin);
        assert_eq!(config.detector, detector);
        assert_eq!(config.threshold_bps, 7500u32);
        assert!(config.enabled);

        // Detect an anomaly
        let metadata = String::from_str(&env, r#"{"feature_importance": [0.1, 0.8, 0.1]}"#);
        let explanation_ref = String::from_str(&env, "ipfs://anomaly-explanation-123");

        let anomaly_id = client.mock_all_auths().detect_anomaly(
            &detector,
            &1u64, // record_id
            &patient,
            &8000u32, // score_bps (above threshold)
            &4u32,    // severity
            &metadata,
            &explanation_ref,
        );

        assert_eq!(anomaly_id, 1u64);

        // Get the anomaly record
        let record = client.get_anomaly_record(&anomaly_id).unwrap();
        assert_eq!(record.patient, patient);
        assert_eq!(record.score_bps, 8000u32);
        assert_eq!(record.severity, 4u32);
        assert_eq!(record.metadata, metadata);

        // Check stats
        let stats = client.get_stats();
        assert_eq!(stats.total_anomalies, 1);
        assert_eq!(stats.high_severity_count, 1);

        // Check patient anomaly count
        let patient_count = client.get_anomaly_count_for_patient(&patient);
        assert_eq!(patient_count, 1);
    }

    #[test]
    fn test_config_updates() {
        let env = Env::default();
        let contract_id = env.register_contract(None, AnomalyDetectionContract);
        let client = AnomalyDetectionContractClient::new(&env, &contract_id);

        let admin = Address::generate(&env);
        let detector = Address::generate(&env);

        // Initialize contract
        client
            .mock_all_auths()
            .initialize(&admin, &detector, &7500u32);

        // Update config
        assert!(client.mock_all_auths().update_config(
            &admin,
            &Some(Address::generate(&env)), // new detector
            &Some(8000u32),                 // new threshold
            &Some(7u32),                    // new sensitivity
            &Some(false),                   // disable
        ));

        let config = client.get_config().unwrap();
        assert_eq!(config.threshold_bps, 8000u32);
        assert_eq!(config.sensitivity, 7u32);
        assert!(!config.enabled);
    }

    #[test]
    fn test_whitelist_functionality() {
        let env = Env::default();
        let contract_id = env.register_contract(None, AnomalyDetectionContract);
        let client = AnomalyDetectionContractClient::new(&env, &contract_id);

        let admin = Address::generate(&env);
        let detector = Address::generate(&env);

        // Initialize contract
        client
            .mock_all_auths()
            .initialize(&admin, &detector, &7500u32);

        // Check that detector is not whitelisted initially
        assert!(!client.is_whitelisted_detector(&detector));

        // Whitelist the detector
        assert!(client
            .mock_all_auths()
            .whitelist_detector(&admin, &detector));

        // Check that detector is now whitelisted
        assert!(client.is_whitelisted_detector(&detector));
    }

    // ==================== New: Alert Lifecycle Tests ====================

    #[test]
    fn test_alert_lifecycle() {
        let env = Env::default();
        let contract_id = env.register_contract(None, AnomalyDetectionContract);
        let client = AnomalyDetectionContractClient::new(&env, &contract_id);

        let admin = Address::generate(&env);
        let detector = Address::generate(&env);
        let patient = Address::generate(&env);

        client
            .mock_all_auths()
            .initialize(&admin, &detector, &7000u32);

        // Detect an anomaly first
        let anomaly_id = client.mock_all_auths().detect_anomaly(
            &detector,
            &1u64,
            &patient,
            &8000u32,
            &4u32,
            &String::from_str(&env, r#"{"type":"bulk_access"}"#),
            &String::from_str(&env, "ipfs://explanation"),
        );

        // Create alert from the anomaly record
        let alert_id = client.mock_all_auths().create_alert(&admin, &anomaly_id);
        assert_eq!(alert_id, 1u64);
        assert_eq!(client.get_alert_count(), 1u64);

        let alert = client.get_alert(&alert_id).unwrap();
        assert_eq!(alert.status, AlertStatus::Active);
        assert_eq!(alert.score_bps, 8000u32);

        // Acknowledge
        client.mock_all_auths().acknowledge_alert(&admin, &alert_id);
        assert_eq!(
            client.get_alert(&alert_id).unwrap().status,
            AlertStatus::Acknowledged
        );

        // Resolve
        client.mock_all_auths().resolve_alert(
            &admin,
            &alert_id,
            &String::from_str(&env, "Confirmed breach, contained"),
        );
        assert_eq!(
            client.get_alert(&alert_id).unwrap().status,
            AlertStatus::Resolved
        );
    }

    #[test]
    fn test_alert_false_positive_raises_threshold() {
        let env = Env::default();
        let contract_id = env.register_contract(None, AnomalyDetectionContract);
        let client = AnomalyDetectionContractClient::new(&env, &contract_id);

        let admin = Address::generate(&env);
        let detector = Address::generate(&env);
        let patient = Address::generate(&env);

        client
            .mock_all_auths()
            .initialize(&admin, &detector, &7000u32);

        let anomaly_id = client.mock_all_auths().detect_anomaly(
            &detector,
            &1u64,
            &patient,
            &7500u32,
            &3u32,
            &String::from_str(&env, "{}"),
            &String::from_str(&env, "ipfs://expl"),
        );

        let alert_id = client.mock_all_auths().create_alert(&admin, &anomaly_id);
        let initial_threshold = client.get_config().unwrap().threshold_bps;

        client
            .mock_all_auths()
            .mark_false_positive(&admin, &alert_id);

        let updated_threshold = client.get_config().unwrap().threshold_bps;
        assert!(updated_threshold > initial_threshold);
        assert_eq!(updated_threshold, initial_threshold + 50);
        assert_eq!(
            client.get_alert(&alert_id).unwrap().status,
            AlertStatus::FalsePositive
        );
    }

    #[test]
    fn test_adaptive_threshold_feedback() {
        let env = Env::default();
        let contract_id = env.register_contract(None, AnomalyDetectionContract);
        let client = AnomalyDetectionContractClient::new(&env, &contract_id);

        let admin = Address::generate(&env);
        let detector = Address::generate(&env);
        let patient = Address::generate(&env);

        client
            .mock_all_auths()
            .initialize(&admin, &detector, &7000u32);

        let anomaly_id = client.mock_all_auths().detect_anomaly(
            &detector,
            &1u64,
            &patient,
            &8000u32,
            &4u32,
            &String::from_str(&env, "{}"),
            &String::from_str(&env, "ipfs://expl"),
        );

        let t0 = client.get_config().unwrap().threshold_bps;

        // Confirmed → lower threshold
        client
            .mock_all_auths()
            .submit_feedback(&admin, &anomaly_id, &true);
        let t1 = client.get_config().unwrap().threshold_bps;
        assert_eq!(t1, t0 - 50);

        // False positive → raise threshold
        client
            .mock_all_auths()
            .submit_feedback(&admin, &anomaly_id, &false);
        let t2 = client.get_config().unwrap().threshold_bps;
        assert_eq!(t2, t0); // back to original
    }

    #[test]
    fn test_double_resolve_fails() {
        let env = Env::default();
        let contract_id = env.register_contract(None, AnomalyDetectionContract);
        let client = AnomalyDetectionContractClient::new(&env, &contract_id);

        let admin = Address::generate(&env);
        let detector = Address::generate(&env);
        let patient = Address::generate(&env);

        client
            .mock_all_auths()
            .initialize(&admin, &detector, &7000u32);

        let anomaly_id = client.mock_all_auths().detect_anomaly(
            &detector,
            &1u64,
            &patient,
            &8000u32,
            &4u32,
            &String::from_str(&env, "{}"),
            &String::from_str(&env, "ipfs://expl"),
        );

        let alert_id = client.mock_all_auths().create_alert(&admin, &anomaly_id);
        client.mock_all_auths().resolve_alert(
            &admin,
            &alert_id,
            &String::from_str(&env, "resolved"),
        );

        let result = client.mock_all_auths().try_resolve_alert(
            &admin,
            &alert_id,
            &String::from_str(&env, "again"),
        );
        assert_eq!(result, Err(Ok(Error::AlertAlreadyResolved)));
    }
}


#![no_std]
//! anomaly_detector - Healthcare smart contract on Stellar blockchain.
#![allow(clippy::too_many_arguments)]
#![allow(clippy::arithmetic_side_effects)]
#![allow(dead_code)]

#[cfg(test)]
mod test;

use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype, symbol_short, Address, BytesN, Env,
    String, Vec,
};

// ==================== Alert & Status Types ====================

#[derive(Clone, PartialEq, Eq, Debug)]
#[contracttype]
pub enum AlertLevel {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Clone, PartialEq, Eq, Debug)]
#[contracttype]
pub enum AlertStatus {
    Active,
    Acknowledged,
    Resolved,
    FalsePositive,
}

/// Healthcare-specific anomaly pattern categories
#[derive(Clone, PartialEq, Eq, Debug)]
#[contracttype]
pub enum HealthcarePatternType {
    /// Accessing too many records in a short time window
    BulkRecordAccess,
    /// Access outside normal business hours
    UnusualTimeAccess,
    /// Unusual prescription volume or high-risk drug ratio
    PrescriptionAnomaly,
    /// Accessing records outside practitioner specialty scope
    UnauthorizedSpecialtyAccess,
    /// Very rapid sequential access to records
    RapidSequentialAccess,
    /// Attempted bulk export of records
    SuspiciousExport,
    /// Generic ML-scored anomaly (no specific pattern matched)
    MlScored,
}

// ==================== Core Data Structures ====================

/// Per-feature contribution for explainability / audit compliance
#[derive(Clone)]
#[contracttype]
pub struct FeatureContribution {
    pub feature_index: u32,
    pub feature_name: String,
    pub feature_value: u32, // 0-10000 bps (normalized input)
    pub weight: u32,        // 0-10000 bps (model weight)
    pub contribution: u32,  // feature_value * weight / 10000
}

/// Result of running anomaly inference
#[derive(Clone)]
#[contracttype]
pub struct DetectionResult {
    pub anomaly_score: u32, // 0-10000 bps
    pub is_anomalous: bool,
    pub confidence: u32, // 0-10000 bps
    pub alert_level: AlertLevel,
    pub pattern_type: HealthcarePatternType,
    pub top_features: Vec<FeatureContribution>,
    pub explanation_summary: String,
    pub detected_at: u64,
}

/// On-chain ML model: stores metadata and adapts its threshold via feedback
#[derive(Clone)]
#[contracttype]
pub struct AnomalyModel {
    pub model_id: BytesN<32>,
    pub name: String,
    pub feature_count: u32,
    pub threshold_bps: u32, // score above this → anomalous
    pub version: u32,
    pub total_inferences: u64,
    pub confirmed_anomalies: u64,
    pub false_positives: u64,
    pub created_at: u64,
    pub updated_at: u64,
}

/// Per-alert input for batched alert creation.
#[derive(Clone)]
#[contracttype]
pub struct AlertInput {
    pub patient: Address,
    pub model_id: BytesN<32>,
    pub result: DetectionResult,
    pub metadata: String,
}

/// Security alert record
#[derive(Clone)]
#[contracttype]
pub struct Alert {
    pub alert_id: u64,
    pub patient: Address,
    pub triggered_by: Address,
    pub model_id: BytesN<32>,
    pub anomaly_score: u32,
    pub alert_level: AlertLevel,
    pub status: AlertStatus,
    pub pattern_type: HealthcarePatternType,
    pub explanation_summary: String,
    pub metadata: String,
    pub created_at: u64,
    pub updated_at: u64,
}

/// Feedback for adaptive learning: confirms or refutes a flagged anomaly
#[derive(Clone)]
#[contracttype]
pub struct ModelFeedback {
    pub feedback_id: u64,
    pub alert_id: u64,
    pub model_id: BytesN<32>,
    pub submitted_by: Address,
    /// true = confirmed real anomaly (lower threshold), false = false positive (raise threshold)
    pub confirmed: bool,
    pub submitted_at: u64,
}

/// Federated learning update submission (privacy-preserving)
#[derive(Clone)]
#[contracttype]
pub struct FederatedUpdate {
    pub round_id: u64,
    pub participant: Address,
    pub update_hash: BytesN<32>,
    pub num_samples: u32,
    pub submitted_at: u64,
}

/// Per-patient rolling risk profile
#[derive(Clone)]
#[contracttype]
pub struct PatientRiskProfile {
    pub patient: Address,
    pub rolling_risk_score: u32, // 0-10000 bps EMA
    pub total_alerts: u64,
    pub active_alerts: u64,
    pub false_positive_count: u64,
    pub last_alert_at: u64,
}

// ==================== Storage Keys ====================

#[contracttype]
pub enum DataKey {
    Admin,
    Paused,
    AlertCount,
    FeedbackCount,
    /// Model weights stored separately from metadata to keep structs small
    ModelWeights(BytesN<32>),
    Model(BytesN<32>),
    Alert(u64),
    Feedback(u64),
    FederatedUpdate(u64, Address),
    PatientProfile(Address),
    Validator(Address),
}

// ==================== Errors ====================

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    AlreadyInitialized = 1,
    NotInitialized = 2,
    NotAuthorized = 3,
    ContractPaused = 4,
    ModelNotFound = 5,
    AlertNotFound = 6,
    FeatureCountMismatch = 7,
    InvalidWeight = 8,
    InvalidThreshold = 9,
    AlertAlreadyResolved = 10,
    DuplicateFederatedUpdate = 11,
    InvalidFeatureCount = 12,
    InvalidScore = 13,
    BatchTooLarge = 14,
}

// ==================== Contract ====================

#[contract]
pub struct AnomalyDetectorContract;

#[contractimpl]
impl AnomalyDetectorContract {
    // -------------------- Admin / Setup --------------------

    pub fn initialize(env: Env, admin: Address) -> Result<bool, Error> {
        admin.require_auth();
        if env.storage().instance().has(&DataKey::Admin) {
            return Err(Error::AlreadyInitialized);
        }
        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage().instance().set(&DataKey::Paused, &false);
        env.storage().instance().set(&DataKey::AlertCount, &0u64);
        env.storage().instance().set(&DataKey::FeedbackCount, &0u64);
        env.events().publish((symbol_short!("Init"),), admin);
        Ok(true)
    }

    pub fn add_validator(env: Env, caller: Address, validator: Address) -> Result<bool, Error> {
        caller.require_auth();
        Self::require_admin(&env, &caller)?;
        env.storage()
            .instance()
            .set(&DataKey::Validator(validator.clone()), &true);
        env.events()
            .publish((symbol_short!("ValAdded"),), validator);
        Ok(true)
    }

    pub fn remove_validator(env: Env, caller: Address, validator: Address) -> Result<bool, Error> {
        caller.require_auth();
        Self::require_admin(&env, &caller)?;
        env.storage()
            .instance()
            .remove(&DataKey::Validator(validator.clone()));
        env.events().publish((symbol_short!("ValRmvd"),), validator);
        Ok(true)
    }

    pub fn pause(env: Env, caller: Address) -> Result<bool, Error> {
        caller.require_auth();
        Self::require_admin(&env, &caller)?;
        env.storage().instance().set(&DataKey::Paused, &true);
        env.events().publish((symbol_short!("Paused"),), caller);
        Ok(true)
    }

    pub fn unpause(env: Env, caller: Address) -> Result<bool, Error> {
        caller.require_auth();
        Self::require_admin(&env, &caller)?;
        env.storage().instance().set(&DataKey::Paused, &false);
        env.events().publish((symbol_short!("Unpaused"),), caller);
        Ok(true)
    }

    /// Update the anomaly detection threshold for a model (admin only).
    /// `threshold_bps` must be in range 1–9999 (basis points).
    pub fn update_threshold(
        env: Env,
        caller: Address,
        model_id: BytesN<32>,
        threshold_bps: u32,
    ) -> Result<bool, Error> {
        caller.require_auth();
        Self::require_admin(&env, &caller)?;
        if threshold_bps == 0 || threshold_bps >= 10_000 {
            return Err(Error::InvalidThreshold);
        }
        let mut model: AnomalyModel = env
            .storage()
            .persistent()
            .get(&DataKey::Model(model_id.clone()))
            .ok_or(Error::ModelNotFound)?;
        model.threshold_bps = threshold_bps;
        model.updated_at = env.ledger().timestamp();
        env.storage()
            .persistent()
            .set(&DataKey::Model(model_id.clone()), &model);
        env.events()
            .publish((symbol_short!("ThrUpd"),), (model_id, threshold_bps));
        Ok(true)
    }

    /// Clear active alerts up to `count` (admin only). Pass 0 to clear all.
    /// Marks each active alert as Resolved and emits a ClearAlerts event.
    pub fn clear_alerts(env: Env, caller: Address, count: u64) -> Result<u64, Error> {
        caller.require_auth();
        Self::require_admin(&env, &caller)?;
        let total: u64 = env
            .storage()
            .instance()
            .get(&DataKey::AlertCount)
            .unwrap_or(0);
        let limit = if count == 0 || count > total {
            total
        } else {
            count
        };
        let mut cleared: u64 = 0;
        for i in 0..limit {
            if let Some(mut alert) = env
                .storage()
                .persistent()
                .get::<DataKey, Alert>(&DataKey::Alert(i))
            {
                if alert.status == AlertStatus::Active {
                    alert.status = AlertStatus::Resolved;
                    env.storage().persistent().set(&DataKey::Alert(i), &alert);
                    cleared = cleared.saturating_add(1);
                }
            }
        }
        env.events()
            .publish((symbol_short!("ClrAlrt"),), (caller, cleared));
        Ok(cleared)
    }

    // -------------------- Model Management --------------------

    /// Register an ML model with its initial feature weights.
    /// `weights` must have exactly `feature_count` elements, each 0-10000 bps.
    pub fn register_model(
        env: Env,
        caller: Address,
        model_id: BytesN<32>,
        name: String,
        feature_count: u32,
        weights: Vec<u32>,
        threshold_bps: u32,
    ) -> Result<bool, Error> {
        caller.require_auth();
        Self::require_authorized(&env, &caller)?;
        Self::require_not_paused(&env)?;

        if feature_count == 0 || feature_count > 64 {
            return Err(Error::InvalidFeatureCount);
        }
        if weights.len() != feature_count {
            return Err(Error::FeatureCountMismatch);
        }
        if threshold_bps > 10_000 {
            return Err(Error::InvalidThreshold);
        }
        for w in weights.iter() {
            if w > 10_000 {
                return Err(Error::InvalidWeight);
            }
        }

        let now = env.ledger().timestamp();
        let model = AnomalyModel {
            model_id: model_id.clone(),
            name,
            feature_count,
            threshold_bps,
            version: 1,
            total_inferences: 0,
            confirmed_anomalies: 0,
            false_positives: 0,
            created_at: now,
            updated_at: now,
        };

        env.storage()
            .persistent()
            .set(&DataKey::Model(model_id.clone()), &model);
        env.storage()
            .persistent()
            .set(&DataKey::ModelWeights(model_id.clone()), &weights);

        env.events().publish((symbol_short!("MdlReg"),), model_id);
        Ok(true)
    }

    /// Adjust a single feature weight (used by adaptive learning pipeline).
    /// `increase = true` adds `delta`; `increase = false` subtracts.
    pub fn update_model_weight(
        env: Env,
        caller: Address,
        model_id: BytesN<32>,
        feature_index: u32,
        delta: u32,
        increase: bool,
    ) -> Result<bool, Error> {
        caller.require_auth();
        Self::require_authorized(&env, &caller)?;

        let model: AnomalyModel = env
            .storage()
            .persistent()
            .get(&DataKey::Model(model_id.clone()))
            .ok_or(Error::ModelNotFound)?;

        if feature_index >= model.feature_count {
            return Err(Error::InvalidWeight);
        }

        let mut weights: Vec<u32> = env
            .storage()
            .persistent()
            .get(&DataKey::ModelWeights(model_id.clone()))
            .ok_or(Error::ModelNotFound)?;

        let current = weights.get(feature_index).unwrap_or(0);
        let updated = if increase {
            current.saturating_add(delta).min(10_000)
        } else {
            current.saturating_sub(delta)
        };
        weights.set(feature_index, updated);

        env.storage()
            .persistent()
            .set(&DataKey::ModelWeights(model_id.clone()), &weights);

        let mut m = model;
        m.version = m.version.saturating_add(1);
        m.updated_at = env.ledger().timestamp();
        env.storage()
            .persistent()
            .set(&DataKey::Model(model_id.clone()), &m);

        env.events()
            .publish((symbol_short!("WgtUpd"),), (model_id, feature_index));
        Ok(true)
    }

    // -------------------- ML Inference --------------------

    /// Run on-chain ML inference over a feature vector.
    /// Score = weighted average of normalized features (0-10000 bps).
    /// Returns explainability-ready `DetectionResult`.
    pub fn run_inference(
        env: Env,
        caller: Address,
        patient: Address,
        model_id: BytesN<32>,
        features: Vec<u32>,
        feature_names: Vec<String>,
        metadata: String,
    ) -> Result<DetectionResult, Error> {
        caller.require_auth();
        Self::require_authorized(&env, &caller)?;
        Self::require_not_paused(&env)?;

        let mut model: AnomalyModel = env
            .storage()
            .persistent()
            .get(&DataKey::Model(model_id.clone()))
            .ok_or(Error::ModelNotFound)?;

        if features.len() != model.feature_count {
            return Err(Error::FeatureCountMismatch);
        }

        let weights: Vec<u32> = env
            .storage()
            .persistent()
            .get(&DataKey::ModelWeights(model_id.clone()))
            .ok_or(Error::ModelNotFound)?;

        let (score, contributions) =
            Self::compute_weighted_score(&env, &features, &weights, &feature_names);

        let is_anomalous = score > model.threshold_bps;
        let confidence = Self::compute_confidence(score, model.threshold_bps);
        let alert_level = Self::score_to_alert_level(score);

        let summary = if is_anomalous {
            String::from_str(&env, "ML inference: anomaly detected above threshold")
        } else {
            String::from_str(&env, "ML inference: score within normal range")
        };

        let result = DetectionResult {
            anomaly_score: score,
            is_anomalous,
            confidence,
            alert_level,
            pattern_type: HealthcarePatternType::MlScored,
            top_features: contributions,
            explanation_summary: summary,
            detected_at: env.ledger().timestamp(),
        };

        model.total_inferences = model.total_inferences.saturating_add(1);
        env.storage()
            .persistent()
            .set(&DataKey::Model(model_id.clone()), &model);

        if is_anomalous {
            Self::update_patient_profile_score(&env, &patient, score);
        }

        env.events().publish(
            (symbol_short!("Infer"),),
            (model_id, patient, score, is_anomalous),
        );

        let _ = metadata;
        Ok(result)
    }

    // -------------------- Healthcare-Specific Patterns --------------------

    /// Detect prescription anomaly patterns.
    ///
    /// Scoring (weighted average, threshold = 5000 bps):
    /// - `high_risk_ratio` (40%): high_risk_count / drug_count
    /// - `drug_rate_score` (35%): prescriptions per hour, normalized
    /// - `pharmacy_dispersion` (25%): distinct pharmacy count, normalized
    pub fn detect_prescription_anomaly(
        env: Env,
        caller: Address,
        patient: Address,
        drug_count: u32,
        high_risk_count: u32,
        unique_pharmacies: u32,
        time_window_hours: u32,
        metadata: String,
    ) -> Result<DetectionResult, Error> {
        caller.require_auth();
        Self::require_authorized(&env, &caller)?;
        Self::require_not_paused(&env)?;

        // Feature 1: high-risk drug ratio (0-10000)
        let high_risk_ratio = if drug_count > 0 {
            high_risk_count * 10_000 / drug_count
        } else {
            0
        };

        // Feature 2: prescriptions per hour, normalized (>10/hr → 10000)
        let drug_rate_score = if time_window_hours > 0 {
            (drug_count * 1_000 / time_window_hours).min(10_000)
        } else {
            drug_count.saturating_mul(1_000).min(10_000)
        };

        // Feature 3: pharmacy dispersion (4+ pharmacies → 10000)
        let pharmacy_score = unique_pharmacies.saturating_mul(2_500).min(10_000);

        // Weighted average: high_risk 40%, pharmacy_dispersion 45%, rate 15%
        // Dispersion gets highest weight as multi-pharmacy shopping is hardest to explain legitimately
        let score = (high_risk_ratio * 40 + pharmacy_score * 45 + drug_rate_score * 15) / 100;

        let is_anomalous = score > 5_000;
        let confidence = Self::compute_confidence(score, 5_000);
        let alert_level = Self::score_to_alert_level(score);

        let mut top_features = Vec::new(&env);
        top_features.push_back(FeatureContribution {
            feature_index: 0,
            feature_name: String::from_str(&env, "high_risk_ratio"),
            feature_value: high_risk_ratio,
            weight: 4_000,
            contribution: high_risk_ratio * 40 / 100,
        });
        top_features.push_back(FeatureContribution {
            feature_index: 1,
            feature_name: String::from_str(&env, "pharmacy_dispersion"),
            feature_value: pharmacy_score,
            weight: 4_500,
            contribution: pharmacy_score * 45 / 100,
        });
        top_features.push_back(FeatureContribution {
            feature_index: 2,
            feature_name: String::from_str(&env, "drug_rate_per_hour"),
            feature_value: drug_rate_score,
            weight: 1_500,
            contribution: drug_rate_score * 15 / 100,
        });

        let summary = if is_anomalous {
            String::from_str(&env, "Prescription anomaly: unusual pattern detected")
        } else {
            String::from_str(&env, "Prescription pattern within normal range")
        };

        let result = DetectionResult {
            anomaly_score: score,
            is_anomalous,
            confidence,
            alert_level,
            pattern_type: HealthcarePatternType::PrescriptionAnomaly,
            top_features,
            explanation_summary: summary,
            detected_at: env.ledger().timestamp(),
        };

        if is_anomalous {
            Self::update_patient_profile_score(&env, &patient, score);
        }

        env.events().publish(
            (symbol_short!("PrescAnm"),),
            (patient, score, drug_count, high_risk_count),
        );

        let _ = metadata;
        Ok(result)
    }

    /// Detect access behavior anomalies.
    ///
    /// Scoring (threshold = 5000 bps):
    /// - `access_count` (45%): absolute access count (30+ → max score)
    /// - `after_hours` (35%): 8000 bps if is_after_hours, else 0
    /// - `record_type_diversity` (20%): distinct record types accessed
    pub fn detect_access_anomaly(
        env: Env,
        caller: Address,
        patient: Address,
        access_count: u32,
        time_window_secs: u32,
        is_after_hours: bool,
        distinct_record_types: u32,
        metadata: String,
    ) -> Result<DetectionResult, Error> {
        caller.require_auth();
        Self::require_authorized(&env, &caller)?;
        Self::require_not_paused(&env)?;

        // Feature 1: absolute access count (30+ records → 10000; 333 bps per record)
        let count_score = access_count.saturating_mul(333).min(10_000);

        // Feature 2: after-hours access (strong signal: 8000 bps)
        let after_hours_score: u32 = if is_after_hours { 8_000 } else { 0 };

        // Feature 3: record type diversity (5+ types → 10000)
        let bulk_score = distinct_record_types.saturating_mul(2_000).min(10_000);

        // Weighted average: count 45%, after_hours 35%, diversity 20%
        let score = (count_score * 45 + after_hours_score * 35 + bulk_score * 20) / 100;

        // Classify pattern type based on observable signals
        let pattern_type = if is_after_hours && access_count > 10 {
            HealthcarePatternType::UnusualTimeAccess
        } else if access_count > 20 {
            HealthcarePatternType::BulkRecordAccess
        } else if time_window_secs < 60 && access_count > 5 {
            HealthcarePatternType::RapidSequentialAccess
        } else {
            HealthcarePatternType::MlScored
        };

        let is_anomalous = score > 5_000;
        let confidence = Self::compute_confidence(score, 5_000);
        let alert_level = Self::score_to_alert_level(score);

        let mut top_features = Vec::new(&env);
        top_features.push_back(FeatureContribution {
            feature_index: 0,
            feature_name: String::from_str(&env, "access_count"),
            feature_value: count_score,
            weight: 4_500,
            contribution: count_score * 45 / 100,
        });
        top_features.push_back(FeatureContribution {
            feature_index: 1,
            feature_name: String::from_str(&env, "after_hours"),
            feature_value: after_hours_score,
            weight: 3_500,
            contribution: after_hours_score * 35 / 100,
        });
        top_features.push_back(FeatureContribution {
            feature_index: 2,
            feature_name: String::from_str(&env, "record_type_diversity"),
            feature_value: bulk_score,
            weight: 2_000,
            contribution: bulk_score * 20 / 100,
        });

        let summary = if is_anomalous {
            String::from_str(&env, "Access anomaly: unusual access pattern detected")
        } else {
            String::from_str(&env, "Access pattern within normal range")
        };

        let result = DetectionResult {
            anomaly_score: score,
            is_anomalous,
            confidence,
            alert_level,
            pattern_type,
            top_features,
            explanation_summary: summary,
            detected_at: env.ledger().timestamp(),
        };

        if is_anomalous {
            Self::update_patient_profile_score(&env, &patient, score);
        }

        env.events().publish(
            (symbol_short!("AccAnm"),),
            (patient, score, access_count, is_after_hours),
        );

        let _ = metadata;
        Ok(result)
    }

    // -------------------- Alert Lifecycle --------------------

    /// Create a real-time alert from a `DetectionResult`. Returns the new alert_id.
    pub fn create_alert(
        env: Env,
        caller: Address,
        patient: Address,
        model_id: BytesN<32>,
        result: DetectionResult,
        metadata: String,
    ) -> Result<u64, Error> {
        caller.require_auth();
        Self::require_authorized(&env, &caller)?;
        Self::require_not_paused(&env)?;

        let alert_id = Self::next_alert_id(&env);
        let now = env.ledger().timestamp();

        let alert = Alert {
            alert_id,
            patient: patient.clone(),
            triggered_by: caller,
            model_id,
            anomaly_score: result.anomaly_score,
            alert_level: result.alert_level,
            status: AlertStatus::Active,
            pattern_type: result.pattern_type,
            explanation_summary: result.explanation_summary,
            metadata,
            created_at: now,
            updated_at: now,
        };

        env.storage()
            .persistent()
            .set(&DataKey::Alert(alert_id), &alert);

        Self::increment_patient_active_alerts(&env, &patient);

        env.events().publish(
            (symbol_short!("AlertCrt"),),
            (alert_id, patient, alert.anomaly_score),
        );

        Ok(alert_id)
    }

    /// Create multiple alerts in a single atomic call.
    ///
    /// All-or-nothing semantics: if any alert fails, the entire batch is
    /// rejected and no alerts are persisted.
    ///
    /// ## Limits
    /// - Max 50 alerts per batch.
    pub fn create_alert_batch(
        env: Env,
        caller: Address,
        alerts: Vec<AlertInput>,
    ) -> Result<Vec<u64>, Error> {
        caller.require_auth();
        Self::require_authorized(&env, &caller)?;
        Self::require_not_paused(&env)?;

        let count = alerts.len();
        if count == 0 || count > 50 {
            return Err(Error::BatchTooLarge);
        }

        let mut ids: Vec<u64> = Vec::new(&env);
        for input in alerts.iter() {
            let alert_id = Self::next_alert_id(&env);
            let now = env.ledger().timestamp();

            let alert = Alert {
                alert_id,
                patient: input.patient.clone(),
                triggered_by: caller.clone(),
                model_id: input.model_id,
                anomaly_score: input.result.anomaly_score,
                alert_level: input.result.alert_level,
                status: AlertStatus::Active,
                pattern_type: input.result.pattern_type,
                explanation_summary: input.result.explanation_summary,
                metadata: input.metadata,
                created_at: now,
                updated_at: now,
            };

            env.storage()
                .persistent()
                .set(&DataKey::Alert(alert_id), &alert);

            Self::increment_patient_active_alerts(&env, &input.patient);

            env.events().publish(
                (symbol_short!("AlertCrt"),),
                (alert_id, input.patient, alert.anomaly_score),
            );

            ids.push_back(alert_id);
        }

        Ok(ids)
    }

    /// Acknowledge an active alert (marks as reviewed, does not close).
    pub fn acknowledge_alert(env: Env, caller: Address, alert_id: u64) -> Result<bool, Error> {
        caller.require_auth();
        Self::require_authorized(&env, &caller)?;

        let mut alert: Alert = env
            .storage()
            .persistent()
            .get(&DataKey::Alert(alert_id))
            .ok_or(Error::AlertNotFound)?;

        if alert.status != AlertStatus::Active {
            return Err(Error::AlertAlreadyResolved);
        }

        alert.status = AlertStatus::Acknowledged;
        alert.updated_at = env.ledger().timestamp();
        env.storage()
            .persistent()
            .set(&DataKey::Alert(alert_id), &alert);

        env.events()
            .publish((symbol_short!("AlertAck"),), (alert_id, caller));
        Ok(true)
    }

    /// Resolve an alert after investigation. Accepted from Active or Acknowledged state.
    pub fn resolve_alert(
        env: Env,
        caller: Address,
        alert_id: u64,
        resolution_notes: String,
    ) -> Result<bool, Error> {
        caller.require_auth();
        Self::require_authorized(&env, &caller)?;

        let mut alert: Alert = env
            .storage()
            .persistent()
            .get(&DataKey::Alert(alert_id))
            .ok_or(Error::AlertNotFound)?;

        if alert.status == AlertStatus::Resolved || alert.status == AlertStatus::FalsePositive {
            return Err(Error::AlertAlreadyResolved);
        }

        alert.status = AlertStatus::Resolved;
        alert.updated_at = env.ledger().timestamp();
        env.storage()
            .persistent()
            .set(&DataKey::Alert(alert_id), &alert);

        Self::decrement_patient_active_alerts(&env, &alert.patient);

        env.events().publish(
            (symbol_short!("AlertRes"),),
            (alert_id, caller, resolution_notes),
        );
        Ok(true)
    }

    /// Mark an alert as false positive, automatically feeding adaptive learning.
    pub fn mark_false_positive(env: Env, caller: Address, alert_id: u64) -> Result<bool, Error> {
        caller.require_auth();
        Self::require_authorized(&env, &caller)?;

        let mut alert: Alert = env
            .storage()
            .persistent()
            .get(&DataKey::Alert(alert_id))
            .ok_or(Error::AlertNotFound)?;

        if alert.status == AlertStatus::Resolved || alert.status == AlertStatus::FalsePositive {
            return Err(Error::AlertAlreadyResolved);
        }

        alert.status = AlertStatus::FalsePositive;
        alert.updated_at = env.ledger().timestamp();
        env.storage()
            .persistent()
            .set(&DataKey::Alert(alert_id), &alert);

        Self::decrement_patient_active_alerts(&env, &alert.patient);
        Self::increment_patient_false_positives(&env, &alert.patient);

        env.events()
            .publish((symbol_short!("FalsePos"),), (alert_id, caller));
        Ok(true)
    }

    // -------------------- Adaptive Learning --------------------

    /// Submit feedback confirming or refuting an alert.
    ///
    /// - `confirmed = true`: real anomaly → lower model threshold by LEARNING_RATE (more sensitive)
    /// - `confirmed = false`: false positive → raise threshold by LEARNING_RATE (less noisy)
    ///
    /// Learning rate: 50 bps (0.5%) per feedback signal.
    pub fn submit_feedback(
        env: Env,
        caller: Address,
        alert_id: u64,
        model_id: BytesN<32>,
        confirmed: bool,
    ) -> Result<u64, Error> {
        caller.require_auth();
        Self::require_authorized(&env, &caller)?;

        let _alert: Alert = env
            .storage()
            .persistent()
            .get(&DataKey::Alert(alert_id))
            .ok_or(Error::AlertNotFound)?;

        let mut model: AnomalyModel = env
            .storage()
            .persistent()
            .get(&DataKey::Model(model_id.clone()))
            .ok_or(Error::ModelNotFound)?;

        const LEARNING_RATE: u32 = 50;
        if confirmed {
            // True positive: lower threshold to catch similar cases
            model.threshold_bps = model.threshold_bps.saturating_sub(LEARNING_RATE);
            model.confirmed_anomalies = model.confirmed_anomalies.saturating_add(1);
        } else {
            // False positive: raise threshold to reduce noise
            model.threshold_bps = (model.threshold_bps + LEARNING_RATE).min(10_000);
            model.false_positives = model.false_positives.saturating_add(1);
        }
        model.updated_at = env.ledger().timestamp();
        env.storage()
            .persistent()
            .set(&DataKey::Model(model_id.clone()), &model);

        let feedback_id = Self::next_feedback_id(&env);
        let feedback = ModelFeedback {
            feedback_id,
            alert_id,
            model_id: model_id.clone(),
            submitted_by: caller.clone(),
            confirmed,
            submitted_at: env.ledger().timestamp(),
        };
        env.storage()
            .persistent()
            .set(&DataKey::Feedback(feedback_id), &feedback);

        env.events().publish(
            (symbol_short!("Feedback"),),
            (feedback_id, alert_id, model_id, confirmed),
        );

        Ok(feedback_id)
    }

    // -------------------- Federated Learning --------------------

    /// Submit a privacy-preserving model update for a federated learning round.
    /// The `update_hash` commits to gradient updates without exposing patient data.
    /// Duplicate submissions per (round_id, participant) are rejected.
    pub fn submit_federated_update(
        env: Env,
        participant: Address,
        round_id: u64,
        update_hash: BytesN<32>,
        num_samples: u32,
    ) -> Result<bool, Error> {
        participant.require_auth();
        Self::require_not_paused(&env)?;

        let key = DataKey::FederatedUpdate(round_id, participant.clone());
        if env.storage().persistent().has(&key) {
            return Err(Error::DuplicateFederatedUpdate);
        }

        let update = FederatedUpdate {
            round_id,
            participant: participant.clone(),
            update_hash,
            num_samples,
            submitted_at: env.ledger().timestamp(),
        };
        env.storage().persistent().set(&key, &update);

        env.events().publish(
            (symbol_short!("FedUpd"),),
            (round_id, participant, num_samples),
        );
        Ok(true)
    }

    // -------------------- Read Functions --------------------

    pub fn get_alert(env: Env, alert_id: u64) -> Option<Alert> {
        env.storage().persistent().get(&DataKey::Alert(alert_id))
    }

    pub fn get_model(env: Env, model_id: BytesN<32>) -> Option<AnomalyModel> {
        env.storage().persistent().get(&DataKey::Model(model_id))
    }

    pub fn get_model_weights(env: Env, model_id: BytesN<32>) -> Option<Vec<u32>> {
        env.storage()
            .persistent()
            .get(&DataKey::ModelWeights(model_id))
    }

    pub fn get_patient_profile(env: Env, patient: Address) -> Option<PatientRiskProfile> {
        env.storage()
            .persistent()
            .get(&DataKey::PatientProfile(patient))
    }

    pub fn get_alert_count(env: Env) -> u64 {
        env.storage()
            .instance()
            .get(&DataKey::AlertCount)
            .unwrap_or(0)
    }

    pub fn get_feedback(env: Env, feedback_id: u64) -> Option<ModelFeedback> {
        env.storage()
            .persistent()
            .get(&DataKey::Feedback(feedback_id))
    }

    pub fn get_federated_update(
        env: Env,
        round_id: u64,
        participant: Address,
    ) -> Option<FederatedUpdate> {
        env.storage()
            .persistent()
            .get(&DataKey::FederatedUpdate(round_id, participant))
    }

    pub fn is_paused(env: Env) -> bool {
        env.storage()
            .instance()
            .get(&DataKey::Paused)
            .unwrap_or(false)
    }

    pub fn is_validator(env: Env, addr: Address) -> bool {
        env.storage()
            .instance()
            .get(&DataKey::Validator(addr))
            .unwrap_or(false)
    }

    // ==================== Internal Helpers ====================

    fn require_admin(env: &Env, caller: &Address) -> Result<(), Error> {
        let admin: Address = env
            .storage()
            .instance()
            .get(&DataKey::Admin)
            .ok_or(Error::NotInitialized)?;
        if admin != *caller {
            return Err(Error::NotAuthorized);
        }
        Ok(())
    }

    fn require_authorized(env: &Env, caller: &Address) -> Result<(), Error> {
        let admin: Option<Address> = env.storage().instance().get(&DataKey::Admin);
        if let Some(a) = admin {
            if a == *caller {
                return Ok(());
            }
        }
        let is_validator: bool = env
            .storage()
            .instance()
            .get(&DataKey::Validator(caller.clone()))
            .unwrap_or(false);
        if is_validator {
            return Ok(());
        }
        Err(Error::NotAuthorized)
    }

    fn require_not_paused(env: &Env) -> Result<(), Error> {
        let paused: bool = env
            .storage()
            .instance()
            .get(&DataKey::Paused)
            .unwrap_or(false);
        if paused {
            return Err(Error::ContractPaused);
        }
        Ok(())
    }

    fn next_alert_id(env: &Env) -> u64 {
        let count: u64 = env
            .storage()
            .instance()
            .get(&DataKey::AlertCount)
            .unwrap_or(0);
        let next = count.saturating_add(1);
        env.storage().instance().set(&DataKey::AlertCount, &next);
        next
    }

    fn next_feedback_id(env: &Env) -> u64 {
        let count: u64 = env
            .storage()
            .instance()
            .get(&DataKey::FeedbackCount)
            .unwrap_or(0);
        let next = count.saturating_add(1);
        env.storage().instance().set(&DataKey::FeedbackCount, &next);
        next
    }

    /// Weighted-average score: Σ(f_i × w_i) / Σ(w_i), capped at 10000 bps.
    /// Also returns per-feature `FeatureContribution` structs for explainability.
    fn compute_weighted_score(
        env: &Env,
        features: &Vec<u32>,
        weights: &Vec<u32>,
        feature_names: &Vec<String>,
    ) -> (u32, Vec<FeatureContribution>) {
        let n = features.len().min(weights.len());
        let mut weighted_sum: u64 = 0;
        let mut total_weight: u64 = 0;
        let mut contributions = Vec::new(env);

        for i in 0..n {
            let f = features.get(i).unwrap_or(0) as u64;
            let w = weights.get(i).unwrap_or(0) as u64;
            weighted_sum = weighted_sum.saturating_add(f.saturating_mul(w));
            total_weight = total_weight.saturating_add(w);

            let contrib = if w > 0 {
                ((f.saturating_mul(w)) / 10_000) as u32
            } else {
                0
            };
            let name = feature_names
                .get(i)
                .unwrap_or_else(|| String::from_str(env, "unknown"));
            contributions.push_back(FeatureContribution {
                feature_index: i,
                feature_name: name,
                feature_value: f as u32,
                weight: w as u32,
                contribution: contrib,
            });
        }

        let score = if total_weight > 0 {
            ((weighted_sum / total_weight) as u32).min(10_000)
        } else {
            0
        };

        (score, contributions)
    }

    /// Linear confidence mapping:
    /// - Anomalous (score > threshold): maps [threshold, 10000] → [5000, 10000]
    /// - Normal (score ≤ threshold): maps [0, threshold] → [0, 5000]
    fn compute_confidence(score: u32, threshold: u32) -> u32 {
        if score > threshold {
            if threshold >= 10_000 {
                return 5_000;
            }
            let range = 10_000 - threshold;
            let dist = score - threshold;
            5_000 + ((dist as u64 * 5_000) / range as u64).min(5_000) as u32
        } else {
            if threshold == 0 {
                return 0;
            }
            let dist = threshold - score;
            ((dist as u64 * 5_000) / threshold as u64).min(5_000) as u32
        }
    }

    fn score_to_alert_level(score: u32) -> AlertLevel {
        if score > 7_500 {
            AlertLevel::Critical
        } else if score > 5_000 {
            AlertLevel::High
        } else if score > 2_500 {
            AlertLevel::Medium
        } else {
            AlertLevel::Low
        }
    }

    /// Update patient's rolling risk score using exponential moving average (α=0.3).
    fn update_patient_profile_score(env: &Env, patient: &Address, new_score: u32) {
        let mut profile: PatientRiskProfile = env
            .storage()
            .persistent()
            .get(&DataKey::PatientProfile(patient.clone()))
            .unwrap_or(PatientRiskProfile {
                patient: patient.clone(),
                rolling_risk_score: 0,
                total_alerts: 0,
                active_alerts: 0,
                false_positive_count: 0,
                last_alert_at: 0,
            });

        // EMA: new = 0.3 * new_score + 0.7 * old
        profile.rolling_risk_score = (3 * new_score + 7 * profile.rolling_risk_score) / 10;
        profile.total_alerts = profile.total_alerts.saturating_add(1);
        profile.last_alert_at = env.ledger().timestamp();

        env.storage()
            .persistent()
            .set(&DataKey::PatientProfile(patient.clone()), &profile);
    }

    fn increment_patient_active_alerts(env: &Env, patient: &Address) {
        let mut profile: PatientRiskProfile = env
            .storage()
            .persistent()
            .get(&DataKey::PatientProfile(patient.clone()))
            .unwrap_or(PatientRiskProfile {
                patient: patient.clone(),
                rolling_risk_score: 0,
                total_alerts: 0,
                active_alerts: 0,
                false_positive_count: 0,
                last_alert_at: 0,
            });
        profile.active_alerts = profile.active_alerts.saturating_add(1);
        env.storage()
            .persistent()
            .set(&DataKey::PatientProfile(patient.clone()), &profile);
    }

    fn decrement_patient_active_alerts(env: &Env, patient: &Address) {
        if let Some(mut profile) = env
            .storage()
            .persistent()
            .get::<DataKey, PatientRiskProfile>(&DataKey::PatientProfile(patient.clone()))
        {
            profile.active_alerts = profile.active_alerts.saturating_sub(1);
            env.storage()
                .persistent()
                .set(&DataKey::PatientProfile(patient.clone()), &profile);
        }
    }

    fn increment_patient_false_positives(env: &Env, patient: &Address) {
        if let Some(mut profile) = env
            .storage()
            .persistent()
            .get::<DataKey, PatientRiskProfile>(&DataKey::PatientProfile(patient.clone()))
        {
            profile.false_positive_count = profile.false_positive_count.saturating_add(1);
            env.storage()
                .persistent()
                .set(&DataKey::PatientProfile(patient.clone()), &profile);
        }
    }
}
