//! Upgrade Testing Framework (Issue #397)
//!
//! Tests state preservation, backward compatibility, migration validation,
//! and rollback capability for contract upgrades.

#![cfg(test)]

use soroban_sdk::{contract, contractimpl, testutils::Address as _, Address, Bytes, BytesN, Env, Vec};
use upgradeability::{
    migration::{Migratable, UpgradeValidation},
    storage, UpgradeError, UpgradeHistory,
};

// ---------------------------------------------------------------------------
// Minimal dummy contract for env.as_contract() context
// ---------------------------------------------------------------------------

#[contract]
struct DummyContract;
#[contractimpl]
impl DummyContract {}

// ---------------------------------------------------------------------------
// Minimal test implementation of Migratable
// ---------------------------------------------------------------------------

struct TestContract;

impl Migratable for TestContract {
    fn migrate(env: &Env, from_version: u32) -> Result<(), UpgradeError> {
        match from_version {
            1 => {
                let key = soroban_sdk::symbol_short!("CTR");
                let ctr: u32 = env.storage().instance().get(&key).unwrap_or(0);
                env.storage().instance().set(&key, &(ctr + 1));
                Ok(())
            }
            _ => Ok(()),
        }
    }

    fn verify_integrity(env: &Env) -> Result<BytesN<32>, UpgradeError> {
        let version = storage::get_version(env);
        let mut bytes = [0u8; 32];
        bytes[0] = (version & 0xFF) as u8;
        bytes[1] = ((version >> 8) & 0xFF) as u8;
        Ok(BytesN::from_array(env, &bytes))
    }

    fn validate(env: &Env, new_wasm_hash: &BytesN<32>) -> Result<UpgradeValidation, UpgradeError> {
        let zero = BytesN::from_array(env, &[0u8; 32]);
        if *new_wasm_hash == zero {
            return Err(UpgradeError::InvalidWasmHash);
        }
        Ok(UpgradeValidation {
            state_compatible: true,
            api_compatible: true,
            storage_layout_valid: true,
            tests_passed: true,
            gas_impact: 0,
            report: Vec::new(env),
        })
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn setup_env() -> (Env, Address, Address) {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, DummyContract);
    let admin = Address::generate(&env);
    env.as_contract(&contract_id, || {
        storage::set_admin(&env, &admin);
        storage::set_version(&env, 0);
    });
    (env, admin, contract_id)
}

fn make_wasm_hash(env: &Env, _seed: u8) -> BytesN<32> {
    // Upload a minimal valid WASM module so update_current_contract_wasm succeeds.
    // The 8-byte magic+version header is the smallest valid WASM binary.
    let minimal_wasm: &[u8] = &[0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00];
    env.deployer()
        .upload_contract_wasm(soroban_sdk::Bytes::from_slice(env, minimal_wasm))
}

// ---------------------------------------------------------------------------
// Test 1: State preservation during upgrade
// ---------------------------------------------------------------------------

#[test]
fn test_state_preserved_after_upgrade() {
    let (env, _admin, id) = setup_env();

    env.as_contract(&id, || {
        let key = soroban_sdk::symbol_short!("DATA");
        env.storage().instance().set(&key, &42u32);

        upgradeability::execute_upgrade::<TestContract>(
            &env,
            make_wasm_hash(&env, 1),
            1,
            soroban_sdk::symbol_short!("v1"),
        )
        .expect("Upgrade v0→v1 should succeed");

        let stored: u32 = env.storage().instance().get(&key).unwrap_or(0);
        assert_eq!(stored, 42, "State must be preserved across upgrade");
        assert_eq!(storage::get_version(&env), 1);
    });
}

// ---------------------------------------------------------------------------
// Test 2: Backward compatibility – same/lower version rejected
// ---------------------------------------------------------------------------

#[test]
fn test_backward_compatibility_version_check() {
    let (env, _admin, id) = setup_env();

    env.as_contract(&id, || {
        upgradeability::execute_upgrade::<TestContract>(
            &env,
            make_wasm_hash(&env, 1),
            1,
            soroban_sdk::symbol_short!("v1"),
        )
        .expect("v0→v1 should succeed");

        let err = upgradeability::execute_upgrade::<TestContract>(
            &env,
            make_wasm_hash(&env, 1),
            1,
            soroban_sdk::symbol_short!("v1"),
        )
        .expect_err("Re-applying same version must fail");
        assert_eq!(err, UpgradeError::IncompatibleVersion);

        let err = upgradeability::execute_upgrade::<TestContract>(
            &env,
            make_wasm_hash(&env, 0),
            0,
            soroban_sdk::symbol_short!("v0"),
        )
        .expect_err("Downgrade must fail");
        assert_eq!(err, UpgradeError::IncompatibleVersion);
    });
}

// ---------------------------------------------------------------------------
// Test 3: Migration script validation
// ---------------------------------------------------------------------------

#[test]
fn test_migration_script_validation() {
    let (env, _admin, id) = setup_env();

    env.as_contract(&id, || {
        upgradeability::execute_upgrade::<TestContract>(
            &env,
            make_wasm_hash(&env, 1),
            1,
            soroban_sdk::symbol_short!("v1"),
        )
        .expect("v0→v1");

        upgradeability::execute_upgrade::<TestContract>(
            &env,
            make_wasm_hash(&env, 2),
            2,
            soroban_sdk::symbol_short!("v2"),
        )
        .expect("v1→v2");

        let ctr: u32 = env
            .storage()
            .instance()
            .get(&soroban_sdk::symbol_short!("CTR"))
            .unwrap_or(0);
        assert_eq!(ctr, 1, "Migration must have incremented CTR");
        assert_eq!(storage::get_version(&env), 2);
    });
}

// ---------------------------------------------------------------------------
// Test 4: Rollback capability
// ---------------------------------------------------------------------------

#[test]
fn test_rollback_to_previous_version() {
    let (env, _admin, id) = setup_env();

    env.as_contract(&id, || {
        upgradeability::execute_upgrade::<TestContract>(
            &env,
            make_wasm_hash(&env, 1),
            1,
            soroban_sdk::symbol_short!("v1"),
        )
        .expect("v0→v1");

        upgradeability::execute_upgrade::<TestContract>(
            &env,
            make_wasm_hash(&env, 2),
            2,
            soroban_sdk::symbol_short!("v2"),
        )
        .expect("v1→v2");

        assert!(
            upgradeability::rollback(&env).is_ok(),
            "Rollback must succeed when history exists"
        );
    });
}

// ---------------------------------------------------------------------------
// Test 5: Rollback fails without history
// ---------------------------------------------------------------------------

#[test]
fn test_rollback_fails_without_history() {
    let (env, _admin, id) = setup_env();

    env.as_contract(&id, || {
        let err = upgradeability::rollback(&env).expect_err("Rollback without history must fail");
        assert_eq!(err, UpgradeError::HistoryNotFound);
    });
}

// ---------------------------------------------------------------------------
// Test 6: Frozen contract cannot be upgraded
// ---------------------------------------------------------------------------

#[test]
fn test_frozen_contract_cannot_upgrade() {
    let (env, _admin, id) = setup_env();

    env.as_contract(&id, || {
        storage::freeze(&env);

        let err = upgradeability::execute_upgrade::<TestContract>(
            &env,
            make_wasm_hash(&env, 1),
            1,
            soroban_sdk::symbol_short!("v1"),
        )
        .expect_err("Frozen contract must not be upgradeable");
        assert_eq!(err, UpgradeError::ContractPaused);
    });
}

// ---------------------------------------------------------------------------
// Test 7: Validate upgrade rejects zero WASM hash
// ---------------------------------------------------------------------------

#[test]
fn test_validate_upgrade_rejects_zero_hash() {
    let (env, _admin, id) = setup_env();

    env.as_contract(&id, || {
        let zero_hash = BytesN::from_array(&env, &[0u8; 32]);
        let err = upgradeability::validate_upgrade::<TestContract>(&env, zero_hash)
            .expect_err("Zero WASM hash must be rejected");
        assert_eq!(err, UpgradeError::InvalidWasmHash);
    });
}

// ---------------------------------------------------------------------------
// Test 8: Upgrade history is recorded correctly
// ---------------------------------------------------------------------------

#[test]
fn test_upgrade_history_recorded() {
    let (env, _admin, id) = setup_env();

    env.as_contract(&id, || {
        upgradeability::execute_upgrade::<TestContract>(
            &env,
            make_wasm_hash(&env, 1),
            1,
            soroban_sdk::symbol_short!("v1"),
        )
        .expect("v0→v1");

        upgradeability::execute_upgrade::<TestContract>(
            &env,
            make_wasm_hash(&env, 2),
            2,
            soroban_sdk::symbol_short!("v2"),
        )
        .expect("v1→v2");

        let history = storage::get_history(&env);
        assert_eq!(history.len(), 2, "History must contain 2 entries");

        let first: UpgradeHistory = history.get(0).unwrap();
        assert_eq!(first.version, 1);

        let second: UpgradeHistory = history.get(1).unwrap();
        assert_eq!(second.version, 2);
    });
}

// ---------------------------------------------------------------------------
// Test 9: Full upgrade lifecycle (CI integration)
// ---------------------------------------------------------------------------

#[test]
fn test_full_upgrade_lifecycle() {
    let (env, _admin, id) = setup_env();

    env.as_contract(&id, || {
        let wasm_v1 = make_wasm_hash(&env, 1);
        let validation = upgradeability::validate_upgrade::<TestContract>(&env, wasm_v1.clone())
            .expect("Validation must pass");
        assert!(validation.state_compatible);
        assert!(validation.api_compatible);

        upgradeability::execute_upgrade::<TestContract>(
            &env,
            wasm_v1,
            1,
            soroban_sdk::symbol_short!("v1"),
        )
        .expect("v0→v1");

        env.storage()
            .instance()
            .set(&soroban_sdk::symbol_short!("REC"), &100u32);

        upgradeability::execute_upgrade::<TestContract>(
            &env,
            make_wasm_hash(&env, 2),
            2,
            soroban_sdk::symbol_short!("v2"),
        )
        .expect("v1→v2");

        let rec: u32 = env
            .storage()
            .instance()
            .get(&soroban_sdk::symbol_short!("REC"))
            .unwrap_or(0);
        assert_eq!(rec, 100, "Records must survive upgrade");
        assert_eq!(storage::get_version(&env), 2);

        assert!(upgradeability::rollback(&env).is_ok());
    });
}
