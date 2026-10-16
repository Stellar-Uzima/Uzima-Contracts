#![cfg(test)]
#![allow(clippy::unwrap_used)]

use super::*;

use soroban_sdk::{testutils::Address as _, Address, Env, String};

/// Build a fresh registry plus an initialized client bound to a local `Env`.
///
/// This has to be a macro rather than a function: the generated client borrows
/// the `Env`, so it cannot outlive a returned `Env`.
macro_rules! setup {
    () => {{
        let env = Env::default();
        env.mock_all_auths();

        let admin = Address::generate(&env);
        let issuer = Address::generate(&env);
        let treasury = Address::generate(&env);

        let contract_id = env.register_contract(None, TokenizationContract);
        let client = TokenizationContractClient::new(&env, &contract_id);
        client.initialize(&admin, &treasury);

        (env, client, admin, issuer)
    }};
}

fn sid(env: &Env, value: &str) -> String {
    String::from_str(env, value)
}

#[test]
fn test_initialize_sets_admin_and_treasury() {
    let (env, client, admin, _issuer) = setup!();
    let treasury = Address::generate(&env);

    client.set_treasury_address(&treasury);

    assert_eq!(client.admin(), admin);
    assert_eq!(client.treasury(), treasury);
    assert!(!client.paused());
}

#[test]
fn test_initialize_can_only_run_once() {
    let (env, client, admin, _issuer) = setup!();
    let other = Address::generate(&env);

    assert_eq!(
        client.try_initialize(&admin, &other),
        Err(Ok(Error::AlreadyInitialized))
    );
}

#[test]
fn test_register_asset_increments_count() {
    let (env, client, _admin, issuer) = setup!();
    let id = sid(&env, "HOSP-1");

    client.register_asset(&id, &issuer, &1_000_000, &6, &sid(&env, "ipfs://meta"));

    assert_eq!(client.asset_count(), 1);
}

#[test]
fn test_register_asset_rejects_duplicate_id() {
    let (env, client, _admin, issuer) = setup!();
    let id = sid(&env, "HOSP-1");

    client.register_asset(&id, &issuer, &1_000, &6, &sid(&env, "uri"));

    assert_eq!(
        client.try_register_asset(&id, &issuer, &1_000, &6, &sid(&env, "uri")),
        Err(Ok(Error::AssetAlreadyExists))
    );
}

#[test]
fn test_mint_credits_holder_and_supply() {
    let (env, client, _admin, issuer) = setup!();
    let id = sid(&env, "HOSP-1");
    let holder = Address::generate(&env);

    client.register_asset(&id, &issuer, &1_000_000, &6, &sid(&env, "uri"));
    client.mint(&id, &holder, &250_000);

    assert_eq!(client.balance_of(&id, &holder), 250_000);
    assert_eq!(client.total_supply(&id), 250_000);
}

#[test]
fn test_transfer_moves_balance_between_holders() {
    let (env, client, _admin, issuer) = setup!();
    let id = sid(&env, "HOSP-1");
    let alice = Address::generate(&env);
    let bob = Address::generate(&env);

    client.register_asset(&id, &issuer, &1_000_000, &6, &sid(&env, "uri"));
    client.mint(&id, &alice, &100_000);
    client.transfer(&id, &alice, &bob, &40_000);

    assert_eq!(client.balance_of(&id, &alice), 60_000);
    assert_eq!(client.balance_of(&id, &bob), 40_000);
}

#[test]
fn test_burn_reduces_supply() {
    let (env, client, _admin, issuer) = setup!();
    let id = sid(&env, "HOSP-1");
    let holder = Address::generate(&env);

    client.register_asset(&id, &issuer, &1_000_000, &6, &sid(&env, "uri"));
    client.mint(&id, &holder, &100_000);
    client.burn(&id, &holder, &30_000);

    assert_eq!(client.balance_of(&id, &holder), 70_000);
    assert_eq!(client.total_supply(&id), 70_000);
}

#[test]
fn test_get_asset_returns_record() {
    let (env, client, _admin, issuer) = setup!();
    let id = sid(&env, "HOSP-1");

    client.register_asset(&id, &issuer, &500, &2, &sid(&env, "ipfs://x"));

    let asset = client.get_asset(&id);
    assert_eq!(asset.asset_id, id);
    assert_eq!(asset.issuer, issuer);
    assert_eq!(asset.total_supply, 500);
    assert_eq!(asset.decimals, 2);
}

#[test]
fn test_queries_fail_for_unknown_asset() {
    let (env, client, _admin, _issuer) = setup!();
    let id = sid(&env, "MISSING");

    assert_eq!(client.try_get_asset(&id), Err(Ok(Error::AssetNotFound)));
    assert_eq!(client.try_total_supply(&id), Err(Ok(Error::AssetNotFound)));
}

#[test]
fn test_pause_blocks_state_changes() {
    let (env, client, _admin, issuer) = setup!();
    let id = sid(&env, "HOSP-1");
    let holder = Address::generate(&env);

    client.register_asset(&id, &issuer, &1_000, &6, &sid(&env, "uri"));
    client.pause();

    assert!(client.paused());
    assert_eq!(
        client.try_mint(&id, &holder, &1),
        Err(Ok(Error::Unauthorized))
    );

    client.unpause();
    client.mint(&id, &holder, &1);
    assert_eq!(client.balance_of(&id, &holder), 1);
}

#[test]
fn test_transfer_admin_rotates_admin() {
    let (env, client, _admin, _issuer) = setup!();
    let next = Address::generate(&env);

    client.transfer_admin(&next);

    assert_eq!(client.admin(), next);
}

#[test]
fn test_balance_of_unknown_holder_is_zero() {
    let (env, client, _admin, issuer) = setup!();
    let id = sid(&env, "HOSP-1");

    client.register_asset(&id, &issuer, &1_000, &6, &sid(&env, "uri"));

    assert_eq!(client.balance_of(&id, &Address::generate(&env)), 0);
}