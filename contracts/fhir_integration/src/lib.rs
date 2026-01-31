#![no_std]
#![allow(clippy::too_many_arguments)]
#![allow(dead_code)]

use soroban_sdk::symbol_short;
use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype, Address, BytesN, Env, Map, String, Symbol,
    Vec,
};

// ==================== FHIR Data Types ====================

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
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

#[derive(Clone, Copy, PartialEq, Eq, Debug)] // FIXED: Added Debug
#[contracttype]
pub enum CodingSystem {
    ICD10,
    ICD9,
    CPT,
    SNOMEDCT,
    LOINC,
    RxNorm,
    Custom,
}

#[derive(Clone, PartialEq, Eq, Debug)] // FIXED: Added Debug
#[contracttype]
pub struct FHIRCode {
    pub system: CodingSystem,
    pub code: String,
    pub display: String,
}

#[derive(Clone, PartialEq, Eq, Debug)]
#[contracttype]
pub struct FHIRIdentifier {
    pub system: String,
    pub value: String,
    pub use_type: String,
}

#[derive(Clone, PartialEq, Eq, Debug)]
#[contracttype]
pub struct FHIRPatient {
    pub identifiers: Vec<FHIRIdentifier>,
    pub given_name: String,
    pub family_name: String,
    pub birth_date: String,
    pub gender: String,
    pub contact_point: String,
    pub address: String,
    pub communication: Vec<String>,
    pub marital_status: String,
}

#[derive(Clone, PartialEq, Eq, Debug)]
#[contracttype]
pub struct FHIRObservation {
    pub identifier: String,
    pub status: String,
    pub category: FHIRCode,
    pub code: FHIRCode,
    pub subject_reference: String,
    pub effective_datetime: String,
    pub value_quantity_value: i64,
    pub value_quantity_unit: String,
    pub interpretation: Vec<FHIRCode>,
    pub reference_range: String,
}

#[derive(Clone, PartialEq, Eq, Debug)]
#[contracttype]
pub struct FHIRCondition {
    pub identifier: String,
    pub clinical_status: String,
    pub code: FHIRCode,
    pub subject_reference: String,
    pub onset_date_time: String,
    pub recorded_date: String,
    pub severity: Vec<FHIRCode>,
}

#[derive(Clone, PartialEq, Eq, Debug)]
#[contracttype]
pub struct FHIRMedicationStatement {
    pub identifier: String,
    pub status: String,
    pub medication_code: FHIRCode,
    pub subject_reference: String,
    pub effective_period_start: String,
    pub effective_period_end: String,
    pub dosage: String,
    pub reason_code: Vec<FHIRCode>,
}

#[derive(Clone, PartialEq, Eq, Debug)]
#[contracttype]
pub struct FHIRProcedure {
    pub identifier: String,
    pub status: String,
    pub code: FHIRCode,
    pub subject_reference: String,
    pub performed_date_time: String,
    pub performer: Vec<String>,
    pub reason_code: Vec<FHIRCode>,
}

#[derive(Clone, PartialEq, Eq, Debug)]
#[contracttype]
pub struct FHIRAllergyIntolerance {
    pub identifier: String,
    pub clinical_status: String,
    pub verification_status: String,
    pub substance_code: FHIRCode,
    pub patient_reference: String,
    pub recorded_date: String,
    pub manifestation: Vec<FHIRCode>,
    pub severity: String,
}

#[derive(Clone, PartialEq, Eq, Debug)]
#[contracttype]
pub struct FHIRBundle {
    pub bundle_id: String,
    pub timestamp: u64,
    pub bundle_type: String,
    pub total: u32,
}

#[derive(Clone, PartialEq, Eq, Debug)]
#[contracttype]
pub struct HealthcareProvider {
    pub provider_id: String,
    pub name: String,
    pub facility_type: String,
    pub npi: String,
    pub tax_id: String,
    pub address: String,
    pub contact_point: String,
    pub emr_system: String,
    pub fhir_endpoint: String,
    pub is_verified: bool,
    pub verification_timestamp: u64,
    pub credential_id: BytesN<32>,
}

#[derive(Clone, PartialEq, Eq, Debug)]
#[contracttype]
pub struct EMRConfiguration {
    pub provider_id: String,
    pub fhir_version: String,
    pub supported_resources: Vec<FHIRResourceType>,
    pub authentication_type: String,
    pub oauth_endpoint: String,
    pub data_format: String,
    pub batch_size: u32,
    pub retry_policy: String,
}

#[derive(Clone, PartialEq, Eq, Debug)]
#[contracttype]
pub struct DataMapping {
    pub source_system: String,
    pub source_field: String,
    pub target_system: String,
    pub target_field: String,
    pub transformation_rule: String,
    pub status: String,
}

// Storage Keys
const PROVIDERS: Symbol = symbol_short!("PROVIDERS");
const OBSERVATIONS: Symbol = symbol_short!("OBSERVE");
const CONDITIONS: Symbol = symbol_short!("CONDITION");
const MEDICATIONS: Symbol = symbol_short!("MEDICATE");
const PROCEDURES: Symbol = symbol_short!("PROCEDURE");
const ALLERGIES: Symbol = symbol_short!("ALLERGIES");
const EMR_CONFIG: Symbol = symbol_short!("EMR_CFG");
const DATA_MAPPINGS: Symbol = symbol_short!("MAPPINGS");
const ADMIN: Symbol = symbol_short!("ADMIN");
const MEDICAL_RECORD_CONTRACT: Symbol = symbol_short!("MED_REC");
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
    pub fn initialize(
        env: Env,
        admin: Address,
        medical_records_contract: Address,
    ) -> Result<bool, Error> {
        admin.require_auth();
        if env.storage().persistent().has(&ADMIN) {
            return Err(Error::ProviderAlreadyExists);
        }
        env.storage().persistent().set(&ADMIN, &admin);
        env.storage()
            .persistent()
            .set(&MEDICAL_RECORD_CONTRACT, &medical_records_contract);
        env.storage().persistent().set(&PAUSED, &false);
        Ok(true)
    }

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
        if npi.len() != 10 {
            return Err(Error::InvalidNPI);
        }

        let mut providers: Map<String, HealthcareProvider> = env
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
            credential_id: BytesN::from_array(&env, &[0u8; 32]),
        };
        providers.set(provider_id, provider);
        env.storage().persistent().set(&PROVIDERS, &providers);
        Ok(true)
    }

    pub fn get_provider(env: Env, provider_id: String) -> Result<HealthcareProvider, Error> {
        let providers: Map<String, HealthcareProvider> = env
            .storage()
            .persistent()
            .get(&PROVIDERS)
            .ok_or(Error::ProviderNotFound)?;
        providers.get(provider_id).ok_or(Error::ProviderNotFound)
    }

    pub fn store_observation(
        env: Env,
        provider: Address,
        observation: FHIRObservation,
    ) -> Result<bool, Error> {
        provider.require_auth();
        let mut observations: Map<String, FHIRObservation> = env
            .storage()
            .persistent()
            .get(&OBSERVATIONS)
            .unwrap_or(Map::new(&env));
        observations.set(observation.identifier.clone(), observation);
        env.storage().persistent().set(&OBSERVATIONS, &observations);
        Ok(true)
    }

    pub fn get_observation(env: Env, observation_id: String) -> Result<FHIRObservation, Error> {
        let observations: Map<String, FHIRObservation> = env
            .storage()
            .persistent()
            .get(&OBSERVATIONS)
            .ok_or(Error::ObservationNotFound)?;
        observations
            .get(observation_id)
            .ok_or(Error::ObservationNotFound)
    }
}
