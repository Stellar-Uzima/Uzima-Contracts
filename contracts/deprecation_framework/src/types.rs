use soroban_sdk::{contracttype, String, Vec};

#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum DeprecationPhase {
    Announced = 1,
    Supported = 2,
    Sunset = 3,
    Removed = 4,
}

#[contracttype]
#[derive(Clone, Debug)]
pub struct DeprecationStatus {
    pub contract_id: String,
    pub contract_name: String,
    pub reason: String,
    pub replacement_contract: Option<String>,
    pub phase: DeprecationPhase,
    pub marked_at: u64,
    pub last_updated: u64,
    pub is_active: bool,
}

#[contracttype]
#[derive(Clone, Debug)]
pub struct SunsetTimeline {
    pub contract_id: String,
    pub announcement_date: u64,
    pub support_end_date: u64,
    pub removal_date: u64,
    pub created_at: u64,
}

#[contracttype]
#[derive(Clone, Debug)]
pub struct MigrationGuide {
    pub contract_id: String,
    pub guide_title: String,
    pub guide_content: String,
    pub code_examples: Vec<String>,
    pub created_at: u64,
    pub last_updated: u64,
}

#[contracttype]
#[derive(Clone, Debug)]
pub enum DataKey {
    Admin,
    ContractCount,
    CommunicationCount,
    DeprecatedContract(String),
    SunsetTimeline(String),
    MigrationGuide(String),
    RemovalChecklist(String),
    ChecklistItemComplete(String, u32),
    Communication(u64),
}
