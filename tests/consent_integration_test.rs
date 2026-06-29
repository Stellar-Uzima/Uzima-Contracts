#![cfg(test)]

use soroban_sdk::testutils::{Address as _, Events, Ledger as _};
use soroban_sdk::{contract, contractimpl, Address, Env, String, Symbol};

use patient_consent_management::{
    Error, PatientConsentManagement, PatientConsentManagementClient,
};

// ---------------------------------------------------------------------------
// Mock consumer that checks consent before granting access
// ---------------------------------------------------------------------------

#[contract]
struct MockDataConsumer;

#[contractimpl]
impl MockDataConsumer {
    pub fn access_data(
        env: Env,
        consent_contract: Address,
        patient: Address,
        provider: Address,
    ) -> Result<(), Error> {
        let consent_client = PatientConsentManagementClient::new(&env, &consent_contract);
        if consent_client.check_consent(&patient, &provider) {
            env.events().publish(
                (Symbol::new(&env, "DATA_ACC"),),
                (patient, provider),
            );
            Ok(())
        } else {
            Err(Error::ConsentNotGranted)
        }
    }

    pub fn emergency_access(
        env: Env,
        consent_contract: Address,
        patient: Address,
        provider: Address,
    ) -> Result<(), Error> {
        let consent_client = PatientConsentManagementClient::new(&env, &consent_contract);
        if consent_client.check_emergency_override(&provider) {
            env.events().publish(
                (Symbol::new(&env, "EM_ACC"),),
                (patient, provider),
            );
            Ok(())
        } else {
            Err(Error::EmergencyOverrideRequired)
        }
    }
}

// ---------------------------------------------------------------------------
// Consent integration tests
// ---------------------------------------------------------------------------

fn setup() -> (Env, PatientConsentManagementClient<'static>, Address) {
    let env = Env::default();
    env.mock_all_auths();
    env.ledger().set_timestamp(1_000_000);
    let admin = Address::generate(&env);
    let contract_id = env.register_contract(None, PatientConsentManagement);
    let client = PatientConsentManagementClient::new(&env, &contract_id);
    client.initialize(&admin);
    (env, client, admin)
}

#[test]
fn test_consent_grant_and_revoke_lifecycle() {
    let (env, client, _admin) = setup();
    let patient = Address::generate(&env);
    let provider = Address::generate(&env);

    assert!(!client.check_consent(&patient, &provider));
    client.grant_consent(&patient, &provider);
    assert!(client.check_consent(&patient, &provider));
    client.revoke_consent(&patient, &provider);
    assert!(!client.check_consent(&patient, &provider));
}

#[test]
fn test_consent_with_expiry() {
    let (env, client, _admin) = setup();
    let patient = Address::generate(&env);
    let provider = Address::generate(&env);
    let expires_at = env.ledger().timestamp() + 100;

    client.grant_consent_with_expiry(&patient, &provider, &expires_at);
    assert!(client.check_consent(&patient, &provider));

    env.ledger().set_timestamp(expires_at + 1);
    assert!(!client.check_consent(&patient, &provider));
}

#[test]
fn test_cross_contract_consent_check() {
    let (env, client, _admin) = setup();
    let mock_id = env.register_contract(None, MockDataConsumer);
    let mock_client = MockDataConsumerClient::new(&env, &mock_id);

    let patient = Address::generate(&env);
    let provider = Address::generate(&env);
    let consent_id = env.register_contract(None, PatientConsentManagement);
    let consent_client = PatientConsentManagementClient::new(&env, &consent_id);
    consent_client.initialize(&provider);

    consent_client.grant_consent(&patient, &provider);
    let result = mock_client.try_access_data(&consent_id, &patient, &provider);
    assert!(result.is_ok());
    let events = env.events().all();
    let has_data_acc = events.iter().any(|e| {
        e.topics
            .get(0)
            .and_then(|v| Symbol::try_from_val(&env, &v).ok())
            == Some(Symbol::new(&env, "DATA_ACC"))
    });
    assert!(has_data_acc);
}

#[test]
fn test_consent_denied_without_grant() {
    let (env, client, _admin) = setup();
    let mock_id = env.register_contract(None, MockDataConsumer);
    let mock_client = MockDataConsumerClient::new(&env, &mock_id);

    let patient = Address::generate(&env);
    let provider = Address::generate(&env);
    let consent_id = env.register_contract(None, PatientConsentManagement);
    let consent_client = PatientConsentManagementClient::new(&env, &consent_id);
    consent_client.initialize(&provider);

    let result = mock_client.try_access_data(&consent_id, &patient, &provider);
    assert_eq!(result, Err(Ok(Error::ConsentNotGranted)));
}

#[test]
fn test_multiple_providers() {
    let (env, client, _admin) = setup();
    let patient = Address::generate(&env);
    let provider1 = Address::generate(&env);
    let provider2 = Address::generate(&env);

    client.grant_consent(&patient, &provider1);
    assert!(client.check_consent(&patient, &provider1));
    assert!(!client.check_consent(&patient, &provider2));
}

#[test]
fn test_revoke_only_target_provider() {
    let (env, client, _admin) = setup();
    let patient = Address::generate(&env);
    let provider1 = Address::generate(&env);
    let provider2 = Address::generate(&env);

    client.grant_consent(&patient, &provider1);
    client.grant_consent(&patient, &provider2);
    client.revoke_consent(&patient, &provider1);
    assert!(!client.check_consent(&patient, &provider1));
    assert!(client.check_consent(&patient, &provider2));
}
