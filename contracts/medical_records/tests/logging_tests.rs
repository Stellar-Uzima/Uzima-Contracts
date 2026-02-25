#![allow(clippy::unwrap_used)]

use medical_records::{MedicalRecordsContract, MedicalRecordsContractClient, Role, StructuredLog};
use soroban_sdk::testutils::{Address as _, Events};
use soroban_sdk::{symbol_short, Address, Env, String, Symbol, TryFromVal, Vec};

fn setup(env: &Env) -> (MedicalRecordsContractClient<'_>, Address, Address, Address) {
    let contract_id = Address::generate(env);
    env.register_contract(&contract_id, MedicalRecordsContract);
    let client = MedicalRecordsContractClient::new(env, &contract_id);

    let admin = Address::generate(env);
    let doctor = Address::generate(env);
    let patient = Address::generate(env);

    client.initialize(&admin);
    client.manage_user(&admin, &doctor, &Role::Doctor);
    client.manage_user(&admin, &patient, &Role::Patient);

    (client, admin, doctor, patient)
}

fn find_structured_log(env: &Env, level_topic: Symbol, operation: &str) -> Option<StructuredLog> {
    let operation_value = String::from_str(env, operation);
    let events = env.events().all();

    for event in events.iter() {
        if event.1.len() < 2 {
            continue;
        }
        let level_val = event.1.get(1).unwrap();
        let event_level = match Symbol::try_from_val(env, &level_val) {
            Ok(level) => level,
            Err(_) => continue,
        };
        if event_level != level_topic {
            continue;
        }

        let log = match StructuredLog::try_from_val(env, &event.2) {
            Ok(entry) => entry,
            Err(_) => continue,
        };
        if log.operation == operation_value {
            return Some(log);
        }
    }

    None
}

#[test]
fn test_logging_user_management_info() {
    let env = Env::default();
    env.mock_all_auths();

    let (_client, admin, doctor, _patient) = setup(&env);

    let log = find_structured_log(&env, symbol_short!("LOG_INFO"), "manage_user")
        .expect("expected info log for manage_user operation");

    assert_eq!(log.actor, Some(admin));
    assert_eq!(log.target_id, Some(doctor));
    assert_eq!(log.record_id, None);
}

#[test]
fn test_logging_record_operations_info() {
    let env = Env::default();
    env.mock_all_auths();

    let (client, _admin, doctor, patient) = setup(&env);
    let record_id = client.add_record(
        &doctor,
        &patient,
        &String::from_str(&env, "Diagnosis"),
        &String::from_str(&env, "Treatment"),
        &false,
        &Vec::new(&env),
        &String::from_str(&env, "Modern"),
        &String::from_str(&env, "Medication"),
        &String::from_str(&env, "QmYyQSo1c1Ym7orWxLYvCrM2EmxFTANf8wXmmE7DWjhXXXXXx"),
    );

    let create_log = find_structured_log(&env, symbol_short!("LOG_INFO"), "add_record")
        .expect("expected info log for add_record operation");
    assert_eq!(create_log.record_id, Some(record_id));
    assert_eq!(create_log.actor, Some(doctor.clone()));
    assert_eq!(create_log.target_id, Some(patient.clone()));

    let _record = client.get_record(&patient, &record_id);
    let access_log = find_structured_log(&env, symbol_short!("LOG_INFO"), "get_record")
        .expect("expected info log for get_record operation");
    assert_eq!(access_log.record_id, Some(record_id));
}

#[test]
fn test_logging_warning_and_error_levels() {
    let env = Env::default();
    env.mock_all_auths();

    let (client, admin, _doctor, patient) = setup(&env);
    let missing_user = Address::generate(&env);

    let deactivated = client.deactivate_user(&admin, &missing_user);
    assert!(!deactivated);

    let warning_log = find_structured_log(&env, symbol_short!("LOG_WARN"), "deactivate_user")
        .expect("expected warning log for deactivate_user operation");
    assert_eq!(warning_log.target_id, Some(missing_user));

    let result = client.try_add_record(
        &patient,
        &patient,
        &String::from_str(&env, "Unauthorized Diagnosis"),
        &String::from_str(&env, "Unauthorized Treatment"),
        &false,
        &Vec::new(&env),
        &String::from_str(&env, "Modern"),
        &String::from_str(&env, "Medication"),
        &String::from_str(&env, "QmYyQSo1c1Ym7orWxLYvCrM2EmxFTANf8wXmmE7DWjhYYYYY"),
    );
    assert!(result.is_err());

    let error_log = find_structured_log(&env, symbol_short!("LOG_ERROR"), "add_record")
        .expect("expected error log for add_record operation");
    assert_eq!(error_log.actor, Some(patient.clone()));
    assert_eq!(error_log.target_id, Some(patient));
}

#[test]
fn test_logging_admin_actions_info() {
    let env = Env::default();
    env.mock_all_auths();

    let (client, admin, _doctor, _patient) = setup(&env);

    assert!(client.pause(&admin));
    assert!(client.unpause(&admin));

    let pause_log = find_structured_log(&env, symbol_short!("LOG_INFO"), "pause")
        .expect("expected info log for pause operation");
    assert_eq!(pause_log.actor, Some(admin.clone()));

    let unpause_log = find_structured_log(&env, symbol_short!("LOG_INFO"), "unpause")
        .expect("expected info log for unpause operation");
    assert_eq!(unpause_log.actor, Some(admin));
}
