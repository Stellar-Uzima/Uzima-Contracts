//! Integration tests for FractionalOwnership flows (#1794) and
//! access control checks (#1795).

use super::*;
use soroban_sdk::{testutils::Address as _, token, Address, Env};

const TOTAL: i128 = 1_000;

/// Full lifecycle: create -> transfer -> list -> buy -> deposit -> claim.
#[test]
fn full_ownership_flow() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let issuer = Address::generate(&env);
    let alice = Address::generate(&env);
    let bob = Address::generate(&env);
    let depositor = Address::generate(&env);

    let sac = env.register_stellar_asset_contract_v2(admin.clone());
    let token_addr = sac.address();
    let pay = token::Client::new(&env, &token_addr);
    let pay_admin = token::StellarAssetClient::new(&env, &token_addr);

    let contract_id = env.register_contract(None, FractionalOwnership);
    let client = FractionalOwnershipClient::new(&env, &contract_id);
    client.initialize(&admin, &token_addr);

    // Create an asset; the issuer holds the full supply.
    let id = client.create_asset(&admin, &issuer, &TOTAL);
    assert_eq!(client.get_balance(&id, &issuer), TOTAL);

    // Issuer transfers 200 shares to Alice.
    client.transfer(&issuer, &alice, &id, &200);
    assert_eq!(client.get_balance(&id, &alice), 200);

    // Alice lists 100 shares at 5 each; Bob buys them all.
    client.list_shares(&alice, &id, &100, &5);
    pay_admin.mint(&bob, &500);
    let cost = client.buy(&bob, &alice, &id, &100);
    assert_eq!(cost, 500);
    assert_eq!(pay.balance(&alice), 500);
    assert_eq!(client.get_balance(&id, &alice), 100);
    assert_eq!(client.get_balance(&id, &bob), 100);

    // A payout of 1000 is split pro-rata: 800 / 100 / 100.
    pay_admin.mint(&depositor, &1_000);
    client.deposit_payout(&depositor, &id, &1_000);
    assert_eq!(client.claim_payout(&issuer, &id), 800);
    assert_eq!(client.claim_payout(&alice, &id), 100);
    assert_eq!(client.claim_payout(&bob, &id), 100);
    assert_eq!(pay.balance(&contract_id), 0);

    // Shares still sum to the total supply.
    let sum = client.get_balance(&id, &issuer)
        + client.get_balance(&id, &alice)
        + client.get_balance(&id, &bob);
    assert_eq!(sum, TOTAL);
}

/// Only the stored admin can create assets (#1795).
#[test]
fn only_admin_can_create_asset() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let mallory = Address::generate(&env);
    let sac = env.register_stellar_asset_contract_v2(admin.clone());
    let contract_id = env.register_contract(None, FractionalOwnership);
    let client = FractionalOwnershipClient::new(&env, &contract_id);
    client.initialize(&admin, &sac.address());

    assert_eq!(
        client.try_create_asset(&mallory, &mallory, &TOTAL),
        Err(Ok(Error::Unauthorized))
    );
    assert_eq!(client.get_asset_count(), 0);
}

/// Holder actions are authorized by the holder, not by anyone else (#1795).
#[test]
fn transfer_requires_sender_auth() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let issuer = Address::generate(&env);
    let bob = Address::generate(&env);
    let sac = env.register_stellar_asset_contract_v2(admin.clone());
    let contract_id = env.register_contract(None, FractionalOwnership);
    let client = FractionalOwnershipClient::new(&env, &contract_id);
    client.initialize(&admin, &sac.address());
    let id = client.create_asset(&admin, &issuer, &TOTAL);

    client.transfer(&issuer, &bob, &id, &10);
    let auths = env.auths();
    assert_eq!(auths.len(), 1);
    assert_eq!(auths[0].0, issuer);
}
