#![no_std]

use soroban_sdk::{contracttype, Address, Env, Symbol, Vec as SorobanVec};

// ==================== Authorization Observability Types ====================

/// Reasons why an authorization attempt was denied.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[contracttype]
#[repr(u32)]
pub enum DenialReason {
    NotAdmin = 0,
    NotAuthorized = 1,
    InsufficientRole = 2,
    ContractPaused = 3,
    RateLimited = 4,
    InvalidSignature = 5,
    ExpiredToken = 6,
    MissingConsent = 7,
    PolicyDenied = 8,
    Custom(u32),
}

/// Record of a policy evaluation result.
#[derive(Clone)]
#[contracttype]
pub struct PolicyEvaluation {
    pub evaluation_id: u64,
    pub caller: Address,
    pub target_function: Symbol,
    pub decision: PolicyDecision,
    pub denial_reason: Option<DenialReason>,
    pub evaluated_at: u64,
    pub policy_name: Symbol,
}

/// Decision outcome of a policy evaluation.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[contracttype]
pub enum PolicyDecision {
    Allow,
    Deny,
}

/// An observable event emitted on authorization denial.
#[derive(Clone)]
#[contracttype]
pub struct AuthObservabilityEvent {
    pub event_id: u64,
    pub caller: Address,
    pub target_function: Symbol,
    pub denial_reason: DenialReason,
    pub timestamp: u64,
    pub metadata: Symbol,
}

/// Core authorization check: verifies a caller matches the expected admin address.
/// Returns `Ok(())` if authorized, `Err(())` otherwise.
pub fn check_admin(caller: &Address, admin: &Address) -> Result<(), ()> {
    if caller == admin {
        Ok(())
    } else {
        Err(())
    }
}

/// Convenience wrapper: returns `true` when the caller is the admin.
pub fn is_admin(caller: &Address, admin: &Address) -> bool {
    caller == admin
}

/// Macro to generate a `require_admin` function that reads the admin address
/// from instance storage using `DataKey::Admin` and returns an `Error`.
///
/// The calling module must have `DataKey::Admin` and `Error::NotInitialized`
/// / `Error::NotAuthorized` in scope.
///
/// # Example
/// ```ignore
/// require_admin!()
/// ```
/// expands to:
/// ```ignore
/// fn require_admin(env: &Env, caller: &Address) -> Result<(), Error> {
///     let admin: Address = env.storage().instance()
///         .get(&DataKey::Admin)
///         .ok_or(Error::NotInitialized)?;
///     if caller != &admin {
///         return Err(Error::NotAuthorized);
///     }
///     Ok(())
/// }
/// ```
#[macro_export]
macro_rules! require_admin {
    () => {
        fn require_admin(
            env: &soroban_sdk::Env,
            caller: &soroban_sdk::Address,
        ) -> Result<(), Error> {
            let admin: soroban_sdk::Address = env
                .storage()
                .instance()
                .get(&DataKey::Admin)
                .ok_or(Error::NotInitialized)?;
            if caller != &admin {
                return Err(Error::NotAuthorized);
            }
            Ok(())
        }
    };
}

/// Macro to generate a `require_admin` function with custom storage key,
/// storage type (`instance` or `persistent`), and error variants.
#[macro_export]
macro_rules! require_admin_custom {
    ($store:ident, $key:expr, $not_init:path, $not_auth:path) => {
        fn require_admin(
            env: &soroban_sdk::Env,
            caller: &soroban_sdk::Address,
        ) -> Result<(), Error> {
            let admin: soroban_sdk::Address = env
                .storage()
                .$store()
                .get(&$key)
                .ok_or($not_init)?;
            if caller != &admin {
                return Err($not_auth);
            }
            Ok(())
        }
    };
}

// ==================== Authorization Observability Types ====================

/// Reasons why an authorization attempt was denied.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[contracttype]
#[repr(u32)]
pub enum DenialReason {
    NotAdmin = 0,
    NotAuthorized = 1,
    InsufficientRole = 2,
    ContractPaused = 3,
    RateLimited = 4,
    InvalidSignature = 5,
    ExpiredToken = 6,
    MissingConsent = 7,
    PolicyDenied = 8,
    Custom(u32),
}

/// Record of a policy evaluation result.
#[derive(Clone)]
#[contracttype]
pub struct PolicyEvaluation {
    pub evaluation_id: u64,
    pub caller: Address,
    pub target_function: Symbol,
    pub decision: PolicyDecision,
    pub denial_reason: Option<DenialReason>,
    pub evaluated_at: u64,
    pub policy_name: Symbol,
}

/// Decision outcome of a policy evaluation.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[contracttype]
pub enum PolicyDecision {
    Allow,
    Deny,
}

/// An observable event emitted on authorization denial.
#[derive(Clone)]
#[contracttype]
pub struct AuthObservabilityEvent {
    pub event_id: u64,
    pub caller: Address,
    pub target_function: Symbol,
    pub denial_reason: DenialReason,
    pub timestamp: u64,
    pub metadata: Symbol,
}

// ==================== Authorization Observability Functions ====================

/// Log an authorization denial reason to persistent storage.
/// Emits an event for external monitoring and audit trail.
pub fn log_denial_reason(
    env: &Env,
    caller: &Address,
    target_function: Symbol,
    denial_reason: DenialReason,
) {
    let event_id: u64 = env
        .storage()
        .instance()
        .get(&soroban_sdk::symbol_short!("DEN_CNT"))
        .unwrap_or(0u64)
        .saturating_add(1);

    let event = AuthObservabilityEvent {
        event_id,
        caller: caller.clone(),
        target_function: target_function.clone(),
        denial_reason,
        timestamp: env.ledger().timestamp(),
        metadata: soroban_sdk::symbol_short!("DENIED"),
    };

    env.storage()
        .persistent()
        .set(&soroban_sdk::symbol_short!("DEN_EVT"), &event);
    env.storage()
        .instance()
        .set(&soroban_sdk::symbol_short!("DEN_CNT"), &event_id);

    env.events().publish(
        (soroban_sdk::symbol_short!("AUTH_DENY"), target_function),
        (caller.clone(), denial_reason as u32, event_id),
    );
}

/// Get the denial history count.
pub fn get_denial_history_count(env: &Env) -> u64 {
    env.storage()
        .instance()
        .get(&soroban_sdk::symbol_short!("DEN_CNT"))
        .unwrap_or(0u64)
}

/// Get the last denial event.
pub fn get_last_denial_event(env: &Env) -> Option<AuthObservabilityEvent> {
    env.storage()
        .persistent()
        .get(&soroban_sdk::symbol_short!("DEN_EVT"))
}

/// Evaluate a policy rule and return a decision.
/// Records the evaluation result for observability.
pub fn evaluate_policy(
    env: &Env,
    caller: &Address,
    target_function: Symbol,
    policy_name: Symbol,
    is_authorized: bool,
) -> PolicyDecision {
    let decision = if is_authorized {
        PolicyDecision::Allow
    } else {
        PolicyDecision::Deny
    };

    let evaluation_id: u64 = env
        .storage()
        .instance()
        .get(&soroban_sdk::symbol_short!("EVAL_CNT"))
        .unwrap_or(0u64)
        .saturating_add(1);

    let evaluation = PolicyEvaluation {
        evaluation_id,
        caller: caller.clone(),
        target_function: target_function.clone(),
        decision,
        denial_reason: if is_authorized {
            None
        } else {
            Some(DenialReason::PolicyDenied)
        },
        evaluated_at: env.ledger().timestamp(),
        policy_name,
    };

    env.storage()
        .persistent()
        .set(&soroban_sdk::symbol_short!("POL_EVAL"), &evaluation);
    env.storage()
        .instance()
        .set(&soroban_sdk::symbol_short!("EVAL_CNT"), &evaluation_id);

    env.events().publish(
        (soroban_sdk::symbol_short!("POLICY"), target_function),
        (caller.clone(), decision as u32, evaluation_id),
    );

    decision
}

/// Get the last policy evaluation.
pub fn get_last_policy_evaluation(env: &Env) -> Option<PolicyEvaluation> {
    env.storage()
        .persistent()
        .get(&soroban_sdk::symbol_short!("POL_EVAL"))
}
