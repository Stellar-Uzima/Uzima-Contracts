#![no_std]

#[cfg(test)]
mod test;

use soroban_sdk::symbol_short;
use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype, vec, Address, BytesN, Env, Map, String,
    Symbol, Vec,
};

// ==================== Emergency Telemedicine Types ====================

/// Emergency Level
#[derive(Clone, Copy, PartialEq, Eq)]
#[contracttype]
pub enum EmergencyLevel {
    Low,
    Medium,
    High,
    Critical,
    LifeThreatening,
}

/// Emergency Type
#[derive(Clone, Copy, PartialEq, Eq)]
#[contracttype]
pub enum EmergencyType {
    Cardiac,
    Respiratory,
    Neurological,
    Trauma,
    Psychiatric,
    Obstetric,
    Pediatric,
    Toxicological,
    Metabolic,
    Allergic,
    Other,
}

/// Response Status
#[derive(Clone, Copy, PartialEq, Eq)]
#[contracttype]
pub enum ResponseStatus {
    Initiated,
    Responding,
    OnScene,
    Transporting,
    AtFacility,
    Resolved,
    Closed,
}

/// Triage Category
#[derive(Clone, Copy, PartialEq, Eq)]
#[contracttype]
pub enum TriageCategory {
    Immediate, // Red - Life-threatening
    Urgent,    // Yellow - Serious
    Delayed,   // Green - Non-urgent
    Minor,     // Blue - Minor
    Deceased,  // Black - Deceased
}

/// Emergency Protocol
#[derive(Clone)]
#[contracttype]
pub struct EmergencyProtocol {
    pub protocol_id: u64,
    pub emergency_type: EmergencyType,
    pub name: String,
    pub description: String,
    pub response_time_target: u32, // minutes
    pub assessment_steps: Vec<String>,
    pub interventions: Vec<String>,
    pub medications: Vec<String>,
    pub equipment_required: Vec<String>,
    pub specialist_required: bool,
    pub specialist_type: Option<String>,
    pub transport_required: bool,
    pub transport_level: String, // "BLS", "ALS", "Critical Care"
    pub documentation_required: Vec<String>,
    pub follow_up_required: bool,
    pub quality_metrics: Vec<String>,
    pub contraindications: Vec<String>,
    pub complications: Vec<String>,
    pub outcome_indicators: Vec<String>,
    pub created_at: u64,
    pub updated_at: u64,
    pub version: String,
    pub is_active: bool,
}

/// Emergency Session
#[derive(Clone)]
#[contracttype]
pub struct EmergencySession {
    pub session_id: u64,
    pub patient: Address,
    pub initiator: Address, // Who initiated the emergency
    pub emergency_type: EmergencyType,
    pub emergency_level: EmergencyLevel,
    pub triage_category: TriageCategory,
    pub chief_complaint: String,
    pub symptoms: Vec<String>,
    pub vital_signs: VitalSigns,
    pub medical_history: Vec<String>,
    pub allergies: Vec<String>,
    pub medications: Vec<String>,
    pub location: String, // GPS coordinates or address
    pub scene_safety: String,
    pub bystander_interventions: Vec<String>,
    pub protocol_id: u64,
    pub response_status: ResponseStatus,
    pub initiated_at: u64,
    pub first_response_at: Option<u64>,
    pub specialist_connected_at: Option<u64>,
    pub transport_dispatched_at: Option<u64>,
    pub arrived_at_facility_at: Option<u64>,
    pub resolved_at: Option<u64>,
    pub outcome: String,
    pub complications: Vec<String>,
    pub follow_up_plan: Vec<String>,
    pub quality_score: u8, // 0-100
    pub documentation_complete: bool,
    pub consent_obtained: bool,
    pub consent_token_id: u64,
}

/// Vital Signs
#[derive(Clone)]
#[contracttype]
pub struct VitalSigns {
    pub heart_rate: Option<u16>,
    pub blood_pressure_systolic: Option<u16>,
    pub blood_pressure_diastolic: Option<u16>,
    pub respiratory_rate: Option<u8>,
    pub oxygen_saturation: Option<u8>,
    pub temperature: Option<f32>,
    pub blood_glucose: Option<f32>,
    pub pain_score: Option<u8>,              // 0-10 scale
    pub consciousness_level: Option<String>, // "Alert", "Verbal", "Pain", "Unresponsive"
    pub pupil_reaction: Option<String>,
    pub skin_color: Option<String>,
    pub skin_temperature: Option<String>,
    pub capillary_refill: Option<String>,
    pub recorded_at: u64,
}

/// Emergency Response Team
#[derive(Clone)]
#[contracttype]
pub struct EmergencyResponseTeam {
    pub team_id: u64,
    pub session_id: u64,
    pub team_type: String, // "telemedicine", "ground_ambulance", "air_ambulance", "specialist"
    pub members: Vec<EmergencyTeamMember>,
    pub dispatch_time: u64,
    pub en_route_time: Option<u64>,
    pub on_scene_time: Option<u64>,
    pub transport_time: Option<u64>,
    pub arrival_time: Option<u64>,
    pub team_status: String, // "dispatched", "en_route", "on_scene", "transporting", "arrived"
    pub equipment_used: Vec<String>,
    pub interventions_performed: Vec<String>,
    pub medications_administered: Vec<MedicationAdministration>,
    pub communication_log: Vec<CommunicationEntry>,
    pub handover_summary: String,
}

/// Emergency Team Member
#[derive(Clone)]
#[contracttype]
pub struct EmergencyTeamMember {
    pub member_id: Address,
    pub role: String, // "physician", "nurse", "paramedic", "specialist", "technician"
    pub qualifications: Vec<String>,
    pub license_number: String,
    pub contact_info: String,
    pub availability: bool,
    pub current_location: String,
    pub specialization: Option<String>,
    pub experience_years: u16,
    pub certifications: Vec<String>,
}

/// Medication Administration
#[derive(Clone)]
#[contracttype]
pub struct MedicationAdministration {
    pub medication_name: String,
    pub dosage: String,
    pub route: String, // "IV", "IM", "PO", "SubQ", "IN", "Topical"
    pub administered_by: Address,
    pub administered_at: u64,
    pub indication: String,
    pub reaction: Option<String>,
    pub effectiveness: Option<String>,
}

/// Communication Entry
#[derive(Clone)]
#[contracttype]
pub struct CommunicationEntry {
    pub timestamp: u64,
    pub sender: Address,
    pub recipient: Address,
    pub message_type: String, // "voice", "text", "video", "data"
    pub content: String,
    pub priority: String,         // "routine", "urgent", "critical"
    pub delivery_status: String,  // "sent", "delivered", "read", "acknowledged"
    pub attachments: Vec<String>, // IPFS hashes
}

/// Emergency Resource
#[derive(Clone)]
#[contracttype]
pub struct EmergencyResource {
    pub resource_id: u64,
    pub resource_type: String, // "ambulance", "helicopter", "specialist", "equipment", "facility"
    pub name: String,
    pub location: String,
    pub status: String, // "available", "busy", "offline", "maintenance"
    pub capabilities: Vec<String>,
    pub capacity: u8,
    pub current_load: u8,
    pub response_time_estimate: u32, // minutes
    pub cost_per_use: Option<u64>,
    pub currency: Option<String>,
    pub contact_info: String,
    pub operating_hours: String,
    pub service_area: String,
    pub last_updated: u64,
}

/// Emergency Alert
#[derive(Clone)]
#[contracttype]
pub struct EmergencyAlert {
    pub alert_id: u64,
    pub session_id: u64,
    pub alert_type: String, // "vital_signs_deterioration", "no_response", "equipment_failure", "communication_lost"
    pub severity: String,   // "low", "medium", "high", "critical"
    pub message: String,
    pub triggered_by: Address,
    pub triggered_at: u64,
    pub acknowledged_by: Option<Address>,
    pub acknowledged_at: Option<u64>,
    pub resolved_by: Option<Address>,
    pub resolved_at: Option<u64>,
    pub resolution_actions: Vec<String>,
    pub escalation_level: u8, // 0-5
    pub notifications_sent: Vec<Address>,
}

/// Emergency Quality Metric
#[derive(Clone)]
#[contracttype]
pub struct EmergencyQualityMetric {
    pub metric_id: u64,
    pub session_id: u64,
    pub metric_name: String,
    pub category: String, // "response_time", "clinical_outcome", "documentation", "communication"
    pub target_value: f32,
    pub actual_value: f32,
    pub achievement_rate: u8, // 0-100
    pub variance_reason: String,
    pub benchmark_comparison: Option<f32>,
    pub impact_assessment: String,
    pub recorded_at: u64,
}

/// Emergency Statistics
#[derive(Clone)]
#[contracttype]
pub struct EmergencyStatistics {
    pub period_start: u64,
    pub period_end: u64,
    pub total_emergencies: u32,
    pub by_type: Map<EmergencyType, u32>,
    pub by_level: Map<EmergencyLevel, u32>,
    pub by_triage: Map<TriageCategory, u32>,
    pub average_response_time: f32,             // minutes
    pub average_on_scene_time: f32,             // minutes
    pub average_transport_time: f32,            // minutes
    pub outcomes: Map<String, u32>,             // outcome -> count
    pub complications: Map<String, u32>,        // complication -> count
    pub quality_scores: Map<String, f32>,       // metric_category -> average_score
    pub resource_utilization: Map<String, f32>, // resource_type -> utilization_rate
    pub satisfaction_scores: Map<String, f32>,  // stakeholder -> satisfaction_score
    pub cost_analysis: CostAnalysis,
}

/// Cost Analysis
#[derive(Clone)]
#[contracttype]
pub struct CostAnalysis {
    pub total_cost: u64,
    pub currency: String,
    pub cost_per_session: f64,
    pub cost_by_type: Map<EmergencyType, u64>,
    pub cost_by_level: Map<EmergencyLevel, u64>,
    pub resource_costs: Map<String, u64>,
    pub personnel_costs: u64,
    pub equipment_costs: u64,
    pub transport_costs: u64,
    pub facility_costs: u64,
    pub medication_costs: u64,
    pub overhead_costs: u64,
}

// Storage Keys
const ADMIN: Symbol = symbol_short!("ADMIN");
const EMERGENCY_PROTOCOLS: Symbol = symbol_short!("PROTOCOLS");
const EMERGENCY_SESSIONS: Symbol = symbol_short!("SESSIONS");
const RESPONSE_TEAMS: Symbol = symbol_short!("TEAMS");
const EMERGENCY_RESOURCES: Symbol = symbol_short!("RESOURCES");
const EMERGENCY_ALERTS: Symbol = symbol_short!("ALERTS");
const QUALITY_METRICS: Symbol = symbol_short!("METRICS");
const EMERGENCY_STATISTICS: Symbol = symbol_short!("STATISTICS");
const PROTOCOL_COUNTER: Symbol = symbol_short!("PROTOCOL_CNT");
const SESSION_COUNTER: Symbol = symbol_short!("SESSION_CNT");
const TEAM_COUNTER: Symbol = symbol_short!("TEAM_CNT");
const RESOURCE_COUNTER: Symbol = symbol_short!("RESOURCE_CNT");
const ALERT_COUNTER: Symbol = symbol_short!("ALERT_CNT");
const METRIC_COUNTER: Symbol = symbol_short!("METRIC_CNT");
const PAUSED: Symbol = symbol_short!("PAUSED");
const CONSENT_CONTRACT: Symbol = symbol_short!("CONSENT");
const MEDICAL_RECORDS_CONTRACT: Symbol = symbol_short!("MEDICAL");

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    NotAuthorized = 1,
    ContractPaused = 2,
    ProtocolNotFound = 3,
    SessionNotFound = 4,
    ResourceNotFound = 5,
    AlertNotFound = 6,
    InvalidEmergencyType = 7,
    InvalidEmergencyLevel = 8,
    InvalidVitalSigns = 9,
    ConsentRequired = 10,
    ConsentRevoked = 11,
    ResourceUnavailable = 12,
    SpecialistUnavailable = 13,
    TransportUnavailable = 14,
    DocumentationIncomplete = 15,
    QualityThresholdBreached = 16,
    ResponseTimeExceeded = 17,
    MedicalRecordsContractNotSet = 18,
    ConsentContractNotSet = 19,
}

#[contract]
pub struct EmergencyTelemedicineContract;

#[contractimpl]
impl EmergencyTelemedicineContract {
    /// Initialize the emergency telemedicine contract
    pub fn initialize(
        env: Env,
        admin: Address,
        consent_contract: Address,
        medical_records_contract: Address,
    ) -> Result<bool, Error> {
        admin.require_auth();

        if env.storage().persistent().has(&ADMIN) {
            return Err(Error::ProtocolNotFound);
        }

        env.storage().persistent().set(&ADMIN, &admin);
        env.storage()
            .persistent()
            .set(&CONSENT_CONTRACT, &consent_contract);
        env.storage()
            .persistent()
            .set(&MEDICAL_RECORDS_CONTRACT, &medical_records_contract);
        env.storage().persistent().set(&PAUSED, &false);
        env.storage().persistent().set(&PROTOCOL_COUNTER, &0u64);
        env.storage().persistent().set(&SESSION_COUNTER, &0u64);
        env.storage().persistent().set(&TEAM_COUNTER, &0u64);
        env.storage().persistent().set(&RESOURCE_COUNTER, &0u64);
        env.storage().persistent().set(&ALERT_COUNTER, &0u64);
        env.storage().persistent().set(&METRIC_COUNTER, &0u64);

        // Initialize standard emergency protocols
        Self::initialize_emergency_protocols(&env)?;

        Ok(true)
    }

    /// Create emergency protocol
    pub fn create_emergency_protocol(
        env: Env,
        admin: Address,
        emergency_type: EmergencyType,
        name: String,
        description: String,
        response_time_target: u32,
        assessment_steps: Vec<String>,
        interventions: Vec<String>,
        medications: Vec<String>,
        equipment_required: Vec<String>,
        specialist_required: bool,
        specialist_type: Option<String>,
        transport_required: bool,
        transport_level: String,
        documentation_required: Vec<String>,
        follow_up_required: bool,
        quality_metrics: Vec<String>,
        contraindications: Vec<String>,
        complications: Vec<String>,
        outcome_indicators: Vec<String>,
    ) -> Result<u64, Error> {
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

        let protocol_id = Self::get_and_increment_protocol_counter(&env);
        let timestamp = env.ledger().timestamp();

        let protocol = EmergencyProtocol {
            protocol_id,
            emergency_type,
            name: name.clone(),
            description,
            response_time_target,
            assessment_steps,
            interventions,
            medications,
            equipment_required,
            specialist_required,
            specialist_type,
            transport_required,
            transport_level,
            documentation_required,
            follow_up_required,
            quality_metrics,
            contraindications,
            complications,
            outcome_indicators,
            created_at: timestamp,
            updated_at: timestamp,
            version: "1.0".to_string(),
            is_active: true,
        };

        let mut protocols: Map<u64, EmergencyProtocol> = env
            .storage()
            .persistent()
            .get(&EMERGENCY_PROTOCOLS)
            .unwrap_or(Map::new(&env));
        protocols.set(protocol_id, protocol);
        env.storage()
            .persistent()
            .set(&EMERGENCY_PROTOCOLS, &protocols);

        // Emit event
        env.events().publish(
            (symbol_short!("Protocol"), symbol_short!("Created")),
            (protocol_id, name),
        );

        Ok(protocol_id)
    }

    /// Initiate emergency session
    pub fn initiate_emergency_session(
        env: Env,
        initiator: Address,
        patient: Address,
        emergency_type: EmergencyType,
        emergency_level: EmergencyLevel,
        chief_complaint: String,
        symptoms: Vec<String>,
        vital_signs: VitalSigns,
        medical_history: Vec<String>,
        allergies: Vec<String>,
        medications: Vec<String>,
        location: String,
        scene_safety: String,
        bystander_interventions: Vec<String>,
        consent_token_id: u64,
    ) -> Result<u64, Error> {
        initiator.require_auth();

        if env.storage().persistent().get(&PAUSED).unwrap_or(false) {
            return Err(Error::ContractPaused);
        }

        // Verify consent
        if !Self::verify_consent_token(&env, consent_token_id, patient.clone(), initiator.clone())?
        {
            return Err(Error::ConsentRequired);
        }

        // Get appropriate protocol
        let protocol_id = Self::get_protocol_for_emergency(&env, emergency_type)?;
        let protocols: Map<u64, EmergencyProtocol> = env
            .storage()
            .persistent()
            .get(&EMERGENCY_PROTOCOLS)
            .ok_or(Error::ProtocolNotFound)?;

        let protocol = protocols.get(protocol_id).ok_or(Error::ProtocolNotFound)?;

        // Perform triage
        let triage_category = Self::perform_triage(&env, emergency_level, &vital_signs, &symptoms)?;

        let session_id = Self::get_and_increment_session_counter(&env);
        let timestamp = env.ledger().timestamp();

        let session = EmergencySession {
            session_id,
            patient: patient.clone(),
            initiator: initiator.clone(),
            emergency_type,
            emergency_level,
            triage_category,
            chief_complaint,
            symptoms,
            vital_signs,
            medical_history,
            allergies,
            medications,
            location,
            scene_safety,
            bystander_interventions,
            protocol_id,
            response_status: ResponseStatus::Initiated,
            initiated_at: timestamp,
            first_response_at: None,
            specialist_connected_at: None,
            transport_dispatched_at: None,
            arrived_at_facility_at: None,
            resolved_at: None,
            outcome: String::from_str(&env, ""),
            complications: Vec::new(&env),
            follow_up_plan: Vec::new(&env),
            quality_score: 0,
            documentation_complete: false,
            consent_obtained: true,
            consent_token_id,
        };

        let mut sessions: Map<u64, EmergencySession> = env
            .storage()
            .persistent()
            .get(&EMERGENCY_SESSIONS)
            .unwrap_or(Map::new(&env));
        sessions.set(session_id, session);
        env.storage()
            .persistent()
            .set(&EMERGENCY_SESSIONS, &sessions);

        // Initiate emergency response
        Self::initiate_emergency_response(&env, session_id, patient.clone(), &protocol)?;

        // Create quality metrics tracking
        Self::create_quality_metrics(&env, session_id, &protocol)?;

        // Emit event
        env.events().publish(
            (symbol_short!("Emergency"), symbol_short!("Initiated")),
            (session_id, patient, emergency_type),
        );

        Ok(session_id)
    }

    /// Update emergency session status
    pub fn update_session_status(
        env: Env,
        session_id: u64,
        responder: Address,
        new_status: ResponseStatus,
        notes: String,
    ) -> Result<bool, Error> {
        responder.require_auth();

        if env.storage().persistent().get(&PAUSED).unwrap_or(false) {
            return Err(Error::ContractPaused);
        }

        let mut sessions: Map<u64, EmergencySession> = env
            .storage()
            .persistent()
            .get(&EMERGENCY_SESSIONS)
            .ok_or(Error::SessionNotFound)?;

        let mut session = sessions.get(session_id).ok_or(Error::SessionNotFound)?;

        // Validate status transition
        if !Self::is_valid_status_transition(session.response_status, new_status) {
            return Err(Error::SessionNotFound); // Using existing error for simplicity
        }

        let timestamp = env.ledger().timestamp();

        // Update timestamps based on status
        match new_status {
            ResponseStatus::Responding => {
                if session.first_response_at.is_none() {
                    session.first_response_at = Some(timestamp);
                }
            }
            ResponseStatus::OnScene => {
                // Would update on_scene_time in response team
            }
            ResponseStatus::AtFacility => {
                session.arrived_at_facility_at = Some(timestamp);
            }
            ResponseStatus::Resolved => {
                session.resolved_at = Some(timestamp);
                session.outcome = notes;
            }
            _ => {}
        }

        session.response_status = new_status;

        // Check response time compliance
        if let Some(first_response) = session.first_response_at {
            let response_time = (first_response - session.initiated_at) / 60; // Convert to minutes
            let protocols: Map<u64, EmergencyProtocol> = env
                .storage()
                .persistent()
                .get(&EMERGENCY_PROTOCOLS)
                .ok_or(Error::ProtocolNotFound)?;

            let protocol = protocols
                .get(session.protocol_id)
                .ok_or(Error::ProtocolNotFound)?;

            if response_time > protocol.response_time_target as u64 {
                Self::create_response_time_alert(
                    &env,
                    session_id,
                    response_time,
                    protocol.response_time_target,
                )?;
            }
        }

        sessions.set(session_id, session);
        env.storage()
            .persistent()
            .set(&EMERGENCY_SESSIONS, &sessions);

        // Emit event
        env.events().publish(
            (symbol_short!("Session"), symbol_short!("StatusUpdated")),
            (session_id, new_status),
        );

        Ok(true)
    }

    /// Register emergency resource
    pub fn register_emergency_resource(
        env: Env,
        admin: Address,
        resource_type: String,
        name: String,
        location: String,
        capabilities: Vec<String>,
        capacity: u8,
        response_time_estimate: u32,
        cost_per_use: Option<u64>,
        currency: Option<String>,
        contact_info: String,
        operating_hours: String,
        service_area: String,
    ) -> Result<u64, Error> {
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

        let resource_id = Self::get_and_increment_resource_counter(&env);
        let timestamp = env.ledger().timestamp();

        let resource = EmergencyResource {
            resource_id,
            resource_type: resource_type.clone(),
            name: name.clone(),
            location,
            status: "available".to_string(),
            capabilities,
            capacity,
            current_load: 0,
            response_time_estimate,
            cost_per_use,
            currency,
            contact_info,
            operating_hours,
            service_area,
            last_updated: timestamp,
        };

        let mut resources: Map<u64, EmergencyResource> = env
            .storage()
            .persistent()
            .get(&EMERGENCY_RESOURCES)
            .unwrap_or(Map::new(&env));
        resources.set(resource_id, resource);
        env.storage()
            .persistent()
            .set(&EMERGENCY_RESOURCES, &resources);

        // Emit event
        env.events().publish(
            (symbol_short!("Resource"), symbol_short!("Registered")),
            (resource_id, name),
        );

        Ok(resource_id)
    }

    /// Dispatch emergency resource
    pub fn dispatch_resource(
        env: Env,
        session_id: u64,
        resource_id: u64,
        dispatcher: Address,
        priority: String,
    ) -> Result<bool, Error> {
        dispatcher.require_auth();

        if env.storage().persistent().get(&PAUSED).unwrap_or(false) {
            return Err(Error::ContractPaused);
        }

        // Check resource availability
        let mut resources: Map<u64, EmergencyResource> = env
            .storage()
            .persistent()
            .get(&EMERGENCY_RESOURCES)
            .ok_or(Error::ResourceNotFound)?;

        let mut resource = resources.get(resource_id).ok_or(Error::ResourceNotFound)?;

        if resource.status != "available" || resource.current_load >= resource.capacity {
            return Err(Error::ResourceUnavailable);
        }

        // Update resource status
        resource.current_load += 1;
        if resource.current_load >= resource.capacity {
            resource.status = "busy".to_string();
        }
        resource.last_updated = env.ledger().timestamp();

        resources.set(resource_id, resource);
        env.storage()
            .persistent()
            .set(&EMERGENCY_RESOURCES, &resources);

        // Create response team
        Self::create_response_team(&env, session_id, resource_id, dispatcher)?;

        // Emit event
        env.events().publish(
            (symbol_short!("Resource"), symbol_short!("Dispatched")),
            (resource_id, session_id),
        );

        Ok(true)
    }

    /// Record vital signs update
    pub fn record_vital_signs(
        env: Env,
        session_id: u64,
        recorder: Address,
        vital_signs: VitalSigns,
    ) -> Result<bool, Error> {
        recorder.require_auth();

        if env.storage().persistent().get(&PAUSED).unwrap_or(false) {
            return Err(Error::ContractPaused);
        }

        let mut sessions: Map<u64, EmergencySession> = env
            .storage()
            .persistent()
            .get(&EMERGENCY_SESSIONS)
            .ok_or(Error::SessionNotFound)?;

        let mut session = sessions.get(session_id).ok_or(Error::SessionNotFound)?;

        session.vital_signs = vital_signs.clone();

        // Check for deterioration
        if Self::check_vital_signs_deterioration(&vital_signs) {
            Self::create_vital_signs_alert(&env, session_id, &vital_signs)?;
        }

        sessions.set(session_id, session);
        env.storage()
            .persistent()
            .set(&EMERGENCY_SESSIONS, &sessions);

        // Emit event
        env.events().publish(
            (symbol_short!("Vitals"), symbol_short!("Recorded")),
            (session_id),
        );

        Ok(true)
    }

    /// Complete emergency session
    pub fn complete_emergency_session(
        env: Env,
        session_id: u64,
        provider: Address,
        outcome: String,
        complications: Vec<String>,
        follow_up_plan: Vec<String>,
        quality_score: u8,
    ) -> Result<bool, Error> {
        provider.require_auth();

        if env.storage().persistent().get(&PAUSED).unwrap_or(false) {
            return Err(Error::ContractPaused);
        }

        let mut sessions: Map<u64, EmergencySession> = env
            .storage()
            .persistent()
            .get(&EMERGENCY_SESSIONS)
            .ok_or(Error::SessionNotFound)?;

        let mut session = sessions.get(session_id).ok_or(Error::SessionNotFound)?;

        session.response_status = ResponseStatus::Resolved;
        session.resolved_at = Some(env.ledger().timestamp());
        session.outcome = outcome;
        session.complications = complications;
        session.follow_up_plan = follow_up_plan;
        session.quality_score = quality_score;
        session.documentation_complete = true;

        sessions.set(session_id, session);
        env.storage()
            .persistent()
            .set(&EMERGENCY_SESSIONS, &sessions);

        // Update final quality metrics
        Self::update_final_quality_metrics(&env, session_id, quality_score)?;

        // Release resources
        Self::release_session_resources(&env, session_id)?;

        // Emit event
        env.events().publish(
            (symbol_short!("Emergency"), symbol_short!("Completed")),
            (session_id, outcome),
        );

        Ok(true)
    }

    /// Generate emergency statistics
    pub fn generate_emergency_statistics(
        env: Env,
        period_start: u64,
        period_end: u64,
    ) -> Result<u64, Error> {
        if env.storage().persistent().get(&PAUSED).unwrap_or(false) {
            return Err(Error::ContractPaused);
        }

        let timestamp = env.ledger().timestamp();
        let sessions: Map<u64, EmergencySession> = env
            .storage()
            .persistent()
            .get(&EMERGENCY_SESSIONS)
            .unwrap_or(Map::new(&env));

        // Calculate statistics
        let mut total_emergencies = 0u32;
        let mut by_type = Map::new(&env);
        let mut by_level = Map::new(&env);
        let mut by_triage = Map::new(&env);
        let mut response_times = Vec::new(&env);
        let mut outcomes = Map::new(&env);

        for session in sessions.values() {
            if session.initiated_at >= period_start && session.initiated_at <= period_end {
                total_emergencies += 1;

                // Count by type
                let type_count = by_type.get(session.emergency_type).unwrap_or(0u32);
                by_type.set(session.emergency_type, type_count + 1);

                // Count by level
                let level_count = by_level.get(session.emergency_level).unwrap_or(0u32);
                by_level.set(session.emergency_level, level_count + 1);

                // Count by triage
                let triage_count = by_triage.get(session.triage_category).unwrap_or(0u32);
                by_triage.set(session.triage_category, triage_count + 1);

                // Response times
                if let Some(first_response) = session.first_response_at {
                    let response_time = (first_response - session.initiated_at) as f32 / 60.0;
                    response_times.push_back(response_time);
                }

                // Outcomes
                if !session.outcome.is_empty() {
                    let outcome_count = outcomes.get(session.outcome.clone()).unwrap_or(0u32);
                    outcomes.set(session.outcome, outcome_count + 1);
                }
            }
        }

        let average_response_time = if response_times.is_empty() {
            0.0
        } else {
            let total: f32 = response_times.iter().sum();
            total / response_times.len() as f32
        };

        let statistics = EmergencyStatistics {
            period_start,
            period_end,
            total_emergencies,
            by_type,
            by_level,
            by_triage,
            average_response_time,
            average_on_scene_time: 0.0,  // Would calculate from team data
            average_transport_time: 0.0, // Would calculate from team data
            outcomes,
            complications: Map::new(&env),        // Would calculate
            quality_scores: Map::new(&env),       // Would calculate
            resource_utilization: Map::new(&env), // Would calculate
            satisfaction_scores: Map::new(&env),  // Would calculate
            cost_analysis: CostAnalysis {
                total_cost: 0,
                currency: "USD".to_string(),
                cost_per_session: 0.0,
                cost_by_type: Map::new(&env),
                cost_by_level: Map::new(&env),
                resource_costs: Map::new(&env),
                personnel_costs: 0,
                equipment_costs: 0,
                transport_costs: 0,
                facility_costs: 0,
                medication_costs: 0,
                overhead_costs: 0,
            },
        };

        let mut statistics_storage: Map<u64, EmergencyStatistics> = env
            .storage()
            .persistent()
            .get(&EMERGENCY_STATISTICS)
            .unwrap_or(Map::new(&env));
        statistics_storage.set(timestamp, statistics);
        env.storage()
            .persistent()
            .set(&EMERGENCY_STATISTICS, &statistics_storage);

        // Emit event
        env.events().publish(
            (symbol_short!("Statistics"), symbol_short!("Generated")),
            (timestamp, total_emergencies),
        );

        Ok(timestamp)
    }

    /// Get emergency session
    pub fn get_emergency_session(env: Env, session_id: u64) -> Result<EmergencySession, Error> {
        let sessions: Map<u64, EmergencySession> = env
            .storage()
            .persistent()
            .get(&EMERGENCY_SESSIONS)
            .ok_or(Error::SessionNotFound)?;

        sessions.get(session_id).ok_or(Error::SessionNotFound)
    }

    /// Get emergency protocol
    pub fn get_emergency_protocol(env: Env, protocol_id: u64) -> Result<EmergencyProtocol, Error> {
        let protocols: Map<u64, EmergencyProtocol> = env
            .storage()
            .persistent()
            .get(&EMERGENCY_PROTOCOLS)
            .ok_or(Error::ProtocolNotFound)?;

        protocols.get(protocol_id).ok_or(Error::ProtocolNotFound)
    }

    /// Get emergency resource
    pub fn get_emergency_resource(env: Env, resource_id: u64) -> Result<EmergencyResource, Error> {
        let resources: Map<u64, EmergencyResource> = env
            .storage()
            .persistent()
            .get(&EMERGENCY_RESOURCES)
            .ok_or(Error::ResourceNotFound)?;

        resources.get(resource_id).ok_or(Error::ResourceNotFound)
    }

    /// Get available resources
    pub fn get_available_resources(
        env: Env,
        resource_type: String,
        location: String,
    ) -> Result<Vec<EmergencyResource>, Error> {
        let resources: Map<u64, EmergencyResource> = env
            .storage()
            .persistent()
            .get(&EMERGENCY_RESOURCES)
            .unwrap_or(Map::new(&env));

        let mut available = Vec::new(&env);

        for resource in resources.values() {
            if resource.resource_type == resource_type
                && resource.status == "available"
                && (location.is_empty() || resource.service_area.contains(&location))
            {
                available.push_back(resource);
            }
        }

        Ok(available)
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

    fn initialize_emergency_protocols(env: &Env) -> Result<(), Error> {
        let timestamp = env.ledger().timestamp();

        // Cardiac Emergency Protocol
        let protocol_id = Self::get_and_increment_protocol_counter(env);
        let cardiac_protocol = EmergencyProtocol {
            protocol_id,
            emergency_type: EmergencyType::Cardiac,
            name: "Cardiac Arrest Emergency".to_string(),
            description: "Protocol for managing cardiac emergencies including cardiac arrest, chest pain, and arrhythmias".to_string(),
            response_time_target: 8, // 8 minutes
            assessment_steps: vec![env, "Assess consciousness".to_string(), "Check breathing".to_string(), "Check pulse".to_string(), "Attach monitor".to_string()],
            interventions: vec![env, "CPR".to_string(), "Defibrillation".to_string(), "Airway management".to_string(), "IV access".to_string()],
            medications: vec![env, "Epinephrine".to_string(), "Amiodarone".to_string(), "Atropine".to_string(), "Lidocaine".to_string()],
            equipment_required: vec![env, "Defibrillator".to_string(), "Cardiac monitor".to_string(), "Airway kit".to_string(), "IV supplies".to_string()],
            specialist_required: true,
            specialist_type: Some("Cardiologist".to_string()),
            transport_required: true,
            transport_level: "Critical Care".to_string(),
            documentation_required: vec![env, "Time stamps".to_string(), "Interventions".to_string(), "Medications".to_string(), "Rhythm strips".to_string()],
            follow_up_required: true,
            quality_metrics: vec![env, "Response time".to_string(), "ROSC rate".to_string(), "Documentation completeness".to_string()],
            contraindications: vec![env, "Do not use defibrillator in water".to_string()],
            complications: vec![env, "Aspiration".to_string(), "Rib fractures".to_string()],
            outcome_indicators: vec![env, "ROSC".to_string(), "Survival to discharge".to_string(), "Neurological outcome".to_string()],
            created_at: timestamp,
            updated_at: timestamp,
            version: "1.0".to_string(),
            is_active: true,
        };

        let mut protocols: Map<u64, EmergencyProtocol> = env
            .storage()
            .persistent()
            .get(&EMERGENCY_PROTOCOLS)
            .unwrap_or(Map::new(env));
        protocols.set(protocol_id, cardiac_protocol);
        env.storage()
            .persistent()
            .set(&EMERGENCY_PROTOCOLS, &protocols);

        // Respiratory Emergency Protocol
        let protocol_id = Self::get_and_increment_protocol_counter(env);
        let respiratory_protocol = EmergencyProtocol {
            protocol_id,
            emergency_type: EmergencyType::Respiratory,
            name: "Respiratory Distress Emergency".to_string(),
            description: "Protocol for managing respiratory emergencies including asthma, COPD exacerbation, and airway obstruction".to_string(),
            response_time_target: 10, // 10 minutes
            assessment_steps: vec![env, "Assess airway".to_string(), "Check breathing effort".to_string(), "Measure oxygen saturation".to_string(), "Assess work of breathing".to_string()],
            interventions: vec![env, "Oxygen therapy".to_string(), "Nebulized medications".to_string(), "CPAP/BiPAP".to_string(), "Intubation".to_string()],
            medications: vec![env, "Albuterol".to_string(), "Ipratropium".to_string(), "Steroids".to_string(), "Magnesium".to_string()],
            equipment_required: vec![env, "Oxygen tank".to_string(), "Nebulizer".to_string(), "CPAP machine".to_string(), "Intubation kit".to_string()],
            specialist_required: false,
            specialist_type: None,
            transport_required: true,
            transport_level: "ALS".to_string(),
            documentation_required: vec![env, "Oxygen saturation".to_string(), "Breath sounds".to_string(), "Medication response".to_string()],
            follow_up_required: true,
            quality_metrics: vec![env, "Oxygen saturation improvement".to_string(), "Intubation success rate".to_string()],
            contraindications: vec![env, "Avoid high oxygen in COPD".to_string()],
            complications: vec![env, "Barotrauma".to_string(), "Hypotension".to_string()],
            outcome_indicators: vec![env, "Respiratory stability".to_string(), "Avoided intubation".to_string(), "Length of stay".to_string()],
            created_at: timestamp,
            updated_at: timestamp,
            version: "1.0".to_string(),
            is_active: true,
        };

        protocols.set(protocol_id, respiratory_protocol);
        env.storage()
            .persistent()
            .set(&EMERGENCY_PROTOCOLS, &protocols);

        Ok(())
    }

    fn get_protocol_for_emergency(env: &Env, emergency_type: EmergencyType) -> Result<u64, Error> {
        let protocols: Map<u64, EmergencyProtocol> = env
            .storage()
            .persistent()
            .get(&EMERGENCY_PROTOCOLS)
            .unwrap_or(Map::new(env));

        for (protocol_id, protocol) in protocols.iter() {
            if protocol.emergency_type == emergency_type && protocol.is_active {
                return Ok(*protocol_id);
            }
        }

        Err(Error::ProtocolNotFound)
    }

    fn perform_triage(
        env: &Env,
        emergency_level: EmergencyLevel,
        vital_signs: &VitalSigns,
        symptoms: &Vec<String>,
    ) -> Result<TriageCategory, Error> {
        // Simplified triage algorithm
        match emergency_level {
            EmergencyLevel::LifeThreatening => Ok(TriageCategory::Immediate),
            EmergencyLevel::Critical => {
                // Check vital signs for immediate category
                if let Some(o2_sat) = vital_signs.oxygen_saturation {
                    if o2_sat < 85 {
                        return Ok(TriageCategory::Immediate);
                    }
                }
                if let Some(heart_rate) = vital_signs.heart_rate {
                    if heart_rate < 40 || heart_rate > 140 {
                        return Ok(TriageCategory::Immediate);
                    }
                }
                Ok(TriageCategory::Urgent)
            }
            EmergencyLevel::High => Ok(TriageCategory::Urgent),
            EmergencyLevel::Medium => Ok(TriageCategory::Delayed),
            EmergencyLevel::Low => Ok(TriageCategory::Minor),
        }
    }

    fn initiate_emergency_response(
        env: &Env,
        session_id: u64,
        patient: Address,
        protocol: &EmergencyProtocol,
    ) -> Result<(), Error> {
        // Find and dispatch appropriate resources
        let resources: Map<u64, EmergencyResource> = env
            .storage()
            .persistent()
            .get(&EMERGENCY_RESOURCES)
            .unwrap_or(Map::new(env));

        // Find nearest available ambulance
        let mut best_resource = None;
        let mut best_distance = f64::MAX;

        for resource in resources.values() {
            if resource.resource_type == "ambulance" && resource.status == "available" {
                // Simplified distance calculation - would use actual geolocation
                let distance = 10.0; // Placeholder
                if distance < best_distance {
                    best_distance = distance;
                    best_resource = Some(resource);
                }
            }
        }

        if let Some(resource) = best_resource {
            Self::dispatch_resource(
                env,
                session_id,
                resource.resource_id,
                patient,
                "urgent".to_string(),
            )?;
        }

        // Request specialist if required
        if protocol.specialist_required {
            Self::request_specialist(env, session_id, protocol.specialist_type.clone())?;
        }

        Ok(())
    }

    fn create_quality_metrics(
        env: &Env,
        session_id: u64,
        protocol: &EmergencyProtocol,
    ) -> Result<(), Error> {
        for metric_name in protocol.quality_metrics.iter() {
            let metric_id = Self::get_and_increment_metric_counter(env);

            let quality_metric = EmergencyQualityMetric {
                metric_id,
                session_id,
                metric_name: metric_name.clone(),
                category: "response_time".to_string(), // Would categorize properly
                target_value: protocol.response_time_target as f32,
                actual_value: 0.0, // To be updated as session progresses
                achievement_rate: 0,
                variance_reason: String::from_str(env, ""),
                benchmark_comparison: Some(protocol.response_time_target as f32),
                impact_assessment: String::from_str(env, ""),
                recorded_at: env.ledger().timestamp(),
            };

            let mut metrics: Map<u64, EmergencyQualityMetric> = env
                .storage()
                .persistent()
                .get(&QUALITY_METRICS)
                .unwrap_or(Map::new(env));
            metrics.set(metric_id, quality_metric);
            env.storage().persistent().set(&QUALITY_METRICS, &metrics);
        }

        Ok(())
    }

    fn create_response_team(
        env: &Env,
        session_id: u64,
        resource_id: u64,
        dispatcher: Address,
    ) -> Result<(), Error> {
        let team_id = Self::get_and_increment_team_counter(env);
        let timestamp = env.ledger().timestamp();

        let team = EmergencyResponseTeam {
            team_id,
            session_id,
            team_type: "ground_ambulance".to_string(),
            members: Vec::new(env), // Would populate with actual team members
            dispatch_time: timestamp,
            en_route_time: None,
            on_scene_time: None,
            transport_time: None,
            arrival_time: None,
            team_status: "dispatched".to_string(),
            equipment_used: Vec::new(env),
            interventions_performed: Vec::new(env),
            medications_administered: Vec::new(env),
            communication_log: Vec::new(env),
            handover_summary: String::from_str(env, ""),
        };

        let mut teams: Map<u64, EmergencyResponseTeam> = env
            .storage()
            .persistent()
            .get(&RESPONSE_TEAMS)
            .unwrap_or(Map::new(env));
        teams.set(team_id, team);
        env.storage().persistent().set(&RESPONSE_TEAMS, &teams);

        Ok(())
    }

    fn request_specialist(
        env: &Env,
        session_id: u64,
        specialist_type: Option<String>,
    ) -> Result<(), Error> {
        // This would find and connect appropriate specialist
        // For now, just emit an event
        env.events().publish(
            (symbol_short!("Specialist"), symbol_short!("Requested")),
            (session_id, specialist_type.unwrap_or("general".to_string())),
        );

        Ok(())
    }

    fn is_valid_status_transition(current: ResponseStatus, new: ResponseStatus) -> bool {
        match (current, new) {
            (ResponseStatus::Initiated, ResponseStatus::Responding) => true,
            (ResponseStatus::Responding, ResponseStatus::OnScene) => true,
            (ResponseStatus::OnScene, ResponseStatus::Transporting) => true,
            (ResponseStatus::Transporting, ResponseStatus::AtFacility) => true,
            (ResponseStatus::AtFacility, ResponseStatus::Resolved) => true,
            (ResponseStatus::AtFacility, ResponseStatus::Closed) => true,
            (ResponseStatus::Resolved, ResponseStatus::Closed) => true,
            _ => false,
        }
    }

    fn create_response_time_alert(
        env: &Env,
        session_id: u64,
        actual_time: u64,
        target_time: u32,
    ) -> Result<(), Error> {
        let alert_id = Self::get_and_increment_alert_counter(env);
        let timestamp = env.ledger().timestamp();

        let alert = EmergencyAlert {
            alert_id,
            session_id,
            alert_type: "response_time_exceeded".to_string(),
            severity: if actual_time > target_time as u64 * 2 {
                "critical"
            } else {
                "high"
            }
            .to_string(),
            message: format!(
                "Response time {} minutes exceeded target of {} minutes",
                actual_time, target_time
            ),
            triggered_by: Address::from_array(env, &[0u8; 32]), // System triggered
            triggered_at: timestamp,
            acknowledged_by: None,
            acknowledged_at: None,
            resolved_by: None,
            resolved_at: None,
            resolution_actions: Vec::new(env),
            escalation_level: 3,
            notifications_sent: Vec::new(env),
        };

        let mut alerts: Map<u64, EmergencyAlert> = env
            .storage()
            .persistent()
            .get(&EMERGENCY_ALERTS)
            .unwrap_or(Map::new(env));
        alerts.set(alert_id, alert);
        env.storage().persistent().set(&EMERGENCY_ALERTS, &alerts);

        Ok(())
    }

    fn check_vital_signs_deterioration(vital_signs: &VitalSigns) -> bool {
        // Check for concerning vital sign patterns
        if let Some(o2_sat) = vital_signs.oxygen_saturation {
            if o2_sat < 85 {
                return true;
            }
        }

        if let Some(heart_rate) = vital_signs.heart_rate {
            if heart_rate < 40 || heart_rate > 140 {
                return true;
            }
        }

        if let Some(bp_systolic) = vital_signs.blood_pressure_systolic {
            if bp_systolic < 70 || bp_systolic > 200 {
                return true;
            }
        }

        false
    }

    fn create_vital_signs_alert(
        env: &Env,
        session_id: u64,
        vital_signs: &VitalSigns,
    ) -> Result<(), Error> {
        let alert_id = Self::get_and_increment_alert_counter(env);
        let timestamp = env.ledger().timestamp();

        let alert = EmergencyAlert {
            alert_id,
            session_id,
            alert_type: "vital_signs_deterioration".to_string(),
            severity: "high".to_string(),
            message: "Vital signs showing deterioration - immediate attention required".to_string(),
            triggered_by: Address::from_array(env, &[0u8; 32]), // System triggered
            triggered_at: timestamp,
            acknowledged_by: None,
            acknowledged_at: None,
            resolved_by: None,
            resolved_at: None,
            resolution_actions: Vec::new(env),
            escalation_level: 4,
            notifications_sent: Vec::new(env),
        };

        let mut alerts: Map<u64, EmergencyAlert> = env
            .storage()
            .persistent()
            .get(&EMERGENCY_ALERTS)
            .unwrap_or(Map::new(env));
        alerts.set(alert_id, alert);
        env.storage().persistent().set(&EMERGENCY_ALERTS, &alerts);

        Ok(())
    }

    fn update_final_quality_metrics(
        env: &Env,
        session_id: u64,
        quality_score: u8,
    ) -> Result<(), Error> {
        let metrics: Map<u64, EmergencyQualityMetric> = env
            .storage()
            .persistent()
            .get(&QUALITY_METRICS)
            .unwrap_or(Map::new(env));

        for (metric_id, mut metric) in metrics.iter() {
            if metric.session_id == session_id {
                metric.actual_value = quality_score as f32;
                metric.achievement_rate = quality_score;
                metric.recorded_at = env.ledger().timestamp();

                let mut updated_metrics = metrics.clone();
                updated_metrics.set(*metric_id, metric);
                env.storage()
                    .persistent()
                    .set(&QUALITY_METRICS, &updated_metrics);
            }
        }

        Ok(())
    }

    fn release_session_resources(env: &Env, session_id: u64) -> Result<(), Error> {
        let teams: Map<u64, EmergencyResponseTeam> = env
            .storage()
            .persistent()
            .get(&RESPONSE_TEAMS)
            .unwrap_or(Map::new(env));

        // Find teams for this session and release resources
        for team in teams.values() {
            if team.session_id == session_id {
                // Update resource load
                let resources: Map<u64, EmergencyResource> = env
                    .storage()
                    .persistent()
                    .get(&EMERGENCY_RESOURCES)
                    .unwrap_or(Map::new(env));

                // This is simplified - would need to track which resources were used
                for mut resource in resources.values() {
                    if resource.current_load > 0 {
                        resource.current_load -= 1;
                        if resource.current_load < resource.capacity {
                            resource.status = "available".to_string();
                        }
                    }
                }
            }
        }

        Ok(())
    }

    fn get_and_increment_protocol_counter(env: &Env) -> u64 {
        let count: u64 = env
            .storage()
            .persistent()
            .get(&PROTOCOL_COUNTER)
            .unwrap_or(0);
        let next = count + 1;
        env.storage().persistent().set(&PROTOCOL_COUNTER, &next);
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

    fn get_and_increment_team_counter(env: &Env) -> u64 {
        let count: u64 = env.storage().persistent().get(&TEAM_COUNTER).unwrap_or(0);
        let next = count + 1;
        env.storage().persistent().set(&TEAM_COUNTER, &next);
        next
    }

    fn get_and_increment_resource_counter(env: &Env) -> u64 {
        let count: u64 = env
            .storage()
            .persistent()
            .get(&RESOURCE_COUNTER)
            .unwrap_or(0);
        let next = count + 1;
        env.storage().persistent().set(&RESOURCE_COUNTER, &next);
        next
    }

    fn get_and_increment_alert_counter(env: &Env) -> u64 {
        let count: u64 = env.storage().persistent().get(&ALERT_COUNTER).unwrap_or(0);
        let next = count + 1;
        env.storage().persistent().set(&ALERT_COUNTER, &next);
        next
    }

    fn get_and_increment_metric_counter(env: &Env) -> u64 {
        let count: u64 = env.storage().persistent().get(&METRIC_COUNTER).unwrap_or(0);
        let next = count + 1;
        env.storage().persistent().set(&METRIC_COUNTER, &next);
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
