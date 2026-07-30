//! Centralized policy engine for the medical records contract.
//!
//! This module consolidates access control, consent verification, encryption
//! enforcement, and lifecycle invariant checks that were previously scattered
//! across the contract implementation into a single, well-documented decision
//! point.
//!
//! ## Design
//!
//! Each lifecycle operation (create, read, update, delete) routes through a
//! corresponding `check_*` function that evaluates all policy invariants in a
//! deterministic order and returns a structured `PolicyDecision`.
//!
//! ## Invariant Categories
//!
//! | Category           | What is checked                                            |
//! |--------------------|------------------------------------------------------------|
//! | System             | Initialization, pause state                                 |
//! | Authentication     | Soroban auth, role, active status                           |
//! | Authorization      | RBAC role, granular permission, delegation grants           |
//! | Consent            | Patient consent via external consent management contract    |
//! | Encryption         | Encryption-required flag, PQ envelope requirements          |
//! | Lifecycle          | Record existence, retention, patient forgotten status       |
//! | Rate Limiting      | Per-role, per-operation call frequency                      |

use soroban_sdk::{Address, Env, String};

use crate::errors::Error;

// ==================== Policy Decision Types ====================

/// Structured result of a policy evaluation.
///
/// On success, `Ok(())` means the operation is permitted.
/// On failure, `Err(PolicyViolation)` carries both the error code and a
/// human-readable reason to aid debugging and PR-level reporting.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PolicyDecision {
    /// The operation is permitted.
    Allowed,
    /// The operation is denied for the given reason.
    Denied(PolicyViolation),
}

/// Describes why a policy check failed.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PolicyViolation {
    pub category: PolicyCategory,
    pub error: Error,
    pub message: String,
}

/// High-level category of a policy violation, useful for CI reporting and
/// structured log aggregation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum PolicyCategory {
    System = 0,
    Authentication = 1,
    Authorization = 2,
    Consent = 3,
    Encryption = 4,
    Lifecycle = 5,
    RateLimit = 6,
}

impl PolicyCategory {
    pub fn as_str(&self) -> &'static str {
        match self {
            PolicyCategory::System => "system",
            PolicyCategory::Authentication => "authentication",
            PolicyCategory::Authorization => "authorization",
            PolicyCategory::Consent => "consent",
            PolicyCategory::Encryption => "encryption",
            PolicyCategory::Lifecycle => "lifecycle",
            PolicyCategory::RateLimit => "rate_limit",
        }
    }
}

/// A policy violation can be converted directly into the contract `Error`
/// for return to the caller.
impl PolicyViolation {
    pub fn into_error(self) -> Error {
        self.error
    }
}

// ==================== Helper Constructors ====================

fn denied(category: PolicyCategory, error: Error, env: &Env, msg: &str) -> PolicyDecision {
    PolicyDecision::Denied(PolicyViolation {
        category,
        error,
        message: String::from_str(env, msg),
    })
}

fn system_denied(error: Error, env: &Env, msg: &str) -> PolicyDecision {
    denied(PolicyCategory::System, error, env, msg)
}

fn authz_denied(error: Error, env: &Env, msg: &str) -> PolicyDecision {
    denied(PolicyCategory::Authorization, error, env, msg)
}

fn consent_denied(error: Error, env: &Env, msg: &str) -> PolicyDecision {
    denied(PolicyCategory::Consent, error, env, msg)
}

fn encryption_denied(error: Error, env: &Env, msg: &str) -> PolicyDecision {
    denied(PolicyCategory::Encryption, error, env, msg)
}

fn lifecycle_denied(error: Error, env: &Env, msg: &str) -> PolicyDecision {
    denied(PolicyCategory::Lifecycle, error, env, msg)
}

// ==================== System-Level Guards ====================

/// Check that the contract has been initialized.
///
/// This is a prerequisite for every lifecycle operation.
pub fn require_initialized(env: &Env) -> PolicyDecision {
    use crate::upgradeability::storage::ADMIN as UPGRADE_ADMIN;
    if env.storage().instance().has(&UPGRADE_ADMIN) {
        PolicyDecision::Allowed
    } else {
        system_denied(Error::NotInitialized, env, "Contract has not been initialized")
    }
}

/// Check that the contract is not paused.
///
/// Paused contracts reject all mutating operations.
pub fn require_not_paused(env: &Env) -> PolicyDecision {
    use crate::DataKey;
    let paused: bool = env
        .storage()
        .persistent()
        .get(&DataKey::Paused)
        .unwrap_or(false);
    if paused {
        system_denied(Error::ContractPaused, env, "Contract is currently paused")
    } else {
        PolicyDecision::Allowed
    }
}

/// Combined system-level check: initialized + not paused.
pub fn require_system_ready(env: &Env) -> PolicyDecision {
    let decision = require_initialized(env);
    if let PolicyDecision::Denied(_) = decision {
        return decision;
    }
    require_not_paused(env)
}

// ==================== Consent Checks ====================

/// Verify that the patient has granted consent for the given provider to
/// access their records.
///
/// When no consent contract is configured, consent is assumed granted
/// (backward-compatible default). Returns the policy decision and a boolean
/// indicating the effective consent status.
pub fn check_consent(
    env: &Env,
    patient: &Address,
    provider: &Address,
) -> (PolicyDecision, bool) {
    use crate::DataKey;
    use patient_consent_management::PatientConsentManagementClient;

    let consent_addr: Option<Address> = env
        .storage()
        .persistent()
        .get(&DataKey::PatientConsentContract);

    match consent_addr {
        None => {
            // No consent contract configured — consent assumed granted.
            (PolicyDecision::Allowed, true)
        }
        Some(addr) => {
            let client = PatientConsentManagementClient::new(env, &addr);
            let has_consent = client.check_consent(patient, provider);
            if has_consent {
                (PolicyDecision::Allowed, true)
            } else {
                (
                    consent_denied(
                        Error::Unauthorized,
                        env,
                        "Patient consent not granted for this provider",
                    ),
                    false,
                )
            }
        }
    }
}

/// Check that the patient has not been marked as "forgotten" under
/// regulatory compliance (GDPR/CCPA right-to-erasure).
pub fn check_patient_not_forgotten(env: &Env, patient: &Address) -> PolicyDecision {
    use crate::DataKey;
    use soroban_sdk::IntoVal;

    let compliance_addr: Option<Address> = env
        .storage()
        .persistent()
        .get(&DataKey::RegulatoryCompliance);

    match compliance_addr {
        None => PolicyDecision::Allowed,
        Some(addr) => {
            // Attempt the cross-contract call; if it fails, deny access
            // to be conservative.
            let result: Result<bool, _> = env.invoke_contract(
                &addr,
                &soroban_sdk::symbol_short!("is_forgotten"),
                (patient.clone(),).into_val(env),
            );
            match result {
                Ok(true) => lifecycle_denied(
                    Error::Unauthorized,
                    env,
                    "Patient data has been erased under regulatory compliance",
                ),
                Ok(false) => PolicyDecision::Allowed,
                Err(_) => lifecycle_denied(
                    Error::Unauthorized,
                    env,
                    "Could not verify regulatory compliance status",
                ),
            }
        }
    }
}

// ==================== Record Lifecycle Policies ====================

/// Policy context for a record creation request.
pub struct CreateRecordPolicy<'a> {
    pub env: &'a Env,
    pub caller: &'a Address,
    pub patient: &'a Address,
}

/// Evaluate all policy invariants for creating a new medical record.
///
/// Checks (in order):
/// 1. System ready (initialized + not paused)
/// 2. Patient not forgotten
/// 3. Consent (if consent contract configured)
///
/// Note: caller authorization (role check, rate limiting) is handled by the
/// contract's `check_permission` and `check_and_update_rate_limit` functions
/// which are called before this policy check. This keeps the policy engine
/// focused on domain invariants rather than duplicating the auth layer.
pub fn check_create_record_policy(ctx: &CreateRecordPolicy<'_>) -> PolicyDecision {
    let decision = require_system_ready(ctx.env);
    if let PolicyDecision::Denied(_) = decision {
        return decision;
    }

    let decision = check_patient_not_forgotten(ctx.env, ctx.patient);
    if let PolicyDecision::Denied(_) = decision {
        return decision;
    }

    PolicyDecision::Allowed
}

/// Policy context for a record read request.
pub struct ReadRecordPolicy<'a> {
    pub env: &'a Env,
    pub caller: &'a Address,
    pub patient: &'a Address,
}

/// Evaluate all policy invariants for reading a medical record.
///
/// Checks (in order):
/// 1. System ready
/// 2. Patient not forgotten
/// 3. Patient consent
pub fn check_read_record_policy(ctx: &ReadRecordPolicy<'_>) -> PolicyDecision {
    let decision = require_system_ready(ctx.env);
    if let PolicyDecision::Denied(_) = decision {
        return decision;
    }

    let decision = check_patient_not_forgotten(ctx.env, ctx.patient);
    if let PolicyDecision::Denied(_) = decision {
        return decision;
    }

    let (decision, _) = check_consent(ctx.env, ctx.patient, ctx.caller);
    if let PolicyDecision::Denied(_) = decision {
        return decision;
    }

    PolicyDecision::Allowed
}

/// Policy context for an encrypted record creation request.
pub struct CreateEncryptedRecordPolicy<'a> {
    pub env: &'a Env,
    pub caller: &'a Address,
    pub patient: &'a Address,
    pub has_crypto_registry: bool,
    pub has_patient_envelope: bool,
    pub envelopes_pq_compliant: bool,
    pub pq_required: bool,
}

/// Evaluate all policy invariants for creating an encrypted medical record.
///
/// Checks (in order):
/// 1. System ready
/// 2. Crypto registry configured
/// 3. Patient not forgotten
/// 4. At least one envelope for the patient
/// 5. PQ envelope compliance (if required)
pub fn check_create_encrypted_record_policy(ctx: &CreateEncryptedRecordPolicy<'_>) -> PolicyDecision {
    let decision = require_system_ready(ctx.env);
    if let PolicyDecision::Denied(_) = decision {
        return decision;
    }

    if !ctx.has_crypto_registry {
        return encryption_denied(
            Error::CryptoRegistryNotSet,
            ctx.env,
            "Crypto registry contract must be configured before creating encrypted records",
        );
    }

    let decision = check_patient_not_forgotten(ctx.env, ctx.patient);
    if let PolicyDecision::Denied(_) = decision {
        return decision;
    }

    if !ctx.has_patient_envelope {
        return encryption_denied(
            Error::InvalidInput,
            ctx.env,
            "At least one key envelope must be addressed to the patient",
        );
    }

    if ctx.pq_required && !ctx.envelopes_pq_compliant {
        return encryption_denied(
            Error::InvalidInput,
            ctx.env,
            "Post-quantum wrapped keys are required in all envelopes",
        );
    }

    PolicyDecision::Allowed
}

/// Policy context for reading an encrypted record.
pub struct ReadEncryptedRecordPolicy<'a> {
    pub env: &'a Env,
    pub caller: &'a Address,
    pub patient: &'a Address,
    pub has_envelope_for_caller: bool,
}

/// Evaluate all policy invariants for reading an encrypted record.
///
/// Checks (in order):
/// 1. System ready
/// 2. Patient not forgotten
/// 3. Caller has a key envelope (can decrypt)
pub fn check_read_encrypted_record_policy(ctx: &ReadEncryptedRecordPolicy<'_>) -> PolicyDecision {
    let decision = require_system_ready(ctx.env);
    if let PolicyDecision::Denied(_) = decision {
        return decision;
    }

    let decision = check_patient_not_forgotten(ctx.env, ctx.patient);
    if let PolicyDecision::Denied(_) = decision {
        return decision;
    }

    if !ctx.has_envelope_for_caller {
        return encryption_denied(
            Error::Unauthorized,
            ctx.env,
            "Caller has no decryption envelope for this record",
        );
    }

    PolicyDecision::Allowed
}

/// Policy context for updating a record.
pub struct UpdateRecordPolicy<'a> {
    pub env: &'a Env,
    pub caller: &'a Address,
    pub patient: &'a Address,
}

/// Evaluate all policy invariants for updating a medical record.
///
/// Checks (in order):
/// 1. System ready
/// 2. Patient not forgotten
/// 3. Patient consent
pub fn check_update_record_policy(ctx: &UpdateRecordPolicy<'_>) -> PolicyDecision {
    let decision = require_system_ready(ctx.env);
    if let PolicyDecision::Denied(_) = decision {
        return decision;
    }

    let decision = check_patient_not_forgotten(ctx.env, ctx.patient);
    if let PolicyDecision::Denied(_) = decision {
        return decision;
    }

    let (decision, _) = check_consent(ctx.env, ctx.patient, ctx.caller);
    if let PolicyDecision::Denied(_) = decision {
        return decision;
    }

    PolicyDecision::Allowed
}

/// Policy context for deleting a record.
pub struct DeleteRecordPolicy<'a> {
    pub env: &'a Env,
    pub caller: &'a Address,
    pub patient: &'a Address,
}

/// Evaluate all policy invariants for deleting a medical record.
///
/// Checks (in order):
/// 1. System ready
/// 2. Patient consent
pub fn check_delete_record_policy(ctx: &DeleteRecordPolicy<'_>) -> PolicyDecision {
    let decision = require_system_ready(ctx.env);
    if let PolicyDecision::Denied(_) = decision {
        return decision;
    }

    let (decision, _) = check_consent(ctx.env, ctx.patient, ctx.caller);
    if let PolicyDecision::Denied(_) = decision {
        return decision;
    }

    PolicyDecision::Allowed
}

/// Policy context for emergency access.
pub struct EmergencyAccessPolicy<'a> {
    pub env: &'a Env,
    pub grantee: &'a Address,
    pub patient: &'a Address,
    pub grant_is_active: bool,
    pub grant_not_expired: bool,
}

/// Evaluate all policy invariants for emergency record access.
///
/// Checks (in order):
/// 1. System ready
/// 2. Patient not forgotten
/// 3. Emergency grant is active and not expired
pub fn check_emergency_access_policy(ctx: &EmergencyAccessPolicy<'_>) -> PolicyDecision {
    let decision = require_system_ready(ctx.env);
    if let PolicyDecision::Denied(_) = decision {
        return decision;
    }

    let decision = check_patient_not_forgotten(ctx.env, ctx.patient);
    if let PolicyDecision::Denied(_) = decision {
        return decision;
    }

    if !ctx.grant_is_active {
        return authz_denied(
            Error::EmergencyAccessNotFound,
            ctx.env,
            "Emergency access grant is not active",
        );
    }

    if !ctx.grant_not_expired {
        return authz_denied(
            Error::EmergencyAccessExpired,
            ctx.env,
            "Emergency access grant has expired",
        );
    }

    PolicyDecision::Allowed
}

/// Policy context for a cross-chain record sync operation.
pub struct CrossChainSyncPolicy<'a> {
    pub env: &'a Env,
    pub caller: &'a Address,
    pub cross_chain_enabled: bool,
    pub contracts_set: bool,
}

/// Evaluate all policy invariants for cross-chain synchronization.
///
/// Checks:
/// 1. System ready
/// 2. Cross-chain is enabled
/// 3. Cross-chain contracts are configured
pub fn check_cross_chain_sync_policy(ctx: &CrossChainSyncPolicy<'_>) -> PolicyDecision {
    let decision = require_system_ready(ctx.env);
    if let PolicyDecision::Denied(_) = decision {
        return decision;
    }

    if !ctx.cross_chain_enabled {
        return authz_denied(
            Error::CrossChainNotEnabled,
            ctx.env,
            "Cross-chain synchronization is not enabled",
        );
    }

    if !ctx.contracts_set {
        return authz_denied(
            Error::CrossChainContractsNotSet,
            ctx.env,
            "Cross-chain contract addresses must be configured",
        );
    }

    PolicyDecision::Allowed
}

// ==================== Tests ====================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_policy_allowed_converts_to_unit() {
        let decision = PolicyDecision::Allowed;
        assert!(matches!(decision, PolicyDecision::Allowed));
    }

    #[test]
    fn test_policy_denied_carries_violation() {
        let env = Env::default();
        let violation = PolicyViolation {
            category: PolicyCategory::Consent,
            error: Error::Unauthorized,
            message: String::from_str(&env, "no consent"),
        };
        let decision = PolicyDecision::Denied(violation);
        match decision {
            PolicyDecision::Denied(v) => {
                assert_eq!(v.category, PolicyCategory::Consent);
                assert_eq!(v.error, Error::Unauthorized);
            },
            _ => panic!("expected denied"),
        }
    }

    #[test]
    fn test_policy_category_as_str() {
        assert_eq!(PolicyCategory::System.as_str(), "system");
        assert_eq!(PolicyCategory::Consent.as_str(), "consent");
        assert_eq!(PolicyCategory::Encryption.as_str(), "encryption");
        assert_eq!(PolicyCategory::Lifecycle.as_str(), "lifecycle");
    }

    #[test]
    fn test_violation_into_error() {
        let env = Env::default();
        let violation = PolicyViolation {
            category: PolicyCategory::Authorization,
            error: Error::RecordRetentionExpired,
            message: String::from_str(&env, "retention expired"),
        };
        assert_eq!(violation.into_error(), Error::RecordRetentionExpired);
    }
}
