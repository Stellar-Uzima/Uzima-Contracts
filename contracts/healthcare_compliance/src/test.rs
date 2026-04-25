extern crate std;

use crate::{DataType, HealthcareComplianceContractClient};
use soroban_sdk::{testutils::Ledger, Address, BytesN, Env, String};

fn setup_contract(env: &Env) -> (HealthcareComplianceContractClient<'_>, Address) {
    env.mock_all_auths();
    let contract_id = env.register_contract(None, crate::HealthcareComplianceContract);
    let client = HealthcareComplianceContractClient::new(env, &contract_id);

    let admin = Address::generate(env);
    client.initialize(&admin);
    (client, admin)
}

#[test]
fn test_submit_and_get_compliance_report() {
    let env = Env::default();
    let (client, _admin) = setup_contract(&env);

    let reporter = Address::generate(&env);
    let report_id = String::from_str(&env, "report-1");
    let report_hash = BytesN::from_array(&env, &[1u8; 32]);
    let uri = String::from_str(&env, "ipfs://report-1");

    let r = client.submit_compliance_report(&reporter, &report_id, &report_hash, &uri);
    assert_eq!(r, ());

    let rec = client.get_compliance_report(&report_id).expect("report should exist");
    assert_eq!(rec.report_id, report_id);
    assert_eq!(rec.reporter, reporter);
    assert_eq!(rec.report_hash, report_hash);
    assert_eq!(rec.uri, uri);
}

#[test]
fn test_default_retention_policies_exist() {
    let env = Env::default();
    let (client, _admin) = setup_contract(&env);

    let med = client.get_retention_policy(&DataType::MedicalRecords).expect("policy");
    let audit = client.get_retention_policy(&DataType::AuditLogs).expect("policy");
    let temp = client
        .get_retention_policy(&DataType::TemporaryData)
        .expect("policy");
    let pref = client
        .get_retention_policy(&DataType::UserPreferences)
        .expect("policy");

    assert_eq!(med.auto_delete, false);
    assert_eq!(med.retention_period, 0);
    assert_eq!(audit.auto_delete, true);
    assert_eq!(audit.retention_period, 6 * 365 * 24 * 60 * 60);
    assert_eq!(temp.auto_delete, true);
    assert_eq!(temp.retention_period, 90 * 24 * 60 * 60);
    assert_eq!(pref.auto_delete, false);
}

#[test]
fn test_enforce_retention_deletes_expired_temporary_data() {
    let env = Env::default();
    let (client, _admin) = setup_contract(&env);
    let actor = Address::generate(&env);
    let owner = Address::generate(&env);
    let record_id = String::from_str(&env, "tmp-1");

    env.ledger().with_mut(|li| li.timestamp = 1);
    client.register_retention_record(
        &actor,
        &record_id,
        &DataType::TemporaryData,
        &owner,
    );

    env.ledger()
        .with_mut(|li| li.timestamp = 90 * 24 * 60 * 60 + 2);
    let deleted = client.enforce_retention().expect("enforce");
    assert_eq!(deleted, 1);

    let audit = client.get_deletion_audit();
    assert_eq!(audit.len(), 1);
    let entry = audit.get(0).expect("entry");
    assert_eq!(entry.record_id, record_id);
}

#[test]
fn test_request_data_deletion_for_user_preferences() {
    let env = Env::default();
    let (client, _admin) = setup_contract(&env);
    let actor = Address::generate(&env);
    let pref_id = String::from_str(&env, "pref-1");

    client.register_retention_record(
        &actor,
        &pref_id,
        &DataType::UserPreferences,
        &actor,
    );
    client
        .request_data_deletion(&actor, &pref_id)
        .expect("delete request should succeed");

    let audit = client.get_deletion_audit();
    assert_eq!(audit.len(), 1);
    let entry = audit.get(0).expect("entry");
    assert_eq!(entry.record_id, pref_id);
    assert_eq!(entry.reason, String::from_str(&env, "user_deletion_request"));
}
