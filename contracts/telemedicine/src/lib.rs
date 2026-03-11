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
    contract, contracterror, contractimpl, contracttype, log, Address, BytesN, Env, String, Vec,
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
pub enum QualityRating {
    Poor = 0,
    Fair = 1,
    Good = 2,
    VeryGood = 3,
    Excellent = 4,
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
    pub technical_quality: QualityRating,
    pub clinical_quality: QualityRating,
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
    Admin, // stores the admin Address
    Paused,
    Provider(BytesN<32>),
    Patient(BytesN<32>),
    Consent(BytesN<32>),
    PatientConsent(BytesN<32>, u32), // (patient_id_hash, consent_type as u32)
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
}

// ============================================================
// CONTRACT IMPLEMENTATION
// ============================================================

#[contract]
pub struct TelemedicineContract;

#[contractimpl]
impl TelemedicineContract {
    // ============================================================
    // ADMIN FUNCTIONS
    // ============================================================

    pub fn initialize(env: Env, admin: Address) -> Result<(), TelemedicineError> {
        if env.storage().persistent().has(&DataKey::Paused) {
            return Err(TelemedicineError::NotPaused);
        }

        // Store the admin address so require_admin can verify it
        env.storage().persistent().set(&DataKey::Admin, &admin);
        env.storage().persistent().set(&DataKey::Paused, &false);
        env.storage().persistent().set(
            &DataKey::PlatformStats,
            &(0u64, 0u64, 0u64, 0u64, 0u64, 0u64),
        );

        log!(&env, "Telemedicine contract initialized");
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
    ) -> Result<(), TelemedicineError> {
        Self::require_not_paused(env)?;

        if env
            .storage()
            .persistent()
            .has(&DataKey::Provider(provider_id.clone()))
        {
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
        };

        env.storage()
            .persistent()
            .set(&DataKey::Provider(provider_id), &provider);
        Self::increment_platform_stat(env, 0);

        log!(&env, "Provider registered");
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
        provider_id: BytesN<32>,
    ) -> Result<(), TelemedicineError> {
        Self::require_admin(env)?;

        let mut provider: Provider = env
            .storage()
            .persistent()
            .get(&DataKey::Provider(provider_id.clone()))
            .ok_or(TelemedicineError::ProviderNotFound)?;

        provider.is_active = false;
        env.storage()
            .persistent()
            .set(&DataKey::Provider(provider_id), &provider);

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
    ) -> Result<(), TelemedicineError> {
        Self::require_not_paused(env)?;

        if env
            .storage()
            .persistent()
            .has(&DataKey::Patient(patient_id.clone()))
        {
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
        };

        env.storage()
            .persistent()
            .set(&DataKey::Patient(patient_id), &patient);
        Self::increment_platform_stat(env, 1);

        log!(&env, "Patient registered");
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
        expiry: Option<u64>,
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

        // Store by consent_id for revocation
        env.storage()
            .persistent()
            .set(&DataKey::Consent(consent_id), &consent);

        // Also store a lookup by (patient_id, consent_type) for has_valid_consent
        let type_index = consent_type as u32;
        env.storage()
            .persistent()
            .set(&DataKey::PatientConsent(patient_id, type_index), &true);

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
        env.storage()
            .persistent()
            .set(&DataKey::Consent(consent_id), &consent);

        // Remove the patient-consent lookup so has_valid_consent returns false
        let type_index = consent.consent_type as u32;
        env.storage()
            .persistent()
            .remove(&DataKey::PatientConsent(consent.patient_id, type_index));

        log!(&env, "Consent revoked");
        Ok(())
    }

    pub fn has_valid_consent(
        env: &Env,
        patient_id: BytesN<32>,
        consent_type: ConsentType,
    ) -> Result<bool, TelemedicineError> {
        let type_index = consent_type as u32;
        let has = env
            .storage()
            .persistent()
            .get(&DataKey::PatientConsent(patient_id, type_index))
            .unwrap_or(false);
        Ok(has)
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
        _appointment_id: BytesN<32>,
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
        };

        env.storage()
            .persistent()
            .set(&DataKey::Consultation(session_id), &consultation);
        Self::increment_platform_stat(env, 2);

        log!(&env, "Consultation scheduled");
        Ok(())
    }

    pub fn start_consultation(
        env: &Env,
        session_id: BytesN<32>,
        caller: Address,
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

        env.storage()
            .persistent()
            .set(&DataKey::Consultation(session_id), &consultation);

        log!(&env, "Consultation started");
        Ok(())
    }

    pub fn complete_consultation(
        env: &Env,
        session_id: BytesN<32>,
        provider_address: Address,
        recording_hash: BytesN<32>,
        _appointment_id: BytesN<32>,
        quality_score: u32,
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
        consultation.recording_hash = recording_hash;
        consultation.quality_score = quality_score;

        env.storage()
            .persistent()
            .set(&DataKey::Consultation(session_id), &consultation);

        log!(&env, "Consultation completed");
        Ok(())
    }

    pub fn get_consultation(
        env: &Env,
        session_id: BytesN<32>,
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
        pharmacy_id: String,
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

        env.storage()
            .persistent()
            .set(&DataKey::Prescription(prescription_id), &prescription);
        Self::increment_platform_stat(env, 3);

        log!(&env, "Prescription issued");
        Ok(())
    }

    pub fn get_prescription(
        env: &Env,
        prescription_id: BytesN<32>,
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
        _duration_hours: u32,
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

        env.storage()
            .persistent()
            .set(&DataKey::MonitoringSession(session_id), &session);

        log!(&env, "Monitoring session started");
        Ok(())
    }

    pub fn end_monitoring_session(
        env: &Env,
        session_id: BytesN<32>,
    ) -> Result<MonitoringSession, TelemedicineError> {
        Self::require_not_paused(env)?;

        let mut session: MonitoringSession = env
            .storage()
            .persistent()
            .get(&DataKey::MonitoringSession(session_id.clone()))
            .ok_or(TelemedicineError::MonitoringSessionNotFound)?;

        session.is_active = false;
        session.end_time = env.ledger().timestamp();

        env.storage()
            .persistent()
            .set(&DataKey::MonitoringSession(session_id), &session.clone());

        log!(&env, "Monitoring session ended");
        Ok(session)
    }

    // ============================================================
    // UTILITY FUNCTIONS
    // ============================================================

    /// Verify that the stored admin address has authorized this call.
    fn require_admin(env: &Env) -> Result<(), TelemedicineError> {
        let admin: Address = env
            .storage()
            .persistent()
            .get(&DataKey::Admin)
            .ok_or(TelemedicineError::NotAdmin)?;
        admin.require_auth();
        Ok(())
    }

    fn require_not_paused(env: &Env) -> Result<(), TelemedicineError> {
        if env
            .storage()
            .persistent()
            .get(&DataKey::Paused)
            .unwrap_or(false)
        {
            return Err(TelemedicineError::ContractPaused);
        }
        Ok(())
    }

    #[allow(dead_code)]
    fn is_valid_jurisdiction(jurisdiction: &str) -> bool {
        matches!(
            jurisdiction,
            "US" | "GB" | "CA" | "AU" | "DE" | "FR" | "KE" | "ZA" | "NG" | "IN"
        )
    }

    fn increment_platform_stat(env: &Env, stat_index: usize) {
        let mut stats: (u64, u64, u64, u64, u64, u64) = env
            .storage()
            .persistent()
            .get(&DataKey::PlatformStats)
            .unwrap_or((0, 0, 0, 0, 0, 0));

        match stat_index {
            0 => stats.0 += 1,
            1 => stats.1 += 1,
            2 => stats.2 += 1,
            3 => stats.3 += 1,
            4 => stats.4 += 1,
            5 => stats.5 += 1,
            _ => {}
        }

        env.storage()
            .persistent()
            .set(&DataKey::PlatformStats, &stats);
    }

    pub fn get_platform_stats(env: Env) -> (u64, u64, u64, u64, u64, u64) {
        env.storage()
            .persistent()
            .get(&DataKey::PlatformStats)
            .unwrap_or((0, 0, 0, 0, 0, 0))
    }
}
