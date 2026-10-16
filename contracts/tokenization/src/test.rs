#![cfg(test)]
#![allow(clippy::unwrap_used)]

use super::*;

use soroban_sdk::{symbol_short, testutils::Address as _, Address, Env, String};

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

/// Count events published under the given topic symbols.
fn count_events(env: &Env, topics: &[soroban_sdk::Symbol]) -> usize {
    env.events()
        .all()
        .iter()
        .filter(|event| {
            topics.len() <= event.topics.len()
                && topics
                    .iter()
                    .enumerate()
                    .all(|(i, expected)| event.topics[i] == expected.clone().into())
        })
        .count()
}

// ---------------------------------------------------------------------------
// Initialization and administration
// ---------------------------------------------------------------------------

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
fn test_transfer_admin_rotates_admin() {
    let (env, client, _admin, _issuer) = setup!();
    let next = Address::generate(&env);

    client.transfer_admin(&next);

    assert_eq!(client.admin(), next);
}

#[test]
fn test_pause_blocks_state_changes_and_resumes() {
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

// ---------------------------------------------------------------------------
// Input validation (#1759)
// ---------------------------------------------------------------------------

#[test]
fn test_register_asset_rejects_empty_asset_id() {
    let (env, client, _admin, issuer) = setup!();

    assert_eq!(
        client.try_register_asset(
            &sid(&env, ""),
            &issuer,
            &1_000,
            &6,
            &sid(&env, "uri")
        ),
        Err(Ok(Error::InvalidAssetId))
    );
}

#[test]
fn test_register_asset_rejects_short_asset_id() {
    let (env, client, _admin, issuer) = setup!();

    assert_eq!(
        client.try_register_asset(
            &sid(&env, "ab"),
            &issuer,
            &1_000,
            &6,
            &sid(&env, "uri")
        ),
        Err(Ok(Error::InvalidAssetId))
    );
}

#[test]
fn test_register_asset_rejects_characters_outside_the_alphabet() {
    let (env, client, _admin, issuer) = setup!();

    assert_eq!(
        client.try_register_asset(
            &sid(&env, "HOSP/001"),
            &issuer,
            &1_000,
            &6,
            &sid(&env, "uri")
        ),
        Err(Ok(Error::InvalidAssetId))
    );
}

#[test]
fn test_register_asset_rejects_decimals_above_the_sac_cap() {
    let (env, client, _admin, issuer) = setup!();

    assert_eq!(
        client.try_register_asset(
            &sid(&env, "HOSP-1"),
            &issuer,
            &1_000,
            &19,
            &sid(&env, "uri")
        ),
        Err(Ok(Error::DecimalsOutOfRange))
    );
}

#[test]
fn test_register_asset_rejects_empty_metadata_uri() {
    let (env, client, _admin, issuer) = setup!();

    assert_eq!(
        client.try_register_asset(
            &sid(&env, "HOSP-1"),
            &issuer,
            &1_000,
            &6,
            &sid(&env, "")
        ),
        Err(Ok(Error::InvalidMetadataUri))
    );
}

#[test]
fn test_register_asset_rejects_zero_supply_cap() {
    let (env, client, _admin, issuer) = setup!();

    assert_eq!(
        client.try_register_asset(
            &sid(&env, "HOSP-1"),
            &issuer,
            &0,
            &6,
            &sid(&env, "uri")
        ),
        Err(Ok(Error::ZeroAmount))
    );
}

#[test]
fn test_validation_runs_before_authorization_is_consumed() {
    // A malformed call must be rejected on the argument, not on auth, so the
    // error a caller sees cannot be used to probe the admin set.
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let issuer = Address::generate(&env);
    let contract_id = env.register_contract(None, TokenizationContract);
    let client = TokenizationContractClient::new(&env, &contract_id);
    client.initialize(&admin, &issuer);

    assert_eq!(
        client.try_register_asset(
            &sid(&env, ""),
            &issuer,
            &1_000,
            &6,
            &sid(&env, "uri")
        ),
        Err(Ok(Error::InvalidAssetId))
    );
    assert_eq!(client.asset_count(), 0);
}

#[test]
fn test_mint_rejects_zero_amount() {
    let (env, client, _admin, issuer) = setup!();
    let id = sid(&env, "HOSP-1");
    let holder = Address::generate(&env);

    client.register_asset(&id, &issuer, &1_000, &6, &sid(&env, "uri"));

    assert_eq!(
        client.try_mint(&id, &holder, &0),
        Err(Ok(Error::ZeroAmount))
    );
}

#[test]
fn test_transfer_rejects_self_transfer() {
    let (env, client, _admin, issuer) = setup!();
    let id = sid(&env, "HOSP-1");
    let alice = Address::generate(&env);

    client.register_asset(&id, &issuer, &1_000, &6, &sid(&env, "uri"));
    client.mint(&id, &alice, &100);

    assert_eq!(
        client.try_transfer(&id, &alice, &alice, &10),
        Err(Ok(Error::TransferToSelf))
    );
    assert_eq!(client.balance_of(&id, &alice), 100);
}

// ---------------------------------------------------------------------------
// Typed custom errors (#1760)
// ---------------------------------------------------------------------------

#[test]
fn test_zero_amount_and_short_balance_are_distinguishable() {
    let (env, client, _admin, issuer) = setup!();
    let id = sid(&env, "HOSP-1");
    let alice = Address::generate(&env);

    client.register_asset(&id, &issuer, &1_000, &6, &sid(&env, "uri"));
    client.mint(&id, &alice, &100);

    // Same operation, two different root causes, two different discriminants.
    assert_eq!(
        client.try_transfer(&id, &alice, &Address::generate(&env), &0),
        Err(Ok(Error::ZeroAmount))
    );
    assert_eq!(
        client.try_transfer(&id, &alice, &Address::generate(&env), &101),
        Err(Ok(Error::InsufficientBalance))
    );
}

#[test]
fn test_supply_cap_exceeded_is_its_own_error() {
    let (env, client, _admin, issuer) = setup!();
    let id = sid(&env, "HOSP-1");
    let alice = Address::generate(&env);

    client.register_asset(&id, &issuer, &1_000, &6, &sid(&env, "uri"));
    client.mint(&id, &alice, &900);

    assert_eq!(
        client.try_mint(&id, &alice, &101),
        Err(Ok(Error::SupplyCapExceeded))
    );
    assert_eq!(client.total_supply(&id), 900);
    assert_eq!(client.balance_of(&id, &alice), 900);
}

#[test]
fn test_mint_up_to_the_cap_is_allowed_and_beyond_is_not() {
    let (env, client, _admin, issuer) = setup!();
    let id = sid(&env, "HOSP-1");
    let alice = Address::generate(&env);

    client.register_asset(&id, &issuer, &1_000, &6, &sid(&env, "uri"));
    client.mint(&id, &alice, &1_000);

    assert_eq!(client.total_supply(&id), 1_000);
    assert_eq!(client.supply_cap(&id), 1_000);
    assert_eq!(
        client.try_mint(&id, &alice, &1),
        Err(Ok(Error::SupplyCapExceeded))
    );
}

#[test]
fn test_burn_frees_headroom_under_the_cap() {
    let (env, client, _admin, issuer) = setup!();
    let id = sid(&env, "HOSP-1");
    let alice = Address::generate(&env);

    client.register_asset(&id, &issuer, &1_000, &6, &sid(&env, "uri"));
    client.mint(&id, &alice, &1_000);
    client.burn(&id, &alice, &400);
    client.mint(&id, &alice, &400);

    assert_eq!(client.total_supply(&id), 1_000);
}

#[test]
fn test_error_variants_map_to_stable_discriminants() {
    // These numbers are part of the contract interface: clients match on them.
    assert_eq!(Error::InvalidAssetId as u32, 6);
    assert_eq!(Error::DecimalsOutOfRange as u32, 7);
    assert_eq!(Error::InvalidMetadataUri as u32, 8);
    assert_eq!(Error::ZeroAmount as u32, 9);
    assert_eq!(Error::TransferToSelf as u32, 10);
    assert_eq!(Error::InsufficientBalance as u32, 11);
    assert_eq!(Error::SupplyCapExceeded as u32, 12);
    assert_eq!(Error::ArithmeticOverflow as u32, 13);
    assert_eq!(Error::ArithmeticUnderflow as u32, 14);
}

// ---------------------------------------------------------------------------
// Events (#1757)
// ---------------------------------------------------------------------------

#[test]
fn test_initialize_emits_initialized_event() {
    let (env, client, admin, _issuer) = setup!();

    assert_eq!(
        count_events(&env, &[symbol_short!("init")]),
        1
    );
    assert_eq!(client.admin(), admin);
}

#[test]
fn test_admin_and_treasury_changes_emit_events() {
    let (env, client, _admin, _issuer) = setup!();
    let next_admin = Address::generate(&env);
    let next_treasury = Address::generate(&env);

    client.transfer_admin(&next_admin);
    client.set_treasury_address(&next_treasury);

    assert_eq!(
        count_events(&env, &[symbol_short!("adm_xfer")]),
        1
    );
    assert_eq!(count_events(&env, &[symbol_short!("treasury")]), 1);
}

#[test]
fn test_pause_and_unpause_each_emit_one_event() {
    let (env, client, _admin, _issuer) = setup!();

    client.pause();
    assert_eq!(count_events(&env, &[symbol_short!("pause")]), 1);

    client.unpause();
    assert_eq!(count_events(&env, &[symbol_short!("pause")]), 2);
}

#[test]
fn test_asset_lifecycle_emits_one_event_per_state_change() {
    let (env, client, _admin, issuer) = setup!();
    let id = sid(&env, "HOSP-1");
    let alice = Address::generate(&env);
    let bob = Address::generate(&env);

    client.register_asset(&id, &issuer, &1_000, &6, &sid(&env, "uri"));
    client.mint(&id, &alice, &100);
    client.transfer(&id, &alice, &bob, &40);
    client.burn(&id, &alice, &10);

    assert_eq!(
        count_events(&env, &[symbol_short!("asset"), symbol_short!("reg")]),
        1
    );
    assert_eq!(
        count_events(&env, &[symbol_short!("asset"), symbol_short!("mint")]),
        1
    );
    assert_eq!(
        count_events(&env, &[symbol_short!("asset"), symbol_short!("xfer")]),
        1
    );
    assert_eq!(
        count_events(&env, &[symbol_short!("asset"), symbol_short!("burn")]),
        1
    );
}

#[test]
fn test_rejected_calls_emit_no_events() {
    let (env, client, _admin, issuer) = setup!();
    let id = sid(&env, "HOSP-1");
    let alice = Address::generate(&env);

    client.register_asset(&id, &issuer, &1_000, &6, &sid(&env, "uri"));
    let before = env.events().all().len();

    assert!(client.try_mint(&id, &alice, &0).is_err());
    assert!(client.try_transfer(&id, &alice, &alice, &1).is_err());

    assert_eq!(env.events().all().len(), before);
}

// ---------------------------------------------------------------------------
// Behaviour
// ---------------------------------------------------------------------------

#[test]
fn test_register_asset_rejects_duplicate_id() {
    let (env, client, _admin, issuer) = setup!();
    let id = sid(&env, "HOSP-1");

    client.register_asset(&id, &issuer, &1_000, &6, &sid(&env, "uri"));

    assert_eq!(
        client.try_register_asset(&id, &issuer, &1_000, &6, &sid(&env, "uri")),
        Err(Ok(Error::AssetAlreadyExists))
    );
    assert_eq!(client.asset_count(), 1);
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
    // A transfer must not move supply.
    assert_eq!(client.total_supply(&id), 100_000);
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
    assert_eq!(asset.total_supply, 0);
    assert_eq!(asset.supply_cap, 500);
    assert_eq!(asset.decimals, 2);
}

#[test]
fn test_queries_fail_for_unknown_asset() {
    let (env, client, _admin, _issuer) = setup!();
    let id = sid(&env, "MISSING");

    assert_eq!(client.try_get_asset(&id), Err(Ok(Error::AssetNotFound)));
    assert_eq!(client.try_total_supply(&id), Err(Ok(Error::AssetNotFound)));
    assert_eq!(client.try_supply_cap(&id), Err(Ok(Error::AssetNotFound)));
}

#[test]
fn test_balance_of_unknown_holder_is_zero() {
    let (env, client, _admin, issuer) = setup!();
    let id = sid(&env, "HOSP-1");

    client.register_asset(&id, &issuer, &1_000, &6, &sid(&env, "uri"));

    assert_eq!(client.balance_of(&id, &Address::generate(&env)), 0);
}