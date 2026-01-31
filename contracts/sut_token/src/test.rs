use crate::{SutToken, SutTokenClient};
use soroban_sdk::{testutils::Address as _, Address, Env, String};

fn create_token_contract(env: &Env) -> Address {
    env.register_contract(None, SutToken)
}

#[test]
fn test_initialize() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_addr = create_token_contract(&env);
    let client = SutTokenClient::new(&env, &contract_addr);
    let admin = Address::generate(&env);

    // Initialize matches lib.rs: admin, name, symbol, decimals, supply_cap
    client.initialize(
        &admin,
        &String::from_str(&env, "SUT Token"),
        &String::from_str(&env, "SUT"),
        &7u32,
        &1_000_000_000_000i128, // supply cap
    );

    assert_eq!(client.name(), String::from_str(&env, "SUT Token"));
    assert_eq!(client.symbol(), String::from_str(&env, "SUT"));
    assert_eq!(client.decimals(), 7);

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
        &String::from_str(&env, "SUT"),
        &String::from_str(&env, "SUT"),
        &7u32,
        &1_000_000i128,
    );

    // mint(minter, to, amount)
    client.mint(&admin, &user, &1000);
    assert_eq!(client.balance_of(&user), 1000);
    assert_eq!(client.total_supply(), 1000);

    // burn(minter, from, amount)
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
        &String::from_str(&env, "SUT"),
        &String::from_str(&env, "SUT"),
        &7u32,
        &1_000_000i128,
    );

    client.mint(&admin, &sender, &1000);

    // transfer(from, to, amount)
    client.transfer(&sender, &recipient, &200);

    assert_eq!(client.balance_of(&sender), 800);
    assert_eq!(client.balance_of(&recipient), 200);
}

#[test]
fn test_allowance() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_addr = create_token_contract(&env);
    let client = SutTokenClient::new(&env, &contract_addr);
    let admin = Address::generate(&env);
    let owner = Address::generate(&env);
    let spender = Address::generate(&env);

    client.initialize(
        &admin,
        &String::from_str(&env, "SUT"),
        &String::from_str(&env, "SUT"),
        &7u32,
        &1_000_000i128,
    );

    client.mint(&admin, &owner, &1000);

    // approve(owner, spender, amount)
    client.approve(&owner, &spender, &500);

    assert_eq!(client.allowance(&owner, &spender), 500);

    // transfer_from(spender, from, to, amount)
    client.transfer_from(&spender, &owner, &spender, &200);

    assert_eq!(client.balance_of(&owner), 800);
    assert_eq!(client.balance_of(&spender), 200);
    assert_eq!(client.allowance(&owner, &spender), 300);
}

#[test]
fn test_minter_management() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_addr = create_token_contract(&env);
    let client = SutTokenClient::new(&env, &contract_addr);
    let admin = Address::generate(&env);
    let new_minter = Address::generate(&env);

    client.initialize(
        &admin,
        &String::from_str(&env, "SUT"),
        &String::from_str(&env, "SUT"),
        &7u32,
        &1_000_000i128,
    );

    assert!(client.is_minter(&admin));
    assert!(!client.is_minter(&new_minter));

    // add_minter(minter) - called by admin (mocked)
    client.add_minter(&new_minter);
    assert!(client.is_minter(&new_minter));

    // remove_minter(minter)
    client.remove_minter(&new_minter);
    assert!(!client.is_minter(&new_minter));
}
