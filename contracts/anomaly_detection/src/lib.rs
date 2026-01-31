// Anomaly Detection Contract - Healthcare anomaly detection with proper validation
#![no_std]
#![allow(clippy::too_many_arguments)]
#![allow(clippy::arithmetic_side_effects)]
#![allow(clippy::panic)]
#![allow(dead_code)]

use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype, symbol_short, Address, Env, String, Symbol,
};

#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub struct AnomalyDetectionConfig {
    pub admin: Address,
    pub detector: Address,
    pub threshold_bps: u32, // Threshold in basis points (0-10000)
    pub sensitivity: u32,   // Sensitivity level (1-10)
    pub enabled: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub struct AnomalyRecord {
    pub record_id: u64,
    pub patient: Address,
    pub detector_address: Address,
    pub score_bps: u32,
    pub severity: u32,
    pub detected_at: u64,
    pub metadata: String,
    pub explanation_ref: String,
    pub status: AnomalyStatus,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[contracttype]
pub enum AnomalyStatus {
    Pending,
    Confirmed,
    FalsePositive,
    Ignored,
}

#[derive(Clone, Debug, Eq, PartialEq)]
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
}

const ANOMALY_COUNTER: Symbol = symbol_short!("ANOM_CT");

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    NotAuthorized = 1,
    ContractPaused = 2,
    InvalidAnomaly = 3,
    InvalidThreshold = 4,
    DetectorNotWhitelisted = 5,
    ConfigNotSet = 6,
    Disabled = 7,
    InvalidScore = 8,
    InvalidSeverity = 9,
}

#[contract]
pub struct AnomalyDetectionContract;

#[contractimpl]
impl AnomalyDetectionContract {
    pub fn initialize(
        env: Env,
        admin: Address,
        primary_detector: Address,
        threshold: u32,
    ) -> Result<bool, Error> {
        admin.require_auth();

        if env.storage().instance().has(&DataKey::Config) {
            panic!("Already initialized");
        }

        if threshold > 10_000 {
            return Err(Error::InvalidThreshold);
        }

        let config = AnomalyDetectionConfig {
            admin,
            detector: primary_detector,
            threshold_bps: threshold,
            sensitivity: 5,
            enabled: true,
        };

        env.storage().instance().set(&DataKey::Config, &config);
        env.storage().instance().set(&ANOMALY_COUNTER, &0u64);
        Ok(true)
    }

    pub fn update_config(
        env: Env,
        admin: Address,
        new_detector: Option<Address>,
        new_threshold: Option<u32>,
        new_sensitivity: Option<u32>,
        enabled: Option<bool>,
    ) -> Result<bool, Error> {
        admin.require_auth();
        let mut config: AnomalyDetectionConfig = env
            .storage()
            .instance()
            .get(&DataKey::Config)
            .ok_or(Error::ConfigNotSet)?;

        if admin != config.admin {
            return Err(Error::NotAuthorized);
        }

        if let Some(det) = new_detector {
            config.detector = det.clone();
            env.storage()
                .instance()
                .set(&DataKey::Whitelist(det), &true);
        }
        if let Some(thresh) = new_threshold {
            config.threshold_bps = thresh;
        }
        if let Some(sense) = new_sensitivity {
            config.sensitivity = sense;
        }
        if let Some(en) = enabled {
            config.enabled = en;
        }

        env.storage().instance().set(&DataKey::Config, &config);
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
        let config: AnomalyDetectionConfig = env
            .storage()
            .instance()
            .get(&DataKey::Config)
            .ok_or(Error::ConfigNotSet)?;

        if !config.enabled {
            return Err(Error::Disabled);
        }
        if score_bps > 10_000 {
            return Err(Error::InvalidScore);
        }
        if severity == 0 || severity > 5 {
            return Err(Error::InvalidSeverity);
        }

        let anomaly_id = env
            .storage()
            .instance()
            .get(&ANOMALY_COUNTER)
            .unwrap_or(0u64)
            + 1;
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
            status: AnomalyStatus::Pending,
        };

        env.storage()
            .instance()
            .set(&DataKey::AnomalyRecord(anomaly_id), &anomaly_record);
        env.storage().instance().set(&ANOMALY_COUNTER, &anomaly_id);

        let patient_count: u64 = env
            .storage()
            .instance()
            .get(&DataKey::AnomalyCountByPatient(patient.clone()))
            .unwrap_or(0);
        env.storage().instance().set(
            &DataKey::AnomalyCountByPatient(patient),
            &(patient_count + 1),
        );

        Ok(anomaly_id)
    }

    pub fn update_status(
        env: Env,
        admin: Address,
        id: u64,
        status: AnomalyStatus,
    ) -> Result<bool, Error> {
        admin.require_auth();
        let mut record: AnomalyRecord = env
            .storage()
            .instance()
            .get(&DataKey::AnomalyRecord(id))
            .ok_or(Error::InvalidAnomaly)?;
        record.status = status;
        env.storage()
            .instance()
            .set(&DataKey::AnomalyRecord(id), &record);
        Ok(true)
    }

    pub fn get_anomaly_record(env: Env, id: u64) -> Option<AnomalyRecord> {
        env.storage().instance().get(&DataKey::AnomalyRecord(id))
    }

    pub fn get_config(env: Env) -> Option<AnomalyDetectionConfig> {
        env.storage().instance().get(&DataKey::Config)
    }

    pub fn whitelist_detector(
        env: Env,
        caller: Address,
        detector_addr: Address,
    ) -> Result<bool, Error> {
        caller.require_auth();
        env.storage()
            .instance()
            .set(&DataKey::Whitelist(detector_addr), &true);
        Ok(true)
    }

    pub fn is_whitelisted_detector(env: Env, addr: Address) -> bool {
        env.storage()
            .instance()
            .get(&DataKey::Whitelist(addr))
            .unwrap_or(false)
    }
}
