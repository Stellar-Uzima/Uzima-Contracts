#![no_std]
#![allow(clippy::too_many_arguments)]

#[cfg(test)]
mod test;

use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype, symbol_short, Address, BytesN, Env,
    String, Symbol, Vec,
};

const ADMIN: Symbol = symbol_short!("ADMIN");
const PAUSED: Symbol = symbol_short!("PAUSED");
const NEXT_RES: Symbol = symbol_short!("NRES");
const NEXT_SEG: Symbol = symbol_short!("NSEG");

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[contracttype]
pub enum ImagingModality {
    XRay,
    MRI,
    CT,
    Ultrasound,
    PET,
    Mammography,
    Custom(u32),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[contracttype]
pub enum ModelStatus {
    Active,
    Degraded,
    Deactivated,
    Retired,
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub struct BoundingBox {
    pub x_min: u32,
    pub y_min: u32,
    pub x_max: u32,
    pub y_max: u32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub struct CnnModelMetadata {
    pub model_id: BytesN<32>,
    pub owner: Address,
    pub version: u32,
    pub modality: ImagingModality,
    pub architecture_hash: BytesN<32>,
    pub layer_count: u32,
    pub input_rows: u32,
    pub input_cols: u32,
    pub input_channels: u32,
    pub training_samples: u64,
    pub validation_accuracy_bps: u32,
    pub training_dataset_hash: BytesN<32>,
    pub signing_pubkey: BytesN<32>,
    pub status: ModelStatus,
    pub registered_at: u64,
    pub last_evaluated_at: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub struct Finding {
    pub finding_id: u32,
    pub condition_hash: BytesN<32>,
    pub confidence_bps: u32,
    pub severity: u32,
    pub region: BoundingBox,
    pub explanation_ref: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub struct AnalysisResult {
    pub result_id: u64,
    pub image_id: u64,
    pub model_id: BytesN<32>,
    pub submitter: Address,
    pub attestation_hash: BytesN<32>,
    pub signature: BytesN<64>,
    pub findings: Vec<Finding>,
    pub overall_confidence_bps: u32,
    pub processing_time_ms: u32,
    pub created_at: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub struct SegmentedRegion {
    pub label_hash: BytesN<32>,
    pub pixel_count: u64,
    pub volume_mm3: u64,
    pub mean_intensity: u32,
    pub mask_ref: String,
    pub bounds: BoundingBox,
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub struct SegmentationResult {
    pub seg_id: u64,
    pub image_id: u64,
    pub model_id: BytesN<32>,
    pub submitter: Address,
    pub attestation_hash: BytesN<32>,
    pub signature: BytesN<64>,
    pub regions: Vec<SegmentedRegion>,
    pub processing_time_ms: u32,
    pub created_at: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub struct ModelPerformance {
    pub model_id: BytesN<32>,
    pub modality: ImagingModality,
    pub total_evaluated: u64,
    pub correct_count: u64,
    pub lifetime_accuracy_bps: u32,
    pub window_size: u64,
    pub window_correct: u64,
    pub window_total: u64,
    pub rolling_accuracy_bps: u32,
    pub avg_processing_time_ms: u32,
    pub warning_threshold_bps: u32,
    pub critical_threshold_bps: u32,
    pub min_sample_size: u64,
    pub last_updated: u64,
}

#[derive(Clone)]
#[contracttype]
pub enum DataKey {
    CnnModel(BytesN<32>),
    AnalysisResult(u64),
    SegResult(u64),
    Performance(BytesN<32>),
    ImageResults(u64),
    ImageSegResults(u64),
    Evaluator(Address),
    DefaultWarningBps,
    DefaultCriticalBps,
    DefaultMinSamples,
}

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    AlreadyInitialized = 1,
    NotInitialized = 2,
    NotAuthorized = 3,
    ContractPaused = 4,
    InvalidInput = 5,
    ModelNotFound = 6,
    ModelNotActive = 7,
    ModelAlreadyExists = 8,
    ResultNotFound = 9,
    SegmentationNotFound = 10,
    TooManyFindings = 11,
    TooManyRegions = 12,
    InvalidConfidence = 13,
    InvalidSeverity = 14,
    InvalidThreshold = 15,
    AttestationInvalid = 16,
    DuplicateResult = 17,
    InsufficientSamples = 18,
}

#[contract]
pub struct MedicalImagingAiContract;

#[contractimpl]
impl MedicalImagingAiContract {
    // Implementation will be added in subsequent tasks
}
