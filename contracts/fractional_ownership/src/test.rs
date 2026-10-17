use super::*;
use soroban_sdk::{
    symbol_short,
    testutils::{Address as _, Events as _},
    token, Address, Env, Symbol, TryFromVal,
};

const TOTAL: i128 = 1_000;

struct Fixture<'a> {
    client: FractionalOwnershipClient<'a>,
    contract_id: Address,
    admin: Address,
    token: token::Client<'a>,
    token_admin: token::StellarAssetClient<'a>,
}

fn setup<'a>(env: &'a Env) -> Fixture<'a> {
    env.mock_all_auths();
    let admin = Address::generate(env);
    let sac = env.register_stellar_asset_contract_v2(admin.clone());
    let token_addr = sac.address();
    let contract_id = env.register_contract(None, FractionalOwnership);
    let client = FractionalOwnershipClient::new(env, &contract_id);
    client.initialize(&admin, &token_addr);
    Fixture {
        client,
        contract_id,
        admin,
        token: token::Client::new(env, &token_addr),
        token_admin: token::StellarAssetClient::new(env, &token_addr),
    }
}

/// Setup plus one asset with `TOTAL` shares held by a fresh issuer.
fn setup_asset<'a>(env: &'a Env) -> (Fixture<'a>, Address, u64) {
    let f = setup(env);
    let issuer = Address::generate(env);
    let asset_id = f.client.create_asset(&f.admin, &issuer, &TOTAL);
    (f, issuer, asset_id)
}

/// True if the most recent invocation emitted an event from `contract` whose
/// first topic is `name`.
fn has_event(env: &Env, contract: &Address, name: Symbol) -> bool {
    env.events().all().iter().any(|(c, topics, _)| {
        c == *contract
            && topics
                .first()
                .and_then(|v| Symbol::try_from_val(env, &v).ok())
                .map(|s| s == name)
                .unwrap_or(false)
    })
}

// ---------------------------------------------------------------------------
// initialize
// ---------------------------------------------------------------------------

#[test]
fn initialize_stores_config() {
    let env = Env::default();
    let f = setup(&env);
    assert_eq!(f.client.get_admin(), f.admin);
    assert_eq!(f.client.get_payment_token(), f.token.address);
    assert_eq!(f.client.get_asset_count(), 0);
}

#[test]
fn initialize_emits_event() {
    let env = Env::default();
    env.mock_all_auths();
    let admin = Address::generate(&env);
    let token = Address::generate(&env);
    let id = env.register_contract(None, FractionalOwnership);
    let client = FractionalOwnershipClient::new(&env, &id);
    client.initialize(&admin, &token);
    assert!(has_event(&env, &id, symbol_short!("init")));
}

#[test]
fn initialize_twice_fails() {
    let env = Env::default();
    let f = setup(&env);
    let other = Address::generate(&env);
    assert_eq!(
        f.client.try_initialize(&other, &f.token.address),
        Err(Ok(Error::AlreadyInitialized))
    );
    // The original admin is untouched.
    assert_eq!(f.client.get_admin(), f.admin);
}

#[test]
fn views_fail_before_initialize() {
    let env = Env::default();
    env.mock_all_auths();
    let id = env.register_contract(None, FractionalOwnership);
    let client = FractionalOwnershipClient::new(&env, &id);
    assert_eq!(client.try_get_admin(), Err(Ok(Error::NotInitialized)));
    assert_eq!(
        client.try_get_payment_token(),
        Err(Ok(Error::NotInitialized))
    );
    assert_eq!(client.get_asset_count(), 0);
}

#[test]
fn create_asset_before_initialize_fails() {
    let env = Env::default();
    env.mock_all_auths();
    let id = env.register_contract(None, FractionalOwnership);
    let client = FractionalOwnershipClient::new(&env, &id);
    let who = Address::generate(&env);
    assert_eq!(
        client.try_create_asset(&who, &who, &TOTAL),
        Err(Ok(Error::NotInitialized))
    );
}

#[test]
#[should_panic(expected = "Auth")]
fn initialize_requires_admin_auth() {
    // No mock_all_auths: the missing authorization must trap.
    let env = Env::default();
    let admin = Address::generate(&env);
    let token = Address::generate(&env);
    let id = env.register_contract(None, FractionalOwnership);
    let client = FractionalOwnershipClient::new(&env, &id);
    client.initialize(&admin, &token);
}

// ---------------------------------------------------------------------------
// create_asset
// ---------------------------------------------------------------------------

#[test]
fn create_asset_mints_all_shares_to_issuer() {
    let env = Env::default();
    let (f, issuer, asset_id) = setup_asset(&env);
    assert_eq!(asset_id, 1);
    assert_eq!(f.client.get_asset_count(), 1);
    assert_eq!(f.client.get_total_shares(&asset_id), TOTAL);
    assert_eq!(f.client.get_balance(&asset_id, &issuer), TOTAL);
}

#[test]
fn create_asset_emits_event() {
    let env = Env::default();
    let f = setup(&env);
    let issuer = Address::generate(&env);
    f.client.create_asset(&f.admin, &issuer, &TOTAL);
    assert!(has_event(&env, &f.contract_id, symbol_short!("created")));
}

#[test]
fn create_asset_ids_are_sequential_and_independent() {
    let env = Env::default();
    let (f, issuer, first) = setup_asset(&env);
    let other_issuer = Address::generate(&env);
    let second = f.client.create_asset(&f.admin, &other_issuer, &250);
    assert_eq!((first, second), (1, 2));
    assert_eq!(f.client.get_asset_count(), 2);
    assert_eq!(f.client.get_total_shares(&second), 250);
    assert_eq!(f.client.get_balance(&second, &other_issuer), 250);
    assert_eq!(f.client.get_balance(&second, &issuer), 0);
    assert_eq!(f.client.get_balance(&first, &other_issuer), 0);
}

#[test]
fn create_asset_rejects_non_admin() {
    let env = Env::default();
    let f = setup(&env);
    let mallory = Address::generate(&env);
    assert_eq!(
        f.client.try_create_asset(&mallory, &mallory, &TOTAL),
        Err(Ok(Error::Unauthorized))
    );
    assert_eq!(f.client.get_asset_count(), 0);
}

#[test]
fn create_asset_rejects_non_positive_supply() {
    let env = Env::default();
    let f = setup(&env);
    let issuer = Address::generate(&env);
    assert_eq!(
        f.client.try_create_asset(&f.admin, &issuer, &0),
        Err(Ok(Error::InvalidAmount))
    );
    assert_eq!(
        f.client.try_create_asset(&f.admin, &issuer, &-5),
        Err(Ok(Error::InvalidAmount))
    );
    assert_eq!(f.client.get_asset_count(), 0);
}

#[test]
fn unknown_asset_queries() {
    let env = Env::default();
    let f = setup(&env);
    let who = Address::generate(&env);
    assert_eq!(
        f.client.try_get_total_shares(&9),
        Err(Ok(Error::AssetNotFound))
    );
    assert_eq!(f.client.get_balance(&9, &who), 0);
    assert_eq!(f.client.get_listing(&9, &who), None);
    assert_eq!(f.client.get_payout_index(&9), 0);
}

// ---------------------------------------------------------------------------
// transfer
// ---------------------------------------------------------------------------

#[test]
fn transfer_moves_shares() {
    let env = Env::default();
    let (f, issuer, id) = setup_asset(&env);
    let bob = Address::generate(&env);
    f.client.transfer(&issuer, &bob, &id, &300);
    assert_eq!(f.client.get_balance(&id, &issuer), 700);
    assert_eq!(f.client.get_balance(&id, &bob), 300);
    assert_eq!(f.client.get_total_shares(&id), TOTAL);
    assert!(has_event(&env, &f.contract_id, symbol_short!("transfer")));
}

#[test]
fn transfer_entire_balance() {
    let env = Env::default();
    let (f, issuer, id) = setup_asset(&env);
    let bob = Address::generate(&env);
    f.client.transfer(&issuer, &bob, &id, &TOTAL);
    assert_eq!(f.client.get_balance(&id, &issuer), 0);
    assert_eq!(f.client.get_balance(&id, &bob), TOTAL);
}

#[test]
fn transfer_rejects_bad_input() {
    let env = Env::default();
    let (f, issuer, id) = setup_asset(&env);
    let bob = Address::generate(&env);
    assert_eq!(
        f.client.try_transfer(&issuer, &bob, &id, &0),
        Err(Ok(Error::InvalidAmount))
    );
    assert_eq!(
        f.client.try_transfer(&issuer, &bob, &id, &-1),
        Err(Ok(Error::InvalidAmount))
    );
    assert_eq!(
        f.client.try_transfer(&issuer, &issuer, &id, &1),
        Err(Ok(Error::SelfTransfer))
    );
    assert_eq!(
        f.client.try_transfer(&issuer, &bob, &99, &1),
        Err(Ok(Error::AssetNotFound))
    );
    assert_eq!(
        f.client.try_transfer(&issuer, &bob, &id, &(TOTAL + 1)),
        Err(Ok(Error::InsufficientShares))
    );
    assert_eq!(
        f.client.try_transfer(&bob, &issuer, &id, &1),
        Err(Ok(Error::InsufficientShares))
    );
    // Nothing moved.
    assert_eq!(f.client.get_balance(&id, &issuer), TOTAL);
    assert_eq!(f.client.get_balance(&id, &bob), 0);
}

#[test]
#[should_panic(expected = "Auth")]
fn transfer_requires_sender_auth() {
    // No mock_all_auths: the sender's missing authorization must trap before
    // any other check runs.
    let env = Env::default();
    let id = env.register_contract(None, FractionalOwnership);
    let client = FractionalOwnershipClient::new(&env, &id);
    let from = Address::generate(&env);
    let to = Address::generate(&env);
    client.transfer(&from, &to, &1, &1);
}

// ---------------------------------------------------------------------------
// list_shares / cancel_listing
// ---------------------------------------------------------------------------

#[test]
fn list_shares_stores_listing() {
    let env = Env::default();
    let (f, issuer, id) = setup_asset(&env);
    f.client.list_shares(&issuer, &id, &400, &7);
    assert_eq!(
        f.client.get_listing(&id, &issuer),
        Some(Listing {
            amount: 400,
            price_per_share: 7
        })
    );
    assert!(has_event(&env, &f.contract_id, symbol_short!("listed")));
}

#[test]
fn listing_does_not_escrow_shares() {
    let env = Env::default();
    let (f, issuer, id) = setup_asset(&env);
    f.client.list_shares(&issuer, &id, &400, &7);
    assert_eq!(f.client.get_balance(&id, &issuer), TOTAL);
}

#[test]
fn list_shares_replaces_previous_listing() {
    let env = Env::default();
    let (f, issuer, id) = setup_asset(&env);
    f.client.list_shares(&issuer, &id, &400, &7);
    f.client.list_shares(&issuer, &id, &100, &9);
    assert_eq!(
        f.client.get_listing(&id, &issuer),
        Some(Listing {
            amount: 100,
            price_per_share: 9
        })
    );
}

#[test]
fn list_shares_rejects_bad_input() {
    let env = Env::default();
    let (f, issuer, id) = setup_asset(&env);
    let bob = Address::generate(&env);
    assert_eq!(
        f.client.try_list_shares(&issuer, &id, &0, &7),
        Err(Ok(Error::InvalidAmount))
    );
    assert_eq!(
        f.client.try_list_shares(&issuer, &id, &10, &0),
        Err(Ok(Error::InvalidAmount))
    );
    assert_eq!(
        f.client.try_list_shares(&issuer, &id, &10, &-3),
        Err(Ok(Error::InvalidAmount))
    );
    assert_eq!(
        f.client.try_list_shares(&issuer, &42, &10, &7),
        Err(Ok(Error::AssetNotFound))
    );
    assert_eq!(
        f.client.try_list_shares(&issuer, &id, &(TOTAL + 1), &7),
        Err(Ok(Error::InsufficientShares))
    );
    assert_eq!(
        f.client.try_list_shares(&bob, &id, &1, &7),
        Err(Ok(Error::InsufficientShares))
    );
    assert_eq!(f.client.get_listing(&id, &issuer), None);
}

#[test]
fn cancel_listing_removes_it() {
    let env = Env::default();
    let (f, issuer, id) = setup_asset(&env);
    f.client.list_shares(&issuer, &id, &400, &7);
    f.client.cancel_listing(&issuer, &id);
    assert_eq!(f.client.get_listing(&id, &issuer), None);
    assert!(has_event(&env, &f.contract_id, symbol_short!("unlisted")));
}

#[test]
fn cancel_listing_without_listing_fails() {
    let env = Env::default();
    let (f, issuer, id) = setup_asset(&env);
    assert_eq!(
        f.client.try_cancel_listing(&issuer, &id),
        Err(Ok(Error::NoListing))
    );
}

// ---------------------------------------------------------------------------
// buy
// ---------------------------------------------------------------------------

#[test]
fn buy_full_listing_moves_shares_and_funds() {
    let env = Env::default();
    let (f, issuer, id) = setup_asset(&env);
    let buyer = Address::generate(&env);
    f.token_admin.mint(&buyer, &10_000);
    f.client.list_shares(&issuer, &id, &400, &7);

    let cost = f.client.buy(&buyer, &issuer, &id, &400);

    assert_eq!(cost, 2_800);
    assert_eq!(f.client.get_balance(&id, &buyer), 400);
    assert_eq!(f.client.get_balance(&id, &issuer), 600);
    assert_eq!(f.token.balance(&buyer), 10_000 - 2_800);
    assert_eq!(f.token.balance(&issuer), 2_800);
    assert_eq!(f.client.get_listing(&id, &issuer), None);
    assert!(has_event(&env, &f.contract_id, symbol_short!("bought")));
}

#[test]
fn buy_partial_listing_keeps_remainder() {
    let env = Env::default();
    let (f, issuer, id) = setup_asset(&env);
    let buyer = Address::generate(&env);
    f.token_admin.mint(&buyer, &10_000);
    f.client.list_shares(&issuer, &id, &400, &7);

    assert_eq!(f.client.buy(&buyer, &issuer, &id, &150), 1_050);
    assert_eq!(
        f.client.get_listing(&id, &issuer),
        Some(Listing {
            amount: 250,
            price_per_share: 7
        })
    );
    assert_eq!(f.client.buy(&buyer, &issuer, &id, &250), 1_750);
    assert_eq!(f.client.get_listing(&id, &issuer), None);
    assert_eq!(f.client.get_balance(&id, &buyer), 400);
}

#[test]
fn buy_rejects_bad_input() {
    let env = Env::default();
    let (f, issuer, id) = setup_asset(&env);
    let buyer = Address::generate(&env);
    f.token_admin.mint(&buyer, &10_000);

    assert_eq!(
        f.client.try_buy(&buyer, &issuer, &id, &0),
        Err(Ok(Error::InvalidAmount))
    );
    assert_eq!(
        f.client.try_buy(&issuer, &issuer, &id, &1),
        Err(Ok(Error::SelfTransfer))
    );
    assert_eq!(
        f.client.try_buy(&buyer, &issuer, &77, &1),
        Err(Ok(Error::AssetNotFound))
    );
    assert_eq!(
        f.client.try_buy(&buyer, &issuer, &id, &1),
        Err(Ok(Error::NoListing))
    );

    f.client.list_shares(&issuer, &id, &10, &7);
    assert_eq!(
        f.client.try_buy(&buyer, &issuer, &id, &11),
        Err(Ok(Error::ExceedsListing))
    );
}

#[test]
fn buy_fails_if_seller_moved_shares_away() {
    let env = Env::default();
    let (f, issuer, id) = setup_asset(&env);
    let buyer = Address::generate(&env);
    let other = Address::generate(&env);
    f.token_admin.mint(&buyer, &10_000);
    f.client.list_shares(&issuer, &id, &800, &1);
    f.client.transfer(&issuer, &other, &id, &500);

    assert_eq!(
        f.client.try_buy(&buyer, &issuer, &id, &800),
        Err(Ok(Error::InsufficientShares))
    );
    assert_eq!(f.token.balance(&buyer), 10_000);
    assert_eq!(f.client.get_balance(&id, &buyer), 0);
}

#[test]
fn buy_cost_overflow_is_reported() {
    let env = Env::default();
    let (f, issuer, id) = setup_asset(&env);
    let buyer = Address::generate(&env);
    f.client.list_shares(&issuer, &id, &10, &i128::MAX);
    assert_eq!(
        f.client.try_buy(&buyer, &issuer, &id, &2),
        Err(Ok(Error::Overflow))
    );
}

#[test]
fn buy_with_insufficient_funds_reverts_everything() {
    let env = Env::default();
    let (f, issuer, id) = setup_asset(&env);
    let buyer = Address::generate(&env);
    f.token_admin.mint(&buyer, &5);
    f.client.list_shares(&issuer, &id, &10, &7);

    // The payment token traps; the whole call, including the share
    // movement that precedes it, is rolled back.
    assert!(f.client.try_buy(&buyer, &issuer, &id, &10).is_err());
    assert_eq!(f.client.get_balance(&id, &buyer), 0);
    assert_eq!(f.client.get_balance(&id, &issuer), TOTAL);
    assert_eq!(
        f.client.get_listing(&id, &issuer),
        Some(Listing {
            amount: 10,
            price_per_share: 7
        })
    );
    assert_eq!(f.token.balance(&buyer), 5);
}

// ---------------------------------------------------------------------------
// payouts
// ---------------------------------------------------------------------------

#[test]
fn single_holder_receives_whole_payout() {
    let env = Env::default();
    let (f, issuer, id) = setup_asset(&env);
    let funder = Address::generate(&env);
    f.token_admin.mint(&funder, &5_000);

    f.client.deposit_payout(&funder, &id, &5_000);
    assert!(has_event(&env, &f.contract_id, symbol_short!("deposit")));
    assert_eq!(f.token.balance(&f.contract_id), 5_000);
    assert_eq!(f.client.get_pending_payout(&id, &issuer), 5_000);

    assert_eq!(f.client.claim_payout(&issuer, &id), 5_000);
    assert!(has_event(&env, &f.contract_id, symbol_short!("claimed")));
    assert_eq!(f.token.balance(&issuer), 5_000);
    assert_eq!(f.token.balance(&f.contract_id), 0);
    assert_eq!(f.client.get_pending_payout(&id, &issuer), 0);
}

#[test]
fn payout_is_split_pro_rata() {
    let env = Env::default();
    let (f, issuer, id) = setup_asset(&env);
    let bob = Address::generate(&env);
    let funder = Address::generate(&env);
    f.token_admin.mint(&funder, &1_000);
    f.client.transfer(&issuer, &bob, &id, &250);

    f.client.deposit_payout(&funder, &id, &1_000);

    assert_eq!(f.client.get_pending_payout(&id, &issuer), 750);
    assert_eq!(f.client.get_pending_payout(&id, &bob), 250);
    assert_eq!(f.client.claim_payout(&bob, &id), 250);
    assert_eq!(f.client.claim_payout(&issuer, &id), 750);
    assert_eq!(f.token.balance(&f.contract_id), 0);
}

#[test]
fn transfer_after_deposit_keeps_sellers_entitlement() {
    let env = Env::default();
    let (f, issuer, id) = setup_asset(&env);
    let bob = Address::generate(&env);
    let funder = Address::generate(&env);
    f.token_admin.mint(&funder, &1_000);

    f.client.deposit_payout(&funder, &id, &1_000);
    // Moving every share after the deposit must not move the accrued payout.
    f.client.transfer(&issuer, &bob, &id, &TOTAL);

    assert_eq!(f.client.get_pending_payout(&id, &issuer), 1_000);
    assert_eq!(f.client.get_pending_payout(&id, &bob), 0);
    assert_eq!(f.client.claim_payout(&issuer, &id), 1_000);
}

#[test]
fn late_buyer_does_not_get_earlier_payouts() {
    let env = Env::default();
    let (f, issuer, id) = setup_asset(&env);
    let buyer = Address::generate(&env);
    let funder = Address::generate(&env);
    f.token_admin.mint(&funder, &2_000);
    f.token_admin.mint(&buyer, &10_000);

    f.client.deposit_payout(&funder, &id, &1_000);
    f.client.list_shares(&issuer, &id, &500, &1);
    f.client.buy(&buyer, &issuer, &id, &500);
    assert_eq!(f.client.get_pending_payout(&id, &buyer), 0);

    f.client.deposit_payout(&funder, &id, &1_000);
    assert_eq!(f.client.get_pending_payout(&id, &buyer), 500);
    assert_eq!(f.client.get_pending_payout(&id, &issuer), 1_500);
}

#[test]
fn multiple_deposits_accumulate() {
    let env = Env::default();
    let (f, issuer, id) = setup_asset(&env);
    let funder = Address::generate(&env);
    f.token_admin.mint(&funder, &600);
    f.client.deposit_payout(&funder, &id, &100);
    f.client.deposit_payout(&funder, &id, &200);
    f.client.deposit_payout(&funder, &id, &300);
    assert_eq!(f.client.get_pending_payout(&id, &issuer), 600);
    assert_eq!(f.client.claim_payout(&issuer, &id), 600);
}

#[test]
fn claiming_twice_pays_once() {
    let env = Env::default();
    let (f, issuer, id) = setup_asset(&env);
    let funder = Address::generate(&env);
    f.token_admin.mint(&funder, &100);
    f.client.deposit_payout(&funder, &id, &100);
    assert_eq!(f.client.claim_payout(&issuer, &id), 100);
    assert_eq!(
        f.client.try_claim_payout(&issuer, &id),
        Err(Ok(Error::NothingToClaim))
    );
    assert_eq!(f.token.balance(&issuer), 100);
}

#[test]
fn rounding_dust_stays_in_the_contract() {
    let env = Env::default();
    let f = setup(&env);
    let issuer = Address::generate(&env);
    let id = f.client.create_asset(&f.admin, &issuer, &3);
    let b = Address::generate(&env);
    let c = Address::generate(&env);
    f.client.transfer(&issuer, &b, &id, &1);
    f.client.transfer(&issuer, &c, &id, &1);
    let funder = Address::generate(&env);
    f.token_admin.mint(&funder, &10);

    f.client.deposit_payout(&funder, &id, &10);
    let total_paid = f.client.claim_payout(&issuer, &id)
        + f.client.claim_payout(&b, &id)
        + f.client.claim_payout(&c, &id);

    // 10 split three ways floors to 3 each; 1 unit of dust remains.
    assert_eq!(total_paid, 9);
    assert_eq!(f.token.balance(&f.contract_id), 1);
}

#[test]
fn payout_rejects_bad_input() {
    let env = Env::default();
    let (f, issuer, id) = setup_asset(&env);
    let funder = Address::generate(&env);
    f.token_admin.mint(&funder, &100);

    assert_eq!(
        f.client.try_deposit_payout(&funder, &id, &0),
        Err(Ok(Error::InvalidAmount))
    );
    assert_eq!(
        f.client.try_deposit_payout(&funder, &id, &-1),
        Err(Ok(Error::InvalidAmount))
    );
    assert_eq!(
        f.client.try_deposit_payout(&funder, &55, &1),
        Err(Ok(Error::AssetNotFound))
    );
    assert_eq!(
        f.client.try_claim_payout(&issuer, &55),
        Err(Ok(Error::AssetNotFound))
    );
    assert_eq!(
        f.client.try_claim_payout(&issuer, &id),
        Err(Ok(Error::NothingToClaim))
    );
}

#[test]
fn deposit_too_small_to_distribute_is_rejected() {
    let env = Env::default();
    let f = setup(&env);
    let issuer = Address::generate(&env);
    // Supply larger than amount * PAYOUT_PRECISION, so the index would not move.
    let supply = PAYOUT_PRECISION * 10;
    let id = f.client.create_asset(&f.admin, &issuer, &supply);
    let funder = Address::generate(&env);
    f.token_admin.mint(&funder, &1);
    assert_eq!(
        f.client.try_deposit_payout(&funder, &id, &1),
        Err(Ok(Error::PayoutTooSmall))
    );
    assert_eq!(f.token.balance(&funder), 1);
}

#[test]
fn deposit_amount_overflow_is_reported() {
    let env = Env::default();
    let (f, _issuer, id) = setup_asset(&env);
    let funder = Address::generate(&env);
    assert_eq!(
        f.client.try_deposit_payout(&funder, &id, &i128::MAX),
        Err(Ok(Error::Overflow))
    );
}

#[test]
fn payouts_are_isolated_per_asset() {
    let env = Env::default();
    let (f, issuer, first) = setup_asset(&env);
    let second = f.client.create_asset(&f.admin, &issuer, &TOTAL);
    let funder = Address::generate(&env);
    f.token_admin.mint(&funder, &300);
    f.client.deposit_payout(&funder, &first, &300);
    assert_eq!(f.client.get_pending_payout(&first, &issuer), 300);
    assert_eq!(f.client.get_pending_payout(&second, &issuer), 0);
}
