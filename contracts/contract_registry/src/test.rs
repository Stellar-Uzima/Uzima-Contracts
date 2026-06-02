use soroban_sdk::testutils::{Address as TestAddress, Ledger};
use soroban_sdk::{Address, Env, String};

use crate::{ContractRegistry, ContractRegistryClient, RegistryError};

fn generate_test_address(env: &Env) -> Address {
    <Address as TestAddress>::generate(env)
}

#[test]
fn test_registry_lifecycle() {
    let env = Env::default();
    env.ledger().set(Ledger::default());

    let admin = generate_test_address(&env);
    let registry_id = env.register_contract(None, ContractRegistry);
    let client = ContractRegistryClient::new(&env, &registry_id);

    assert_eq!(client.initialize(&admin), ());
    assert!(matches!(client.register_contract(&admin, String::from_str(&env, "rbac"), generate_test_address(&env)), Ok(())));
    assert!(client.has_contract(String::from_str(&env, "rbac")).unwrap());
    let address = client.get_contract(String::from_str(&env, "rbac")).unwrap();
    assert!(address.is_some());
    assert_eq!(address.unwrap(), client.get_contract(String::from_str(&env, "rbac")).unwrap().unwrap());
    assert!(matches!(client.unregister_contract(&admin, String::from_str(&env, "rbac")), Ok(())));
    assert!(!client.has_contract(String::from_str(&env, "rbac")).unwrap());
}
