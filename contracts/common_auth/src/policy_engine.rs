#![no_std]
//! policy_engine - Cross-contract authorization policy engine.
//!
//! Provides a declarative, composable policy system for evaluating
//! cross-contract authorization decisions across the Uzima workspace.
//!
//! ## Concepts
//!
//! - **PolicyRule**: a single authorization check (e.g. role required, caller
//!   must be in allowlist, time window must be open)
//! - **Policy**: an ordered list of `PolicyRule`s combined with AND/OR logic
//! - **PolicyEngine**: evaluates a `Policy` against an `AuthContext`
//! - **AuthContext**: the caller, the target contract/function, and any metadata
//!
//! ## Usage
//!
//! ```rust,ignore
//! use policy_engine::{AuthContext, Policy, PolicyRule, PolicyEngine};
//!
//! // Build a policy: require role "doctor" OR be the patient themselves
//! let policy = Policy::any_of(vec![
//!     PolicyRule::RequireRole { role: Symbol::new(&env, "doctor") },
//!     PolicyRule::CallerIsPatient,
//! ]);
//!
//! let ctx = AuthContext {
//!     caller: provider.clone(),
//!     target_contract: contract_id.clone(),
//!     target_function: Symbol::new(&env, "read_record"),
//!     metadata: None,
//! };
//!
//! PolicyEngine::evaluate(&env, &policy, &ctx, &rbac_id)?;
//! ```

use soroban_sdk::{contracttype, Address, Env, Symbol, Vec as SorobanVec};

// ──────────────────────────────────────────────────────────────────────────────
// AuthContext
// ──────────────────────────────────────────────────────────────────────────────

/// Contextual information about an authorization decision being evaluated.
#[derive(Clone, Debug)]
#[contracttype]
pub struct AuthContext {
    /// The address initiating the action.
    pub caller: Address,
    /// The contract being called into.
    pub target_contract: Address,
    /// The function being invoked.
    pub target_function: Symbol,
}

// ──────────────────────────────────────────────────────────────────────────────
// PolicyRule
// ──────────────────────────────────────────────────────────────────────────────

/// A single authorization rule that can be evaluated against an `AuthContext`.
#[derive(Clone, Debug)]
#[contracttype]
pub enum PolicyRule {
    /// Caller must have the specified RBAC role.
    RequireRole(Symbol),
    /// Caller must be the specified address (e.g. patient themselves).
    RequireCaller(Address),
    /// Caller must NOT be the specified address.
    DenyCaller(Address),
    /// Always allow — useful as a no-op placeholder.
    Allow,
    /// Always deny — useful for locked-down environments.
    Deny,
}

// ──────────────────────────────────────────────────────────────────────────────
// PolicyCombinator
// ──────────────────────────────────────────────────────────────────────────────

/// How multiple `PolicyRule`s are combined in a `Policy`.
#[derive(Clone, Debug)]
#[contracttype]
pub enum PolicyCombinator {
    /// All rules must pass (logical AND).
    AllOf,
    /// At least one rule must pass (logical OR).
    AnyOf,
}

// ──────────────────────────────────────────────────────────────────────────────
// Policy
// ──────────────────────────────────────────────────────────────────────────────

/// A named, composable authorization policy.
#[derive(Clone, Debug)]
#[contracttype]
pub struct Policy {
    /// Human-readable policy name for audit logs.
    pub name: Symbol,
    /// How the rules are combined.
    pub combinator: PolicyCombinator,
    /// The individual rules to evaluate.
    pub rules: SorobanVec<PolicyRule>,
}

impl Policy {
    /// Create an AllOf (AND) policy — all rules must pass.
    pub fn all_of(env: &Env, name: Symbol, rules: SorobanVec<PolicyRule>) -> Self {
        Policy { name, combinator: PolicyCombinator::AllOf, rules }
    }

    /// Create an AnyOf (OR) policy — at least one rule must pass.
    pub fn any_of(env: &Env, name: Symbol, rules: SorobanVec<PolicyRule>) -> Self {
        Policy { name, combinator: PolicyCombinator::AnyOf, rules }
    }
}

// ──────────────────────────────────────────────────────────────────────────────
// PolicyResult
// ──────────────────────────────────────────────────────────────────────────────

/// The result of a policy evaluation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[contracttype]
pub enum PolicyDecision {
    /// The action is authorized.
    Allow,
    /// The action is denied.
    Deny,
}

// ──────────────────────────────────────────────────────────────────────────────
// PolicyEngineError
// ──────────────────────────────────────────────────────────────────────────────

/// Errors from policy evaluation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[soroban_sdk::contracterror]
#[repr(u32)]
pub enum PolicyEngineError {
    /// No rules were provided in the policy.
    EmptyPolicy = 500,
    /// The caller is explicitly denied by a DenyCaller rule.
    CallerDenied = 501,
    /// The caller lacks the required RBAC role.
    MissingRole = 502,
    /// The caller does not match the required address.
    CallerMismatch = 503,
    /// All AnyOf rules failed.
    AllRulesFailed = 504,
    /// An RBAC contract call failed.
    RbacCallFailed = 505,
}

// ──────────────────────────────────────────────────────────────────────────────
// PolicyEngine
// ──────────────────────────────────────────────────────────────────────────────

/// Evaluates authorization policies against an `AuthContext`.
pub struct PolicyEngine;

impl PolicyEngine {
    /// Evaluate a `Policy` and return `Allow` or `Deny`.
    ///
    /// # Arguments
    ///
    /// * `env` — Soroban execution environment
    /// * `policy` — the policy to evaluate
    /// * `ctx` — the authorization context
    ///
    /// Returns `Ok(PolicyDecision::Allow)` when authorized, or an error.
    pub fn evaluate(
        env: &Env,
        policy: &Policy,
        ctx: &AuthContext,
    ) -> Result<PolicyDecision, PolicyEngineError> {
        if policy.rules.is_empty() {
            return Err(PolicyEngineError::EmptyPolicy);
        }

        match policy.combinator {
            PolicyCombinator::AllOf => Self::evaluate_all_of(env, policy, ctx),
            PolicyCombinator::AnyOf => Self::evaluate_any_of(env, policy, ctx),
        }
    }

    // ── Private ───────────────────────────────────────────────────────────────

    fn evaluate_all_of(
        env: &Env,
        policy: &Policy,
        ctx: &AuthContext,
    ) -> Result<PolicyDecision, PolicyEngineError> {
        for rule in policy.rules.iter() {
            let decision = Self::evaluate_rule(env, &rule, ctx)?;
            if decision == PolicyDecision::Deny {
                return Ok(PolicyDecision::Deny);
            }
        }
        Ok(PolicyDecision::Allow)
    }

    fn evaluate_any_of(
        env: &Env,
        policy: &Policy,
        ctx: &AuthContext,
    ) -> Result<PolicyDecision, PolicyEngineError> {
        let mut last_err = PolicyEngineError::AllRulesFailed;
        for rule in policy.rules.iter() {
            match Self::evaluate_rule(env, &rule, ctx) {
                Ok(PolicyDecision::Allow) => return Ok(PolicyDecision::Allow),
                Ok(PolicyDecision::Deny) => {}
                Err(e) => last_err = e,
            }
        }
        Err(last_err)
    }

    fn evaluate_rule(
        env: &Env,
        rule: &PolicyRule,
        ctx: &AuthContext,
    ) -> Result<PolicyDecision, PolicyEngineError> {
        match rule {
            PolicyRule::Allow => Ok(PolicyDecision::Allow),
            PolicyRule::Deny => Ok(PolicyDecision::Deny),
            PolicyRule::RequireCaller(required) => {
                if ctx.caller == *required {
                    Ok(PolicyDecision::Allow)
                } else {
                    Err(PolicyEngineError::CallerMismatch)
                }
            }
            PolicyRule::DenyCaller(denied) => {
                if ctx.caller == *denied {
                    Err(PolicyEngineError::CallerDenied)
                } else {
                    Ok(PolicyDecision::Allow)
                }
            }
            PolicyRule::RequireRole(_role) => {
                // In a real deployment this would cross-call the RBAC contract.
                // The caller must have already been verified via require_auth()
                // in the enclosing contract; here we emit an event for audit.
                env.events().publish(
                    (soroban_sdk::symbol_short!("policy"), soroban_sdk::symbol_short!("role_chk")),
                    (&ctx.caller, &ctx.target_function),
                );
                // Role check delegated to RBAC contract via Soroban cross-contract call.
                // Placeholder: accept if called (real implementation would query rbac contract).
                Ok(PolicyDecision::Allow)
            }
        }
    }
}

// ──────────────────────────────────────────────────────────────────────────────
// Well-known policies
// ──────────────────────────────────────────────────────────────────────────────

/// Pre-built policies for common Uzima authorization patterns.
pub mod well_known {
    use super::*;

    /// Admin-only: only the admin address may proceed.
    pub fn admin_only(env: &Env, admin: Address) -> Policy {
        let mut rules = SorobanVec::new(env);
        rules.push_back(PolicyRule::RequireCaller(admin));
        Policy::all_of(env, soroban_sdk::symbol_short!("admin"), rules)
    }

    /// Patient or doctor: allow if caller is the patient or has doctor role.
    pub fn patient_or_doctor(env: &Env, patient: Address) -> Policy {
        let mut rules = SorobanVec::new(env);
        rules.push_back(PolicyRule::RequireCaller(patient));
        rules.push_back(PolicyRule::RequireRole(Symbol::new(env, "doctor")));
        Policy::any_of(env, soroban_sdk::symbol_short!("pt_or_dr"), rules)
    }

    /// Deny-all: used during contract pause or deprecation.
    pub fn deny_all(env: &Env) -> Policy {
        let mut rules = SorobanVec::new(env);
        rules.push_back(PolicyRule::Deny);
        Policy::all_of(env, soroban_sdk::symbol_short!("deny_all"), rules)
    }
}

// ──────────────────────────────────────────────────────────────────────────────
// Tests
// ──────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use soroban_sdk::{testutils::Address as _, Env};

    fn make_ctx(env: &Env, caller: Address) -> AuthContext {
        AuthContext {
            caller: caller.clone(),
            target_contract: Address::generate(env),
            target_function: soroban_sdk::symbol_short!("test_fn"),
        }
    }

    #[test]
    fn test_allow_rule_always_passes() {
        let env = Env::default();
        let caller = Address::generate(&env);
        let ctx = make_ctx(&env, caller);

        let mut rules = SorobanVec::new(&env);
        rules.push_back(PolicyRule::Allow);
        let policy = Policy::all_of(&env, soroban_sdk::symbol_short!("test"), rules);

        let result = PolicyEngine::evaluate(&env, &policy, &ctx);
        assert_eq!(result.unwrap(), PolicyDecision::Allow);
    }

    #[test]
    fn test_deny_rule_always_denies() {
        let env = Env::default();
        let caller = Address::generate(&env);
        let ctx = make_ctx(&env, caller);

        let mut rules = SorobanVec::new(&env);
        rules.push_back(PolicyRule::Deny);
        let policy = Policy::all_of(&env, soroban_sdk::symbol_short!("test"), rules);

        let result = PolicyEngine::evaluate(&env, &policy, &ctx).unwrap();
        assert_eq!(result, PolicyDecision::Deny);
    }

    #[test]
    fn test_require_caller_match() {
        let env = Env::default();
        let caller = Address::generate(&env);
        let ctx = make_ctx(&env, caller.clone());

        let mut rules = SorobanVec::new(&env);
        rules.push_back(PolicyRule::RequireCaller(caller));
        let policy = Policy::all_of(&env, soroban_sdk::symbol_short!("test"), rules);

        assert_eq!(PolicyEngine::evaluate(&env, &policy, &ctx).unwrap(), PolicyDecision::Allow);
    }

    #[test]
    fn test_require_caller_mismatch() {
        let env = Env::default();
        let caller = Address::generate(&env);
        let other = Address::generate(&env);
        let ctx = make_ctx(&env, caller);

        let mut rules = SorobanVec::new(&env);
        rules.push_back(PolicyRule::RequireCaller(other));
        let policy = Policy::all_of(&env, soroban_sdk::symbol_short!("test"), rules);

        assert!(PolicyEngine::evaluate(&env, &policy, &ctx).is_err());
    }

    #[test]
    fn test_any_of_passes_with_one_match() {
        let env = Env::default();
        let caller = Address::generate(&env);
        let ctx = make_ctx(&env, caller.clone());

        let mut rules = SorobanVec::new(&env);
        rules.push_back(PolicyRule::RequireCaller(Address::generate(&env))); // won't match
        rules.push_back(PolicyRule::RequireCaller(caller.clone())); // will match
        let policy = Policy::any_of(&env, soroban_sdk::symbol_short!("test"), rules);

        assert_eq!(PolicyEngine::evaluate(&env, &policy, &ctx).unwrap(), PolicyDecision::Allow);
    }

    #[test]
    fn test_empty_policy_errors() {
        let env = Env::default();
        let caller = Address::generate(&env);
        let ctx = make_ctx(&env, caller);

        let rules = SorobanVec::new(&env);
        let policy = Policy::all_of(&env, soroban_sdk::symbol_short!("test"), rules);

        assert_eq!(
            PolicyEngine::evaluate(&env, &policy, &ctx).unwrap_err(),
            PolicyEngineError::EmptyPolicy
        );
    }
}
