#![allow(clippy::unwrap_used)]

use crate::{contract::TokenSaleContractClient, vesting::VestingContractClient};

use super::*;
use soroban_sdk::{
    testutils::{Address as _, Ledger},
    token, Address, Env,
};

fn create_token_contract<'a>(
    e: &Env,
    admin: &Address,
) -> (Address, token::Client<'a>, token::StellarAssetClient<'a>) {
    let contract_address = e
        .register_stellar_asset_contract_v2(admin.clone())
        .address();
    (
        contract_address.clone(),
        token::Client::new(e, &contract_address),
        token::StellarAssetClient::new(e, &contract_address),
    )
}

#[test]
fn test_token_sale_initialization() {
    let env = Env::default();
    env.mock_all_auths();

    let owner = Address::generate(&env);
    let treasury = Address::generate(&env);
    let (token_address, _token_client, _token_admin) = create_token_contract(&env, &owner);

    let contract_id = env.register_contract(None, TokenSaleContract);
    let client = TokenSaleContractClient::new(&env, &contract_id);

    client.initialize(&owner, &token_address, &treasury, &1000, &10000);

    let config = client.get_config();
    assert_eq!(config.token_address, token_address);
    assert_eq!(config.treasury, treasury);
    assert_eq!(config.soft_cap, 1000);
    assert_eq!(config.hard_cap, 10000);
    assert!(!config.is_finalized);
}

#[test]
fn test_add_sale_phase() {
    let env = Env::default();
    env.mock_all_auths();

    let owner = Address::generate(&env);
    let treasury = Address::generate(&env);
    let (token_address, _token_client, _token_admin) = create_token_contract(&env, &owner);

    let contract_id = env.register_contract(None, TokenSaleContract);
    let client = TokenSaleContractClient::new(&env, &contract_id);

    client.initialize(&owner, &token_address, &treasury, &1000, &10000);

    // Add a sale phase
    let start_time = 1000;
    let end_time = 2000;
    let price_per_token = 100; // 100 payment tokens per SUT token
    let max_tokens = 50000000; // Increased to accommodate the calculation
    let per_address_cap = 1000;

    client.add_sale_phase(
        &start_time,
        &end_time,
        &price_per_token,
        &max_tokens,
        &per_address_cap,
    );

    let phase = client.get_sale_phase(&0).unwrap();
    assert_eq!(phase.start_time, start_time);
    assert_eq!(phase.end_time, end_time);
    assert_eq!(phase.price_per_token, price_per_token);
    assert_eq!(phase.max_tokens, max_tokens);
    assert_eq!(phase.per_address_cap, per_address_cap);
    assert!(phase.is_active);
    assert_eq!(phase.sold_tokens, 0);
}

#[test]
fn test_contribution_and_claim() {
    let env = Env::default();
    env.mock_all_auths();

    let owner = Address::generate(&env);
    let treasury = Address::generate(&env);
    let contributor = Address::generate(&env);

    let (sut_token_address, sut_token_client, sut_token_admin) =
        create_token_contract(&env, &owner);
    let (payment_token_address, _payment_token_client, payment_token_admin) =
        create_token_contract(&env, &owner);

    let contract_id = env.register_contract(None, TokenSaleContract);
    let client = TokenSaleContractClient::new(&env, &contract_id);

    // Initialize contract
    client.initialize(&owner, &sut_token_address, &treasury, &500, &10000);

    // Add supported payment token
    client.add_supported_token(&payment_token_address);

    // Set up ledger time
    env.ledger().with_mut(|li| {
        li.timestamp = 1500; // Within phase time
    });

    // Add a sale phase
    client.add_sale_phase(&1000, &2000, &100, &50000000, &1000);

    // Mint payment tokens to contributor
    payment_token_admin.mint(&contributor, &1000);

    // Mint SUT tokens to contract for distribution
    sut_token_admin.mint(&contract_id, &50000000);

    // Contribute to sale
    client.contribute(&contributor, &0, &payment_token_address, &500);

    // Check contribution
    let contribution = client.get_contribution(&contributor).unwrap();
    assert_eq!(contribution.amount, 500);
    assert_eq!(contribution.tokens_allocated, 5000000); // 500 * 1_000_000 / 100

    // Finalize sale
    client.finalize_sale();

    // Claim tokens
    client.claim_tokens(&contributor);

    // Verify tokens were transferred
    assert_eq!(sut_token_client.balance(&contributor), 5000000);
}

#[test]
fn test_vesting_contract() {
    let env = Env::default();
    env.mock_all_auths();

    let owner = Address::generate(&env);
    let beneficiary = Address::generate(&env);
    let (token_address, token_client, token_admin) = create_token_contract(&env, &owner);

    let contract_id = env.register_contract(None, VestingContract);
    let client = VestingContractClient::new(&env, &contract_id);

    // Initialize vesting contract
    client.initialize_vesting(&owner, &token_address);

    // Mint tokens to vesting contract
    token_admin.mint(&contract_id, &10000);

    // Set initial time
    env.ledger().with_mut(|li| {
        li.timestamp = 1000;
    });

    // Create vesting schedule: 30 day cliff, 365 day total vesting
    let cliff_duration = 30 * 24 * 60 * 60; // 30 days in seconds
    let vesting_duration = 365 * 24 * 60 * 60; // 365 days in seconds
    let total_amount = 10000;

    client.create_vesting_schedule(
        &beneficiary,
        &cliff_duration,
        &vesting_duration,
        &total_amount,
    );

    // Check initial state - nothing releasable before cliff
    assert_eq!(client.get_releasable_amount(&beneficiary), 0);

    // Move past cliff (move to 10% through vesting period)
    env.ledger().with_mut(|li| {
        li.timestamp = 1000 + cliff_duration + (vesting_duration - cliff_duration) / 10;
    });

    // Should have some tokens releasable now
    let releasable = client.get_releasable_amount(&beneficiary);
    assert!(releasable > 0);

    // Release tokens
    let released = client.release_tokens(&beneficiary);
    assert_eq!(released, releasable);
    assert_eq!(token_client.balance(&beneficiary), released as i128);

    // Move to end of vesting period
    env.ledger().with_mut(|li| {
        li.timestamp = 1000 + vesting_duration;
    });

    // Release remaining tokens
    let _remaining_releasable = client.get_releasable_amount(&beneficiary);
    client.release_tokens(&beneficiary);

    // Should have all tokens now
    assert_eq!(token_client.balance(&beneficiary), total_amount as i128);
}

#[test]
fn test_refund_mechanism() {
    let env = Env::default();
    env.mock_all_auths();

    let owner = Address::generate(&env);
    let treasury = Address::generate(&env);
    let contributor = Address::generate(&env);

    let (sut_token_address, _sut_token_client, _sut_token_admin) =
        create_token_contract(&env, &owner);
    let (payment_token_address, payment_token_client, payment_token_admin) =
        create_token_contract(&env, &owner);

    let contract_id = env.register_contract(None, TokenSaleContract);
    let client = TokenSaleContractClient::new(&env, &contract_id);

    // Initialize with high soft cap that won't be met
    client.initialize(&owner, &sut_token_address, &treasury, &10000, &20000);
    client.add_supported_token(&payment_token_address);

    env.ledger().with_mut(|li| {
        li.timestamp = 1500;
    });

    client.add_sale_phase(&1000, &2000, &100, &50000000, &1000);

    // Mint and contribute (but not enough to meet soft cap)
    payment_token_admin.mint(&contributor, &1000);
    payment_token_admin.mint(&contract_id, &1000); // For refunds

    client.contribute(&contributor, &0, &payment_token_address, &500);

    // Finalize sale (will enable refunds since soft cap not met)
    client.finalize_sale();

    let config = client.get_config();
    assert!(config.refunds_enabled);

    // Claim refund
    let initial_balance = payment_token_client.balance(&contributor);
    client.claim_refund(&contributor, &payment_token_address);

    // Should have received refund
    assert_eq!(
        payment_token_client.balance(&contributor),
        initial_balance + 500
    );
}
