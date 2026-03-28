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
pub struct CnnModelInput {
    pub architecture_hash: BytesN<32>,
    pub version: u32,
    pub layer_count: u32,
    pub input_rows: u32,
    pub input_cols: u32,
    pub input_channels: u32,
    pub training_samples: u64,
    pub validation_accuracy_bps: u32,
    pub training_dataset_hash: BytesN<32>,
    pub signing_pubkey: BytesN<32>,
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
    // ── Public methods ──────────────────────────────────────────────────

    pub fn initialize(
        env: Env,
        admin: Address,
        default_warning_bps: u32,
        default_critical_bps: u32,
        default_min_samples: u64,
    ) -> Result<bool, Error> {
        admin.require_auth();
        if env.storage().instance().has(&ADMIN) {
            return Err(Error::AlreadyInitialized);
        }
        if default_warning_bps <= default_critical_bps || default_warning_bps > 10_000 {
            return Err(Error::InvalidThreshold);
        }
        if default_min_samples == 0 {
            return Err(Error::InvalidInput);
        }

        env.storage().instance().set(&ADMIN, &admin);
        env.storage().instance().set(&PAUSED, &false);
        env.storage().instance().set(&NEXT_RES, &1u64);
        env.storage().instance().set(&NEXT_SEG, &1u64);
        env.storage()
            .persistent()
            .set(&DataKey::DefaultWarningBps, &default_warning_bps);
        env.storage()
            .persistent()
            .set(&DataKey::DefaultCriticalBps, &default_critical_bps);
        env.storage()
            .persistent()
            .set(&DataKey::DefaultMinSamples, &default_min_samples);

        Ok(true)
    }

    pub fn pause(env: Env, admin: Address) -> Result<bool, Error> {
        admin.require_auth();
        Self::require_admin(&env, &admin)?;
        env.storage().instance().set(&PAUSED, &true);
        Ok(true)
    }

    pub fn unpause(env: Env, admin: Address) -> Result<bool, Error> {
        admin.require_auth();
        Self::require_admin(&env, &admin)?;
        env.storage().instance().set(&PAUSED, &false);
        Ok(true)
    }

    pub fn register_evaluator(
        env: Env,
        admin: Address,
        evaluator: Address,
    ) -> Result<bool, Error> {
        admin.require_auth();
        Self::require_admin(&env, &admin)?;
        env.storage()
            .persistent()
            .set(&DataKey::Evaluator(evaluator), &true);
        Ok(true)
    }

    pub fn revoke_evaluator(
        env: Env,
        admin: Address,
        evaluator: Address,
    ) -> Result<bool, Error> {
        admin.require_auth();
        Self::require_admin(&env, &admin)?;
        env.storage()
            .persistent()
            .set(&DataKey::Evaluator(evaluator), &false);
        Ok(true)
    }

    pub fn register_cnn_model(
        env: Env,
        caller: Address,
        model_id: BytesN<32>,
        modality: ImagingModality,
        input: CnnModelInput,
    ) -> Result<bool, Error> {
        caller.require_auth();
        Self::require_admin(&env, &caller)?;
        Self::require_not_paused(&env)?;

        if input.version == 0
            || input.layer_count == 0
            || input.input_rows == 0
            || input.input_cols == 0
            || input.input_channels == 0
        {
            return Err(Error::InvalidInput);
        }
        if input.validation_accuracy_bps > 10_000 {
            return Err(Error::InvalidConfidence);
        }
        if env
            .storage()
            .persistent()
            .has(&DataKey::CnnModel(model_id.clone()))
        {
            return Err(Error::ModelAlreadyExists);
        }

        let model = CnnModelMetadata {
            model_id: model_id.clone(),
            owner: caller.clone(),
            version: input.version,
            modality,
            architecture_hash: input.architecture_hash,
            layer_count: input.layer_count,
            input_rows: input.input_rows,
            input_cols: input.input_cols,
            input_channels: input.input_channels,
            training_samples: input.training_samples,
            validation_accuracy_bps: input.validation_accuracy_bps,
            training_dataset_hash: input.training_dataset_hash,
            signing_pubkey: input.signing_pubkey,
            status: ModelStatus::Active,
            registered_at: env.ledger().timestamp(),
            last_evaluated_at: 0,
        };

        env.storage()
            .persistent()
            .set(&DataKey::CnnModel(model_id.clone()), &model);

        env.events()
            .publish((symbol_short!("MDL_REG"),), model_id);

        Ok(true)
    }

    pub fn update_model_status(
        env: Env,
        admin: Address,
        model_id: BytesN<32>,
        new_status: ModelStatus,
    ) -> Result<bool, Error> {
        admin.require_auth();
        Self::require_admin(&env, &admin)?;
        Self::require_not_paused(&env)?;

        let mut model: CnnModelMetadata = env
            .storage()
            .persistent()
            .get(&DataKey::CnnModel(model_id.clone()))
            .ok_or(Error::ModelNotFound)?;

        model.status = new_status;
        env.storage()
            .persistent()
            .set(&DataKey::CnnModel(model_id.clone()), &model);

        match new_status {
            ModelStatus::Active => {
                env.events()
                    .publish((symbol_short!("MDL_REACT"),), model_id);
            }
            ModelStatus::Retired => {
                env.events()
                    .publish((symbol_short!("MDL_RET"),), model_id);
            }
            _ => {}
        }

        Ok(true)
    }

    pub fn get_model(env: Env, model_id: BytesN<32>) -> CnnModelMetadata {
        env.storage()
            .persistent()
            .get(&DataKey::CnnModel(model_id))
            .unwrap()
    }

    pub fn is_model_active(env: Env, model_id: BytesN<32>) -> bool {
        let model: CnnModelMetadata = env
            .storage()
            .persistent()
            .get(&DataKey::CnnModel(model_id))
            .unwrap();
        matches!(model.status, ModelStatus::Active | ModelStatus::Degraded)
    }

    // ── Private helpers ─────────────────────────────────────────────────

    fn require_admin(env: &Env, caller: &Address) -> Result<(), Error> {
        let admin: Address = env
            .storage()
            .instance()
            .get(&ADMIN)
            .ok_or(Error::NotInitialized)?;
        if &admin != caller {
            return Err(Error::NotAuthorized);
        }
        Ok(())
    }

    fn require_not_paused(env: &Env) -> Result<(), Error> {
        let paused: bool = env.storage().instance().get(&PAUSED).unwrap_or(false);
        if paused {
            return Err(Error::ContractPaused);
        }
        Ok(())
    }

    #[allow(dead_code)]
    fn require_evaluator(env: &Env, caller: &Address) -> Result<(), Error> {
        let is_eval: bool = env
            .storage()
            .persistent()
            .get(&DataKey::Evaluator(caller.clone()))
            .unwrap_or(false);
        if !is_eval {
            return Err(Error::NotAuthorized);
        }
        Ok(())
    }

    #[allow(dead_code)]
    fn next_counter(env: &Env, key: &Symbol) -> u64 {
        let current: u64 = env.storage().instance().get(key).unwrap_or(1u64);
        env.storage()
            .instance()
            .set(key, &current.saturating_add(1));
        current
    }

    #[allow(dead_code)]
    fn append_u64(env: &Env, key: DataKey, value: u64) {
        let mut values: Vec<u64> = env
            .storage()
            .persistent()
            .get(&key)
            .unwrap_or(Vec::new(env));
        if !values.iter().any(|item| item == value) {
            values.push_back(value);
            env.storage().persistent().set(&key, &values);
        }
    }
}
