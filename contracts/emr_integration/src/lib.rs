#![no_std]
#![allow(clippy::too_many_arguments)]

// FIXED: Removed 'mod test;' because the file does not exist.
// Once you create contracts/emr_integration/src/test.rs, you can uncomment this.
// #[cfg(test)]
// mod test;

use soroban_sdk::symbol_short;
use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype, vec, Address, BytesN, Env, Map, String,
    Symbol, Vec,
};

// ==================== EMR System Types ====================

/// EMR System Status
#[derive(Clone, Copy, PartialEq, Eq)]
#[contracttype]
pub enum EMRStatus {
    Active,
    Inactive,
    Suspended,
    Decommissioned,
}

/// Integration Status
#[derive(Clone, Copy, PartialEq, Eq)]
#[contracttype]
pub enum IntegrationStatus {
    Pending,
    InProgress,
    Completed,
    Failed,
    Paused,
}

/// EMR System Information
#[derive(Clone)]
#[contracttype]
pub struct EMRSystem {
    pub system_id: String,
    pub vendor_name: String,
    pub vendor_contact: String,
    pub system_version: String,
    pub supported_standards: Vec<String>, // HL7 v2, HL7 FHIR, CDA, etc.
    pub api_endpoints: Vec<String>,       // Available API endpoints
    pub status: EMRStatus,
    pub last_activity: u64,
    pub integration_date: u64,
}

/// Provider Onboarding Record
#[derive(Clone)]
#[contracttype]
pub struct ProviderOnboarding {
    pub onboarding_id: String,
    pub provider_id: String,
    pub provider_name: String,
    pub provider_email: String,
    pub facility_name: String,
    pub npi: String,
    pub emr_system_id: String,
    pub status: IntegrationStatus,
    pub created_at: u64,
    pub completed_at: u64,
    pub verification_document_hash: BytesN<32>,
    pub compliance_checklist: Vec<String>,
    pub notes: String,
}

/// Provider Verification Record
#[derive(Clone)]
#[contracttype]
pub struct ProviderVerification {
    pub verification_id: String,
    pub provider_id: String,
    pub verified_by: Address,
    pub verification_timestamp: u64,
    pub license_number: String,
    pub license_state: String,
    pub license_expiration: String,
    pub board_certification: Vec<String>,
    pub malpractice_insurance: String,
    pub background_check_id: String,
    pub verification_status: String, // approved, rejected, pending, expired
}

/// Healthcare Network Node (for provider directory)
#[derive(Clone)]
#[contracttype]
pub struct NetworkNode {
    pub node_id: String,
    pub provider_id: String,
    pub node_type: String, // hospital, clinic, lab, pharmacy, specialist
    pub network_name: String,
    pub geographic_region: String,
    pub specialties: Vec<String>,
    pub bed_capacity: u32, // 0 for non-hospital
    pub operating_hours: String,
    pub emergency_services: bool,
    pub telemedicine_enabled: bool,
    pub coordinates: String,     // lat,long format
    pub connectivity_score: u32, // 0-100
}

/// Interoperability Agreement
#[derive(Clone)]
#[contracttype]
pub struct InteroperabilityAgreement {
    pub agreement_id: String,
    pub initiating_provider: String,
    pub receiving_provider: String,
    pub effective_date: String,
    pub expiration_date: String,
    pub supported_data_types: Vec<String>,
    pub access_level: String,      // read-only, read-write, limited, full
    pub audit_requirement: String, // none, monthly, quarterly, yearly
    pub data_encryption: String,   // TLS, end-to-end, both
    pub status: String,            // active, suspended, terminated
}

/// Interoperability Test Result
#[derive(Clone)]
#[contracttype]
pub struct InteroperabilityTest {
    pub test_id: String,
    pub test_date: u64,
    pub provider_a: String,
    pub provider_b: String,
    pub test_type: String, // data-exchange, api-connectivity, format-conversion, performance
    pub result_status: String, // passed, failed, partial
    pub success_rate: u32, // 0-100
    pub data_exchanged: u64, // bytes
    pub latency_ms: u32,
    pub error_details: String,
    pub tester_address: Address,
}

// Storage Keys
const ADMIN: Symbol = symbol_short!("ADMIN");
const EMR_SYSTEMS: Symbol = symbol_short!("EMR_SYS");
const PROVIDER_ONBOARDING: Symbol = symbol_short!("ONBOARD");
const PROVIDER_VERIFICATION: Symbol = symbol_short!("VERIFY");
const NETWORK_NODES: Symbol = symbol_short!("NODES");
const INTEROP_AGREEMENTS: Symbol = symbol_short!("AGREE");
const INTEROP_TESTS: Symbol = symbol_short!("TESTS");
const PAUSED: Symbol = symbol_short!("PAUSED");
const FHIR_CONTRACT: Symbol = symbol_short!("FHIR");

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    NotAuthorized = 1,
    ContractPaused = 2,
    EMRSystemNotFound = 3,
    EMRSystemAlreadyExists = 4,
    OnboardingNotFound = 5,
    OnboardingAlreadyExists = 6,
    VerificationNotFound = 7,
    NetworkNodeNotFound = 8,
    AgreementNotFound = 9,
    TestNotFound = 10,
    InvalidStatus = 11,
    InvalidEMRSystem = 12,
    ProviderNotFound = 13,
    InvalidNPI = 14,
    InvalidLicenseNumber = 15,
    LicenseExpired = 16,
    InvalidAgreement = 17,
    AgreementNotActive = 18,
    TestFailed = 19,
    InvalidTestType = 20,
    DuplicateTest = 21,
    FHIRContractNotSet = 22,
    OperationFailed = 23,
}

#[contract]
pub struct EMRIntegrationContract;

#[contractimpl]
impl EMRIntegrationContract {
    /// Initialize the EMR integration contract
    pub fn initialize(env: Env, admin: Address, fhir_contract: Address) -> Result<bool, Error> {
        admin.require_auth();

        if env.storage().persistent().has(&ADMIN) {
            return Err(Error::EMRSystemAlreadyExists);
        }

        env.storage().persistent().set(&ADMIN, &admin);
        env.storage()
            .persistent()
            .set(&FHIR_CONTRACT, &fhir_contract);
        env.storage().persistent().set(&PAUSED, &false);

        Ok(true)
    }

    /// Register an EMR system vendor
    pub fn register_emr_system(
        env: Env,
        admin: Address,
        system_id: String,
        vendor_name: String,
        vendor_contact: String,
        system_version: String,
        supported_standards: Vec<String>,
        api_endpoints: Vec<String>,
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

        let mut systems: Map<String, EMRSystem> = env
            .storage()
            .persistent()
            .get(&EMR_SYSTEMS)
            .unwrap_or(Map::new(&env));

        if systems.contains_key(system_id.clone()) {
            return Err(Error::EMRSystemAlreadyExists);
        }

        let system = EMRSystem {
            system_id: system_id.clone(),
            vendor_name,
            vendor_contact,
            system_version,
            supported_standards,
            api_endpoints,
            status: EMRStatus::Active,
            last_activity: env.ledger().timestamp(),
            integration_date: env.ledger().timestamp(),
        };

        systems.set(system_id, system);
        env.storage().persistent().set(&EMR_SYSTEMS, &systems);

        Ok(true)
    }

    /// Get EMR system details
    pub fn get_emr_system(env: Env, system_id: String) -> Result<EMRSystem, Error> {
        let systems: Map<String, EMRSystem> = env
            .storage()
            .persistent()
            .get(&EMR_SYSTEMS)
            .ok_or(Error::EMRSystemNotFound)?;

        systems.get(system_id).ok_or(Error::EMRSystemNotFound)
    }

    /// Start provider onboarding process
    pub fn initiate_onboarding(
        env: Env,
        provider: Address,
        onboarding_id: String,
        provider_id: String,
        provider_name: String,
        provider_email: String,
        facility_name: String,
        npi: String,
        emr_system_id: String,
        compliance_checklist: Vec<String>,
    ) -> Result<bool, Error> {
        provider.require_auth();

        if env.storage().persistent().get(&PAUSED).unwrap_or(false) {
            return Err(Error::ContractPaused);
        }

        // Validate NPI
        if npi.len() != 10 {
            return Err(Error::InvalidNPI);
        }

        // Verify EMR system exists
        let systems: Map<String, EMRSystem> = env
            .storage()
            .persistent()
            .get(&EMR_SYSTEMS)
            .ok_or(Error::EMRSystemNotFound)?;

        if !systems.contains_key(emr_system_id.clone()) {
            return Err(Error::InvalidEMRSystem);
        }

        let mut onboardings: Map<String, ProviderOnboarding> = env
            .storage()
            .persistent()
            .get(&PROVIDER_ONBOARDING)
            .unwrap_or(Map::new(&env));

        if onboardings.contains_key(onboarding_id.clone()) {
            return Err(Error::OnboardingAlreadyExists);
        }

        let onboarding = ProviderOnboarding {
            onboarding_id: onboarding_id.clone(),
            provider_id,
            provider_name,
            provider_email,
            facility_name,
            npi,
            emr_system_id,
            status: IntegrationStatus::Pending,
            created_at: env.ledger().timestamp(),
            completed_at: 0,
            verification_document_hash: BytesN::from_array(&env, &[0u8; 32]),
            compliance_checklist,
            notes: String::from_str(&env, ""),
        };

        onboardings.set(onboarding_id, onboarding);
        env.storage()
            .persistent()
            .set(&PROVIDER_ONBOARDING, &onboardings);

        Ok(true)
    }

    /// Complete provider onboarding with verification
    pub fn complete_onboarding(
        env: Env,
        admin: Address,
        onboarding_id: String,
        verification_id: String,
        license_number: String,
        license_state: String,
        license_expiration: String,
        board_certifications: Vec<String>,
        malpractice_insurance: String,
        background_check_id: String,
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

        // Validate license hasn't expired
        // (In production, would parse and validate the expiration date)
        if license_expiration.is_empty() {
            return Err(Error::InvalidLicenseNumber);
        }

        let mut onboardings: Map<String, ProviderOnboarding> = env
            .storage()
            .persistent()
            .get(&PROVIDER_ONBOARDING)
            .ok_or(Error::OnboardingNotFound)?;

        let mut onboarding = onboardings
            .get(onboarding_id.clone())
            .ok_or(Error::OnboardingNotFound)?;

        // Update onboarding status
        onboarding.status = IntegrationStatus::Completed;
        onboarding.completed_at = env.ledger().timestamp();

        onboardings.set(onboarding_id, onboarding.clone());
        env.storage()
            .persistent()
            .set(&PROVIDER_ONBOARDING, &onboardings);

        // Create verification record
        let verification = ProviderVerification {
            verification_id: verification_id.clone(),
            provider_id: onboarding.provider_id,
            verified_by: admin,
            verification_timestamp: env.ledger().timestamp(),
            license_number,
            license_state,
            license_expiration,
            board_certification: board_certifications,
            malpractice_insurance,
            background_check_id,
            verification_status: String::from_str(&env, "approved"),
        };

        let mut verifications: Map<String, ProviderVerification> = env
            .storage()
            .persistent()
            .get(&PROVIDER_VERIFICATION)
            .unwrap_or(Map::new(&env));

        verifications.set(verification_id, verification);
        env.storage()
            .persistent()
            .set(&PROVIDER_VERIFICATION, &verifications);

        Ok(true)
    }

    /// Get provider onboarding status
    pub fn get_onboarding_status(
        env: Env,
        onboarding_id: String,
    ) -> Result<ProviderOnboarding, Error> {
        let onboardings: Map<String, ProviderOnboarding> = env
            .storage()
            .persistent()
            .get(&PROVIDER_ONBOARDING)
            .ok_or(Error::OnboardingNotFound)?;

        onboardings
            .get(onboarding_id)
            .ok_or(Error::OnboardingNotFound)
    }

    /// Get provider verification
    pub fn get_provider_verification(
        env: Env,
        verification_id: String,
    ) -> Result<ProviderVerification, Error> {
        let verifications: Map<String, ProviderVerification> = env
            .storage()
            .persistent()
            .get(&PROVIDER_VERIFICATION)
            .ok_or(Error::VerificationNotFound)?;

        verifications
            .get(verification_id)
            .ok_or(Error::VerificationNotFound)
    }

    /// Register a healthcare network node
    pub fn register_network_node(
        env: Env,
        admin: Address,
        node: NetworkNode,
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

        let mut nodes: Map<String, NetworkNode> = env
            .storage()
            .persistent()
            .get(&NETWORK_NODES)
            .unwrap_or(Map::new(&env));

        nodes.set(node.node_id.clone(), node);
        env.storage().persistent().set(&NETWORK_NODES, &nodes);

        Ok(true)
    }

    /// Get network node details
    pub fn get_network_node(env: Env, node_id: String) -> Result<NetworkNode, Error> {
        let nodes: Map<String, NetworkNode> = env
            .storage()
            .persistent()
            .get(&NETWORK_NODES)
            .ok_or(Error::NetworkNodeNotFound)?;

        nodes.get(node_id).ok_or(Error::NetworkNodeNotFound)
    }

    /// Register interoperability agreement between providers
    pub fn register_interop_agreement(
        env: Env,
        admin: Address,
        agreement: InteroperabilityAgreement,
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

        let mut agreements: Map<String, InteroperabilityAgreement> = env
            .storage()
            .persistent()
            .get(&INTEROP_AGREEMENTS)
            .unwrap_or(Map::new(&env));

        agreements.set(agreement.agreement_id.clone(), agreement);
        env.storage()
            .persistent()
            .set(&INTEROP_AGREEMENTS, &agreements);

        Ok(true)
    }

    /// Get interoperability agreement
    pub fn get_interop_agreement(
        env: Env,
        agreement_id: String,
    ) -> Result<InteroperabilityAgreement, Error> {
        let agreements: Map<String, InteroperabilityAgreement> = env
            .storage()
            .persistent()
            .get(&INTEROP_AGREEMENTS)
            .ok_or(Error::AgreementNotFound)?;

        agreements.get(agreement_id).ok_or(Error::AgreementNotFound)
    }

    /// Record interoperability test results
    pub fn record_interop_test(
        env: Env,
        tester: Address,
        test: InteroperabilityTest,
    ) -> Result<bool, Error> {
        tester.require_auth();

        if env.storage().persistent().get(&PAUSED).unwrap_or(false) {
            return Err(Error::ContractPaused);
        }

        // Validate test type
        let valid_types = vec![
            &env,
            String::from_str(&env, "data-exchange"),
            String::from_str(&env, "api-connectivity"),
            String::from_str(&env, "format-conversion"),
            String::from_str(&env, "performance"),
        ];

        if !valid_types.contains(&test.test_type) {
            return Err(Error::InvalidTestType);
        }

        // Validate success rate
        if test.success_rate > 100 {
            return Err(Error::InvalidStatus);
        }

        let mut tests: Map<String, InteroperabilityTest> = env
            .storage()
            .persistent()
            .get(&INTEROP_TESTS)
            .unwrap_or(Map::new(&env));

        tests.set(test.test_id.clone(), test);
        env.storage().persistent().set(&INTEROP_TESTS, &tests);

        Ok(true)
    }

    /// Get interoperability test results
    pub fn get_interop_test(env: Env, test_id: String) -> Result<InteroperabilityTest, Error> {
        let tests: Map<String, InteroperabilityTest> = env
            .storage()
            .persistent()
            .get(&INTEROP_TESTS)
            .ok_or(Error::TestNotFound)?;

        tests.get(test_id).ok_or(Error::TestNotFound)
    }

    /// Pause contract operations
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
