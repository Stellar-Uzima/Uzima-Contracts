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

    let connectivity = soroban_sdk::Vec::from_array(
        &env,
        [
            soroban_sdk::String::from_str(&env, "WiFi"),
            soroban_sdk::String::from_str(&env, "Bluetooth"),
        ],
    );

    client.register_device(&admin, &1, &0, &patient, &connectivity); // 0 for BloodPressureMonitor

    // Verify device is registered
    let device = client.get_device(&1);
    assert_eq!(device.map(|registered| registered.device_type), Some(0));
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
    let connectivity =
        soroban_sdk::Vec::from_array(&env, [soroban_sdk::String::from_str(&env, "WiFi")]);
    client.register_device(&admin, &1, &1, &patient, &connectivity); // 1 for HeartRateMonitor

    client.submit_vital_sign(
        &admin,
        &patient,
        &1,
        &soroban_sdk::String::from_str(&env, "heart_rate"),
        &80,
        &soroban_sdk::String::from_str(&env, "bpm"),
        &95, // quality
    );

    // Check if alert is triggered if threshold set
    // First set threshold
    client.set_threshold(
        &patient,
        &patient,
        &soroban_sdk::String::from_str(&env, "heart_rate"),
        &60,
        &100,
        &3,
    );

    // Submit out of range
    client.submit_vital_sign(
        &admin,
        &patient,
        &1,
        &soroban_sdk::String::from_str(&env, "heart_rate"),
        &120,
        &soroban_sdk::String::from_str(&env, "bpm"),
        &90,
    );

    // Should trigger alert
}

#[test]
fn test_add_caregiver() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, RemotePatientMonitoringContract);
    let client = RemotePatientMonitoringContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let patient = Address::generate(&env);
    let caregiver = Address::generate(&env);

    client.initialize(&admin);
    let connectivity =
        soroban_sdk::Vec::from_array(&env, [soroban_sdk::String::from_str(&env, "WiFi")]);
    client.register_device(&admin, &1, &0, &patient, &connectivity);

    client.add_caregiver(&patient, &1, &caregiver);

    let device = client.get_device(&1);
    assert!(device.is_some());
    let caregiver_added = device
        .map(|registered| registered.caregivers.contains(&caregiver))
        .unwrap_or(false);
    assert!(caregiver_added);
}

#[test]
fn test_threshold_alerts_and_caregiver_notifications() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, RemotePatientMonitoringContract);
    let client = RemotePatientMonitoringContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let patient = Address::generate(&env);
    let caregiver = Address::generate(&env);

    client.initialize(&admin);
    let connectivity = soroban_sdk::Vec::from_array(
        &env,
        [
            soroban_sdk::String::from_str(&env, "WiFi"),
            soroban_sdk::String::from_str(&env, "Cellular"),
        ],
    );
    client.register_device(&admin, &1, &1, &patient, &connectivity);
    client.add_caregiver(&patient, &1, &caregiver);
    client.set_threshold(
        &patient,
        &patient,
        &soroban_sdk::String::from_str(&env, "heart_rate"),
        &60,
        &100,
        &3,
    );

    client.submit_vital_sign(
        &admin,
        &patient,
        &1,
        &soroban_sdk::String::from_str(&env, "heart_rate"),
        &120,
        &soroban_sdk::String::from_str(&env, "bpm"),
        &95,
    );

    let alerts = client.get_alerts(&patient, &10);
    assert_eq!(alerts.len(), 1);
    assert_eq!(alerts.get(0).map(|alert| alert.alert_type), Some(0));

    let caregiver_alerts = client.get_caregiver_alerts(&caregiver);
    assert_eq!(caregiver_alerts.len(), 1);
    assert_eq!(caregiver_alerts.get(0).map(|alert| alert.severity), Some(3));
}
