#![cfg(test)]
#![allow(clippy::unwrap_used)]

use crate::contract::TokenSaleContractClient;
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

fn setup_basic_sale(env: &Env) -> (TokenSaleContractClient, Address, Address, Address) {
    env.mock_all_auths();
    let owner = Address::generate(env);
    let treasury = Address::generate(env);
    let (sut_token_address, _sut_client, sut_admin) = create_token_contract(env, &owner);
    let (payment_token_address, _pay_client, pay_admin) = create_token_contract(env, &owner);

    let contract_id = env.register_contract(None, crate::TokenSaleContract);
    let client = TokenSaleContractClient::new(env, &contract_id);

    client.initialize(&owner, &sut_token_address, &treasury, &500, &10000, &7);
    client.add_supported_token(&payment_token_address);

    env.ledger().with_mut(|li| {
        li.timestamp = 1500;
    });
    client.add_sale_phase(&1000, &2000, &100, &50000000, &5000);
    client.add_sale_phase(&2001, &3000, &150, &30000000, &3000);

    sut_admin.mint(&contract_id, &50000000);
    pay_admin.mint(&owner, &i128::MAX);

    (client, owner, payment_token_address, contract_id)
}

#[test]
fn test_invariant_total_raised_equals_sum_contributions() {
    let env = Env::default();
    let (client, _owner, pay_token, _cid) = setup_basic_sale(&env);

    let c1 = Address::generate(&env);
    let c2 = Address::generate(&env);

    // Fund contributors
    let pay_admin = token::StellarAssetClient::new(&env, &pay_token);
    pay_admin.mint(&c1, &10000);
    pay_admin.mint(&c2, &10000);

    client.contribute(&c1, &0, &pay_token, &1000);
    client.contribute(&c2, &0, &pay_token, &2000);

    let total_raised = client.get_total_raised();
    assert_eq!(total_raised, 3000);

    let cont1 = client.get_contribution(&c1).unwrap();
    let cont2 = client.get_contribution(&c2).unwrap();
    assert_eq!(cont1.amount + cont2.amount, total_raised);
}

#[test]
fn test_invariant_each_phase_sold_does_not_exceed_max() {
    let env = Env::default();
    let (client, _owner, pay_token, _cid) = setup_basic_sale(&env);

    let c1 = Address::generate(&env);
    let c2 = Address::generate(&env);

    let pay_admin = token::StellarAssetClient::new(&env, &pay_token);
    pay_admin.mint(&c1, &100000);
    pay_admin.mint(&c2, &100000);

    client.contribute(&c1, &0, &pay_token, &4000);
    client.contribute(&c2, &0, &pay_token, &4000);

    let phase0 = client.get_sale_phase(&0).unwrap();
    assert!(phase0.sold_tokens <= phase0.max_tokens);
    assert!(phase0.sold_tokens > 0);

    // Move to phase 2
    env.ledger().with_mut(|li| {
        li.timestamp = 2500;
    });
    client.contribute(&c1, &1, &pay_token, &1500);

    let phase1 = client.get_sale_phase(&1).unwrap();
    assert!(phase1.sold_tokens <= phase1.max_tokens);
}

#[test]
fn test_invariant_hard_cap_not_exceeded() {
    let env = Env::default();
    let (client, _owner, _pay_token, _cid) = setup_basic_sale(&env);

    let config = client.get_config();
    let hard_cap = config.hard_cap;
    let soft_cap = config.soft_cap;

    assert!(soft_cap <= hard_cap);
}

#[test]
fn test_invariant_contribution_cap_not_exceeded() {
    let env = Env::default();
    let (client, _owner, pay_token, _cid) = setup_basic_sale(&env);

    let c1 = Address::generate(&env);
    let pay_admin = token::StellarAssetClient::new(&env, &pay_token);
    pay_admin.mint(&c1, &100000);

    // Contribute up to per-address cap for phase 0
    client.contribute(&c1, &0, &pay_token, &4000);

    let cont = client.get_contribution(&c1).unwrap();
    let phase0 = client.get_sale_phase(&0).unwrap();

    assert!(cont.amount <= phase0.per_address_cap);

    // Second contribution in same phase should be capped
    let result = client.try_contribute(&c1, &0, &pay_token, &4000);
    assert_eq!(result, Err(Ok(crate::Error::CapExceeded)));
}

#[test]
fn test_invariant_phase_transition_allocation() {
    let env = Env::default();
    let (client, _owner, pay_token, _cid) = setup_basic_sale(&env);

    let c1 = Address::generate(&env);
    let c2 = Address::generate(&env);

    let pay_admin = token::StellarAssetClient::new(&env, &pay_token);
    pay_admin.mint(&c1, &100000);
    pay_admin.mint(&c2, &100000);

    // Phase 0 contributions
    client.contribute(&c1, &0, &pay_token, &1000);
    client.contribute(&c2, &0, &pay_token, &2000);

    let phase0_before = client.get_sale_phase(&0).unwrap();
    let phase1_before = client.get_sale_phase(&1).unwrap();

    // Move to phase 1
    env.ledger().with_mut(|li| {
        li.timestamp = 2500;
    });

    client.contribute(&c1, &1, &pay_token, &500);
    client.contribute(&c2, &1, &pay_token, &1000);

    let phase0_after = client.get_sale_phase(&0).unwrap();
    let phase1_after = client.get_sale_phase(&1).unwrap();

    // Phase 0 sold should not change after phase 1 starts
    assert_eq!(phase0_before.sold_tokens, phase0_after.sold_tokens);
    assert!(phase1_after.sold_tokens > phase1_before.sold_tokens);
}

#[test]
fn test_invariant_total_sold_equals_sum_phase_sold() {
    let env = Env::default();
    let (client, _owner, pay_token, _cid) = setup_basic_sale(&env);

    let c1 = Address::generate(&env);
    let c2 = Address::generate(&env);

    let pay_admin = token::StellarAssetClient::new(&env, &pay_token);
    pay_admin.mint(&c1, &100000);
    pay_admin.mint(&c2, &100000);

    client.contribute(&c1, &0, &pay_token, &1000);
    client.contribute(&c2, &0, &pay_token, &2000);

    let phase0 = client.get_sale_phase(&0).unwrap();

    // All contributions went to phase 0
    let cont1 = client.get_contribution(&c1).unwrap();
    let cont2 = client.get_contribution(&c2).unwrap();
    let sum_phase_sold = phase0.sold_tokens;
    let sum_contrib_tokens = cont1.tokens_allocated + cont2.tokens_allocated;

    assert_eq!(sum_phase_sold, sum_contrib_tokens);
}

#[test]
fn test_invariant_claim_releases_correct_tokens() {
    let env = Env::default();
    let (client, _owner, pay_token, _cid) = setup_basic_sale(&env);

    let c1 = Address::generate(&env);
    let c2 = Address::generate(&env);

    let pay_admin = token::StellarAssetClient::new(&env, &pay_token);
    pay_admin.mint(&c1, &10000);
    pay_admin.mint(&c2, &10000);

    client.contribute(&c1, &0, &pay_token, &500);
    client.contribute(&c2, &0, &pay_token, &1500);

    let cont1_before = client.get_contribution(&c1).unwrap();
    let cont2_before = client.get_contribution(&c2).unwrap();

    client.finalize_sale();

    client.claim_tokens(&c1);
    client.claim_tokens(&c2);

    let cont1_after = client.get_contribution(&c1).unwrap();
    let cont2_after = client.get_contribution(&c2).unwrap();

    assert!(cont1_after.claimed);
    assert!(cont2_after.claimed);

    let total_allocated = cont1_before.tokens_allocated + cont2_before.tokens_allocated;
    let sut_client = token::Client::new(&env, &client.get_config().token_address);
    assert_eq!(sut_client.balance(&c1) + sut_client.balance(&c2), total_allocated as i128);
}

#[test]
fn test_invariant_refund_returns_exact_contribution() {
    let env = Env::default();
    env.mock_all_auths();

    let owner = Address::generate(&env);
    let treasury = Address::generate(&env);
    let c1 = Address::generate(&env);

    let (sut_address, _sut_client, _sut_admin) = create_token_contract(&env, &owner);
    let (pay_address, pay_client, pay_admin) = create_token_contract(&env, &owner);

    let contract_id = env.register_contract(None, crate::TokenSaleContract);
    let client = TokenSaleContractClient::new(&env, &contract_id);

    // High soft cap to trigger refund
    client.initialize(&owner, &sut_address, &treasury, &10000, &20000, &7);
    client.add_supported_token(&pay_address);

    env.ledger().with_mut(|li| {
        li.timestamp = 1500;
    });
    client.add_sale_phase(&1000, &2000, &100, &50000000, &5000);

    pay_admin.mint(&c1, &10000);
    pay_admin.mint(&contract_id, &10000);

    let contribution_amount = 3000u128;
    client.contribute(&c1, &0, &pay_address, &contribution_amount);

    let balance_before = pay_client.balance(&c1);
    client.finalize_sale();

    let config = client.get_config();
    assert!(config.refunds_enabled);

    client.claim_refund(&c1, &pay_address);
    assert_eq!(pay_client.balance(&c1), balance_before + contribution_amount as i128);
}
