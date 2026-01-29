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

    // Construct the args struct
    let args = GrantAccessArgs {
        grantor: patient.clone(),
        grantee_chain: ChainId::Ethereum,
        grantee_address: grantee_addr,
        permission_level: PermissionLevel::Read,
        record_scope: AccessScope::AllRecords,
        duration: 3600,
        conditions,
    };

    let grant_id = client.grant_access(&args);

    assert_eq!(grant_id, 1);

    // Verify grant exists
    let grant = client.get_grant(&grant_id).unwrap();
    assert_eq!(grant.grantor, patient);
    
    // Fixed boolean assertion
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

    let args = GrantAccessArgs {
        grantor: patient.clone(),
        grantee_chain: ChainId::Ethereum,
        grantee_address: grantee_addr,
        permission_level: PermissionLevel::Read,
        record_scope: AccessScope::AllRecords,
        duration: 3600,
        conditions,
    };

    let grant_id = client.grant_access(&args);

    client.revoke_access(&patient, &grant_id);
    
    let grant = client.get_grant(&grant_id).unwrap();
    
    // Fixed boolean assertion
    assert!(!grant.is_active);
}