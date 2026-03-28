#![cfg(test)]

use super::*;
use soroban_sdk::testutils::{Address as _, Env as _};
use soroban_sdk::{Address, Env};

#[test]
fn test_register_device() {
    let env = Env::default();
    let contract_id = env.register_contract(None, RemotePatientMonitoringContract);
    let client = RemotePatientMonitoringContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let patient = Address::generate(&env);

    client.initialize(&admin);

    client.register_device(&admin, &1, &"BloodPressureMonitor".into(), &patient);

    // Verify device is registered
    // Note: Need to add getter if not present
}

#[test]
fn test_submit_vital_sign() {
    let env = Env::default();
    let contract_id = env.register_contract(None, RemotePatientMonitoringContract);
    let client = RemotePatientMonitoringContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let patient = Address::generate(&env);

    client.initialize(&admin);
    client.register_device(&admin, &1, &"HeartRateMonitor".into(), &patient);

    client.submit_vital_sign(&admin, &patient, &1, &"heart_rate".into(), &80, &"bpm".into());

    // Check if alert is triggered if threshold set
}