//! Integration tests for the upgrade safety framework.

use soroban_sdk::{testutils::Address as _, BytesN, Env, String, Symbol};

use crate::upgrade_safety::{
    create_default_manifest, emit_upgrade_event, validate_storage_compatibility,
    InvariantSeverity, MigrationInvariant, StorageCompatibilityRules, UpgradeManifest,
    UpgradePhase, UpgradePolicy,
};

use crate::{storage, UpgradeError};

#[test]
fn test_create_default_manifest() {
    let env = Env::default();
    let wasm_hash = BytesN::from_array(&env, &[1u8; 32]);
    let manifest = create_default_manifest(&env, 2, wasm_hash.clone(), "Test upgrade");

    assert_eq!(manifest.target_version, 2);
    assert_eq!(manifest.new_wasm_hash, wasm_hash);
    assert!(manifest.rollback_supported);
    assert_eq!(manifest.migration_timeout_ledgers, 100);
    assert!(manifest.storage_rules.preserve_existing_keys);
    assert!(manifest.storage_rules.allow_new_keys);
}

#[test]
fn test_default_upgrade_policy() {
    let policy = UpgradePolicy::default();
    assert!(policy.require_dry_run);
    assert!(policy.require_rollback_test);
    assert_eq!(policy.max_rollback_attempts, 3);
    assert!(policy.emit_detailed_events);
    assert!(!policy.pause_during_migration);
}

#[test]
fn test_validate_storage_compatibility_pass() {
    let env = Env::default();
    let rules = StorageCompatibilityRules {
        preserve_existing_keys: true,
        allow_new_keys: true,
        allow_type_changes: false,
        max_new_entries: 100,
        required_keys: soroban_sdk::Vec::new(&env),
        removed_keys: soroban_sdk::Vec::new(&env),
    };

    assert!(validate_storage_compatibility(&env, &rules).is_ok());
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

#[test]
fn test_emit_upgrade_event() {
    let env = Env::default();
    // Should not panic
    emit_upgrade_event(
        &env,
        1,
        2,
        UpgradePhase::Completed,
        true,
        "Test completed",
    );
}

#[test]
fn test_manifest_schema_version() {
    use crate::upgrade_safety::MANIFEST_SCHEMA_VERSION;
    assert_eq!(MANIFEST_SCHEMA_VERSION, 1);
}

#[test]
fn test_event_schema_version() {
    use crate::upgrade_safety::EVENT_SCHEMA_VERSION;
    assert_eq!(EVENT_SCHEMA_VERSION, 1);
}

#[test]
fn test_migration_invariant_creation() {
    let env = Env::default();
    let invariant = MigrationInvariant {
        name: Symbol::new(&env, "test_invariant"),
        description: String::from_str(&env, "Test invariant description"),
        severity: InvariantSeverity::Critical,
    };

    assert_eq!(invariant.name, Symbol::new(&env, "test_invariant"));
    assert_eq!(invariant.severity, InvariantSeverity::Critical);
}
