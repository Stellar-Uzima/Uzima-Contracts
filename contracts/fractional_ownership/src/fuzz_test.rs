//! Property-based (fuzz) tests for `FractionalOwnership`.
//!
//! Run with `PROPTEST_CASES=<n> cargo test -p fractional_ownership fuzz_`.
//!
//! * `fuzz_random_sequences_match_model` drives random operation sequences
//!   and compares every outcome and every balance with a plain-Rust model.
//! * `fuzz_extreme_amounts_never_trap` feeds boundary and random `i128`
//!   values to every entry point and requires a typed error, never a host
//!   trap (which would mean unchecked arithmetic).
//! * `fuzz_payout_split_is_pro_rata_and_solvent` checks rounding bounds.

use super::*;
use proptest::collection::vec as pvec;
use proptest::prelude::*;
use soroban_sdk::{testutils::Address as _, token, Address, Env};
use std::vec::Vec;

const SUPPLY: i128 = 1_000_000;
const USERS: usize = 4;
const FUNDS: i128 = 1_000_000_000_000;

#[derive(Clone, Debug)]
enum Op {
    Transfer {
        from: usize,
        to: usize,
        amount: i128,
    },
    List {
        seller: usize,
        amount: i128,
        price: i128,
    },
    Cancel {
        seller: usize,
    },
    Buy {
        buyer: usize,
        seller: usize,
        amount: i128,
    },
    Deposit {
        amount: i128,
    },
    Claim {
        who: usize,
    },
}

fn shares() -> impl Strategy<Value = i128> {
    prop_oneof![0i128..=10, 0i128..=(SUPPLY + 10)]
}

fn op_strategy() -> impl Strategy<Value = Op> {
    let user = || 0..USERS;
    prop_oneof![
        (user(), user(), shares()).prop_map(|(from, to, amount)| Op::Transfer { from, to, amount }),
        (user(), shares(), 0i128..=50).prop_map(|(seller, amount, price)| Op::List {
            seller,
            amount,
            price
        }),
        user().prop_map(|seller| Op::Cancel { seller }),
        (user(), user(), shares()).prop_map(|(buyer, seller, amount)| Op::Buy {
            buyer,
            seller,
            amount
        }),
        (0i128..=1_000_000).prop_map(|amount| Op::Deposit { amount }),
        user().prop_map(|who| Op::Claim { who }),
    ]
}

fn run_sequence(ops: Vec<Op>) -> Result<(), TestCaseError> {
    let env = Env::default();
    env.mock_all_auths();
    // One Env serves a whole sequence of calls plus per-step invariant reads,
    // so the default per-transaction test budget would be exhausted by the
    // harness itself rather than by any single contract call.
    env.budget().reset_unlimited();
    let admin = Address::generate(&env);
    let token_addr = env
        .register_stellar_asset_contract_v2(admin.clone())
        .address();
    let token = token::Client::new(&env, &token_addr);
    let mint = token::StellarAssetClient::new(&env, &token_addr);
    let cid = env.register_contract(None, FractionalOwnership);
    let client = FractionalOwnershipClient::new(&env, &cid);
    client.initialize(&admin, &token_addr);

    let users: Vec<Address> = (0..USERS).map(|_| Address::generate(&env)).collect();
    let funder = Address::generate(&env);
    mint.mint(&funder, &FUNDS);
    for u in &users {
        mint.mint(u, &FUNDS);
    }
    let id = client.create_asset(&admin, &users[0], &SUPPLY);

    let mut bal = [0i128; USERS];
    bal[0] = SUPPLY;
    let mut listings: [Option<(i128, i128)>; USERS] = [None; USERS];
    let mut deposited = 0i128;
    let mut claimed = 0i128;

    for op in ops {
        match op {
            Op::Transfer { from, to, amount } => {
                let ok = amount > 0 && from != to && bal[from] >= amount;
                let res = client.try_transfer(&users[from], &users[to], &id, &amount);
                match (&res, ok) {
                    (Ok(Ok(())), true) => {
                        bal[from] -= amount;
                        bal[to] += amount;
                    },
                    (Err(Ok(_)), false) => {},
                    _ => prop_assert!(false, "transfer {:?} expected_ok={}", res, ok),
                }
            },
            Op::List {
                seller,
                amount,
                price,
            } => {
                let ok = amount > 0 && price > 0 && bal[seller] >= amount;
                let res = client.try_list_shares(&users[seller], &id, &amount, &price);
                match (&res, ok) {
                    (Ok(Ok(())), true) => listings[seller] = Some((amount, price)),
                    (Err(Ok(_)), false) => {},
                    _ => prop_assert!(false, "list {:?} expected_ok={}", res, ok),
                }
            },
            Op::Cancel { seller } => {
                let ok = listings[seller].is_some();
                let res = client.try_cancel_listing(&users[seller], &id);
                match (&res, ok) {
                    (Ok(Ok(())), true) => listings[seller] = None,
                    (Err(Ok(_)), false) => {},
                    _ => prop_assert!(false, "cancel {:?} expected_ok={}", res, ok),
                }
            },
            Op::Buy {
                buyer,
                seller,
                amount,
            } => {
                let ok = amount > 0
                    && buyer != seller
                    && match listings[seller] {
                        Some((la, _)) => amount <= la && bal[seller] >= amount,
                        None => false,
                    };
                let res = client.try_buy(&users[buyer], &users[seller], &id, &amount);
                match (&res, ok) {
                    (Ok(Ok(cost)), true) => {
                        let (la, price) = listings[seller].unwrap();
                        prop_assert_eq!(*cost, amount * price);
                        bal[seller] -= amount;
                        bal[buyer] += amount;
                        listings[seller] = if la == amount {
                            None
                        } else {
                            Some((la - amount, price))
                        };
                    },
                    (Err(Ok(_)), false) => {},
                    _ => prop_assert!(false, "buy {:?} expected_ok={}", res, ok),
                }
            },
            Op::Deposit { amount } => {
                let ok = amount > 0;
                let res = client.try_deposit_payout(&funder, &id, &amount);
                match (&res, ok) {
                    (Ok(Ok(())), true) => deposited += amount,
                    (Err(Ok(_)), false) => {},
                    _ => prop_assert!(false, "deposit {:?} expected_ok={}", res, ok),
                }
            },
            Op::Claim { who } => {
                let pending = client.get_pending_payout(&id, &users[who]);
                let res = client.try_claim_payout(&users[who], &id);
                match (&res, pending > 0) {
                    (Ok(Ok(paid)), true) => {
                        prop_assert_eq!(*paid, pending);
                        claimed += paid;
                    },
                    (Err(Ok(_)), false) => {},
                    _ => prop_assert!(false, "claim {:?} pending={}", res, pending),
                }
            },
        }

        // Invariants after every step.
        let mut held = 0i128;
        let mut owed = 0i128;
        for (i, u) in users.iter().enumerate() {
            let b = client.get_balance(&id, u);
            prop_assert_eq!(b, bal[i]);
            prop_assert!(b >= 0);
            held += b;
            owed += client.get_pending_payout(&id, u);
            let expected = listings[i].map(|(amount, price_per_share)| Listing {
                amount,
                price_per_share,
            });
            prop_assert_eq!(client.get_listing(&id, u), expected);
        }
        prop_assert_eq!(held, SUPPLY);
        prop_assert_eq!(client.get_total_shares(&id), SUPPLY);
        let contract_funds = token.balance(&cid);
        prop_assert_eq!(contract_funds, deposited - claimed);
        // Solvency: the contract never owes more than it holds.
        prop_assert!(
            owed <= contract_funds,
            "owed {} > held {}",
            owed,
            contract_funds
        );
    }
    Ok(())
}

proptest! {
    #[test]
    fn fuzz_random_sequences_match_model(ops in pvec(op_strategy(), 1..40)) {
        run_sequence(ops)?;
    }
}

fn extreme() -> impl Strategy<Value = i128> {
    prop_oneof![
        Just(i128::MIN),
        Just(i128::MIN + 1),
        Just(-1i128),
        Just(0i128),
        Just(1i128),
        Just(i128::MAX / 2),
        Just(i128::MAX - 1),
        Just(i128::MAX),
        any::<i128>(),
    ]
}

proptest! {
    #[test]
    fn fuzz_extreme_amounts_never_trap(a in extreme(), b in extreme(), c in extreme()) {
        let env = Env::default();
        env.mock_all_auths();
        let admin = Address::generate(&env);
        let token_addr = env.register_stellar_asset_contract_v2(admin.clone()).address();
        let mint = token::StellarAssetClient::new(&env, &token_addr);
        let cid = env.register_contract(None, FractionalOwnership);
        let client = FractionalOwnershipClient::new(&env, &cid);
        client.initialize(&admin, &token_addr);
        let alice = Address::generate(&env);
        let bob = Address::generate(&env);
        let funder = Address::generate(&env);
        mint.mint(&funder, &(i128::MAX / 2));
        mint.mint(&bob, &i128::MAX);

        // A hostile supply must be rejected or accepted, never trap.
        let hostile = client.try_create_asset(&admin, &alice, &a);
        prop_assert!(matches!(hostile, Ok(Ok(_)) | Err(Ok(_))), "create {:?}", hostile);

        let id = client.create_asset(&admin, &alice, &SUPPLY);

        let r = client.try_transfer(&alice, &bob, &id, &b);
        prop_assert!(matches!(r, Ok(Ok(_)) | Err(Ok(_))), "transfer {:?}", r);
        let r = client.try_list_shares(&alice, &id, &b, &c);
        prop_assert!(matches!(r, Ok(Ok(_)) | Err(Ok(_))), "list {:?}", r);
        let r = client.try_buy(&bob, &alice, &id, &b);
        prop_assert!(matches!(r, Ok(Ok(_)) | Err(Ok(_))), "buy {:?}", r);
        let r = client.try_deposit_payout(&funder, &id, &c);
        prop_assert!(matches!(r, Ok(Ok(_)) | Err(Ok(_))), "deposit {:?}", r);
        let r = client.try_deposit_payout(&funder, &id, &a);
        prop_assert!(matches!(r, Ok(Ok(_)) | Err(Ok(_))), "deposit {:?}", r);
        let r = client.try_claim_payout(&alice, &id);
        prop_assert!(matches!(r, Ok(Ok(_)) | Err(Ok(_))), "claim {:?}", r);
        let r = client.try_get_pending_payout(&id, &alice);
        prop_assert!(matches!(r, Ok(Ok(_)) | Err(Ok(_))), "pending {:?}", r);
    }
}

proptest! {
    #[test]
    fn fuzz_payout_split_is_pro_rata_and_solvent(
        supply in 1i128..=1_000_000,
        cuts in pvec(1i128..=1_000_000, 1..5),
        deposit in 1i128..=1_000_000_000_000,
    ) {
        let env = Env::default();
        env.mock_all_auths();
        let admin = Address::generate(&env);
        let token_addr = env.register_stellar_asset_contract_v2(admin.clone()).address();
        let token = token::Client::new(&env, &token_addr);
        let mint = token::StellarAssetClient::new(&env, &token_addr);
        let cid = env.register_contract(None, FractionalOwnership);
        let client = FractionalOwnershipClient::new(&env, &cid);
        client.initialize(&admin, &token_addr);
        let issuer = Address::generate(&env);
        let id = client.create_asset(&admin, &issuer, &supply);

        // Hand out slices of the supply while shares remain.
        let mut holders: Vec<(Address, i128)> = Vec::new();
        let mut left = supply;
        for cut in cuts {
            let give = cut.min(left);
            if give == 0 {
                break;
            }
            let h = Address::generate(&env);
            client.transfer(&issuer, &h, &id, &give);
            holders.push((h, give));
            left -= give;
        }
        holders.push((issuer.clone(), left));

        let funder = Address::generate(&env);
        mint.mint(&funder, &deposit);
        client.deposit_payout(&funder, &id, &deposit);

        let n = holders.len() as i128;
        let mut paid_total = 0i128;
        for (h, held) in &holders {
            let exact = deposit * held / supply;
            let pending = client.get_pending_payout(&id, h);
            prop_assert!(pending <= exact, "pending {} > exact {}", pending, exact);
            prop_assert!(exact - pending <= 2, "pending {} too far below exact {}", pending, exact);
            if pending > 0 {
                prop_assert_eq!(client.claim_payout(h, &id), pending);
                paid_total += pending;
            }
        }
        prop_assert!(paid_total <= deposit);
        prop_assert!(deposit - paid_total <= 2 * n, "dust {} for {} holders", deposit - paid_total, n);
        prop_assert_eq!(token.balance(&cid), deposit - paid_total);
    }
}
