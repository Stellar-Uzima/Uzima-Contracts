use soroban_sdk::{contracttype, Address, Bytes, BytesN, Env, Map, String, Vec};

#[derive(Clone)]
#[contracttype]
pub struct UserProfile {
    pub user_id: Address,
    pub user_type: UserType,
    pub registration_timestamp: u64,
    pub consent_version: u32,
    pub privacy_settings: PrivacySettings,
    pub emergency_contacts: Vec<EmergencyContact>,
    pub risk_level: RiskLevel,
    pub last_activity: u64,
}

#[derive(Clone, Copy, PartialEq, Eq)]
#[contracttype]
pub enum UserType {
    Patient,
    MentalHealthProfessional,
    Caregiver,
    Researcher,
    Admin,
}

#[derive(Clone)]
#[contracttype]
pub struct PrivacySettings {
    pub data_sharing_consent: bool,
    pub research_consent: bool,
    pub emergency_access: bool,
    pub analytics_consent: bool,
    pub third_party_sharing: bool,
}

impl PrivacySettings {
    pub fn default() -> Self {
        PrivacySettings {
            data_sharing_consent: false,
            research_consent: false,
            emergency_access: true,
            analytics_consent: false,
            third_party_sharing: false,
        }
    }
}

#[derive(Clone)]
#[contracttype]
pub struct EmergencyContact {
    pub name: String,
    pub relationship: String,
    pub phone: String,
    pub email: String,
    pub priority: u32,
}

#[derive(Clone, Copy, PartialEq, Eq)]
#[contracttype]
pub enum RiskLevel {
    Low,
    Moderate,
    High,
    Critical,
}

#[derive(Clone, Copy, PartialEq, Eq)]
#[contracttype]
pub enum Permission {
    ReadOwnData,
    WriteOwnData,
    ReadPatientData,
    WritePatientData,
    EmergencyAccess,
    AdminAccess,
}

// Therapy Session Types
#[derive(Clone)]
#[contracttype]
pub struct TherapySession {
    pub session_id: u64,
    pub patient_id: Address,
    pub therapist_id: Address,
    pub session_type: SessionType,
    pub timestamp: u64,
    pub duration_minutes: u32,
    pub notes: String, // Encrypted
    pub recording_hash: Option<BytesN<32>>,
    pub ai_insights: Option<String>,
    pub follow_up_required: bool,
    pub confidentiality_level: ConfidentialityLevel,
}

#[derive(Clone, Copy, PartialEq, Eq)]
#[contracttype]
pub enum SessionType {
    Individual,
    Group,
    Family,
    Crisis,
    Assessment,
}

#[derive(Clone, Copy, PartialEq, Eq)]
#[contracttype]
pub enum ConfidentialityLevel {
    Standard,
    High,
    Absolute,
}

// Mood Tracking Types
#[derive(Clone)]
#[contracttype]
pub struct MoodEntry {
    pub entry_id: u64,
    pub patient_id: Address,
    pub timestamp: u64,
    pub mood_score: i32, // -10 to 10
    pub emotions: Vec<String>,
    pub triggers: Vec<String>,
    pub notes: String,
    pub location_context: Option<String>,
    pub ai_analysis: Option<MoodAnalysis>,
}

#[derive(Clone)]
#[contracttype]
pub struct MoodAnalysis {
    pub sentiment_score: f32,
    pub dominant_emotion: String,
    pub risk_indicators: Vec<String>,
    pub recommendations: Vec<String>,
    pub trend_analysis: String,
}

// Assessment Types
#[derive(Clone)]
#[contracttype]
pub struct Assessment {
    pub assessment_id: u64,
    pub patient_id: Address,
    pub assessment_type: AssessmentType,
    pub timestamp: u64,
    pub responses: Map<String, String>,
    pub score: Option<u32>,
    pub interpretation: String,
    pub risk_flags: Vec<String>,
    pub recommendations: Vec<String>,
    pub administered_by: Address,
}

#[derive(Clone, Copy, PartialEq, Eq)]
#[contracttype]
pub enum AssessmentType {
    PHQ9,  // Depression
    GAD7,  // Anxiety
    PCL5,  // PTSD
    AUDIT, // Alcohol Use
    DAST,  // Drug Use
    BDI,   // Beck Depression Inventory
    BAI,   // Beck Anxiety Inventory
    Custom,
}

// Crisis Intervention Types
#[derive(Clone)]
#[contracttype]
pub struct CrisisAlert {
    pub alert_id: u64,
    pub patient_id: Address,
    pub alert_type: CrisisType,
    pub severity: CrisisSeverity,
    pub timestamp: u64,
    pub description: String,
    pub location: Option<String>,
    pub immediate_actions_taken: Vec<String>,
    pub emergency_contacts_notified: Vec<Address>,
    pub resolution_status: CrisisResolution,
    pub follow_up_required: bool,
}

#[derive(Clone, Copy, PartialEq, Eq)]
#[contracttype]
pub enum CrisisType {
    SuicidalIdeation,
    SelfHarm,
    SevereDepression,
    PanicAttack,
    Psychosis,
    SubstanceOverdose,
    DomesticViolence,
    Other,
}

#[derive(Clone, Copy, PartialEq, Eq)]
#[contracttype]
pub enum CrisisSeverity {
    Low,
    Moderate,
    High,
    Critical,
}

#[derive(Clone, Copy, PartialEq, Eq)]
#[contracttype]
pub enum CrisisResolution {
    Resolved,
    Ongoing,
    Escalated,
    FalseAlarm,
}

// Medication Management Types
#[derive(Clone)]
#[contracttype]
pub struct MedicationPlan {
    pub plan_id: u64,
    pub patient_id: Address,
    pub medication_name: String,
    pub dosage: String,
    pub frequency: String,
    pub start_date: u64,
    pub end_date: Option<u64>,
    pub prescribed_by: Address,
    pub side_effects: Vec<String>,
    pub adherence_tracking: Vec<AdherenceEntry>,
    pub effectiveness_rating: Option<u32>, // 1-10
    pub notes: String,
}

#[derive(Clone)]
#[contracttype]
pub struct AdherenceEntry {
    pub timestamp: u64,
    pub taken: bool,
    pub dosage_taken: Option<String>,
    pub side_effects_experienced: Vec<String>,
    pub notes: String,
}

// Peer Support Types
#[derive(Clone)]
#[contracttype]
pub struct PeerGroup {
    pub group_id: String,
    pub name: String,
    pub description: String,
    pub focus_area: String,
    pub moderator: Address,
    pub members: Vec<Address>,
    pub max_members: u32,
    pub privacy_level: GroupPrivacy,
    pub rules: Vec<String>,
    pub created_timestamp: u64,
}

#[derive(Clone, Copy, PartialEq, Eq)]
#[contracttype]
pub enum GroupPrivacy {
    Public,
    Private,
    InviteOnly,
}

#[derive(Clone)]
#[contracttype]
pub struct PeerMessage {
    pub message_id: u64,
    pub group_id: String,
    pub sender: Address,
    pub timestamp: u64,
    pub content: String, // Encrypted
    pub message_type: MessageType,
    pub moderated: bool,
}

#[derive(Clone, Copy, PartialEq, Eq)]
#[contracttype]
pub enum MessageType {
    Text,
    Support,
    Resource,
    CrisisAlert,
}

// Professional Directory Types
#[derive(Clone)]
#[contracttype]
pub struct MentalHealthProfessional {
    pub professional_id: Address,
    pub name: String,
    pub credentials: Vec<Credential>,
    pub specializations: Vec<String>,
    pub languages: Vec<String>,
    pub availability: AvailabilitySchedule,
    pub contact_info: ContactInfo,
    pub bio: String,
    pub rating: f32,
    pub review_count: u32,
    pub verified: bool,
    pub insurance_accepted: Vec<String>,
}

#[derive(Clone)]
#[contracttype]
pub struct Credential {
    pub credential_type: String,
    pub issuer: String,
    pub issue_date: u64,
    pub expiry_date: Option<u64>,
    pub verification_hash: BytesN<32>,
}

#[derive(Clone)]
#[contracttype]
pub struct AvailabilitySchedule {
    pub timezone: String,
    pub weekly_schedule: Map<String, Vec<TimeSlot>>,
    pub exceptions: Vec<ScheduleException>,
}

#[derive(Clone)]
#[contracttype]
pub struct TimeSlot {
    pub start_time: u64,
    pub end_time: u64,
    pub available: bool,
}

#[derive(Clone)]
#[contracttype]
pub struct ScheduleException {
    pub date: u64,
    pub available: bool,
    pub reason: String,
}

#[derive(Clone)]
#[contracttype]
pub struct ContactInfo {
    pub email: String,
    pub phone: String,
    pub emergency_phone: Option<String>,
    pub address: Option<String>,
}

// Wellness Program Types
#[derive(Clone)]
#[contracttype]
pub struct WellnessProgram {
    pub program_id: u64,
    pub name: String,
    pub description: String,
    pub category: WellnessCategory,
    pub duration_weeks: u32,
    pub enrolled_users: Vec<Address>,
    pub modules: Vec<WellnessModule>,
    pub completion_rate: f32,
    pub effectiveness_score: f32,
}

#[derive(Clone, Copy, PartialEq, Eq)]
#[contracttype]
pub enum WellnessCategory {
    Mindfulness,
    Exercise,
    Nutrition,
    Sleep,
    StressManagement,
    SocialConnection,
    CognitiveTraining,
}

#[derive(Clone)]
#[contracttype]
pub struct WellnessModule {
    pub module_id: u64,
    pub title: String,
    pub content: String,
    pub duration_minutes: u32,
    pub completion_criteria: Vec<String>,
    pub resources: Vec<String>,
}

#[derive(Clone)]
#[contracttype]
pub struct UserWellnessProgress {
    pub user_id: Address,
    pub program_id: u64,
    pub enrolled_date: u64,
    pub completed_modules: Vec<u64>,
    pub current_streak: u32,
    pub total_sessions: u32,
    pub last_activity: u64,
    pub progress_percentage: f32,
}

// Data Anonymization Types
#[derive(Clone)]
#[contracttype]
pub struct AnonymizedDataset {
    pub dataset_id: u64,
    pub name: String,
    pub description: String,
    pub data_fields: Vec<String>,
    pub record_count: u64,
    pub anonymization_method: AnonymizationMethod,
    pub created_timestamp: u64,
    pub approved_researchers: Vec<Address>,
    pub usage_restrictions: Vec<String>,
}

#[derive(Clone, Copy, PartialEq, Eq)]
#[contracttype]
pub enum AnonymizationMethod {
    KAnonymity,
    LDiversity,
    TCloseness,
    DifferentialPrivacy,
    HomomorphicEncryption,
}

#[derive(Clone)]
#[contracttype]
pub struct ResearchQuery {
    pub query_id: u64,
    pub researcher_id: Address,
    pub dataset_id: u64,
    pub query_type: QueryType,
    pub parameters: Map<String, String>,
    pub approval_status: ApprovalStatus,
    pub results_hash: Option<BytesN<32>>,
    pub timestamp: u64,
}

#[derive(Clone, Copy, PartialEq, Eq)]
#[contracttype]
pub enum QueryType {
    AggregateStatistics,
    CorrelationAnalysis,
    TrendAnalysis,
    PredictiveModeling,
}

#[derive(Clone, Copy, PartialEq, Eq)]
#[contracttype]
pub enum ApprovalStatus {
    Pending,
    Approved,
    Rejected,
    Completed,
}

// Suicide Prevention Types
#[derive(Clone)]
#[contracttype]
pub struct SuicidePreventionProtocol {
    pub protocol_id: u64,
    pub name: String,
    pub triggers: Vec<String>,
    pub risk_factors: Vec<String>,
    pub intervention_steps: Vec<String>,
    pub emergency_contacts: Vec<EmergencyContact>,
    pub resources: Vec<String>,
    pub success_rate: f32,
}

#[derive(Clone)]
#[contracttype]
pub struct PreventionAlert {
    pub alert_id: u64,
    pub patient_id: Address,
    pub protocol_id: u64,
    pub trigger_reason: String,
    pub risk_score: f32,
    pub timestamp: u64,
    pub actions_taken: Vec<String>,
    pub outcome: PreventionOutcome,
}

#[derive(Clone, Copy, PartialEq, Eq)]
#[contracttype]
pub enum PreventionOutcome {
    InterventionSuccessful,
    EscalatedToEmergency,
    FalsePositive,
    OngoingMonitoring,
}
