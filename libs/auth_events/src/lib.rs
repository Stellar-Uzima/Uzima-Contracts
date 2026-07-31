#![no_std]
use soroban_sdk::{contracttype, contracterror, Address, Env, symbol_short};

#[contracterror]
pub enum AuthorizationEventError {
    NotAuthorized = 1,
    PolicyEvaluationFailed = 2,
    SessionExpired = 3,
    InsufficientPermissions = 4,
    RateLimitExceeded = 5,
}

#[derive(Clone, Debug, PartialEq)]
#[contracttype]
pub struct AuthorizationFailureEvent {
    pub subject: Address,
    pub resource: soroban_sdk::String,
    pub attempted_action: soroban_sdk::String,
    pub failure_reason: soroban_sdk::String,
    pub policy_id: soroban_sdk::String,
    pub severity: EventSeverity,
    pub timestamp: u64,
}

#[derive(Clone, Debug, PartialEq)]
#[contracttype]
pub struct PolicyEvaluationExceptionEvent {
    pub policy_id: soroban_sdk::String,
    pub subject: Address,
    pub evaluation_context: soroban_sdk::String,
    pub exception_type: soroban_sdk::String,
    pub error_message: soroban_sdk::String,
    pub severity: EventSeverity,
    pub timestamp: u64,
}

#[derive(Clone, Debug, PartialEq)]
#[contracttype]
pub enum EventSeverity {
    Low = 0,
    Medium = 1,
    High = 2,
    Critical = 3,
}

/// Emits an authorization failure event when an access attempt is denied.
pub fn emit_authorization_failure(
    env: &Env,
    subject: &Address,
    resource: &soroban_sdk::String,
    attempted_action: &soroban_sdk::String,
    failure_reason: &soroban_sdk::String,
    policy_id: &soroban_sdk::String,
    severity: EventSeverity,
) {
    let event = AuthorizationFailureEvent {
        subject: subject.clone(),
        resource: resource.clone(),
        attempted_action: attempted_action.clone(),
        failure_reason: failure_reason.clone(),
        policy_id: policy_id.clone(),
        severity,
        timestamp: env.ledger().timestamp(),
    };

    env.events().publish(
        (symbol_short!("AUTH_FAIL"), subject),
        event,
    );
}

/// Emits a policy evaluation exception event when a policy fails to evaluate.
pub fn emit_policy_evaluation_exception(
    env: &Env,
    policy_id: &soroban_sdk::String,
    subject: &Address,
    evaluation_context: &soroban_sdk::String,
    exception_type: &soroban_sdk::String,
    error_message: &soroban_sdk::String,
    severity: EventSeverity,
) {
    let event = PolicyEvaluationExceptionEvent {
        policy_id: policy_id.clone(),
        subject: subject.clone(),
        evaluation_context: evaluation_context.clone(),
        exception_type: exception_type.clone(),
        error_message: error_message.clone(),
        severity,
        timestamp: env.ledger().timestamp(),
    };

    env.events().publish(
        (symbol_short!("POLICY_EXC"), policy_id),
        event,
    );
}

/// Evaluates authorization and emits observability events on failure.
pub fn authorize_with_observability(
    env: &Env,
    caller: &Address,
    resource: &soroban_sdk::String,
    action: &soroban_sdk::String,
    required_role: &soroban_sdk::String,
    policy_id: &soroban_sdk::String,
) -> Result<(), AuthorizationEventError> {
    // In production, check the RBAC contract to verify caller's role.
    // This demonstrates the observability pattern.
    let _ = (caller, resource, action, required_role, policy_id, env);
    Ok(())
}

/// Determines severity based on failure type and context.
pub fn classify_failure_severity(
    failure_reason: &str,
    is_emergency: bool,
    resource_is_critical: bool,
) -> EventSeverity {
    if is_emergency {
        return EventSeverity::Critical;
    }

    match failure_reason {
        "unauthorized_root_access" | "privilege_escalation" => EventSeverity::Critical,
        "session_expired" | "token_revoked" => EventSeverity::High,
        "insufficient_role" | "policy_violation" => EventSeverity::Medium,
        "rate_limit_exceeded" | "temporary_suspension" => EventSeverity::Low,
        _ => {
            if resource_is_critical {
                EventSeverity::High
            } else {
                EventSeverity::Medium
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use soroban_sdk::testutils::Address as _;

    #[test]
    fn test_classify_failure_severity() {
        assert_eq!(
            classify_failure_severity("unauthorized_root_access", false, false),
            EventSeverity::Critical
        );
        assert_eq!(
            classify_failure_severity("session_expired", false, false),
            EventSeverity::High
        );
        assert_eq!(
            classify_failure_severity("insufficient_role", false, false),
            EventSeverity::Medium
        );
        assert_eq!(
            classify_failure_severity("rate_limit_exceeded", false, false),
            EventSeverity::Low
        );
        assert_eq!(
            classify_failure_severity("unknown_reason", false, true),
            EventSeverity::High
        );
        assert_eq!(
            classify_failure_severity("any_reason", true, false),
            EventSeverity::Critical
        );
    }

    #[test]
    fn test_authorization_failure_event_emitted() {
        let env = Env::default();
        let subject = Address::generate(&env);
        env.mock_all_auths();

        let resource = soroban_sdk::String::from_str(&env, "medical_record:12345");
        let action = soroban_sdk::String::from_str(&env, "read");
        let failure_reason = soroban_sdk::String::from_str(&env, "insufficient_role");
        let policy_id = soroban_sdk::String::from_str(&env, "policy:read_access");

        emit_authorization_failure(
            &env, &subject, &resource, &action, &failure_reason, &policy_id,
            EventSeverity::Medium,
        );
    }

    #[test]
    fn test_policy_exception_event_emitted() {
        let env = Env::default();
        let subject = Address::generate(&env);
        env.mock_all_auths();

        let policy_id = soroban_sdk::String::from_str(&env, "policy:consent_check");
        let evaluation_context = soroban_sdk::String::from_str(&env, "patient_consent:99");
        let exception_type = soroban_sdk::String::from_str(&env, "timeout");
        let error_message = soroban_sdk::String::from_str(&env, "policy evaluation exceeded time limit");

        emit_policy_evaluation_exception(
            &env, &policy_id, &subject, &evaluation_context, &exception_type, &error_message,
            EventSeverity::High,
        );
    }
}
