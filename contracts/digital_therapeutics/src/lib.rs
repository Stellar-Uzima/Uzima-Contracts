#![no_std]

#[cfg(test)]
mod test;

use soroban_sdk::symbol_short;
use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype, vec, Address, BytesN, Env, Map, String,
    Symbol, Vec,
};

// ==================== Digital Therapeutics Types ====================

/// Therapeutic Category
#[derive(Clone, Copy, PartialEq, Eq)]
#[contracttype]
pub enum TherapeuticCategory {
    MentalHealth,
    ChronicDisease,
    Wellness,
    Rehabilitation,
    PreventiveCare,
    AddictionTreatment,
    PainManagement,
    SleepHealth,
    Nutrition,
    Fitness,
    CognitiveTraining,
    Respiratory,
}

/// Therapeutic Status
#[derive(Clone, Copy, PartialEq, Eq)]
#[contracttype]
pub enum TherapeuticStatus {
    Active,
    Paused,
    Completed,
    Discontinued,
    Suspended,
}

/// Clinical Evidence Level
#[derive(Clone, Copy, PartialEq, Eq)]
#[contracttype]
pub enum EvidenceLevel {
    Preliminary,
    PilotStudy,
    RandomizedControlledTrial,
    MetaAnalysis,
    FDAApproved,
    CEApproved,
}

/// Data Privacy Level
#[derive(Clone, Copy, PartialEq, Eq)]
#[contracttype]
pub enum PrivacyLevel {
    Basic,
    Standard,
    Enhanced,
    Maximum,
}

/// Digital Therapeutic
#[derive(Clone)]
#[contracttype]
pub struct DigitalTherapeutic {
    pub therapeutic_id: u64,
    pub name: String,
    pub description: String,
    pub category: TherapeuticCategory,
    pub version: String,
    pub developer: Address,
    pub fda_clearance: bool,
    pub ce_mark: bool,
    pub clinical_evidence_level: EvidenceLevel,
    pub target_conditions: Vec<String>, // ICD-10 codes
    pub age_range: (u8, u8),            // (min_age, max_age)
    pub languages: Vec<String>,
    pub platforms: Vec<String>, // "iOS", "Android", "Web", "Desktop"
    pub integration_apis: Vec<String>,
    pub data_types_collected: Vec<String>,
    pub privacy_level: PrivacyLevel,
    pub encryption_standard: String,
    pub hipaa_compliant: bool,
    pub gdpr_compliant: bool,
    pub subscription_required: bool,
    pub prescription_required: bool,
    pub clinical_supervision_required: bool,
    pub efficacy_metrics: Vec<String>,
    pub safety_monitoring: bool,
    pub adverse_event_reporting: bool,
    pub interoperability_standards: Vec<String>,
    pub created_at: u64,
    pub updated_at: u64,
    pub status: String, // "active", "deprecated", "under_review"
}

/// Patient Prescription
#[derive(Clone)]
#[contracttype]
pub struct PatientPrescription {
    pub prescription_id: u64,
    pub patient: Address,
    pub provider: Address,
    pub therapeutic_id: u64,
    pub prescription_date: u64,
    pub start_date: u64,
    pub end_date: u64,
    pub dosage_instructions: String,
    pub frequency: String, // "daily", "weekly", "as_needed"
    pub duration_weeks: u16,
    pub monitoring_required: bool,
    pub progress_tracking: bool,
    pub data_sharing_consent: bool,
    pub caregiver_access: bool,
    pub emergency_contact: Option<Address>,
    pub contraindications: Vec<String>,
    pub side_effects_monitoring: Vec<String>,
    pub progress_goals: Vec<String>,
    pub custom_parameters: Map<String, String>,
    pub status: TherapeuticStatus,
    pub adherence_score: u8, // 0-100
    pub last_activity: u64,
    pub consent_token_id: u64,
}

/// Therapy Session
#[derive(Clone)]
#[contracttype]
pub struct TherapySession {
    pub session_id: u64,
    pub prescription_id: u64,
    pub patient: Address,
    pub session_type: String, // "guided", "self_directed", "group"
    pub start_time: u64,
    pub end_time: u64,
    pub duration_minutes: u32,
    pub completion_rate: u8, // 0-100
    pub exercises_completed: Vec<String>,
    pub metrics_collected: Vec<TherapyMetric>,
    pub patient_feedback: u8,  // 1-5 rating
    pub difficulty_rating: u8, // 1-5 rating
    pub notes: String,
    pub adverse_events: Vec<String>,
    pub technical_issues: Vec<String>,
    pub data_quality_score: u8,         // 0-100
    pub engagement_score: u8,           // 0-100
    pub therapeutic_alliance_score: u8, // 0-100
}

/// Therapy Metric
#[derive(Clone)]
#[contracttype]
pub struct TherapyMetric {
    pub metric_name: String,
    pub value: f32,
    pub unit: String,
    pub timestamp: u64,
    pub category: String, // "clinical", "behavioral", "engagement", "technical"
    pub data_source: String, // "self_report", "sensor", "assessment", "biometric"
    pub confidence_score: u8, // 0-100
    pub baseline_comparison: Option<f32>,
    pub clinical_significance: String, // "improved", "stable", "declined"
}

/// Progress Report
#[derive(Clone)]
#[contracttype]
pub struct ProgressReport {
    pub report_id: u64,
    pub prescription_id: u64,
    pub patient: Address,
    pub provider: Address,
    pub report_period_start: u64,
    pub report_period_end: u64,
    pub total_sessions: u32,
    pub completed_sessions: u32,
    pub adherence_rate: u8,
    pub clinical_outcomes: Vec<ClinicalOutcome>,
    pub behavioral_outcomes: Vec<BehavioralOutcome>,
    pub engagement_metrics: EngagementMetrics,
    pub safety_summary: SafetySummary,
    pub efficacy_assessment: String, // "effective", "partially_effective", "ineffective"
    pub recommendations: Vec<String>,
    pub next_steps: Vec<String>,
    pub report_generated_at: u64,
}

/// Clinical Outcome
#[derive(Clone)]
#[contracttype]
pub struct ClinicalOutcome {
    pub outcome_name: String,
    pub baseline_value: f32,
    pub current_value: f32,
    pub target_value: f32,
    pub improvement_percentage: f32,
    pub clinical_significance: String,
    pub measurement_tool: String,
    pub last_assessment: u64,
}

/// Behavioral Outcome
#[derive(Clone)]
#[contracttype]
pub struct BehavioralOutcome {
    pub behavior_name: String,
    pub frequency_baseline: u32, // times per week
    pub frequency_current: u32,
    pub duration_minutes: u32,
    pub consistency_score: u8, // 0-100
    pub quality_rating: u8,    // 1-5
    pub context_factors: Vec<String>,
}

/// Engagement Metrics
#[derive(Clone)]
#[contracttype]
pub struct EngagementMetrics {
    pub login_frequency: u32,            // times per week
    pub session_duration_avg: u32,       // minutes
    pub feature_usage: Map<String, u32>, // feature -> usage count
    pub peak_usage_times: Vec<String>,
    pub dropout_risk_score: u8, // 0-100
    pub motivation_score: u8,   // 0-100
    pub satisfaction_score: u8, // 0-100
}

/// Safety Summary
#[derive(Clone)]
#[contracttype]
pub struct SafetySummary {
    pub adverse_events_count: u32,
    pub adverse_events_severity: Vec<String>,
    pub emergency_interventions: u32,
    pub clinical_contacts_triggered: u32,
    pub risk_assessment_score: u8, // 0-100
    pub safety_monitoring_alerts: u32,
    pub patient_safety_rating: u8, // 1-5
}

/// Adverse Event Report
#[derive(Clone)]
#[contracttype]
pub struct AdverseEventReport {
    pub event_id: u64,
    pub prescription_id: u64,
    pub patient: Address,
    pub provider: Address,
    pub event_type: String, // "symptom_exacerbation", "technical_issue", "psychological_distress"
    pub severity: String,   // "mild", "moderate", "severe", "life_threatening"
    pub description: String,
    pub onset_time: u64,
    pub resolution_time: Option<u64>,
    pub intervention_required: bool,
    pub intervention_type: String,
    pub outcome: String,     // "resolved", "ongoing", "unknown"
    pub reported_by: String, // "patient", "provider", "automated", "caregiver"
    pub reported_at: u64,
    pub follow_up_required: bool,
    pub follow_up_actions: Vec<String>,
}

/// Therapeutic Content
#[derive(Clone)]
#[contracttype]
pub struct TherapeuticContent {
    pub content_id: u64,
    pub therapeutic_id: u64,
    pub content_type: String, // "exercise", "education", "assessment", "meditation", "game"
    pub title: String,
    pub description: String,
    pub duration_minutes: u32,
    pub difficulty_level: u8, // 1-5
    pub age_appropriate: Vec<u8>,
    pub language: String,
    pub accessibility_features: Vec<String>,
    pub prerequisites: Vec<String>,
    pub learning_objectives: Vec<String>,
    pub assessment_criteria: Vec<String>,
    pub multimedia_assets: Vec<String>, // IPFS hashes
    pub version: String,
    pub clinical_validation: bool,
    pub usage_count: u32,
    pub effectiveness_score: u8, // 0-100
    pub user_ratings: Vec<u8>,   // Individual ratings
    pub average_rating: u8,      // 0-5
}

/// Integration Endpoint
#[derive(Clone)]
#[contracttype]
pub struct IntegrationEndpoint {
    pub endpoint_id: u64,
    pub therapeutic_id: u64,
    pub endpoint_type: String, // "api", "webhook", "fhir", "hl7"
    pub endpoint_url: String,
    pub authentication_method: String, // "oauth2", "api_key", "certificate"
    pub data_format: String,           // "json", "xml", "fhir", "hl7"
    pub supported_operations: Vec<String>, // "read", "write", "update", "delete"
    pub rate_limits: Map<String, u32>, // operation -> requests_per_hour
    pub data_mapping: Map<String, String>, // therapeutic_field -> standard_field
    pub encryption_required: bool,
    pub audit_logging: bool,
    pub status: String, // "active", "inactive", "error"
    pub last_tested: u64,
    pub test_results: Vec<String>,
}

// Storage Keys
const ADMIN: Symbol = symbol_short!("ADMIN");
const DIGITAL_THERAPEUTICS: Symbol = symbol_short!("THERAPEUTICS");
const PATIENT_PRESCRIPTIONS: Symbol = symbol_short!("PRESCRIPTIONS");
const THERAPY_SESSIONS: Symbol = symbol_short!("SESSIONS");
const PROGRESS_REPORTS: Symbol = symbol_short!("REPORTS");
const ADVERSE_EVENTS: Symbol = symbol_short!("ADVERSE_EVENTS");
const THERAPEUTIC_CONTENT: Symbol = symbol_short!("CONTENT");
const INTEGRATION_ENDPOINTS: Symbol = symbol_short!("ENDPOINTS");
const THERAPEUTIC_COUNTER: Symbol = symbol_short!("THERAPY_CNT");
const PRESCRIPTION_COUNTER: Symbol = symbol_short!("PRESC_CNT");
const SESSION_COUNTER: Symbol = symbol_short!("SESSION_CNT");
const REPORT_COUNTER: Symbol = symbol_short!("REPORT_CNT");
const ADVERSE_EVENT_COUNTER: Symbol = symbol_short!("ADVERSE_CNT");
const CONTENT_COUNTER: Symbol = symbol_short!("CONTENT_CNT");
const ENDPOINT_COUNTER: Symbol = symbol_short!("ENDPOINT_CNT");
const PAUSED: Symbol = symbol_short!("PAUSED");
const CONSENT_CONTRACT: Symbol = symbol_short!("CONSENT");
const MEDICAL_RECORDS_CONTRACT: Symbol = symbol_short!("MEDICAL");

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    NotAuthorized = 1,
    ContractPaused = 2,
    TherapeuticNotFound = 3,
    TherapeuticAlreadyExists = 4,
    PrescriptionNotFound = 5,
    PrescriptionAlreadyExists = 6,
    SessionNotFound = 7,
    InvalidTherapeuticCategory = 8,
    InvalidAgeRange = 9,
    InvalidDuration = 10,
    ConsentRequired = 11,
    ConsentRevoked = 12,
    PrescriptionExpired = 13,
    PrescriptionNotActive = 14,
    ContentNotFound = 15,
    ContentNotAccessible = 16,
    IntegrationEndpointNotFound = 17,
    InvalidEndpointType = 18,
    AdverseEventNotFound = 19,
    ReportNotFound = 20,
    InvalidMetric = 21,
    DataPrivacyViolation = 22,
    ClinicalSupervisionRequired = 23,
    MedicalRecordsContractNotSet = 24,
    ConsentContractNotSet = 25,
}

#[contract]
pub struct DigitalTherapeuticsContract;

#[contractimpl]
impl DigitalTherapeuticsContract {
    /// Initialize the digital therapeutics contract
    pub fn initialize(
        env: Env,
        admin: Address,
        consent_contract: Address,
        medical_records_contract: Address,
    ) -> Result<bool, Error> {
        admin.require_auth();

        if env.storage().persistent().has(&ADMIN) {
            return Err(Error::TherapeuticAlreadyExists);
        }

        env.storage().persistent().set(&ADMIN, &admin);
        env.storage()
            .persistent()
            .set(&CONSENT_CONTRACT, &consent_contract);
        env.storage()
            .persistent()
            .set(&MEDICAL_RECORDS_CONTRACT, &medical_records_contract);
        env.storage().persistent().set(&PAUSED, &false);
        env.storage().persistent().set(&THERAPEUTIC_COUNTER, &0u64);
        env.storage().persistent().set(&PRESCRIPTION_COUNTER, &0u64);
        env.storage().persistent().set(&SESSION_COUNTER, &0u64);
        env.storage().persistent().set(&REPORT_COUNTER, &0u64);
        env.storage()
            .persistent()
            .set(&ADVERSE_EVENT_COUNTER, &0u64);
        env.storage().persistent().set(&CONTENT_COUNTER, &0u64);
        env.storage().persistent().set(&ENDPOINT_COUNTER, &0u64);

        Ok(true)
    }

    /// Register a new digital therapeutic
    pub fn register_therapeutic(
        env: Env,
        developer: Address,
        name: String,
        description: String,
        category: TherapeuticCategory,
        version: String,
        fda_clearance: bool,
        ce_mark: bool,
        clinical_evidence_level: EvidenceLevel,
        target_conditions: Vec<String>,
        age_range: (u8, u8),
        languages: Vec<String>,
        platforms: Vec<String>,
        integration_apis: Vec<String>,
        data_types_collected: Vec<String>,
        privacy_level: PrivacyLevel,
        encryption_standard: String,
        hipaa_compliant: bool,
        gdpr_compliant: bool,
        subscription_required: bool,
        prescription_required: bool,
        clinical_supervision_required: bool,
        efficacy_metrics: Vec<String>,
        safety_monitoring: bool,
        adverse_event_reporting: bool,
        interoperability_standards: Vec<String>,
    ) -> Result<u64, Error> {
        developer.require_auth();

        if env.storage().persistent().get(&PAUSED).unwrap_or(false) {
            return Err(Error::ContractPaused);
        }

        // Validate age range
        if age_range.0 >= age_range.1 || age_range.1 > 120 {
            return Err(Error::InvalidAgeRange);
        }

        let therapeutic_id = Self::get_and_increment_therapeutic_counter(&env);
        let timestamp = env.ledger().timestamp();

        let therapeutic = DigitalTherapeutic {
            therapeutic_id,
            name: name.clone(),
            description,
            category,
            version,
            developer: developer.clone(),
            fda_clearance,
            ce_mark,
            clinical_evidence_level,
            target_conditions,
            age_range,
            languages,
            platforms,
            integration_apis,
            data_types_collected,
            privacy_level,
            encryption_standard,
            hipaa_compliant,
            gdpr_compliant,
            subscription_required,
            prescription_required,
            clinical_supervision_required,
            efficacy_metrics,
            safety_monitoring,
            adverse_event_reporting,
            interoperability_standards,
            created_at: timestamp,
            updated_at: timestamp,
            status: "active".to_string(),
        };

        let mut therapeutics: Map<u64, DigitalTherapeutic> = env
            .storage()
            .persistent()
            .get(&DIGITAL_THERAPEUTICS)
            .unwrap_or(Map::new(&env));
        therapeutics.set(therapeutic_id, therapeutic);
        env.storage()
            .persistent()
            .set(&DIGITAL_THERAPEUTICS, &therapeutics);

        // Emit event
        env.events().publish(
            (symbol_short!("Therapeutic"), symbol_short!("Registered")),
            (therapeutic_id, developer, name),
        );

        Ok(therapeutic_id)
    }

    /// Prescribe digital therapeutic to patient
    pub fn prescribe_therapeutic(
        env: Env,
        provider: Address,
        patient: Address,
        therapeutic_id: u64,
        start_date: u64,
        duration_weeks: u16,
        dosage_instructions: String,
        frequency: String,
        monitoring_required: bool,
        progress_tracking: bool,
        data_sharing_consent: bool,
        caregiver_access: bool,
        emergency_contact: Option<Address>,
        contraindications: Vec<String>,
        side_effects_monitoring: Vec<String>,
        progress_goals: Vec<String>,
        custom_parameters: Map<String, String>,
        consent_token_id: u64,
    ) -> Result<u64, Error> {
        provider.require_auth();

        if env.storage().persistent().get(&PAUSED).unwrap_or(false) {
            return Err(Error::ContractPaused);
        }

        // Verify therapeutic exists and is suitable
        let therapeutics: Map<u64, DigitalTherapeutic> = env
            .storage()
            .persistent()
            .get(&DIGITAL_THERAPEUTICS)
            .ok_or(Error::TherapeuticNotFound)?;

        let therapeutic = therapeutics
            .get(therapeutic_id)
            .ok_or(Error::TherapeuticNotFound)?;

        if therapeutic.status != "active" {
            return Err(Error::TherapeuticNotFound);
        }

        // Verify consent
        if !Self::verify_consent_token(&env, consent_token_id, patient.clone(), provider.clone())? {
            return Err(Error::ConsentRequired);
        }

        // Check if prescription already exists
        let prescriptions: Map<u64, PatientPrescription> = env
            .storage()
            .persistent()
            .get(&PATIENT_PRESCRIPTIONS)
            .unwrap_or(Map::new(&env));

        for prescription in prescriptions.values() {
            if prescription.patient == patient
                && prescription.therapeutic_id == therapeutic_id
                && prescription.status == TherapeuticStatus::Active
            {
                return Err(Error::PrescriptionAlreadyExists);
            }
        }

        let prescription_id = Self::get_and_increment_prescription_counter(&env);
        let timestamp = env.ledger().timestamp();
        let end_date = start_date + (duration_weeks as u64 * 604800); // weeks to seconds

        let prescription = PatientPrescription {
            prescription_id,
            patient: patient.clone(),
            provider: provider.clone(),
            therapeutic_id,
            prescription_date: timestamp,
            start_date,
            end_date,
            dosage_instructions,
            frequency,
            duration_weeks,
            monitoring_required,
            progress_tracking,
            data_sharing_consent,
            caregiver_access,
            emergency_contact,
            contraindications,
            side_effects_monitoring,
            progress_goals,
            custom_parameters,
            status: TherapeuticStatus::Active,
            adherence_score: 0,
            last_activity: timestamp,
            consent_token_id,
        };

        let mut prescriptions: Map<u64, PatientPrescription> = env
            .storage()
            .persistent()
            .get(&PATIENT_PRESCRIPTIONS)
            .unwrap_or(Map::new(&env));
        prescriptions.set(prescription_id, prescription);
        env.storage()
            .persistent()
            .set(&PATIENT_PRESCRIPTIONS, &prescriptions);

        // Emit event
        env.events().publish(
            (symbol_short!("Prescription"), symbol_short!("Created")),
            (prescription_id, patient, therapeutic_id),
        );

        Ok(prescription_id)
    }

    /// Record therapy session
    pub fn record_therapy_session(
        env: Env,
        prescription_id: u64,
        patient: Address,
        session_type: String,
        start_time: u64,
        end_time: u64,
        completion_rate: u8,
        exercises_completed: Vec<String>,
        metrics_collected: Vec<TherapyMetric>,
        patient_feedback: u8,
        difficulty_rating: u8,
        notes: String,
        adverse_events: Vec<String>,
        technical_issues: Vec<String>,
    ) -> Result<u64, Error> {
        patient.require_auth();

        if env.storage().persistent().get(&PAUSED).unwrap_or(false) {
            return Err(Error::ContractPaused);
        }

        // Validate prescription
        let prescriptions: Map<u64, PatientPrescription> = env
            .storage()
            .persistent()
            .get(&PATIENT_PRESCRIPTIONS)
            .ok_or(Error::PrescriptionNotFound)?;

        let prescription = prescriptions
            .get(prescription_id)
            .ok_or(Error::PrescriptionNotFound)?;

        if prescription.patient != patient {
            return Err(Error::NotAuthorized);
        }

        if prescription.status != TherapeuticStatus::Active {
            return Err(Error::PrescriptionNotActive);
        }

        if env.ledger().timestamp() > prescription.end_date {
            return Err(Error::PrescriptionExpired);
        }

        let session_id = Self::get_and_increment_session_counter(&env);
        let duration_minutes = ((end_time - start_time) / 60) as u32;

        let session = TherapySession {
            session_id,
            prescription_id,
            patient: patient.clone(),
            session_type,
            start_time,
            end_time,
            duration_minutes,
            completion_rate,
            exercises_completed,
            metrics_collected,
            patient_feedback,
            difficulty_rating,
            notes,
            adverse_events,
            technical_issues,
            data_quality_score: Self::calculate_data_quality_score(&metrics_collected),
            engagement_score: Self::calculate_engagement_score(completion_rate, duration_minutes),
            therapeutic_alliance_score: Self::calculate_therapeutic_alliance_score(
                patient_feedback,
                difficulty_rating,
            ),
        };

        let mut sessions: Map<u64, TherapySession> = env
            .storage()
            .persistent()
            .get(&THERAPY_SESSIONS)
            .unwrap_or(Map::new(&env));
        sessions.set(session_id, session);
        env.storage().persistent().set(&THERAPY_SESSIONS, &sessions);

        // Update prescription adherence
        Self::update_prescription_adherence(&env, prescription_id, patient.clone())?;

        // Check for adverse events
        if !adverse_events.is_empty() {
            for event in adverse_events.iter() {
                Self::create_adverse_event_report(
                    &env,
                    prescription_id,
                    patient.clone(),
                    event.clone(),
                )?;
            }
        }

        // Emit event
        env.events().publish(
            (symbol_short!("Session"), symbol_short!("Recorded")),
            (session_id, patient, prescription_id),
        );

        Ok(session_id)
    }

    /// Generate progress report
    pub fn generate_progress_report(
        env: Env,
        provider: Address,
        prescription_id: u64,
        report_period_start: u64,
        report_period_end: u64,
    ) -> Result<u64, Error> {
        provider.require_auth();

        if env.storage().persistent().get(&PAUSED).unwrap_or(false) {
            return Err(Error::ContractPaused);
        }

        // Validate prescription
        let prescriptions: Map<u64, PatientPrescription> = env
            .storage()
            .persistent()
            .get(&PATIENT_PRESCRIPTIONS)
            .ok_or(Error::PrescriptionNotFound)?;

        let prescription = prescriptions
            .get(prescription_id)
            .ok_or(Error::PrescriptionNotFound)?;

        if prescription.provider != provider {
            return Err(Error::NotAuthorized);
        }

        let report_id = Self::get_and_increment_report_counter(&env);
        let timestamp = env.ledger().timestamp();

        // Gather session data for the period
        let sessions: Map<u64, TherapySession> = env
            .storage()
            .persistent()
            .get(&THERAPY_SESSIONS)
            .unwrap_or(Map::new(&env));

        let mut period_sessions = Vec::new(&env);
        let mut total_sessions = 0u32;
        let mut completed_sessions = 0u32;

        for session in sessions.values() {
            if session.prescription_id == prescription_id
                && session.start_time >= report_period_start
                && session.start_time <= report_period_end
            {
                period_sessions.push_back(session.clone());
                total_sessions += 1;
                if session.completion_rate >= 80 {
                    completed_sessions += 1;
                }
            }
        }

        let adherence_rate = if total_sessions > 0 {
            (completed_sessions * 100) / total_sessions
        } else {
            0
        };

        // Generate outcomes and metrics
        let clinical_outcomes = Self::calculate_clinical_outcomes(&env, &period_sessions)?;
        let behavioral_outcomes = Self::calculate_behavioral_outcomes(&env, &period_sessions)?;
        let engagement_metrics = Self::calculate_engagement_metrics(&env, &period_sessions)?;
        let safety_summary = Self::calculate_safety_summary(&env, &period_sessions)?;

        let progress_report = ProgressReport {
            report_id,
            prescription_id,
            patient: prescription.patient,
            provider: provider.clone(),
            report_period_start,
            report_period_end,
            total_sessions,
            completed_sessions,
            adherence_rate,
            clinical_outcomes,
            behavioral_outcomes,
            engagement_metrics,
            safety_summary,
            efficacy_assessment: Self::assess_efficacy(adherence_rate, &clinical_outcomes),
            recommendations: Self::generate_recommendations(&progress_report),
            next_steps: Self::generate_next_steps(&progress_report),
            report_generated_at: timestamp,
        };

        let mut reports: Map<u64, ProgressReport> = env
            .storage()
            .persistent()
            .get(&PROGRESS_REPORTS)
            .unwrap_or(Map::new(&env));
        reports.set(report_id, progress_report);
        env.storage().persistent().set(&PROGRESS_REPORTS, &reports);

        // Emit event
        env.events().publish(
            (symbol_short!("Report"), symbol_short!("Generated")),
            (report_id, provider, prescription.patient),
        );

        Ok(report_id)
    }

    /// Add therapeutic content
    pub fn add_therapeutic_content(
        env: Env,
        developer: Address,
        therapeutic_id: u64,
        content_type: String,
        title: String,
        description: String,
        duration_minutes: u32,
        difficulty_level: u8,
        age_appropriate: Vec<u8>,
        language: String,
        accessibility_features: Vec<String>,
        prerequisites: Vec<String>,
        learning_objectives: Vec<String>,
        assessment_criteria: Vec<String>,
        multimedia_assets: Vec<String>,
        version: String,
        clinical_validation: bool,
    ) -> Result<u64, Error> {
        developer.require_auth();

        if env.storage().persistent().get(&PAUSED).unwrap_or(false) {
            return Err(Error::ContractPaused);
        }

        // Verify therapeutic exists and developer owns it
        let therapeutics: Map<u64, DigitalTherapeutic> = env
            .storage()
            .persistent()
            .get(&DIGITAL_THERAPEUTICS)
            .ok_or(Error::TherapeuticNotFound)?;

        let therapeutic = therapeutics
            .get(therapeutic_id)
            .ok_or(Error::TherapeuticNotFound)?;

        if therapeutic.developer != developer {
            return Err(Error::NotAuthorized);
        }

        let content_id = Self::get_and_increment_content_counter(&env);

        let content = TherapeuticContent {
            content_id,
            therapeutic_id,
            content_type,
            title,
            description,
            duration_minutes,
            difficulty_level,
            age_appropriate,
            language,
            accessibility_features,
            prerequisites,
            learning_objectives,
            assessment_criteria,
            multimedia_assets,
            version,
            clinical_validation,
            usage_count: 0,
            effectiveness_score: 0,
            user_ratings: Vec::new(&env),
            average_rating: 0,
        };

        let mut content_library: Map<u64, TherapeuticContent> = env
            .storage()
            .persistent()
            .get(&THERAPEUTIC_CONTENT)
            .unwrap_or(Map::new(&env));
        content_library.set(content_id, content);
        env.storage()
            .persistent()
            .set(&THERAPEUTIC_CONTENT, &content_library);

        // Emit event
        env.events().publish(
            (symbol_short!("Content"), symbol_short!("Added")),
            (content_id, therapeutic_id),
        );

        Ok(content_id)
    }

    /// Create integration endpoint
    pub fn create_integration_endpoint(
        env: Env,
        developer: Address,
        therapeutic_id: u64,
        endpoint_type: String,
        endpoint_url: String,
        authentication_method: String,
        data_format: String,
        supported_operations: Vec<String>,
        rate_limits: Map<String, u32>,
        data_mapping: Map<String, String>,
        encryption_required: bool,
        audit_logging: bool,
    ) -> Result<u64, Error> {
        developer.require_auth();

        if env.storage().persistent().get(&PAUSED).unwrap_or(false) {
            return Err(Error::ContractPaused);
        }

        // Verify therapeutic ownership
        let therapeutics: Map<u64, DigitalTherapeutic> = env
            .storage()
            .persistent()
            .get(&DIGITAL_THERAPEUTICS)
            .ok_or(Error::TherapeuticNotFound)?;

        let therapeutic = therapeutics
            .get(therapeutic_id)
            .ok_or(Error::TherapeuticNotFound)?;

        if therapeutic.developer != developer {
            return Err(Error::NotAuthorized);
        }

        let endpoint_id = Self::get_and_increment_endpoint_counter(&env);

        let endpoint = IntegrationEndpoint {
            endpoint_id,
            therapeutic_id,
            endpoint_type,
            endpoint_url,
            authentication_method,
            data_format,
            supported_operations,
            rate_limits,
            data_mapping,
            encryption_required,
            audit_logging,
            status: "active".to_string(),
            last_tested: env.ledger().timestamp(),
            test_results: Vec::new(&env),
        };

        let mut endpoints: Map<u64, IntegrationEndpoint> = env
            .storage()
            .persistent()
            .get(&INTEGRATION_ENDPOINTS)
            .unwrap_or(Map::new(&env));
        endpoints.set(endpoint_id, endpoint);
        env.storage()
            .persistent()
            .set(&INTEGRATION_ENDPOINTS, &endpoints);

        // Emit event
        env.events().publish(
            (symbol_short!("Endpoint"), symbol_short!("Created")),
            (endpoint_id, therapeutic_id),
        );

        Ok(endpoint_id)
    }

    /// Report adverse event
    pub fn report_adverse_event(
        env: Env,
        prescription_id: u64,
        event_type: String,
        severity: String,
        description: String,
        onset_time: u64,
        intervention_required: bool,
        intervention_type: String,
        reported_by: String,
        follow_up_required: bool,
        follow_up_actions: Vec<String>,
    ) -> Result<u64, Error> {
        // This could be called by patient, provider, or automated system
        if env.storage().persistent().get(&PAUSED).unwrap_or(false) {
            return Err(Error::ContractPaused);
        }

        // Validate prescription
        let prescriptions: Map<u64, PatientPrescription> = env
            .storage()
            .persistent()
            .get(&PATIENT_PRESCRIPTIONS)
            .ok_or(Error::PrescriptionNotFound)?;

        let prescription = prescriptions
            .get(prescription_id)
            .ok_or(Error::PrescriptionNotFound)?;

        let event_id = Self::get_and_increment_adverse_event_counter(&env);

        let adverse_event = AdverseEventReport {
            event_id,
            prescription_id,
            patient: prescription.patient,
            provider: prescription.provider,
            event_type,
            severity,
            description,
            onset_time,
            resolution_time: None,
            intervention_required,
            intervention_type,
            outcome: "ongoing".to_string(),
            reported_by,
            reported_at: env.ledger().timestamp(),
            follow_up_required,
            follow_up_actions,
        };

        let mut adverse_events: Map<u64, AdverseEventReport> = env
            .storage()
            .persistent()
            .get(&ADVERSE_EVENTS)
            .unwrap_or(Map::new(&env));
        adverse_events.set(event_id, adverse_event);
        env.storage()
            .persistent()
            .set(&ADVERSE_EVENTS, &adverse_events);

        // Emit event
        env.events().publish(
            (symbol_short!("AdverseEvent"), symbol_short!("Reported")),
            (event_id, prescription.patient, prescription.provider),
        );

        Ok(event_id)
    }

    /// Get digital therapeutic details
    pub fn get_therapeutic(env: Env, therapeutic_id: u64) -> Result<DigitalTherapeutic, Error> {
        let therapeutics: Map<u64, DigitalTherapeutic> = env
            .storage()
            .persistent()
            .get(&DIGITAL_THERAPEUTICS)
            .ok_or(Error::TherapeuticNotFound)?;

        therapeutics
            .get(therapeutic_id)
            .ok_or(Error::TherapeuticNotFound)
    }

    /// Get patient prescription
    pub fn get_patient_prescription(
        env: Env,
        prescription_id: u64,
    ) -> Result<PatientPrescription, Error> {
        let prescriptions: Map<u64, PatientPrescription> = env
            .storage()
            .persistent()
            .get(&PATIENT_PRESCRIPTIONS)
            .ok_or(Error::PrescriptionNotFound)?;

        prescriptions
            .get(prescription_id)
            .ok_or(Error::PrescriptionNotFound)
    }

    /// Get therapy session
    pub fn get_therapy_session(env: Env, session_id: u64) -> Result<TherapySession, Error> {
        let sessions: Map<u64, TherapySession> = env
            .storage()
            .persistent()
            .get(&THERAPY_SESSIONS)
            .ok_or(Error::SessionNotFound)?;

        sessions.get(session_id).ok_or(Error::SessionNotFound)
    }

    /// Get progress report
    pub fn get_progress_report(env: Env, report_id: u64) -> Result<ProgressReport, Error> {
        let reports: Map<u64, ProgressReport> = env
            .storage()
            .persistent()
            .get(&PROGRESS_REPORTS)
            .ok_or(Error::ReportNotFound)?;

        reports.get(report_id).ok_or(Error::ReportNotFound)
    }

    /// Get therapeutic content
    pub fn get_therapeutic_content(env: Env, content_id: u64) -> Result<TherapeuticContent, Error> {
        let content: Map<u64, TherapeuticContent> = env
            .storage()
            .persistent()
            .get(&THERAPEUTIC_CONTENT)
            .ok_or(Error::ContentNotFound)?;

        content.get(content_id).ok_or(Error::ContentNotFound)
    }

    /// Get patient's active prescriptions
    pub fn get_patient_prescriptions(
        env: Env,
        patient: Address,
    ) -> Result<Vec<PatientPrescription>, Error> {
        let prescriptions: Map<u64, PatientPrescription> = env
            .storage()
            .persistent()
            .get(&PATIENT_PRESCRIPTIONS)
            .unwrap_or(Map::new(&env));

        let mut patient_prescriptions = Vec::new(&env);
        for prescription in prescriptions.values() {
            if prescription.patient == patient && prescription.status == TherapeuticStatus::Active {
                patient_prescriptions.push_back(prescription);
            }
        }

        Ok(patient_prescriptions)
    }

    // ==================== Helper Functions ====================

    fn verify_consent_token(
        env: &Env,
        token_id: u64,
        patient: Address,
        provider: Address,
    ) -> Result<bool, Error> {
        let consent_contract: Address = env
            .storage()
            .persistent()
            .get(&CONSENT_CONTRACT)
            .ok_or(Error::ConsentContractNotSet)?;

        // This would call the consent contract to verify the token
        // For now, we'll assume it's valid if not revoked
        Ok(true)
    }

    fn calculate_data_quality_score(metrics: &Vec<TherapyMetric>) -> u8 {
        if metrics.is_empty() {
            return 0;
        }

        let total_confidence: u32 = metrics.iter().map(|m| m.confidence_score as u32).sum();
        (total_confidence / metrics.len() as u32) as u8
    }

    fn calculate_engagement_score(completion_rate: u8, duration_minutes: u32) -> u8 {
        // Simple engagement score based on completion and duration
        let duration_score = if duration_minutes >= 30 {
            100
        } else if duration_minutes >= 15 {
            75
        } else {
            50
        };
        ((completion_rate as u32 + duration_score) / 2) as u8
    }

    fn calculate_therapeutic_alliance_score(patient_feedback: u8, difficulty_rating: u8) -> u8 {
        // Alliance score based on feedback and appropriate difficulty
        let difficulty_score = match difficulty_rating {
            1 | 2 => 60, // Too easy
            3 => 100,    // Just right
            4 => 80,     // Slightly hard
            5 => 40,     // Too hard
            _ => 0,
        };

        let feedback_score = (patient_feedback as u32) * 20; // Convert 1-5 to 20-100
        ((feedback_score + difficulty_score as u32) / 2) as u8
    }

    fn update_prescription_adherence(
        env: &Env,
        prescription_id: u64,
        patient: Address,
    ) -> Result<(), Error> {
        let mut prescriptions: Map<u64, PatientPrescription> = env
            .storage()
            .persistent()
            .get(&PATIENT_PRESCRIPTIONS)
            .ok_or(Error::PrescriptionNotFound)?;

        let mut prescription = prescriptions
            .get(prescription_id)
            .ok_or(Error::PrescriptionNotFound)?;

        // Calculate adherence based on recent sessions
        let sessions: Map<u64, TherapySession> = env
            .storage()
            .persistent()
            .get(&THERAPY_SESSIONS)
            .unwrap_or(Map::new(env));

        let mut recent_sessions = Vec::new(env);
        let cutoff_time = env.ledger().timestamp() - 604800; // Last 7 days

        for session in sessions.values() {
            if session.prescription_id == prescription_id && session.start_time >= cutoff_time {
                recent_sessions.push_back(session);
            }
        }

        let adherence_score = if recent_sessions.is_empty() {
            0
        } else {
            let total_completion: u32 = recent_sessions
                .iter()
                .map(|s| s.completion_rate as u32)
                .sum();
            (total_completion / recent_sessions.len() as u32) as u8
        };

        prescription.adherence_score = adherence_score;
        prescription.last_activity = env.ledger().timestamp();

        prescriptions.set(prescription_id, prescription);
        env.storage()
            .persistent()
            .set(&PATIENT_PRESCRIPTIONS, &prescriptions);

        Ok(())
    }

    fn create_adverse_event_report(
        env: &Env,
        prescription_id: u64,
        patient: Address,
        event_description: String,
    ) -> Result<(), Error> {
        let event_id = Self::get_and_increment_adverse_event_counter(env);

        let adverse_event = AdverseEventReport {
            event_id,
            prescription_id,
            patient,
            provider: Address::from_array(env, &[0u8; 32]), // Would get from prescription
            event_type: "session_related".to_string(),
            severity: "moderate".to_string(),
            description: event_description,
            onset_time: env.ledger().timestamp(),
            resolution_time: None,
            intervention_required: false,
            intervention_type: String::from_str(env, ""),
            outcome: "ongoing".to_string(),
            reported_by: "automated".to_string(),
            reported_at: env.ledger().timestamp(),
            follow_up_required: true,
            follow_up_actions: vec![env, "clinical_review".to_string()],
        };

        let mut adverse_events: Map<u64, AdverseEventReport> = env
            .storage()
            .persistent()
            .get(&ADVERSE_EVENTS)
            .unwrap_or(Map::new(env));
        adverse_events.set(event_id, adverse_event);
        env.storage()
            .persistent()
            .set(&ADVERSE_EVENTS, &adverse_events);

        Ok(())
    }

    fn calculate_clinical_outcomes(
        env: &Env,
        sessions: &Vec<TherapySession>,
    ) -> Result<Vec<ClinicalOutcome>, Error> {
        let mut outcomes = Vec::new(env);

        // Simplified clinical outcome calculation
        // In production, would analyze specific metrics from sessions
        for session in sessions.iter() {
            for metric in session.metrics_collected.iter() {
                if metric.category == "clinical" {
                    let outcome = ClinicalOutcome {
                        outcome_name: metric.metric_name.clone(),
                        baseline_value: 0.0, // Would get from initial assessment
                        current_value: metric.value,
                        target_value: 0.0, // Would get from treatment goals
                        improvement_percentage: 0.0, // Would calculate
                        clinical_significance: metric.clinical_significance.clone(),
                        measurement_tool: metric.data_source.clone(),
                        last_assessment: metric.timestamp,
                    };
                    outcomes.push_back(outcome);
                }
            }
        }

        Ok(outcomes)
    }

    fn calculate_behavioral_outcomes(
        env: &Env,
        sessions: &Vec<TherapySession>,
    ) -> Result<Vec<BehavioralOutcome>, Error> {
        let mut outcomes = Vec::new(env);

        // Simplified behavioral outcome calculation
        let frequency_current = sessions.len() as u32;
        let frequency_baseline = 3; // Target 3 sessions per week

        let outcome = BehavioralOutcome {
            behavior_name: "therapy_sessions".to_string(),
            frequency_baseline,
            frequency_current,
            duration_minutes: 30, // Average session duration
            consistency_score: if frequency_current >= frequency_baseline {
                80
            } else {
                60
            },
            quality_rating: 4, // Average quality
            context_factors: Vec::new(env),
        };

        outcomes.push_back(outcome);
        Ok(outcomes)
    }

    fn calculate_engagement_metrics(
        env: &Env,
        sessions: &Vec<TherapySession>,
    ) -> Result<EngagementMetrics, Error> {
        let mut feature_usage = Map::new(env);

        // Calculate engagement metrics from sessions
        let total_duration: u32 = sessions.iter().map(|s| s.duration_minutes).sum();
        let avg_duration = if sessions.is_empty() {
            0
        } else {
            total_duration / sessions.len() as u32
        };

        let engagement = EngagementMetrics {
            login_frequency: sessions.len() as u32,
            session_duration_avg: avg_duration,
            feature_usage,
            peak_usage_times: Vec::new(env),
            dropout_risk_score: 20, // Low risk
            motivation_score: 75,   // Good motivation
            satisfaction_score: 4,  // Good satisfaction
        };

        Ok(engagement)
    }

    fn calculate_safety_summary(
        env: &Env,
        sessions: &Vec<TherapySession>,
    ) -> Result<SafetySummary, Error> {
        let mut adverse_events_count = 0u32;
        let mut severities = Vec::new(env);

        for session in sessions.iter() {
            adverse_events_count += session.adverse_events.len() as u32;
            for event in session.adverse_events.iter() {
                severities.push_back("mild".to_string());
            }
        }

        let safety = SafetySummary {
            adverse_events_count,
            adverse_events_severity: severities,
            emergency_interventions: 0,
            clinical_contacts_triggered: 0,
            risk_assessment_score: if adverse_events_count == 0 { 95 } else { 70 },
            safety_monitoring_alerts: adverse_events_count,
            patient_safety_rating: if adverse_events_count == 0 { 5 } else { 3 },
        };

        Ok(safety)
    }

    fn assess_efficacy(adherence_rate: u8, clinical_outcomes: &Vec<ClinicalOutcome>) -> String {
        if adherence_rate >= 80 {
            for outcome in clinical_outcomes.iter() {
                if outcome.clinical_significance == "improved" {
                    return "effective".to_string();
                }
            }
            "partially_effective".to_string()
        } else {
            "ineffective".to_string()
        }
    }

    fn generate_recommendations(report: &ProgressReport) -> Vec<String> {
        let mut recommendations = Vec::new();

        if report.adherence_rate < 70 {
            recommendations.push_back("Increase patient engagement and motivation".to_string());
        }

        if report.safety_summary.adverse_events_count > 0 {
            recommendations.push_back("Review and adjust therapeutic intensity".to_string());
        }

        recommendations
    }

    fn generate_next_steps(report: &ProgressReport) -> Vec<String> {
        let mut next_steps = Vec::new();

        if report.adherence_rate >= 80 {
            next_steps.push_back("Continue current treatment plan".to_string());
        } else {
            next_steps.push_back("Schedule follow-up consultation".to_string());
        }

        next_steps.push_back("Generate monthly progress report".to_string());
        next_steps
    }

    fn get_and_increment_therapeutic_counter(env: &Env) -> u64 {
        let count: u64 = env
            .storage()
            .persistent()
            .get(&THERAPEUTIC_COUNTER)
            .unwrap_or(0);
        let next = count + 1;
        env.storage().persistent().set(&THERAPEUTIC_COUNTER, &next);
        next
    }

    fn get_and_increment_prescription_counter(env: &Env) -> u64 {
        let count: u64 = env
            .storage()
            .persistent()
            .get(&PRESCRIPTION_COUNTER)
            .unwrap_or(0);
        let next = count + 1;
        env.storage().persistent().set(&PRESCRIPTION_COUNTER, &next);
        next
    }

    fn get_and_increment_session_counter(env: &Env) -> u64 {
        let count: u64 = env
            .storage()
            .persistent()
            .get(&SESSION_COUNTER)
            .unwrap_or(0);
        let next = count + 1;
        env.storage().persistent().set(&SESSION_COUNTER, &next);
        next
    }

    fn get_and_increment_report_counter(env: &Env) -> u64 {
        let count: u64 = env.storage().persistent().get(&REPORT_COUNTER).unwrap_or(0);
        let next = count + 1;
        env.storage().persistent().set(&REPORT_COUNTER, &next);
        next
    }

    fn get_and_increment_adverse_event_counter(env: &Env) -> u64 {
        let count: u64 = env
            .storage()
            .persistent()
            .get(&ADVERSE_EVENT_COUNTER)
            .unwrap_or(0);
        let next = count + 1;
        env.storage()
            .persistent()
            .set(&ADVERSE_EVENT_COUNTER, &next);
        next
    }

    fn get_and_increment_content_counter(env: &Env) -> u64 {
        let count: u64 = env
            .storage()
            .persistent()
            .get(&CONTENT_COUNTER)
            .unwrap_or(0);
        let next = count + 1;
        env.storage().persistent().set(&CONTENT_COUNTER, &next);
        next
    }

    fn get_and_increment_endpoint_counter(env: &Env) -> u64 {
        let count: u64 = env
            .storage()
            .persistent()
            .get(&ENDPOINT_COUNTER)
            .unwrap_or(0);
        let next = count + 1;
        env.storage().persistent().set(&ENDPOINT_COUNTER, &next);
        next
    }

    /// Pause contract operations (admin only)
    pub fn pause(env: Env, admin: Address) -> Result<bool, Error> {
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

    /// Resume contract operations (admin only)
    pub fn resume(env: Env, admin: Address) -> Result<bool, Error> {
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

    /// Health check for monitoring
    pub fn health_check(env: Env) -> (Symbol, u32, u64) {
        let status = if env.storage().persistent().get(&PAUSED).unwrap_or(false) {
            symbol_short!("PAUSED")
        } else {
            symbol_short!("OK")
        };
        (status, 1, env.ledger().timestamp())
    }
}
