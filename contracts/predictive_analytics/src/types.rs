use soroban_sdk::{contracterror, contracttype, Address, BytesN, String, Vec};

#[derive(Clone)]
#[contracttype]
pub struct PredictionConfig {
    pub admin: Address,
    pub predictor: Address,
    pub prediction_horizon_days: u32,
    pub enabled: bool,
    pub min_confidence_bps: u32,
}

#[derive(Clone)]
#[contracttype]
pub struct HealthPrediction {
    pub patient: Address,
    pub model_id: BytesN<32>,
    pub outcome_type: String,
    pub predicted_value: u32,
    pub confidence_bps: u32,
    pub prediction_date: u64,
    pub horizon_start: u64,
    pub horizon_end: u64,
    pub features_used: Vec<String>,
    pub explanation_ref: String,
    pub risk_factors: Vec<String>,
}

#[derive(Clone)]
#[contracttype]
pub struct PredictionMetrics {
    pub accuracy_bps: u32,
    pub precision_bps: u32,
    pub recall_bps: u32,
    pub f1_score_bps: u32,
    pub last_updated: u64,
}

#[derive(Clone)]
#[contracttype]
pub struct PatientPredictionsSummary {
    pub latest_prediction_id: u64,
    pub high_risk_predictions: u32,
    pub total_predictions: u32,
    pub avg_confidence_bps: u32,
    pub last_prediction_date: u64,
}

#[derive(Clone)]
#[contracttype]
pub enum DataKey {
    Config,
    Prediction(u64),
    PatientSummary(Address),
    ModelMetrics(BytesN<32>),
    PredictionCounter,
    Whitelist(Address),
}

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    NotAuthorized = 1,
    ConfigNotSet = 2,
    Disabled = 3,
    InvalidValue = 4,
    InvalidConfidence = 5,
    RecordNotFound = 6,
    LowConfidence = 7,
    InvalidHorizon = 8,
    EmptyInput = 9,
}
