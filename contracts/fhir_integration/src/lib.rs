#![no_std]

#[cfg(test)]
mod test;

use soroban_sdk::symbol_short;
use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype, vec, Address, BytesN, Env, Map, String,
    Symbol, Vec,
};

// ==================== FHIR Data Types ====================

/// FHIR Resource Types supported
#[derive(Clone, Copy, PartialEq, Eq)]
#[contracttype]
pub enum FHIRResourceType {
    Patient = 0,
    Observation = 1,
    Condition = 2,
    MedicationStatement = 3,
    Procedure = 4,
    AllergyIntolerance = 5,
    CareTeam = 6,
    Encounter = 7,
    DiagnosticReport = 8,
    Immunization = 9,
    DocumentReference = 10,
}

/// FHIR Coding System (standard healthcare coding systems)
#[derive(Clone)]
#[contracttype]
pub enum CodingSystem {
    /// ICD-10 - International Classification of Diseases
    ICD10,
    /// ICD-9 - Legacy diagnosis codes
    ICD9,
    /// CPT - Current Procedural Terminology
    CPT,
    /// SNOMED CT - Clinical coding terminology
    SNOMEDCT,
    /// LOINC - Laboratory codes
    LOINC,
    /// RxNorm - Medications
    RxNorm,
    /// HL7 Custom
    Custom(String),
}

/// FHIR Code structure (coding + text)
#[derive(Clone)]
#[contracttype]
pub struct FHIRCode {
    pub system: CodingSystem,
    pub code: String,
    pub display: String,
}

/// FHIR Identifier (unique identifier with system)
#[derive(Clone)]
#[contracttype]
pub struct FHIRIdentifier {
    pub system: String,        // e.g., "urn:mrn:hospital-a"
    pub value: String,         // Actual identifier value
    pub use_type: String,      // e.g., "official", "usual", "secondary"
}

/// FHIR Patient Information (simplified)
#[derive(Clone)]
#[contracttype]
pub struct FHIRPatient {
    pub identifiers: Vec<FHIRIdentifier>,
    pub given_name: String,
    pub family_name: String,
    pub birth_date: String,    // YYYY-MM-DD format
    pub gender: String,         // male, female, other, unknown
    pub contact_point: String,  // Email/phone
    pub address: String,
    pub communication: Vec<String>, // Language codes (e.g., "en", "es")
    pub marital_status: String,
}

/// FHIR Observation (vital signs, lab results, etc.)
#[derive(Clone)]
#[contracttype]
pub struct FHIRObservation {
    pub identifier: String,
    pub status: String,         // registered, preliminary, final, amended, cancelled
    pub category: FHIRCode,
    pub code: FHIRCode,
    pub subject_reference: String,  // Reference to Patient
    pub effective_datetime: String, // ISO 8601 timestamp
    pub value_quantity_value: i64,
    pub value_quantity_unit: String,
    pub interpretation: Option<FHIRCode>,
    pub reference_range: String,    // Human-readable reference range
}

/// FHIR Condition (diagnosis)
#[derive(Clone)]
#[contracttype]
pub struct FHIRCondition {
    pub identifier: String,
    pub clinical_status: String,    // active, recurrence, remission, inactive
    pub code: FHIRCode,
    pub subject_reference: String,  // Reference to Patient
    pub onset_date_time: String,
    pub recorded_date: String,
    pub severity: Option<FHIRCode>,
}

/// FHIR Medication Statement
#[derive(Clone)]
#[contracttype]
pub struct FHIRMedicationStatement {
    pub identifier: String,
    pub status: String,             // active, completed, entered-in-error, intended, stopped, on-hold
    pub medication_code: FHIRCode,
    pub subject_reference: String,  // Reference to Patient
    pub effective_period_start: String,
    pub effective_period_end: String,
    pub dosage: String,
    pub reason_code: Vec<FHIRCode>,
}

/// FHIR Procedure
#[derive(Clone)]
#[contracttype]
pub struct FHIRProcedure {
    pub identifier: String,
    pub status: String,             // preparation, in-progress, not-done, on-hold, stopped, completed, entered-in-error, unknown
    pub code: FHIRCode,
    pub subject_reference: String,  // Reference to Patient
    pub performed_date_time: String,
    pub performer: Vec<String>,     // References to practitioners
    pub reason_code: Vec<FHIRCode>,
}

/// FHIR AllergyIntolerance
#[derive(Clone)]
#[contracttype]
pub struct FHIRAllergyIntolerance {
    pub identifier: String,
    pub clinical_status: String,    // active, inactive, resolved
    pub verification_status: String, // unconfirmed, confirmed, refuted, entered-in-error
    pub substance_code: FHIRCode,
    pub patient_reference: String,
    pub recorded_date: String,
    pub manifestation: Vec<FHIRCode>,
    pub severity: String,           // mild, moderate, severe
}

/// FHIR Bundle for batch operations
#[derive(Clone)]
#[contracttype]
pub struct FHIRBundle {
    pub bundle_id: String,
    pub timestamp: u64,
    pub bundle_type: String,        // document, message, transaction, transaction-response, batch, batch-response, history, searchset, collection
    pub total: u32,
}

/// Healthcare Provider Information (for EMR integration)
#[derive(Clone)]
#[contracttype]
pub struct HealthcareProvider {
    pub provider_id: String,
    pub name: String,
    pub facility_type: String,      // hospital, clinic, lab, pharmacy, etc.
    pub npi: String,                // National Provider Identifier
    pub tax_id: String,
    pub address: String,
    pub contact_point: String,
    pub emr_system: String,         // EHR system vendor name
    pub fhir_endpoint: String,      // Base URL for FHIR API
    pub is_verified: bool,
    pub verification_timestamp: u64,
    pub credential_id: Option<BytesN<32>>,
}

/// EMR Integration Configuration
#[derive(Clone)]
#[contracttype]
pub struct EMRConfiguration {
    pub provider_id: String,
    pub fhir_version: String,       // e.g., "R4", "R5"
    pub supported_resources: Vec<FHIRResourceType>,
    pub authentication_type: String, // "oauth2", "api-key", "mutual-tls"
    pub oauth_endpoint: String,
    pub data_format: String,        // "json", "xml"
    pub batch_size: u32,
    pub retry_policy: String,       // Retry configuration
}

/// Healthcare Data Mapping (for format conversion)
#[derive(Clone)]
#[contracttype]
pub struct DataMapping {
    pub source_system: String,
    pub source_field: String,
    pub target_system: String,
    pub target_field: String,
    pub transformation_rule: String, // Description of transformation
    pub status: String,             // active, deprecated, deprecated
}

// Storage Keys
const PROVIDERS: Symbol = symbol_short!("PROVIDERS");
const OBSERVATIONS: Symbol = symbol_short!("OBSERVE");
const CONDITIONS: Symbol = symbol_short!("CONDITION");
const MEDICATIONS: Symbol = symbol_short!("MEDICATE");
const PROCEDURES: Symbol = symbol_short!("PROCEDURE");
const ALLERGIES: Symbol = symbol_short!("ALLERGIES");
const BUNDLES: Symbol = symbol_short!("BUNDLES");
const EMR_CONFIG: Symbol = symbol_short!("EMR_CFG");
const DATA_MAPPINGS: Symbol = symbol_short!("MAPPINGS");
const ADMIN: Symbol = symbol_short!("ADMIN");
const MEDICAL_RECORD_CONTRACT: Symbol = symbol_short!("MED_REC");
const PROVIDER_COUNT: Symbol = symbol_short!("PROV_CNT");
const PAUSED: Symbol = symbol_short!("PAUSED");

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    NotAuthorized = 1,
    ContractPaused = 2,
    ProviderNotFound = 3,
    ProviderAlreadyExists = 4,
    ObservationNotFound = 5,
    ConditionNotFound = 6,
    InvalidFHIRData = 7,
    EMRConfigNotSet = 8,
    InvalidResourceType = 9,
    MappingNotFound = 10,
    ProviderNotVerified = 11,
    InvalidNPI = 12,
    InvalidTaxId = 13,
    BundleNotFound = 14,
    InvalidDataFormat = 15,
    ProviderAlreadyVerified = 16,
    MedicalRecordsContractNotSet = 17,
    OperationFailed = 18,
    InvalidBundleType = 19,
    DataMappingFailed = 20,
}

#[contract]
pub struct FHIRIntegrationContract;

#[contractimpl]
impl FHIRIntegrationContract {
    /// Initialize the FHIR integration contract
    pub fn initialize(env: Env, admin: Address, medical_records_contract: Address) -> Result<bool, Error> {
        admin.require_auth();

        // Check if already initialized
        if env.storage().persistent().has(&ADMIN) {
            return Err(Error::ProviderAlreadyExists);
        }

        env.storage().persistent().set(&ADMIN, &admin);
        env.storage().persistent().set(&MEDICAL_RECORD_CONTRACT, &medical_records_contract);
        env.storage().persistent().set(&PAUSED, &false);

        Ok(true)
    }

    /// Register a healthcare provider with EMR system
    pub fn register_provider(
        env: Env,
        admin: Address,
        provider_id: String,
        name: String,
        facility_type: String,
        npi: String,
        tax_id: String,
        address: String,
        contact_point: String,
        emr_system: String,
        fhir_endpoint: String,
    ) -> Result<bool, Error> {
        admin.require_auth();

        // Check authorization
        let contract_admin: Address = env
            .storage()
            .persistent()
            .get(&ADMIN)
            .ok_or(Error::NotAuthorized)?;
        if admin != contract_admin {
            return Err(Error::NotAuthorized);
        }

        if env.storage().persistent().get(&PAUSED).unwrap_or(false) {
            return Err(Error::ContractPaused);
        }

        // Validate NPI format (simple validation - should be 10 digits)
        if npi.len() != 10 {
            return Err(Error::InvalidNPI);
        }

        // Validate Tax ID format (simple validation)
        if tax_id.is_empty() || tax_id.len() > 20 {
            return Err(Error::InvalidTaxId);
        }

        // Check if provider already exists
        let providers: Map<String, HealthcareProvider> = env
            .storage()
            .persistent()
            .get(&PROVIDERS)
            .unwrap_or(Map::new(&env));

        if providers.contains_key(provider_id.clone()) {
            return Err(Error::ProviderAlreadyExists);
        }

        let provider = HealthcareProvider {
            provider_id: provider_id.clone(),
            name,
            facility_type,
            npi,
            tax_id,
            address,
            contact_point,
            emr_system,
            fhir_endpoint,
            is_verified: false,
            verification_timestamp: 0,
            credential_id: None,
        };

        let mut providers_map = providers;
        providers_map.set(provider_id, provider);
        env.storage().persistent().set(&PROVIDERS, &providers_map);

        Ok(true)
    }

    /// Verify a healthcare provider (onboarding completion)
    pub fn verify_provider(
        env: Env,
        admin: Address,
        provider_id: String,
        credential_id: BytesN<32>,
    ) -> Result<bool, Error> {
        admin.require_auth();

        // Check authorization
        let contract_admin: Address = env
            .storage()
            .persistent()
            .get(&ADMIN)
            .ok_or(Error::NotAuthorized)?;
        if admin != contract_admin {
            return Err(Error::NotAuthorized);
        }

        let mut providers: Map<String, HealthcareProvider> = env
            .storage()
            .persistent()
            .get(&PROVIDERS)
            .ok_or(Error::ProviderNotFound)?;

        let mut provider = providers
            .get(provider_id.clone())
            .ok_or(Error::ProviderNotFound)?;

        if provider.is_verified {
            return Err(Error::ProviderAlreadyVerified);
        }

        provider.is_verified = true;
        provider.verification_timestamp = env.ledger().timestamp();
        provider.credential_id = Some(credential_id);

        providers.set(provider_id, provider);
        env.storage().persistent().set(&PROVIDERS, &providers_map);

        Ok(true)
    }

    /// Get provider information
    pub fn get_provider(env: Env, provider_id: String) -> Result<HealthcareProvider, Error> {
        let providers: Map<String, HealthcareProvider> = env
            .storage()
            .persistent()
            .get(&PROVIDERS)
            .ok_or(Error::ProviderNotFound)?;

        providers
            .get(provider_id)
            .ok_or(Error::ProviderNotFound)
    }

    /// Configure EMR system for a provider
    pub fn configure_emr(
        env: Env,
        admin: Address,
        provider_id: String,
        fhir_version: String,
        supported_resources: Vec<FHIRResourceType>,
        authentication_type: String,
        oauth_endpoint: String,
        data_format: String,
        batch_size: u32,
        retry_policy: String,
    ) -> Result<bool, Error> {
        admin.require_auth();

        // Check authorization
        let contract_admin: Address = env
            .storage()
            .persistent()
            .get(&ADMIN)
            .ok_or(Error::NotAuthorized)?;
        if admin != contract_admin {
            return Err(Error::NotAuthorized);
        }

        // Verify provider exists and is verified
        let providers: Map<String, HealthcareProvider> = env
            .storage()
            .persistent()
            .get(&PROVIDERS)
            .ok_or(Error::ProviderNotFound)?;

        let provider = providers
            .get(provider_id.clone())
            .ok_or(Error::ProviderNotFound)?;

        if !provider.is_verified {
            return Err(Error::ProviderNotVerified);
        }

        let config = EMRConfiguration {
            provider_id,
            fhir_version,
            supported_resources,
            authentication_type,
            oauth_endpoint,
            data_format,
            batch_size,
            retry_policy,
        };

        let mut configs: Map<String, EMRConfiguration> = env
            .storage()
            .persistent()
            .get(&EMR_CONFIG)
            .unwrap_or(Map::new(&env));

        configs.set(config.provider_id.clone(), config);
        env.storage().persistent().set(&EMR_CONFIG, &configs);

        Ok(true)
    }

    /// Store an observation (vital signs, lab results, etc.)
    pub fn store_observation(
        env: Env,
        provider: Address,
        observation: FHIRObservation,
    ) -> Result<bool, Error> {
        provider.require_auth();

        if env.storage().persistent().get(&PAUSED).unwrap_or(false) {
            return Err(Error::ContractPaused);
        }

        let mut observations: Map<String, FHIRObservation> = env
            .storage()
            .persistent()
            .get(&OBSERVATIONS)
            .unwrap_or(Map::new(&env));

        observations.set(observation.identifier.clone(), observation);
        env.storage()
            .persistent()
            .set(&OBSERVATIONS, &observations);

        Ok(true)
    }

    /// Get observation by identifier
    pub fn get_observation(
        env: Env,
        observation_id: String,
    ) -> Result<FHIRObservation, Error> {
        let observations: Map<String, FHIRObservation> = env
            .storage()
            .persistent()
            .get(&OBSERVATIONS)
            .ok_or(Error::ObservationNotFound)?;

        observations
            .get(observation_id)
            .ok_or(Error::ObservationNotFound)
    }

    /// Store a condition (diagnosis)
    pub fn store_condition(
        env: Env,
        provider: Address,
        condition: FHIRCondition,
    ) -> Result<bool, Error> {
        provider.require_auth();

        if env.storage().persistent().get(&PAUSED).unwrap_or(false) {
            return Err(Error::ContractPaused);
        }

        let mut conditions: Map<String, FHIRCondition> = env
            .storage()
            .persistent()
            .get(&CONDITIONS)
            .unwrap_or(Map::new(&env));

        conditions.set(condition.identifier.clone(), condition);
        env.storage().persistent().set(&CONDITIONS, &conditions);

        Ok(true)
    }

    /// Get condition by identifier
    pub fn get_condition(env: Env, condition_id: String) -> Result<FHIRCondition, Error> {
        let conditions: Map<String, FHIRCondition> = env
            .storage()
            .persistent()
            .get(&CONDITIONS)
            .ok_or(Error::ConditionNotFound)?;

        conditions
            .get(condition_id)
            .ok_or(Error::ConditionNotFound)
    }

    /// Store medication statement
    pub fn store_medication(
        env: Env,
        provider: Address,
        medication: FHIRMedicationStatement,
    ) -> Result<bool, Error> {
        provider.require_auth();

        if env.storage().persistent().get(&PAUSED).unwrap_or(false) {
            return Err(Error::ContractPaused);
        }

        let mut medications: Map<String, FHIRMedicationStatement> = env
            .storage()
            .persistent()
            .get(&MEDICATIONS)
            .unwrap_or(Map::new(&env));

        medications.set(medication.identifier.clone(), medication);
        env.storage()
            .persistent()
            .set(&MEDICATIONS, &medications);

        Ok(true)
    }

    /// Get medication statement by identifier
    pub fn get_medication(
        env: Env,
        medication_id: String,
    ) -> Result<FHIRMedicationStatement, Error> {
        let medications: Map<String, FHIRMedicationStatement> = env
            .storage()
            .persistent()
            .get(&MEDICATIONS)
            .ok_or(Error::ConditionNotFound)?;

        medications
            .get(medication_id)
            .ok_or(Error::ConditionNotFound)
    }

    /// Store procedure
    pub fn store_procedure(
        env: Env,
        provider: Address,
        procedure: FHIRProcedure,
    ) -> Result<bool, Error> {
        provider.require_auth();

        if env.storage().persistent().get(&PAUSED).unwrap_or(false) {
            return Err(Error::ContractPaused);
        }

        let mut procedures: Map<String, FHIRProcedure> = env
            .storage()
            .persistent()
            .get(&PROCEDURES)
            .unwrap_or(Map::new(&env));

        procedures.set(procedure.identifier.clone(), procedure);
        env.storage()
            .persistent()
            .set(&PROCEDURES, &procedures);

        Ok(true)
    }

    /// Get procedure by identifier
    pub fn get_procedure(env: Env, procedure_id: String) -> Result<FHIRProcedure, Error> {
        let procedures: Map<String, FHIRProcedure> = env
            .storage()
            .persistent()
            .get(&PROCEDURES)
            .ok_or(Error::ConditionNotFound)?;

        procedures
            .get(procedure_id)
            .ok_or(Error::ConditionNotFound)
    }

    /// Store allergy intolerance
    pub fn store_allergy(
        env: Env,
        provider: Address,
        allergy: FHIRAllergyIntolerance,
    ) -> Result<bool, Error> {
        provider.require_auth();

        if env.storage().persistent().get(&PAUSED).unwrap_or(false) {
            return Err(Error::ContractPaused);
        }

        let mut allergies: Map<String, FHIRAllergyIntolerance> = env
            .storage()
            .persistent()
            .get(&ALLERGIES)
            .unwrap_or(Map::new(&env));

        allergies.set(allergy.identifier.clone(), allergy);
        env.storage().persistent().set(&ALLERGIES, &allergies);

        Ok(true)
    }

    /// Get allergy intolerance by identifier
    pub fn get_allergy(env: Env, allergy_id: String) -> Result<FHIRAllergyIntolerance, Error> {
        let allergies: Map<String, FHIRAllergyIntolerance> = env
            .storage()
            .persistent()
            .get(&ALLERGIES)
            .ok_or(Error::ConditionNotFound)?;

        allergies
            .get(allergy_id)
            .ok_or(Error::ConditionNotFound)
    }

    /// Register data mapping for format conversion
    pub fn register_data_mapping(
        env: Env,
        admin: Address,
        mapping: DataMapping,
    ) -> Result<bool, Error> {
        admin.require_auth();

        let contract_admin: Address = env
            .storage()
            .persistent()
            .get(&ADMIN)
            .ok_or(Error::NotAuthorized)?;
        if admin != contract_admin {
            return Err(Error::NotAuthorized);
        }

        let mut mappings: Map<String, DataMapping> = env
            .storage()
            .persistent()
            .get(&DATA_MAPPINGS)
            .unwrap_or(Map::new(&env));

        let key = String::from_str(
            &env,
            &format!(
                "{}-{}",
                &mapping.source_system,
                &mapping.source_field
            ),
        );

        mappings.set(key, mapping);
        env.storage().persistent().set(&DATA_MAPPINGS, &mappings);

        Ok(true)
    }

    /// Get data mapping
    pub fn get_data_mapping(
        env: Env,
        source_system: String,
        source_field: String,
    ) -> Result<DataMapping, Error> {
        let mappings: Map<String, DataMapping> = env
            .storage()
            .persistent()
            .get(&DATA_MAPPINGS)
            .ok_or(Error::MappingNotFound)?;

        let key = String::from_str(
            &env,
            &format!("{}-{}", &source_system, &source_field),
        );

        mappings.get(key).ok_or(Error::MappingNotFound)
    }

    /// Pause contract operations (emergency)
    pub fn pause(env: Env, admin: Address) -> Result<bool, Error> {
        admin.require_auth();

        let contract_admin: Address = env
            .storage()
            .persistent()
            .get(&ADMIN)
            .ok_or(Error::NotAuthorized)?;
        if admin != contract_admin {
            return Err(Error::NotAuthorized);
        }

        env.storage().persistent().set(&PAUSED, &true);
        Ok(true)
    }

    /// Resume contract operations
    pub fn resume(env: Env, admin: Address) -> Result<bool, Error> {
        admin.require_auth();

        let contract_admin: Address = env
            .storage()
            .persistent()
            .get(&ADMIN)
            .ok_or(Error::NotAuthorized)?;
        if admin != contract_admin {
            return Err(Error::NotAuthorized);
        }

        env.storage().persistent().set(&PAUSED, &false);
        Ok(true)
    }
}
