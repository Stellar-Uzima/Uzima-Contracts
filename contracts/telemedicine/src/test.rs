use soroban_sdk::testutils::{ Address as TestAddress, Ledger as TestLedger };
use soroban_sdk::{ Address, BytesN, Env, String, Vec };

use crate::{
    ConsentType,
    ConsultationStatus,
    TelemedicineContract,
    TelemedicineContractClient,
    TelemedicineError,
    VideoQuality,
    VideoCodec,
    EncryptionLevel,
    DeviceType,
    WaitingRoomStatus,
    ESIGNStatus,
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
        let client = TelemedicineContractClient::new(
            unsafe {
                &*(&env as *const Env)
            },
            &contract_id
        );

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

        let mut supported_qualities = Vec::new(&self.env);
        supported_qualities.push_back(VideoQuality::FullHD);
        supported_qualities.push_back(VideoQuality::High);

        let mut supported_codecs = Vec::new(&self.env);
        supported_codecs.push_back(VideoCodec::H264);
        supported_codecs.push_back(VideoCodec::VP8);

        self.client.register_provider(
            &self.provider_id,
            &self.provider,
            &String::from_str(&self.env, "Dr. John Smith"),
            &BytesN::from_array(&self.env, &[10u8; 32]),
            &jurisdictions,
            &String::from_str(&self.env, "General Practice"),
            &2_000_000u64,
            &10u32,
            &supported_qualities,
            &supported_codecs,
            &true,
            &BytesN::from_array(&self.env, &[11u8; 32])
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
            &true,
            &true,
            &true,
            &VideoQuality::FullHD,
            &10000u32
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
            &None
        );
        consent_id
    }
}

// ============================================================
// TELEHEALTH INTEGRATION TESTS
// ============================================================

#[test]
fn test_video_session_creation() {
    let ctx = TestContext::new();
    ctx.setup_provider();
    ctx.setup_patient();
    ctx.setup_consent(ConsentType::VideoConsultation);

    let session_id = BytesN::from_array(&ctx.env, &[200u8; 32]);
    let consultation_id = BytesN::from_array(&ctx.env, &[201u8; 32]);
    let appointment_id = BytesN::from_array(&ctx.env, &[202u8; 32]);

    // First schedule a consultation
    ctx.client.schedule_consultation(
        &consultation_id,
        &ctx.patient_id,
        &ctx.provider_id,
        &1_500_000u64,
        &String::from_str(&ctx.env, "General Consultation"),
        &appointment_id
    );

    // Create video session
    ctx.client.create_video_session(
        &session_id,
        &consultation_id,
        &ctx.patient_id,
        &ctx.provider_id,
        &String::from_str(&ctx.env, "room-123"),
        &VideoQuality::FullHD,
        &VideoCodec::H264,
        &EncryptionLevel::Enhanced,
        &true
    );

    // Start the video session
    ctx.client.start_video_session(&session_id, &String::from_str(&ctx.env, "sdp-offer-data"));

    // Verify 1080p 30fps quality metrics
    let metrics = crate::VideoQualityMetrics {
        session_id,
        resolution_width: 1920,
        resolution_height: 1080,
        frame_rate: 30,
        bitrate_kbps: 5000,
        cpu_usage: 45,
        memory_usage: 60,
        network_jitter: 20,
        round_trip_time: 50,
        video_freeze_count: 0,
        audio_freeze_count: 0,
        quality_score: 95,
        hipaa_compliance_score: 100,
        timestamp: ctx.env.ledger().timestamp(),
    };

    ctx.client.update_video_quality(&session_id, metrics);

    // End the session
    ctx.client.end_video_session(&session_id, &BytesN::from_array(&ctx.env, &[203u8; 32]));
}

#[test]
fn test_concurrent_session_limit() {
    let ctx = TestContext::new();
    ctx.setup_provider();
    ctx.setup_patient();
    ctx.setup_consent(ConsentType::VideoConsultation);

    // Test that the system can handle the concurrent session limit
    let consultation_id = BytesN::from_array(&ctx.env, &[210u8; 32]);
    let appointment_id = BytesN::from_array(&ctx.env, &[211u8; 32]);

    ctx.client.schedule_consultation(
        &consultation_id,
        &ctx.patient_id,
        &ctx.provider_id,
        &1_500_000u64,
        &String::from_str(&ctx.env, "General Consultation"),
        &appointment_id
    );

    // This should succeed as we're under the 10,000 limit
    ctx.client.create_video_session(
        &BytesN::from_array(&ctx.env, &[212u8; 32]),
        &consultation_id,
        &ctx.patient_id,
        &ctx.provider_id,
        &String::from_str(&ctx.env, "room-124"),
        &VideoQuality::FullHD,
        &VideoCodec::H264,
        &EncryptionLevel::Enhanced,
        &true
    );
}

#[test]
fn test_virtual_waiting_room() {
    let ctx = TestContext::new();
    ctx.setup_provider();
    ctx.setup_patient();

    let room_id = BytesN::from_array(&ctx.env, &[300u8; 32]);
    let patient2_id = BytesN::from_array(&ctx.env, &[301u8; 32]);

    // Create waiting room
    ctx.client.create_waiting_room(
        &room_id,
        &ctx.provider_id,
        &5u32 // max capacity
    );

    // Patient joins waiting room
    ctx.client.join_waiting_room(&room_id, &ctx.patient_id);

    // Register and add second patient
    ctx.client.register_patient(
        &patient2_id,
        &generate_test_address(&ctx.env),
        &ctx.provider_id,
        &String::from_str(&ctx.env, "KE"),
        &String::from_str(&ctx.env, "+254700000002"),
        &String::from_str(&ctx.env, "English"),
        &true,
        &true,
        &true,
        &VideoQuality::FullHD,
        &10000u32
    );

    ctx.client.join_waiting_room(&room_id, &patient2_id);

    // Patient leaves waiting room
    ctx.client.leave_waiting_room(&room_id, &ctx.patient_id);
}

#[test]
fn test_remote_device_monitoring() {
    let ctx = TestContext::new();
    ctx.setup_provider();
    ctx.setup_patient();

    let device_id = BytesN::from_array(&ctx.env, &[400u8; 32]);
    let reading_id = BytesN::from_array(&ctx.env, &[401u8; 32]);

    // Register remote device
    ctx.client.register_remote_device(
        &device_id,
        &ctx.patient_id,
        &DeviceType::BloodPressure,
        &String::from_str(&ctx.env, "Omron BP7000"),
        &String::from_str(&ctx.env, "Omron Healthcare"),
        &String::from_str(&ctx.env, "BP7000"),
        &String::from_str(&ctx.env, "1.2.3"),
        &String::from_str(&ctx.env, "OM123456789"),
        &String::from_str(&ctx.env, "00:1A:7D:DA:71:13"),
        &true,
        &BytesN::from_array(&ctx.env, &[402u8; 32])
    );

    // Connect device
    ctx.client.connect_device(&device_id, &ctx.patient_id);

    // Record vital signs
    ctx.client.record_vital_signs(
        &reading_id,
        &device_id,
        &ctx.patient_id,
        &DeviceType::BloodPressure,
        &120u32, // systolic BP
        &String::from_str(&ctx.env, "mmHg"),
        &BytesN::from_array(&ctx.env, &[403u8; 32])
    );
}

#[test]
fn test_esign_prescription() {
    let ctx = TestContext::new();
    ctx.setup_provider();
    ctx.setup_patient();
    ctx.setup_consent(ConsentType::VideoConsultation);

    let consultation_id = BytesN::from_array(&ctx.env, &[500u8; 32]);
    let appointment_id = BytesN::from_array(&ctx.env, &[501u8; 32]);
    let prescription_id = BytesN::from_array(&ctx.env, &[502u8; 32]);

    // Complete consultation first
    ctx.client.schedule_consultation(
        &consultation_id,
        &ctx.patient_id,
        &ctx.provider_id,
        &1_500_000u64,
        &String::from_str(&ctx.env, "General Consultation"),
        &appointment_id
    );

    ctx.env.ledger().with_mut(|l| {
        l.timestamp = 1_500_000;
    });
    ctx.client.start_consultation(&consultation_id, &ctx.provider);

    ctx.env.ledger().with_mut(|l| {
        l.timestamp = 1_501_800;
    });
    ctx.client.complete_consultation(
        &consultation_id,
        &ctx.provider,
        &BytesN::from_array(&ctx.env, &[503u8; 32]),
        &appointment_id,
        &85u32
    );

    // Issue e-sign prescription
    let mut medications = Vec::new(&ctx.env);
    medications.push_back(String::from_str(&ctx.env, "Amoxicillin 500mg"));

    let mut dosage_instructions = Vec::new(&ctx.env);
    dosage_instructions.push_back(String::from_str(&ctx.env, "Take twice daily with food"));

    ctx.client.issue_esign_prescription(
        &prescription_id,
        &consultation_id,
        &ctx.patient_id,
        &ctx.provider_id,
        &ctx.provider,
        &medications,
        &dosage_instructions,
        &14u64,
        &String::from_str(&ctx.env, "PHARMACY-KE-001"),
        &BytesN::from_array(&ctx.env, &[504u8; 64]), // digital signature
        &BytesN::from_array(&ctx.env, &[11u8; 32]), // certificate id
        &String::from_str(&ctx.env, "KE")
    );

    // Patient signs prescription
    ctx.client.sign_prescription(&prescription_id, &ctx.patient);
}

#[test]
fn test_hipaa_compliance() {
    let ctx = TestContext::new();
    ctx.setup_provider();
    ctx.setup_patient();
    ctx.setup_consent(ConsentType::VideoConsultation);

    let session_id = BytesN::from_array(&ctx.env, &[600u8; 32]);
    let consultation_id = BytesN::from_array(&ctx.env, &[601u8; 32]);
    let appointment_id = BytesN::from_array(&ctx.env, &[602u8; 32]);

    // Schedule consultation
    ctx.client.schedule_consultation(
        &consultation_id,
        &ctx.patient_id,
        &ctx.provider_id,
        &1_500_000u64,
        &String::from_str(&ctx.env, "General Consultation"),
        &appointment_id
    );

    // Create HIPAA-compliant video session
    ctx.client.create_video_session(
        &session_id,
        &consultation_id,
        &ctx.patient_id,
        &ctx.provider_id,
        &String::from_str(&ctx.env, "room-600"),
        &VideoQuality::FullHD,
        &VideoCodec::H264,
        &EncryptionLevel::Maximum, // Maximum encryption for HIPAA
        &true // recording enabled
    );

    // Verify HIPAA compliance
    let is_compliant = ctx.client.verify_hipaa_compliance(&session_id);
    assert!(is_compliant);
}

#[test]
fn test_video_quality_requirements() {
    let ctx = TestContext::new();
    ctx.setup_provider();
    ctx.setup_patient();
    ctx.setup_consent(ConsentType::VideoConsultation);

    let session_id = BytesN::from_array(&ctx.env, &[700u8; 32]);
    let consultation_id = BytesN::from_array(&ctx.env, &[701u8; 32]);
    let appointment_id = BytesN::from_array(&ctx.env, &[702u8; 32]);

    ctx.client.schedule_consultation(
        &consultation_id,
        &ctx.patient_id,
        &ctx.provider_id,
        &1_500_000u64,
        &String::from_str(&ctx.env, "General Consultation"),
        &appointment_id
    );

    ctx.client.create_video_session(
        &session_id,
        &consultation_id,
        &ctx.patient_id,
        &ctx.provider_id,
        &String::from_str(&ctx.env, "room-700"),
        &VideoQuality::FullHD,
        &VideoCodec::H264,
        &EncryptionLevel::Enhanced,
        &true
    );

    ctx.client.start_video_session(&session_id, &String::from_str(&ctx.env, "sdp-offer-data"));

    // Test valid 1080p 30fps metrics
    let valid_metrics = crate::VideoQualityMetrics {
        session_id,
        resolution_width: 1920,
        resolution_height: 1080,
        frame_rate: 30,
        bitrate_kbps: 5000,
        cpu_usage: 45,
        memory_usage: 60,
        network_jitter: 20,
        round_trip_time: 50,
        video_freeze_count: 0,
        audio_freeze_count: 0,
        quality_score: 95,
        hipaa_compliance_score: 100,
        timestamp: ctx.env.ledger().timestamp(),
    };

    ctx.client.update_video_quality(&session_id, valid_metrics);

    // Test invalid quality (below 1080p)
    let invalid_metrics = crate::VideoQualityMetrics {
        session_id: BytesN::from_array(&ctx.env, &[703u8; 32]),
        resolution_width: 1280, // Not 1920
        resolution_height: 720, // Not 1080
        frame_rate: 30,
        bitrate_kbps: 2500,
        cpu_usage: 30,
        memory_usage: 40,
        network_jitter: 15,
        round_trip_time: 40,
        video_freeze_count: 0,
        audio_freeze_count: 0,
        quality_score: 80,
        hipaa_compliance_score: 90,
        timestamp: ctx.env.ledger().timestamp(),
    };

    let result = ctx.client.try_update_video_quality(
        &BytesN::from_array(&ctx.env, &[703u8; 32]),
        invalid_metrics
    );
    assert_err!(result, TelemedicineError::VideoQualityUnsupported);
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
        &String::from_str(&ctx.env, "English")
    );
    assert_err!(result, TelemedicineError::ContractPaused);

    ctx.client.unpause();

    ctx.client.register_patient(
        &BytesN::from_array(&ctx.env, &[3u8; 32]),
        &ctx.patient,
        &BytesN::from_array(&ctx.env, &[4u8; 32]),
        &String::from_str(&ctx.env, "KE"),
        &String::from_str(&ctx.env, "+254700000002"),
        &String::from_str(&ctx.env, "English")
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
    assert_eq!(provider.specialty, String::from_str(&ctx.env, "General Practice"));
    assert_eq!(provider.jurisdictions.len(), 2);

    let (providers, _, _, _, _, _) = ctx.client.get_platform_stats();
    assert_eq!(providers, 1);
}

#[test]
fn test_register_expired_provider() {
    let ctx = TestContext::new();
    ctx.env.ledger().with_mut(|l| {
        l.timestamp = 1_000_000;
    });

    let mut jurisdictions = Vec::new(&ctx.env);
    jurisdictions.push_back(String::from_str(&ctx.env, "KE"));

    let result = ctx.client.try_register_provider(
        &BytesN::from_array(&ctx.env, &[4u8; 32]),
        &ctx.provider,
        &String::from_str(&ctx.env, "Dr. Expired"),
        &BytesN::from_array(&ctx.env, &[11u8; 32]),
        &jurisdictions,
        &String::from_str(&ctx.env, "Cardiology"),
        &500_000u64
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
    assert_eq!(patient.contact_info, String::from_str(&ctx.env, "+254700000001"));
    assert_eq!(patient.preferred_language, String::from_str(&ctx.env, "English"));
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
        &Some(2_000_000u64)
    );

    let has_consent = ctx.client.has_valid_consent(
        &ctx.patient_id,
        &ConsentType::VideoConsultation
    );
    assert!(has_consent);
}

#[test]
fn test_revoke_consent() {
    let ctx = TestContext::new();
    ctx.setup_provider();
    ctx.setup_patient();
    // setup_consent returns the id it used — reuse it for revoke
    let consent_id = ctx.setup_consent(ConsentType::VideoConsultation);

    let has_consent = ctx.client.has_valid_consent(
        &ctx.patient_id,
        &ConsentType::VideoConsultation
    );
    assert!(has_consent);

    ctx.client.revoke_consent(&consent_id);

    let has_consent = ctx.client.has_valid_consent(
        &ctx.patient_id,
        &ConsentType::VideoConsultation
    );
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
        &appointment_id
    );

    let consultation = ctx.client.get_consultation(&session_id);
    assert!(matches!(consultation.status, ConsultationStatus::Scheduled));
    assert_eq!(consultation.patient_id, ctx.patient_id);
    assert_eq!(consultation.provider_id, ctx.provider_id);

    ctx.env.ledger().with_mut(|l| {
        l.timestamp = 1_500_000;
    });
    ctx.client.start_consultation(&session_id, &ctx.provider);

    let consultation = ctx.client.get_consultation(&session_id);
    assert!(matches!(consultation.status, ConsultationStatus::Active));
    assert_eq!(consultation.start_time, 1_500_000u64);

    ctx.env.ledger().with_mut(|l| {
        l.timestamp = 1_501_800;
    });
    ctx.client.complete_consultation(
        &session_id,
        &ctx.provider,
        &BytesN::from_array(&ctx.env, &[46u8; 32]),
        &appointment_id,
        &85u32
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
        &BytesN::from_array(&ctx.env, &[48u8; 32])
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
        &appointment_id
    );

    ctx.env.ledger().with_mut(|l| {
        l.timestamp = 1_500_000;
    });
    ctx.client.start_consultation(&session_id, &ctx.provider);

    ctx.env.ledger().with_mut(|l| {
        l.timestamp = 1_501_800;
    });
    ctx.client.complete_consultation(
        &session_id,
        &ctx.provider,
        &recording_hash,
        &appointment_id,
        &85u32
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
        &String::from_str(&ctx.env, "PHARMACY-KE-001")
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

    ctx.client.start_monitoring_session(&monitoring_id, &ctx.patient_id, &ctx.provider_id, &24u32);

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
        &appointment_id
    );

    ctx.env.ledger().with_mut(|l| {
        l.timestamp = 1_500_000;
    });
    ctx.client.start_consultation(&session_id, &ctx.provider);

    ctx.env.ledger().with_mut(|l| {
        l.timestamp = 1_501_800;
    });
    ctx.client.complete_consultation(
        &session_id,
        &ctx.provider,
        &recording_hash,
        &appt_complete,
        &85u32
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
        &String::from_str(&ctx.env, "PHARMACY-KE-001")
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
    ctx.env.ledger().with_mut(|l| {
        l.timestamp = 1_000_000;
    });

    let result = ctx.client.try_register_provider(
        &BytesN::from_array(&ctx.env, &[80u8; 32]),
        &ctx.provider,
        &String::from_str(&ctx.env, "Dr. Expired"),
        &BytesN::from_array(&ctx.env, &[81u8; 32]),
        &jurisdictions,
        &String::from_str(&ctx.env, "General Practice"),
        &500_000u64 // expired: 500_000 < 1_000_000
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
        &String::from_str(&ctx.env, "English")
    );
    assert_err!(result, TelemedicineError::ContractPaused);

    ctx.client.unpause();

    ctx.client.register_patient(
        &BytesN::from_array(&ctx.env, &[90u8; 32]),
        &ctx.patient,
        &BytesN::from_array(&ctx.env, &[91u8; 32]),
        &String::from_str(&ctx.env, "KE"),
        &String::from_str(&ctx.env, "+254700000001"),
        &String::from_str(&ctx.env, "English")
    );
}
