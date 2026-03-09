// Public Health Surveillance Platform - Privacy-Preserving Disease Monitoring and Response
#![no_std]
#![allow(clippy::too_many_arguments)]
#![allow(clippy::arithmetic_side_effects)]

#[cfg(test)]
mod test;

use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype, symbol_short, Address, Bytes, BytesN, Env,
    String, Symbol, Vec,
};

// =============================================================================
// Types
// =============================================================================

/// Disease severity levels for public health response
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[contracttype]
pub enum DiseaseSeverity {
    Low = 1,
    Medium = 2,
    High = 3,
    Critical = 4,
}

/// Public health alert types
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[contracttype]
pub enum AlertType {
    DiseaseOutbreak,
    EnvironmentalHazard,
    VaccineShortage,
    AntimicrobialResistance,
    SupplyChainDisruption,
    EmergingPathogen,
    SeasonalEpidemic,
}

/// Outbreak detection status
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[contracttype]
pub enum OutbreakStatus {
    Monitoring,
    Detected,
    Investigating,
    Confirmed,
    Contained,
    Resolved,
}

/// Privacy-preserving data aggregation method
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[contracttype]
pub enum AggregationMethod {
    DifferentialPrivacy,
    SecureMultipartyComputation,
    HomomorphicEncryption,
    ZeroKnowledgeProofs,
    FederatedLearning,
}

/// Disease outbreak data point
#[derive(Clone)]
#[contracttype]
pub struct OutbreakData {
    /// Unique identifier for this data point
    pub data_id: BytesN<32>,
    /// Geographic region (encrypted for privacy)
    pub encrypted_region: Bytes,
    /// Disease identifier (ICD-10 code)
    pub disease_code: String,
    /// Number of cases (aggregated with privacy)
    pub aggregated_cases: u64,
    /// Time period for this data
    pub time_period_start: u64,
    pub time_period_end: u64,
    /// Data aggregation method used
    pub aggregation_method: AggregationMethod,
    /// Privacy budget used (epsilon)
    pub privacy_epsilon: u64,
    /// Confidence level in basis points
    pub confidence_bps: u32,
    /// Data provider (health authority)
    pub provider: Address,
    /// Timestamp when data was reported
    pub reported_at: u64,
}

/// Epidemic model parameters
#[derive(Clone)]
#[contracttype]
pub struct EpidemicModel {
    /// Model identifier
    pub model_id: BytesN<32>,
    /// Disease being modeled
    pub disease_code: String,
    /// Geographic scope (encrypted)
    pub encrypted_scope: Bytes,
    /// Model type (SEIR, SIR, etc.)
    pub model_type: String,
    /// Basic reproduction number (R0)
    pub r0_estimate: u64, // Scaled by 1000
    /// Incubation period in days
    pub incubation_days: u32,
    /// Infectious period in days
    pub infectious_days: u32,
    /// Case fatality rate in basis points
    pub case_fatality_bps: u32,
    /// Model parameters (encrypted)
    pub encrypted_params: Bytes,
    /// Prediction horizon in days
    pub prediction_horizon: u32,
    /// Model confidence in basis points
    pub confidence_bps: u32,
    /// Last updated timestamp
    pub last_updated: u64,
    /// Model creator
    pub creator: Address,
}

/// Public health alert
#[derive(Clone)]
#[contracttype]
pub struct PublicHealthAlert {
    /// Unique alert identifier
    pub alert_id: u64,
    /// Type of alert
    pub alert_type: AlertType,
    /// Severity level
    pub severity: DiseaseSeverity,
    /// Geographic scope (encrypted)
    pub encrypted_affected_regions: Bytes,
    /// Alert message
    pub message: String,
    /// Recommended actions
    pub recommended_actions: Vec<String>,
    /// Alert source (health authority)
    pub source: Address,
    /// Alert creation timestamp
    pub created_at: u64,
    /// Alert expiration timestamp
    pub expires_at: u64,
    /// Is alert active
    pub is_active: bool,
    /// Number of acknowledgments
    pub acknowledgment_count: u32,
}

/// Vaccination coverage data
#[derive(Clone)]
#[contracttype]
pub struct VaccinationCoverage {
    /// Unique coverage identifier
    pub coverage_id: BytesN<32>,
    /// Geographic region (encrypted)
    pub encrypted_region: Bytes,
    /// Vaccine type
    pub vaccine_type: String,
    /// Target population (encrypted)
    pub encrypted_target_population: u64,
    /// Number vaccinated (privacy-preserving count)
    private_vaccinated_count: u64,
    /// Coverage percentage in basis points
    pub coverage_bps: u32,
    /// Data aggregation method
    pub aggregation_method: AggregationMethod,
    /// Privacy budget used
    pub privacy_epsilon: u64,
    /// Reporting period
    pub reporting_period_start: u64,
    pub reporting_period_end: u64,
    /// Data provider
    pub provider: Address,
    /// Timestamp when reported
    pub reported_at: u64,
}

/// Environmental health data
#[derive(Clone)]
#[contracttype]
pub struct EnvironmentalHealth {
    /// Unique environmental data identifier
    pub env_data_id: BytesN<32>,
    /// Geographic location (encrypted)
    pub encrypted_location: Bytes,
    /// Environmental metric type
    pub metric_type: String, // air_quality, water_quality, temperature, etc.
    /// Measured value (privacy-preserving)
    pub aggregated_value: u64,
    /// Risk level in basis points
    pub risk_bps: u32,
    /// Measurement period
    pub measurement_period_start: u64,
    pub measurement_period_end: u64,
    /// Data aggregation method
    pub aggregation_method: AggregationMethod,
    /// Privacy budget used
    pub privacy_epsilon: u64,
    /// Environmental monitoring station
    pub monitoring_station: Address,
    /// Timestamp when measured
    pub measured_at: u64,
}

/// Antimicrobial resistance data
#[derive(Clone)]
#[contracttype]
pub struct AntimicrobialResistance {
    /// Unique AMR data identifier
    pub amr_data_id: BytesN<32>,
    /// Geographic region (encrypted)
    pub encrypted_region: Bytes,
    /// Pathogen identifier
    pub pathogen_code: String,
    /// Antibiotic class
    pub antibiotic_class: String,
    /// Resistance percentage in basis points
    pub resistance_bps: u32,
    /// Sample size (privacy-preserving)
    private_sample_size: u64,
    /// Data aggregation method
    pub aggregation_method: AggregationMethod,
    /// Privacy budget used
    pub privacy_epsilon: u64,
    /// Testing laboratory
    pub testing_lab: Address,
    /// Timestamp when tested
    pub tested_at: u64,
}

/// Social determinants of health data
#[derive(Clone)]
#[contracttype]
pub struct SocialHealthDeterminants {
    /// Unique SDOH data identifier
    pub sdoh_data_id: BytesN<32>,
    /// Geographic region (encrypted)
    pub encrypted_region: Bytes,
    /// Determinant type
    pub determinant_type: String, // income, education, housing, access_to_care, etc.
    /// Aggregated metric value (privacy-preserving)
    pub aggregated_metric: u64,
    /// Impact score in basis points
    pub impact_bps: u32,
    /// Data aggregation method
    pub aggregation_method: AggregationMethod,
    /// Privacy budget used
    pub privacy_epsilon: u64,
    /// Data source (public health agency)
    pub data_source: Address,
    /// Timestamp when collected
    pub collected_at: u64,
}

/// Public health intervention
#[derive(Clone)]
#[contracttype]
pub struct PublicHealthIntervention {
    /// Unique intervention identifier
    pub intervention_id: BytesN<32>,
    /// Intervention type
    pub intervention_type: String, // vaccination_campaign, education_program, etc.
    /// Target population (encrypted)
    pub encrypted_target_population: Bytes,
    /// Geographic scope (encrypted)
    pub encrypted_scope: Bytes,
    /// Start date
    pub start_date: u64,
    /// End date
    pub end_date: u64,
    /// Implementation cost
    pub implementation_cost: u64,
    /// Expected outcomes
    pub expected_outcomes: Vec<String>,
    /// Measured effectiveness in basis points
    pub effectiveness_bps: u32,
    /// Data aggregation method
    pub aggregation_method: AggregationMethod,
    /// Intervention coordinator
    pub coordinator: Address,
    /// Timestamp when created
    pub created_at: u64,
}

/// Global health collaboration data
#[derive(Clone)]
#[contracttype]
pub struct GlobalHealthCollaboration {
    /// Unique collaboration identifier
    pub collaboration_id: BytesN<32>,
    /// Participating countries/regions
    pub participants: Vec<Address>,
    /// Collaboration type
    pub collaboration_type: String, // research, surveillance, response, etc.
    /// Data sharing protocol
    pub data_sharing_protocol: String,
    /// Privacy-preserving data exchange method
    pub exchange_method: AggregationMethod,
    /// Collaboration objectives
    pub objectives: Vec<String>,
    /// Start date
    pub start_date: u64,
    /// End date (0 for ongoing)
    pub end_date: u64,
    /// Collaboration lead
    pub lead_organization: Address,
    /// Timestamp when established
    pub established_at: u64,
}

// =============================================================================
// Storage
// =============================================================================

#[contracttype]
pub enum DataKey {
    Initialized,
    Admin,
    OutbreakData(BytesN<32>),
    EpidemicModel(BytesN<32>),
    PublicHealthAlert(u64),
    VaccinationCoverage(BytesN<32>),
    EnvironmentalHealth(BytesN<32>),
    AntimicrobialResistance(BytesN<32>),
    SocialHealthDeterminants(BytesN<32>),
    PublicHealthIntervention(BytesN<32>),
    GlobalHealthCollaboration(BytesN<32>),
    AlertCounter,
    ModelCounter,
    CoverageCounter,
    InterventionCounter,
    CollaborationCounter,
    PrivacyBudget(Address),
}

const ADMIN: Symbol = symbol_short!("ADMIN");

// =============================================================================
// Errors
// =============================================================================

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    AlreadyInitialized = 1,
    NotInitialized = 2,
    NotAuthorized = 3,
    InvalidInput = 4,
    DataNotFound = 5,
    InvalidAggregationMethod = 6,
    PrivacyBudgetExceeded = 7,
    InsufficientPrivilege = 8,
    InvalidSeverity = 9,
    AlertExpired = 10,
    ModelNotFound = 11,
    InterventionNotFound = 12,
    CollaborationNotFound = 13,
    InvalidTimeRange = 14,
    InvalidRegion = 15,
}

// =============================================================================
// Contract
// =============================================================================

#[contract]
pub struct PublicHealthSurveillance;

#[contractimpl]
impl PublicHealthSurveillance {
    /// Initialize the public health surveillance platform
    pub fn initialize(env: Env, admin: Address) -> Result<(), Error> {
        admin.require_auth();
        if env.storage().instance().has(&DataKey::Initialized) {
            return Err(Error::AlreadyInitialized);
        }
        env.storage().instance().set(&DataKey::Initialized, &true);
        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage().persistent().set(&ADMIN, &admin);
        env.events()
            .publish((symbol_short!("phs"), symbol_short!("init")), admin);
        Ok(())
    }

    /// Report outbreak data with privacy preservation
    #[allow(clippy::too_many_arguments)]
    pub fn report_outbreak_data(
        env: Env,
        provider: Address,
        data_id: BytesN<32>,
        encrypted_region: Bytes,
        disease_code: String,
        aggregated_cases: u64,
        time_period_start: u64,
        time_period_end: u64,
        aggregation_method: AggregationMethod,
        privacy_epsilon: u64,
        confidence_bps: u32,
    ) -> Result<(), Error> {
        provider.require_auth();
        Self::require_initialized(&env)?;

        // Validate time range
        if time_period_start >= time_period_end {
            return Err(Error::InvalidTimeRange);
        }

        // Check privacy budget
        Self::check_privacy_budget(&env, &provider, privacy_epsilon)?;

        // Create outbreak data
        let outbreak_data = OutbreakData {
            data_id: data_id.clone(),
            encrypted_region: encrypted_region.clone(),
            disease_code: disease_code.clone(),
            aggregated_cases,
            time_period_start,
            time_period_end,
            aggregation_method,
            privacy_epsilon,
            confidence_bps,
            provider: provider.clone(),
            reported_at: env.ledger().timestamp(),
        };

        // Store outbreak data
        env.storage()
            .persistent()
            .set(&DataKey::OutbreakData(data_id), &outbreak_data);

        // Update privacy budget
        Self::update_privacy_budget(&env, &provider, privacy_epsilon);

        // Detect outbreak if threshold exceeded
        let outbreak_detected = Self::detect_outbreak_internal(&outbreak_data);

        // Emit events
        env.events().publish(
            (symbol_short!("phs"), symbol_short!("out_rpt")),
            (provider, disease_code, outbreak_detected),
        );

        if outbreak_detected {
            Self::create_automatic_alert(&env, &outbreak_data)?;
        }

        Ok(())
    }

    /// Create epidemic model for disease prediction
    #[allow(clippy::too_many_arguments)]
    pub fn create_epidemic_model(
        env: Env,
        modeler: Address,
        model_id: BytesN<32>,
        disease_code: String,
        encrypted_scope: Bytes,
        model_type: String,
        r0_estimate: u64,
        incubation_days: u32,
        infectious_days: u32,
        case_fatality_bps: u32,
    ) -> Result<(), Error> {
        modeler.require_auth();
        Self::require_initialized(&env)?;

        // Validate model parameters
        if r0_estimate == 0 || incubation_days == 0 || infectious_days == 0 {
            return Err(Error::InvalidInput);
        }

        let encrypted_params = Bytes::from_slice(&env, b"default_params");
        let prediction_horizon = 30u32;
        let confidence_bps = 9000u32;

        let model = EpidemicModel {
            model_id: model_id.clone(),
            disease_code: disease_code.clone(),
            encrypted_scope: encrypted_scope.clone(),
            model_type: model_type.clone(),
            r0_estimate,
            incubation_days,
            infectious_days,
            case_fatality_bps,
            encrypted_params: encrypted_params.clone(),
            prediction_horizon,
            confidence_bps,
            last_updated: env.ledger().timestamp(),
            creator: modeler.clone(),
        };

        // Store model
        env.storage()
            .persistent()
            .set(&DataKey::EpidemicModel(model_id), &model);

        // Update model counter
        let counter: u64 = env
            .storage()
            .persistent()
            .get(&DataKey::ModelCounter)
            .unwrap_or(0);
        env.storage()
            .persistent()
            .set(&DataKey::ModelCounter, &(counter.saturating_add(1)));

        // Emit events
        env.events().publish(
            (symbol_short!("phs"), symbol_short!("model_crt")),
            (modeler, disease_code, model_type),
        );

        Ok(())
    }

    /// Create public health alert
    #[allow(clippy::too_many_arguments)]
    pub fn create_public_health_alert(
        env: Env,
        authority: Address,
        alert_type: AlertType,
        severity: DiseaseSeverity,
        encrypted_affected_regions: Bytes,
        message: String,
        recommended_actions: Vec<String>,
        expiration_hours: u32,
    ) -> Result<u64, Error> {
        authority.require_auth();
        Self::require_initialized(&env)?;

        // Get next alert ID
        let alert_id: u64 = env
            .storage()
            .persistent()
            .get(&DataKey::AlertCounter)
            .unwrap_or(0);

        let now = env.ledger().timestamp();
        let expires_at = now.saturating_add((expiration_hours as u64) * 3600);

        let alert = PublicHealthAlert {
            alert_id,
            alert_type,
            severity,
            encrypted_affected_regions: encrypted_affected_regions.clone(),
            message: message.clone(),
            recommended_actions: recommended_actions.clone(),
            source: authority.clone(),
            created_at: now,
            expires_at,
            is_active: true,
            acknowledgment_count: 0,
        };

        // Store alert
        env.storage()
            .persistent()
            .set(&DataKey::PublicHealthAlert(alert_id), &alert);

        // Update alert counter
        env.storage()
            .persistent()
            .set(&DataKey::AlertCounter, &(alert_id.saturating_add(1)));

        // Emit events
        env.events().publish(
            (symbol_short!("phs"), symbol_short!("alert_crt")),
            (authority, alert_id, severity),
        );

        Ok(alert_id)
    }

    /// Report vaccination coverage with privacy preservation
    #[allow(clippy::too_many_arguments)]
    pub fn report_vaccination_coverage(
        env: Env,
        provider: Address,
        coverage_id: BytesN<32>,
        encrypted_region: Bytes,
        vaccine_type: String,
        encrypted_target_population: u64,
        private_vaccinated_count: u64,
        coverage_bps: u32,
        reporting_period_start: u64,
        reporting_period_end: u64,
    ) -> Result<(), Error> {
        provider.require_auth();
        Self::require_initialized(&env)?;

        // Validate time range
        if reporting_period_start >= reporting_period_end {
            return Err(Error::InvalidTimeRange);
        }

        let aggregation_method = AggregationMethod::SecureMultipartyComputation;
        let privacy_epsilon = 15u64;

        // Check privacy budget
        Self::check_privacy_budget(&env, &provider, privacy_epsilon)?;

        let coverage = VaccinationCoverage {
            coverage_id: coverage_id.clone(),
            encrypted_region: encrypted_region.clone(),
            vaccine_type: vaccine_type.clone(),
            encrypted_target_population,
            private_vaccinated_count,
            coverage_bps,
            reporting_period_start,
            reporting_period_end,
            aggregation_method,
            privacy_epsilon,
            provider: provider.clone(),
            reported_at: env.ledger().timestamp(),
        };

        // Store coverage data
        env.storage()
            .persistent()
            .set(&DataKey::VaccinationCoverage(coverage_id), &coverage);

        // Update privacy budget
        Self::update_privacy_budget(&env, &provider, privacy_epsilon);

        // Update coverage counter
        let counter: u64 = env
            .storage()
            .persistent()
            .get(&DataKey::CoverageCounter)
            .unwrap_or(0);
        env.storage()
            .persistent()
            .set(&DataKey::CoverageCounter, &(counter.saturating_add(1)));

        // Emit events
        env.events().publish(
            (symbol_short!("phs"), symbol_short!("cov_rpt")),
            (provider, vaccine_type, coverage_bps),
        );

        Ok(())
    }

    /// Report environmental health data
    #[allow(clippy::too_many_arguments)]
    pub fn report_environmental_health(
        env: Env,
        monitoring_station: Address,
        env_data_id: BytesN<32>,
        encrypted_location: Bytes,
        metric_type: String,
        aggregated_value: u64,
        risk_bps: u32,
        measurement_period_start: u64,
        measurement_period_end: u64,
        aggregation_method: AggregationMethod,
        privacy_epsilon: u64,
    ) -> Result<(), Error> {
        monitoring_station.require_auth();
        Self::require_initialized(&env)?;

        // Validate time range
        if measurement_period_start >= measurement_period_end {
            return Err(Error::InvalidTimeRange);
        }

        // Check privacy budget
        Self::check_privacy_budget(&env, &monitoring_station, privacy_epsilon)?;

        let env_health = EnvironmentalHealth {
            env_data_id: env_data_id.clone(),
            encrypted_location: encrypted_location.clone(),
            metric_type: metric_type.clone(),
            aggregated_value,
            risk_bps,
            measurement_period_start,
            measurement_period_end,
            aggregation_method,
            privacy_epsilon,
            monitoring_station: monitoring_station.clone(),
            measured_at: env.ledger().timestamp(),
        };

        // Store environmental data
        env.storage().persistent().set(
            &DataKey::EnvironmentalHealth(env_data_id.clone()),
            &env_health,
        );

        // Update privacy budget
        Self::update_privacy_budget(&env, &monitoring_station, privacy_epsilon);

        // Check for environmental hazards
        if risk_bps > 8000 {
            // High risk threshold
            Self::create_environmental_alert(&env, &env_health)?;
        }

        // Emit events
        env.events().publish(
            (symbol_short!("phs"), symbol_short!("env_rpt")),
            (monitoring_station, metric_type, risk_bps),
        );

        Ok(())
    }

    /// Report antimicrobial resistance data
    #[allow(clippy::too_many_arguments)]
    pub fn report_antimicrobial_resistance(
        env: Env,
        testing_lab: Address,
        amr_data_id: BytesN<32>,
        encrypted_region: Bytes,
        pathogen_code: String,
        antibiotic_class: String,
        resistance_bps: u32,
        private_sample_size: u64,
        aggregation_method: AggregationMethod,
        privacy_epsilon: u64,
    ) -> Result<(), Error> {
        testing_lab.require_auth();
        Self::require_initialized(&env)?;

        // Check privacy budget
        Self::check_privacy_budget(&env, &testing_lab, privacy_epsilon)?;

        let amr_data = AntimicrobialResistance {
            amr_data_id: amr_data_id.clone(),
            encrypted_region: encrypted_region.clone(),
            pathogen_code: pathogen_code.clone(),
            antibiotic_class: antibiotic_class.clone(),
            resistance_bps,
            private_sample_size,
            aggregation_method,
            privacy_epsilon,
            testing_lab: testing_lab.clone(),
            tested_at: env.ledger().timestamp(),
        };

        // Store AMR data
        env.storage().persistent().set(
            &DataKey::AntimicrobialResistance(amr_data_id.clone()),
            &amr_data,
        );

        // Update privacy budget
        Self::update_privacy_budget(&env, &testing_lab, privacy_epsilon);

        // Check for high resistance levels
        if resistance_bps > 5000 {
            // 50% resistance threshold
            Self::create_amr_alert(&env, &amr_data)?;
        }

        // Emit events
        env.events().publish(
            (symbol_short!("phs"), symbol_short!("amr_rpt")),
            (testing_lab, pathogen_code, resistance_bps),
        );

        Ok(())
    }

    /// Report social determinants of health data
    #[allow(clippy::too_many_arguments)]
    pub fn report_social_determinants(
        env: Env,
        data_source: Address,
        sdoh_data_id: BytesN<32>,
        encrypted_region: Bytes,
        determinant_type: String,
        aggregated_metric: u64,
        impact_bps: u32,
        aggregation_method: AggregationMethod,
        privacy_epsilon: u64,
    ) -> Result<(), Error> {
        data_source.require_auth();
        Self::require_initialized(&env)?;

        // Check privacy budget
        Self::check_privacy_budget(&env, &data_source, privacy_epsilon)?;

        let sdoh_data = SocialHealthDeterminants {
            sdoh_data_id: sdoh_data_id.clone(),
            encrypted_region: encrypted_region.clone(),
            determinant_type: determinant_type.clone(),
            aggregated_metric,
            impact_bps,
            aggregation_method,
            privacy_epsilon,
            data_source: data_source.clone(),
            collected_at: env.ledger().timestamp(),
        };

        // Store SDOH data
        env.storage().persistent().set(
            &DataKey::SocialHealthDeterminants(sdoh_data_id.clone()),
            &sdoh_data,
        );

        // Update privacy budget
        Self::update_privacy_budget(&env, &data_source, privacy_epsilon);

        // Emit events
        env.events().publish(
            (symbol_short!("phs"), symbol_short!("sdoh_rpt")),
            (data_source, determinant_type, impact_bps),
        );

        Ok(())
    }

    /// Create public health intervention
    #[allow(clippy::too_many_arguments)]
    pub fn create_intervention(
        env: Env,
        coordinator: Address,
        intervention_id: BytesN<32>,
        intervention_type: String,
        encrypted_target_population: Bytes,
        encrypted_scope: Bytes,
        start_date: u64,
        end_date: u64,
        implementation_cost: u64,
        expected_outcomes: Vec<String>,
        aggregation_method: AggregationMethod,
    ) -> Result<(), Error> {
        coordinator.require_auth();
        Self::require_initialized(&env)?;

        // Validate time range
        if start_date >= end_date {
            return Err(Error::InvalidTimeRange);
        }

        let intervention = PublicHealthIntervention {
            intervention_id: intervention_id.clone(),
            intervention_type: intervention_type.clone(),
            encrypted_target_population: encrypted_target_population.clone(),
            encrypted_scope: encrypted_scope.clone(),
            start_date,
            end_date,
            implementation_cost,
            expected_outcomes: expected_outcomes.clone(),
            effectiveness_bps: 0, // Will be updated when measured
            aggregation_method,
            coordinator: coordinator.clone(),
            created_at: env.ledger().timestamp(),
        };

        // Store intervention
        env.storage().persistent().set(
            &DataKey::PublicHealthIntervention(intervention_id.clone()),
            &intervention,
        );

        // Update intervention counter
        let counter: u64 = env
            .storage()
            .persistent()
            .get(&DataKey::InterventionCounter)
            .unwrap_or(0);
        env.storage()
            .persistent()
            .set(&DataKey::InterventionCounter, &(counter.saturating_add(1)));

        // Emit events
        env.events().publish(
            (symbol_short!("phs"), symbol_short!("intv_crt")),
            (coordinator, intervention_type, implementation_cost),
        );

        Ok(())
    }

    /// Create global health collaboration
    #[allow(clippy::too_many_arguments)]
    pub fn create_global_collaboration(
        env: Env,
        lead_organization: Address,
        collaboration_id: BytesN<32>,
        participants: Vec<Address>,
        collaboration_type: String,
        data_sharing_protocol: String,
        exchange_method: AggregationMethod,
        objectives: Vec<String>,
        start_date: u64,
        end_date: u64,
    ) -> Result<(), Error> {
        lead_organization.require_auth();
        Self::require_initialized(&env)?;

        // Validate participants
        if participants.is_empty() {
            return Err(Error::InvalidInput);
        }

        let collaboration = GlobalHealthCollaboration {
            collaboration_id: collaboration_id.clone(),
            participants: participants.clone(),
            collaboration_type: collaboration_type.clone(),
            data_sharing_protocol: data_sharing_protocol.clone(),
            exchange_method,
            objectives: objectives.clone(),
            start_date,
            end_date,
            lead_organization: lead_organization.clone(),
            established_at: env.ledger().timestamp(),
        };

        // Store collaboration
        env.storage().persistent().set(
            &DataKey::GlobalHealthCollaboration(collaboration_id.clone()),
            &collaboration,
        );

        // Update collaboration counter
        let counter: u64 = env
            .storage()
            .persistent()
            .get(&DataKey::CollaborationCounter)
            .unwrap_or(0);
        env.storage()
            .persistent()
            .set(&DataKey::CollaborationCounter, &(counter.saturating_add(1)));

        // Emit events
        env.events().publish(
            (symbol_short!("phs"), symbol_short!("colab_crt")),
            (lead_organization, collaboration_type, participants.len()),
        );

        Ok(())
    }

    /// Get outbreak data
    pub fn get_outbreak_data(env: Env, data_id: BytesN<32>) -> Result<OutbreakData, Error> {
        Self::require_initialized(&env)?;
        env.storage()
            .persistent()
            .get(&DataKey::OutbreakData(data_id))
            .ok_or(Error::DataNotFound)
    }

    /// Get epidemic model
    pub fn get_epidemic_model(env: Env, model_id: BytesN<32>) -> Result<EpidemicModel, Error> {
        Self::require_initialized(&env)?;
        env.storage()
            .persistent()
            .get(&DataKey::EpidemicModel(model_id))
            .ok_or(Error::ModelNotFound)
    }

    /// Get public health alert
    pub fn get_public_health_alert(env: Env, alert_id: u64) -> Result<PublicHealthAlert, Error> {
        Self::require_initialized(&env)?;
        env.storage()
            .persistent()
            .get(&DataKey::PublicHealthAlert(alert_id))
            .ok_or(Error::DataNotFound)
    }

    /// Get vaccination coverage
    pub fn get_vaccination_coverage(
        env: Env,
        coverage_id: BytesN<32>,
    ) -> Result<VaccinationCoverage, Error> {
        Self::require_initialized(&env)?;
        env.storage()
            .persistent()
            .get(&DataKey::VaccinationCoverage(coverage_id))
            .ok_or(Error::DataNotFound)
    }

    /// Get environmental health data
    pub fn get_environmental_health(
        env: Env,
        env_data_id: BytesN<32>,
    ) -> Result<EnvironmentalHealth, Error> {
        Self::require_initialized(&env)?;
        env.storage()
            .persistent()
            .get(&DataKey::EnvironmentalHealth(env_data_id))
            .ok_or(Error::DataNotFound)
    }

    /// Get antimicrobial resistance data
    pub fn get_antimicrobial_resistance(
        env: Env,
        amr_data_id: BytesN<32>,
    ) -> Result<AntimicrobialResistance, Error> {
        Self::require_initialized(&env)?;
        env.storage()
            .persistent()
            .get(&DataKey::AntimicrobialResistance(amr_data_id))
            .ok_or(Error::DataNotFound)
    }

    /// Get social determinants of health data
    pub fn get_social_determinants(
        env: Env,
        sdoh_data_id: BytesN<32>,
    ) -> Result<SocialHealthDeterminants, Error> {
        Self::require_initialized(&env)?;
        env.storage()
            .persistent()
            .get(&DataKey::SocialHealthDeterminants(sdoh_data_id))
            .ok_or(Error::DataNotFound)
    }

    /// Get public health intervention
    pub fn get_public_health_intervention(
        env: Env,
        intervention_id: BytesN<32>,
    ) -> Result<PublicHealthIntervention, Error> {
        Self::require_initialized(&env)?;
        env.storage()
            .persistent()
            .get(&DataKey::PublicHealthIntervention(intervention_id))
            .ok_or(Error::InterventionNotFound)
    }

    /// Get global health collaboration
    pub fn get_global_collaboration(
        env: Env,
        collaboration_id: BytesN<32>,
    ) -> Result<GlobalHealthCollaboration, Error> {
        Self::require_initialized(&env)?;
        env.storage()
            .persistent()
            .get(&DataKey::GlobalHealthCollaboration(collaboration_id))
            .ok_or(Error::CollaborationNotFound)
    }

    /// Get privacy budget for address
    pub fn get_privacy_budget(env: Env, user: Address) -> Result<u64, Error> {
        Self::require_initialized(&env)?;
        Ok(env
            .storage()
            .persistent()
            .get(&DataKey::PrivacyBudget(user))
            .unwrap_or(1000)) // Default privacy budget
    }

    // -------------------------------------------------------------------------
    // Internal helper functions
    // -------------------------------------------------------------------------

    fn require_initialized(env: &Env) -> Result<(), Error> {
        if !env.storage().instance().has(&DataKey::Initialized) {
            return Err(Error::NotInitialized);
        }
        Ok(())
    }

    /// Detect outbreak from aggregated data
    fn detect_outbreak_internal(data: &OutbreakData) -> bool {
        // Simple outbreak detection: if cases exceed threshold based on confidence
        let threshold_multiplier = match data.confidence_bps {
            0..=3000 => 2.0,    // Low confidence
            3001..=7000 => 1.5, // Medium confidence
            _ => 1.0,           // High confidence
        };

        // Calculate expected baseline (simplified - in production would use historical data)
        let baseline_cases = 100u64; // Example baseline
        let outbreak_threshold = (baseline_cases as f64 * threshold_multiplier) as u64;

        data.aggregated_cases > outbreak_threshold
    }

    /// Create automatic alert for detected outbreak
    fn create_automatic_alert(env: &Env, outbreak_data: &OutbreakData) -> Result<(), Error> {
        let alert_id: u64 = env
            .storage()
            .persistent()
            .get(&DataKey::AlertCounter)
            .unwrap_or(0);

        let alert = PublicHealthAlert {
            alert_id,
            alert_type: AlertType::DiseaseOutbreak,
            severity: DiseaseSeverity::High,
            encrypted_affected_regions: outbreak_data.encrypted_region.clone(),
            message: String::from_str(env, "Automatic outbreak detection"),
            recommended_actions: Vec::new(env),
            source: outbreak_data.provider.clone(),
            created_at: env.ledger().timestamp(),
            expires_at: env.ledger().timestamp().saturating_add(24 * 3600), // 24 hours
            is_active: true,
            acknowledgment_count: 0,
        };

        env.storage()
            .persistent()
            .set(&DataKey::PublicHealthAlert(alert_id), &alert);
        env.storage()
            .persistent()
            .set(&DataKey::AlertCounter, &(alert_id.saturating_add(1)));

        env.events().publish(
            (symbol_short!("phs"), symbol_short!("auto_alrt")),
            (alert_id, DiseaseSeverity::High),
        );

        Ok(())
    }

    /// Create environmental hazard alert
    fn create_environmental_alert(
        env: &Env,
        env_health: &EnvironmentalHealth,
    ) -> Result<(), Error> {
        let alert_id: u64 = env
            .storage()
            .persistent()
            .get(&DataKey::AlertCounter)
            .unwrap_or(0);

        let alert = PublicHealthAlert {
            alert_id,
            alert_type: AlertType::EnvironmentalHazard,
            severity: match env_health.risk_bps {
                0..=3000 => DiseaseSeverity::Low,
                3001..=6000 => DiseaseSeverity::Medium,
                6001..=8000 => DiseaseSeverity::High,
                _ => DiseaseSeverity::Critical,
            },
            encrypted_affected_regions: env_health.encrypted_location.clone(),
            message: String::from_str(env, "Environmental health hazard detected"),
            recommended_actions: Vec::new(env),
            source: env_health.monitoring_station.clone(),
            created_at: env.ledger().timestamp(),
            expires_at: env.ledger().timestamp().saturating_add(12 * 3600), // 12 hours
            is_active: true,
            acknowledgment_count: 0,
        };

        env.storage()
            .persistent()
            .set(&DataKey::PublicHealthAlert(alert_id), &alert);
        env.storage()
            .persistent()
            .set(&DataKey::AlertCounter, &(alert_id.saturating_add(1)));

        env.events().publish(
            (symbol_short!("phs"), symbol_short!("env_alert")),
            (alert_id, env_health.risk_bps),
        );

        Ok(())
    }

    /// Create antimicrobial resistance alert
    fn create_amr_alert(env: &Env, amr_data: &AntimicrobialResistance) -> Result<(), Error> {
        let alert_id: u64 = env
            .storage()
            .persistent()
            .get(&DataKey::AlertCounter)
            .unwrap_or(0);

        let alert = PublicHealthAlert {
            alert_id,
            alert_type: AlertType::AntimicrobialResistance,
            severity: match amr_data.resistance_bps {
                0..=3000 => DiseaseSeverity::Medium,
                3001..=7000 => DiseaseSeverity::High,
                _ => DiseaseSeverity::Critical,
            },
            encrypted_affected_regions: amr_data.encrypted_region.clone(),
            message: String::from_str(env, "High antimicrobial resistance detected"),
            recommended_actions: Vec::new(env),
            source: amr_data.testing_lab.clone(),
            created_at: env.ledger().timestamp(),
            expires_at: env.ledger().timestamp().saturating_add(48 * 3600), // 48 hours
            is_active: true,
            acknowledgment_count: 0,
        };

        env.storage()
            .persistent()
            .set(&DataKey::PublicHealthAlert(alert_id), &alert);
        env.storage()
            .persistent()
            .set(&DataKey::AlertCounter, &(alert_id.saturating_add(1)));

        env.events().publish(
            (symbol_short!("phs"), symbol_short!("amr_alert")),
            (alert_id, amr_data.resistance_bps),
        );

        Ok(())
    }

    /// Check privacy budget for user
    fn check_privacy_budget(env: &Env, user: &Address, required_epsilon: u64) -> Result<(), Error> {
        let current_budget: u64 = env
            .storage()
            .persistent()
            .get(&DataKey::PrivacyBudget(user.clone()))
            .unwrap_or(1000);

        if required_epsilon > current_budget {
            return Err(Error::PrivacyBudgetExceeded);
        }

        Ok(())
    }

    /// Update privacy budget for user
    fn update_privacy_budget(env: &Env, user: &Address, used_epsilon: u64) {
        let current_budget: u64 = env
            .storage()
            .persistent()
            .get(&DataKey::PrivacyBudget(user.clone()))
            .unwrap_or(1000);

        let new_budget = current_budget.saturating_sub(used_epsilon);
        env.storage()
            .persistent()
            .set(&DataKey::PrivacyBudget(user.clone()), &new_budget);
    }
}
