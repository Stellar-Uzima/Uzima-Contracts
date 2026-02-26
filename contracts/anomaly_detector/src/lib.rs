#![no_std]
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
        let n = features.len().min(weights.len()) as u32;
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
