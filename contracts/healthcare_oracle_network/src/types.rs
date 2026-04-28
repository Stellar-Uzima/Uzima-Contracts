use soroban_sdk::{contracterror, contracttype, Address, String, Vec};

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    AlreadyInitialized = 1,
    NotInitialized = 2,
    Unauthorized = 3,
    OracleAlreadyRegistered = 4,
    OracleNotFound = 5,
    OracleNotVerified = 6,
    OracleInactive = 7,
    InvalidData = 8,
    SubmissionAlreadyExists = 9,
    RoundNotFound = 10,
    InsufficientSubmissions = 11,
    ConsensusAlreadyFinalized = 12,
    ConsensusNotFound = 13,
    DisputeNotFound = 14,
    DisputeAlreadyResolved = 15,
    InvalidDisputeState = 16,
    InvalidFeedType = 17,
    ArbiterExists = 18,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[contracttype]
pub enum FeedKind {
    DrugPricing = 1,
    ClinicalTrial = 2,
    RegulatoryUpdate = 3,
    TreatmentOutcome = 4,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[contracttype]
pub enum SourceType {
    PharmaSupplier = 1,
    ClinicalRegistry = 2,
    RegulatoryBody = 3,
    MarketAggregator = 4,
    HospitalNetwork = 5,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[contracttype]
pub enum RegulatoryAuthority {
    FDA = 1,
    EMA = 2,
    MHRA = 3,
    PMDA = 4,
    WHO = 5,
    CDSCO = 6,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[contracttype]
pub enum RegulatoryStatus {
    Approved = 1,
    SafetyWarning = 2,
    Recall = 3,
    GuidelineUpdate = 4,
    TrialHold = 5,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[contracttype]
pub enum DisputeStatus {
    Open = 1,
    ResolvedValid = 2,
    ResolvedInvalid = 3,
}

#[derive(Clone)]
#[contracttype]
pub struct FeedKey {
    pub kind: FeedKind,
    pub feed_id: String,
}

#[derive(Clone)]
#[contracttype]
pub struct DrugPriceData {
    pub ndc_code: String,
    pub currency: String,
    pub price_minor: i128,
    pub availability_units: u32,
    pub observed_at: u64,
}

#[derive(Clone)]
#[contracttype]
pub struct ClinicalTrialData {
    pub trial_id: String,
    pub phase: u32,
    pub enrolled: u32,
    pub success_rate_bps: u32,
    pub adverse_event_rate_bps: u32,
    pub result_hash: String,
    pub published_at: u64,
}

#[derive(Clone)]
#[contracttype]
pub struct TreatmentOutcomeData {
    pub outcome_id: String,
    pub condition_code: String,
    pub treatment_code: String,
    pub improvement_rate_bps: u32,
    pub readmission_rate_bps: u32,
    pub mortality_rate_bps: u32,
    pub sample_size: u32,
    pub reported_at: u64,
}

#[derive(Clone)]
#[contracttype]
pub struct RegulatoryUpdateData {
    pub regulation_id: String,
    pub authority: RegulatoryAuthority,
    pub status: RegulatoryStatus,
    pub title: String,
    pub details_hash: String,
    pub effective_at: u64,
}

#[derive(Clone)]
#[contracttype]
pub enum FeedPayload {
    DrugPrice(DrugPriceData),
    ClinicalTrial(ClinicalTrialData),
    RegulatoryUpdate(RegulatoryUpdateData),
    TreatmentOutcome(TreatmentOutcomeData),
}

#[derive(Clone)]
#[contracttype]
pub struct OracleNode {
    pub operator: Address,
    pub endpoint: String,
    pub source_type: SourceType,
    pub verified: bool,
    pub active: bool,
    pub reputation: i128,
    pub submissions: u32,
    pub disputes: u32,
    pub last_seen: u64,
}

#[derive(Clone)]
#[contracttype]
pub struct Config {
    pub admin: Address,
    pub arbiters: Vec<Address>,
    pub min_submissions: u32,
    pub min_reputation: i128,
    pub max_drug_price_minor: i128,
    pub max_availability_units: u32,
}

#[derive(Clone)]
#[contracttype]
pub struct AggregationRound {
    pub id: u64,
    pub started_at: u64,
    pub finalized: bool,
    pub submissions: u32,
}

#[derive(Clone)]
#[contracttype]
pub struct ConsensusRecord {
    pub key: FeedKey,
    pub payload: FeedPayload,
    pub round_id: u64,
    pub finalized_at: u64,
    pub submitters: Vec<Address>,
    pub confidence_bps: u32,
    pub disputed: bool,
}

#[derive(Clone)]
#[contracttype]
pub struct Dispute {
    pub id: u64,
    pub key: FeedKey,
    pub round_id: u64,
    pub challenger: Address,
    pub reason: String,
    pub status: DisputeStatus,
    pub opened_at: u64,
    pub resolved_at: Option<u64>,
    pub resolver: Option<Address>,
    pub ruling: Option<String>,
}

#[derive(Clone)]
#[contracttype]
pub enum DataKey {
    Config,
    Oracle(Address),
    OracleList,
    RoundCounter(FeedKey),
    Round(FeedKey, u64),
    Submission(FeedKey, u64, Address),
    Consensus(FeedKey),
    DisputeCount,
    Dispute(u64),
}
