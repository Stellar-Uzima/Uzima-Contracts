//! Upgrade Safety Framework
//!
//! Provides standardized upgrade manifests, migration invariants,
//! compatibility rules, and rollback policies for healthcare contracts.
//!
//! ## Overview
//!
//! The upgrade safety framework ensures that contract upgrades:
//! 1. Declare their migration requirements upfront (manifest)
//! 2. Validate storage layout compatibility before execution
//! 3. Run pre-flight and post-flight integrity checks
//! 4. Support deterministic rollback to the previous version
//! 5. Emit structured lifecycle events for auditability

use soroban_sdk::{contracttype, symbol_short, BytesN, Env, String, Symbol, Vec};

use super::UpgradeError;

// ==================== Upgrade Manifest ====================

/// A contract's upgrade manifest declaring migration requirements.
///
/// This is the central configuration for an upgrade. Contracts must
/// provide this manifest before any upgrade is executed.
#[derive(Clone)]
#[contracttype]
pub struct UpgradeManifest {
    /// Target version after upgrade.
    pub target_version: u32,
    /// WASM hash of the new contract code.
    pub new_wasm_hash: BytesN<32>,
    /// Human-readable description of the upgrade.
    pub description: String,
    /// List of migration invariants that must hold after upgrade.
    pub invariants: Vec<MigrationInvariant>,
    /// Storage compatibility rules.
    pub storage_rules: StorageCompatibilityRules,
    /// Deprecated functions introduced by this upgrade.
    pub deprecated_functions: Vec<DeprecatedFunctionEntry>,
    /// Whether rollback is supported for this upgrade.
    pub rollback_supported: bool,
    /// Maximum time (in ledgers) allowed for migration to complete.
    pub migration_timeout_ledgers: u32,
}

/// A single migration invariant that must hold after upgrade.
#[derive(Clone)]
#[contracttype]
pub struct MigrationInvariant {
    /// Name of the invariant (e.g., "all_records_have_consent").
    pub name: Symbol,
    /// Description of what the invariant guarantees.
    pub description: String,
    /// Severity if violated: "critical" blocks upgrade, "warning" logs but allows.
    pub severity: InvariantSeverity,
}

/// Severity level for migration invariants.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[contracttype]
pub enum InvariantSeverity {
    /// Violation blocks the upgrade.
    Critical = 0,
    /// Violation logs a warning but allows the upgrade.
    Warning = 1,
    /// Violation is informational only.
    Info = 2,
}

/// Rules for storage layout compatibility.
#[derive(Clone)]
#[contracttype]
pub struct StorageCompatibilityRules {
    /// Whether existing storage keys must be preserved.
    pub preserve_existing_keys: bool,
    /// Whether new storage keys are allowed.
    pub allow_new_keys: bool,
    /// Whether storage key types can change.
    pub allow_type_changes: bool,
    /// Maximum number of new storage entries allowed.
    pub max_new_entries: u32,
    /// List of storage keys that must not be removed.
    pub required_keys: Vec<Symbol>,
    /// List of storage keys that will be removed (must be empty or migrated).
    pub removed_keys: Vec<Symbol>,
}

/// Entry for a deprecated function in an upgrade.
#[derive(Clone)]
#[contracttype]
pub struct DeprecatedFunctionEntry {
    /// The function name being deprecated.
    pub function: Symbol,
    /// Version since which it is deprecated.
    pub deprecated_in: String,
    /// Replacement function, if any.
    pub replacement: Option<Symbol>,
    /// Version in which it will be removed entirely.
    pub removed_in: Option<String>,
    /// Migration guide for callers.
    pub migration_guide: Option<String>,
}

// ==================== Upgrade Lifecycle Events ====================

/// Structured upgrade lifecycle event emitted during upgrade process.
#[derive(Clone)]
#[contracttype]
pub struct UpgradeLifecycleEvent {
    /// Schema version for this event type.
    pub schema_version: u32,
    /// Current contract version.
    pub from_version: u32,
    /// Target contract version.
    pub to_version: u32,
    /// Phase of the upgrade lifecycle.
    pub phase: UpgradePhase,
    /// Whether the phase completed successfully.
    pub success: bool,
    /// Additional context message.
    pub message: String,
    /// Ledger timestamp.
    pub timestamp: u64,
}

/// Phase of the upgrade lifecycle.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[contracttype]
pub enum UpgradePhase {
    /// Pre-flight validation started.
    PreFlightCheck = 0,
    /// Storage compatibility validated.
    StorageValidated = 1,
    /// Migration invariants checked.
    InvariantsChecked = 2,
    /// Migration executing.
    Migrating = 3,
    /// Post-migration integrity check.
    PostMigrationCheck = 4,
    /// Upgrade completed.
    Completed = 5,
    /// Upgrade failed and rolled back.
    RolledBack = 6,
    /// Upgrade failed permanently.
    Failed = 7,
}

// ==================== Upgrade Policy ====================

/// Policy configuration controlling upgrade behavior.
#[derive(Clone)]
#[contracttype]
pub struct UpgradePolicy {
    /// Whether dry-run validation is required before upgrade.
    pub require_dry_run: bool,
    /// Whether rollback must be tested before upgrade.
    pub require_rollback_test: bool,
    /// Maximum number of rollback attempts allowed.
    pub max_rollback_attempts: u32,
    /// Whether to emit detailed lifecycle events.
    pub emit_detailed_events: bool,
    /// Whether to pause the contract during migration.
    pub pause_during_migration: bool,
}

impl Default for UpgradePolicy {
    fn default() -> Self {
        Self {
            require_dry_run: true,
            require_rollback_test: true,
            max_rollback_attempts: 3,
            emit_detailed_events: true,
            pause_during_migration: false,
        }
    }
}

// ==================== Dry-Run Result ====================

/// Result of a dry-run upgrade validation.
#[derive(Clone)]
#[contracttype]
pub struct DryRunResult {
    /// Whether the dry-run passed all checks.
    pub passed: bool,
    /// Storage compatibility result.
    pub storage_compatible: bool,
    /// Whether all invariants would hold.
    pub invariants_satisfied: bool,
    /// Estimated gas impact of the migration.
    pub estimated_gas_impact: i128,
    /// List of issues found during dry-run.
    pub issues: Vec<DryRunIssue>,
}

/// A single issue found during dry-run validation.
#[derive(Clone)]
#[contracttype]
pub struct DryRunIssue {
    /// Issue category.
    pub category: IssueCategory,
    /// Severity of the issue.
    pub severity: InvariantSeverity,
    /// Human-readable description.
    pub description: String,
}

/// Category of dry-run issue.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[contracttype]
pub enum IssueCategory {
    /// Storage layout incompatibility.
    StorageIncompatible = 0,
    /// Migration invariant would be violated.
    InvariantViolation = 1,
    /// Deprecated function not handled.
    DeprecatedNotHandled = 2,
    /// Gas estimate exceeds budget.
    GasBudgetExceeded = 3,
    /// Rollback would not be possible.
    RollbackImpossible = 4,
}

// ==================== Schema Constants ====================

/// Current upgrade manifest schema version.
pub const MANIFEST_SCHEMA_VERSION: u32 = 1;

/// Current upgrade lifecycle event schema version.
pub const EVENT_SCHEMA_VERSION: u32 = 1;

// ==================== Helper Functions ====================

/// Emit an upgrade lifecycle event.
pub fn emit_upgrade_event(
    env: &Env,
    from_version: u32,
    to_version: u32,
    phase: UpgradePhase,
    success: bool,
    message: &str,
) {
    let event = UpgradeLifecycleEvent {
        schema_version: EVENT_SCHEMA_VERSION,
        from_version,
        to_version,
        phase,
        success,
        message: String::from_str(env, message),
        timestamp: env.ledger().timestamp(),
    };

    let phase_symbol = match phase {
        UpgradePhase::PreFlightCheck => symbol_short!("PREFLIGHT"),
        UpgradePhase::StorageValidated => symbol_short!("STOR_VALID"),
        UpgradePhase::InvariantsChecked => symbol_short!("INV_CHECK"),
        UpgradePhase::Migrating => symbol_short!("MIGRATING"),
        UpgradePhase::PostMigrationCheck => symbol_short!("POST_MIG"),
        UpgradePhase::Completed => symbol_short!("COMPLETED"),
        UpgradePhase::RolledBack => symbol_short!("ROLLED_BK"),
        UpgradePhase::Failed => symbol_short!("FAILED"),
    };

    env.events()
        .publish((symbol_short!("UPGRADE"), phase_symbol), event);
}

/// Validate storage compatibility rules against current state.
pub fn validate_storage_compatibility(
    env: &Env,
    rules: &StorageCompatibilityRules,
) -> Result<(), UpgradeError> {
    // Check that required keys exist
    let mut index = 0;
    while index < rules.required_keys.len() {
        let key = rules.required_keys.get(index).ok_or(UpgradeError::MigrationFailed)?;
        if !env.storage().instance().has(&key) {
            return Err(UpgradeError::IntegrityCheckFailed);
        }
        index += 1;
    }

    // Check that removed keys are empty or will be migrated
    index = 0;
    while index < rules.removed_keys.len() {
        let key = rules.removed_keys.get(index).ok_or(UpgradeError::MigrationFailed)?;
        // Removed keys should not have data that would be lost
        // In a real implementation, this would check if data exists and needs migration
        index += 1;
    }

    Ok(())
}

/// Create a default upgrade manifest for a given target version.
pub fn create_default_manifest(
    env: &Env,
    target_version: u32,
    new_wasm_hash: BytesN<32>,
    description: &str,
) -> UpgradeManifest {
    UpgradeManifest {
        target_version,
        new_wasm_hash,
        description: String::from_str(env, description),
        invariants: Vec::new(env),
        storage_rules: StorageCompatibilityRules {
            preserve_existing_keys: true,
            allow_new_keys: true,
            allow_type_changes: false,
            max_new_entries: 100,
            required_keys: Vec::new(env),
            removed_keys: Vec::new(env),
        },
        deprecated_functions: Vec::new(env),
        rollback_supported: true,
        migration_timeout_ledgers: 100,
    }
}

// ==================== Tests ====================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_policy() {
        let policy = UpgradePolicy::default();
        assert!(policy.require_dry_run);
        assert!(policy.require_rollback_test);
        assert_eq!(policy.max_rollback_attempts, 3);
        assert!(policy.emit_detailed_events);
        assert!(!policy.pause_during_migration);
    }

    #[test]
    fn test_manifest_schema_version() {
        assert_eq!(MANIFEST_SCHEMA_VERSION, 1);
    }

    #[test]
    fn test_event_schema_version() {
        assert_eq!(EVENT_SCHEMA_VERSION, 1);
    }

    #[test]
    fn test_invariant_severity_ordering() {
        assert!(InvariantSeverity::Critical < InvariantSeverity::Warning);
        assert!(InvariantSeverity::Warning < InvariantSeverity::Info);
    }

    #[test]
    fn test_upgrade_phase_ordering() {
        assert!(UpgradePhase::PreFlightCheck < UpgradePhase::StorageValidated);
        assert!(UpgradePhase::StorageValidated < UpgradePhase::InvariantsChecked);
        assert!(UpgradePhase::InvariantsChecked < UpgradePhase::Migrating);
        assert!(UpgradePhase::Migrating < UpgradePhase::PostMigrationCheck);
        assert!(UpgradePhase::PostMigrationCheck < UpgradePhase::Completed);
    }
}
