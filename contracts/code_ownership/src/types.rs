use soroban_sdk::{contracttype, Address, String, Vec};

#[contracttype]
#[derive(Clone, Debug)]
pub struct ModuleOwnership {
    pub module_id: String,
    pub module_name: String,
    pub primary_owner: Address,
    pub secondary_owners: Vec<Address>,
    pub expertise_areas: Vec<String>,
    pub registered_at: u64,
    pub last_updated: u64,
}

#[contracttype]
#[derive(Clone, Debug)]
pub struct ReviewRoute {
    pub module_id: String,
    pub required_reviewers: u32,
    pub escalation_threshold: u32,
    pub escalation_owner: Address,
    pub created_at: u64,
}

#[contracttype]
#[derive(Clone, Debug)]
pub struct OwnershipMatrix {
    pub total_modules: u32,
    pub modules: Vec<ModuleOwnership>,
    pub generated_at: u64,
}

#[contracttype]
#[derive(Clone, Debug)]
pub enum DataKey {
    Admin,
    ModuleCount,
    Module(String),
    ModuleIndex(u32),
    ReviewRoute(String),
}
