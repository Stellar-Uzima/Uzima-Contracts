#![no_std]
#![allow(dead_code)]
#![allow(clippy::too_many_arguments)]

#[cfg(test)]
mod test;

mod errors;
mod events;
mod types;
mod therapy;
mod mood_tracking;
mod assessments;
mod crisis_intervention;
mod medication;
mod peer_support;
mod anonymization;
mod wellness;
mod professional_directory;
mod suicide_prevention;

pub use errors::Error;
pub use types::*;

use soroban_sdk::{
    contract, contractimpl, contracttype, symbol_short, Address, Bytes, BytesN, Env, Map, String,
    Symbol, Vec,
};
use upgradeability::storage::{ADMIN as UPGRADE_ADMIN, VERSION};

#[contract]
pub struct MentalHealthPlatform;

#[contractimpl]
impl MentalHealthPlatform {
    /// Initialize the mental health platform
    pub fn initialize(env: Env, admin: Address) {
        if env.storage().instance().has(&symbol_short!("init")) {
            panic!("Already initialized");
        }

        env.storage().instance().set(&symbol_short!("init"), &true);
        env.storage().instance().set(&UPGRADE_ADMIN, &admin);
        env.storage().instance().set(&VERSION, &1u32);

        // Initialize storage maps
        env.storage().instance().set(&symbol_short!("therapy_sessions"), &Map::<Address, Vec<TherapySession>>::new(&env));
        env.storage().instance().set(&symbol_short!("mood_entries"), &Map::<Address, Vec<MoodEntry>>::new(&env));
        env.storage().instance().set(&symbol_short!("assessments"), &Map::<Address, Vec<Assessment>>::new(&env));
        env.storage().instance().set(&symbol_short!("medication_plans"), &Map::<Address, Vec<MedicationPlan>>::new(&env));
        env.storage().instance().set(&symbol_short!("crisis_alerts"), &Map::<Address, Vec<CrisisAlert>>::new(&env));
        env.storage().instance().set(&symbol_short!("peer_groups"), &Map::<String, PeerGroup>::new(&env));
        env.storage().instance().set(&symbol_short!("professionals"), &Map::<Address, MentalHealthProfessional>::new(&env));
        env.storage().instance().set(&symbol_short!("wellness_programs"), &Map::<Address, Vec<WellnessProgram>>::new(&env));

        env.events().publish((symbol_short!("init"),), (admin,));
    }

    /// Register a user in the mental health platform
    pub fn register_user(env: Env, user: Address, user_type: UserType, consent_given: bool) {
        Self::require_initialized(&env);

        if !consent_given {
            panic!("User consent required for mental health data processing");
        }

        let user_profile = UserProfile {
            user_id: user.clone(),
            user_type,
            registration_timestamp: env.ledger().timestamp(),
            consent_version: 1,
            privacy_settings: PrivacySettings::default(),
            emergency_contacts: Vec::new(&env),
            risk_level: RiskLevel::Low,
            last_activity: env.ledger().timestamp(),
        };

        env.storage().instance().set(&user, &user_profile);
        env.events().publish((symbol_short!("user_registered"),), (user, user_type));
    }

    /// Update user privacy settings
    pub fn update_privacy_settings(env: Env, user: Address, settings: PrivacySettings) {
        Self::require_initialized(&env);
        Self::require_user_registered(&env, &user);

        let mut profile: UserProfile = env.storage().instance().get(&user).unwrap();
        profile.privacy_settings = settings;
        env.storage().instance().set(&user, &profile);

        env.events().publish((symbol_short!("privacy_updated"),), (user,));
    }

    /// Add emergency contact
    pub fn add_emergency_contact(env: Env, user: Address, contact: EmergencyContact) {
        Self::require_initialized(&env);
        Self::require_user_registered(&env, &user);

        let mut profile: UserProfile = env.storage().instance().get(&user).unwrap();
        profile.emergency_contacts.push_back(contact);
        env.storage().instance().set(&user, &profile);

        env.events().publish((symbol_short!("emergency_contact_added"),), (user, contact.name));
    }

    // ========== THERAPY SESSION FUNCTIONS ==========

    /// Create a therapy session
    pub fn create_therapy_session(
        env: Env,
        patient_id: Address,
        therapist_id: Address,
        session_type: SessionType,
        duration_minutes: u32,
        confidentiality_level: ConfidentialityLevel,
    ) -> Result<u64, Error> {
        Self::require_initialized(&env);
        therapy::TherapyManager::create_session(
            &env, patient_id, therapist_id, session_type, duration_minutes, confidentiality_level
        )
    }

    /// Record therapy session notes
    pub fn record_session_notes(
        env: Env,
        session_id: u64,
        patient_id: Address,
        notes: String,
        ai_insights: Option<String>,
    ) -> Result<(), Error> {
        Self::require_initialized(&env);
        therapy::TherapyManager::record_session_notes(&env, session_id, patient_id, notes, ai_insights)
    }

    /// Get patient therapy sessions
    pub fn get_patient_sessions(env: Env, patient_id: Address) -> Vec<TherapySession> {
        Self::require_initialized(&env);
        therapy::TherapyManager::get_patient_sessions(&env, patient_id)
    }

    // ========== MOOD TRACKING FUNCTIONS ==========

    /// Record mood entry
    pub fn record_mood(
        env: Env,
        patient_id: Address,
        mood_score: i32,
        emotions: Vec<String>,
        triggers: Vec<String>,
        notes: String,
        location_context: Option<String>,
    ) -> Result<u64, Error> {
        Self::require_initialized(&env);
        mood_tracking::MoodTracker::record_mood(
            &env, patient_id, mood_score, emotions, triggers, notes, location_context
        )
    }

    /// Get mood history
    pub fn get_mood_history(env: Env, patient_id: Address, limit: Option<u32>) -> Vec<MoodEntry> {
        Self::require_initialized(&env);
        mood_tracking::MoodTracker::get_mood_history(&env, patient_id, limit)
    }

    /// Analyze mood trends
    pub fn analyze_mood_trends(env: Env, patient_id: Address, days: u32) -> mood_tracking::MoodTrendAnalysis {
        Self::require_initialized(&env);
        mood_tracking::MoodTracker::analyze_mood_trends(&env, patient_id, days)
    }

    // ========== ASSESSMENT FUNCTIONS ==========

    /// Create mental health assessment
    pub fn create_assessment(
        env: Env,
        patient_id: Address,
        assessment_type: AssessmentType,
        administered_by: Address,
    ) -> Result<u64, Error> {
        Self::require_initialized(&env);
        assessments::AssessmentManager::create_assessment(&env, patient_id, assessment_type, administered_by)
    }

    /// Submit assessment responses
    pub fn submit_assessment_responses(
        env: Env,
        assessment_id: u64,
        patient_id: Address,
        responses: Map<String, String>,
    ) -> Result<(), Error> {
        Self::require_initialized(&env);
        assessments::AssessmentManager::submit_assessment_responses(&env, assessment_id, patient_id, responses)
    }

    /// Get patient assessments
    pub fn get_patient_assessments(env: Env, patient_id: Address) -> Vec<Assessment> {
        Self::require_initialized(&env);
        assessments::AssessmentManager::get_patient_assessments(&env, patient_id)
    }

    // ========== CRISIS INTERVENTION FUNCTIONS ==========

    /// Create crisis alert
    pub fn create_crisis_alert(
        env: Env,
        patient_id: Address,
        alert_type: CrisisType,
        severity: CrisisSeverity,
        description: String,
        location: Option<String>,
    ) -> Result<u64, Error> {
        Self::require_initialized(&env);
        crisis_intervention::CrisisInterventionManager::create_crisis_alert(
            &env, patient_id, alert_type, severity, description, location
        )
    }

    /// Update crisis resolution
    pub fn update_crisis_resolution(
        env: Env,
        alert_id: u64,
        patient_id: Address,
        resolution_status: CrisisResolution,
        actions_taken: Vec<String>,
        follow_up_required: bool,
    ) -> Result<(), Error> {
        Self::require_initialized(&env);
        crisis_intervention::CrisisInterventionManager::update_crisis_resolution(
            &env, alert_id, patient_id, resolution_status, actions_taken, follow_up_required
        )
    }

    /// Assess crisis risk
    pub fn assess_crisis_risk(
        env: Env,
        patient_id: Address,
        indicators: Vec<String>,
    ) -> crisis_intervention::CrisisRiskAssessment {
        Self::require_initialized(&env);
        crisis_intervention::CrisisInterventionManager::assess_crisis_risk(&env, patient_id, indicators)
    }

    // ========== MEDICATION MANAGEMENT FUNCTIONS ==========

    /// Create medication plan
    pub fn create_medication_plan(
        env: Env,
        patient_id: Address,
        medication_name: String,
        dosage: String,
        frequency: String,
        start_date: u64,
        end_date: Option<u64>,
        prescribed_by: Address,
        side_effects: Vec<String>,
    ) -> Result<u64, Error> {
        Self::require_initialized(&env);
        medication::MedicationManager::create_medication_plan(
            &env, patient_id, medication_name, dosage, frequency, start_date, end_date,
            prescribed_by, side_effects
        )
    }

    /// Record medication adherence
    pub fn record_medication_adherence(
        env: Env,
        plan_id: u64,
        patient_id: Address,
        taken: bool,
        dosage_taken: Option<String>,
        side_effects_experienced: Vec<String>,
        notes: String,
    ) -> Result<(), Error> {
        Self::require_initialized(&env);
        medication::MedicationManager::record_adherence(
            &env, plan_id, patient_id, taken, dosage_taken, side_effects_experienced, notes
        )
    }

    /// Calculate adherence rate
    pub fn calculate_adherence_rate(
        env: Env,
        plan_id: u64,
        patient_id: Address,
        days: u32,
    ) -> Result<f32, Error> {
        Self::require_initialized(&env);
        medication::MedicationManager::calculate_adherence_rate(&env, plan_id, patient_id, days)
    }

    // ========== PEER SUPPORT FUNCTIONS ==========

    /// Create peer group
    pub fn create_peer_group(
        env: Env,
        group_id: String,
        name: String,
        description: String,
        focus_area: String,
        moderator: Address,
        max_members: u32,
        privacy_level: GroupPrivacy,
    ) -> Result<(), Error> {
        Self::require_initialized(&env);
        peer_support::PeerSupportManager::create_peer_group(
            &env, group_id, name, description, focus_area, moderator, max_members, privacy_level
        )
    }

    /// Join peer group
    pub fn join_peer_group(env: Env, group_id: String, user: Address) -> Result<(), Error> {
        Self::require_initialized(&env);
        peer_support::PeerSupportManager::join_peer_group(&env, group_id, user)
    }

    /// Post message to peer group
    pub fn post_peer_message(
        env: Env,
        group_id: String,
        sender: Address,
        content: String,
        message_type: MessageType,
    ) -> Result<u64, Error> {
        Self::require_initialized(&env);
        peer_support::PeerSupportManager::post_message(&env, group_id, sender, content, message_type)
    }

    // ========== PROFESSIONAL DIRECTORY FUNCTIONS ==========

    /// Register mental health professional
    pub fn register_professional(
        env: Env,
        professional_id: Address,
        name: String,
        credentials: Vec<Credential>,
        specializations: Vec<String>,
        languages: Vec<String>,
        availability: AvailabilitySchedule,
        contact_info: ContactInfo,
        bio: String,
        insurance_accepted: Vec<String>,
    ) -> Result<(), Error> {
        Self::require_initialized(&env);
        professional_directory::ProfessionalDirectoryManager::register_professional(
            &env, professional_id, name, credentials, specializations, languages,
            availability, contact_info, bio, insurance_accepted
        )
    }

    /// Schedule appointment
    pub fn schedule_appointment(
        env: Env,
        patient_id: Address,
        professional_id: Address,
        appointment_time: u64,
        appointment_type: String,
        notes: String,
    ) -> Result<u64, Error> {
        Self::require_initialized(&env);
        professional_directory::ProfessionalDirectoryManager::schedule_appointment(
            &env, patient_id, professional_id, appointment_time, appointment_type, notes
        )
    }

    // ========== WELLNESS PROGRAM FUNCTIONS ==========

    /// Create wellness program
    pub fn create_wellness_program(
        env: Env,
        name: String,
        description: String,
        category: WellnessCategory,
        duration_weeks: u32,
        modules: Vec<WellnessModule>,
    ) -> Result<u64, Error> {
        Self::require_initialized(&env);
        wellness::WellnessManager::create_wellness_program(
            &env, name, description, category, duration_weeks, modules
        )
    }

    /// Enroll in wellness program
    pub fn enroll_wellness_program(env: Env, program_id: u64, user_id: Address) -> Result<(), Error> {
        Self::require_initialized(&env);
        wellness::WellnessManager::enroll_in_program(&env, program_id, user_id)
    }

    /// Complete wellness module
    pub fn complete_wellness_module(
        env: Env,
        program_id: u64,
        user_id: Address,
        module_id: u64,
        session_duration: u32,
    ) -> Result<(), Error> {
        Self::require_initialized(&env);
        wellness::WellnessManager::complete_module(&env, program_id, user_id, module_id, session_duration)
    }

    /// Get personalized wellness recommendations
    pub fn get_wellness_recommendations(env: Env, user_id: Address) -> Vec<wellness::WellnessRecommendation> {
        Self::require_initialized(&env);
        wellness::WellnessManager::generate_personalized_recommendations(&env, user_id)
    }

    // ========== DATA ANONYMIZATION FUNCTIONS ==========

    /// Create anonymized dataset
    pub fn create_anonymized_dataset(
        env: Env,
        name: String,
        description: String,
        data_fields: Vec<String>,
        anonymization_method: AnonymizationMethod,
        creator: Address,
    ) -> Result<u64, Error> {
        Self::require_initialized(&env);
        anonymization::DataAnonymizationManager::create_anonymized_dataset(
            &env, name, description, data_fields, anonymization_method, creator
        )
    }

    /// Submit research query
    pub fn submit_research_query(
        env: Env,
        researcher_id: Address,
        dataset_id: u64,
        query_type: QueryType,
        parameters: Map<String, String>,
    ) -> Result<u64, Error> {
        Self::require_initialized(&env);
        anonymization::DataAnonymizationManager::submit_research_query(
            &env, researcher_id, dataset_id, query_type, parameters
        )
    }

    // ========== SUICIDE PREVENTION FUNCTIONS ==========

    /// Create prevention protocol
    pub fn create_prevention_protocol(
        env: Env,
        name: String,
        triggers: Vec<String>,
        risk_factors: Vec<String>,
        intervention_steps: Vec<String>,
        emergency_contacts: Vec<EmergencyContact>,
        resources: Vec<String>,
    ) -> Result<u64, Error> {
        Self::require_initialized(&env);
        suicide_prevention::SuicidePreventionManager::create_prevention_protocol(
            &env, name, triggers, risk_factors, intervention_steps, emergency_contacts, resources
        )
    }

    /// Detect suicide risk
    pub fn detect_suicide_risk(
        env: Env,
        patient_id: Address,
        indicators: Vec<String>,
        context_data: Map<String, String>,
    ) -> suicide_prevention::SuicideRiskAssessment {
        Self::require_initialized(&env);
        suicide_prevention::SuicidePreventionManager::detect_suicide_risk(
            &env, patient_id, indicators, context_data
        )
    }

    /// Get suicide prevention hotlines
    pub fn get_suicide_hotlines(env: Env) -> Vec<suicide_prevention::SuicideHotline> {
        Self::require_initialized(&env);
        suicide_prevention::SuicidePreventionManager::get_suicide_hotlines(&env)
    }

    /// Create safety plan
    pub fn create_safety_plan(
        env: Env,
        patient_id: Address,
        warning_signs: Vec<String>,
        coping_strategies: Vec<String>,
        reasons_to_live: Vec<String>,
        support_contacts: Vec<EmergencyContact>,
        professional_contacts: Vec<Address>,
    ) -> Result<u64, Error> {
        Self::require_initialized(&env);
        suicide_prevention::SuicidePreventionManager::create_safety_plan(
            &env, patient_id, warning_signs, coping_strategies, reasons_to_live,
            support_contacts, professional_contacts
        )
    }
}

impl MentalHealthPlatform {
    fn require_initialized(env: &Env) {
        if !env.storage().instance().has(&symbol_short!("init")) {
            panic!("Contract not initialized");
        }
    }

    fn require_user_registered(env: &Env, user: &Address) -> UserProfile {
        env.storage().instance().get(user).unwrap_or_else(|| {
            panic!("User not registered");
        })
    }

    fn check_permission(env: &Env, user: &Address, permission: Permission) -> bool {
        // Implementation would check against healthcare_compliance contract
        // For now, allow all registered users
        env.storage().instance().has(user)
    }
}