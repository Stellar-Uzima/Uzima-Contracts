// Anomaly Detection Contract - Healthcare anomaly detection with proper validation
#![no_std]
#![allow(clippy::too_many_arguments)]
#![allow(clippy::arithmetic_side_effects)]
#![allow(clippy::panic)]
#![allow(dead_code)]

use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype, symbol_short, Address, BytesN, Env,
    IntoVal, Map, String, Symbol, Vec,
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
pub struct ProviderProfile {
    pub provider: Address,
    pub total_claims: u64,
    pub total_amount: i128,
    pub high_risk_claims: u64,
    pub risk_score_bps: u32,
}

#[derive(Clone)]
#[contracttype]
pub enum DataKey {
    Config,
    AnomalyRecord(u64),
    AnomalyCountByPatient(Address),
    Stats,
    Whitelist(Address),
    ProviderProfile(Address),
    ProviderNeighbors(Address),
    PatientProviders(Address),
    PaymentContract,
    Alert(u64),
    AlertCount,
    FeedbackCount,
    ConfirmedFeedbackCount,
    FalsePositiveFeedbackCount,
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

    fn get_provider_profile(env: &Env, provider: &Address) -> ProviderProfile {
        env.storage()
            .instance()
            .get(&DataKey::ProviderProfile(provider.clone()))
            .unwrap_or(ProviderProfile {
                provider: provider.clone(),
                total_claims: 0,
                total_amount: 0,
                high_risk_claims: 0,
                risk_score_bps: 0,
            })
    }

    fn set_provider_profile(env: &Env, profile: ProviderProfile) {
        env.storage()
            .instance()
            .set(&DataKey::ProviderProfile(profile.provider.clone()), &profile);
    }

    fn add_provider_neighbor(env: &Env, provider: Address, neighbor: Address) {
        if provider == neighbor {
            return;
        }
        let mut neighbors: Vec<Address> = env
            .storage()
            .instance()
            .get(&DataKey::ProviderNeighbors(provider.clone()))
            .unwrap_or(Vec::new(&env));

        let mut exists = false;
        for existing in neighbors.iter() {
            if existing == neighbor {
                exists = true;
                break;
            }
        }

        if !exists {
            neighbors.push_back(neighbor.clone());
            env.storage()
                .instance()
                .set(&DataKey::ProviderNeighbors(provider.clone()), &neighbors);
        }
    }

    fn add_patient_provider(env: &Env, patient: Address, provider: Address) {
        let mut providers: Vec<Address> = env
            .storage()
            .instance()
            .get(&DataKey::PatientProviders(patient.clone()))
            .unwrap_or(Vec::new(&env));

        let mut exists = false;
        for p in providers.iter() {
            if p == provider {
                exists = true;
                break;
            }
        }

        if !exists {
            for p in providers.iter() {
                Self::add_provider_neighbor(env, p.clone(), provider.clone());
                Self::add_provider_neighbor(env, provider.clone(), p.clone());
            }
            providers.push_back(provider.clone());
            env.storage()
                .instance()
                .set(&DataKey::PatientProviders(patient), &providers);
        }
    }

    fn compute_network_influence(env: &Env, provider: &Address) -> u32 {
        let neighbors: Vec<Address> = env
            .storage()
            .instance()
            .get(&DataKey::ProviderNeighbors(provider.clone()))
            .unwrap_or(Vec::new(&env));

        let mut influence: u32 = 0;
        for neighbor in neighbors.iter() {
            let profile = Self::get_provider_profile(env, &neighbor);
            influence = influence.saturating_add(profile.risk_score_bps / 10);
        }
        influence.min(5000) // cap influence at 50%
    }

    fn recompute_provider_risk(env: &Env, provider: &Address) {
        let mut profile = Self::get_provider_profile(env, provider);
        if profile.total_claims == 0 {
            profile.risk_score_bps = 0;
        } else {
            let base_risk = (profile.high_risk_claims.saturating_mul(10_000) / profile.total_claims)
                .min(10_000);
            let network = Self::compute_network_influence(env, provider);
            profile.risk_score_bps = (base_risk.saturating_add(network)).min(10_000);
        }
        Self::set_provider_profile(env, profile);
    }

    pub fn assess_payment_claim(
        env: Env,
        caller: Address,
        claim_id: u64,
        provider: Address,
        patient: Address,
        amount: i128,
        service_id: String,
    ) -> Result<u32, Error> {
        let config = Self::ensure_enabled(&env)?;

        // if payment contract is set, enforce it for caller
        if let Some(payment_contract) = env.storage().instance().get(&DataKey::PaymentContract) {
            if payment_contract != caller {
                return Err(Error::NotAuthorized);
            }
        }

        let mut profile = Self::get_provider_profile(&env, &provider);
        profile.total_claims = profile.total_claims.saturating_add(1);
        profile.total_amount = profile.total_amount.saturating_add(amount);

        let mut claim_score = 0u32;
        if amount > 10_000 {
            claim_score = claim_score.saturating_add(3_000);
        }
        if amount > 50_000 {
            claim_score = claim_score.saturating_add(3_000);
        }
        if claim_score == 0 {
            claim_score = 500;
        }

        if profile.total_claims > 20 && profile.total_amount as u128 > 1_000_000u128 {
            profile.high_risk_claims = profile.high_risk_claims.saturating_add(1);
            claim_score = claim_score.saturating_add(2_000);
        }

        Self::add_patient_provider(&env, patient.clone(), provider.clone());
        Self::set_provider_profile(&env, profile);
        Self::recompute_provider_risk(&env, &provider);

        let provider_risk = Self::get_provider_profile(&env, &provider).risk_score_bps;
        let network_risk = Self::compute_network_influence(&env, &provider);
        let final_score = (claim_score + provider_risk + network_risk).min(10_000);

        if final_score >= config.threshold_bps {
            let metadata = String::from_str(&env, "{\"pattern\":\"network_behavior\"}");
            let explanation_ref = String::from_str(&env, "ipfs://network-fraud-explain");

            let _ = Self::detect_anomaly(
                env.clone(),
                config.detector.clone(),
                claim_id,
                patient.clone(),
                final_score,
                4,
                metadata,
                explanation_ref,
            );

            env.events().publish((symbol_short!("RealTime"),), (claim_id, final_score));
        }

        Ok(final_score)
    }

    pub fn get_provider_profile_record(env: Env, provider: Address) -> ProviderProfile {
        Self::get_provider_profile(&env, &provider)
    }

    pub fn get_detection_performance(env: Env) -> (u32, u32) {
        let confirmed: u64 = env
            .storage()
            .instance()
            .get(&DataKey::ConfirmedFeedbackCount)
            .unwrap_or(0);
        let false_pos: u64 = env
            .storage()
            .instance()
            .get(&DataKey::FalsePositiveFeedbackCount)
            .unwrap_or(0);
        let total = confirmed.saturating_add(false_pos);
        if total == 0 {
            return (10_000, 0); // 100% accuracy if no feedback yet
        }

        let accuracy = ((confirmed as u128).saturating_mul(10_000) / (total as u128)) as u32;
        let false_rate = ((false_pos as u128).saturating_mul(10_000) / (total as u128)) as u32;
        (accuracy.min(10_000), false_rate.min(10_000))
    }

    pub fn detect_network_fraud(env: Env, provider: Address) -> u32 {
        let profile = Self::get_provider_profile(&env, &provider);
        let net = Self::compute_network_influence(&env, &provider);
        (profile.risk_score_bps.saturating_add(net)).min(10_000)
    }

    pub fn report_provider_behavior(
        env: Env,
        provider: Address,
        claim_id: u64,
        score_bps: u32,
    ) -> bool {
        let mut profile = Self::get_provider_profile(&env, &provider);
        profile.total_claims = profile.total_claims.saturating_add(1);
        if score_bps >= 7000 {
            profile.high_risk_claims = profile.high_risk_claims.saturating_add(1);
        }
        Self::set_provider_profile(&env, profile);
        Self::recompute_provider_risk(&env, &provider);
        env.events().publish((symbol_short!("ProvBeh"),), (provider, claim_id, score_bps));
        true
    }

    pub fn create_realtime_alert(env: Env, caller: Address, anomaly_id: u64) -> Result<u64, Error> {
        // pass through to existing create_alert;
        Self::create_alert(env, caller, anomaly_id)
    }

    pub fn get_alert_accuracy_rates(env: Env) -> (u32, u32) {
        Self::get_detection_performance(env)
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

    pub fn set_audit_forensics(
        env: Env,
        admin: Address,
        forensics: Address,
    ) -> Result<bool, Error> {
        admin.require_auth();
        Self::ensure_admin(&env, &admin)?;
        env.storage()
            .instance()
            .set(&DataKey::AuditForensicsContract, &forensics);
        Ok(true)
    }

    pub fn set_payment_contract(
        env: Env,
        admin: Address,
        payment_contract: Address,
    ) -> Result<bool, Error> {
        admin.require_auth();
        Self::ensure_admin(&env, &admin)?;
        env.storage()
            .instance()
            .set(&DataKey::PaymentContract, &payment_contract);
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

        // Real-time alerting for high-severity or high-score anomalies
        if severity >= 4 || score_bps >= 7500 {
            env.events().publish(
                (symbol_short!("RealTimeAlert"),),
                (anomaly_id, record_id, score_bps, severity),
            );
        }

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
            let current: u64 = env
                .storage()
                .instance()
                .get(&DataKey::ConfirmedFeedbackCount)
                .unwrap_or(0);
            env.storage()
                .instance()
                .set(&DataKey::ConfirmedFeedbackCount, &(current.saturating_add(1)));
        } else {
            config.threshold_bps = (config.threshold_bps + LEARNING_RATE).min(10_000);
            let current: u64 = env
                .storage()
                .instance()
                .get(&DataKey::FalsePositiveFeedbackCount)
                .unwrap_or(0);
            env.storage()
                .instance()
                .set(&DataKey::FalsePositiveFeedbackCount, &(current.saturating_add(1)));
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

    #[test]
    fn test_assess_payment_claim_and_performance() {
        let env = Env::default();
        let contract_id = env.register_contract(None, AnomalyDetectionContract);
        let client = AnomalyDetectionContractClient::new(&env, &contract_id);

        let admin = Address::generate(&env);
        let detector = Address::generate(&env);
        let provider = Address::generate(&env);
        let patient = Address::generate(&env);

        client.mock_all_auths().initialize(&admin, &detector, &7500u32);
        client
            .mock_all_auths()
            .update_config(&admin, &None, &Some(1000u32), &None, &Some(true));

        let score = client
            .assess_payment_claim(
                &env.current_contract_address(),
                &1u64,
                &provider,
                &patient,
                &20_000i128,
                &String::from_str(&env, "SERVICE-ABC"),
            )
            .unwrap();

        assert!(score >= 6000);

        let anomaly = client.get_anomaly_record(&1u64).unwrap();
        assert_eq!(anomaly.record_id, 1u64);

        assert!(client.submit_feedback(&admin, &1u64, &true).unwrap());
        assert!(client.submit_feedback(&admin, &1u64, &false).unwrap());
        let (accuracy, false_rate) = client.get_alert_accuracy_rates();
        assert!(accuracy >= 5000); // at least 50%
        assert!(false_rate <= 5000);
    }

    #[test]
    fn test_network_fraud_score_increases_from_provider_graph() {
        let env = Env::default();
        let contract_id = env.register_contract(None, AnomalyDetectionContract);
        let client = AnomalyDetectionContractClient::new(&env, &contract_id);

        let admin = Address::generate(&env);
        let detector = Address::generate(&env);
        let provider1 = Address::generate(&env);
        let provider2 = Address::generate(&env);
        let patient = Address::generate(&env);

        client.mock_all_auths().initialize(&admin, &detector, &5000u32);

        let _ = client
            .assess_payment_claim(
                &env.current_contract_address(),
                &1u64,
                &provider1,
                &patient,
                &10_000i128,
                &String::from_str(&env, "SVC1"),
            )
            .unwrap();
        let _ = client
            .assess_payment_claim(
                &env.current_contract_address(),
                &2u64,
                &provider2,
                &patient,
                &15_000i128,
                &String::from_str(&env, "SVC2"),
            )
            .unwrap();

        let risk1 = client.detect_network_fraud(&provider1);
        let risk2 = client.detect_network_fraud(&provider2);

        assert!(risk1 > 0);
        assert!(risk2 > 0);
    }
}
