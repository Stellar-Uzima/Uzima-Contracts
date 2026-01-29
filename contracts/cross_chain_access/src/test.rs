use super::*;
use soroban_sdk::{testutils::Address as _, Env};

#[test]
fn test_initialize() {
    let env = Env::default();
    let contract_id = env.register_contract(None, CrossChainAccessContract);
    let client = CrossChainAccessContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let bridge = Address::generate(&env);
    let identity = Address::generate(&env);

    // Should succeed
    let res = client.initialize(&admin, &bridge, &identity);
    assert!(res);

    // Verify double init fails
    let res = client.try_initialize(&admin, &bridge, &identity);
    assert!(res.is_err());
}

#[test]
fn test_grant_access() {
    let env = Env::default();
    env.mock_all_auths();
    
    let contract_id = env.register_contract(None, CrossChainAccessContract);
    let client = CrossChainAccessContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let bridge = Address::generate(&env);
    let identity = Address::generate(&env);
    client.initialize(&admin, &bridge, &identity);

    let patient = Address::generate(&env);
    let grantee_addr = String::from_str(&env, "0x123...");
    
    let conditions: Vec<AccessCondition> = Vec::new(&env);

    let grant_id = client.grant_access(
        &patient,
        &ChainId::Ethereum,
        &grantee_addr,
        &PermissionLevel::Read,
        &AccessScope::AllRecords,
        &3600, // 1 hour
        &conditions
    );

    assert_eq!(grant_id, 1);

    // Verify grant exists
    let grant = client.get_grant(&grant_id).unwrap();
    assert_eq!(grant.grantor, patient);
    
    // Fixed: bool comparison
    assert!(grant.is_active);
}

#[test]
fn test_revoke_access() {
    let env = Env::default();
    env.mock_all_auths();
    
    let contract_id = env.register_contract(None, CrossChainAccessContract);
    let client = CrossChainAccessContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let bridge = Address::generate(&env);
    let identity = Address::generate(&env);
    client.initialize(&admin, &bridge, &identity);

    let patient = Address::generate(&env);
    let grantee_addr = String::from_str(&env, "0x123...");
    let conditions: Vec<AccessCondition> = Vec::new(&env);

    let grant_id = client.grant_access(
        &patient,
        &ChainId::Ethereum,
        &grantee_addr,
        &PermissionLevel::Read,
        &AccessScope::AllRecords,
        &3600,
        &conditions
    );

    client.revoke_access(&patient, &grant_id);
    
    let grant = client.get_grant(&grant_id).unwrap();
    
    // Fixed: bool comparison
    assert!(!grant.is_active);
}