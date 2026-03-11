use soroban_sdk::testutils::{Address as TestAddress, Ledger as TestLedger};
use soroban_sdk::{Address, BytesN, Env, String, Vec};

use crate::{
    ConsentType, ConsultationStatus, TelemedicineContract, TelemedicineContractClient,
    TelemedicineError,
};

fn generate_test_address(env: &Env) -> Address {
    <Address as TestAddress>::generate(env)
}

// Extracts the contract error from a try_ call without using unwrap/unwrap_err
macro_rules! assert_err {
    ($result:expr, $expected:expr) => {
        match $result {
            Err(Ok(e)) => assert_eq!(e, $expected),
            other => panic!("expected Err(Ok({:?})), got {:?}", $expected, other),
        }
    };
}

struct TestContext {
    env: Env,
    client: TelemedicineContractClient<'static>,
    admin: Address,
    provider: Address,
    provider_id: BytesN<32>,
    patient: Address,
    patient_id: BytesN<32>,
}

impl TestContext {
    fn new() -> Self {
        let env = Env::default();
        env.mock_all_auths();

        let contract_id = env.register_contract(None, TelemedicineContract);
        // SAFETY: env outlives client within the same test function
        let client =
            TelemedicineContractClient::new(unsafe { &*(&env as *const Env) }, &contract_id);

        let admin = generate_test_address(&env);
        let provider = generate_test_address(&env);
        let patient = generate_test_address(&env);

        let provider_id = BytesN::from_array(&env, &[1u8; 32]);
        let patient_id = BytesN::from_array(&env, &[2u8; 32]);

        client.initialize(&admin);

        Self {
            env,
            client,
            admin,
            provider,
            provider_id,
            patient,
            patient_id,
        }
    }

    fn setup_provider(&self) {
        let mut jurisdictions = Vec::new(&self.env);
        jurisdictions.push_back(String::from_str(&self.env, "KE"));
        jurisdictions.push_back(String::from_str(&self.env, "US"));

        self.client.register_provider(
            &self.provider_id,
            &self.provider,
            &String::from_str(&self.env, "Dr. John Smith"),
            &BytesN::from_array(&self.env, &[10u8; 32]),
            &jurisdictions,
            &String::from_str(&self.env, "General Practice"),
            &2_000_000u64,
        );
    }

    fn setup_patient(&self) {
        self.client.register_patient(
            &self.patient_id,
            &self.patient,
            &self.provider_id,
            &String::from_str(&self.env, "KE"),
            &String::from_str(&self.env, "+254700000001"),
            &String::from_str(&self.env, "English"),
        );
    }

    /// Grant consent and return the consent_id used so callers can revoke it
    fn setup_consent(&self, consent_type: ConsentType) -> BytesN<32> {
        let consent_id = BytesN::from_array(&self.env, &[100u8; 32]);
        self.client.grant_consent(
            &consent_id,
            &self.patient_id,
            &consent_type,
            &String::from_str(&self.env, "General consent"),
            &None,
        );
        consent_id
    }
}

// ============================================================
// INITIALIZATION TESTS
// ============================================================

#[test]
fn test_initialize_contract() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, TelemedicineContract);
    let client = TelemedicineContractClient::new(&env, &contract_id);
    let admin = generate_test_address(&env);

    client.initialize(&admin);

    let (providers, patients, consultations, prescriptions, alerts, emergencies) =
        client.get_platform_stats();
    assert_eq!(providers, 0);
    assert_eq!(patients, 0);
    assert_eq!(consultations, 0);
    assert_eq!(prescriptions, 0);
    assert_eq!(alerts, 0);
    assert_eq!(emergencies, 0);
}

#[test]
fn test_double_initialization() {
    let ctx = TestContext::new();
    let result = ctx.client.try_initialize(&ctx.admin);
    assert_err!(result, TelemedicineError::NotPaused);
}

// ============================================================
// ADMIN FUNCTIONS TESTS
// ============================================================

#[test]
fn test_pause_unpause() {
    let ctx = TestContext::new();

    ctx.client.pause();

    let result = ctx.client.try_register_patient(
        &BytesN::from_array(&ctx.env, &[3u8; 32]),
        &ctx.patient,
        &BytesN::from_array(&ctx.env, &[4u8; 32]),
        &String::from_str(&ctx.env, "KE"),
        &String::from_str(&ctx.env, "+254700000002"),
        &String::from_str(&ctx.env, "English"),
    );
    assert_err!(result, TelemedicineError::ContractPaused);

    ctx.client.unpause();

    ctx.client.register_patient(
        &BytesN::from_array(&ctx.env, &[3u8; 32]),
        &ctx.patient,
        &BytesN::from_array(&ctx.env, &[4u8; 32]),
        &String::from_str(&ctx.env, "KE"),
        &String::from_str(&ctx.env, "+254700000002"),
        &String::from_str(&ctx.env, "English"),
    );
}

// ============================================================
// PROVIDER MANAGEMENT TESTS
// ============================================================

#[test]
fn test_register_provider() {
    let ctx = TestContext::new();
    ctx.setup_provider();

    let provider = ctx.client.get_provider(&ctx.provider_id);
    assert_eq!(provider.name, String::from_str(&ctx.env, "Dr. John Smith"));
    assert!(provider.is_active);
    assert_eq!(
        provider.specialty,
        String::from_str(&ctx.env, "General Practice")
    );
    assert_eq!(provider.jurisdictions.len(), 2);

    let (providers, _, _, _, _, _) = ctx.client.get_platform_stats();
    assert_eq!(providers, 1);
}

#[test]
fn test_register_expired_provider() {
    let ctx = TestContext::new();
    ctx.env.ledger().with_mut(|l| l.timestamp = 1_000_000);

    let mut jurisdictions = Vec::new(&ctx.env);
    jurisdictions.push_back(String::from_str(&ctx.env, "KE"));

    let result = ctx.client.try_register_provider(
        &BytesN::from_array(&ctx.env, &[4u8; 32]),
        &ctx.provider,
        &String::from_str(&ctx.env, "Dr. Expired"),
        &BytesN::from_array(&ctx.env, &[11u8; 32]),
        &jurisdictions,
        &String::from_str(&ctx.env, "Cardiology"),
        &500_000u64,
    );
    assert_err!(result, TelemedicineError::LicenseExpired);
}

#[test]
fn test_deactivate_provider() {
    let ctx = TestContext::new();
    ctx.setup_provider();

    ctx.client.deactivate_provider(&ctx.provider_id);

    let provider = ctx.client.get_provider(&ctx.provider_id);
    assert!(!provider.is_active);
}

// ============================================================
// PATIENT MANAGEMENT TESTS
// ============================================================

#[test]
fn test_register_patient() {
    let ctx = TestContext::new();
    ctx.setup_provider();
    ctx.setup_patient();

    let patient = ctx.client.get_patient(&ctx.patient_id);
    assert_eq!(patient.jurisdiction, String::from_str(&ctx.env, "KE"));
    assert_eq!(
        patient.contact_info,
        String::from_str(&ctx.env, "+254700000001")
    );
    assert_eq!(
        patient.preferred_language,
        String::from_str(&ctx.env, "English")
    );
    assert_eq!(patient.primary_care_physician, ctx.provider_id);

    let (_, patients, _, _, _, _) = ctx.client.get_platform_stats();
    assert_eq!(patients, 1);
}

// ============================================================
// CONSENT MANAGEMENT TESTS
// ============================================================

#[test]
fn test_grant_consent() {
    let ctx = TestContext::new();
    ctx.setup_provider();
    ctx.setup_patient();

    ctx.client.grant_consent(
        &BytesN::from_array(&ctx.env, &[101u8; 32]),
        &ctx.patient_id,
        &ConsentType::VideoConsultation,
        &String::from_str(&ctx.env, "Video consultation consent"),
        &Some(2_000_000u64),
    );

    let has_consent = ctx
        .client
        .has_valid_consent(&ctx.patient_id, &ConsentType::VideoConsultation);
    assert!(has_consent);
}

#[test]
fn test_revoke_consent() {
    let ctx = TestContext::new();
    ctx.setup_provider();
    ctx.setup_patient();
    // setup_consent returns the id it used — reuse it for revoke
    let consent_id = ctx.setup_consent(ConsentType::VideoConsultation);

    let has_consent = ctx
        .client
        .has_valid_consent(&ctx.patient_id, &ConsentType::VideoConsultation);
    assert!(has_consent);

    ctx.client.revoke_consent(&consent_id);

    let has_consent = ctx
        .client
        .has_valid_consent(&ctx.patient_id, &ConsentType::VideoConsultation);
    assert!(!has_consent);
}

// ============================================================
// CONSULTATION MANAGEMENT TESTS
// ============================================================

#[test]
fn test_consultation_lifecycle() {
    let ctx = TestContext::new();
    ctx.setup_provider();
    ctx.setup_patient();
    ctx.setup_consent(ConsentType::VideoConsultation);

    let session_id = BytesN::from_array(&ctx.env, &[44u8; 32]);
    let appointment_id = BytesN::from_array(&ctx.env, &[45u8; 32]);

    ctx.client.schedule_consultation(
        &session_id,
        &ctx.patient_id,
        &ctx.provider_id,
        &1_500_000u64,
        &String::from_str(&ctx.env, "General Consultation"),
        &appointment_id,
    );

    let consultation = ctx.client.get_consultation(&session_id);
    assert!(matches!(consultation.status, ConsultationStatus::Scheduled));
    assert_eq!(consultation.patient_id, ctx.patient_id);
    assert_eq!(consultation.provider_id, ctx.provider_id);

    ctx.env.ledger().with_mut(|l| l.timestamp = 1_500_000);
    ctx.client.start_consultation(&session_id, &ctx.provider);

    let consultation = ctx.client.get_consultation(&session_id);
    assert!(matches!(consultation.status, ConsultationStatus::Active));
    assert_eq!(consultation.start_time, 1_500_000u64);

    ctx.env.ledger().with_mut(|l| l.timestamp = 1_501_800);
    ctx.client.complete_consultation(
        &session_id,
        &ctx.provider,
        &BytesN::from_array(&ctx.env, &[46u8; 32]),
        &appointment_id,
        &85u32,
    );

    let consultation = ctx.client.get_consultation(&session_id);
    assert!(matches!(consultation.status, ConsultationStatus::Completed));
    assert_eq!(consultation.start_time, 1_500_000u64);
    assert_eq!(consultation.end_time, 1_501_800u64);
    assert_eq!(consultation.quality_score, 85u32);

    let (_, _, consultations, _, _, _) = ctx.client.get_platform_stats();
    assert_eq!(consultations, 1);
}

#[test]
fn test_consultation_without_consent() {
    let ctx = TestContext::new();
    ctx.setup_provider();
    ctx.setup_patient();
    // No consent granted — schedule should fail

    let result = ctx.client.try_schedule_consultation(
        &BytesN::from_array(&ctx.env, &[47u8; 32]),
        &ctx.patient_id,
        &ctx.provider_id,
        &1_500_000u64,
        &String::from_str(&ctx.env, "General Consultation"),
        &BytesN::from_array(&ctx.env, &[48u8; 32]),
    );
    assert_err!(result, TelemedicineError::ConsentNotGiven);
}

// ============================================================
// PRESCRIPTION MANAGEMENT TESTS
// ============================================================

#[test]
fn test_prescription_issuance() {
    let ctx = TestContext::new();
    ctx.setup_provider();
    ctx.setup_patient();
    ctx.setup_consent(ConsentType::VideoConsultation);

    let session_id = BytesN::from_array(&ctx.env, &[50u8; 32]);
    let appointment_id = BytesN::from_array(&ctx.env, &[51u8; 32]);
    let recording_hash = BytesN::from_array(&ctx.env, &[52u8; 32]);
    let prescription_id = BytesN::from_array(&ctx.env, &[53u8; 32]);

    ctx.client.schedule_consultation(
        &session_id,
        &ctx.patient_id,
        &ctx.provider_id,
        &1_500_000u64,
        &String::from_str(&ctx.env, "General Consultation"),
        &appointment_id,
    );

    ctx.env.ledger().with_mut(|l| l.timestamp = 1_500_000);
    ctx.client.start_consultation(&session_id, &ctx.provider);

    ctx.env.ledger().with_mut(|l| l.timestamp = 1_501_800);
    ctx.client.complete_consultation(
        &session_id,
        &ctx.provider,
        &recording_hash,
        &appointment_id,
        &85u32,
    );

    let mut meds = Vec::new(&ctx.env);
    meds.push_back(String::from_str(&ctx.env, "J01CA04"));

    ctx.client.issue_prescription(
        &prescription_id,
        &session_id,
        &ctx.patient_id,
        &ctx.provider_id,
        &ctx.provider,
        &meds,
        &14u64,
        &String::from_str(&ctx.env, "PHARMACY-KE-001"),
    );

    let prescription = ctx.client.get_prescription(&prescription_id);
    assert_eq!(prescription.patient_id, ctx.patient_id);
    assert_eq!(prescription.provider_id, ctx.provider_id);
    assert_eq!(prescription.consultation_id, session_id);
    assert!(prescription.is_active);
    assert_eq!(prescription.valid_days, 14);

    let (_, _, _, prescriptions, _, _) = ctx.client.get_platform_stats();
    assert_eq!(prescriptions, 1);
}

// ============================================================
// MONITORING SESSION TESTS
// ============================================================

#[test]
fn test_monitoring_session() {
    let ctx = TestContext::new();
    ctx.setup_provider();
    ctx.setup_patient();

    let monitoring_id = BytesN::from_array(&ctx.env, &[60u8; 32]);

    ctx.client
        .start_monitoring_session(&monitoring_id, &ctx.patient_id, &ctx.provider_id, &24u32);

    let session = ctx.client.end_monitoring_session(&monitoring_id);
    assert!(!session.is_active);
}

// ============================================================
// PLATFORM STATS TESTS
// ============================================================

#[test]
fn test_platform_statistics() {
    let ctx = TestContext::new();
    ctx.setup_provider();
    ctx.setup_patient();
    ctx.setup_consent(ConsentType::VideoConsultation);

    let (providers, patients, consultations, prescriptions, alerts, emergencies) =
        ctx.client.get_platform_stats();
    assert_eq!(providers, 1);
    assert_eq!(patients, 1);
    assert_eq!(consultations, 0);
    assert_eq!(prescriptions, 0);
    assert_eq!(alerts, 0);
    assert_eq!(emergencies, 0);

    let session_id = BytesN::from_array(&ctx.env, &[70u8; 32]);
    let appointment_id = BytesN::from_array(&ctx.env, &[71u8; 32]);
    let recording_hash = BytesN::from_array(&ctx.env, &[72u8; 32]);
    let appt_complete = BytesN::from_array(&ctx.env, &[73u8; 32]);
    let prescription_id = BytesN::from_array(&ctx.env, &[74u8; 32]);

    ctx.client.schedule_consultation(
        &session_id,
        &ctx.patient_id,
        &ctx.provider_id,
        &1_500_000u64,
        &String::from_str(&ctx.env, "consultation"),
        &appointment_id,
    );

    ctx.env.ledger().with_mut(|l| l.timestamp = 1_500_000);
    ctx.client.start_consultation(&session_id, &ctx.provider);

    ctx.env.ledger().with_mut(|l| l.timestamp = 1_501_800);
    ctx.client.complete_consultation(
        &session_id,
        &ctx.provider,
        &recording_hash,
        &appt_complete,
        &85u32,
    );

    let mut meds = Vec::new(&ctx.env);
    meds.push_back(String::from_str(&ctx.env, "J01CA04"));

    ctx.client.issue_prescription(
        &prescription_id,
        &session_id,
        &ctx.patient_id,
        &ctx.provider_id,
        &ctx.provider,
        &meds,
        &14u64,
        &String::from_str(&ctx.env, "PHARMACY-KE-001"),
    );

    let (providers, patients, consultations, prescriptions, alerts, emergencies) =
        ctx.client.get_platform_stats();
    assert_eq!(providers, 1);
    assert_eq!(patients, 1);
    assert_eq!(consultations, 1);
    assert_eq!(prescriptions, 1);
    assert_eq!(alerts, 0);
    assert_eq!(emergencies, 0);
}

// ============================================================
// ERROR HANDLING TESTS
// ============================================================

#[test]
fn test_invalid_jurisdiction() {
    let ctx = TestContext::new();
    ctx.setup_provider();
    ctx.setup_patient();
    // Placeholder: jurisdiction validation not yet implemented
}

#[test]
fn test_expired_provider_license() {
    let ctx = TestContext::new();

    // Register a provider with an already-expired license (no timestamp set = 0)
    let mut jurisdictions = Vec::new(&ctx.env);
    jurisdictions.push_back(String::from_str(&ctx.env, "KE"));

    // license_expiry 0 < current timestamp 0 is NOT expired (equal), so advance time first
    ctx.env.ledger().with_mut(|l| l.timestamp = 1_000_000);

    let result = ctx.client.try_register_provider(
        &BytesN::from_array(&ctx.env, &[80u8; 32]),
        &ctx.provider,
        &String::from_str(&ctx.env, "Dr. Expired"),
        &BytesN::from_array(&ctx.env, &[81u8; 32]),
        &jurisdictions,
        &String::from_str(&ctx.env, "General Practice"),
        &500_000u64, // expired: 500_000 < 1_000_000
    );
    assert_err!(result, TelemedicineError::LicenseExpired);
}

#[test]
fn test_contract_pause() {
    let ctx = TestContext::new();

    ctx.client.pause();

    let result = ctx.client.try_register_patient(
        &BytesN::from_array(&ctx.env, &[90u8; 32]),
        &ctx.patient,
        &BytesN::from_array(&ctx.env, &[91u8; 32]),
        &String::from_str(&ctx.env, "KE"),
        &String::from_str(&ctx.env, "+254700000001"),
        &String::from_str(&ctx.env, "English"),
    );
    assert_err!(result, TelemedicineError::ContractPaused);

    ctx.client.unpause();

    ctx.client.register_patient(
        &BytesN::from_array(&ctx.env, &[90u8; 32]),
        &ctx.patient,
        &BytesN::from_array(&ctx.env, &[91u8; 32]),
        &String::from_str(&ctx.env, "KE"),
        &String::from_str(&ctx.env, "+254700000001"),
        &String::from_str(&ctx.env, "English"),
    );
}
