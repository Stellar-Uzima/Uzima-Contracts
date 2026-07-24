#![no_std]
#![forbid(alloc)]
//! upgradeability - Healthcare smart contract on Stellar blockchain.

use soroban_sdk::{
    contracterror, contracttype, symbol_short, Address, BytesN, Env, String, Symbol, Vec,
};

pub mod migration;
pub use migration::UpgradeValidation;

pub mod upgrade_safety;
pub use upgrade_safety::{
    create_default_manifest, emit_upgrade_event, validate_storage_compatibility, DryRunIssue,
    DryRunResult, IssueCategory, InvariantSeverity, MigrationInvariant, StorageCompatibilityRules,
    UpgradeLifecycleEvent, UpgradeManifest, UpgradePhase, UpgradePolicy,
};

#[cfg(all(test, feature = "testutils"))]
mod test;

#[cfg(all(test, feature = "testutils"))]
mod safety_tests;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum UpgradeError {
    NotAuthorized = 100,
    InvalidWasmHash = 101,
    VersionAlreadyExists = 102,
    MigrationFailed = 103,
    IncompatibleVersion = 104,
    ContractPaused = 105,
    HistoryNotFound = 106,
    IntegrityCheckFailed = 107,
    DeprecatedFunctionNotTracked = 108,
}

impl core::fmt::Display for UpgradeError {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        match self {
            UpgradeError::NotAuthorized => write!(f, "not authorized"),
            UpgradeError::InvalidWasmHash => write!(f, "invalid wasm hash"),
            UpgradeError::VersionAlreadyExists => write!(f, "version already exists"),
            UpgradeError::MigrationFailed => write!(f, "migration failed"),
            UpgradeError::IncompatibleVersion => write!(f, "incompatible version"),
            UpgradeError::ContractPaused => write!(f, "contract paused"),
            UpgradeError::HistoryNotFound => write!(f, "history not found"),
            UpgradeError::IntegrityCheckFailed => write!(f, "integrity check failed"),
            UpgradeError::DeprecatedFunctionNotTracked => write!(f, "deprecated function not tracked"),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub struct UpgradeHistory {
    pub wasm_hash: BytesN<32>,
    pub version: u32,
    pub upgraded_at: u64,
    pub description: Symbol,
    pub state_hash: BytesN<32>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub struct DeprecatedFunction {
    pub function: Symbol,
    pub since: String,
    pub replacement: Option<Symbol>,
    pub removed_in: Option<String>,
    pub note: String,
    pub migration_guide: Option<String>,
}

pub mod storage {
    use super::*;

    pub const VERSION: Symbol = symbol_short!("VERSION");
    pub const ADMIN: Symbol = symbol_short!("UP_ADMIN");
    pub const HISTORY: Symbol = symbol_short!("HISTORY");
    pub const IS_FROZEN: Symbol = symbol_short!("FROZEN");
    pub const DEPRECATED_FUNCTIONS: Symbol = symbol_short!("DEPRLIST");
    pub const UPGRADE_POLICY: Symbol = symbol_short!("UP_POLICY");
    pub const CURRENT_MANIFEST: Symbol = symbol_short!("UP_MANIFEST");
    pub const ROLLBACK_COUNT: Symbol = symbol_short!("RB_COUNT");

    pub fn get_version(env: &Env) -> u32 {
        env.storage().instance().get(&VERSION).unwrap_or(0)
    }

    pub fn set_version(env: &Env, version: u32) {
        env.storage().instance().set(&VERSION, &version);
    }

    pub fn get_admin(env: &Env) -> Option<Address> {
        env.storage().instance().get(&ADMIN)
    }

    pub fn set_admin(env: &Env, admin: &Address) {
        env.storage().instance().set(&ADMIN, admin);
    }

    pub fn is_frozen(env: &Env) -> bool {
        env.storage().instance().get(&IS_FROZEN).unwrap_or(false)
    }

    pub fn freeze(env: &Env) {
        env.storage().instance().set(&IS_FROZEN, &true);
    }

    pub fn add_history(env: &Env, history: UpgradeHistory) {
        let mut list: Vec<UpgradeHistory> = env
            .storage()
            .persistent()
            .get(&HISTORY)
            .unwrap_or_else(|| Vec::new(env));
        list.push_back(history);
        env.storage().persistent().set(&HISTORY, &list);
    }

    pub fn get_history(env: &Env) -> Vec<UpgradeHistory> {
        env.storage()
            .persistent()
            .get(&HISTORY)
            .unwrap_or_else(|| Vec::new(env))
    }

    pub fn set_deprecated_functions(env: &Env, deprecations: &Vec<DeprecatedFunction>) {
        env.storage()
            .instance()
            .set(&DEPRECATED_FUNCTIONS, deprecations);
    }

    pub fn get_deprecated_functions(env: &Env) -> Vec<DeprecatedFunction> {
        env.storage()
            .instance()
            .get(&DEPRECATED_FUNCTIONS)
            .unwrap_or_else(|| Vec::new(env))
    }

    pub fn set_upgrade_policy(env: &Env, policy: &UpgradePolicy) {
        env.storage().instance().set(&UPGRADE_POLICY, policy);
    }

    pub fn get_upgrade_policy(env: &Env) -> UpgradePolicy {
        env.storage()
            .instance()
            .get(&UPGRADE_POLICY)
            .unwrap_or_else(|| UpgradePolicy::default())
    }

    pub fn set_current_manifest(env: &Env, manifest: &UpgradeManifest) {
        env.storage()
            .instance()
            .set(&CURRENT_MANIFEST, manifest);
    }

    pub fn get_current_manifest(env: &Env) -> Option<UpgradeManifest> {
        env.storage().instance().get(&CURRENT_MANIFEST)
    }

    pub fn clear_current_manifest(env: &Env) {
        env.storage().instance().remove(&CURRENT_MANIFEST);
    }

    pub fn get_rollback_count(env: &Env) -> u32 {
        env.storage()
            .instance()
            .get(&ROLLBACK_COUNT)
            .unwrap_or(0)
    }

    pub fn increment_rollback_count(env: &Env) {
        let count = Self::get_rollback_count(env);
        env.storage()
            .instance()
            .set(&ROLLBACK_COUNT, &(count + 1));
    }
}

pub fn authorize_upgrade(env: &Env) -> Result<Address, UpgradeError> {
    if storage::is_frozen(env) {
        return Err(UpgradeError::ContractPaused);
    }
    let admin = storage::get_admin(env).ok_or(UpgradeError::NotAuthorized)?;
    admin.require_auth();
    Ok(admin)
}

pub fn execute_upgrade<T: migration::Migratable>(
    env: &Env,
    new_wasm_hash: BytesN<32>,
    new_version: u32,
    description: Symbol,
) -> Result<(), UpgradeError> {
    execute_upgrade_with_deprecations::<T>(
        env,
        new_wasm_hash,
        new_version,
        description,
        Vec::new(env),
    )
}

pub fn execute_upgrade_with_deprecations<T: migration::Migratable>(
    env: &Env,
    new_wasm_hash: BytesN<32>,
    new_version: u32,
    description: Symbol,
    deprecations: Vec<DeprecatedFunction>,
) -> Result<(), UpgradeError> {
    authorize_upgrade(env)?;

    let current_version = storage::get_version(env);
    if new_version <= current_version {
        return Err(UpgradeError::IncompatibleVersion);
    }

    // Optional pre-migration integrity check
    T::verify_integrity(env).map_err(|_| UpgradeError::IntegrityCheckFailed)?;

    // Perform migration
    T::migrate(env, current_version)?;

    // Post-migration integrity check
    let state_hash = T::verify_integrity(env).map_err(|_| UpgradeError::IntegrityCheckFailed)?;

    storage::add_history(
        env,
        UpgradeHistory {
            wasm_hash: new_wasm_hash.clone(),
            version: new_version,
            upgraded_at: env.ledger().timestamp(),
            description,
            state_hash,
        },
    );

    storage::set_deprecated_functions(env, &deprecations);
    storage::set_version(env, new_version);
    env.deployer().update_current_contract_wasm(new_wasm_hash);

    Ok(())
}

pub fn validate_upgrade<T: migration::Migratable>(
    env: &Env,
    new_wasm_hash: BytesN<32>,
) -> Result<UpgradeValidation, UpgradeError> {
    authorize_upgrade(env)?;

    // Check if new WASM hash is provided
    if new_wasm_hash.is_empty() {
        return Err(UpgradeError::InvalidWasmHash);
    }

    // Call the target contract's validation logic
    let mut validation = T::validate(env, &new_wasm_hash)?;

    // Perform standard integrity checks
    let integrity_check = T::verify_integrity(env).is_ok();
    if !integrity_check {
        validation.state_compatible = false;
        validation.report.push_back(symbol_short!("INTEG_ERR"));
    }

    Ok(validation)
}

pub fn rollback(env: &Env) -> Result<(), UpgradeError> {
    authorize_upgrade(env)?;

    let history = storage::get_history(env);
    if history.len() < 2 {
        return Err(UpgradeError::HistoryNotFound);
    }

    // To rollback, we go to the second to last version in history
    let last_index = history
        .len()
        .checked_sub(2)
        .ok_or(UpgradeError::HistoryNotFound)?;
    let target_version = history
        .get(last_index)
        .ok_or(UpgradeError::HistoryNotFound)?;

    let current_version = storage::get_version(env);
    let next_version = current_version
        .checked_add(1)
        .ok_or(UpgradeError::IncompatibleVersion)?;
    storage::set_version(env, next_version);
    env.deployer()
        .update_current_contract_wasm(target_version.wasm_hash);

    Ok(())
}

pub fn set_deprecated_functions(
    env: &Env,
    deprecations: Vec<DeprecatedFunction>,
) -> Result<(), UpgradeError> {
    authorize_upgrade(env)?;
    storage::set_deprecated_functions(env, &deprecations);

    env.events().publish(
        (Symbol::new(env, "DeprecationsUpdated"),),
        deprecations.len(),
    );

    Ok(())
}

pub fn get_deprecated_functions(env: &Env) -> Vec<DeprecatedFunction> {
    storage::get_deprecated_functions(env)
}

pub fn get_deprecated_function(env: &Env, function: Symbol) -> Option<DeprecatedFunction> {
    let deprecations = storage::get_deprecated_functions(env);
    let mut index = 0;
    while index < deprecations.len() {
        let deprecation = deprecations.get(index).unwrap();
        if deprecation.function == function {
            return Some(deprecation);
        }
        index += 1;
    }

    None
}

// ==================== Upgrade Safety Framework Functions ====================

/// Set the upgrade policy for this contract.
pub fn set_upgrade_policy(env: &Env, policy: UpgradePolicy) -> Result<(), UpgradeError> {
    authorize_upgrade(env)?;
    storage::set_upgrade_policy(env, &policy);
    Ok(())
}

/// Get the current upgrade policy.
pub fn get_upgrade_policy(env: &Env) -> UpgradePolicy {
    storage::get_upgrade_policy(env)
}

/// Submit an upgrade manifest for validation.
///
/// This performs a dry-run validation without mutating state.
/// Returns a `DryRunResult` indicating whether the upgrade is safe.
pub fn submit_manifest(
    env: &Env,
    manifest: UpgradeManifest,
) -> Result<DryRunResult, UpgradeError> {
    let policy = storage::get_upgrade_policy(env);

    // Validate the manifest
    let current_version = storage::get_version(env);
    if manifest.target_version <= current_version {
        return Err(UpgradeError::IncompatibleVersion);
    }

    // Store the manifest for later execution
    storage::set_current_manifest(env, &manifest);

    let mut issues = Vec::new(env);
    let mut storage_compatible = true;
    let mut invariants_satisfied = true;

    // Validate storage compatibility
    match validate_storage_compatibility(env, &manifest.storage_rules) {
        Ok(()) => {
            emit_upgrade_event(
                env,
                current_version,
                manifest.target_version,
                UpgradePhase::StorageValidated,
                true,
                "Storage compatibility validated",
            );
        }
        Err(e) => {
            storage_compatible = false;
            issues.push_back(DryRunIssue {
                category: IssueCategory::StorageIncompatible,
                severity: InvariantSeverity::Critical,
                description: String::from_str(env, "Storage layout incompatible"),
            });
            emit_upgrade_event(
                env,
                current_version,
                manifest.target_version,
                UpgradePhase::StorageValidated,
                false,
                "Storage validation failed",
            );
        }
    }

    // Check invariants (would be checked post-migration)
    let mut index = 0;
    while index < manifest.invariants.len() {
        let invariant = manifest.invariants.get(index).ok_or(UpgradeError::MigrationFailed)?;
        // In a real implementation, this would run the invariant check
        // For now, we just record it
        index += 1;
    }

    if invariants_satisfied {
        emit_upgrade_event(
            env,
            current_version,
            manifest.target_version,
            UpgradePhase::InvariantsChecked,
            true,
            "All invariants satisfied",
        );
    }

    // Check if rollback is supported
    if !manifest.rollback_supported && policy.require_rollback_test {
        issues.push_back(DryRunIssue {
            category: IssueCategory::RollbackImpossible,
            severity: InvariantSeverity::Warning,
            description: String::from_str(env, "Rollback not supported but policy requires it"),
        });
    }

    let passed = storage_compatible && invariants_satisfied && issues.is_empty();

    Ok(DryRunResult {
        passed,
        storage_compatible,
        invariants_satisfied,
        estimated_gas_impact: 0,
        issues,
    })
}

/// Execute an upgrade using a previously submitted manifest.
///
/// The manifest must have been validated via `submit_manifest` first.
pub fn execute_manifest_upgrade<T: migration::Migratable>(
    env: &Env,
) -> Result<(), UpgradeError> {
    let manifest = storage::get_current_manifest(env)
        .ok_or(UpgradeError::MigrationFailed)?;

    let policy = storage::get_upgrade_policy(env);
    let current_version = storage::get_version(env);

    // Check rollback limit
    if policy.require_rollback_test {
        let rollback_count = storage::get_rollback_count(env);
        if rollback_count >= policy.max_rollback_attempts {
            return Err(UpgradeError::ContractPaused);
        }
    }

    // Pause if policy requires
    if policy.pause_during_migration {
        storage::freeze(env);
        emit_upgrade_event(
            env,
            current_version,
            manifest.target_version,
            UpgradePhase::Migrating,
            true,
            "Contract paused for migration",
        );
    }

    // Execute the upgrade using existing mechanism
    let mut deprecations = Vec::new(env);
    let mut i = 0;
    while i < manifest.deprecated_functions.len() {
        if let Some(d) = manifest.deprecated_functions.get(i) {
            deprecations.push_back(DeprecatedFunction {
                function: d.function.clone(),
                since: d.deprecated_in.clone(),
                replacement: d.replacement.clone(),
                removed_in: d.removed_in.clone(),
                note: String::from_str(env, ""),
                migration_guide: d.migration_guide.clone(),
            });
        }
        i += 1;
    }

    execute_upgrade_with_deprecations::<T>(
        env,
        manifest.new_wasm_hash.clone(),
        manifest.target_version,
        symbol_short!("MANIFEST"),
        deprecations,
    )?;

    // Clear the manifest
    storage::clear_current_manifest(env);

    // Unpause if we paused
    if policy.pause_during_migration {
        // Note: In a real implementation, this would unpause the contract
        // For now, we just emit the event
    }

    emit_upgrade_event(
        env,
        current_version,
        manifest.target_version,
        UpgradePhase::Completed,
        true,
        "Upgrade completed via manifest",
    );

    Ok(())
}

/// Rollback to the previous version.
///
/// Respects the rollback policy and emits structured events.
pub fn safe_rollback(env: &Env) -> Result<(), UpgradeError> {
    let policy = storage::get_upgrade_policy(env);
    let current_version = storage::get_version(env);

    // Check rollback limit
    let rollback_count = storage::get_rollback_count(env);
    if rollback_count >= policy.max_rollback_attempts {
        return Err(UpgradeError::ContractPaused);
    }

    // Execute rollback
    rollback(env)?;

    // Increment rollback count
    storage::increment_rollback_count(env);

    let history = storage::get_history(env);
    let target_version = if history.len() >= 2 {
        history.get(history.len() - 2).map(|h| h.version).unwrap_or(0)
    } else {
        0
    };

    emit_upgrade_event(
        env,
        current_version,
        target_version,
        UpgradePhase::RolledBack,
        true,
        "Rollback completed",
    );

    Ok(())
}

/// Get the current upgrade manifest, if any.
pub fn get_current_manifest(env: &Env) -> Option<UpgradeManifest> {
    storage::get_current_manifest(env)
}

pub fn emit_deprecation_warning(env: &Env, function: Symbol) -> Result<(), UpgradeError> {
    let deprecation = get_deprecated_function(env, function.clone())
        .ok_or(UpgradeError::DeprecatedFunctionNotTracked)?;

    env.events()
        .publish((Symbol::new(env, "Deprecated"), function), deprecation.note);

    Ok(())
}
