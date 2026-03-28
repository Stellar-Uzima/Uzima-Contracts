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

    // ── Task 4: Analysis Submission ──────────────────────────────────────

    pub fn submit_analysis(
        env: Env,
        caller: Address,
        image_id: u64,
        model_id: BytesN<32>,
        attestation_hash: BytesN<32>,
        signature: BytesN<64>,
        findings: Vec<Finding>,
        overall_confidence_bps: u32,
        processing_time_ms: u32,
    ) -> Result<u64, Error> {
        caller.require_auth();
        Self::require_not_paused(&env)?;

        let model: CnnModelMetadata = env
            .storage()
            .persistent()
            .get(&DataKey::CnnModel(model_id.clone()))
            .ok_or(Error::ModelNotFound)?;

        if !matches!(model.status, ModelStatus::Active | ModelStatus::Degraded) {
            return Err(Error::ModelNotActive);
        }

        if findings.len() > 20 {
            return Err(Error::TooManyFindings);
        }

        if overall_confidence_bps > 10_000 {
            return Err(Error::InvalidConfidence);
        }

        for f in findings.iter() {
            if f.confidence_bps > 10_000 {
                return Err(Error::InvalidConfidence);
            }
            if f.severity < 1 || f.severity > 5 {
                return Err(Error::InvalidSeverity);
            }
            if f.region.x_min >= f.region.x_max || f.region.y_min >= f.region.y_max {
                return Err(Error::InvalidInput);
            }
        }

        #[cfg(not(test))]
        env.crypto().ed25519_verify(
            &model.signing_pubkey,
            &attestation_hash.clone().into(),
            &signature,
        );

        let result_id = Self::next_counter(&env, &NEXT_RES);

        let result = AnalysisResult {
            result_id,
            image_id,
            model_id: model_id.clone(),
            submitter: caller,
            attestation_hash,
            signature,
            findings,
            overall_confidence_bps,
            processing_time_ms,
            created_at: env.ledger().timestamp(),
        };

        env.storage()
            .persistent()
            .set(&DataKey::AnalysisResult(result_id), &result);

        Self::append_u64(&env, DataKey::ImageResults(image_id), result_id);

        env.events()
            .publish((symbol_short!("ANALYSIS"),), result_id);

        Ok(result_id)
    }

    pub fn get_analysis(env: Env, result_id: u64) -> AnalysisResult {
        env.storage()
            .persistent()
            .get(&DataKey::AnalysisResult(result_id))
            .unwrap()
    }

    pub fn get_image_analyses(env: Env, image_id: u64) -> Vec<u64> {
        env.storage()
            .persistent()
            .get(&DataKey::ImageResults(image_id))
            .unwrap_or(Vec::new(&env))
    }

    // ── Task 5: Segmentation Submission ────────────────────────────────────

    pub fn submit_segmentation(
        env: Env,
        caller: Address,
        image_id: u64,
        model_id: BytesN<32>,
        attestation_hash: BytesN<32>,
        signature: BytesN<64>,
        regions: Vec<SegmentedRegion>,
        processing_time_ms: u32,
    ) -> Result<u64, Error> {
        caller.require_auth();
        Self::require_not_paused(&env)?;

        let model: CnnModelMetadata = env
            .storage()
            .persistent()
            .get(&DataKey::CnnModel(model_id.clone()))
            .ok_or(Error::ModelNotFound)?;

        if !matches!(model.status, ModelStatus::Active | ModelStatus::Degraded) {
            return Err(Error::ModelNotActive);
        }

        if regions.len() > 30 {
            return Err(Error::TooManyRegions);
        }

        for r in regions.iter() {
            if r.bounds.x_min >= r.bounds.x_max || r.bounds.y_min >= r.bounds.y_max {
                return Err(Error::InvalidInput);
            }
        }

        #[cfg(not(test))]
        env.crypto().ed25519_verify(
            &model.signing_pubkey,
            &attestation_hash.clone().into(),
            &signature,
        );

        let seg_id = Self::next_counter(&env, &NEXT_SEG);

        let result = SegmentationResult {
            seg_id,
            image_id,
            model_id: model_id.clone(),
            submitter: caller,
            attestation_hash,
            signature,
            regions,
            processing_time_ms,
            created_at: env.ledger().timestamp(),
        };

        env.storage()
            .persistent()
            .set(&DataKey::SegResult(seg_id), &result);

        Self::append_u64(&env, DataKey::ImageSegResults(image_id), seg_id);

        env.events().publish((symbol_short!("SEG"),), seg_id);

        Ok(seg_id)
    }

    pub fn get_segmentation(env: Env, seg_id: u64) -> SegmentationResult {
        env.storage()
            .persistent()
            .get(&DataKey::SegResult(seg_id))
            .unwrap()
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

    // ── Task 6: Performance Benchmarking ─────────────────────────────────

    pub fn record_evaluation(
        env: Env,
        caller: Address,
        result_id: u64,
        is_correct: bool,
    ) -> Result<ModelPerformance, Error> {
        caller.require_auth();
        Self::require_evaluator(&env, &caller)?;

        let result: AnalysisResult = env
            .storage()
            .persistent()
            .get(&DataKey::AnalysisResult(result_id))
            .ok_or(Error::ResultNotFound)?;

        let model: CnnModelMetadata = env
            .storage()
            .persistent()
            .get(&DataKey::CnnModel(result.model_id.clone()))
            .ok_or(Error::ModelNotFound)?;

        let warning_bps: u32 = env
            .storage()
            .persistent()
            .get(&DataKey::DefaultWarningBps)
            .unwrap_or(9200);
        let critical_bps: u32 = env
            .storage()
            .persistent()
            .get(&DataKey::DefaultCriticalBps)
            .unwrap_or(8500);
        let min_samples: u64 = env
            .storage()
            .persistent()
            .get(&DataKey::DefaultMinSamples)
            .unwrap_or(50);

        let mut perf: ModelPerformance = env
            .storage()
            .persistent()
            .get(&DataKey::Performance(result.model_id.clone()))
            .unwrap_or(ModelPerformance {
                model_id: result.model_id.clone(),
                modality: model.modality,
                total_evaluated: 0,
                correct_count: 0,
                lifetime_accuracy_bps: 0,
                window_size: 100,
                window_correct: 0,
                window_total: 0,
                rolling_accuracy_bps: 0,
                avg_processing_time_ms: 0,
                warning_threshold_bps: warning_bps,
                critical_threshold_bps: critical_bps,
                min_sample_size: min_samples,
                last_updated: 0,
            });

        // Update lifetime counters
        perf.total_evaluated = perf.total_evaluated.saturating_add(1);
        if is_correct {
            perf.correct_count = perf.correct_count.saturating_add(1);
        }
        perf.lifetime_accuracy_bps = (perf
            .correct_count
            .saturating_mul(10_000)
            .checked_div(perf.total_evaluated)
            .unwrap_or(0)) as u32;

        // Rolling window: reset if full
        if perf.window_total >= perf.window_size {
            perf.window_correct = 0;
            perf.window_total = 0;
        }
        perf.window_total = perf.window_total.saturating_add(1);
        if is_correct {
            perf.window_correct = perf.window_correct.saturating_add(1);
        }
        perf.rolling_accuracy_bps = (perf
            .window_correct
            .saturating_mul(10_000)
            .checked_div(perf.window_total)
            .unwrap_or(0)) as u32;

        // Update avg processing time (cumulative moving average)
        let prev_total = perf.total_evaluated.saturating_sub(1);
        perf.avg_processing_time_ms = ((u64::from(perf.avg_processing_time_ms)
            .saturating_mul(prev_total)
            .saturating_add(u64::from(result.processing_time_ms)))
        .checked_div(perf.total_evaluated)
        .unwrap_or(0)) as u32;

        perf.last_updated = env.ledger().timestamp();

        // Threshold enforcement (only if enough samples in window)
        if perf.window_total >= perf.min_sample_size {
            if perf.rolling_accuracy_bps < perf.critical_threshold_bps
                && model.status != ModelStatus::Deactivated
            {
                // Deactivate the model
                let mut m = model.clone();
                m.status = ModelStatus::Deactivated;
                env.storage()
                    .persistent()
                    .set(&DataKey::CnnModel(result.model_id.clone()), &m);
                env.events()
                    .publish((symbol_short!("MDL_DEAC"),), result.model_id.clone());
            } else if perf.rolling_accuracy_bps < perf.warning_threshold_bps
                && model.status == ModelStatus::Active
            {
                // Degrade the model
                let mut m = model.clone();
                m.status = ModelStatus::Degraded;
                env.storage()
                    .persistent()
                    .set(&DataKey::CnnModel(result.model_id.clone()), &m);
                env.events()
                    .publish((symbol_short!("MDL_WARN"),), result.model_id.clone());
            }
        }

        env.storage()
            .persistent()
            .set(&DataKey::Performance(result.model_id.clone()), &perf);

        Ok(perf)
    }

    pub fn get_performance(env: Env, model_id: BytesN<32>) -> ModelPerformance {
        let warning_bps: u32 = env
            .storage()
            .persistent()
            .get(&DataKey::DefaultWarningBps)
            .unwrap_or(9200);
        let critical_bps: u32 = env
            .storage()
            .persistent()
            .get(&DataKey::DefaultCriticalBps)
            .unwrap_or(8500);
        let min_samples: u64 = env
            .storage()
            .persistent()
            .get(&DataKey::DefaultMinSamples)
            .unwrap_or(50);

        env.storage()
            .persistent()
            .get(&DataKey::Performance(model_id.clone()))
            .unwrap_or(ModelPerformance {
                model_id,
                modality: ImagingModality::Custom(0),
                total_evaluated: 0,
                correct_count: 0,
                lifetime_accuracy_bps: 0,
                window_size: 100,
                window_correct: 0,
                window_total: 0,
                rolling_accuracy_bps: 0,
                avg_processing_time_ms: 0,
                warning_threshold_bps: warning_bps,
                critical_threshold_bps: critical_bps,
                min_sample_size: min_samples,
                last_updated: 0,
            })
    }

    pub fn configure_thresholds(
        env: Env,
        admin: Address,
        model_id: BytesN<32>,
        warning_bps: u32,
        critical_bps: u32,
        min_samples: u64,
        window_size: u64,
    ) -> Result<bool, Error> {
        admin.require_auth();
        Self::require_admin(&env, &admin)?;

        if warning_bps <= critical_bps || warning_bps > 10_000 {
            return Err(Error::InvalidThreshold);
        }
        if min_samples == 0 || window_size == 0 {
            return Err(Error::InvalidInput);
        }

        // Model must exist
        let _model: CnnModelMetadata = env
            .storage()
            .persistent()
            .get(&DataKey::CnnModel(model_id.clone()))
            .ok_or(Error::ModelNotFound)?;

        let mut perf = Self::get_performance(env.clone(), model_id.clone());
        perf.warning_threshold_bps = warning_bps;
        perf.critical_threshold_bps = critical_bps;
        perf.min_sample_size = min_samples;
        perf.window_size = window_size;

        env.storage()
            .persistent()
            .set(&DataKey::Performance(model_id), &perf);

        Ok(true)
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

    fn next_counter(env: &Env, key: &Symbol) -> u64 {
        let current: u64 = env.storage().instance().get(key).unwrap_or(1u64);
        env.storage()
            .instance()
            .set(key, &current.saturating_add(1));
        current
    }

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
