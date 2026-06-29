use soroban_sdk::{contracttype, symbol_short, Address, Env, String};

#[derive(Clone, Copy, PartialEq, Eq)]
#[contracttype]
pub enum EventType {
    Initialized,
    RoleAssigned,
    RoleRemoved,
    ConfigUpdated,
    RoleChecked,
}

#[derive(Clone, Copy, PartialEq, Eq)]
#[contracttype]
pub enum OperationCategory {
    UserManagement,
    Administrative,
}

#[derive(Clone)]
#[contracttype]
pub struct RBACEventData {
    pub target_address: Address,
    pub role: Option<String>,
    pub success: bool,
}

#[derive(Clone)]
#[contracttype]
pub struct RBACEvent {
    pub event_type: EventType,
    pub category: OperationCategory,
    pub timestamp: u64,
    pub user_id: Address,
    pub block_height: u64,
    pub data: RBACEventData,
}

pub fn emit_initialized(env: &Env, admin: Address) {
    let event = RBACEvent {
        event_type: EventType::Initialized,
        category: OperationCategory::Administrative,
        timestamp: env.ledger().timestamp(),
        user_id: admin.clone(),
        block_height: env.ledger().sequence() as u64,
        data: RBACEventData {
            target_address: admin,
            role: None,
            success: true,
        },
    };
    env.events()
        .publish(("EVENT", symbol_short!("RBAC_INIT")), event);
}

pub fn emit_role_assigned(env: &Env, admin: Address, target: Address, role: String, success: bool) {
    let event = RBACEvent {
        event_type: EventType::RoleAssigned,
        category: OperationCategory::UserManagement,
        timestamp: env.ledger().timestamp(),
        user_id: admin,
        block_height: env.ledger().sequence() as u64,
        data: RBACEventData {
            target_address: target,
            role: Some(role),
            success,
        },
    };
    env.events()
        .publish(("EVENT", symbol_short!("ROLE_ADD")), event);
}

pub fn emit_role_removed(env: &Env, admin: Address, target: Address, role: String, success: bool) {
    let event = RBACEvent {
        event_type: EventType::RoleRemoved,
        category: OperationCategory::UserManagement,
        timestamp: env.ledger().timestamp(),
        user_id: admin,
        block_height: env.ledger().sequence() as u64,
        data: RBACEventData {
            target_address: target,
            role: Some(role),
            success,
        },
    };
    env.events()
        .publish(("EVENT", symbol_short!("ROLE_REM")), event);
}
