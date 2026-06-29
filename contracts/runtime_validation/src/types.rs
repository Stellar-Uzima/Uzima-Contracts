use soroban_sdk::{contracttype, Address, String};

#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum ViolationType {
    InvariantViolation = 1,
    StateInconsistency = 2,
    PermissionDenied = 3,
    ResourceExceeded = 4,
    UnexpectedBehavior = 5,
}

#[contracttype]
#[derive(Clone, Debug)]
pub struct InvariantCheck {
    pub check_id: String,
    pub description: String,
    pub severity: u32,
    pub is_active: bool,
    pub created_at: u64,
    pub violation_count: u64,
}

#[contracttype]
#[derive(Clone, Debug)]
pub struct StateConsistencyCheck {
    pub check_id: String,
    pub description: String,
    pub expected_state: String,
    pub is_active: bool,
    pub created_at: u64,
    pub last_verified: u64,
    pub violation_count: u64,
}

#[contracttype]
#[derive(Clone, Debug)]
pub struct PermissionCheck {
    pub check_id: String,
    pub description: String,
    pub required_role: String,
    pub is_active: bool,
    pub created_at: u64,
    pub violation_count: u64,
}

#[contracttype]
#[derive(Clone, Debug)]
pub struct ResourceTracker {
    pub tracker_id: String,
    pub resource_type: String,
    pub max_allocation: i128,
    pub current_usage: i128,
    pub created_at: u64,
    pub last_updated: u64,
}

#[contracttype]
#[derive(Clone, Debug)]
pub struct ValidationReport {
    pub violation_id: u64,
    pub check_id: String,
    pub violation_type: ViolationType,
    pub reporter: Address,
    pub details: String,
    pub timestamp: u64,
    pub resolved: bool,
}

#[contracttype]
#[derive(Clone, Debug)]
pub enum DataKey {
    Admin,
    ViolationCount,
    CheckCount,
    Invariant(String),
    StateCheck(String),
    PermissionCheck(String),
    ResourceTracker(String),
    Violation(u64),
}
