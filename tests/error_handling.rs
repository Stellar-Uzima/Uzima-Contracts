use medical_records::{
    Error, ErrorLogEntry, MedicalRecordsContract, MedicalRecordsContractClient, Role,
};
use soroban_sdk::{Address, Env, String, Vec};

fn setup_contract(env: &Env) -> (MedicalRecordsContractClient, Address, Address, Address) {
    env.mock_all_auths();
    let contract_id = env.register_contract(None, MedicalRecordsContract);
    let client = MedicalRecordsContractClient::new(env, &contract_id);

    let admin = Address::generate(env);
    let doctor = Address::generate(env);
    let patient = Address::generate(env);

    client.initialize(&admin);
    client.manage_user(&admin, &doctor, &Role::Doctor);
    client.manage_user(&admin, &patient, &Role::Patient);

    (client, admin, doctor, patient)
}

#[test]
fn test_error_info_exposes_code_and_message() {
    let env = Env::default();
    let (client, _admin, _doctor, _patient) = setup_contract(&env);

    let info = client.get_error_info(&Error::RecordNotFound);
    assert_eq!(info.code, Error::RecordNotFound as u32);
    assert!(info.message.len() > 0);
}

#[test]
fn test_error_log_for_missing_record() {
    let env = Env::default();
    let (client, _admin, _doctor, patient) = setup_contract(&env);

    let res = client.try_get_record(&patient, &999u64);
    assert_eq!(res, Err(Ok(Error::RecordNotFound)));

    let logs = client.get_error_logs(&0u32, &10u32);
    assert_eq!(logs.len(), 1);
    let entry: ErrorLogEntry = logs.get(0).unwrap();
    assert_eq!(entry.error, Error::RecordNotFound);
    assert_eq!(entry.code, Error::RecordNotFound as u32);
    assert_eq!(
        entry.context,
        String::from_str(&env, "get_record:not_found")
    );
}

#[test]
fn test_emergency_access_expiry_cleanup() {
    let env = Env::default();
    let (client, _admin, _doctor, patient) = setup_contract(&env);
    let grantee = Address::generate(&env);

    client.grant_emergency_access(&patient, &grantee, &1u64, &Vec::new(&env));
    let now = env.ledger().timestamp();
    env.ledger().with_mut(|l| l.timestamp = now + 2);

    assert!(!client.has_emergency_access(&grantee, &patient, &0u64));
    let active = client.get_patient_emergency_grants(&patient);
    assert_eq!(active.len(), 0);
}

#[test]
fn test_invalid_pagination_returns_error() {
    let env = Env::default();
    let (client, _admin, _doctor, patient) = setup_contract(&env);

    let result = client.try_get_history(&patient, &patient, &0u32, &0u32);
    assert_eq!(result, Err(Ok(Error::InvalidPagination)));
}
