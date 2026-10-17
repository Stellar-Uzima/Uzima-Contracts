//! Reentrancy tests (#1788).
//!
//! `ReentrantToken` is a hostile payment token: its `transfer` calls back
//! into `FractionalOwnership` before returning, trying to double-claim,
//! double-buy, re-deposit or move shares while the outer call is mid-flight.
//!
//! Two layers stop this. Soroban's host refuses to re-enter a contract that
//! is already on the call stack, which is what the hostile token hits first.
//! The contract's own storage lock is defense in depth; it is exercised
//! directly by `lock_blocks_every_mutator`.

use super::*;
use soroban_sdk::{
    contract, contractimpl, contracttype, testutils::Address as _, Address, Env, InvokeError,
};

const TOTAL: i128 = 1_000;

#[contracttype]
enum EKey {
    Target,
    Victim,
    Asset,
    Mode,
    Outcome,
}

/// What the hostile token tries to do when the contract calls `transfer`.
const MODE_NONE: u32 = 0;
const MODE_CLAIM: u32 = 1;
const MODE_BUY: u32 = 2;
const MODE_DEPOSIT: u32 = 3;
const MODE_TRANSFER: u32 = 4;
const MODE_LIST: u32 = 5;

/// Outcome codes recorded by the hostile token.
const OUT_UNSET: u32 = 0;
/// The re-entrant call succeeded. Any test seeing this is a vulnerability.
const OUT_SUCCEEDED: u32 = 1;
/// Rejected by the Soroban host (contract already on the call stack).
const OUT_HOST_REJECTED: u32 = 2;
/// Rejected by the contract's own `Reentrancy` error.
const OUT_LOCK_REJECTED: u32 = 3;
/// Rejected with some other typed contract error.
const OUT_OTHER_ERROR: u32 = 4;

fn classify<T, C>(r: Result<Result<T, C>, Result<Error, InvokeError>>) -> u32 {
    match r {
        Ok(Ok(_)) => OUT_SUCCEEDED,
        Err(Ok(Error::Reentrancy)) => OUT_LOCK_REJECTED,
        Err(Ok(_)) => OUT_OTHER_ERROR,
        Err(Err(_)) | Ok(Err(_)) => OUT_HOST_REJECTED,
    }
}

#[contract]
pub struct ReentrantToken;

#[contractimpl]
impl ReentrantToken {
    pub fn configure(env: Env, target: Address, victim: Address, asset_id: u64, mode: u32) {
        env.storage().instance().set(&EKey::Target, &target);
        env.storage().instance().set(&EKey::Victim, &victim);
        env.storage().instance().set(&EKey::Asset, &asset_id);
        env.storage().instance().set(&EKey::Mode, &mode);
        env.storage().instance().set(&EKey::Outcome, &OUT_UNSET);
    }

    pub fn outcome(env: Env) -> u32 {
        env.storage()
            .instance()
            .get(&EKey::Outcome)
            .unwrap_or(OUT_UNSET)
    }

    pub fn transfer(env: Env, _from: Address, _to: Address, _amount: i128) {
        let mode: u32 = env
            .storage()
            .instance()
            .get(&EKey::Mode)
            .unwrap_or(MODE_NONE);
        if mode == MODE_NONE {
            return;
        }
        // Re-enter at most once.
        env.storage().instance().set(&EKey::Mode, &MODE_NONE);
        let target: Address = env.storage().instance().get(&EKey::Target).unwrap();
        let victim: Address = env.storage().instance().get(&EKey::Victim).unwrap();
        let asset: u64 = env.storage().instance().get(&EKey::Asset).unwrap();
        let client = FractionalOwnershipClient::new(&env, &target);
        let other = Address::generate(&env);

        let code = match mode {
            MODE_CLAIM => classify(client.try_claim_payout(&victim, &asset)),
            MODE_BUY => classify(client.try_buy(&other, &victim, &asset, &1)),
            MODE_DEPOSIT => classify(client.try_deposit_payout(&victim, &asset, &1)),
            MODE_TRANSFER => classify(client.try_transfer(&victim, &other, &asset, &1)),
            MODE_LIST => classify(client.try_list_shares(&victim, &asset, &1, &1)),
            _ => OUT_UNSET,
        };
        env.storage().instance().set(&EKey::Outcome, &code);
    }
}

struct Fixture<'a> {
    client: FractionalOwnershipClient<'a>,
    evil: ReentrantTokenClient<'a>,
    contract_id: Address,
    admin: Address,
    holder: Address,
    asset_id: u64,
}

/// A deployment whose payment token is the hostile token, with one asset
/// held entirely by `holder`.
fn setup<'a>(env: &'a Env) -> Fixture<'a> {
    env.mock_all_auths();
    let admin = Address::generate(env);
    let holder = Address::generate(env);
    let evil_id = env.register_contract(None, ReentrantToken);
    let evil = ReentrantTokenClient::new(env, &evil_id);
    let contract_id = env.register_contract(None, FractionalOwnership);
    let client = FractionalOwnershipClient::new(env, &contract_id);
    client.initialize(&admin, &evil_id);
    let asset_id = client.create_asset(&admin, &holder, &TOTAL);
    Fixture {
        client,
        evil,
        contract_id,
        admin,
        holder,
        asset_id,
    }
}

fn arm(f: &Fixture, mode: u32) {
    f.evil
        .configure(&f.contract_id, &f.holder, &f.asset_id, &mode);
}

fn assert_rejected(f: &Fixture) {
    let outcome = f.evil.outcome();
    assert!(
        outcome == OUT_HOST_REJECTED || outcome == OUT_LOCK_REJECTED,
        "re-entrant call was not rejected, outcome code {outcome}"
    );
}

#[test]
fn claim_cannot_be_reentered() {
    let env = Env::default();
    let f = setup(&env);
    f.client.deposit_payout(&f.holder, &f.asset_id, &1_000);
    arm(&f, MODE_CLAIM);

    assert_eq!(f.client.claim_payout(&f.holder, &f.asset_id), 1_000);

    assert_rejected(&f);
    assert_eq!(f.client.get_pending_payout(&f.asset_id, &f.holder), 0);
    assert_eq!(
        f.client.try_claim_payout(&f.holder, &f.asset_id),
        Err(Ok(Error::NothingToClaim))
    );
}

#[test]
fn buy_cannot_be_reentered() {
    let env = Env::default();
    let f = setup(&env);
    let buyer = Address::generate(&env);
    f.client.list_shares(&f.holder, &f.asset_id, &10, &1);
    arm(&f, MODE_BUY);

    assert_eq!(f.client.buy(&buyer, &f.holder, &f.asset_id, &1), 1);

    assert_rejected(&f);
    assert_eq!(f.client.get_balance(&f.asset_id, &buyer), 1);
    assert_eq!(f.client.get_balance(&f.asset_id, &f.holder), TOTAL - 1);
    // Exactly one share left the listing, not two.
    assert_eq!(
        f.client.get_listing(&f.asset_id, &f.holder),
        Some(Listing {
            amount: 9,
            price_per_share: 1
        })
    );
}

#[test]
fn deposit_cannot_be_reentered() {
    let env = Env::default();
    let f = setup(&env);
    arm(&f, MODE_DEPOSIT);

    f.client.deposit_payout(&f.holder, &f.asset_id, &500);

    assert_rejected(&f);
    // Exactly one 500 deposit is reflected, not 500 + 1.
    assert_eq!(f.client.get_pending_payout(&f.asset_id, &f.holder), 500);
}

#[test]
fn share_movement_cannot_be_reentered_during_a_deposit() {
    for mode in [MODE_TRANSFER, MODE_LIST] {
        let env = Env::default();
        let f = setup(&env);
        arm(&f, mode);

        f.client.deposit_payout(&f.holder, &f.asset_id, &100);

        assert_rejected(&f);
        assert_eq!(f.client.get_balance(&f.asset_id, &f.holder), TOTAL);
        assert_eq!(f.client.get_listing(&f.asset_id, &f.holder), None);
    }
}

#[test]
fn lock_blocks_every_mutator() {
    let env = Env::default();
    let f = setup(&env);
    let other = Address::generate(&env);

    // Simulate a guarded call in progress.
    env.as_contract(&f.contract_id, || {
        env.storage().instance().set(&DataKey::Locked, &true);
    });

    let id = f.asset_id;
    assert_eq!(
        f.client.try_create_asset(&f.admin, &other, &1),
        Err(Ok(Error::Reentrancy))
    );
    assert_eq!(
        f.client.try_transfer(&f.holder, &other, &id, &1),
        Err(Ok(Error::Reentrancy))
    );
    assert_eq!(
        f.client.try_list_shares(&f.holder, &id, &1, &1),
        Err(Ok(Error::Reentrancy))
    );
    assert_eq!(
        f.client.try_cancel_listing(&f.holder, &id),
        Err(Ok(Error::Reentrancy))
    );
    assert_eq!(
        f.client.try_buy(&other, &f.holder, &id, &1),
        Err(Ok(Error::Reentrancy))
    );
    assert_eq!(
        f.client.try_deposit_payout(&other, &id, &1),
        Err(Ok(Error::Reentrancy))
    );
    assert_eq!(
        f.client.try_claim_payout(&f.holder, &id),
        Err(Ok(Error::Reentrancy))
    );

    // Read-only views stay available and nothing changed.
    assert_eq!(f.client.get_balance(&id, &f.holder), TOTAL);
    assert_eq!(f.client.get_asset_count(), 1);

    // Releasing the lock restores normal operation.
    env.as_contract(&f.contract_id, || {
        env.storage().instance().set(&DataKey::Locked, &false);
    });
    f.client.transfer(&f.holder, &other, &id, &1);
    assert_eq!(f.client.get_balance(&id, &other), 1);
}

#[test]
fn lock_is_released_after_every_guarded_call() {
    let env = Env::default();
    let f = setup(&env);
    let other = Address::generate(&env);
    let id = f.asset_id;

    f.client.deposit_payout(&f.holder, &id, &100);
    f.client.transfer(&f.holder, &other, &id, &1);

    f.client.claim_payout(&f.holder, &id);
    f.client.transfer(&f.holder, &other, &id, &1);

    f.client.list_shares(&f.holder, &id, &5, &1);
    f.client.buy(&other, &f.holder, &id, &1);
    f.client.transfer(&f.holder, &other, &id, &1);

    // Three transfers plus one purchase, one share each.
    assert_eq!(f.client.get_balance(&id, &other), 4);
}

#[test]
fn lock_is_not_left_behind_by_a_failing_guarded_call() {
    let env = Env::default();
    let f = setup(&env);
    let other = Address::generate(&env);
    let id = f.asset_id;

    // Each guarded entry point fails after taking the lock.
    assert_eq!(
        f.client.try_claim_payout(&f.holder, &id),
        Err(Ok(Error::NothingToClaim))
    );
    assert_eq!(
        f.client.try_buy(&other, &f.holder, &id, &1),
        Err(Ok(Error::NoListing))
    );
    assert_eq!(
        f.client.try_deposit_payout(&other, &id, &0),
        Err(Ok(Error::InvalidAmount))
    );

    // A failed call reverts the lock together with everything else.
    f.client.transfer(&f.holder, &other, &id, &1);
    assert_eq!(f.client.get_balance(&id, &other), 1);
}
