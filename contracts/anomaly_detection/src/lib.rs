#![no_std]

use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype, symbol_short, Address, BytesN, Env, Map,
    String, Symbol, Vec,
};

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
    AnomalyRecord(u64), // Record ID -> AnomalyRecord
    AnomalyCountByPatient(Address),
    Stats,
    Whitelist(Address),
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
}

#[contract]
pub struct AnomalyDetectionContract;

#[contractimpl]
impl AnomalyDetectionContract {
    pub fn initialize(env: Env, admin: Address, detector: Address, threshold_bps: u32) -> bool {
        admin.require_auth();

        if env.storage().instance().has(&DataKey::Config) {
            panic!("Already initialized");
        }

        if threshold_bps > 10_000 {
            panic!("threshold_bps must be <= 10000");
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
        true
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
    ) -> Result<bool, Error> {
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

        Ok(true)
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

        let config = Self::ensure_detector(&env, &caller)?;

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
        let config = Self::ensure_admin(&env, &caller)?;

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
}

#[cfg(all(test, feature = "testutils"))]
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
        assert_eq!(config.enabled, true);

        // Detect an anomaly
        let metadata = String::from_str(&env, r#"{"feature_importance": [0.1, 0.8, 0.1]}"#);
        let explanation_ref = String::from_str(&env, "ipfs://anomaly-explanation-123");

        let anomaly_id = client
            .mock_all_auths()
            .detect_anomaly(
                &detector,
                &1u64, // record_id
                &patient,
                &8000u32, // score_bps (above threshold)
                &4u32,    // severity
                &metadata,
                &explanation_ref,
            )
            .unwrap();

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
        assert!(client
            .mock_all_auths()
            .update_config(
                &admin,
                Some(Address::generate(&env)), // new detector
                Some(8000u32),                 // new threshold
                Some(7u32),                    // new sensitivity
                Some(false),                   // disable
            )
            .is_ok());

        let config = client.get_config().unwrap();
        assert_eq!(config.threshold_bps, 8000u32);
        assert_eq!(config.sensitivity, 7u32);
        assert_eq!(config.enabled, false);
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
        assert_eq!(client.is_whitelisted_detector(&detector), false);

        // Whitelist the detector
        assert!(client
            .mock_all_auths()
            .whitelist_detector(&admin, &detector)
            .is_ok());

        // Check that detector is now whitelisted
        assert_eq!(client.is_whitelisted_detector(&detector), true);
    }
}
