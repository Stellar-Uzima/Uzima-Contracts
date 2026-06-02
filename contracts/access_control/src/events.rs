//! # Access Control Events Module
//!
//! Emits events for all access control state changes.
//!
//! Events are critical for monitoring:
//! - Admin transfers
//! - Role assignments and revocations
//! - Permission grants and revokes

use soroban_sdk::{contracttype, symbol_short, Address, Env, String};

// ── Event Type Definitions ─────────────────────────────────────────────────

#[derive(Clone, Copy, PartialEq, Eq)]
#[contracttype]
pub enum EventType {
    /// Contract initialized with admin
    Initialized,
    /// Admin rights transferred to new address
    AdminTransferred,
    /// Role assigned to address
    RoleAssigned,
    /// Role revoked from address
    RoleRevoked,
    /// Permission granted
    PermissionGranted,
    /// Permission revoked
    PermissionRevoked,
    /// Authorization check performed
    AuthorizationChecked,
}

#[derive(Clone, Copy, PartialEq, Eq)]
#[contracttype]
pub enum OperationCategory {
    UserManagement,
    Administrative,
    AccessControl,
}

// ── Event Data Structures ──────────────────────────────────────────────────

#[derive(Clone)]
#[contracttype]
pub struct AccessControlEventData {
    /// Primary target of the event
    pub target_address: Address,
    /// Role affected (if applicable)
    pub role: Option<String>,
    /// Previous value (for transitions)
    pub previous_value: Option<String>,
    /// Whether operation succeeded
    pub success: bool,
    /// Optional details
    pub details: Option<String>,
}

#[derive(Clone)]
#[contracttype]
pub struct AccessControlEvent {
    pub event_type: EventType,
    pub category: OperationCategory,
    pub timestamp: u64,
    pub user_id: Address,
    pub block_height: u64,
    pub data: AccessControlEventData,
}

// ── Event Emission Functions ───────────────────────────────────────────────

/// Emit Initialized event when contract is initialized
pub fn emit_initialized(env: &Env, admin: Address) {
    let event = AccessControlEvent {
        event_type: EventType::Initialized,
        category: OperationCategory::Administrative,
        timestamp: env.ledger().timestamp(),
        user_id: admin.clone(),
        block_height: env.ledger().sequence() as u64,
        data: AccessControlEventData {
            target_address: admin,
            role: None,
            previous_value: None,
            success: true,
            details: None,
        },
    };
    env.events()
        .publish((symbol_short!("AC"), symbol_short!("INIT")), event);
}

/// Emit AdminTransferred event when admin rights change
pub fn emit_admin_transferred(env: &Env, caller: Address, old_admin: Address, new_admin: Address) {
    let event = AccessControlEvent {
        event_type: EventType::AdminTransferred,
        category: OperationCategory::Administrative,
        timestamp: env.ledger().timestamp(),
        user_id: caller,
        block_height: env.ledger().sequence() as u64,
        data: AccessControlEventData {
            target_address: new_admin,
            role: None,
            previous_value: Some(old_admin.to_string()),
            success: true,
            details: None,
        },
    };
    env.events()
        .publish((symbol_short!("AC"), symbol_short!("ADMIN")), event);
}

/// Emit RoleAssigned event
pub fn emit_role_assigned(
    env: &Env,
    caller: Address,
    target: Address,
    role: String,
    success: bool,
) {
    let event = AccessControlEvent {
        event_type: EventType::RoleAssigned,
        category: OperationCategory::UserManagement,
        timestamp: env.ledger().timestamp(),
        user_id: caller,
        block_height: env.ledger().sequence() as u64,
        data: AccessControlEventData {
            target_address: target,
            role: Some(role),
            previous_value: None,
            success,
            details: None,
        },
    };
    env.events()
        .publish((symbol_short!("AC"), symbol_short!("GRANT")), event);
}

/// Emit RoleRevoked event
pub fn emit_role_revoked(env: &Env, caller: Address, target: Address, role: String, success: bool) {
    let event = AccessControlEvent {
        event_type: EventType::RoleRevoked,
        category: OperationCategory::UserManagement,
        timestamp: env.ledger().timestamp(),
        user_id: caller,
        block_height: env.ledger().sequence() as u64,
        data: AccessControlEventData {
            target_address: target,
            role: Some(role),
            previous_value: None,
            success,
            details: None,
        },
    };
    env.events()
        .publish((symbol_short!("AC"), symbol_short!("REVOK")), event);
}

/// Emit PermissionGranted event
pub fn emit_permission_granted(
    env: &Env,
    caller: Address,
    target: Address,
    permission: String,
) {
    let event = AccessControlEvent {
        event_type: EventType::PermissionGranted,
        category: OperationCategory::AccessControl,
        timestamp: env.ledger().timestamp(),
        user_id: caller,
        block_height: env.ledger().sequence() as u64,
        data: AccessControlEventData {
            target_address: target,
            role: Some(permission),
            previous_value: None,
            success: true,
            details: None,
        },
    };
    env.events()
        .publish((symbol_short!("AC"), symbol_short!("PERM")), event);
}

/// Emit PermissionRevoked event
pub fn emit_permission_revoked(
    env: &Env,
    caller: Address,
    target: Address,
    permission: String,
) {
    let event = AccessControlEvent {
        event_type: EventType::PermissionRevoked,
        category: OperationCategory::AccessControl,
        timestamp: env.ledger().timestamp(),
        user_id: caller,
        block_height: env.ledger().sequence() as u64,
        data: AccessControlEventData {
            target_address: target,
            role: Some(permission),
            previous_value: None,
            success: true,
            details: None,
        },
    };
    env.events()
        .publish((symbol_short!("AC"), symbol_short!("PREM")), event);
}

/// Emit AuthorizationChecked event (for audit trail)
pub fn emit_authorization_checked(env: &Env, caller: Address, required_role: String, result: bool) {
    let event = AccessControlEvent {
        event_type: EventType::AuthorizationChecked,
        category: OperationCategory::AccessControl,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: AccessControlEventData {
            target_address: caller,
            role: Some(required_role),
            previous_value: None,
            success: result,
            details: None,
        },
    };
    env.events()
        .publish((symbol_short!("AC"), symbol_short!("CHECK")), event);
}
