#![cfg(test)]

use super::*;
use soroban_sdk::{testutils::Ledger, Env};

#[test]
fn test_initialize() {
    let env = Env::default();
    let contract_id = env.register_contract(None, HealthCheckContract);
    let client = HealthCheckContractClient::new(&env, &contract_id);

    assert!(client.initialize());
    assert!(!client.initialize()); // Second init should fail
}

#[test]
fn test_health_check() {
    let env = Env::default();
    let contract_id = env.register_contract(None, HealthCheckContract);
    let client = HealthCheckContractClient::new(&env, &contract_id);

    client.initialize();

    let health = client.health_check();
    assert_eq!(health.version, String::from_str(&env, "1.0.0"));
    assert!(!health.is_paused);
    assert_eq!(health.total_operations, 0);
    assert_eq!(health.failed_operations, 0);
    assert_eq!(health.success_rate, 10000); // 100%
}

#[test]
fn test_record_operation() {
    let env = Env::default();
    let contract_id = env.register_contract(None, HealthCheckContract);
    let client = HealthCheckContractClient::new(&env, &contract_id);

    client.initialize();

    // Record successful operation
    client.record_operation(&1000, &true);

    let health = client.health_check();
    assert_eq!(health.total_operations, 1);
    assert_eq!(health.failed_operations, 0);
    assert_eq!(health.success_rate, 10000);

    // Record failed operation
    client.record_operation(&1500, &false);

    let health = client.health_check();
    assert_eq!(health.total_operations, 2);
    assert_eq!(health.failed_operations, 1);
    assert_eq!(health.success_rate, 5000); // 50%
}

#[test]
fn test_gas_metrics() {
    let env = Env::default();
    let contract_id = env.register_contract(None, HealthCheckContract);
    let client = HealthCheckContractClient::new(&env, &contract_id);

    client.initialize();

    client.record_operation(&1000, &true);
    client.record_operation(&2000, &true);
    client.record_operation(&1500, &true);

    let gas_metrics = client.get_gas_metrics();
    assert_eq!(gas_metrics.total_consumed, 4500);
    assert_eq!(gas_metrics.average_usage, 1500);
    assert_eq!(gas_metrics.peak_usage, 2000);
    assert_eq!(gas_metrics.operation_count, 3);
}

#[test]
fn test_error_metrics() {
    let env = Env::default();
    let contract_id = env.register_contract(None, HealthCheckContract);
    let client = HealthCheckContractClient::new(&env, &contract_id);

    client.initialize();

    client.record_error(&101);
    client.record_error(&102);
    client.record_error(&101);

    let error_metrics = client.get_error_metrics();
    assert_eq!(error_metrics.total_errors, 3);
    assert_eq!(error_metrics.recent_errors, 3);
    assert_eq!(error_metrics.common_error_code, 101);
}

#[test]
fn test_monitoring_metrics() {
    let env = Env::default();
    let contract_id = env.register_contract(None, HealthCheckContract);
    let client = HealthCheckContractClient::new(&env, &contract_id);

    client.initialize();

    client.record_operation(&1000, &true);
    client.record_error(&101);

    let metrics = client.get_monitoring_metrics();
    assert_eq!(metrics.version, String::from_str(&env, "1.0.0"));
    assert!(!metrics.is_paused);
    assert_eq!(metrics.total_operations, 1);
    assert_eq!(metrics.error_count, 1);
    assert_eq!(metrics.avg_gas_usage, 1000);
}

#[test]
fn test_pause_functionality() {
    let env = Env::default();
    let contract_id = env.register_contract(None, HealthCheckContract);
    let client = HealthCheckContractClient::new(&env, &contract_id);

    client.initialize();

    let health = client.health_check();
    assert!(!health.is_paused);

    client.set_paused(&true);

    let health = client.health_check();
    assert!(health.is_paused);
}

#[test]
fn test_alert_thresholds() {
    let env = Env::default();
    let contract_id = env.register_contract(None, HealthCheckContract);
    let client = HealthCheckContractClient::new(&env, &contract_id);

    client.initialize();

    // Initially no alerts
    let alerts = client.check_alert_thresholds();
    assert!(alerts.is_empty() || alerts.len() == 1); // May have inactivity alert

    // Trigger high error rate
    client.record_operation(&1000, &false);
    client.record_operation(&1000, &false);
    client.record_operation(&1000, &false);
    client.record_operation(&1000, &true);

    let alerts = client.check_alert_thresholds();
    assert!(alerts.len() > 0);

    // Trigger pause alert
    client.set_paused(&true);
    let alerts = client.check_alert_thresholds();
    assert!(alerts.len() > 0);
}

#[test]
fn test_last_activity_tracking() {
    let env = Env::default();
    env.ledger().with_mut(|li| {
        li.timestamp = 1000;
    });

    let contract_id = env.register_contract(None, HealthCheckContract);
    let client = HealthCheckContractClient::new(&env, &contract_id);

    client.initialize();

    let health = client.health_check();
    assert_eq!(health.last_activity, 1000);

    env.ledger().with_mut(|li| {
        li.timestamp = 2000;
    });

    client.record_operation(&1000, &true);

    let health = client.health_check();
    assert_eq!(health.last_activity, 2000);
}

#[test]
fn test_reset_recent_errors() {
    let env = Env::default();
    let contract_id = env.register_contract(None, HealthCheckContract);
    let client = HealthCheckContractClient::new(&env, &contract_id);

    client.initialize();

    client.record_error(&101);
    client.record_error(&102);

    let error_metrics = client.get_error_metrics();
    assert_eq!(error_metrics.recent_errors, 2);

    client.reset_recent_errors();

    let error_metrics = client.get_error_metrics();
    assert_eq!(error_metrics.recent_errors, 0);
    assert_eq!(error_metrics.total_errors, 2); // Total should remain
}
