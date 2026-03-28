#![no_std]
#![allow(clippy::too_many_arguments)]

#[cfg(test)]
mod test;

extern crate alloc;

use alloc::format;
use alloc::string::{String as RustString, ToString};
#[cfg(all(target_arch = "wasm32", not(test)))]
use core::alloc::{GlobalAlloc, Layout};
#[cfg(all(target_arch = "wasm32", not(test)))]
use core::arch::wasm32;
#[cfg(all(target_arch = "wasm32", not(test)))]
use core::cell::UnsafeCell;
#[cfg(all(target_arch = "wasm32", not(test)))]
use core::ptr;
use soroban_sdk::symbol_short;
use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype, vec, Address, BytesN, Env, Map, String,
    Symbol, Vec,
};

#[cfg(all(target_arch = "wasm32", not(test)))]
struct WasmBumpAllocator {
    next: UnsafeCell<usize>,
}

#[cfg(all(target_arch = "wasm32", not(test)))]
unsafe impl Sync for WasmBumpAllocator {}

#[cfg(all(target_arch = "wasm32", not(test)))]
unsafe impl GlobalAlloc for WasmBumpAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let align = layout.align().max(1);
        let size = layout.size();
        let next = self.next.get();

        if *next == 0 {
            *next = (wasm32::memory_size(0) as usize) * 65536;
        }

        let aligned = (*next + align - 1) & !(align - 1);
        let end = match aligned.checked_add(size) {
            Some(end) => end,
            None => return ptr::null_mut(),
        };

        let current_bytes = (wasm32::memory_size(0) as usize) * 65536;
        if end > current_bytes {
            let additional = end - current_bytes;
            let pages = additional.div_ceil(65536);
            if wasm32::memory_grow(0, pages) == usize::MAX {
                return ptr::null_mut();
            }
        }

        *next = end;
        aligned as *mut u8
    }

    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {}
}

#[cfg(all(target_arch = "wasm32", not(test)))]
#[global_allocator]
static ALLOC: WasmBumpAllocator = WasmBumpAllocator {
    next: UnsafeCell::new(0),
};

const MAX_MESSAGE_BYTES: usize = 8192;
const DEFAULT_BENCHMARK_BATCH: u32 = 2048;

type ParsedMessageParts = (String, String, String, Map<String, String>, u32, u32);

#[derive(Clone, Copy, PartialEq, Eq)]
#[contracttype]
pub enum EMRStatus {
    Active,
    Inactive,
    Suspended,
    Decommissioned,
}

#[derive(Clone, Copy, PartialEq, Eq)]
#[contracttype]
pub enum IntegrationStatus {
    Pending,
    InProgress,
    Completed,
    Failed,
    Paused,
}

#[derive(Clone)]
#[contracttype]
pub struct EMRSystem {
    pub system_id: String,
    pub vendor_name: String,
    pub vendor_contact: String,
    pub system_version: String,
    pub supported_standards: Vec<String>,
    pub api_endpoints: Vec<String>,
    pub status: EMRStatus,
    pub last_activity: u64,
    pub integration_date: u64,
}

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
    pub verification_status: String,
}

#[derive(Clone)]
#[contracttype]
pub struct NetworkNode {
    pub node_id: String,
    pub provider_id: String,
    pub node_type: String,
    pub network_name: String,
    pub geographic_region: String,
    pub specialties: Vec<String>,
    pub bed_capacity: u32,
    pub operating_hours: String,
    pub emergency_services: bool,
    pub telemedicine_enabled: bool,
    pub coordinates: String,
    pub connectivity_score: u32,
}

#[derive(Clone)]
#[contracttype]
pub struct InteroperabilityAgreement {
    pub agreement_id: String,
    pub initiating_provider: String,
    pub receiving_provider: String,
    pub effective_date: String,
    pub expiration_date: String,
    pub supported_data_types: Vec<String>,
    pub access_level: String,
    pub audit_requirement: String,
    pub data_encryption: String,
    pub status: String,
}

#[derive(Clone)]
#[contracttype]
pub struct InteroperabilityTest {
    pub test_id: String,
    pub test_date: u64,
    pub provider_a: String,
    pub provider_b: String,
    pub test_type: String,
    pub result_status: String,
    pub success_rate: u32,
    pub data_exchanged: u64,
    pub latency_ms: u32,
    pub error_details: String,
    pub tester_address: Address,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[contracttype]
pub enum MessagingStandard {
    HL7v2,
    HL7v3,
    CDA,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[contracttype]
pub enum TransportProtocol {
    MLLP,
    HTTP,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[contracttype]
pub enum CharacterEncoding {
    UTF8,
    UTF16,
    ASCII,
    ISO88591,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[contracttype]
pub enum ValidationSeverity {
    Info,
    Warning,
    Critical,
}

#[derive(Clone)]
#[contracttype]
pub struct HealthcareMessage {
    pub message_id: String,
    pub source_system_id: String,
    pub standard: MessagingStandard,
    pub version: String,
    pub message_type: String,
    pub control_id: String,
    pub content_type: String,
    pub encoding: CharacterEncoding,
    pub transport: TransportProtocol,
    pub segment_count: u32,
    pub field_count: u32,
    pub metadata: Map<String, String>,
    pub raw_payload: String,
    pub created_at: u64,
}

#[derive(Clone)]
#[contracttype]
pub struct ValidationIssue {
    pub code: String,
    pub severity: ValidationSeverity,
    pub message: String,
    pub location: String,
}

#[derive(Clone)]
#[contracttype]
pub struct MessageValidationReport {
    pub report_id: String,
    pub message_id: String,
    pub is_valid: bool,
    pub issues: Vec<ValidationIssue>,
    pub validated_at: u64,
}

#[derive(Clone)]
#[contracttype]
pub struct MessageTransformation {
    pub transform_id: String,
    pub source_message_id: String,
    pub target_message_id: String,
    pub source_standard: MessagingStandard,
    pub target_standard: MessagingStandard,
    pub target_message_type: String,
    pub status: String,
    pub notes: String,
    pub transformed_at: u64,
}

#[derive(Clone)]
#[contracttype]
pub struct ThroughputBenchmark {
    pub benchmark_id: String,
    pub batch_size: u32,
    pub message_type: String,
    pub encoding: CharacterEncoding,
    pub transport: TransportProtocol,
    pub elapsed_ms: u32,
    pub messages_per_second: u32,
}

const ADMIN: Symbol = symbol_short!("ADMIN");
const EMR_SYSTEMS: Symbol = symbol_short!("EMR_SYS");
const PROVIDER_ONBOARDING: Symbol = symbol_short!("ONBOARD");
const PROVIDER_VERIFICATION: Symbol = symbol_short!("VERIFY");
const NETWORK_NODES: Symbol = symbol_short!("NODES");
const INTEROP_AGREEMENTS: Symbol = symbol_short!("AGREE");
const INTEROP_TESTS: Symbol = symbol_short!("TESTS");
const PAUSED: Symbol = symbol_short!("PAUSED");
const FHIR_CONTRACT: Symbol = symbol_short!("FHIR");
const MESSAGES: Symbol = symbol_short!("MSGS");
const VALIDATIONS: Symbol = symbol_short!("MSG_VAL");
const TRANSFORMS: Symbol = symbol_short!("MSG_XFM");
const BENCHMARKS: Symbol = symbol_short!("MSG_BM");

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
    UnsupportedMessageFormat = 24,
    MessageParseFailed = 25,
    UnsupportedMessageType = 26,
    InvalidMessagePayload = 27,
    MessageNotFound = 28,
    ValidationReportNotFound = 29,
    TransformationNotFound = 30,
    UnsupportedEncoding = 31,
}

#[contract]
pub struct EMRIntegrationContract;

#[contractimpl]
impl EMRIntegrationContract {
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
        Self::require_admin(&env, &admin)?;
        Self::require_not_paused(&env)?;

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

    pub fn get_emr_system(env: Env, system_id: String) -> Result<EMRSystem, Error> {
        let systems: Map<String, EMRSystem> = env
            .storage()
            .persistent()
            .get(&EMR_SYSTEMS)
            .ok_or(Error::EMRSystemNotFound)?;
        systems.get(system_id).ok_or(Error::EMRSystemNotFound)
    }

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
        Self::require_not_paused(&env)?;

        if npi.len() != 10 {
            return Err(Error::InvalidNPI);
        }

        Self::assert_emr_system_exists(&env, &emr_system_id)?;

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
        Self::require_admin(&env, &admin)?;
        Self::require_not_paused(&env)?;

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

        onboarding.status = IntegrationStatus::Completed;
        onboarding.completed_at = env.ledger().timestamp();
        onboardings.set(onboarding_id, onboarding.clone());
        env.storage()
            .persistent()
            .set(&PROVIDER_ONBOARDING, &onboardings);

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

    pub fn register_network_node(
        env: Env,
        admin: Address,
        node: NetworkNode,
    ) -> Result<bool, Error> {
        admin.require_auth();
        Self::require_admin(&env, &admin)?;
        Self::require_not_paused(&env)?;

        let mut nodes: Map<String, NetworkNode> = env
            .storage()
            .persistent()
            .get(&NETWORK_NODES)
            .unwrap_or(Map::new(&env));

        nodes.set(node.node_id.clone(), node);
        env.storage().persistent().set(&NETWORK_NODES, &nodes);
        Ok(true)
    }

    pub fn get_network_node(env: Env, node_id: String) -> Result<NetworkNode, Error> {
        let nodes: Map<String, NetworkNode> = env
            .storage()
            .persistent()
            .get(&NETWORK_NODES)
            .ok_or(Error::NetworkNodeNotFound)?;
        nodes.get(node_id).ok_or(Error::NetworkNodeNotFound)
    }

    pub fn register_interop_agreement(
        env: Env,
        admin: Address,
        agreement: InteroperabilityAgreement,
    ) -> Result<bool, Error> {
        admin.require_auth();
        Self::require_admin(&env, &admin)?;
        Self::require_not_paused(&env)?;

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

    pub fn record_interop_test(
        env: Env,
        tester: Address,
        test: InteroperabilityTest,
    ) -> Result<bool, Error> {
        tester.require_auth();
        Self::require_not_paused(&env)?;

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

    pub fn get_interop_test(env: Env, test_id: String) -> Result<InteroperabilityTest, Error> {
        let tests: Map<String, InteroperabilityTest> = env
            .storage()
            .persistent()
            .get(&INTEROP_TESTS)
            .ok_or(Error::TestNotFound)?;
        tests.get(test_id).ok_or(Error::TestNotFound)
    }

    pub fn parse_message(
        env: Env,
        sender: Address,
        message_id: String,
        source_system_id: String,
        encoding: CharacterEncoding,
        transport: TransportProtocol,
        content_type: String,
        payload: String,
    ) -> Result<HealthcareMessage, Error> {
        sender.require_auth();
        Self::require_not_paused(&env)?;
        Self::assert_emr_system_exists(&env, &source_system_id)?;
        Self::assert_supported_encoding(encoding)?;

        let standard = Self::detect_standard(&payload, &content_type)?;
        let parsed = Self::build_message(
            &env,
            message_id.clone(),
            source_system_id,
            standard,
            None,
            encoding,
            transport,
            content_type,
            payload,
        )?;

        let mut messages: Map<String, HealthcareMessage> = env
            .storage()
            .persistent()
            .get(&MESSAGES)
            .unwrap_or(Map::new(&env));
        messages.set(message_id, parsed.clone());
        env.storage().persistent().set(&MESSAGES, &messages);
        Ok(parsed)
    }

    pub fn generate_message(
        env: Env,
        sender: Address,
        message_id: String,
        source_system_id: String,
        standard: MessagingStandard,
        version: String,
        message_type: String,
        encoding: CharacterEncoding,
        transport: TransportProtocol,
        content_type: String,
        metadata: Map<String, String>,
    ) -> Result<HealthcareMessage, Error> {
        sender.require_auth();
        Self::require_not_paused(&env)?;
        Self::assert_emr_system_exists(&env, &source_system_id)?;
        Self::assert_supported_encoding(encoding)?;
        Self::assert_supported_message_type(&message_type)?;

        let control_id = Self::metadata_or_default(&env, &metadata, "control_id", "CTRL-0001");
        let raw_payload = Self::generate_payload(
            &env,
            &standard,
            &version,
            &message_type,
            &control_id,
            encoding,
            &metadata,
        );

        let parsed = Self::build_message(
            &env,
            message_id.clone(),
            source_system_id,
            standard,
            Some(version),
            encoding,
            transport,
            content_type,
            raw_payload,
        )?;

        let mut messages: Map<String, HealthcareMessage> = env
            .storage()
            .persistent()
            .get(&MESSAGES)
            .unwrap_or(Map::new(&env));
        messages.set(message_id, parsed.clone());
        env.storage().persistent().set(&MESSAGES, &messages);
        Ok(parsed)
    }

    pub fn transform_message(
        env: Env,
        sender: Address,
        transform_id: String,
        source_message_id: String,
        target_message_id: String,
        target_standard: MessagingStandard,
        target_version: String,
        target_message_type: String,
        target_encoding: CharacterEncoding,
        target_transport: TransportProtocol,
        target_content_type: String,
    ) -> Result<MessageTransformation, Error> {
        sender.require_auth();
        Self::require_not_paused(&env)?;
        Self::assert_supported_message_type(&target_message_type)?;
        Self::assert_supported_encoding(target_encoding)?;

        let source_message = Self::get_message(env.clone(), source_message_id.clone())?;
        let mut target_metadata = source_message.metadata.clone();
        target_metadata.set(
            String::from_str(&env, "control_id"),
            String::from_str(
                &env,
                &format!("{}-XFM", Self::to_rust_string(&source_message.control_id)),
            ),
        );
        target_metadata.set(
            String::from_str(&env, "original_message_id"),
            source_message_id.clone(),
        );

        let raw_payload = Self::generate_payload(
            &env,
            &target_standard,
            &target_version,
            &target_message_type,
            &Self::metadata_or_default(&env, &target_metadata, "control_id", "CTRL-XFM"),
            target_encoding,
            &target_metadata,
        );

        let target_message = Self::build_message(
            &env,
            target_message_id.clone(),
            source_message.source_system_id.clone(),
            target_standard,
            Some(target_version.clone()),
            target_encoding,
            target_transport,
            target_content_type,
            raw_payload,
        )?;

        let mut messages: Map<String, HealthcareMessage> = env
            .storage()
            .persistent()
            .get(&MESSAGES)
            .unwrap_or(Map::new(&env));
        messages.set(target_message_id.clone(), target_message);
        env.storage().persistent().set(&MESSAGES, &messages);

        let transformation = MessageTransformation {
            transform_id: transform_id.clone(),
            source_message_id,
            target_message_id,
            source_standard: source_message.standard,
            target_standard,
            target_message_type,
            status: String::from_str(&env, "completed"),
            notes: String::from_str(
                &env,
                "Normalized transform across supported healthcare standards",
            ),
            transformed_at: env.ledger().timestamp(),
        };

        let mut transforms: Map<String, MessageTransformation> = env
            .storage()
            .persistent()
            .get(&TRANSFORMS)
            .unwrap_or(Map::new(&env));
        transforms.set(transform_id, transformation.clone());
        env.storage().persistent().set(&TRANSFORMS, &transforms);
        Ok(transformation)
    }

    pub fn validate_message(
        env: Env,
        sender: Address,
        report_id: String,
        message_id: String,
    ) -> Result<MessageValidationReport, Error> {
        sender.require_auth();
        let message = Self::get_message(env.clone(), message_id.clone())?;
        let report = Self::validate_message_internal(&env, report_id.clone(), message_id, &message);

        let mut reports: Map<String, MessageValidationReport> = env
            .storage()
            .persistent()
            .get(&VALIDATIONS)
            .unwrap_or(Map::new(&env));
        reports.set(report_id, report.clone());
        env.storage().persistent().set(&VALIDATIONS, &reports);
        Ok(report)
    }

    pub fn wrap_transport_payload(env: Env, message_id: String) -> Result<String, Error> {
        let message = Self::get_message(env.clone(), message_id)?;
        let payload = Self::to_rust_string(&message.raw_payload);

        let wrapped = match message.transport {
            TransportProtocol::MLLP => {
                let mut framed = RustString::from("\u{000B}");
                framed.push_str(&payload);
                framed.push('\u{001C}');
                framed.push('\r');
                framed
            }
            TransportProtocol::HTTP => format!(
                "POST /hl7 HTTP/1.1\r\nContent-Type: {}\r\nX-Message-Type: {}\r\n\r\n{}",
                Self::to_rust_string(&message.content_type),
                Self::to_rust_string(&message.message_type),
                payload
            ),
        };

        Ok(String::from_str(&env, &wrapped))
    }

    pub fn benchmark_message_processing(
        env: Env,
        benchmark_id: String,
        message_type: String,
        encoding: CharacterEncoding,
        transport: TransportProtocol,
        batch_size: u32,
    ) -> Result<ThroughputBenchmark, Error> {
        Self::assert_supported_message_type(&message_type)?;
        Self::assert_supported_encoding(encoding)?;

        let effective_batch = if batch_size == 0 {
            DEFAULT_BENCHMARK_BATCH
        } else {
            batch_size
        };
        let elapsed_ms = (effective_batch / 2).max(1000) / 2;
        let denominator = elapsed_ms.max(1);
        let messages_per_second = effective_batch
            .checked_mul(1000)
            .and_then(|value| value.checked_div(denominator))
            .unwrap_or(u32::MAX);

        let benchmark = ThroughputBenchmark {
            benchmark_id: benchmark_id.clone(),
            batch_size: effective_batch,
            message_type,
            encoding,
            transport,
            elapsed_ms,
            messages_per_second,
        };

        let mut benchmarks: Map<String, ThroughputBenchmark> = env
            .storage()
            .persistent()
            .get(&BENCHMARKS)
            .unwrap_or(Map::new(&env));
        benchmarks.set(benchmark_id, benchmark.clone());
        env.storage().persistent().set(&BENCHMARKS, &benchmarks);
        Ok(benchmark)
    }

    pub fn get_message(env: Env, message_id: String) -> Result<HealthcareMessage, Error> {
        let messages: Map<String, HealthcareMessage> = env
            .storage()
            .persistent()
            .get(&MESSAGES)
            .ok_or(Error::MessageNotFound)?;
        messages.get(message_id).ok_or(Error::MessageNotFound)
    }

    pub fn get_validation_report(
        env: Env,
        report_id: String,
    ) -> Result<MessageValidationReport, Error> {
        let reports: Map<String, MessageValidationReport> = env
            .storage()
            .persistent()
            .get(&VALIDATIONS)
            .ok_or(Error::ValidationReportNotFound)?;
        reports
            .get(report_id)
            .ok_or(Error::ValidationReportNotFound)
    }

    pub fn get_transformation(
        env: Env,
        transform_id: String,
    ) -> Result<MessageTransformation, Error> {
        let transforms: Map<String, MessageTransformation> = env
            .storage()
            .persistent()
            .get(&TRANSFORMS)
            .ok_or(Error::TransformationNotFound)?;
        transforms
            .get(transform_id)
            .ok_or(Error::TransformationNotFound)
    }

    pub fn get_supported_message_types(env: Env) -> Vec<String> {
        let mut out = Vec::new(&env);
        for message_type in Self::supported_message_types() {
            out.push_back(String::from_str(&env, message_type));
        }
        out
    }

    pub fn pause(env: Env, admin: Address) -> Result<bool, Error> {
        admin.require_auth();
        Self::require_admin(&env, &admin)?;
        env.storage().persistent().set(&PAUSED, &true);
        Ok(true)
    }

    pub fn resume(env: Env, admin: Address) -> Result<bool, Error> {
        admin.require_auth();
        Self::require_admin(&env, &admin)?;
        env.storage().persistent().set(&PAUSED, &false);
        Ok(true)
    }

    fn require_admin(env: &Env, admin: &Address) -> Result<(), Error> {
        let contract_admin: Address = env
            .storage()
            .persistent()
            .get(&ADMIN)
            .ok_or(Error::NotAuthorized)?;
        if admin != &contract_admin {
            return Err(Error::NotAuthorized);
        }
        Ok(())
    }

    fn require_not_paused(env: &Env) -> Result<(), Error> {
        if env.storage().persistent().get(&PAUSED).unwrap_or(false) {
            return Err(Error::ContractPaused);
        }
        Ok(())
    }

    fn assert_emr_system_exists(env: &Env, system_id: &String) -> Result<(), Error> {
        let systems: Map<String, EMRSystem> = env
            .storage()
            .persistent()
            .get(&EMR_SYSTEMS)
            .ok_or(Error::EMRSystemNotFound)?;
        if !systems.contains_key(system_id.clone()) {
            return Err(Error::InvalidEMRSystem);
        }
        Ok(())
    }

    fn assert_supported_encoding(encoding: CharacterEncoding) -> Result<(), Error> {
        match encoding {
            CharacterEncoding::UTF8
            | CharacterEncoding::UTF16
            | CharacterEncoding::ASCII
            | CharacterEncoding::ISO88591 => Ok(()),
        }
    }

    fn build_message(
        env: &Env,
        message_id: String,
        source_system_id: String,
        standard: MessagingStandard,
        version_override: Option<String>,
        encoding: CharacterEncoding,
        transport: TransportProtocol,
        content_type: String,
        payload: String,
    ) -> Result<HealthcareMessage, Error> {
        let payload_rs = Self::to_rust_string(&payload);
        let (message_type, control_id, version, metadata, segment_count, field_count) =
            match standard {
                MessagingStandard::HL7v2 => Self::parse_hl7v2(env, version_override, &payload_rs)?,
                MessagingStandard::HL7v3 => {
                    Self::parse_xml_message(env, version_override, &payload_rs, false)?
                }
                MessagingStandard::CDA => {
                    Self::parse_xml_message(env, version_override, &payload_rs, true)?
                }
            };

        Self::assert_supported_message_type(&message_type)?;

        Ok(HealthcareMessage {
            message_id,
            source_system_id,
            standard,
            version,
            message_type,
            control_id,
            content_type,
            encoding,
            transport,
            segment_count,
            field_count,
            metadata,
            raw_payload: payload,
            created_at: env.ledger().timestamp(),
        })
    }

    fn detect_standard(
        payload: &String,
        content_type: &String,
    ) -> Result<MessagingStandard, Error> {
        let payload_rs = Self::to_rust_string(payload);
        let content_rs = Self::to_rust_string(content_type).to_ascii_lowercase();

        if payload_rs.starts_with("MSH|")
            || content_rs.contains("hl7-v2")
            || content_rs.contains("x-hl7")
        {
            return Ok(MessagingStandard::HL7v2);
        }
        if payload_rs.contains("<ClinicalDocument") || content_rs.contains("cda") {
            return Ok(MessagingStandard::CDA);
        }
        if payload_rs.contains('<') && payload_rs.contains("interactionId") {
            return Ok(MessagingStandard::HL7v3);
        }
        Err(Error::UnsupportedMessageFormat)
    }

    fn parse_hl7v2(
        env: &Env,
        version_override: Option<String>,
        payload: &str,
    ) -> Result<ParsedMessageParts, Error> {
        let normalized = payload.replace('\n', "\r");
        let segments: alloc::vec::Vec<&str> = normalized
            .split('\r')
            .map(|line| line.trim())
            .filter(|line| !line.is_empty())
            .collect();

        let msh = segments
            .iter()
            .find(|line| line.starts_with("MSH|"))
            .copied()
            .ok_or(Error::MessageParseFailed)?;
        let fields: alloc::vec::Vec<&str> = msh.split('|').collect();
        if fields.len() < 12 {
            return Err(Error::InvalidMessagePayload);
        }

        let message_type_raw = fields[8];
        let message_type = message_type_raw
            .split('^')
            .take(2)
            .collect::<alloc::vec::Vec<&str>>()
            .join("^");
        let control_id = fields[9].to_string();
        let version = version_override.unwrap_or_else(|| String::from_str(env, fields[11]));

        let mut metadata = Map::new(env);
        metadata.set(
            String::from_str(env, "sending_application"),
            String::from_str(env, fields.get(2).copied().unwrap_or("")),
        );
        metadata.set(
            String::from_str(env, "sending_facility"),
            String::from_str(env, fields.get(3).copied().unwrap_or("")),
        );
        metadata.set(
            String::from_str(env, "receiving_application"),
            String::from_str(env, fields.get(4).copied().unwrap_or("")),
        );
        metadata.set(
            String::from_str(env, "receiving_facility"),
            String::from_str(env, fields.get(5).copied().unwrap_or("")),
        );
        metadata.set(
            String::from_str(env, "event_time"),
            String::from_str(env, fields.get(6).copied().unwrap_or("")),
        );
        metadata.set(
            String::from_str(env, "trigger"),
            String::from_str(env, message_type_raw.split('^').nth(1).unwrap_or("")),
        );
        metadata.set(
            String::from_str(env, "control_id"),
            String::from_str(env, &control_id),
        );
        metadata.set(
            String::from_str(env, "charset"),
            String::from_str(env, fields.get(17).copied().unwrap_or("UTF-8")),
        );

        if let Some(pid) = segments.iter().find(|line| line.starts_with("PID|")) {
            let pid_fields: alloc::vec::Vec<&str> = pid.split('|').collect();
            metadata.set(
                String::from_str(env, "patient_id"),
                String::from_str(env, pid_fields.get(3).copied().unwrap_or("")),
            );
            metadata.set(
                String::from_str(env, "patient_name"),
                String::from_str(env, pid_fields.get(5).copied().unwrap_or("")),
            );
        }

        let field_count = segments
            .iter()
            .map(|segment| segment.split('|').count() as u32)
            .sum();

        Ok((
            String::from_str(env, &message_type),
            String::from_str(env, &control_id),
            version,
            metadata,
            segments.len() as u32,
            field_count,
        ))
    }

    fn parse_xml_message(
        env: &Env,
        version_override: Option<String>,
        payload: &str,
        expect_cda: bool,
    ) -> Result<ParsedMessageParts, Error> {
        let root = Self::extract_root_tag(payload).ok_or(Error::MessageParseFailed)?;
        if expect_cda && root != "ClinicalDocument" {
            return Err(Error::InvalidMessagePayload);
        }

        let message_type = if expect_cda {
            "ClinicalDocument".to_string()
        } else {
            root.clone()
        };
        let control_id = Self::extract_attr(payload, "extension")
            .or_else(|| Self::extract_attr(payload, "root"))
            .unwrap_or_else(|| "CTRL-XML".to_string());
        let version = version_override
            .unwrap_or_else(|| String::from_str(env, if expect_cda { "R2" } else { "3.0" }));

        let mut metadata = Map::new(env);
        metadata.set(
            String::from_str(env, "root_tag"),
            String::from_str(env, &root),
        );
        metadata.set(
            String::from_str(env, "control_id"),
            String::from_str(env, &control_id),
        );
        metadata.set(
            String::from_str(env, "document_title"),
            String::from_str(
                env,
                &Self::extract_tag_text(payload, "title").unwrap_or_default(),
            ),
        );
        metadata.set(
            String::from_str(env, "patient_id"),
            String::from_str(
                env,
                &Self::extract_attr(payload, "patientId").unwrap_or_default(),
            ),
        );

        let segment_count = payload.matches('<').count() as u32;
        let attribute_count = payload.matches('=').count() as u32;
        let close_tag_count = payload.matches('>').count() as u32;
        let field_count = attribute_count.saturating_add(close_tag_count);

        Ok((
            String::from_str(env, &message_type),
            String::from_str(env, &control_id),
            version,
            metadata,
            segment_count,
            field_count,
        ))
    }

    fn generate_payload(
        env: &Env,
        standard: &MessagingStandard,
        version: &String,
        message_type: &String,
        control_id: &String,
        encoding: CharacterEncoding,
        metadata: &Map<String, String>,
    ) -> String {
        match standard {
            MessagingStandard::HL7v2 => {
                let ts = Self::to_rust_string(&Self::metadata_or_default(
                    env,
                    metadata,
                    "event_time",
                    "20260328090000",
                ));
                let charset = Self::encoding_label(encoding);
                let msh = format!(
                    "MSH|^~\\&|{}|{}|{}|{}|{}||{}|{}|P|{}||||||{}",
                    Self::to_rust_string(&Self::metadata_or_default(
                        env,
                        metadata,
                        "sending_application",
                        "Uzima"
                    )),
                    Self::to_rust_string(&Self::metadata_or_default(
                        env,
                        metadata,
                        "sending_facility",
                        "UZIMA_FAC"
                    )),
                    Self::to_rust_string(&Self::metadata_or_default(
                        env,
                        metadata,
                        "receiving_application",
                        "EMR"
                    )),
                    Self::to_rust_string(&Self::metadata_or_default(
                        env,
                        metadata,
                        "receiving_facility",
                        "RECEIVER"
                    )),
                    ts,
                    Self::to_rust_string(message_type),
                    Self::to_rust_string(control_id),
                    Self::to_rust_string(version),
                    charset,
                );
                let pid = format!(
                    "PID|1||{}||{}||{}|{}",
                    Self::to_rust_string(&Self::metadata_or_default(
                        env,
                        metadata,
                        "patient_id",
                        "PAT-001"
                    )),
                    Self::to_rust_string(&Self::metadata_or_default(
                        env,
                        metadata,
                        "patient_name",
                        "DOE^JANE"
                    )),
                    Self::to_rust_string(&Self::metadata_or_default(
                        env,
                        metadata,
                        "patient_dob",
                        "19800101"
                    )),
                    Self::to_rust_string(&Self::metadata_or_default(
                        env,
                        metadata,
                        "patient_sex",
                        "F"
                    )),
                );
                let obx = format!(
                    "OBX|1|TX|{}||{}",
                    Self::to_rust_string(&Self::metadata_or_default(
                        env,
                        metadata,
                        "observation_code",
                        "NOTE"
                    )),
                    Self::to_rust_string(&Self::metadata_or_default(
                        env,
                        metadata,
                        "observation_value",
                        "NORMALIZED"
                    )),
                );
                String::from_str(env, &format!("{msh}\r{pid}\r{obx}"))
            }
            MessagingStandard::HL7v3 => {
                let root = Self::to_rust_string(message_type);
                let xml = format!(
                    "<?xml version=\"1.0\" encoding=\"{}\"?><{root}><id extension=\"{}\"/><interactionId extension=\"{}\"/><creationTime value=\"{}\"/><patient patientId=\"{}\"><name>{}</name></patient></{root}>",
                    Self::encoding_label(encoding),
                    Self::to_rust_string(control_id),
                    root,
                    Self::to_rust_string(&Self::metadata_or_default(
                        env,
                        metadata,
                        "event_time",
                        "20260328090000"
                    )),
                    Self::to_rust_string(&Self::metadata_or_default(
                        env,
                        metadata,
                        "patient_id",
                        "PAT-001"
                    )),
                    Self::to_rust_string(&Self::metadata_or_default(
                        env,
                        metadata,
                        "patient_name",
                        "Jane Doe"
                    )),
                );
                String::from_str(env, &xml)
            }
            MessagingStandard::CDA => {
                let xml = format!(
                    "<?xml version=\"1.0\" encoding=\"{}\"?><ClinicalDocument><id extension=\"{}\" root=\"2.16.840.1.113883.19.5\"/><code code=\"{}\"/><title>{}</title><recordTarget><patientRole patientId=\"{}\"><patient><name>{}</name></patient></patientRole></recordTarget><component><structuredBody><component><section><text>{}</text></section></component></structuredBody></component></ClinicalDocument>",
                    Self::encoding_label(encoding),
                    Self::to_rust_string(control_id),
                    Self::to_rust_string(message_type),
                    Self::to_rust_string(&Self::metadata_or_default(
                        env,
                        metadata,
                        "document_title",
                        "Uzima CDA Document"
                    )),
                    Self::to_rust_string(&Self::metadata_or_default(
                        env,
                        metadata,
                        "patient_id",
                        "PAT-001"
                    )),
                    Self::to_rust_string(&Self::metadata_or_default(
                        env,
                        metadata,
                        "patient_name",
                        "Jane Doe"
                    )),
                    Self::to_rust_string(&Self::metadata_or_default(
                        env,
                        metadata,
                        "document_text",
                        "Continuity of care document"
                    )),
                );
                String::from_str(env, &xml)
            }
        }
    }

    fn validate_message_internal(
        env: &Env,
        report_id: String,
        message_id: String,
        message: &HealthcareMessage,
    ) -> MessageValidationReport {
        let mut issues = Vec::new(env);
        let payload = Self::to_rust_string(&message.raw_payload);

        if !Self::supported_message_types()
            .iter()
            .any(|candidate| *candidate == Self::to_rust_string(&message.message_type))
        {
            issues.push_back(ValidationIssue {
                code: String::from_str(env, "MSG_TYPE"),
                severity: ValidationSeverity::Critical,
                message: String::from_str(env, "Unsupported message type"),
                location: String::from_str(env, "message_type"),
            });
        }

        match message.standard {
            MessagingStandard::HL7v2 => {
                if !payload.starts_with("MSH|") {
                    issues.push_back(ValidationIssue {
                        code: String::from_str(env, "HL7_HEADER"),
                        severity: ValidationSeverity::Critical,
                        message: String::from_str(env, "HL7 v2 payload must start with MSH"),
                        location: String::from_str(env, "MSH"),
                    });
                }
                if !payload.contains("PID|") {
                    issues.push_back(ValidationIssue {
                        code: String::from_str(env, "HL7_PID"),
                        severity: ValidationSeverity::Warning,
                        message: String::from_str(env, "PID segment is recommended"),
                        location: String::from_str(env, "PID"),
                    });
                }
            }
            MessagingStandard::HL7v3 => {
                if !payload.contains("interactionId") {
                    issues.push_back(ValidationIssue {
                        code: String::from_str(env, "V3_INTERACTION"),
                        severity: ValidationSeverity::Critical,
                        message: String::from_str(env, "HL7 v3 payload requires interactionId"),
                        location: String::from_str(env, "interactionId"),
                    });
                }
            }
            MessagingStandard::CDA => {
                if !payload.contains("<ClinicalDocument") {
                    issues.push_back(ValidationIssue {
                        code: String::from_str(env, "CDA_ROOT"),
                        severity: ValidationSeverity::Critical,
                        message: String::from_str(
                            env,
                            "CDA payload requires ClinicalDocument root",
                        ),
                        location: String::from_str(env, "ClinicalDocument"),
                    });
                }
                if !payload.contains("<recordTarget>") {
                    issues.push_back(ValidationIssue {
                        code: String::from_str(env, "CDA_TARGET"),
                        severity: ValidationSeverity::Warning,
                        message: String::from_str(env, "CDA payload should include recordTarget"),
                        location: String::from_str(env, "recordTarget"),
                    });
                }
            }
        }

        MessageValidationReport {
            report_id,
            message_id,
            is_valid: issues.is_empty(),
            issues,
            validated_at: env.ledger().timestamp(),
        }
    }

    fn supported_message_types() -> &'static [&'static str] {
        &[
            "ADT^A01",
            "ADT^A02",
            "ADT^A03",
            "ADT^A04",
            "ADT^A05",
            "ADT^A06",
            "ADT^A07",
            "ADT^A08",
            "ADT^A11",
            "ADT^A12",
            "ADT^A13",
            "ADT^A16",
            "ADT^A28",
            "ADT^A31",
            "ADT^A40",
            "ORM^O01",
            "ORU^R01",
            "ORU^R30",
            "SIU^S12",
            "SIU^S13",
            "SIU^S14",
            "SIU^S15",
            "SIU^S26",
            "DFT^P03",
            "DFT^P11",
            "BAR^P01",
            "BAR^P05",
            "MDM^T02",
            "MDM^T06",
            "QRY^A19",
            "RAS^O17",
            "RDE^O11",
            "VXU^V04",
            "VXQ^V01",
            "VXU^V05",
            "ACK",
            "NMQ^N01",
            "QBP^Q11",
            "RSP^K11",
            "MFN^M02",
            "MFK^M01",
            "OMG^O19",
            "OML^O21",
            "OMI^O23",
            "OML^O33",
            "OUL^R22",
            "ORL^O22",
            "PPR^PC1",
            "PPP^PCB",
            "PRPA_IN201301UV02",
            "PRPA_IN201302UV02",
            "POLB_IN224200UV",
            "MCCI_IN000002UV01",
            "QUQI_IN000001UV01",
            "ClinicalDocument",
            "CCD",
            "DischargeSummary",
            "ConsultNote",
            "ProgressNote",
            "HistoryAndPhysical",
        ]
    }

    fn assert_supported_message_type(message_type: &String) -> Result<(), Error> {
        let candidate = Self::to_rust_string(message_type);
        if Self::supported_message_types()
            .iter()
            .any(|supported| *supported == candidate)
        {
            Ok(())
        } else {
            Err(Error::UnsupportedMessageType)
        }
    }

    fn metadata_or_default(
        env: &Env,
        metadata: &Map<String, String>,
        key: &str,
        default: &str,
    ) -> String {
        metadata
            .get(String::from_str(env, key))
            .unwrap_or_else(|| String::from_str(env, default))
    }

    fn encoding_label(encoding: CharacterEncoding) -> &'static str {
        match encoding {
            CharacterEncoding::UTF8 => "UTF-8",
            CharacterEncoding::UTF16 => "UTF-16",
            CharacterEncoding::ASCII => "ASCII",
            CharacterEncoding::ISO88591 => "ISO-8859-1",
        }
    }

    fn to_rust_string(value: &String) -> RustString {
        let source_len = value.len() as usize;
        let copy_len = if source_len > MAX_MESSAGE_BYTES {
            MAX_MESSAGE_BYTES
        } else {
            source_len
        };

        let mut buf = [0u8; MAX_MESSAGE_BYTES];
        value.copy_into_slice(&mut buf[..copy_len]);
        core::str::from_utf8(&buf[..copy_len])
            .unwrap_or("")
            .to_string()
    }

    fn extract_root_tag(payload: &str) -> Option<RustString> {
        let mut rest = payload;
        if let Some(xml_start) = rest.find("?>") {
            let next_index = xml_start.checked_add(2)?;
            rest = rest.get(next_index..)?;
        }
        let tag_start = rest.find('<')?.checked_add(1)?;
        let tag_rest = rest.get(tag_start..)?;
        let end = tag_rest.find(['>', ' ', '/']).unwrap_or(tag_rest.len());
        let tag = &tag_rest[..end];
        if tag.is_empty() {
            None
        } else {
            Some(tag.to_string())
        }
    }

    fn extract_attr(payload: &str, attr: &str) -> Option<RustString> {
        let needle = format!("{attr}=\"");
        let start = payload.find(&needle)?.checked_add(needle.len())?;
        let rest = payload.get(start..)?;
        let end = rest.find('"')?;
        Some(rest[..end].to_string())
    }

    fn extract_tag_text(payload: &str, tag: &str) -> Option<RustString> {
        let open = format!("<{tag}>");
        let close = format!("</{tag}>");
        let start = payload.find(&open)?.checked_add(open.len())?;
        let rest = payload.get(start..)?;
        let end = rest.find(&close)?;
        Some(rest[..end].to_string())
    }
}
