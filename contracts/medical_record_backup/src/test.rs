#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]

use super::*;
use soroban_sdk::testutils::{Address as _, Ledger};
use soroban_sdk::{Address, BytesN, Env, String};

fn setup(env: &Env) -> (MedicalRecordBackupContractClient<'_>, Address) {
    let contract_id = Address::generate(env);
    env.register_contract(&contract_id, MedicalRecordBackupContract);
    let client = MedicalRecordBackupContractClient::new(env, &contract_id);
    let admin = Address::generate(env);
    client.initialize(&admin);
    (client, admin)
}

fn sample_hash(env: &Env, v: u8) -> BytesN<32> {
    BytesN::from_array(env, &[v; 32])
}

fn register_two_targets(
    client: &MedicalRecordBackupContractClient<'_>,
    admin: &Address,
    env: &Env,
) {
    client.register_target(
        admin,
        &BackupNetwork::Ipfs,
        &GeoRegion::UsEast,
        &sample_hash(env, 1),
        &true,
        &10,
        &1000,
    );
    client.register_target(
        admin,
        &BackupNetwork::Arweave,
        &GeoRegion::EuCentral,
        &sample_hash(env, 2),
        &true,
        &15,
        &1000,
    );
}

#[test]
fn backup_run_creates_geo_redundant_artifact() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin) = setup(&env);
    register_two_targets(&client, &admin, &env);

    let id = client.run_backup_now(
        &admin,
        &sample_hash(&env, 7),
        &String::from_str(&env, "ipfs://snapshot-a"),
        &1,
    );
    let artifact = client.get_artifact(&id).unwrap();
    assert_eq!(artifact.target_ids.len(), 2);
    assert_eq!(artifact.region_count, 2);
    assert!(artifact.encrypted);
}

#[test]
fn scheduled_backup_respects_interval() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin) = setup(&env);
    register_two_targets(&client, &admin, &env);

    let policy = BackupPolicy {
        interval_seconds: 1_000,
        retention_seconds: 10_000,
        max_active_backups: 10,
        min_targets_per_backup: 2,
        min_region_count: 2,
        max_total_cost_weight: 1_000,
        verify_on_write: true,
        encryption_required: true,
        auto_cleanup: false,
        min_restore_approvals: 1,
    };
    client.set_policy(&admin, &policy);

    client.run_scheduled_backup(
        &admin,
        &sample_hash(&env, 9),
        &String::from_str(&env, "ipfs://snapshot-b"),
        &2,
    );

    let err = client.try_run_scheduled_backup(
        &admin,
        &sample_hash(&env, 10),
        &String::from_str(&env, "ipfs://snapshot-c"),
        &2,
    );
    assert_eq!(err, Err(Ok(Error::ScheduleNotDue)));
}

#[test]
fn integrity_mismatch_creates_alert() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin) = setup(&env);
    register_two_targets(&client, &admin, &env);

    let id = client.run_backup_now(
        &admin,
        &sample_hash(&env, 3),
        &String::from_str(&env, "ipfs://snapshot-d"),
        &1,
    );
    let ok = client.verify_backup_integrity(&admin, &id, &sample_hash(&env, 255));
    assert!(!ok);

    let alerts = client.list_alerts(&true);
    assert!(!alerts.is_empty());
}

#[test]
fn restore_workflow_executes_after_approval() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin) = setup(&env);
    register_two_targets(&client, &admin, &env);

    let id = client.run_backup_now(
        &admin,
        &sample_hash(&env, 4),
        &String::from_str(&env, "ipfs://snapshot-restore"),
        &1,
    );
    let request_id = client.request_restore(&admin, &id, &sample_hash(&env, 11));
    client.approve_restore(&admin, &request_id);
    let restored_ref = client.execute_restore(&admin, &request_id);
    assert_eq!(
        restored_ref,
        String::from_str(&env, "ipfs://snapshot-restore")
    );
}

#[test]
fn cleanup_archives_old_backups() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin) = setup(&env);
    register_two_targets(&client, &admin, &env);

    client.set_policy(
        &admin,
        &BackupPolicy {
            interval_seconds: 1,
            retention_seconds: 2,
            max_active_backups: 1,
            min_targets_per_backup: 2,
            min_region_count: 2,
            max_total_cost_weight: 1_000,
            verify_on_write: true,
            encryption_required: true,
            auto_cleanup: false,
            min_restore_approvals: 1,
        },
    );

    env.ledger().set_timestamp(1000);
    client.run_backup_now(
        &admin,
        &sample_hash(&env, 21),
        &String::from_str(&env, "ipfs://old"),
        &1,
    );
    env.ledger().set_timestamp(2000);
    client.run_backup_now(
        &admin,
        &sample_hash(&env, 22),
        &String::from_str(&env, "ipfs://new"),
        &1,
    );
    env.ledger().set_timestamp(3000);
    let report = client.optimize_and_cleanup(&admin);
    assert!(report.archived_backups >= 1);
}
