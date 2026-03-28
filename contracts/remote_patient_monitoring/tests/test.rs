#![cfg(test)]

use remote_patient_monitoring::{
    RemotePatientMonitoringContract, RemotePatientMonitoringContractClient,
};
use soroban_sdk::testutils::Address as _;
use soroban_sdk::{Address, Env};

#[test]
fn test_register_device() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, RemotePatientMonitoringContract);
    let client = RemotePatientMonitoringContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let patient = Address::generate(&env);

    client.initialize(&admin);

    client.register_device(
        &admin,
        &1,
        &soroban_sdk::String::from_str(&env, "BloodPressureMonitor"),
        &patient,
    );

    // Verify device is registered
    // Note: Need to add getter if not present
}

#[test]
fn test_submit_vital_sign() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, RemotePatientMonitoringContract);
    let client = RemotePatientMonitoringContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let patient = Address::generate(&env);

    client.initialize(&admin);
    client.register_device(
        &admin,
        &1,
        &soroban_sdk::String::from_str(&env, "HeartRateMonitor"),
        &patient,
    );

    client.submit_vital_sign(
        &admin,
        &patient,
        &1,
        &soroban_sdk::String::from_str(&env, "heart_rate"),
        &80,
        &soroban_sdk::String::from_str(&env, "bpm"),
    );

    // Check if alert is triggered if threshold set
}
