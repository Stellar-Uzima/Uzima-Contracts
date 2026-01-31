use crate::{TreasuryControllerContract, TreasuryControllerContractClient};
use soroban_sdk::{testutils::Address as _, vec, Address, Env};

#[test]
fn test_treasury_flow() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let token = Address::generate(&env);
    let signer = Address::generate(&env);

    let contract_id = env.register_contract(None, TreasuryControllerContract);
    let client = TreasuryControllerContractClient::new(&env, &contract_id);

    // FIXED: Removed .is_ok() because initialize returns bool, not Result
    let init_res = client.initialize(&admin, &token, &vec![&env, signer.clone()], &1, &3600);
    assert!(init_res);

    // Verify configuration was stored correctly
    let config = client.get_config();
    assert_eq!(config.admin, admin);
    assert_eq!(config.multisig_config.threshold, 1);
}
