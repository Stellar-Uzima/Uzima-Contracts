#![cfg(test)]

use super::*;
use soroban_sdk::{testutils::Address as _, Address, Env, String};

fn create_token_contract<'a>(env: &Env) -> Address {
    env.register_contract(None, SutToken)
}

fn initialize_token(env: &Env, contract_id: &Address, admin: &Address) -> (String, String, u32, i128) {
    let name = String::from_str(env, "Stellar Utility Token");
    let symbol = String::from_str(env, "SUT");
    let decimals = 18u32;
    let supply_cap = 1_000_000_000i128 * 10i128.pow(decimals); // 1 billion tokens

    let client = SutTokenClient::new(env, contract_id);
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
    assert_eq!(client.is_minter(&admin), true);
}

#[test]
fn test_initialize_twice_fails() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let contract_id = create_token_contract(&env);
    let client = SutTokenClient::new(&env, &contract_id);
    
    let name = String::from_str(&env, "Test Token");
    let symbol = String::from_str(&env, "TEST");
    let decimals = 18u32;
    let supply_cap = 1000000i128;

    // First initialization should succeed
    client.initialize(&admin, &name, &symbol, &decimals, &supply_cap);

    // Second initialization should fail
    let result = client.try_initialize(&admin, &name, &symbol, &decimals, &supply_cap);
    assert_eq!(result, Err(Ok(Error::AlreadyInitialized)));
}

#[test]
fn test_mint() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let user = Address::generate(&env);
    let contract_id = create_token_contract(&env);
    
    initialize_token(&env, &contract_id, &admin);
    let client = SutTokenClient::new(&env, &contract_id);

    let mint_amount = 1000i128;
    
    // Mint tokens
    client.mint(&admin, &user, &mint_amount);
    
    // Check balances
    assert_eq!(client.balance_of(&user), mint_amount);
    assert_eq!(client.total_supply(), mint_amount);
}

#[test]
fn test_mint_exceeds_cap() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let user = Address::generate(&env);
    let contract_id = create_token_contract(&env);
    let client = SutTokenClient::new(&env, &contract_id);
    
    let name = String::from_str(&env, "Test Token");
    let symbol = String::from_str(&env, "TEST");
    let decimals = 18u32;
    let supply_cap = 1000i128;

    client.initialize(&admin, &name, &symbol, &decimals, &supply_cap);

    // Try to mint more than cap
    let result = client.try_mint(&admin, &user, &(supply_cap + 1));
    assert_eq!(result, Err(Ok(Error::ExceedsSupplyCap)));
}

#[test]
fn test_burn() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let user = Address::generate(&env);
    let contract_id = create_token_contract(&env);
    
    initialize_token(&env, &contract_id, &admin);
    let client = SutTokenClient::new(&env, &contract_id);

    let mint_amount = 1000i128;
    let burn_amount = 300i128;
    
    // Mint then burn
    client.mint(&admin, &user, &mint_amount);
    client.burn(&admin, &user, &burn_amount);
    
    // Check balances
    assert_eq!(client.balance_of(&user), mint_amount - burn_amount);
    assert_eq!(client.total_supply(), mint_amount - burn_amount);
}

#[test]
fn test_burn_insufficient_balance() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let user = Address::generate(&env);
    let contract_id = create_token_contract(&env);
    
    initialize_token(&env, &contract_id, &admin);
    let client = SutTokenClient::new(&env, &contract_id);

    // Try to burn without balance
    let result = client.try_burn(&admin, &user, &100i128);
    assert_eq!(result, Err(Ok(Error::InsufficientBalance)));
}

#[test]
fn test_transfer() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let user1 = Address::generate(&env);
    let user2 = Address::generate(&env);
    let contract_id = create_token_contract(&env);
    
    initialize_token(&env, &contract_id, &admin);
    let client = SutTokenClient::new(&env, &contract_id);

    let mint_amount = 1000i128;
    let transfer_amount = 300i128;
    
    // Mint to user1, then transfer to user2
    client.mint(&admin, &user1, &mint_amount);
    client.transfer(&user1, &user2, &transfer_amount);
    
    // Check balances
    assert_eq!(client.balance_of(&user1), mint_amount - transfer_amount);
    assert_eq!(client.balance_of(&user2), transfer_amount);
    assert_eq!(client.total_supply(), mint_amount);
}

#[test]
fn test_transfer_insufficient_balance() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let user1 = Address::generate(&env);
    let user2 = Address::generate(&env);
    let contract_id = create_token_contract(&env);
    
    initialize_token(&env, &contract_id, &admin);
    let client = SutTokenClient::new(&env, &contract_id);

    // Try to transfer without sufficient balance
    let result = client.try_transfer(&user1, &user2, &100i128);
    assert_eq!(result, Err(Ok(Error::InsufficientBalance)));
}

#[test]
fn test_approve_and_transfer_from() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let owner = Address::generate(&env);
    let spender = Address::generate(&env);
    let recipient = Address::generate(&env);
    let contract_id = create_token_contract(&env);
    
    initialize_token(&env, &contract_id, &admin);
    let client = SutTokenClient::new(&env, &contract_id);

    let mint_amount = 1000i128;
    let approve_amount = 300i128;
    let transfer_amount = 200i128;
    
    // Mint to owner
    client.mint(&admin, &owner, &mint_amount);
    
    // Owner approves spender
    client.approve(&owner, &spender, &approve_amount);
    assert_eq!(client.allowance(&owner, &spender), approve_amount);
    
    // Spender transfers from owner to recipient
    client.transfer_from(&spender, &owner, &recipient, &transfer_amount);
    
    // Check balances and allowance
    assert_eq!(client.balance_of(&owner), mint_amount - transfer_amount);
    assert_eq!(client.balance_of(&recipient), transfer_amount);
    assert_eq!(client.allowance(&owner, &spender), approve_amount - transfer_amount);
}

#[test]
fn test_transfer_from_insufficient_allowance() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let owner = Address::generate(&env);
    let spender = Address::generate(&env);
    let recipient = Address::generate(&env);
    let contract_id = create_token_contract(&env);
    
    initialize_token(&env, &contract_id, &admin);
    let client = SutTokenClient::new(&env, &contract_id);

    let mint_amount = 1000i128;
    
    // Mint to owner
    client.mint(&admin, &owner, &mint_amount);
    
    // Try to transfer without allowance
    let result = client.try_transfer_from(&spender, &owner, &recipient, &100i128);
    assert_eq!(result, Err(Ok(Error::InsufficientAllowance)));
}

#[test]
fn test_minter_role_management() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let new_minter = Address::generate(&env);
    let user = Address::generate(&env);
    let contract_id = create_token_contract(&env);
    
    initialize_token(&env, &contract_id, &admin);
    let client = SutTokenClient::new(&env, &contract_id);

    // Admin should be initial minter
    assert_eq!(client.is_minter(&admin), true);
    assert_eq!(client.is_minter(&new_minter), false);
    
    // Add new minter
    client.add_minter(&new_minter);
    assert_eq!(client.is_minter(&new_minter), true);
    
    // Remove minter
    client.remove_minter(&new_minter);
    assert_eq!(client.is_minter(&new_minter), false);
}

#[test]
fn test_snapshot_functionality() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let user1 = Address::generate(&env);
    let user2 = Address::generate(&env);
    let contract_id = create_token_contract(&env);

    initialize_token(&env, &contract_id, &admin);
    let client = SutTokenClient::new(&env, &contract_id);

    // Mint some tokens
    client.mint(&admin, &user1, &1000i128);
    client.mint(&admin, &user2, &500i128);

    let total_supply_before = client.total_supply();

    // Create snapshot
    let snapshot_id = client.snapshot();
    assert_eq!(snapshot_id, 1u32);

    // Check snapshot data
    assert_eq!(client.total_supply_at(&snapshot_id), total_supply_before);

    // Mint more tokens after snapshot
    client.mint(&admin, &user1, &200i128);

    // Current supply should be different from snapshot
    assert_ne!(client.total_supply(), client.total_supply_at(&snapshot_id));
}

#[test]
fn test_historical_balances_comprehensive() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let user1 = Address::generate(&env);
    let user2 = Address::generate(&env);
    let user3 = Address::generate(&env);
    let contract_id = create_token_contract(&env);

    initialize_token(&env, &contract_id, &admin);
    let client = SutTokenClient::new(&env, &contract_id);

    // Initial state: no balances
    assert_eq!(client.balance_of(&user1), 0);
    assert_eq!(client.balance_of(&user2), 0);
    assert_eq!(client.balance_of(&user3), 0);

    // Mint initial tokens
    client.mint(&admin, &user1, &1000i128);
    client.mint(&admin, &user2, &500i128);

    // Create first snapshot
    let snapshot1 = client.snapshot();

    // Verify balances at first snapshot
    assert_eq!(client.balance_of_at(&user1, &snapshot1), 1000i128);
    assert_eq!(client.balance_of_at(&user2, &snapshot1), 500i128);
    assert_eq!(client.balance_of_at(&user3, &snapshot1), 0i128);

    // Transfer some tokens
    client.transfer(&user1, &user2, &300i128);
    client.transfer(&user1, &user3, &200i128);

    // Create second snapshot
    let snapshot2 = client.snapshot();

    // Verify balances at second snapshot
    assert_eq!(client.balance_of_at(&user1, &snapshot2), 500i128);
    assert_eq!(client.balance_of_at(&user2, &snapshot2), 800i128);
    assert_eq!(client.balance_of_at(&user3, &snapshot2), 200i128);

    // First snapshot should still have old balances
    assert_eq!(client.balance_of_at(&user1, &snapshot1), 1000i128);
    assert_eq!(client.balance_of_at(&user2, &snapshot1), 500i128);
    assert_eq!(client.balance_of_at(&user3, &snapshot1), 0i128);

    // Mint more tokens and burn some
    client.mint(&admin, &user3, &300i128);
    client.burn(&admin, &user2, &100i128);

    // Create third snapshot
    let snapshot3 = client.snapshot();

    // Verify balances at third snapshot
    assert_eq!(client.balance_of_at(&user1, &snapshot3), 500i128);
    assert_eq!(client.balance_of_at(&user2, &snapshot3), 700i128);
    assert_eq!(client.balance_of_at(&user3, &snapshot3), 500i128);

    // Previous snapshots should remain unchanged
    assert_eq!(client.balance_of_at(&user1, &snapshot1), 1000i128);
    assert_eq!(client.balance_of_at(&user2, &snapshot1), 500i128);
    assert_eq!(client.balance_of_at(&user3, &snapshot1), 0i128);

    assert_eq!(client.balance_of_at(&user1, &snapshot2), 500i128);
    assert_eq!(client.balance_of_at(&user2, &snapshot2), 800i128);
    assert_eq!(client.balance_of_at(&user3, &snapshot2), 200i128);

    // Verify total supplies at different snapshots
    assert_eq!(client.total_supply_at(&snapshot1), 1500i128);
    assert_eq!(client.total_supply_at(&snapshot2), 1500i128);
    assert_eq!(client.total_supply_at(&snapshot3), 1700i128);
}

#[test]
fn test_balance_history_with_zero_balance() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let user = Address::generate(&env);
    let contract_id = create_token_contract(&env);

    initialize_token(&env, &contract_id, &admin);
    let client = SutTokenClient::new(&env, &contract_id);

    // User starts with zero balance
    assert_eq!(client.balance_of(&user), 0);

    // Create snapshot while user has zero balance
    let snapshot1 = client.snapshot();
    assert_eq!(client.balance_of_at(&user, &snapshot1), 0);

    // Mint tokens to user
    client.mint(&admin, &user, &1000i128);

    // Create snapshot with positive balance
    let snapshot2 = client.snapshot();
    assert_eq!(client.balance_of_at(&user, &snapshot2), 1000i128);

    // Burn all tokens
    client.burn(&admin, &user, &1000i128);

    // Create snapshot with zero balance again
    let snapshot3 = client.snapshot();
    assert_eq!(client.balance_of_at(&user, &snapshot3), 0);

    // Previous snapshots should maintain their values
    assert_eq!(client.balance_of_at(&user, &snapshot1), 0);
    assert_eq!(client.balance_of_at(&user, &snapshot2), 1000i128);
}

#[test]
fn test_snapshot_nonexistent_error() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let user = Address::generate(&env);
    let contract_id = create_token_contract(&env);

    initialize_token(&env, &contract_id, &admin);
    let client = SutTokenClient::new(&env, &contract_id);

    // Try to get balance at nonexistent snapshot
    let result = client.try_balance_of_at(&user, &999u32);
    assert_eq!(result, Err(Ok(Error::SnapshotNotFound)));

    let result = client.try_total_supply_at(&999u32);
    assert_eq!(result, Err(Ok(Error::SnapshotNotFound)));
}

#[test]
fn test_multiple_operations_same_snapshot() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let user1 = Address::generate(&env);
    let user2 = Address::generate(&env);
    let contract_id = create_token_contract(&env);

    initialize_token(&env, &contract_id, &admin);
    let client = SutTokenClient::new(&env, &contract_id);

    // Mint tokens
    client.mint(&admin, &user1, &1000i128);

    // Create snapshot
    let snapshot_id = client.snapshot();

    // Perform multiple operations
    client.transfer(&user1, &user2, &300i128);
    client.mint(&admin, &user1, &100i128);
    client.burn(&admin, &user2, &50i128);

    // Balances at snapshot should reflect state before operations
    assert_eq!(client.balance_of_at(&user1, &snapshot_id), 1000i128);
    assert_eq!(client.balance_of_at(&user2, &snapshot_id), 0i128);

    // Current balances should be different
    assert_eq!(client.balance_of(&user1), 800i128); // 1000 - 300 + 100
    assert_eq!(client.balance_of(&user2), 250i128); // 300 - 50
}

#[test]
fn test_invalid_amounts() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let user = Address::generate(&env);
    let contract_id = create_token_contract(&env);
    
    initialize_token(&env, &contract_id, &admin);
    let client = SutTokenClient::new(&env, &contract_id);

    // Test negative amounts
    assert_eq!(client.try_mint(&admin, &user, &(-100i128)), Err(Ok(Error::InvalidAmount)));
    assert_eq!(client.try_burn(&admin, &user, &(-100i128)), Err(Ok(Error::InvalidAmount)));
    assert_eq!(client.try_transfer(&user, &user, &(-100i128)), Err(Ok(Error::InvalidAmount)));
    assert_eq!(client.try_approve(&user, &user, &(-100i128)), Err(Ok(Error::InvalidAmount)));
}

#[test]
fn test_zero_amount_transfers() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let user1 = Address::generate(&env);
    let user2 = Address::generate(&env);
    let contract_id = create_token_contract(&env);
    
    initialize_token(&env, &contract_id, &admin);
    let client = SutTokenClient::new(&env, &contract_id);

    // Zero amount transfers should succeed but do nothing
    client.transfer(&user1, &user2, &0i128);
    client.transfer_from(&user1, &user1, &user2, &0i128);
    
    assert_eq!(client.balance_of(&user1), 0);
    assert_eq!(client.balance_of(&user2), 0);
}

#[test]
fn test_edge_case_burn_all_balance() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let user = Address::generate(&env);
    let contract_id = create_token_contract(&env);
    
    initialize_token(&env, &contract_id, &admin);
    let client = SutTokenClient::new(&env, &contract_id);

    let mint_amount = 1000i128;
    
    // Mint and burn all
    client.mint(&admin, &user, &mint_amount);
    client.burn(&admin, &user, &mint_amount);
    
    // Balance should be 0 and storage should be cleaned up
    assert_eq!(client.balance_of(&user), 0);
    assert_eq!(client.total_supply(), 0);
}

#[test]
fn test_edge_case_approve_zero() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let owner = Address::generate(&env);
    let spender = Address::generate(&env);
    let contract_id = create_token_contract(&env);
    
    initialize_token(&env, &contract_id, &admin);
    let client = SutTokenClient::new(&env, &contract_id);

    // Approve some amount then approve zero (should clear allowance)
    client.approve(&owner, &spender, &1000i128);
    assert_eq!(client.allowance(&owner, &spender), 1000i128);
    
    client.approve(&owner, &spender, &0i128);
    assert_eq!(client.allowance(&owner, &spender), 0i128);
}

// Helper to generate the client
use soroban_sdk::contractclient;

#[contractclient(name = "SutTokenClient")]
pub trait SutTokenTrait {
    fn initialize(
        env: Env,
        admin: Address,
        name: String,
        symbol: String,
        decimals: u32,
        supply_cap: i128,
    ) -> Result<(), Error>;
    
    fn name(env: Env) -> Result<String, Error>;
    fn symbol(env: Env) -> Result<String, Error>;
    fn decimals(env: Env) -> Result<u32, Error>;
    fn total_supply(env: Env) -> Result<i128, Error>;
    fn supply_cap(env: Env) -> Result<i128, Error>;
    fn balance_of(env: Env, account: Address) -> i128;
    fn allowance(env: Env, owner: Address, spender: Address) -> i128;
    
    fn transfer(env: Env, from: Address, to: Address, amount: i128) -> Result<(), Error>;
    fn transfer_from(env: Env, spender: Address, from: Address, to: Address, amount: i128) -> Result<(), Error>;
    fn approve(env: Env, owner: Address, spender: Address, amount: i128) -> Result<(), Error>;
    
    fn mint(env: Env, minter: Address, to: Address, amount: i128) -> Result<(), Error>;
    fn burn(env: Env, minter: Address, from: Address, amount: i128) -> Result<(), Error>;
    
    fn add_minter(env: Env, minter: Address) -> Result<(), Error>;
    fn remove_minter(env: Env, minter: Address) -> Result<(), Error>;
    fn is_minter(env: Env, address: Address) -> bool;
    
    fn snapshot(env: Env) -> Result<u32, Error>;
    fn balance_of_at(env: Env, account: Address, snapshot_id: u32) -> Result<i128, Error>;
    fn total_supply_at(env: Env, snapshot_id: u32) -> Result<i128, Error>;
}
