//! Rental-yield (payout) integration and invariant tests.

use super::*;
use soroban_sdk::{testutils::Address as _, token, Address, Env};

const TOTAL: i128 = 1_000;

struct Fixture<'a> {
    client: FractionalOwnershipClient<'a>,
    contract_id: Address,
    token: token::Client<'a>,
    token_admin: token::StellarAssetClient<'a>,
    issuer: Address,
    asset_id: u64,
}

fn setup<'a>(env: &'a Env) -> Fixture<'a> {
    env.mock_all_auths();
    let admin = Address::generate(env);
    let sac = env.register_stellar_asset_contract_v2(admin.clone());
    let token_addr = sac.address();
    let contract_id = env.register_contract(None, FractionalOwnership);
    let client = FractionalOwnershipClient::new(env, &contract_id);
    client.initialize(&admin, &token_addr);
    let issuer = Address::generate(env);
    let asset_id = client.create_asset(&admin, &issuer, &TOTAL);
    Fixture {
        client,
        contract_id,
        token: token::Client::new(env, &token_addr),
        token_admin: token::StellarAssetClient::new(env, &token_addr),
        issuer,
        asset_id,
    }
}

// ---------------------------------------------------------------------------
// integration: full rental-yield flow
// ---------------------------------------------------------------------------

#[test]
fn rental_yield_full_flow() {
    let env = Env::default();
    let f = setup(&env);
    let id = f.asset_id;
    let bob = Address::generate(&env);
    let carol = Address::generate(&env);
    let tenant = Address::generate(&env);
    f.token_admin.mint(&tenant, &2_000);
    f.token_admin.mint(&carol, &400);

    // Issuer splits ownership, then the first rent arrives.
    f.client.transfer(&f.issuer, &bob, &id, &300);
    f.client.deposit_payout(&tenant, &id, &1_000);
    assert_eq!(f.client.get_pending_payout(&id, &f.issuer), 700);
    assert_eq!(f.client.get_pending_payout(&id, &bob), 300);

    // Carol buys 200 shares on the market before the second rent.
    f.client.list_shares(&f.issuer, &id, &200, &2);
    f.client.buy(&carol, &f.issuer, &id, &200);
    assert_eq!(f.client.get_pending_payout(&id, &carol), 0);

    f.client.deposit_payout(&tenant, &id, &1_000);
    assert_eq!(f.client.get_pending_payout(&id, &f.issuer), 1_200);
    assert_eq!(f.client.get_pending_payout(&id, &bob), 600);
    assert_eq!(f.client.get_pending_payout(&id, &carol), 200);

    // Everyone claims; the pool is emptied exactly.
    assert_eq!(f.client.claim_payout(&f.issuer, &id), 1_200);
    assert_eq!(f.client.claim_payout(&bob, &id), 600);
    assert_eq!(f.client.claim_payout(&carol, &id), 200);
    assert_eq!(f.token.balance(&f.contract_id), 0);
    assert_eq!(f.token.balance(&f.issuer), 1_600); // 400 sale + 1_200 rent
    assert_eq!(f.token.balance(&bob), 600);
    assert_eq!(f.token.balance(&carol), 200); // 400 - 400 cost + 200 rent
}

// ---------------------------------------------------------------------------
// invariants
// ---------------------------------------------------------------------------

/// Across a sequence of deposits, transfers and claims:
/// - shares are conserved,
/// - pending payouts never exceed what the contract holds,
/// - claimed + held always equals deposited (nothing created or lost).
#[test]
fn rental_yield_invariants_hold() {
    let env = Env::default();
    let f = setup(&env);
    let id = f.asset_id;
    let bob = Address::generate(&env);
    let carol = Address::generate(&env);
    let tenant = Address::generate(&env);
    f.token_admin.mint(&tenant, &1_000_000);

    let holders = [f.issuer.clone(), bob.clone(), carol.clone()];
    let amounts: [i128; 6] = [7, 333, 1_001, 50, 999, 12_345];
    let mut deposited: i128 = 0;
    let mut claimed: i128 = 0;

    for (i, amount) in amounts.iter().enumerate() {
        f.client.deposit_payout(&tenant, &id, amount);
        deposited += amount;

        let to = if i % 2 == 0 { &bob } else { &carol };
        f.client.transfer(&f.issuer, to, &id, &50);

        let claimer = &holders[i % holders.len()];
        if f.client.get_pending_payout(&id, claimer) > 0 {
            claimed += f.client.claim_payout(claimer, &id);
        }

        let shares: i128 = holders.iter().map(|h| f.client.get_balance(&id, h)).sum();
        assert_eq!(shares, TOTAL);

        let held = f.token.balance(&f.contract_id);
        let pending: i128 = holders
            .iter()
            .map(|h| f.client.get_pending_payout(&id, h))
            .sum();
        assert!(pending <= held);
        assert_eq!(claimed + held, deposited);
    }
}
