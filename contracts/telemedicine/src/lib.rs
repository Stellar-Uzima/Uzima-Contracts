#![no_std]
#![allow(clippy::too_many_arguments)]
#![allow(clippy::fn_params_excessive_bools)]
#![allow(clippy::needless_pass_by_value)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_lossless)]
#![allow(clippy::used_underscore_binding)]

#[cfg(test)]
mod test;

use soroban_sdk::{
    contract,
    contracterror,
    contractimpl,
    contracttype,
    log,
    Address,
    BytesN,
    Env,
    String,
    Vec,
};

// ============================================================
// ERROR DEFINITIONS
// ============================================================

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
pub enum TelemedicineError {
    ContractPaused = 1,
    NotPaused = 2,
    NotAdmin = 3,
    ProviderAlreadyRegistered = 4,
    ProviderNotFound = 5,
    ProviderNotActive = 6,
    LicenseExpired = 7,
    PatientAlreadyRegistered = 8,
    PatientNotFound = 9,
    ConsentNotGiven = 10,
    ConsultationNotFound = 11,
    ConsultationNotScheduled = 12,
    ConsultationNotActive = 13,
    ConsultationAlreadyCompleted = 14,
    PrescriptionNotFound = 15,
    MonitoringSessionNotFound = 16,
    AppointmentNotFound = 17,
    DigitalTherapeuticNotFound = 18,
    QualityAssessmentNotFound = 19,
    EmergencyNotFound = 20,
    EmergencyAlreadyResolved = 21,
    InvalidJurisdiction = 22,
    DataTransferNotApproved = 23,
    // New telehealth integration errors
    VideoSessionNotFound = 24,
    VideoSessionAlreadyActive = 25,
    VideoSessionNotActive = 26,
    InsufficientBandwidth = 27,
    DeviceNotSupported = 28,
    WaitingRoomFull = 29,
    InvalidEncryptionKey = 30,
    ESIGNInvalidSignature = 31,
    ESIGNExpiredCertificate = 32,
    ESIGNRevokedCertificate = 33,
    ConcurrentSessionLimitExceeded = 34,
    VideoQualityUnsupported = 35,
    HIPAAComplianceFailed = 36,
    DeviceConnectionFailed = 37,
    VitalSignsOutOfRange = 38,
}

// ============================================================
// DATA STRUCTURES
// ============================================================

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[contracttype]
pub enum ConsentType {
    VideoConsultation = 0,
    RemoteMonitoring = 1,
    DigitalTherapeutic = 2,
    EmergencyContact = 3,
    DataSharing = 4,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[contracttype]
pub enum ConsultationStatus {
    Scheduled = 0,
    Active = 1,
    Completed = 2,
    Cancelled = 3,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[contracttype]
pub enum EmergencyLevel {
    Low = 0,
    Medium = 1,
    High = 2,
    Critical = 3,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[contracttype]
pub enum VideoQuality {
    Standard = 0, // 480p 30fps
    High = 1, // 720p 30fps
    FullHD = 2, // 1080p 30fps
    Premium = 3, // 1080p 60fps
    UltraHD = 4, // 4K 30fps
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[contracttype]
pub enum VideoCodec {
    VP8 = 0,
    VP9 = 1,
    H264 = 2,
    H265 = 3,
    AV1 = 4,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[contracttype]
pub enum EncryptionLevel {
    Standard = 0, // AES-128
    Enhanced = 1, // AES-256
    Maximum = 2, // AES-256 + RSA
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[contracttype]
pub enum DeviceType {
    BloodPressure = 0,
    HeartRate = 1,
    GlucoseMeter = 2,
    PulseOximeter = 3,
    Thermometer = 4,
    ECG = 5,
    WeightScale = 6,
    Spirometer = 7,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[contracttype]
pub enum WaitingRoomStatus {
    Waiting = 0,
    InSession = 1,
    Completed = 2,
    Cancelled = 3,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[contracttype]
pub enum ESIGNStatus {
    Pending = 0,
    Signed = 1,
    Rejected = 2,
    Expired = 3,
    Revoked = 4,
}

#[derive(Clone, Debug)]
#[contracttype]
pub struct VideoSession {
    pub session_id: BytesN<32>,
    pub consultation_id: BytesN<32>,
    pub patient_id: BytesN<32>,
    pub provider_id: BytesN<32>,
    pub room_id: String,
    pub sdp_offer: String,
    pub sdp_answer: String,
    pub ice_candidates: Vec<String>,
    pub video_quality: VideoQuality,
    pub video_codec: VideoCodec,
    pub encryption_level: EncryptionLevel,
    pub bandwidth_kbps: u32,
    pub latency_ms: u32,
    pub packet_loss: u32,
    pub is_active: bool,
    pub start_time: u64,
    pub end_time: u64,
    pub recording_enabled: bool,
    pub recording_hash: BytesN<32>,
    pub hipaa_compliant: bool,
}

#[derive(Clone, Debug)]
#[contracttype]
pub struct VirtualWaitingRoom {
    pub room_id: BytesN<32>,
    pub provider_id: BytesN<32>,
    pub patient_queue: Vec<BytesN<32>>,
    pub max_capacity: u32,
    pub current_count: u32,
    pub average_wait_time: u32,
    pub estimated_wait_time: u32,
    pub status: WaitingRoomStatus,
    pub created_at: u64,
    pub last_updated: u64,
}

#[derive(Clone, Debug)]
#[contracttype]
pub struct RemoteDevice {
    pub device_id: BytesN<32>,
    pub patient_id: BytesN<32>,
    pub device_type: DeviceType,
    pub device_name: String,
    pub manufacturer: String,
    pub model: String,
    pub firmware_version: String,
    pub serial_number: String,
    pub bluetooth_address: String,
    pub wifi_enabled: bool,
    pub battery_level: u32,
    pub last_calibration: u64,
    pub certification_hash: BytesN<32>,
    pub is_active: bool,
    pub connection_status: bool,
}

#[derive(Clone, Debug)]
#[contracttype]
pub struct DeviceReading {
    pub reading_id: BytesN<32>,
    pub device_id: BytesN<32>,
    pub patient_id: BytesN<32>,
    pub reading_type: DeviceType,
    pub value: u32,
    pub unit: String,
    pub timestamp: u64,
    pub quality_score: u32,
    pub anomaly_detected: bool,
    pub encrypted_data: BytesN<32>,
    pub compliance_verified: bool,
}

#[derive(Clone, Debug)]
#[contracttype]
pub struct ESIGNPrescription {
    pub prescription_id: BytesN<32>,
    pub consultation_id: BytesN<32>,
    pub patient_id: BytesN<32>,
    pub provider_id: BytesN<32>,
    pub medications: Vec<String>,
    pub dosage_instructions: Vec<String>,
    pub issued_date: u64,
    pub valid_days: u64,
    pub pharmacy_id: String,
    pub digital_signature: BytesN<64>,
    pub certificate_id: BytesN<32>,
    pub signature_timestamp: u64,
    pub status: ESIGNStatus,
    pub verification_hash: BytesN<32>,
    pub jurisdiction: String,
    pub cross_border: bool,
}

#[derive(Clone, Debug)]
#[contracttype]
pub struct VideoQualityMetrics {
    pub session_id: BytesN<32>,
    pub resolution_width: u32,
    pub resolution_height: u32,
    pub frame_rate: u32,
    pub bitrate_kbps: u32,
    pub cpu_usage: u32,
    pub memory_usage: u32,
    pub network_jitter: u32,
    pub round_trip_time: u32,
    pub video_freeze_count: u32,
    pub audio_freeze_count: u32,
    pub quality_score: u32,
    pub hipaa_compliance_score: u32,
    pub timestamp: u64,
}

#[derive(Clone, Debug)]
#[contracttype]
pub struct ConcurrentSessionTracker {
    pub total_sessions: u32,
    pub active_sessions: u32,
    pub peak_sessions: u32,
    pub average_duration: u32,
    pub bandwidth_utilization: u32,
    pub server_load: u32,
    pub quality_degradation_count: u32,
    pub timestamp: u64,
}

#[derive(Clone, Debug)]
#[contracttype]
pub struct Provider {
    pub provider_id: BytesN<32>,
    pub address: Address,
    pub name: String,
    pub credentials: BytesN<32>,
    pub jurisdictions: Vec<String>,
    pub specialty: String,
    pub license_expiry: u64,
    pub is_active: bool,
    pub registration_date: u64,
    pub max_concurrent_sessions: u32,
    pub supported_video_qualities: Vec<VideoQuality>,
    pub supported_codecs: Vec<VideoCodec>,
    pub hipaa_training_verified: bool,
    pub digital_signature_certificate: BytesN<32>,
}

#[derive(Clone, Debug)]
#[contracttype]
pub struct Patient {
    pub patient_id: BytesN<32>,
    pub address: Address,
    pub primary_care_physician: BytesN<32>,
    pub monitoring_device: String,
    pub jurisdiction: String,
    pub contact_info: String,
    pub preferred_language: String,
    pub registration_date: u64,
    pub video_consent_granted: bool,
    pub remote_monitoring_consent: bool,
    pub device_consent_granted: bool,
    pub max_video_quality: VideoQuality,
    pub bandwidth_limit_kbps: u32,
}

#[derive(Clone, Debug)]
#[contracttype]
pub struct ConsentRecord {
    pub consent_id: BytesN<32>,
    pub patient_id: BytesN<32>,
    pub consent_type: ConsentType,
    pub granted: bool,
    pub timestamp: u64,
    pub expiry: u64,
    pub scope: String,
}

#[derive(Clone, Debug)]
#[contracttype]
pub struct Consultation {
    pub session_id: BytesN<32>,
    pub patient_id: BytesN<32>,
    pub provider_id: BytesN<32>,
    pub scheduled_time: u64,
    pub start_time: u64,
    pub end_time: u64,
    pub status: ConsultationStatus,
    pub recording_hash: BytesN<32>,
    pub appointment_id: BytesN<32>,
    pub consultation_type: String,
    pub quality_score: u32,
    pub video_session_id: BytesN<32>,
    pub waiting_room_id: BytesN<32>,
    pub devices_used: Vec<BytesN<32>>,
    pub hipaa_compliance_verified: bool,
}

#[derive(Clone, Debug)]
#[contracttype]
pub struct Prescription {
    pub prescription_id: BytesN<32>,
    pub consultation_id: BytesN<32>,
    pub patient_id: BytesN<32>,
    pub provider_id: BytesN<32>,
    pub medications: Vec<String>,
    pub issued_date: u64,
    pub valid_days: u64,
    pub pharmacy_id: String,
    pub is_active: bool,
    pub cross_border: bool,
    pub jurisdiction: String,
}

#[derive(Clone, Debug)]
#[contracttype]
pub struct VitalSigns {
    pub heart_rate: u32,
    pub blood_pressure_systolic: u32,
    pub blood_pressure_diastolic: u32,
    pub spo2: u32,
    pub temperature: u32,
    pub respiratory_rate: u32,
    pub blood_glucose: u32,
    pub device_id: String,
    pub timestamp: u64,
}

#[derive(Clone, Debug)]
#[contracttype]
pub struct MonitoringSession {
    pub session_id: BytesN<32>,
    pub patient_id: BytesN<32>,
    pub provider_id: BytesN<32>,
    pub start_time: u64,
    pub end_time: u64,
    pub is_active: bool,
    pub vital_signs_count: u32,
    pub alerts_count: u32,
}

#[derive(Clone, Debug)]
#[contracttype]
pub struct AppointmentSlot {
    pub appointment_id: BytesN<32>,
    pub provider_id: BytesN<32>,
    pub patient_id: BytesN<32>,
    pub start_time: u64,
    pub end_time: u64,
    pub consultation_type: String,
    pub is_confirmed: bool,
    pub telemedicine_room: String,
}

#[derive(Clone, Debug)]
#[contracttype]
pub struct ComplianceRecord {
    pub record_id: BytesN<32>,
    pub consultation_id: BytesN<32>,
    pub patient_jurisdiction: String,
    pub provider_jurisdiction: String,
    pub compliance_framework: String,
    pub data_transfer_approved: bool,
    pub gdpr_compliant: bool,
    pub hipaa_compliant: bool,
    pub local_law_compliant: bool,
    pub verification_timestamp: u64,
    pub verified_by: Address,
}

#[derive(Clone, Debug)]
#[contracttype]
pub struct DigitalTherapeutic {
    pub therapeutic_id: BytesN<32>,
    pub patient_id: BytesN<32>,
    pub provider_id: BytesN<32>,
    pub program_name: String,
    pub program_hash: BytesN<32>,
    pub enrollment_date: u64,
    pub completion_percentage: u32,
    pub adherence_score: u32,
    pub session_count: u32,
    pub duration_days: u32,
    pub is_active: bool,
}

#[derive(Clone, Debug)]
#[contracttype]
pub struct QualityAssessment {
    pub assessment_id: BytesN<32>,
    pub consultation_id: BytesN<32>,
    pub assessor_provider: Address,
    pub technical_quality: u32,
    pub clinical_quality: u32,
    pub patient_satisfaction: u32,
    pub connection_quality: u32,
    pub issues: Vec<String>,
    pub assessment_date: u64,
}

#[derive(Clone, Debug)]
#[contracttype]
pub struct EmergencyCase {
    pub emergency_id: BytesN<32>,
    pub patient_id: BytesN<32>,
    pub reporting_provider: BytesN<32>,
    pub responding_provider: BytesN<32>,
    pub emergency_level: EmergencyLevel,
    pub reported_symptoms: String,
    pub triage_notes_hash: BytesN<32>,
    pub triggered_at: u64,
    pub response_time: u64,
    pub resolved_at: u64,
    pub is_resolved: bool,
    pub escalated_to_physical: bool,
}

// ============================================================
// STORAGE KEYS
// ============================================================

#[derive(Clone)]
#[contracttype]
pub enum DataKey {
    // Fix: store admin as a simple key (not keyed by address)
    // so require_admin can retrieve it without knowing the address
    Admin,
    Paused,
    Provider(BytesN<32>),
    Patient(BytesN<32>),
    Consent(BytesN<32>),
    // Index: patient_id -> Vec of consent_ids for that patient
    PatientConsents(BytesN<32>),
    Consultation(BytesN<32>),
    Prescription(BytesN<32>),
    MonitoringSession(BytesN<32>),
    Appointment(BytesN<32>),
    ComplianceRecord(BytesN<32>),
    DigitalTherapeutic(BytesN<32>),
    QualityAssessment(BytesN<32>),
    Emergency(BytesN<32>),
    ProviderSchedule(BytesN<32>),
    ActiveEmergencies,
    PlatformStats,
    // New telehealth integration keys
    VideoSession(BytesN<32>),
    VirtualWaitingRoom(BytesN<32>),
    RemoteDevice(BytesN<32>),
    DeviceReading(BytesN<32>),
    ESIGNPrescription(BytesN<32>),
    VideoQualityMetrics(BytesN<32>),
    ConcurrentSessionTracker,
    ActiveVideoSessions,
    DeviceRegistry,
    ESIGNCertificates,
}

// ============================================================
// CONTRACT IMPLEMENTATION
// ============================================================

#[contract]
pub struct TelemedicineContract;

#[derive(Clone, Debug)]
#[contracttype]
pub struct ProviderCapabilities {
    pub max_concurrent_sessions: u32,
    pub supported_video_qualities: Vec<VideoQuality>,
    pub supported_codecs: Vec<VideoCodec>,
    pub hipaa_training_verified: bool,
    pub digital_signature_certificate: BytesN<32>,
}

#[derive(Clone, Debug)]
#[contracttype]
pub struct ESIGNRequest {
    pub medications: Vec<String>,
    pub dosage_instructions: Vec<String>,
    pub valid_days: u64,
    pub pharmacy_id: String,
    pub digital_signature: BytesN<64>,
    pub certificate_id: BytesN<32>,
    pub jurisdiction: String,
}

#[derive(Clone, Debug)]
#[contracttype]
pub struct PatientCapabilities {
    pub video_consent_granted: bool,
    pub remote_monitoring_consent: bool,
    pub device_consent_granted: bool,
    pub max_video_quality: VideoQuality,
    pub bandwidth_limit_kbps: u32,
}

#[contractimpl]
impl TelemedicineContract {
    // ============================================================
    // ADMIN FUNCTIONS
    // ============================================================

    pub fn initialize(env: Env, admin: Address) -> Result<(), TelemedicineError> {
        if env.storage().persistent().has(&DataKey::Paused) {
            return Err(TelemedicineError::NotPaused);
        }
        // Store admin address under a simple key
        env.storage().persistent().set(&DataKey::Admin, &admin);
        env.storage().persistent().set(&DataKey::Paused, &false);
        env.storage()
            .persistent()
            .set(&DataKey::PlatformStats, &(0u64, 0u64, 0u64, 0u64, 0u64, 0u64));
        // Initialize telehealth platform settings
        env.storage().persistent().set(
            &DataKey::ConcurrentSessionTracker,
            &(ConcurrentSessionTracker {
                total_sessions: 0,
                active_sessions: 0,
                peak_sessions: 0,
                average_duration: 0,
                bandwidth_utilization: 0,
                server_load: 0,
                quality_degradation_count: 0,
                timestamp: env.ledger().timestamp(),
            })
        );
        env.storage()
            .persistent()
            .set(&DataKey::ActiveVideoSessions, &Vec::<BytesN<32>>::new(&env));
        env.storage()
            .persistent()
            .set(&DataKey::DeviceRegistry, &Vec::<BytesN<32>>::new(&env));
        env.storage()
            .persistent()
            .set(&DataKey::ESIGNCertificates, &Vec::<BytesN<32>>::new(&env));
        log!(&env, "Telemedicine contract initialized with telehealth integration");
        Ok(())
    }

    pub fn pause(env: Env) -> Result<(), TelemedicineError> {
        Self::require_admin(&env)?;
        env.storage().persistent().set(&DataKey::Paused, &true);
        log!(&env, "Contract paused");
        Ok(())
    }

    pub fn unpause(env: Env) -> Result<(), TelemedicineError> {
        Self::require_admin(&env)?;
        env.storage().persistent().set(&DataKey::Paused, &false);
        log!(&env, "Contract unpaused");
        Ok(())
    }

    // ============================================================
    // PROVIDER MANAGEMENT
    // ============================================================

    pub fn register_provider(
        env: &Env,
        provider_id: BytesN<32>,
        address: Address,
        name: String,
        credentials: BytesN<32>,
        jurisdictions: Vec<String>,
        specialty: String,
        license_expiry: u64,
        capabilities: ProviderCapabilities
    ) -> Result<(), TelemedicineError> {
        Self::require_not_paused(env)?;
        if env.storage().persistent().has(&DataKey::Provider(provider_id.clone())) {
            return Err(TelemedicineError::ProviderAlreadyRegistered);
        }
        let current_time = env.ledger().timestamp();
        if license_expiry < current_time {
            return Err(TelemedicineError::LicenseExpired);
        }
        let provider = Provider {
            provider_id: provider_id.clone(),
            address,
            name,
            credentials,
            jurisdictions,
            specialty,
            license_expiry,
            is_active: true,
            registration_date: current_time,
            max_concurrent_sessions: capabilities.max_concurrent_sessions,
            supported_video_qualities: capabilities.supported_video_qualities,
            supported_codecs: capabilities.supported_codecs,
            hipaa_training_verified: capabilities.hipaa_training_verified,
            digital_signature_certificate: capabilities.digital_signature_certificate,
        };
        env.storage().persistent().set(&DataKey::Provider(provider_id), &provider);
        Self::increment_platform_stat(env, 0);
        log!(&env, "Provider registered with telehealth capabilities");
        Ok(())
    }

    pub fn get_provider(env: &Env, provider_id: BytesN<32>) -> Result<Provider, TelemedicineError> {
        env.storage()
            .persistent()
            .get(&DataKey::Provider(provider_id))
            .ok_or(TelemedicineError::ProviderNotFound)
    }

    pub fn deactivate_provider(
        env: &Env,
        provider_id: BytesN<32>
    ) -> Result<(), TelemedicineError> {
        Self::require_admin(env)?;
        let mut provider: Provider = env
            .storage()
            .persistent()
            .get(&DataKey::Provider(provider_id.clone()))
            .ok_or(TelemedicineError::ProviderNotFound)?;
        provider.is_active = false;
        env.storage().persistent().set(&DataKey::Provider(provider_id), &provider);
        log!(&env, "Provider deactivated");
        Ok(())
    }

    // ============================================================
    // PATIENT MANAGEMENT
    // ============================================================

    pub fn register_patient(
        env: &Env,
        patient_id: BytesN<32>,
        address: Address,
        primary_care_physician: BytesN<32>,
        jurisdiction: String,
        contact_info: String,
        preferred_language: String,
        capabilities: PatientCapabilities
    ) -> Result<(), TelemedicineError> {
        Self::require_not_paused(env)?;
        if env.storage().persistent().has(&DataKey::Patient(patient_id.clone())) {
            return Err(TelemedicineError::PatientAlreadyRegistered);
        }
        let patient = Patient {
            patient_id: patient_id.clone(),
            address,
            primary_care_physician,
            monitoring_device: String::from_str(env, ""),
            jurisdiction,
            contact_info,
            preferred_language,
            registration_date: env.ledger().timestamp(),
            video_consent_granted: capabilities.video_consent_granted,
            remote_monitoring_consent: capabilities.remote_monitoring_consent,
            device_consent_granted: capabilities.device_consent_granted,
            max_video_quality: capabilities.max_video_quality,
            bandwidth_limit_kbps: capabilities.bandwidth_limit_kbps,
        };
        env.storage().persistent().set(&DataKey::Patient(patient_id), &patient);
        Self::increment_platform_stat(env, 1);
        log!(&env, "Patient registered with telehealth consent");
        Ok(())
    }

    pub fn get_patient(env: &Env, patient_id: BytesN<32>) -> Result<Patient, TelemedicineError> {
        env.storage()
            .persistent()
            .get(&DataKey::Patient(patient_id))
            .ok_or(TelemedicineError::PatientNotFound)
    }

    // ============================================================
    // CONSENT MANAGEMENT
    // ============================================================

    pub fn grant_consent(
        env: &Env,
        consent_id: BytesN<32>,
        patient_id: BytesN<32>,
        consent_type: ConsentType,
        scope: String,
        expiry: Option<u64>
    ) -> Result<(), TelemedicineError> {
        Self::require_not_paused(env)?;
        let consent = ConsentRecord {
            consent_id: consent_id.clone(),
            patient_id: patient_id.clone(),
            consent_type,
            granted: true,
            timestamp: env.ledger().timestamp(),
            expiry: expiry.unwrap_or(u64::MAX),
            scope,
        };
        env.storage().persistent().set(&DataKey::Consent(consent_id.clone()), &consent);

        // Maintain a per-patient index of consent IDs
        let mut ids: Vec<BytesN<32>> = env
            .storage()
            .persistent()
            .get(&DataKey::PatientConsents(patient_id.clone()))
            .unwrap_or_else(|| Vec::new(env));
        ids.push_back(consent_id);
        env.storage().persistent().set(&DataKey::PatientConsents(patient_id), &ids);

        log!(&env, "Consent granted");
        Ok(())
    }

    pub fn revoke_consent(env: &Env, consent_id: BytesN<32>) -> Result<(), TelemedicineError> {
        Self::require_not_paused(env)?;
        let mut consent: ConsentRecord = env
            .storage()
            .persistent()
            .get(&DataKey::Consent(consent_id.clone()))
            .ok_or(TelemedicineError::ConsentNotGiven)?;
        consent.granted = false;
        env.storage().persistent().set(&DataKey::Consent(consent_id), &consent);
        log!(&env, "Consent revoked");
        Ok(())
    }

    /// Returns true only if the patient has at least one active, non-expired
    /// consent record of the requested type.
    pub fn has_valid_consent(
        env: &Env,
        patient_id: BytesN<32>,
        consent_type: ConsentType
    ) -> Result<bool, TelemedicineError> {
        let ids: Vec<BytesN<32>> = env
            .storage()
            .persistent()
            .get(&DataKey::PatientConsents(patient_id))
            .unwrap_or_else(|| Vec::new(env));

        let now = env.ledger().timestamp();
        for id in ids.iter() {
            if
                let Some(record) = env
                    .storage()
                    .persistent()
                    .get::<DataKey, ConsentRecord>(&DataKey::Consent(id.clone()))
            {
                if record.granted && record.consent_type == consent_type && record.expiry >= now {
                    return Ok(true);
                }
            }
        }
        Ok(false)
    }

    // ============================================================
    // CONSULTATION MANAGEMENT
    // ============================================================

    pub fn schedule_consultation(
        env: &Env,
        session_id: BytesN<32>,
        patient_id: BytesN<32>,
        provider_id: BytesN<32>,
        scheduled_time: u64,
        consultation_type: String,
        _appointment_id: BytesN<32>
    ) -> Result<(), TelemedicineError> {
        Self::require_not_paused(env)?;
        let provider: Provider = env
            .storage()
            .persistent()
            .get(&DataKey::Provider(provider_id.clone()))
            .ok_or(TelemedicineError::ProviderNotFound)?;
        if !provider.is_active {
            return Err(TelemedicineError::ProviderNotActive);
        }
        let _: Patient = env
            .storage()
            .persistent()
            .get(&DataKey::Patient(patient_id.clone()))
            .ok_or(TelemedicineError::PatientNotFound)?;
        if !Self::has_valid_consent(env, patient_id.clone(), ConsentType::VideoConsultation)? {
            return Err(TelemedicineError::ConsentNotGiven);
        }
        let consultation = Consultation {
            session_id: session_id.clone(),
            patient_id: patient_id.clone(),
            provider_id: provider_id.clone(),
            scheduled_time,
            start_time: 0,
            end_time: 0,
            status: ConsultationStatus::Scheduled,
            recording_hash: BytesN::from_array(env, &[0u8; 32]),
            appointment_id: _appointment_id.clone(),
            consultation_type,
            quality_score: 0,
            video_session_id: BytesN::from_array(env, &[0u8; 32]),
            waiting_room_id: BytesN::from_array(env, &[0u8; 32]),
            devices_used: Vec::new(env),
            hipaa_compliance_verified: false,
        };
        env.storage().persistent().set(&DataKey::Consultation(session_id), &consultation);
        Self::increment_platform_stat(env, 2);
        log!(&env, "Consultation scheduled");
        Ok(())
    }

    pub fn start_consultation(
        env: &Env,
        session_id: BytesN<32>,
        caller: Address
    ) -> Result<(), TelemedicineError> {
        Self::require_not_paused(env)?;
        caller.require_auth();
        let mut consultation: Consultation = env
            .storage()
            .persistent()
            .get(&DataKey::Consultation(session_id.clone()))
            .ok_or(TelemedicineError::ConsultationNotFound)?;
        if consultation.status != ConsultationStatus::Scheduled {
            return Err(TelemedicineError::ConsultationNotScheduled);
        }
        consultation.status = ConsultationStatus::Active;
        consultation.start_time = env.ledger().timestamp();
        env.storage().persistent().set(&DataKey::Consultation(session_id), &consultation);
        log!(&env, "Consultation started");
        Ok(())
    }

    pub fn complete_consultation(
        env: &Env,
        session_id: BytesN<32>,
        provider_address: Address,
        recording_hash: BytesN<32>,
        _appointment_id: BytesN<32>,
        quality_score: u32
    ) -> Result<(), TelemedicineError> {
        Self::require_not_paused(env)?;
        provider_address.require_auth();
        let mut consultation: Consultation = env
            .storage()
            .persistent()
            .get(&DataKey::Consultation(session_id.clone()))
            .ok_or(TelemedicineError::ConsultationNotFound)?;
        if consultation.status != ConsultationStatus::Active {
            return Err(TelemedicineError::ConsultationNotActive);
        }
        consultation.status = ConsultationStatus::Completed;
        consultation.end_time = env.ledger().timestamp();
        consultation.recording_hash = recording_hash.clone();
        consultation.quality_score = quality_score;
        env.storage().persistent().set(&DataKey::Consultation(session_id), &consultation);
        log!(&env, "Consultation completed");
        Ok(())
    }

    pub fn get_consultation(
        env: &Env,
        session_id: BytesN<32>
    ) -> Result<Consultation, TelemedicineError> {
        env.storage()
            .persistent()
            .get(&DataKey::Consultation(session_id))
            .ok_or(TelemedicineError::ConsultationNotFound)
    }

    // ============================================================
    // PRESCRIPTION MANAGEMENT
    // ============================================================

    pub fn issue_prescription(
        env: &Env,
        prescription_id: BytesN<32>,
        consultation_id: BytesN<32>,
        patient_id: BytesN<32>,
        provider_id: BytesN<32>,
        provider_address: Address,
        medications: Vec<String>,
        valid_days: u64,
        pharmacy_id: String
    ) -> Result<(), TelemedicineError> {
        Self::require_not_paused(env)?;
        provider_address.require_auth();
        let consultation: Consultation = env
            .storage()
            .persistent()
            .get(&DataKey::Consultation(consultation_id.clone()))
            .ok_or(TelemedicineError::ConsultationNotFound)?;
        if consultation.status != ConsultationStatus::Completed {
            return Err(TelemedicineError::ConsultationNotActive);
        }
        let prescription = Prescription {
            prescription_id: prescription_id.clone(),
            consultation_id,
            patient_id,
            provider_id,
            medications,
            issued_date: env.ledger().timestamp(),
            valid_days,
            pharmacy_id,
            is_active: true,
            cross_border: false,
            jurisdiction: String::from_str(env, "KE"),
        };
        env.storage().persistent().set(&DataKey::Prescription(prescription_id), &prescription);
        Self::increment_platform_stat(env, 3);
        log!(&env, "Prescription issued");
        Ok(())
    }

    pub fn get_prescription(
        env: &Env,
        prescription_id: BytesN<32>
    ) -> Result<Prescription, TelemedicineError> {
        env.storage()
            .persistent()
            .get(&DataKey::Prescription(prescription_id))
            .ok_or(TelemedicineError::PrescriptionNotFound)
    }

    // ============================================================
    // MONITORING SESSIONS
    // ============================================================

    pub fn start_monitoring_session(
        env: &Env,
        session_id: BytesN<32>,
        patient_id: BytesN<32>,
        provider_id: BytesN<32>,
        _duration_hours: u32
    ) -> Result<(), TelemedicineError> {
        Self::require_not_paused(env)?;
        let _: Patient = env
            .storage()
            .persistent()
            .get(&DataKey::Patient(patient_id.clone()))
            .ok_or(TelemedicineError::PatientNotFound)?;
        let _: Provider = env
            .storage()
            .persistent()
            .get(&DataKey::Provider(provider_id.clone()))
            .ok_or(TelemedicineError::ProviderNotFound)?;
        let session = MonitoringSession {
            session_id: session_id.clone(),
            patient_id,
            provider_id,
            start_time: env.ledger().timestamp(),
            end_time: 0,
            is_active: true,
            vital_signs_count: 0,
            alerts_count: 0,
        };
        env.storage().persistent().set(&DataKey::MonitoringSession(session_id), &session);
        log!(&env, "Monitoring session started");
        Ok(())
    }

    pub fn end_monitoring_session(
        env: &Env,
        session_id: BytesN<32>
    ) -> Result<MonitoringSession, TelemedicineError> {
        Self::require_not_paused(env)?;
        let mut session: MonitoringSession = env
            .storage()
            .persistent()
            .get(&DataKey::MonitoringSession(session_id.clone()))
            .ok_or(TelemedicineError::MonitoringSessionNotFound)?;
        session.is_active = false;
        session.end_time = env.ledger().timestamp();
        env.storage().persistent().set(&DataKey::MonitoringSession(session_id), &session.clone());
        log!(&env, "Monitoring session ended");
        Ok(session)
    }

    // ============================================================
    // UTILITY FUNCTIONS
    // ============================================================

    fn require_admin(env: &Env) -> Result<(), TelemedicineError> {
        // Retrieve the stored admin address and require their auth
        let admin: Address = env
            .storage()
            .persistent()
            .get(&DataKey::Admin)
            .ok_or(TelemedicineError::NotAdmin)?;
        admin.require_auth();
        Ok(())
    }

    fn require_not_paused(env: &Env) -> Result<(), TelemedicineError> {
        if env.storage().persistent().get(&DataKey::Paused).unwrap_or(false) {
            return Err(TelemedicineError::ContractPaused);
        }
        Ok(())
    }

    #[allow(dead_code)]
    fn is_valid_jurisdiction(jurisdiction: &str) -> bool {
        matches!(jurisdiction, "US" | "GB" | "CA" | "AU" | "DE" | "FR" | "KE" | "ZA" | "NG" | "IN")
    }

    fn increment_platform_stat(env: &Env, stat_index: usize) {
        let mut stats: (u64, u64, u64, u64, u64, u64) = env
            .storage()
            .persistent()
            .get(&DataKey::PlatformStats)
            .unwrap_or((0, 0, 0, 0, 0, 0));
        match stat_index {
            0 => {
                stats.0 = stats.0.saturating_add(1);
            }
            1 => {
                stats.1 = stats.1.saturating_add(1);
            }
            2 => {
                stats.2 = stats.2.saturating_add(1);
            }
            3 => {
                stats.3 = stats.3.saturating_add(1);
            }
            4 => {
                stats.4 = stats.4.saturating_add(1);
            }
            5 => {
                stats.5 = stats.5.saturating_add(1);
            }
            _ => {}
        }
        env.storage().persistent().set(&DataKey::PlatformStats, &stats);
    }

    pub fn get_platform_stats(env: Env) -> (u64, u64, u64, u64, u64, u64) {
        env.storage().persistent().get(&DataKey::PlatformStats).unwrap_or((0, 0, 0, 0, 0, 0))
    }

    // ============================================================
    // TELEHEALTH INTEGRATION - VIDEO SESSIONS
    // ============================================================

    pub fn create_video_session(
        env: &Env,
        session_id: BytesN<32>,
        consultation_id: BytesN<32>,
        patient_id: BytesN<32>,
        provider_id: BytesN<32>,
        room_id: String,
        video_quality: VideoQuality,
        video_codec: VideoCodec,
        encryption_level: EncryptionLevel,
        recording_enabled: bool
    ) -> Result<(), TelemedicineError> {
        Self::require_not_paused(env)?;

        // Check concurrent session limits
        let tracker: ConcurrentSessionTracker = env
            .storage()
            .persistent()
            .get(&DataKey::ConcurrentSessionTracker)
            .ok_or(TelemedicineError::ConcurrentSessionLimitExceeded)?;

        if tracker.active_sessions >= 10000 {
            return Err(TelemedicineError::ConcurrentSessionLimitExceeded);
        }

        // Verify consultation exists and is scheduled
        let consultation: Consultation = env
            .storage()
            .persistent()
            .get(&DataKey::Consultation(consultation_id.clone()))
            .ok_or(TelemedicineError::ConsultationNotFound)?;

        if consultation.status != ConsultationStatus::Scheduled {
            return Err(TelemedicineError::ConsultationNotScheduled);
        }

        // Verify patient consent
        let patient: Patient = env
            .storage()
            .persistent()
            .get(&DataKey::Patient(patient_id.clone()))
            .ok_or(TelemedicineError::PatientNotFound)?;

        if !patient.video_consent_granted {
            return Err(TelemedicineError::ConsentNotGiven);
        }

        // Check bandwidth requirements
        let required_bandwidth = Self::get_required_bandwidth(video_quality);
        if patient.bandwidth_limit_kbps < required_bandwidth {
            return Err(TelemedicineError::InsufficientBandwidth);
        }

        let video_session = VideoSession {
            session_id: session_id.clone(),
            consultation_id: consultation_id.clone(),
            patient_id,
            provider_id,
            room_id,
            sdp_offer: String::from_str(env, ""),
            sdp_answer: String::from_str(env, ""),
            ice_candidates: Vec::new(env),
            video_quality,
            video_codec,
            encryption_level,
            bandwidth_kbps: required_bandwidth,
            latency_ms: 0,
            packet_loss: 0,
            is_active: false,
            start_time: 0,
            end_time: 0,
            recording_enabled,
            recording_hash: BytesN::from_array(env, &[0u8; 32]),
            hipaa_compliant: encryption_level != EncryptionLevel::Standard,
        };

        env.storage().persistent().set(&DataKey::VideoSession(session_id.clone()), &video_session);

        // Update consultation with video session reference
        let mut updated_consultation = consultation;
        updated_consultation.video_session_id = session_id.clone();
        env.storage()
            .persistent()
            .set(&DataKey::Consultation(consultation_id), &updated_consultation);

        log!(&env, "Video session created for telehealth integration");
        Ok(())
    }

    pub fn start_video_session(
        env: &Env,
        session_id: BytesN<32>,
        sdp_offer: String
    ) -> Result<(), TelemedicineError> {
        Self::require_not_paused(env)?;

        let mut session: VideoSession = env
            .storage()
            .persistent()
            .get(&DataKey::VideoSession(session_id.clone()))
            .ok_or(TelemedicineError::VideoSessionNotFound)?;

        if session.is_active {
            return Err(TelemedicineError::VideoSessionAlreadyActive);
        }

        session.is_active = true;
        session.start_time = env.ledger().timestamp();
        session.sdp_offer = sdp_offer;

        env.storage().persistent().set(&DataKey::VideoSession(session_id), &session);

        // Update concurrent session tracker
        Self::increment_active_sessions(env);

        log!(&env, "Video session started - WebRTC connection established");
        Ok(())
    }

    pub fn update_video_quality(
        env: &Env,
        session_id: BytesN<32>,
        metrics: VideoQualityMetrics
    ) -> Result<(), TelemedicineError> {
        Self::require_not_paused(env)?;

        let session: VideoSession = env
            .storage()
            .persistent()
            .get(&DataKey::VideoSession(session_id.clone()))
            .ok_or(TelemedicineError::VideoSessionNotFound)?;

        if !session.is_active {
            return Err(TelemedicineError::VideoSessionNotActive);
        }

        // Verify 1080p at 30fps requirement
        if
            metrics.resolution_width != 1920 ||
            metrics.resolution_height != 1080 ||
            metrics.frame_rate < 30
        {
            return Err(TelemedicineError::VideoQualityUnsupported);
        }

        // Store quality metrics
        env.storage().persistent().set(&DataKey::VideoQualityMetrics(session_id.clone()), &metrics);

        // Update session with current metrics
        let mut updated_session = session;
        updated_session.bandwidth_kbps = metrics.bitrate_kbps;
        updated_session.latency_ms = metrics.round_trip_time;
        updated_session.packet_loss = if metrics.network_jitter > 100 { 1 } else { 0 };

        env.storage().persistent().set(&DataKey::VideoSession(session_id), &updated_session);

        log!(&env, "Video quality metrics updated - 1080p 30fps verified");
        Ok(())
    }

    pub fn end_video_session(
        env: &Env,
        session_id: BytesN<32>,
        recording_hash: BytesN<32>
    ) -> Result<(), TelemedicineError> {
        Self::require_not_paused(env)?;

        let mut session: VideoSession = env
            .storage()
            .persistent()
            .get(&DataKey::VideoSession(session_id.clone()))
            .ok_or(TelemedicineError::VideoSessionNotFound)?;

        if !session.is_active {
            return Err(TelemedicineError::VideoSessionNotActive);
        }

        session.is_active = false;
        session.end_time = env.ledger().timestamp();
        session.recording_hash = recording_hash;

        env.storage().persistent().set(&DataKey::VideoSession(session_id), &session);

        // Update concurrent session tracker
        Self::decrement_active_sessions(env);

        log!(&env, "Video session ended - recording secured");
        Ok(())
    }

    fn get_required_bandwidth(quality: VideoQuality) -> u32 {
        match quality {
            VideoQuality::Standard => 1000, // 1 Mbps
            VideoQuality::High => 2500, // 2.5 Mbps
            VideoQuality::FullHD => 5000, // 5 Mbps - 1080p 30fps
            VideoQuality::Premium => 8000, // 8 Mbps - 1080p 60fps
            VideoQuality::UltraHD => 25000, // 25 Mbps - 4K
        }
    }

    fn increment_active_sessions(env: &Env) {
        let mut tracker: ConcurrentSessionTracker = env
            .storage()
            .persistent()
            .get(&DataKey::ConcurrentSessionTracker)
            .unwrap_or(ConcurrentSessionTracker {
                total_sessions: 0,
                active_sessions: 0,
                peak_sessions: 0,
                average_duration: 0,
                bandwidth_utilization: 0,
                server_load: 0,
                quality_degradation_count: 0,
                timestamp: env.ledger().timestamp(),
            });

        tracker.active_sessions = tracker.active_sessions.saturating_add(1);
        tracker.total_sessions = tracker.total_sessions.saturating_add(1);

        if tracker.active_sessions > tracker.peak_sessions {
            tracker.peak_sessions = tracker.active_sessions;
        }

        env.storage().persistent().set(&DataKey::ConcurrentSessionTracker, &tracker);
    }

    fn decrement_active_sessions(env: &Env) {
        let mut tracker: ConcurrentSessionTracker = env
            .storage()
            .persistent()
            .get(&DataKey::ConcurrentSessionTracker)
            .unwrap_or(ConcurrentSessionTracker {
                total_sessions: 0,
                active_sessions: 0,
                peak_sessions: 0,
                average_duration: 0,
                bandwidth_utilization: 0,
                server_load: 0,
                quality_degradation_count: 0,
                timestamp: env.ledger().timestamp(),
            });

        tracker.active_sessions = tracker.active_sessions.saturating_sub(1);

        env.storage().persistent().set(&DataKey::ConcurrentSessionTracker, &tracker);
    }

    // ============================================================
    // TELEHEALTH INTEGRATION - VIRTUAL WAITING ROOM
    // ============================================================

    pub fn create_waiting_room(
        env: &Env,
        room_id: BytesN<32>,
        provider_id: BytesN<32>,
        max_capacity: u32
    ) -> Result<(), TelemedicineError> {
        Self::require_not_paused(env)?;

        // Verify provider exists and is active
        let provider: Provider = env
            .storage()
            .persistent()
            .get(&DataKey::Provider(provider_id.clone()))
            .ok_or(TelemedicineError::ProviderNotFound)?;

        if !provider.is_active {
            return Err(TelemedicineError::ProviderNotActive);
        }

        let waiting_room = VirtualWaitingRoom {
            room_id: room_id.clone(),
            provider_id,
            patient_queue: Vec::new(env),
            max_capacity,
            current_count: 0,
            average_wait_time: 0,
            estimated_wait_time: 0,
            status: WaitingRoomStatus::Waiting,
            created_at: env.ledger().timestamp(),
            last_updated: env.ledger().timestamp(),
        };

        env.storage().persistent().set(&DataKey::VirtualWaitingRoom(room_id), &waiting_room);

        log!(&env, "Virtual waiting room created");
        Ok(())
    }

    pub fn join_waiting_room(
        env: &Env,
        room_id: BytesN<32>,
        patient_id: BytesN<32>
    ) -> Result<(), TelemedicineError> {
        Self::require_not_paused(env)?;

        let mut room: VirtualWaitingRoom = env
            .storage()
            .persistent()
            .get(&DataKey::VirtualWaitingRoom(room_id.clone()))
            .ok_or(TelemedicineError::WaitingRoomFull)?;

        if room.current_count >= room.max_capacity {
            return Err(TelemedicineError::WaitingRoomFull);
        }

        // Verify patient exists
        let _: Patient = env
            .storage()
            .persistent()
            .get(&DataKey::Patient(patient_id.clone()))
            .ok_or(TelemedicineError::PatientNotFound)?;

        room.patient_queue.push_back(patient_id);
        room.current_count = room.current_count.saturating_add(1);
        room.last_updated = env.ledger().timestamp();
        room.estimated_wait_time = room.current_count * 15; // 15 mins per patient

        env.storage().persistent().set(&DataKey::VirtualWaitingRoom(room_id), &room);

        log!(&env, "Patient joined virtual waiting room");
        Ok(())
    }

    pub fn leave_waiting_room(
        env: &Env,
        room_id: BytesN<32>,
        patient_id: BytesN<32>
    ) -> Result<(), TelemedicineError> {
        Self::require_not_paused(env)?;

        let mut room: VirtualWaitingRoom = env
            .storage()
            .persistent()
            .get(&DataKey::VirtualWaitingRoom(room_id.clone()))
            .ok_or(TelemedicineError::WaitingRoomFull)?;

        // Find and remove patient from queue
        let mut found = false;
        let mut new_queue = Vec::new(env);
        for queued_patient in room.patient_queue.iter() {
            if queued_patient == patient_id {
                found = true;
            } else {
                new_queue.push_back(queued_patient.clone());
            }
        }

        if !found {
            return Err(TelemedicineError::PatientNotFound);
        }

        room.patient_queue = new_queue;
        room.current_count = room.current_count.saturating_sub(1);
        room.last_updated = env.ledger().timestamp();

        if room.current_count > 0 {
            room.estimated_wait_time = room.current_count * 15;
        } else {
            room.estimated_wait_time = 0;
        }

        env.storage().persistent().set(&DataKey::VirtualWaitingRoom(room_id), &room);

        log!(&env, "Patient left virtual waiting room");
        Ok(())
    }

    // ============================================================
    // TELEHEALTH INTEGRATION - E-SIGN PRESCRIPTIONS
    // ============================================================

    pub fn issue_esign_prescription(
        env: &Env,
        prescription_id: BytesN<32>,
        consultation_id: BytesN<32>,
        patient_id: BytesN<32>,
        provider_id: BytesN<32>,
        provider_address: Address,
        request: ESIGNRequest
    ) -> Result<(), TelemedicineError> {
        Self::require_not_paused(env)?;
        provider_address.require_auth();

        // Verify consultation exists and is completed
        let consultation: Consultation = env
            .storage()
            .persistent()
            .get(&DataKey::Consultation(consultation_id.clone()))
            .ok_or(TelemedicineError::ConsultationNotFound)?;

        if consultation.status != ConsultationStatus::Completed {
            return Err(TelemedicineError::ConsultationNotActive);
        }

        // Verify provider has valid digital signature certificate
        let provider: Provider = env
            .storage()
            .persistent()
            .get(&DataKey::Provider(provider_id.clone()))
            .ok_or(TelemedicineError::ProviderNotFound)?;

        if provider.digital_signature_certificate != request.certificate_id {
            return Err(TelemedicineError::ESIGNInvalidSignature);
        }

        if !provider.hipaa_training_verified {
            return Err(TelemedicineError::HIPAAComplianceFailed);
        }

        let prescription = ESIGNPrescription {
            prescription_id: prescription_id.clone(),
            consultation_id,
            patient_id,
            provider_id,
            medications: request.medications,
            dosage_instructions: request.dosage_instructions,
            issued_date: env.ledger().timestamp(),
            valid_days: request.valid_days,
            pharmacy_id: request.pharmacy_id,
            digital_signature: request.digital_signature,
            certificate_id: request.certificate_id,
            signature_timestamp: env.ledger().timestamp(),
            status: ESIGNStatus::Pending,
            verification_hash: BytesN::from_array(env, &[0u8; 32]),
            jurisdiction: request.jurisdiction,
            cross_border: false,
        };

        env.storage().persistent().set(&DataKey::ESIGNPrescription(prescription_id), &prescription);

        log!(&env, "E-SIGN prescription issued with digital signature");
        Ok(())
    }

    pub fn sign_prescription(
        env: &Env,
        prescription_id: BytesN<32>,
        patient_address: Address
    ) -> Result<(), TelemedicineError> {
        Self::require_not_paused(env)?;
        patient_address.require_auth();

        let mut prescription: ESIGNPrescription = env
            .storage()
            .persistent()
            .get(&DataKey::ESIGNPrescription(prescription_id.clone()))
            .ok_or(TelemedicineError::PrescriptionNotFound)?;

        if prescription.status != ESIGNStatus::Pending {
            return Err(TelemedicineError::ESIGNInvalidSignature);
        }

        prescription.status = ESIGNStatus::Signed;
        prescription.signature_timestamp = env.ledger().timestamp();

        env.storage().persistent().set(&DataKey::ESIGNPrescription(prescription_id), &prescription);

        log!(&env, "Prescription digitally signed by patient");
        Ok(())
    }

    pub fn verify_hipaa_compliance(
        env: &Env,
        session_id: BytesN<32>
    ) -> Result<bool, TelemedicineError> {
        Self::require_not_paused(env)?;

        let session: VideoSession = env
            .storage()
            .persistent()
            .get(&DataKey::VideoSession(session_id))
            .ok_or(TelemedicineError::VideoSessionNotFound)?;

        // Verify HIPAA compliance requirements
        let hipaa_compliant =
            session.hipaa_compliant &&
            session.encryption_level != EncryptionLevel::Standard &&
            session.recording_enabled;

        if hipaa_compliant {
            log!(&env, "HIPAA compliance verified for video session");
        } else {
            log!(&env, "HIPAA compliance check failed");
        }

        Ok(hipaa_compliant)
    }
}
