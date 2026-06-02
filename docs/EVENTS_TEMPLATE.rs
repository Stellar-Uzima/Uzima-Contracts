/// # Contract Events Module
///
/// Standardized event emission for [CONTRACT_NAME] contract.
///
/// All state-changing operations MUST emit at least one event for auditability.
///
/// Event Naming: [CONTRACT_SYMBOL]:[ACTION]
///
/// Event Registry: See `schemas/events/registry.json` for complete schema

use soroban_sdk::{contracttype, symbol_short, Address, Env, String};

// ── Event Type Definitions ─────────────────────────────────────────────────

/// All event types emitted by this contract
#[derive(Clone, Copy, PartialEq, Eq)]
#[contracttype]
pub enum EventType {
    /// Triggered on contract initialization
    Initialized,
    /// Triggered when main operation occurs
    OperationExecuted,
    // Add more event types as needed
    // Examples:
    // AdminTransferred,
    // UserCreated,
    // RecordModified,
    // AccessGranted,
    // StatusChanged,
}

/// Categories for event classification
#[derive(Clone, Copy, PartialEq, Eq)]
#[contracttype]
pub enum OperationCategory {
    /// User and role management
    UserManagement,
    /// Data CRUD operations
    RecordOperations,
    /// Access control and permissions
    AccessControl,
    /// Emergency/override operations
    EmergencyAccess,
    /// System administration
    Administrative,
    /// AI/ML integration
    AIIntegration,
    /// Cross-chain operations
    CrossChain,
    /// System-level events
    System,
}

// ── Event Data Structures ──────────────────────────────────────────────────

/// Contract-specific event data (customize for your contract)
#[derive(Clone)]
#[contracttype]
pub struct [CONTRACT_CAMELCASE]EventData {
    /// Primary subject of the event
    pub target_id: Option<u64>,
    /// Relevant address (user, admin, etc.)
    pub target_address: Option<Address>,
    /// Operation status
    pub success: bool,
    /// Additional context
    pub details: Option<String>,
}

/// Standard event envelope with metadata
#[derive(Clone)]
#[contracttype]
pub struct [CONTRACT_CAMELCASE]Event {
    /// Type of event
    pub event_type: EventType,
    /// Category for filtering
    pub category: OperationCategory,
    /// Timestamp (seconds since epoch)
    pub timestamp: u64,
    /// Address that triggered the event
    pub user_id: Address,
    /// Ledger sequence at event time
    pub block_height: u64,
    /// Contract-specific data
    pub data: [CONTRACT_CAMELCASE]EventData,
}

// ── Event Emission Functions ───────────────────────────────────────────────

/// Emit Initialized event
pub fn emit_initialized(env: &Env, admin: Address) {
    let event = [CONTRACT_CAMELCASE]Event {
        event_type: EventType::Initialized,
        category: OperationCategory::Administrative,
        timestamp: env.ledger().timestamp(),
        user_id: admin.clone(),
        block_height: env.ledger().sequence() as u64,
        data: [CONTRACT_CAMELCASE]EventData {
            target_id: None,
            target_address: Some(admin),
            success: true,
            details: None,
        },
    };
    env.events().publish(
        (symbol_short!("[SYMBOL]"), symbol_short!("INIT")),
        event,
    );
}

/// Emit OperationExecuted event
pub fn emit_operation_executed(
    env: &Env,
    user: Address,
    target_id: Option<u64>,
    success: bool,
    details: Option<String>,
) {
    let event = [CONTRACT_CAMELCASE]Event {
        event_type: EventType::OperationExecuted,
        category: OperationCategory::RecordOperations,
        timestamp: env.ledger().timestamp(),
        user_id: user,
        block_height: env.ledger().sequence() as u64,
        data: [CONTRACT_CAMELCASE]EventData {
            target_id,
            target_address: None,
            success,
            details,
        },
    };
    env.events().publish(
        (symbol_short!("[SYMBOL]"), symbol_short!("EXEC")),
        event,
    );
}

// ── Helper Macros (Optional) ───────────────────────────────────────────────
// Uncomment if using logging or tracing:
//
// #[cfg(test)]
// mod tests {
//     use super::*;
//     use soroban_sdk::Env;
//
//     #[test]
//     fn test_event_emission() {
//         let env = Env::default();
//         // Test your event emissions here
//     }
// }
