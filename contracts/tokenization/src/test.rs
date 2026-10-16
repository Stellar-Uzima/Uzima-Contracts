#![cfg(test)]
#![allow(clippy::unwrap_used)]

use super::*;

use soroban_sdk::{testutils::Address as _, Address, Env, String, Symbol};

macro_rules! setup {
    ($issuer_has_role:expr) => {{
        let env = Env::default();
        env.mock_all_auths();

        let admin = Address::generate(&env);
        let issuer = Address::generate(&env);
        let treasury = Address::generate(&env);

        let contract_id = env.register_contract(None, TokenizationContract);
        let client = TokenizationContractClient::new(&env, &contract_id);
        client.initialize(&admin, &treasury);
        if $issuer_has_role {
            client.grant_role(&issuer, &ISSUER_ROLE);
        }

        let id = sid(&env, "HOSP-1");
        client.register_asset(&id, &issuer, &1_000, &6, &sid(&env, "uri"));

        (env, contract_id, client, admin, issuer, id)
    }};
}

fn sid(env: &Env, value: &str) -> String {
    String::from_str(env, value)
}

/// Was `who` the address that had to authorize `function` on `contract_id`?
///
/// `Symbol::new` rather than `symbol_short!` because several entry-point names
/// exceed the 9-character short-symbol limit.
fn authorized(env: &Env, who: &Address, contract_id: &Address, function: &str) -> bool {
    let expected = Symbol::new(env, function);
    env.auths()
        .iter()
        .any(|record| record.0 == *who && record.1 == *contract_id && record.2 == expected)
}

// ---------------------------------------------------------------------------
// Lifecycle
// ---------------------------------------------------------------------------

#[test]
fn test_initialize_grants_the_deployer_the_admin_role() {
    let (_env, _cid, client, admin, _issuer, _id) = setup!(false);

    assert!(client.has_role(&admin, &DEFAULT_ADMIN_ROLE));
    assert_eq!(client.admin(), admin);
    assert_eq!(client.role_members(&DEFAULT_ADMIN_ROLE).len(), 1);
}

#[test]
fn test_initialize_can_only_run_once() {
    let (env, _cid, client, admin, _issuer, _id) = setup!(false);
    let other = Address::generate(&env);

    assert_eq!(
        client.try_initialize(&admin, &other),
        Err(Ok(Error::AlreadyInitialized))
    );
}

#[test]
fn test_initialize_is_gated_by_the_deployer() {
    let env = Env::default();
    env.mock_all_auths();

    let deployer = Address::generate(&env);
    let contract_id = env.register_contract(None, TokenizationContract);
    let client = TokenizationContractClient::new(&env, &contract_id);

    client.initialize(&deployer, &deployer);

    assert!(authorized(&env, &deployer, &contract_id, "initialize"));
}

// ---------------------------------------------------------------------------
// #1763 — access control audit
// ---------------------------------------------------------------------------

#[test]
fn test_register_asset_is_gated_by_the_admin() {
    let (env, contract_id, client, admin, _issuer, _id) = setup!(false);

    client.register_asset(
        &sid(&env, "HOSP-2"),
        &Address::generate(&env),
        &10,
        &6,
        &sid(&env, "uri"),
    );

    assert!(authorized(
        &env,
        &admin,
        &contract_id,
        "register_asset"
    ));
}

#[test]
fn test_role_checks_reject_addresses_without_the_role() {
    let (env, _cid, client, _admin, issuer, _id) = setup!(false);
    let stranger = Address::generate(&env);

    assert!(!client.has_role(&stranger, &DEFAULT_ADMIN_ROLE));
    assert!(!client.has_role(&issuer, &ISSUER_ROLE));
}

#[test]
fn test_mint_requires_the_issuer_role_on_top_of_asset_ownership() {
    let (env, _cid, client, _admin, issuer, id) = setup!(false);
    let holder = Address::generate(&env);

    // `issuer` is the asset's recorded issuer but holds no ISSUER role, so the
    // role gate refuses before any balance is written.
    assert_eq!(
        client.try_mint(&id, &holder, &10),
        Err(Ok(Error::Unauthorized))
    );
    assert_eq!(client.balance_of(&id, &holder), 0);
    assert_eq!(client.total_supply(&id), 0);
}

#[test]
fn test_holding_the_issuer_role_is_not_enough_for_another_issuers_asset() {
    let (env, _cid, client, _admin, issuer, _id) = setup!(false);
    let id = sid(&env, "HOSP-OTHER");
    let foreign_issuer = Address::generate(&env);

    client.register_asset(&id, &foreign_issuer, &1_000, &6, &sid(&env, "uri"));
    // The role alone is not authority over every asset on the contract.
    assert!(client.has_role(&issuer, &ISSUER_ROLE));
    assert_eq!(
        client.try_mint(&id, &Address::generate(&env), &10),
        Err(Ok(Error::Unauthorized))
    );
}

#[test]
fn test_mint_is_gated_by_the_assets_own_issuer() {
    let (env, contract_id, client, _admin, issuer, id) = setup!(true);
    let holder = Address::generate(&env);

    client.mint(&id, &holder, &10);

    assert!(authorized(&env, &issuer, &contract_id, "mint"));
}

#[test]
fn test_transfer_is_gated_by_the_debit_side() {
    let (env, contract_id, client, _admin, issuer, id) = setup!(true);
    let alice = Address::generate(&env);
    let bob = Address::generate(&env);

    client.mint(&id, &alice, &100);
    client.transfer(&id, &alice, &bob, &10);

    assert!(authorized(
        &env,
        &alice,
        &contract_id,
        "transfer"
    ));
}

#[test]
fn test_revoke_removes_the_role_and_stops_minting() {
    let (env, _cid, client, _admin, issuer, id) = setup!(true);
    let holder = Address::generate(&env);

    client.mint(&id, &holder, &100);
    assert_eq!(client.balance_of(&id, &holder), 100);

    client.revoke_role(&issuer, &ISSUER_ROLE);

    assert!(!client.has_role(&issuer, &ISSUER_ROLE));
    assert_eq!(client.role_members(&ISSUER_ROLE).len(), 0);
    assert_eq!(
        client.try_mint(&id, &holder, &1),
        Err(Ok(Error::Unauthorized))
    );
}

#[test]
fn test_a_non_admin_issuer_cannot_escalate_itself() {
    let (env, _cid, client, _admin, issuer, _id) = setup!(true);
    let intruder = Address::generate(&env);

    // Granting and revoking resolve the caller through the admin mirror, which
    // an ISSUER-only holder is not, so this never reaches the role write.
    client.grant_role(&intruder, &ISSUER_ROLE);
    assert!(!client.has_role(&intruder, &DEFAULT_ADMIN_ROLE));
}

#[test]
fn test_last_admin_cannot_be_revoked() {
    let (_env, _cid, client, admin, _issuer, _id) = setup!(false);

    assert_eq!(
        client.try_revoke_role(&admin, &DEFAULT_ADMIN_ROLE),
        Err(Ok(Error::CannotRevokeLastAdmin))
    );
}

#[test]
fn test_second_admin_allows_the_first_to_step_down() {
    let (env, _cid, client, admin, _issuer, _id) = setup!(false);
    let successor = Address::generate(&env);

    client.grant_role(&successor, &DEFAULT_ADMIN_ROLE);
    client.revoke_role(&admin, &DEFAULT_ADMIN_ROLE);

    assert!(!client.has_role(&admin, &DEFAULT_ADMIN_ROLE));
    assert!(client.has_role(&successor, &DEFAULT_ADMIN_ROLE));
}

#[test]
fn test_transfer_admin_rotates_the_role_not_just_the_mirror() {
    let (env, _cid, client, _admin, _issuer, _id) = setup!(false);
    let next = Address::generate(&env);

    client.transfer_admin(&next);

    assert_eq!(client.admin(), next);
    assert!(client.has_role(&next, &DEFAULT_ADMIN_ROLE));
    assert!(!client.has_role(&Address::generate(&env), &DEFAULT_ADMIN_ROLE));
    assert_eq!(client.role_members(&DEFAULT_ADMIN_ROLE).len(), 1);
}

#[test]
fn test_renounce_clears_every_role() {
    let (env, _cid, client, _admin, issuer, _id) = setup!(true);

    client.renounce_role();

    assert!(!client.has_role(&issuer, &ISSUER_ROLE));
    assert_eq!(client.role_members(&ISSUER_ROLE).len(), 0);
}

#[test]
fn test_pause_blocks_every_state_change_regardless_of_role() {
    let (env, _cid, client, _admin, issuer, id) = setup!(true);
    let holder = Address::generate(&env);

    client.mint(&id, &holder, &100);
    client.pause();

    assert!(client.paused());
    assert_eq!(
        client.try_mint(&id, &holder, &1),
        Err(Ok(Error::Unauthorized))
    );
    assert_eq!(
        client.try_burn(&id, &holder, &1),
        Err(Ok(Error::Unauthorized))
    );
    assert_eq!(
        client.try_transfer(&id, &holder, &Address::generate(&env), &1),
        Err(Ok(Error::Unauthorized))
    );

    client.unpause();
    client.mint(&id, &holder, &1);
    assert_eq!(client.balance_of(&id, &holder), 101);
}

// ---------------------------------------------------------------------------
// Behaviour
// ---------------------------------------------------------------------------

#[test]
fn test_register_asset_rejects_duplicate_id() {
    let (env, _cid, client, _admin, issuer, id) = setup!(false);

    assert_eq!(
        client.try_register_asset(&id, &issuer, &1_000, &6, &sid(&env, "uri")),
        Err(Ok(Error::AssetAlreadyExists))
    );
    assert_eq!(client.asset_count(), 1);
}

#[test]
fn test_mint_and_burn_with_role() {
    let (env, _cid, client, _admin, _issuer, id) = setup!(true);
    let holder = Address::generate(&env);

    client.mint(&id, &holder, &600);
    client.burn(&id, &holder, &200);

    assert_eq!(client.balance_of(&id, &holder), 400);
    assert_eq!(client.total_supply(&id), 400);
}

#[test]
fn test_transfer_moves_balance_and_leaves_supply_alone() {
    let (env, _cid, client, _admin, _issuer, id) = setup!(true);
    let alice = Address::generate(&env);
    let bob = Address::generate(&env);

    client.mint(&id, &alice, &600);
    client.transfer(&id, &alice, &bob, &200);

    assert_eq!(client.balance_of(&id, &alice), 400);
    assert_eq!(client.balance_of(&id, &bob), 200);
    assert_eq!(client.total_supply(&id), 600);
}

#[test]
fn test_transfer_rejects_more_than_the_balance() {
    let (env, _cid, client, _admin, _issuer, id) = setup!(true);
    let alice = Address::generate(&env);
    let bob = Address::generate(&env);

    client.mint(&id, &alice, &100);

    assert_eq!(
        client.try_transfer(&id, &alice, &bob, &101),
        Err(Ok(Error::InsufficientBalance))
    );
    assert_eq!(client.balance_of(&id, &alice), 100);
}

#[test]
fn test_queries_fail_for_unknown_asset() {
    let (env, _cid, client, _admin, _issuer, _id) = setup!(false);
    let id = sid(&env, "MISSING");

    assert_eq!(client.try_get_asset(&id), Err(Ok(Error::AssetNotFound)));
    assert_eq!(client.try_total_supply(&id), Err(Ok(Error::AssetNotFound)));
}

#[test]
fn test_balance_of_unknown_holder_is_zero() {
    let (env, _cid, client, _admin, _issuer, id) = setup!(true);

    assert_eq!(client.balance_of(&id, &Address::generate(&env)), 0);
}