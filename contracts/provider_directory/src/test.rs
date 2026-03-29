#![cfg(test)]

use super::*;
use soroban_sdk::testutils::{Address as _, Ledger};
use soroban_sdk::{vec, Env, String, Symbol};

#[test]
fn test_initialize() {
    let env = Env::default();
    let admin = Address::generate(&env);
    let identity_registry = Address::generate(&env);
    
    let contract_id = env.register_contract(None, ProviderDirectoryContract);
    let client = ProviderDirectoryContractClient::new(&env, &contract_id);

    client.initialize(&admin, &identity_registry);

    // Try initializing again should fail
    let result = client.try_initialize(&admin, &identity_registry);
    assert_eq!(result, Err(Ok(Error::AlreadyInitialized)));
}

#[test]
fn test_profile_management() {
    let env = Env::default();
    env.mock_all_auths();
    
    let admin = Address::generate(&env);
    let identity_registry = Address::generate(&env);
    let provider = Address::generate(&env);
    
    let contract_id = env.register_contract(None, ProviderDirectoryContract);
    let client = ProviderDirectoryContractClient::new(&env, &contract_id);
    client.initialize(&admin, &identity_registry);

    let name = String::from_str(&env, "Dr. Smith");
    let specialties = vec![&env, Symbol::new(&env, "Cardiology"), Symbol::new(&env, "InternalMedicine")];
    let bio = String::from_str(&env, "Experienced cardiologist with 10 years experience.");
    let location = String::from_str(&env, "New York, NY");
    let contact = String::from_str(&env, "drsmith@example.com");

    client.update_profile(
        &provider,
        &name,
        &specialties,
        &bio,
        &location,
        &contact,
    );

    let profile = client.get_profile(&provider);
    assert_eq!(profile.name, name);
    assert_eq!(profile.specialties, specialties);
    assert_eq!(profile.is_verified, false);
}

#[test]
fn test_search_by_specialty() {
    let env = Env::default();
    env.mock_all_auths();
    
    let admin = Address::generate(&env);
    let identity_registry = Address::generate(&env);
    let p1 = Address::generate(&env);
    let p2 = Address::generate(&env);
    
    let contract_id = env.register_contract(None, ProviderDirectoryContract);
    let client = ProviderDirectoryContractClient::new(&env, &contract_id);
    client.initialize(&admin, &identity_registry);

    let cardiology = Symbol::new(&env, "Cardiology");
    let neurology = Symbol::new(&env, "Neurology");

    client.update_profile(
        &p1,
        &String::from_str(&env, "P1"),
        &vec![&env, cardiology.clone()],
        &String::from_str(&env, "Bio"),
        &String::from_str(&env, "Loc"),
        &String::from_str(&env, "Contact"),
    );

    client.update_profile(
        &p2,
        &String::from_str(&env, "P2"),
        &vec![&env, neurology.clone()],
        &String::from_str(&env, "Bio"),
        &String::from_str(&env, "Loc"),
        &String::from_str(&env, "Contact"),
    );

    let results = client.search_by_specialty(&cardiology);
    assert_eq!(results.len(), 1);
    assert_eq!(results.get(0).unwrap().address, p1);

    let results = client.search_by_specialty(&neurology);
    assert_eq!(results.len(), 1);
    assert_eq!(results.get(0).unwrap().address, p2);
}

#[test]
fn test_availability() {
    let env = Env::default();
    env.mock_all_auths();
    
    let admin = Address::generate(&env);
    let identity_registry = Address::generate(&env);
    let provider = Address::generate(&env);
    
    let contract_id = env.register_contract(None, ProviderDirectoryContract);
    let client = ProviderDirectoryContractClient::new(&env, &contract_id);
    client.initialize(&admin, &identity_registry);

    client.update_profile(
        &provider,
        &String::from_str(&env, "Dr. Smith"),
        &vec![&env],
        &String::from_str(&env, "Bio"),
        &String::from_str(&env, "Loc"),
        &String::from_str(&env, "Contact"),
    );

    let avail = vec![&env, Availability {
        day_of_week: 1,
        start_hour: 9,
        end_hour: 17,
        timezone: String::from_str(&env, "EST"),
    }];

    client.set_availability(&provider, &avail);
    let stored_avail = client.get_availability(&provider);
    assert_eq!(stored_avail.len(), 1);
    assert_eq!(stored_avail.get(0).unwrap().day_of_week, 1);
}

#[test]
fn test_verification() {
    let env = Env::default();
    env.mock_all_auths();
    
    let admin = Address::generate(&env);
    let identity_registry = Address::generate(&env);
    let provider = Address::generate(&env);
    
    let contract_id = env.register_contract(None, ProviderDirectoryContract);
    let client = ProviderDirectoryContractClient::new(&env, &contract_id);
    client.initialize(&admin, &identity_registry);

    client.update_profile(
        &provider,
        &String::from_str(&env, "Dr. Smith"),
        &vec![&env],
        &String::from_str(&env, "Bio"),
        &String::from_str(&env, "Loc"),
        &String::from_str(&env, "Contact"),
    );

    client.verify_provider(&admin, &provider);
    let profile = client.get_profile(&provider);
    assert_eq!(profile.is_verified, true);
}
