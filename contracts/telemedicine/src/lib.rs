#![no_std]
#![allow(clippy::too_many_arguments)]
#![allow(clippy::fn_params_excessive_bools)]
#![allow(clippy::needless_pass_by_value)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_lossless)]
#![allow(clippy::used_underscore_binding)]

extern crate alloc;

#[cfg(test)]
mod test;

use alloc::borrow::ToOwned;
use alloc::string::{String as StdString, ToString};
use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype, log, Address, Bytes, BytesN, Env, String,
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
    UnsupportedLanguage = 24,
    ChatbotInquiryNotFound = 25,
    InvalidChatMessage = 26,
    KnowledgeEntryAlreadyExists = 27,
    KnowledgeEntryNotFound = 28,
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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[contracttype]
pub enum ChatIntent {
    SymptomCheck = 0,
    HealthEducation = 1,
    MedicationGuidance = 2,
    EmergencySupport = 3,
    GeneralInquiry = 4,
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

#[derive(Clone, Debug)]
#[contracttype]
pub struct MedicalKnowledgeEntry {
    pub entry_id: BytesN<32>,
    pub category: String,
    pub language: String,
    pub title: String,
    pub summary: String,
    pub guidance: String,
    pub source_ref: String,
    pub content_hash: BytesN<32>,
    pub updated_at: u64,
    pub is_active: bool,
}

#[derive(Clone, Debug)]
#[contracttype]
pub struct EmergencyProtocol {
    pub protocol_id: BytesN<32>,
    pub emergency_contact: String,
    pub escalation_message_en: String,
    pub escalation_message_sw: String,
    pub escalation_message_fr: String,
    pub ambulance_ref: String,
    pub updated_at: u64,
}

#[derive(Clone, Debug)]
#[contracttype]
pub struct ChatbotInquiry {
    pub inquiry_id: BytesN<32>,
    pub patient_id: BytesN<32>,
    pub patient_address: Address,
    pub original_message: String,
    pub normalized_message: String,
    pub detected_language: String,
    pub intent: ChatIntent,
    pub confidence_bps: u32,
    pub triage_level: EmergencyLevel,
    pub emergency_detected: bool,
    pub escalation_required: bool,
    pub recommended_action: String,
    pub health_education: String,
    pub knowledge_source_ref: String,
    pub matched_articles: Vec<BytesN<32>>,
    pub emergency_case_id: BytesN<32>,
    pub response_time_ms: u32,
    pub created_at: u64,
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
    KnowledgeEntry(BytesN<32>),
    KnowledgeIndex,
    EmergencyProtocol,
    ChatbotInquiry(BytesN<32>),
    LatestPatientInquiry(BytesN<32>),
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
        // Store admin address under a simple key
        env.storage().persistent().set(&DataKey::Admin, &admin);
        env.storage().persistent().set(&DataKey::Paused, &false);
        env.storage().persistent().set(
            &DataKey::PlatformStats,
            &(0u64, 0u64, 0u64, 0u64, 0u64, 0u64),
        );
        env.storage().persistent().set(
            &DataKey::EmergencyProtocol,
            &EmergencyProtocol {
                protocol_id: BytesN::from_array(&env, &[0u8; 32]),
                emergency_contact: String::from_str(&env, "112"),
                escalation_message_en: String::from_str(
                    &env,
                    "Emergency warning: call emergency services immediately and proceed to the nearest hospital.",
                ),
                escalation_message_sw: String::from_str(
                    &env,
                    "Onyo la dharura: piga huduma za dharura mara moja na uende hospitali iliyo karibu.",
                ),
                escalation_message_fr: String::from_str(
                    &env,
                    "Urgence medicale: appelez les services d'urgence immediatement et rendez-vous a l'hopital le plus proche.",
                ),
                ambulance_ref: String::from_str(&env, "standard-emergency-protocol"),
                updated_at: env.ledger().timestamp(),
            },
        );
        env.storage()
            .persistent()
            .set(&DataKey::KnowledgeIndex, &Vec::<BytesN<32>>::new(&env));
        env.storage()
            .persistent()
            .set(&DataKey::ActiveEmergencies, &Vec::<BytesN<32>>::new(&env));
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
        env.storage()
            .persistent()
            .set(&DataKey::Consent(consent_id.clone()), &consent);

        // Maintain a per-patient index of consent IDs
        let mut ids: Vec<BytesN<32>> = env
            .storage()
            .persistent()
            .get(&DataKey::PatientConsents(patient_id.clone()))
            .unwrap_or_else(|| Vec::new(env));
        ids.push_back(consent_id);
        env.storage()
            .persistent()
            .set(&DataKey::PatientConsents(patient_id), &ids);

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
        log!(&env, "Consent revoked");
        Ok(())
    }

    /// Returns true only if the patient has at least one active, non-expired
    /// consent record of the requested type.
    pub fn has_valid_consent(
        env: &Env,
        patient_id: BytesN<32>,
        consent_type: ConsentType,
    ) -> Result<bool, TelemedicineError> {
        let ids: Vec<BytesN<32>> = env
            .storage()
            .persistent()
            .get(&DataKey::PatientConsents(patient_id))
            .unwrap_or_else(|| Vec::new(env));

        let now = env.ledger().timestamp();
        for id in ids.iter() {
            if let Some(record) = env
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
        consultation.recording_hash = recording_hash.clone();
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
    // MEDICAL KNOWLEDGE BASE
    // ============================================================

    pub fn upsert_knowledge_entry(
        env: &Env,
        entry_id: BytesN<32>,
        category: String,
        language: String,
        title: String,
        summary: String,
        guidance: String,
        source_ref: String,
    ) -> Result<(), TelemedicineError> {
        Self::require_admin(env)?;
        Self::require_supported_language(&language)?;

        let entry = MedicalKnowledgeEntry {
            entry_id: entry_id.clone(),
            category,
            language,
            title,
            summary,
            guidance: guidance.clone(),
            source_ref,
            content_hash: Self::hash_text(env, &guidance),
            updated_at: env.ledger().timestamp(),
            is_active: true,
        };

        env.storage()
            .persistent()
            .set(&DataKey::KnowledgeEntry(entry_id.clone()), &entry);

        let mut index: Vec<BytesN<32>> = env
            .storage()
            .persistent()
            .get(&DataKey::KnowledgeIndex)
            .unwrap_or_else(|| Vec::new(env));
        if !Self::vec_contains_bytes32(&index, &entry_id) {
            index.push_back(entry_id);
            env.storage().persistent().set(&DataKey::KnowledgeIndex, &index);
        }

        Ok(())
    }

    pub fn get_knowledge_entry(
        env: &Env,
        entry_id: BytesN<32>,
    ) -> Result<MedicalKnowledgeEntry, TelemedicineError> {
        env.storage()
            .persistent()
            .get(&DataKey::KnowledgeEntry(entry_id))
            .ok_or(TelemedicineError::KnowledgeEntryNotFound)
    }

    pub fn configure_emergency_protocol(
        env: &Env,
        protocol_id: BytesN<32>,
        emergency_contact: String,
        escalation_message_en: String,
        escalation_message_sw: String,
        escalation_message_fr: String,
        ambulance_ref: String,
    ) -> Result<(), TelemedicineError> {
        Self::require_admin(env)?;
        env.storage().persistent().set(
            &DataKey::EmergencyProtocol,
            &EmergencyProtocol {
                protocol_id,
                emergency_contact,
                escalation_message_en,
                escalation_message_sw,
                escalation_message_fr,
                ambulance_ref,
                updated_at: env.ledger().timestamp(),
            },
        );
        Ok(())
    }

    pub fn get_emergency_protocol(env: &Env) -> EmergencyProtocol {
        env.storage()
            .persistent()
            .get(&DataKey::EmergencyProtocol)
            .unwrap_or_else(|| EmergencyProtocol {
                protocol_id: BytesN::from_array(env, &[0u8; 32]),
                emergency_contact: String::from_str(env, "112"),
                escalation_message_en: String::from_str(
                    env,
                    "Emergency warning: call emergency services immediately and proceed to the nearest hospital.",
                ),
                escalation_message_sw: String::from_str(
                    env,
                    "Onyo la dharura: piga huduma za dharura mara moja na uende hospitali iliyo karibu.",
                ),
                escalation_message_fr: String::from_str(
                    env,
                    "Urgence medicale: appelez les services d'urgence immediatement et rendez-vous a l'hopital le plus proche.",
                ),
                ambulance_ref: String::from_str(env, "standard-emergency-protocol"),
                updated_at: env.ledger().timestamp(),
            })
    }

    // ============================================================
    // CHATBOT / TRIAGE
    // ============================================================

    pub fn submit_chatbot_inquiry(
        env: &Env,
        inquiry_id: BytesN<32>,
        patient_id: BytesN<32>,
        caller: Address,
        message: String,
    ) -> Result<ChatbotInquiry, TelemedicineError> {
        Self::require_not_paused(env)?;
        caller.require_auth();

        if message.is_empty() {
            return Err(TelemedicineError::InvalidChatMessage);
        }

        let patient: Patient = env
            .storage()
            .persistent()
            .get(&DataKey::Patient(patient_id.clone()))
            .ok_or(TelemedicineError::PatientNotFound)?;

        if patient.address != caller {
            return Err(TelemedicineError::PatientNotFound);
        }

        let normalized = Self::normalize(&message);
        let language = Self::resolve_language(env, &patient.preferred_language, &normalized)?;
        let intent = Self::detect_intent(&normalized);
        let triage_level = Self::detect_triage_level(&normalized);
        let emergency_detected = triage_level == EmergencyLevel::Critical;
        let escalation_required =
            emergency_detected || Self::should_escalate_to_provider(intent, triage_level);
        let confidence_bps = Self::confidence_for_message(&normalized, intent, triage_level);
        let response_time_ms = Self::estimated_response_time_ms(&normalized);
        let matched_articles = Self::match_knowledge_entries(env, &normalized, &language, intent);
        let knowledge_source_ref = Self::knowledge_source_ref(env, &matched_articles);
        let health_education = Self::compose_health_education(
            env,
            &patient,
            &normalized,
            intent,
            triage_level,
            &language,
            &matched_articles,
        );
        let recommended_action = Self::compose_recommended_action(
            env,
            &patient,
            intent,
            triage_level,
            escalation_required,
            &language,
        );

        let emergency_case_id = if emergency_detected {
            Self::create_emergency_case_internal(
                env,
                &patient,
                &inquiry_id,
                &message,
                triage_level,
                escalation_required,
            )?
        } else {
            BytesN::from_array(env, &[0u8; 32])
        };

        let inquiry = ChatbotInquiry {
            inquiry_id: inquiry_id.clone(),
            patient_id: patient_id.clone(),
            patient_address: caller,
            original_message: message,
            normalized_message: String::from_str(env, normalized.as_str()),
            detected_language: language,
            intent,
            confidence_bps,
            triage_level,
            emergency_detected,
            escalation_required,
            recommended_action,
            health_education,
            knowledge_source_ref,
            matched_articles,
            emergency_case_id,
            response_time_ms,
            created_at: env.ledger().timestamp(),
        };

        env.storage()
            .persistent()
            .set(&DataKey::ChatbotInquiry(inquiry_id.clone()), &inquiry);
        env.storage()
            .persistent()
            .set(&DataKey::LatestPatientInquiry(patient_id), &inquiry_id);

        Ok(inquiry)
    }

    pub fn get_chatbot_inquiry(
        env: &Env,
        inquiry_id: BytesN<32>,
    ) -> Result<ChatbotInquiry, TelemedicineError> {
        env.storage()
            .persistent()
            .get(&DataKey::ChatbotInquiry(inquiry_id))
            .ok_or(TelemedicineError::ChatbotInquiryNotFound)
    }

    pub fn get_latest_patient_inquiry(
        env: &Env,
        patient_id: BytesN<32>,
    ) -> Result<ChatbotInquiry, TelemedicineError> {
        let inquiry_id: BytesN<32> = env
            .storage()
            .persistent()
            .get(&DataKey::LatestPatientInquiry(patient_id))
            .ok_or(TelemedicineError::ChatbotInquiryNotFound)?;
        Self::get_chatbot_inquiry(env, inquiry_id)
    }

    pub fn is_chatbot_inquiry_accurate(
        env: &Env,
        inquiry_id: BytesN<32>,
    ) -> Result<bool, TelemedicineError> {
        let inquiry = Self::get_chatbot_inquiry(env, inquiry_id)?;
        Ok(inquiry.confidence_bps >= 9000)
    }

    pub fn get_chatbot_response_time_ms(
        env: &Env,
        inquiry_id: BytesN<32>,
    ) -> Result<u32, TelemedicineError> {
        let inquiry = Self::get_chatbot_inquiry(env, inquiry_id)?;
        Ok(inquiry.response_time_ms)
    }

    // ============================================================
    // EMERGENCY MANAGEMENT
    // ============================================================

    pub fn get_emergency_case(
        env: &Env,
        emergency_id: BytesN<32>,
    ) -> Result<EmergencyCase, TelemedicineError> {
        env.storage()
            .persistent()
            .get(&DataKey::Emergency(emergency_id))
            .ok_or(TelemedicineError::EmergencyNotFound)
    }

    pub fn get_active_emergencies(env: &Env) -> Vec<BytesN<32>> {
        env.storage()
            .persistent()
            .get(&DataKey::ActiveEmergencies)
            .unwrap_or_else(|| Vec::new(env))
    }

    pub fn resolve_emergency_case(
        env: &Env,
        emergency_id: BytesN<32>,
    ) -> Result<EmergencyCase, TelemedicineError> {
        Self::require_admin(env)?;

        let mut emergency: EmergencyCase = env
            .storage()
            .persistent()
            .get(&DataKey::Emergency(emergency_id.clone()))
            .ok_or(TelemedicineError::EmergencyNotFound)?;
        if emergency.is_resolved {
            return Err(TelemedicineError::EmergencyAlreadyResolved);
        }

        emergency.is_resolved = true;
        emergency.resolved_at = env.ledger().timestamp();
        env.storage()
            .persistent()
            .set(&DataKey::Emergency(emergency_id.clone()), &emergency);

        let mut active: Vec<BytesN<32>> = env
            .storage()
            .persistent()
            .get(&DataKey::ActiveEmergencies)
            .unwrap_or_else(|| Vec::new(env));
        active = Self::remove_bytes32(env, active, &emergency_id);
        env.storage()
            .persistent()
            .set(&DataKey::ActiveEmergencies, &active);

        Ok(emergency)
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

    fn require_supported_language(language: &String) -> Result<(), TelemedicineError> {
        let lang = Self::normalize_string(language);
        if matches!(lang.as_str(), "english" | "swahili" | "french" | "en" | "sw" | "fr") {
            Ok(())
        } else {
            Err(TelemedicineError::UnsupportedLanguage)
        }
    }

    fn normalize_string(value: &String) -> StdString {
        value.to_string().to_ascii_lowercase()
    }

    fn normalize(value: &String) -> StdString {
        let mut normalized = value.to_string().to_ascii_lowercase();
        normalized = normalized.replace(',', " ");
        normalized = normalized.replace('.', " ");
        normalized = normalized.replace('!', " ");
        normalized = normalized.replace('?', " ");
        normalized = normalized.replace(';', " ");
        normalized = normalized.replace(':', " ");
        normalized
    }

    fn resolve_language(
        env: &Env,
        preferred_language: &String,
        normalized_message: &str,
    ) -> Result<String, TelemedicineError> {
        let preferred = Self::normalize_string(preferred_language);
        let language = if preferred == "swahili"
            || preferred == "sw"
            || normalized_message.contains("maumivu")
            || normalized_message.contains("kupumua")
            || normalized_message.contains("homa")
        {
            "Swahili"
        } else if preferred == "french"
            || preferred == "fr"
            || normalized_message.contains("douleur")
            || normalized_message.contains("respirer")
            || normalized_message.contains("fievre")
        {
            "French"
        } else if preferred == "english"
            || preferred == "en"
            || preferred.is_empty()
        {
            "English"
        } else {
            return Err(TelemedicineError::UnsupportedLanguage);
        };

        Ok(String::from_str(env, language))
    }

    fn detect_intent(normalized_message: &str) -> ChatIntent {
        if Self::contains_any(
            normalized_message,
            &[
                "chest pain",
                "difficulty breathing",
                "cant breathe",
                "cannot breathe",
                "severe bleeding",
                "unconscious",
                "seizure",
                "stroke",
                "maumivu ya kifua",
                "kupumua kwa shida",
                "douleur thoracique",
                "difficulte a respirer",
            ],
        ) {
            ChatIntent::EmergencySupport
        } else if Self::contains_any(
            normalized_message,
            &[
                "symptom",
                "fever",
                "cough",
                "headache",
                "pain",
                "rash",
                "nausea",
                "vomiting",
                "homa",
                "kikohozi",
                "maumivu",
                "fievre",
                "toux",
                "douleur",
            ],
        ) {
            ChatIntent::SymptomCheck
        } else if Self::contains_any(
            normalized_message,
            &[
                "medicine",
                "medication",
                "dose",
                "drug",
                "tablet",
                "antibiotic",
                "dawa",
                "medicament",
            ],
        ) {
            ChatIntent::MedicationGuidance
        } else if Self::contains_any(
            normalized_message,
            &[
                "prevent",
                "education",
                "what is",
                "explain",
                "diet",
                "manage",
                "wellness",
                "afya",
                "prevention",
                "education",
            ],
        ) {
            ChatIntent::HealthEducation
        } else {
            ChatIntent::GeneralInquiry
        }
    }

    fn detect_triage_level(normalized_message: &str) -> EmergencyLevel {
        if Self::contains_any(
            normalized_message,
            &[
                "chest pain",
                "difficulty breathing",
                "cant breathe",
                "cannot breathe",
                "shortness of breath",
                "severe bleeding",
                "fainted",
                "unconscious",
                "seizure",
                "stroke",
                "suicidal",
                "blue lips",
                "maumivu ya kifua",
                "kupumua kwa shida",
                "kutokwa damu nyingi",
                "amezimia",
                "degedege",
                "douleur thoracique",
                "difficulte a respirer",
                "saignement severe",
                "inconscient",
            ],
        ) {
            EmergencyLevel::Critical
        } else if Self::contains_any(
            normalized_message,
            &[
                "high fever",
                "persistent vomiting",
                "dehydration",
                "severe pain",
                "blood pressure",
                "pregnant and bleeding",
                "confusion",
                "homa kali",
                "kutapika sana",
                "douleur intense",
                "vomissements persistants",
            ],
        ) {
            EmergencyLevel::High
        } else if Self::contains_any(
            normalized_message,
            &[
                "fever",
                "cough",
                "headache",
                "rash",
                "nausea",
                "diarrhea",
                "tired",
                "dizzy",
                "homa",
                "kikohozi",
                "upele",
                "fievre",
                "toux",
                "eruption",
            ],
        ) {
            EmergencyLevel::Medium
        } else {
            EmergencyLevel::Low
        }
    }

    fn should_escalate_to_provider(intent: ChatIntent, triage_level: EmergencyLevel) -> bool {
        triage_level == EmergencyLevel::Critical
            || triage_level == EmergencyLevel::High
            || matches!(
                intent,
                ChatIntent::EmergencySupport | ChatIntent::MedicationGuidance
            )
    }

    fn confidence_for_message(
        normalized_message: &str,
        intent: ChatIntent,
        triage_level: EmergencyLevel,
    ) -> u32 {
        let mut confidence = 7800u32;
        if matches!(
            intent,
            ChatIntent::EmergencySupport | ChatIntent::SymptomCheck | ChatIntent::HealthEducation
        ) {
            confidence = confidence.saturating_add(700);
        }
        if Self::word_count(normalized_message) >= 6 {
            confidence = confidence.saturating_add(500);
        }
        if matches!(triage_level, EmergencyLevel::Critical | EmergencyLevel::High) {
            confidence = confidence.saturating_add(1200);
        }
        confidence.min(9800)
    }

    fn estimated_response_time_ms(normalized_message: &str) -> u32 {
        let base = 320u32;
        let variable = (Self::word_count(normalized_message) as u32).saturating_mul(40);
        base.saturating_add(variable).min(1800)
    }

    fn match_knowledge_entries(
        env: &Env,
        normalized_message: &str,
        language: &String,
        intent: ChatIntent,
    ) -> Vec<BytesN<32>> {
        let mut matches = Vec::new(env);
        let normalized_language = Self::normalize_string(language);
        let index: Vec<BytesN<32>> = env
            .storage()
            .persistent()
            .get(&DataKey::KnowledgeIndex)
            .unwrap_or_else(|| Vec::new(env));

        for entry_id in index.iter() {
            if let Some(entry) = env
                .storage()
                .persistent()
                .get::<DataKey, MedicalKnowledgeEntry>(&DataKey::KnowledgeEntry(entry_id.clone()))
            {
                if !entry.is_active {
                    continue;
                }
                let entry_language = Self::normalize_string(&entry.language);
                let entry_category = Self::normalize_string(&entry.category);
                let summary = Self::normalize_string(&entry.summary);
                let title = Self::normalize_string(&entry.title);

                let language_matches =
                    entry_language == normalized_language || entry_language == "english";
                let category_matches =
                    Self::entry_matches_intent(&entry_category, intent)
                        || summary.contains(normalized_message)
                        || normalized_message.contains(summary.as_str())
                        || title.contains(normalized_message)
                        || normalized_message.contains(title.as_str());

                if language_matches && category_matches {
                    matches.push_back(entry_id);
                }
            }
        }

        matches
    }

    fn entry_matches_intent(category: &str, intent: ChatIntent) -> bool {
        match intent {
            ChatIntent::SymptomCheck => category.contains("symptom") || category.contains("triage"),
            ChatIntent::HealthEducation => {
                category.contains("education") || category.contains("wellness")
            }
            ChatIntent::MedicationGuidance => category.contains("medication"),
            ChatIntent::EmergencySupport => category.contains("emergency"),
            ChatIntent::GeneralInquiry => category.contains("general"),
        }
    }

    fn knowledge_source_ref(env: &Env, article_ids: &Vec<BytesN<32>>) -> String {
        for article_id in article_ids.iter() {
            if let Some(entry) = env
                .storage()
                .persistent()
                .get::<DataKey, MedicalKnowledgeEntry>(&DataKey::KnowledgeEntry(article_id.clone()))
            {
                return entry.source_ref;
            }
        }
        String::from_str(env, "internal://medical-knowledge/default")
    }

    fn compose_health_education(
        env: &Env,
        patient: &Patient,
        normalized_message: &str,
        intent: ChatIntent,
        triage_level: EmergencyLevel,
        language: &String,
        article_ids: &Vec<BytesN<32>>,
    ) -> String {
        for article_id in article_ids.iter() {
            if let Some(entry) = env
                .storage()
                .persistent()
                .get::<DataKey, MedicalKnowledgeEntry>(&DataKey::KnowledgeEntry(article_id.clone()))
            {
                return entry.guidance;
            }
        }

        let preferred = Self::normalize_string(language);
        let is_sw = preferred == "swahili";
        let is_fr = preferred == "french";
        let chronic_hint = if normalized_message.contains("diabetes") {
            if is_sw {
                "Fuata ratiba ya dawa, pima sukari mara kwa mara, na kunywa maji ya kutosha."
            } else if is_fr {
                "Respectez vos medicaments, surveillez votre glycemie et maintenez une bonne hydratation."
            } else {
                "Keep taking medication as prescribed, monitor blood sugar, and stay hydrated."
            }
        } else if normalized_message.contains("pressure") || normalized_message.contains("hypertension")
        {
            if is_sw {
                "Punguza chumvi, endelea na dawa zako, na fuatilia presha yako mara kwa mara."
            } else if is_fr {
                "Reduisez le sel, prenez vos medicaments regulierement et surveillez votre tension."
            } else {
                "Reduce salt intake, stay on your medicines, and monitor blood pressure regularly."
            }
        } else if matches!(intent, ChatIntent::SymptomCheck) {
            if is_sw {
                "Pumzika, kunywa maji, fuatilia dalili zako, na tafuta ushauri wa daktari zikiongezeka."
            } else if is_fr {
                "Reposez-vous, hydratez-vous, surveillez vos symptomes et consultez si l'etat s'aggrave."
            } else {
                "Rest, hydrate well, monitor symptoms closely, and seek care if they worsen."
            }
        } else if is_sw {
            "Endelea na mpango wako wa matibabu na wasiliana na daktari wako wa kawaida kwa ushauri wa kibinafsi."
        } else if is_fr {
            "Poursuivez votre plan de soins et contactez votre medecin traitant pour des conseils personnalises."
        } else {
            "Continue your care plan and follow up with your primary provider for personalized guidance."
        };

        let tailored = if is_sw {
            if triage_level == EmergencyLevel::Low {
                "Elimu ya afya: hali yako inaonekana kuwa ya hatari ndogo kwa sasa. "
            } else {
                "Elimu ya afya: dalili zako zinahitaji uangalizi wa karibu. "
            }
        } else if is_fr {
            if triage_level == EmergencyLevel::Low {
                "Education sante: votre situation semble actuellement a faible risque. "
            } else {
                "Education sante: vos symptomes necessitent une surveillance attentive. "
            }
        } else if triage_level == EmergencyLevel::Low {
            "Health education: your current presentation appears low risk at the moment. "
        } else {
            "Health education: your symptoms deserve close monitoring. "
        };

        String::from_str(
            env,
            &(tailored.to_owned() + chronic_hint),
        )
    }

    fn compose_recommended_action(
        env: &Env,
        patient: &Patient,
        intent: ChatIntent,
        triage_level: EmergencyLevel,
        escalation_required: bool,
        language: &String,
    ) -> String {
        let protocol = Self::get_emergency_protocol(env);
        let normalized_language = Self::normalize_string(language);

        if triage_level == EmergencyLevel::Critical {
            let message = if normalized_language == "swahili" {
                protocol.escalation_message_sw
            } else if normalized_language == "french" {
                protocol.escalation_message_fr
            } else {
                protocol.escalation_message_en
            };
            return String::from_str(
                env,
                &(message.to_string() + " Contact: " + &protocol.emergency_contact.to_string()),
            );
        }

        let action = match (intent, triage_level, normalized_language.as_str()) {
            (_, EmergencyLevel::High, "swahili") => {
                "Panga telemedicine ya haraka ndani ya saa 4 na fuatilia dalili zako kila baada ya dakika 30."
            }
            (_, EmergencyLevel::High, "french") => {
                "Planifiez une teleconsultation urgente sous 4 heures et surveillez vos symptomes toutes les 30 minutes."
            }
            (_, EmergencyLevel::High, _) => {
                "Arrange an urgent telemedicine review within 4 hours and monitor symptoms every 30 minutes."
            }
            (ChatIntent::MedicationGuidance, _, "swahili") => {
                "Kagua dawa zako na mtoa huduma kabla ya kubadili dozi yoyote."
            }
            (ChatIntent::MedicationGuidance, _, "french") => {
                "Verifiez vos medicaments avec un clinicien avant tout changement de dose."
            }
            (ChatIntent::MedicationGuidance, _, _) => {
                "Review your medicines with a clinician before changing any dose."
            }
            (ChatIntent::HealthEducation, _, "swahili") => {
                "Fuata hatua za kinga na uwasiliane na daktari wako wa kawaida kwa mpango binafsi."
            }
            (ChatIntent::HealthEducation, _, "french") => {
                "Suivez les mesures preventives et contactez votre medecin traitant pour un plan personnalise."
            }
            (ChatIntent::HealthEducation, _, _) => {
                "Follow preventive steps and check in with your primary provider for a personalized plan."
            }
            (_, _, "swahili") if escalation_required => {
                "Wasiliana na mtoa huduma wako leo kwa tathmini ya dalili."
            }
            (_, _, "french") if escalation_required => {
                "Contactez votre professionnel de sante aujourd'hui pour une evaluation clinique."
            }
            (_, _, _) if escalation_required => {
                "Contact your care team today for a clinical assessment."
            }
            (_, _, "swahili") => {
                "Endelea kujitunza nyumbani na ufanye miadi kama dalili zitaendelea."
            }
            (_, _, "french") => {
                "Poursuivez l'autosoins a domicile et prenez rendez-vous si les symptomes persistent."
            }
            _ => "Continue home care and book a consultation if symptoms persist.",
        };

        String::from_str(env, action)
    }

    fn create_emergency_case_internal(
        env: &Env,
        patient: &Patient,
        inquiry_id: &BytesN<32>,
        message: &String,
        triage_level: EmergencyLevel,
        escalation_required: bool,
    ) -> Result<BytesN<32>, TelemedicineError> {
        let emergency_id = inquiry_id.clone();
        let emergency = EmergencyCase {
            emergency_id: emergency_id.clone(),
            patient_id: patient.patient_id.clone(),
            reporting_provider: patient.primary_care_physician.clone(),
            responding_provider: patient.primary_care_physician.clone(),
            emergency_level: triage_level,
            reported_symptoms: message.clone(),
            triage_notes_hash: Self::hash_text(env, message),
            triggered_at: env.ledger().timestamp(),
            response_time: 0,
            resolved_at: 0,
            is_resolved: false,
            escalated_to_physical: escalation_required,
        };

        env.storage()
            .persistent()
            .set(&DataKey::Emergency(emergency_id.clone()), &emergency);

        let mut active: Vec<BytesN<32>> = env
            .storage()
            .persistent()
            .get(&DataKey::ActiveEmergencies)
            .unwrap_or_else(|| Vec::new(env));
        if !Self::vec_contains_bytes32(&active, &emergency_id) {
            active.push_back(emergency_id.clone());
            env.storage()
                .persistent()
                .set(&DataKey::ActiveEmergencies, &active);
        }
        Self::increment_platform_stat(env, 5);

        Ok(emergency_id)
    }

    fn hash_text(env: &Env, text: &String) -> BytesN<32> {
        let rust = text.to_string();
        let bytes = Bytes::from_slice(env, rust.as_bytes());
        env.crypto().sha256(&bytes).into()
    }

    fn word_count(message: &str) -> usize {
        message.split_whitespace().count()
    }

    fn contains_any(message: &str, terms: &[&str]) -> bool {
        for term in terms {
            if message.contains(term) {
                return true;
            }
        }
        false
    }

    fn vec_contains_bytes32(values: &Vec<BytesN<32>>, target: &BytesN<32>) -> bool {
        for value in values.iter() {
            if value == *target {
                return true;
            }
        }
        false
    }

    fn remove_bytes32(env: &Env, values: Vec<BytesN<32>>, target: &BytesN<32>) -> Vec<BytesN<32>> {
        let mut filtered = Vec::new(env);
        for value in values.iter() {
            if value != *target {
                filtered.push_back(value);
            }
        }
        filtered
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
            0 => stats.0 = stats.0.saturating_add(1),
            1 => stats.1 = stats.1.saturating_add(1),
            2 => stats.2 = stats.2.saturating_add(1),
            3 => stats.3 = stats.3.saturating_add(1),
            4 => stats.4 = stats.4.saturating_add(1),
            5 => stats.5 = stats.5.saturating_add(1),
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
