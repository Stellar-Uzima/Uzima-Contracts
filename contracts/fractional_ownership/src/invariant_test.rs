//! Invariant tests for `FractionalOwnership`: after any sequence of
//! transfers, the sum of all balances always equals the asset's total shares.

use super::*;
use proptest::collection::vec as pvec;
use proptest::prelude::*;
use soroban_sdk::{testutils::Address as _, Address, Env};
use std::vec::Vec;

const TOTAL: i128 = 10_000;
const USERS: usize = 3;

proptest! {
    #![proptest_config(ProptestConfig::with_cases(32))]

    #[test]
    fn invariant_total_shares_conserved(
        ops in pvec((0..USERS, 0..USERS, 0i128..=TOTAL), 1..20)
    ) {
        let env = Env::default();
        env.mock_all_auths();
        let admin = Address::generate(&env);
        let token = Address::generate(&env);
        let id = env.register_contract(None, FractionalOwnership);
        let client = FractionalOwnershipClient::new(&env, &id);
        client.initialize(&admin, &token);

        let users: Vec<Address> = (0..USERS).map(|_| Address::generate(&env)).collect();
        let asset_id = client.create_asset(&admin, &users[0], &TOTAL);

        for (from, to, amount) in ops {
            // Failed transfers are fine; they just must not break the invariant.
            let _ = client.try_transfer(&users[from], &users[to], &asset_id, &amount);

            let sum: i128 = users.iter().map(|u| client.get_balance(&asset_id, u)).sum();
            prop_assert_eq!(sum, TOTAL);
            prop_assert_eq!(client.get_total_shares(&asset_id), TOTAL);
            for u in users.iter() {
                prop_assert!(client.get_balance(&asset_id, u) >= 0);
            }
        }
    }
}
