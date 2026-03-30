use soroban_sdk::{contracttype, Address, Vec};

/// Enumeration of available roles in the healthcare system
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[contracttype]
#[repr(u32)]
pub enum Role {
    /// Administrator with full system access
    Admin = 0,
    /// Doctor/healthcare provider
    Doctor = 1,
    /// Patient
    Patient = 2,
    /// Nurse or other healthcare staff
    Staff = 3,
    /// Insurance provider
    Insurer = 4,
    /// Researcher
    Researcher = 5,
    /// Auditor for compliance
    Auditor = 6,
    /// Service account for systems
    Service = 7,
}

/// Represents a single role assignment to an address
#[derive(Clone)]
#[contracttype]
pub struct RoleAssignment {
    /// The address being assigned a role
    pub address: Address,
    /// The role assigned
    pub role: Role,
    /// Timestamp when the role was assigned
    pub assigned_at: u64,
    /// Address of the admin who assigned the role
    pub assigned_by: Address,
}

/// Summary of roles for an address
#[derive(Clone)]
#[contracttype]
pub struct AddressRoles {
    /// The address
    pub address: Address,
    /// List of all roles assigned to this address
    pub roles: Vec<Role>,
    /// Count of roles
    pub role_count: u32,
}

/// Configuration for RBAC contract behavior
#[derive(Clone)]
#[contracttype]
pub struct RBACConfig {
    /// Whether role changes require event emission
    pub emit_events: bool,
    /// Maximum number of roles per address
    pub max_roles_per_address: u32,
}

/// Storage keys for the contract
#[derive(Clone, Copy, PartialEq, Eq)]
#[contracttype]
pub enum DataKey {
    /// Admin address
    Admin,
    /// Global RBAC configuration
    Config,
    /// Set of roles for an address (Vec<Role>)
    AddressRoles(Address),
    /// Count of addresses with roles
    AddressCount,
    /// Set of addresses with a specific role (Vec<Address>)
    RoleMembers(u32), // Role discriminant as u32
    /// Total number of role assignments
    AssignmentCount,
    /// Individual role assignment history
    Assignment(u64),
    /// Initialization flag
    IsInitialized,
}
