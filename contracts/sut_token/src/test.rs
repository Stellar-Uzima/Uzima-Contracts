use soroban_sdk::{testutils::Address as _, Address, Env, String};
// FIXED: Direct use of SutToken/SutTokenClient to match crate root naming
use crate::{SutToken, SutTokenClient};

fn create_token_contract(env: &Env) -> Address {
    env.register_contract(None, SutToken)
}

// Helper function used by multiple test cases
fn initialize_token(
    env: &Env,
    contract_id: &Address,
    admin: &Address,
) -> (String, String, u32, i128) {
    let client = SutTokenClient::new(env, contract_id);
    let name = String::from_str(env, "Uzima Token");
    let symbol = String::from_str(env, "SUT");
    let decimals = 7u32;
    let supply_cap = 1_000_000_000i128;

    client.initialize(admin, &name, &symbol, &decimals, &supply_cap);
    (name, symbol, decimals, supply_cap)
}

#[test]
fn test_initialize() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let contract_id = create_token_contract(&env);

    let (name, symbol, decimals, supply_cap) = initialize_token(&env, &contract_id, &admin);
    let client = SutTokenClient::new(&env, &contract_id);

    // Test metadata
    assert_eq!(client.name(), name);
    assert_eq!(client.symbol(), symbol);
    assert_eq!(client.decimals(), decimals);

    // Test initial state
    assert_eq!(client.total_supply(), 0);
    assert_eq!(client.supply_cap(), supply_cap);
    assert!(client.is_minter(&admin));
}

#[test]
fn test_mint_and_burn() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_addr = create_token_contract(&env);
    let client = SutTokenClient::new(&env, &contract_addr);
    let admin = Address::generate(&env);
    let user = Address::generate(&env);

    client.initialize(
        &admin,
        &String::from_str(&env, "Uzima"),
        &String::from_str(&env, "SUT"),
        &7u32,
        &1_000_000i128,
    );

    client.mint(&admin, &user, &1000);
    assert_eq!(client.balance_of(&user), 1000);
    assert_eq!(client.total_supply(), 1000);

    client.burn(&admin, &user, &500);
    assert_eq!(client.balance_of(&user), 500);
    assert_eq!(client.total_supply(), 500);
}

#[test]
fn test_transfer() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_addr = create_token_contract(&env);
    let client = SutTokenClient::new(&env, &contract_addr);
    let admin = Address::generate(&env);
    let sender = Address::generate(&env);
    let recipient = Address::generate(&env);

    client.initialize(
        &admin,
        &String::from_str(&env, "U"),
        &String::from_str(&env, "S"),
        &7u32,
        &1_000_000i128,
    );
    client.mint(&admin, &sender, &1000);

    client.transfer(&sender, &recipient, &200);

    assert_eq!(client.balance_of(&sender), 800);
    assert_eq!(client.balance_of(&recipient), 200);
}

#[test]
fn test_minter_management() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let new_minter = Address::generate(&env);
    let contract_id = create_token_contract(&env);

    initialize_token(&env, &contract_id, &admin);
    let client = SutTokenClient::new(&env, &contract_id);

    assert!(client.is_minter(&admin));
    client.add_minter(&new_minter);
    assert!(client.is_minter(&new_minter));

    client.remove_minter(&new_minter);
    assert!(!client.is_minter(&new_minter));
}

#[test]
fn test_snapshot_functionality() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let user1 = Address::generate(&env);
    let contract_id = create_token_contract(&env);

    initialize_token(&env, &contract_id, &admin);
    let client = SutTokenClient::new(&env, &contract_id);

    client.mint(&admin, &user1, &1000i128);
    let total_supply_before = client.total_supply();

    let snapshot_id = client.snapshot();
    assert_eq!(snapshot_id, 1u32);
    assert_eq!(client.total_supply_at(&snapshot_id), total_supply_before);
}
