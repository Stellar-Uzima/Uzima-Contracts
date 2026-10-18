//! Unit and fuzz tests for the rental yield (payout) flow.

use super::*;
use proptest::prelude::*;
use soroban_sdk::{testutils::Address as _, token, Address, Env};

const TOTAL: i128 = 1_000;

fn setup(env: &Env) -> (FractionalOwnershipClient<'_>, token::StellarAssetClient<'_>, Address, u64) {
    env.mock_all_auths();
    let admin = Address::generate(env);
    let token_addr = env.register_stellar_asset_contract_v2(admin.clone()).address();
    let id = env.register_contract(None, FractionalOwnership);
    let client = FractionalOwnershipClient::new(env, &id);
    client.initialize(&admin, &token_addr);
    let issuer = Address::generate(env);
    let asset_id = client.create_asset(&admin, &issuer, &TOTAL);
    (client, token::StellarAssetClient::new(env, &token_addr), issuer, asset_id)
}

// ---------------------------------------------------------------------------
// unit tests
// ---------------------------------------------------------------------------

#[test]
fn rental_yield_split_pro_rata() {
    let env = Env::default();
    let (client, token_admin, issuer, asset_id) = setup(&env);
    let tenant = Address::generate(&env);
    let holder = Address::generate(&env);
    client.transfer(&issuer, &holder, &asset_id, &250);

    token_admin.mint(&tenant, &1_000);
    client.deposit_payout(&tenant, &asset_id, &1_000);

    assert_eq!(client.get_pending_payout(&asset_id, &issuer), 750);
    assert_eq!(client.get_pending_payout(&asset_id, &holder), 250);
}

#[test]
fn rental_yield_claim_pays_and_resets() {
    let env = Env::default();
    let (client, token_admin, issuer, asset_id) = setup(&env);
    let tenant = Address::generate(&env);
    token_admin.mint(&tenant, &500);
    client.deposit_payout(&tenant, &asset_id, &500);

    assert_eq!(client.claim_payout(&issuer, &asset_id), 500);
    assert_eq!(client.get_pending_payout(&asset_id, &issuer), 0);
    assert_eq!(
        client.try_claim_payout(&issuer, &asset_id),
        Err(Ok(Error::NothingToClaim))
    );
}

#[test]
fn rental_yield_stays_with_previous_holder_after_transfer() {
    let env = Env::default();
    let (client, token_admin, issuer, asset_id) = setup(&env);
    let tenant = Address::generate(&env);
    let buyer = Address::generate(&env);
    token_admin.mint(&tenant, &1_000);
    client.deposit_payout(&tenant, &asset_id, &1_000);

    client.transfer(&issuer, &buyer, &asset_id, &TOTAL);

    assert_eq!(client.get_pending_payout(&asset_id, &issuer), 1_000);
    assert_eq!(client.get_pending_payout(&asset_id, &buyer), 0);
}

#[test]
fn rental_yield_rejects_invalid_deposits() {
    let env = Env::default();
    let (client, _, _, asset_id) = setup(&env);
    let tenant = Address::generate(&env);
    assert_eq!(
        client.try_deposit_payout(&tenant, &asset_id, &0),
        Err(Ok(Error::InvalidAmount))
    );
    assert_eq!(
        client.try_deposit_payout(&tenant, &99, &10),
        Err(Ok(Error::AssetNotFound))
    );
}

// ---------------------------------------------------------------------------
// fuzz tests
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(32))]

    /// Holders never get more than was deposited, and lose at most 1 unit
    /// each to rounding.
    #[test]
    fn fuzz_rental_yield_is_solvent(split in 1i128..TOTAL, amount in 1i128..1_000_000) {
        let env = Env::default();
        let (client, token_admin, issuer, asset_id) = setup(&env);
        let tenant = Address::generate(&env);
        let holder = Address::generate(&env);
        client.transfer(&issuer, &holder, &asset_id, &split);
        token_admin.mint(&tenant, &amount);
        client.deposit_payout(&tenant, &asset_id, &amount);

        let a = client.get_pending_payout(&asset_id, &issuer);
        let b = client.get_pending_payout(&asset_id, &holder);
        prop_assert!(a + b <= amount);
        prop_assert!(amount - (a + b) <= 2);
    }
}
