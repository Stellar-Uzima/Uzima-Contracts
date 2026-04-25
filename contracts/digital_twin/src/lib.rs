#![no_std]
#![allow(clippy::too_many_arguments)]
#![allow(clippy::arithmetic_side_effects)]

use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype, symbol_short, xdr::ToXdr,
    Address, Bytes, BytesN, Env, Map, String, Symbol, Vec, U256, I256,
};

use upgradeability::storage::{ADMIN as UPGRADE_ADMIN, VERSION};

// ==================== Core Digital Twin Types ====================

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[contracttype]
pub enum TwinStatus {
    Initializing,
    Active,
    Syncing,
    Simulation,
    Archived,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[contracttype]
pub enum DataSource {
    MedicalRecords,
    GenomicData,
    Wearables,
    EMR,
    LabResults,
    Imaging,
    PatientReported,
    External,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[contracttype]
pub enum DataType {
    VitalSigns,
    LabResults,
    Genomic,
    Imaging,
    Medications,
    Procedures,
    Symptoms,
    Activity,
    Sleep,
    Nutrition,
    Environmental,
    Social,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[contracttype]
pub enum ModelType {
    Predictive,
    Simulation,
    RiskAssessment,
    TreatmentResponse,
    DiseaseProgression,
    Wellness,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[contracttype]
pub enum SimulationType {
    Treatment,
    Lifestyle,
    Environmental,
    Medication,
    Surgical,
    Preventive,
}

// ==================== Data Structures ====================

#[derive(Clone)]
#[contracttype]
pub struct DigitalTwinProfile {
    pub twin_id: u64,
    pub patient_id: Address,
    pub created_at: u64,
    pub updated_at: u64,
    pub status: TwinStatus,
    pub accuracy_score: u32, // Basis points (10000 = 100%)
    pub completeness_score: u32,
    pub sync_frequency: u32, // Seconds between syncs
    pub last_sync: u64,
    pub data_sources: Vec<DataSource>,
    pub model_types: Vec<ModelType>,
    pub consent_version: u32,
    pub research_consent: bool,
}

#[derive(Clone)]
#[contracttype]
pub struct DataStream {
    pub stream_id: u64,
    pub twin_id: u64,
    pub source: DataSource,
    pub data_type: DataType,
    pub provider: Address,
    pub stream_ref: String,
    pub last_update: u64,
    pub update_frequency: u32,
    pub quality_score: u32,
    pub is_active: bool,
    pub encryption_key_id: Option<BytesN<32>>,
}

#[derive(Clone)]
#[contracttype]
pub struct DataPoint {
    pub timestamp: u64,
    pub value: String, // JSON-encoded value
    pub confidence: u32,
    pub source_id: u64,
    pub verification_hash: BytesN<32>,
    pub metadata: Map<String, String>,
}

#[derive(Clone)]
#[contracttype]
pub struct PredictiveModel {
    pub model_id: u64,
    pub twin_id: u64,
    pub model_type: ModelType,
    pub model_ref: String,
    pub version: u32,
    pub accuracy: u32,
    pub last_trained: u64,
    pub training_data_points: u32,
    pub validation_score: u32,
    pub is_active: bool,
}

#[derive(Clone)]
#[contracttype]
pub struct Prediction {
    pub prediction_id: u64,
    pub model_id: u64,
    pub timestamp: u64,
    pub prediction_type: String,
    pub confidence: u32,
    pub result: String, // JSON-encoded prediction result
    pub input_data_hash: BytesN<32>,
    pub explanation_ref: Option<String>,
    pub risk_level: u32,
}

#[derive(Clone)]
#[contracttype]
pub struct Simulation {
    pub simulation_id: u64,
    pub twin_id: u64,
    pub simulation_type: SimulationType,
    pub parameters: Map<String, String>, // JSON-encoded parameters
    pub start_time: u64,
    pub end_time: u64,
    pub results: Map<String, String>, // JSON-encoded results
    pub confidence: u32,
    pub created_by: Address,
    pub is_complete: bool,
}

#[derive(Clone)]
#[contracttype]
pub struct SyncStatus {
    pub twin_id: u64,
    pub source: DataSource,
    pub last_sync: u64,
    pub sync_success: bool,
    pub records_synced: u32,
    pub errors: Vec<String>,
    pub accuracy_delta: i32, // Change in accuracy since last sync
}

#[derive(Clone)]
#[contracttype]
pub struct ResearchSnapshot {
    pub snapshot_id: u64,
    pub twin_id: u64,
    pub researcher: Address,
    pub created_at: u64,
    pub expires_at: u64,
    pub data_types: Vec<DataType>,
    pub privacy_level: u32, // 0-100, higher = more privacy
    pub anonymization_method: String,
    pub snapshot_hash: BytesN<32>,
    pub access_count: u32,
}

// ==================== Storage Keys ====================

#[derive(Clone)]
#[contracttype]
pub enum DataKey {
    Admin,
    MedicalRecordsContract,
    GenomicDataContract,
    NextTwinId,
    Twin(u64),
    TwinByPatient(Address),
    TwinDataStreams(u64),
    DataStream(u64),
    StreamDataPoints(u64), // stream_id -> Vec<DataPoint>
    NextStreamId,
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
    GlobalStats,
}

// ==================== Errors ====================

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    NotAuthorized = 1,
    NotInitialized = 2,
    AlreadyInitialized = 3,
    TwinNotFound = 4,
    InvalidStatus = 5,
    DataStreamNotFound = 6,
    ModelNotFound = 7,
    SimulationNotFound = 8,
    InvalidParameter = 9,
    InsufficientAccuracy = 10,
    SyncInProgress = 11,
    ResearchAccessDenied = 12,
    SnapshotExpired = 13,
    DuplicateDataStream = 14,
    ModelNotActive = 15,
    SimulationInvalid = 16,
    PrivacyLevelInsufficient = 17,
    ConsentRequired = 18,
    ContractNotSet = 19,
}

// ==================== Contract Implementation ====================

#[contract]
pub struct DigitalTwinContract;

#[contractimpl]
impl DigitalTwinContract {
    // ==================== Initialization ====================

    pub fn initialize(env: Env, admin: Address) -> Result<bool, Error> {
        admin.require_auth();
        if env.storage().instance().has(&DataKey::Admin) {
            return Err(Error::AlreadyInitialized);
        }

        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage().instance().set(&VERSION, &1u32);
        env.storage().persistent().set(&DataKey::NextTwinId, &1u64);
        env.storage().persistent().set(&DataKey::NextStreamId, &1u64);
        env.storage().persistent().set(&DataKey::NextModelId, &1u64);
        env.storage().persistent().set(&DataKey::NextPredictionId, &1u64);
        env.storage().persistent().set(&DataKey::NextSimulationId, &1u64);
        env.storage().persistent().set(&DataKey::NextSnapshotId, &1u64);

        // Initialize global statistics
        let mut stats = Map::new(&env);
        stats.set(String::from_str(&env, "total_twins"), &0u64);
        stats.set(String::from_str(&env, "active_twins"), &0u64);
        stats.set(String::from_str(&env, "total_predictions"), &0u64);
        stats.set(String::from_str(&env, "total_simulations"), &0u64);
        env.storage().persistent().set(&DataKey::GlobalStats, &stats);

        env.events().publish((symbol_short!("DT_INIT"),), admin);
        Ok(true)
    }

    pub fn set_medical_records_contract(
        env: Env,
        admin: Address,
        contract_id: Address,
    ) -> Result<bool, Error> {
        admin.require_auth();
        Self::require_admin(&env, &admin)?;
        env.storage()
            .persistent()
            .set(&DataKey::MedicalRecordsContract, &contract_id);
        env.events().publish((symbol_short!("DT_MR_SET"),), contract_id);
        Ok(true)
    }

    pub fn set_genomic_data_contract(
        env: Env,
        admin: Address,
        contract_id: Address,
    ) -> Result<bool, Error> {
        admin.require_auth();
        Self::require_admin(&env, &admin)?;
        env.storage()
            .persistent()
            .set(&DataKey::GenomicDataContract, &contract_id);
        env.events().publish((symbol_short!("DT_GD_SET"),), contract_id);
        Ok(true)
    }

    // ==================== Digital Twin Management ====================

    pub fn create_digital_twin(
        env: Env,
        patient: Address,
        initial_data_sources: Vec<DataSource>,
        model_types: Vec<ModelType>,
        sync_frequency: u32,
    ) -> Result<u64, Error> {
        patient.require_auth();

        // Check if twin already exists
        if env
            .storage()
            .persistent()
            .get::<DataKey, DigitalTwinProfile>(&DataKey::TwinByPatient(patient.clone()))
            .is_some()
        {
            return Err(Error::InvalidParameter);
        }

        let twin_id = Self::next_twin_id(&env);
        let now = env.ledger().timestamp();

        let twin = DigitalTwinProfile {
            twin_id,
            patient_id: patient.clone(),
            created_at: now,
            updated_at: now,
            status: TwinStatus::Initializing,
            accuracy_score: 0, // Will be updated after initial sync
            completeness_score: 0,
            sync_frequency,
            last_sync: 0,
            data_sources: initial_data_sources.clone(),
            model_types,
            consent_version: 1,
            research_consent: false,
        };

        // Store twin
        env.storage()
            .persistent()
            .set(&DataKey::Twin(twin_id), &twin);
        env.storage()
            .persistent()
            .set(&DataKey::TwinByPatient(patient), &twin_id);

        // Initialize data streams
        for source in initial_data_sources {
            Self::create_data_stream_internal(&env, twin_id, source, patient.clone())?;
        }

        // Update global statistics
        Self::update_global_stats(&env, "total_twins", 1);

        env.events().publish((symbol_short!("DT_CREATED"),), (twin_id, patient));
        Ok(twin_id)
    }

    pub fn update_digital_twin_status(
        env: Env,
        admin: Address,
        twin_id: u64,
        new_status: TwinStatus,
    ) -> Result<bool, Error> {
        admin.require_auth();
        Self::require_admin_or_patient(&env, &admin, twin_id)?;

        let mut twin: DigitalTwinProfile = env
            .storage()
            .persistent()
            .get(&DataKey::Twin(twin_id))
            .ok_or(Error::TwinNotFound)?;

        twin.status = new_status;
        twin.updated_at = env.ledger().timestamp();

        env.storage().persistent().set(&DataKey::Twin(twin_id), &twin);

        // Update active twins count
        let mut stats: Map<String, u64> = env
            .storage()
            .persistent()
            .get(&DataKey::GlobalStats)
            .unwrap_or_else(|| Map::new(&env));
        
        let active_count = stats.get_unchecked(String::from_str(&env, "active_twins"));
        if new_status == TwinStatus::Active {
            stats.set(String::from_str(&env, "active_twins"), active_count + 1);
        } else if twin.status == TwinStatus::Active {
            stats.set(String::from_str(&env, "active_twins"), active_count.saturating_sub(1));
        }
        env.storage().persistent().set(&DataKey::GlobalStats, &stats);

        env.events().publish((symbol_short!("DT_STATUS"),), (twin_id, new_status));
        Ok(true)
    }

    // ==================== Data Stream Management ====================

    pub fn add_data_stream(
        env: Env,
        patient: Address,
        twin_id: u64,
        source: DataSource,
        data_type: DataType,
        provider: Address,
        stream_ref: String,
        update_frequency: u32,
    ) -> Result<u64, Error> {
        patient.require_auth();
        Self::require_twin_owner(&env, &patient, twin_id)?;

        // Check for duplicate data stream
        let streams: Vec<u64> = env
            .storage()
            .persistent()
            .get(&DataKey::TwinDataStreams(twin_id))
            .unwrap_or_else(|| Vec::new(&env));
        
        for stream_id in streams.iter() {
            if let Some(stream) = env
                .storage()
                .persistent()
                .get::<DataKey, DataStream>(&DataKey::DataStream(*stream_id))
            {
                if stream.source == source && stream.data_type == data_type {
                    return Err(Error::DuplicateDataStream);
                }
            }
        }

        let stream_id = Self::next_stream_id(&env);
        let now = env.ledger().timestamp();

        let data_stream = DataStream {
            stream_id,
            twin_id,
            source,
            data_type,
            provider,
            stream_ref,
            last_update: now,
            update_frequency,
            quality_score: 100, // Initial quality score
            is_active: true,
            encryption_key_id: None,
        };

        // Store data stream
        env.storage()
            .persistent()
            .set(&DataKey::DataStream(stream_id), &data_stream);
        
        // Add to twin's streams
        let mut streams = env
            .storage()
            .persistent()
            .get(&DataKey::TwinDataStreams(twin_id))
            .unwrap_or_else(|| Vec::new(&env));
        streams.push_back(stream_id);
        env.storage()
            .persistent()
            .set(&DataKey::TwinDataStreams(twin_id), &streams);

        env.events().publish((symbol_short!("DT_STREAM"),), (twin_id, stream_id));
        Ok(stream_id)
    }

    pub fn add_data_point(
        env: Env,
        provider: Address,
        stream_id: u64,
        value: String,
        confidence: u32,
        metadata: Map<String, String>,
    ) -> Result<bool, Error> {
        provider.require_auth();

        let mut stream: DataStream = env
            .storage()
            .persistent()
            .get(&DataKey::DataStream(stream_id))
            .ok_or(Error::DataStreamNotFound)?;

        if !stream.is_active {
            return Err(Error::InvalidParameter);
        }

        let now = env.ledger().timestamp();
        let verification_hash = Self::compute_data_hash(&env, &value, &metadata);

        let data_point = DataPoint {
            timestamp: now,
            value,
            confidence,
            source_id: stream_id,
            verification_hash,
            metadata,
        };

        // Store data point
        let mut data_points: Vec<DataPoint> = env
            .storage()
            .persistent()
            .get(&DataKey::StreamDataPoints(stream_id))
            .unwrap_or_else(|| Vec::new(&env));
        
        data_points.push_back(data_point);
        
        // Keep only last 1000 data points per stream to manage storage
        if data_points.len() > 1000 {
            data_points.remove(0);
        }
        
        env.storage()
            .persistent()
            .set(&DataKey::StreamDataPoints(stream_id), &data_points);

        // Update stream
        stream.last_update = now;
        env.storage()
            .persistent()
            .set(&DataKey::DataStream(stream_id), &stream);

        env.events().publish((symbol_short!("DT_DATAPOINT"),), stream_id);
        Ok(true)
    }

    // ==================== Predictive Modeling ====================

    pub fn add_predictive_model(
        env: Env,
        admin: Address,
        twin_id: u64,
        model_type: ModelType,
        model_ref: String,
    ) -> Result<u64, Error> {
        admin.require_auth();
        Self::require_admin_or_twin_owner(&env, &admin, twin_id)?;

        let model_id = Self::next_model_id(&env);
        let now = env.ledger().timestamp();

        let model = PredictiveModel {
            model_id,
            twin_id,
            model_type,
            model_ref,
            version: 1,
            accuracy: 0, // Will be updated after training
            last_trained: now,
            training_data_points: 0,
            validation_score: 0,
            is_active: true,
        };

        // Store model
        env.storage()
            .persistent()
            .set(&DataKey::PredictiveModel(model_id), &model);
        
        // Add to twin's models
        let mut models = env
            .storage()
            .persistent()
            .get(&DataKey::TwinModels(twin_id))
            .unwrap_or_else(|| Vec::new(&env));
        models.push_back(model_id);
        env.storage()
            .persistent()
            .set(&DataKey::TwinModels(twin_id), &models);

        env.events().publish((symbol_short!("DT_MODEL"),), (twin_id, model_id));
        Ok(model_id)
    }

    pub fn generate_prediction(
        env: Env,
        model_id: u64,
        input_data: String,
        prediction_type: String,
    ) -> Result<u64, Error> {
        let model: PredictiveModel = env
            .storage()
            .persistent()
            .get(&DataKey::PredictiveModel(model_id))
            .ok_or(Error::ModelNotFound)?;

        if !model.is_active {
            return Err(Error::ModelNotActive);
        }

        let prediction_id = Self::next_prediction_id(&env);
        let now = env.ledger().timestamp();
        let input_hash = Self::compute_data_hash(&env, &input_data, &Map::new(&env));

        // In a real implementation, this would call the ML model
        // For now, we'll create a placeholder prediction
        let prediction = Prediction {
            prediction_id,
            model_id,
            timestamp: now,
            prediction_type,
            confidence: 75, // Placeholder confidence
            result: String::from_str(&env, "{\"outcome\": \"positive\", \"probability\": 0.75}"),
            input_data_hash: input_hash,
            explanation_ref: None,
            risk_level: 50,
        };

        // Store prediction
        env.storage()
            .persistent()
            .set(&DataKey::Prediction(prediction_id), &prediction);
        
        // Add to model's predictions
        let mut predictions = env
            .storage()
            .persistent()
            .get(&DataKey::ModelPredictions(model_id))
            .unwrap_or_else(|| Vec::new(&env));
        predictions.push_back(prediction_id);
        env.storage()
            .persistent()
            .set(&DataKey::ModelPredictions(model_id), &predictions);

        // Update global statistics
        Self::update_global_stats(&env, "total_predictions", 1);

        env.events().publish((symbol_short!("DT_PREDICTION"),), (model_id, prediction_id));
        Ok(prediction_id)
    }

    // ==================== Simulation ====================

    pub fn create_simulation(
        env: Env,
        twin_id: u64,
        simulation_type: SimulationType,
        parameters: Map<String, String>,
        created_by: Address,
    ) -> Result<u64, Error> {
        created_by.require_auth();
        Self::require_twin_owner(&env, &created_by, twin_id)?;

        let simulation_id = Self::next_simulation_id(&env);
        let now = env.ledger().timestamp();

        let simulation = Simulation {
            simulation_id,
            twin_id,
            simulation_type,
            parameters,
            start_time: now,
            end_time: now + 3600, // 1 hour default duration
            results: Map::new(&env),
            confidence: 0, // Will be set when simulation completes
            created_by,
            is_complete: false,
        };

        // Store simulation
        env.storage()
            .persistent()
            .set(&DataKey::Simulation(simulation_id), &simulation);
        
        // Add to twin's simulations
        let mut simulations = env
            .storage()
            .persistent()
            .get(&DataKey::TwinSimulations(twin_id))
            .unwrap_or_else(|| Vec::new(&env));
        simulations.push_back(simulation_id);
        env.storage()
            .persistent()
            .set(&DataKey::TwinSimulations(twin_id), &simulations);

        // Update twin status
        let mut twin: DigitalTwinProfile = env
            .storage()
            .persistent()
            .get(&DataKey::Twin(twin_id))
            .ok_or(Error::TwinNotFound)?;
        twin.status = TwinStatus::Simulation;
        twin.updated_at = now;
        env.storage().persistent().set(&DataKey::Twin(twin_id), &twin);

        env.events().publish((symbol_short!("DT_SIM"),), (twin_id, simulation_id));
        Ok(simulation_id)
    }

    pub fn complete_simulation(
        env: Env,
        simulation_id: u64,
        results: Map<String, String>,
        confidence: u32,
    ) -> Result<bool, Error> {
        let mut simulation: Simulation = env
            .storage()
            .persistent()
            .get(&DataKey::Simulation(simulation_id))
            .ok_or(Error::SimulationNotFound)?;

        simulation.results = results;
        simulation.confidence = confidence;
        simulation.is_complete = true;
        simulation.end_time = env.ledger().timestamp();

        env.storage()
            .persistent()
            .set(&DataKey::Simulation(simulation_id), &simulation);

        // Update twin status back to Active
        let mut twin: DigitalTwinProfile = env
            .storage()
            .persistent()
            .get(&DataKey::Twin(simulation.twin_id))
            .ok_or(Error::TwinNotFound)?;
        twin.status = TwinStatus::Active;
        twin.updated_at = env.ledger().timestamp();
        env.storage()
            .persistent()
            .set(&DataKey::Twin(simulation.twin_id), &twin);

        // Update global statistics
        Self::update_global_stats(&env, "total_simulations", 1);

        env.events().publish((symbol_short!("DT_SIM_COMP"),), simulation_id);
        Ok(true)
    }

    // ==================== Research & Privacy ====================

    pub fn create_research_snapshot(
        env: Env,
        researcher: Address,
        twin_id: u64,
        data_types: Vec<DataType>,
        privacy_level: u32,
        anonymization_method: String,
        duration_hours: u32,
    ) -> Result<u64, Error> {
        researcher.require_auth();

        let twin: DigitalTwinProfile = env
            .storage()
            .persistent()
            .get(&DataKey::Twin(twin_id))
            .ok_or(Error::TwinNotFound)?;

        if !twin.research_consent {
            return Err(Error::ConsentRequired);
        }

        if privacy_level < 50 {
            return Err(Error::PrivacyLevelInsufficient);
        }

        let snapshot_id = Self::next_snapshot_id(&env);
        let now = env.ledger().timestamp();
        let expires_at = now + (duration_hours as u64 * 3600);

        // Compute snapshot hash
        let snapshot_data = format!("{}{}{}{}", twin_id, researcher, now, privacy_level);
        let snapshot_hash = env.crypto().sha256(&snapshot_data.into_bytes());

        let snapshot = ResearchSnapshot {
            snapshot_id,
            twin_id,
            researcher,
            created_at: now,
            expires_at,
            data_types,
            privacy_level,
            anonymization_method,
            snapshot_hash,
            access_count: 0,
        };

        // Store snapshot
        env.storage()
            .persistent()
            .set(&DataKey::ResearchSnapshot(snapshot_id), &snapshot);
        
        // Add to twin's snapshots
        let mut snapshots = env
            .storage()
            .persistent()
            .get(&DataKey::TwinSnapshots(twin_id))
            .unwrap_or_else(|| Vec::new(&env));
        snapshots.push_back(snapshot_id);
        env.storage()
            .persistent()
            .set(&DataKey::TwinSnapshots(twin_id), &snapshots);

        env.events().publish((symbol_short!("DT_SNAPSHOT"),), (twin_id, snapshot_id));
        Ok(snapshot_id)
    }

    // ==================== Synchronization ====================

    pub fn sync_with_medical_records(
        env: Env,
        twin_id: u64,
    ) -> Result<SyncStatus, Error> {
        let twin: DigitalTwinProfile = env
            .storage()
            .persistent()
            .get(&DataKey::Twin(twin_id))
            .ok_or(Error::TwinNotFound)?;

        if twin.status == TwinStatus::Syncing {
            return Err(Error::SyncInProgress);
        }

        let medical_records_contract: Address = env
            .storage()
            .persistent()
            .get(&DataKey::MedicalRecordsContract)
            .ok_or(Error::ContractNotSet)?;

        // Update twin status to syncing
        let mut updated_twin = twin.clone();
        updated_twin.status = TwinStatus::Syncing;
        updated_twin.updated_at = env.ledger().timestamp();
        env.storage()
            .persistent()
            .set(&DataKey::Twin(twin_id), &updated_twin);

        // In a real implementation, this would query the medical records contract
        // and sync the data. For now, we'll create a placeholder sync status.
        let sync_status = SyncStatus {
            twin_id,
            source: DataSource::MedicalRecords,
            last_sync: env.ledger().timestamp(),
            sync_success: true,
            records_synced: 10, // Placeholder
            errors: Vec::new(&env),
            accuracy_delta: 50, // Improved accuracy by 0.5%
        };

        // Store sync status
        env.storage()
            .persistent()
            .set(&DataKey::SyncStatus(twin_id, DataSource::MedicalRecords), &sync_status);

        // Update twin accuracy and completeness
        updated_twin.accuracy_score = (updated_twin.accuracy_score + sync_status.accuracy_delta).min(10000);
        updated_twin.completeness_score = (updated_twin.completeness_score + 100).min(10000);
        updated_twin.last_sync = sync_status.last_sync;
        updated_twin.status = TwinStatus::Active;
        env.storage()
            .persistent()
            .set(&DataKey::Twin(twin_id), &updated_twin);

        env.events().publish((symbol_short!("DT_SYNC"),), (twin_id, DataSource::MedicalRecords));
        Ok(sync_status)
    }

    // ==================== Query Functions ====================

    pub fn get_digital_twin(env: Env, twin_id: u64) -> Result<DigitalTwinProfile, Error> {
        env.storage()
            .persistent()
            .get(&DataKey::Twin(twin_id))
            .ok_or(Error::TwinNotFound)
    }

    pub fn get_twin_by_patient(env: Env, patient: Address) -> Result<u64, Error> {
        env.storage()
            .persistent()
            .get(&DataKey::TwinByPatient(patient))
            .ok_or(Error::TwinNotFound)
    }

    pub fn get_data_stream(env: Env, stream_id: u64) -> Result<DataStream, Error> {
        env.storage()
            .persistent()
            .get(&DataKey::DataStream(stream_id))
            .ok_or(Error::DataStreamNotFound)
    }

    pub fn get_data_points(
        env: Env,
        stream_id: u64,
        limit: u32,
    ) -> Result<Vec<DataPoint>, Error> {
        let data_points: Vec<DataPoint> = env
            .storage()
            .persistent()
            .get(&DataKey::StreamDataPoints(stream_id))
            .unwrap_or_else(|| Vec::new(&env));

        let start_idx = if data_points.len() > limit as usize {
            data_points.len() - limit as usize
        } else {
            0
        };

        let mut result = Vec::new(&env);
        for i in start_idx..data_points.len() {
            result.push_back(data_points.get(i).unwrap().clone());
        }

        Ok(result)
    }

    pub fn get_predictive_model(env: Env, model_id: u64) -> Result<PredictiveModel, Error> {
        env.storage()
            .persistent()
            .get(&DataKey::PredictiveModel(model_id))
            .ok_or(Error::ModelNotFound)
    }

    pub fn get_prediction(env: Env, prediction_id: u64) -> Result<Prediction, Error> {
        env.storage()
            .persistent()
            .get(&DataKey::Prediction(prediction_id))
            .ok_or(Error::ModelNotFound) // Reuse error for now
    }

    pub fn get_simulation(env: Env, simulation_id: u64) -> Result<Simulation, Error> {
        env.storage()
            .persistent()
            .get(&DataKey::Simulation(simulation_id))
            .ok_or(Error::SimulationNotFound)
    }

    pub fn get_research_snapshot(env: Env, snapshot_id: u64) -> Result<ResearchSnapshot, Error> {
        let snapshot: ResearchSnapshot = env
            .storage()
            .persistent()
            .get(&DataKey::ResearchSnapshot(snapshot_id))
            .ok_or(Error::SnapshotExpired)?;

        let now = env.ledger().timestamp();
        if now > snapshot.expires_at {
            return Err(Error::SnapshotExpired);
        }

        Ok(snapshot)
    }

    pub fn get_global_stats(env: Env) -> Result<Map<String, u64>, Error> {
        env.storage()
            .persistent()
            .get(&DataKey::GlobalStats)
            .ok_or(Error::NotInitialized)
    }

    // ==================== Helper Functions ====================

    fn require_admin(env: &Env, admin: &Address) -> Result<(), Error> {
        let stored_admin: Address = env
            .storage()
            .instance()
            .get(&DataKey::Admin)
            .ok_or(Error::NotInitialized)?;
        if stored_admin != *admin {
            return Err(Error::NotAuthorized);
        }
        Ok(())
    }

    fn require_admin_or_patient(env: &Env, caller: &Address, twin_id: u64) -> Result<(), Error> {
        let stored_admin: Address = env
            .storage()
            .instance()
            .get(&DataKey::Admin)
            .ok_or(Error::NotInitialized)?;
        
        if stored_admin == *caller {
            return Ok(());
        }

        let twin: DigitalTwinProfile = env
            .storage()
            .persistent()
            .get(&DataKey::Twin(twin_id))
            .ok_or(Error::TwinNotFound)?;
        
        if twin.patient_id == *caller {
            return Ok(());
        }

        Err(Error::NotAuthorized)
    }

    fn require_admin_or_twin_owner(env: &Env, caller: &Address, twin_id: u64) -> Result<(), Error> {
        Self::require_admin_or_patient(env, caller, twin_id)
    }

    fn require_twin_owner(env: &Env, caller: &Address, twin_id: u64) -> Result<(), Error> {
        let twin: DigitalTwinProfile = env
            .storage()
            .persistent()
            .get(&DataKey::Twin(twin_id))
            .ok_or(Error::TwinNotFound)?;
        
        if twin.patient_id != *caller {
            return Err(Error::NotAuthorized);
        }
        Ok(())
    }

    fn next_twin_id(env: &Env) -> u64 {
        let id: u64 = env
            .storage()
            .persistent()
            .get(&DataKey::NextTwinId)
            .unwrap_or(1);
        env.storage().persistent().set(&DataKey::NextTwinId, &(id + 1));
        id
    }

    fn next_stream_id(env: &Env) -> u64 {
        let id: u64 = env
            .storage()
            .persistent()
            .get(&DataKey::NextStreamId)
            .unwrap_or(1);
        env.storage().persistent().set(&DataKey::NextStreamId, &(id + 1));
        id
    }

    fn next_model_id(env: &Env) -> u64 {
        let id: u64 = env
            .storage()
            .persistent()
            .get(&DataKey::NextModelId)
            .unwrap_or(1);
        env.storage().persistent().set(&DataKey::NextModelId, &(id + 1));
        id
    }

    fn next_prediction_id(env: &Env) -> u64 {
        let id: u64 = env
            .storage()
            .persistent()
            .get(&DataKey::NextPredictionId)
            .unwrap_or(1);
        env.storage()
            .persistent()
            .set(&DataKey::NextPredictionId, &(id + 1));
        id
    }

    fn next_simulation_id(env: &Env) -> u64 {
        let id: u64 = env
            .storage()
            .persistent()
            .get(&DataKey::NextSimulationId)
            .unwrap_or(1);
        env.storage()
            .persistent()
            .set(&DataKey::NextSimulationId, &(id + 1));
        id
    }

    fn next_snapshot_id(env: &Env) -> u64 {
        let id: u64 = env
            .storage()
            .persistent()
            .get(&DataKey::NextSnapshotId)
            .unwrap_or(1);
        env.storage()
            .persistent()
            .set(&DataKey::NextSnapshotId, &(id + 1));
        id
    }

    fn compute_data_hash(env: &Env, value: &String, metadata: &Map<String, String>) -> BytesN<32> {
        let mut data = value.clone();
        for (key, val) in metadata.iter() {
            data = format!("{}{}:{}", data, key, val);
        }
        env.crypto().sha256(&data.into_bytes())
    }

    fn update_global_stats(env: &Env, stat_key: &str, increment: u64) {
        let mut stats: Map<String, u64> = env
            .storage()
            .persistent()
            .get(&DataKey::GlobalStats)
            .unwrap_or_else(|| Map::new(env));
        
        let current = stats.get_unchecked(String::from_str(env, stat_key));
        stats.set(String::from_str(env, stat_key), current + increment);
        env.storage().persistent().set(&DataKey::GlobalStats, &stats);
    }

    fn create_data_stream_internal(
        env: &Env,
        twin_id: u64,
        source: DataSource,
        provider: Address,
    ) -> Result<u64, Error> {
        let stream_id = Self::next_stream_id(env);
        let now = env.ledger().timestamp();

        let data_stream = DataStream {
            stream_id,
            twin_id,
            source,
            data_type: match source {
                DataSource::MedicalRecords => DataType::LabResults,
                DataSource::GenomicData => DataType::Genomic,
                DataSource::Wearables => DataType::VitalSigns,
                DataSource::EMR => DataType::Procedures,
                _ => DataType::VitalSigns,
            },
            provider,
            stream_ref: String::from_str(env, "auto_generated"),
            last_update: now,
            update_frequency: 300, // 5 minutes default
            quality_score: 100,
            is_active: true,
            encryption_key_id: None,
        };

        // Store data stream
        env.storage()
            .persistent()
            .set(&DataKey::DataStream(stream_id), &data_stream);
        
        // Add to twin's streams
        let mut streams = env
            .storage()
            .persistent()
            .get(&DataKey::TwinDataStreams(twin_id))
            .unwrap_or_else(|| Vec::new(env));
        streams.push_back(stream_id);
        env.storage()
            .persistent()
            .set(&DataKey::TwinDataStreams(twin_id), &streams);

        Ok(stream_id)
    }
}
