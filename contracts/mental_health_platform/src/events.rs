use soroban_sdk::{contracttype, Address, String, Symbol};

#[contracttype]
#[derive(Clone)]
pub struct UserRegisteredEvent {
    pub user: Address,
    pub user_type: String,
    pub timestamp: u64,
}

#[contracttype]
#[derive(Clone)]
pub struct TherapySessionEvent {
    pub session_id: u64,
    pub patient_id: Address,
    pub therapist_id: Address,
    pub session_type: String,
    pub timestamp: u64,
}

#[contracttype]
#[derive(Clone)]
pub struct MoodEntryEvent {
    pub entry_id: u64,
    pub patient_id: Address,
    pub mood_score: i32,
    pub timestamp: u64,
}

#[contracttype]
#[derive(Clone)]
pub struct CrisisAlertEvent {
    pub alert_id: u64,
    pub patient_id: Address,
    pub alert_type: String,
    pub severity: String,
    pub timestamp: u64,
}

#[contracttype]
#[derive(Clone)]
pub struct AssessmentCompletedEvent {
    pub assessment_id: u64,
    pub patient_id: Address,
    pub assessment_type: String,
    pub score: u32,
    pub timestamp: u64,
}

#[contracttype]
#[derive(Clone)]
pub struct MedicationAdherenceEvent {
    pub plan_id: u64,
    pub patient_id: Address,
    pub taken: bool,
    pub timestamp: u64,
}

#[contracttype]
#[derive(Clone)]
pub struct PeerMessageEvent {
    pub message_id: u64,
    pub group_id: String,
    pub sender: Address,
    pub message_type: String,
    pub timestamp: u64,
}

#[contracttype]
#[derive(Clone)]
pub struct WellnessProgressEvent {
    pub user_id: Address,
    pub program_id: u64,
    pub progress_percentage: f32,
    pub timestamp: u64,
}

#[contracttype]
#[derive(Clone)]
pub struct ResearchQueryEvent {
    pub query_id: u64,
    pub researcher_id: Address,
    pub dataset_id: u64,
    pub query_type: String,
    pub timestamp: u64,
}

#[contracttype]
#[derive(Clone)]
pub struct PreventionAlertEvent {
    pub alert_id: u64,
    pub patient_id: Address,
    pub protocol_id: u64,
    pub risk_score: f32,
    pub timestamp: u64,
}
