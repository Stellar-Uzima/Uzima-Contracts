#![no_std]

extern crate alloc;

use soroban_sdk::{contract, contractimpl, Vec};

use shared_types::rate_curve::Tier;

/// Reward tier calculations for the Uzima network.
///
/// Delegates to the shared `shared_types::rate_curve::calculate_tiered_rate`
/// utility so reward tiers stay consistent with fee and batch-rewards
/// instead of duplicating the tier-matching logic (issue #1341).
#[contract]
pub struct RewardsContract;

#[contractimpl]
impl RewardsContract {
    /// Calculates the applicable reward for `value` using the shared
    /// tiered-rate utility.
    ///
    /// `tiers` must be sorted ascending by threshold; the first tier should
    /// normally have `threshold = 0`. Returns 0 for an empty tier list.
    pub fn calculate_tiered_rate(value: i128, tiers: Vec<Tier>) -> i128 {
        let tiers: alloc::vec::Vec<Tier> = tiers.iter().collect();
        shared_types::rate_curve::calculate_tiered_rate(value, &tiers)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use soroban_sdk::{vec, Env};

    fn sample_tiers(env: &Env) -> Vec<Tier> {
        vec![
            env,
            Tier {
                threshold: 0,
                rate_bps: 100,
            }, // 1.00% from 0
            Tier {
                threshold: 1_000,
                rate_bps: 50,
            }, // 0.50% from 1000
            Tier {
                threshold: 10_000,
                rate_bps: 25,
            }, // 0.25% from 10000
        ]
    }

    #[test]
    fn reward_calculation_uses_shared_utility() {
        let env = Env::default();
        let tiers = sample_tiers(&env);

        // 500 * 1.00% = 5
        assert_eq!(RewardsContract::calculate_tiered_rate(500, tiers.clone()), 5);
        // value == 1000 uses the 0.50% tier: 1000 * 0.50% = 5
        assert_eq!(RewardsContract::calculate_tiered_rate(1_000, tiers.clone()), 5);
        // 999 * 1.00% = 9 (integer division)
        assert_eq!(RewardsContract::calculate_tiered_rate(999, tiers.clone()), 9);
        // 50_000 * 0.25% = 125
        assert_eq!(RewardsContract::calculate_tiered_rate(50_000, tiers.clone()), 125);
        assert_eq!(RewardsContract::calculate_tiered_rate(0, tiers), 0);
    }

    #[test]
    fn matches_shared_implementation_exactly() {
        let env = Env::default();
        let tiers = sample_tiers(&env);
        let shared_tiers = [
            Tier {
                threshold: 0,
                rate_bps: 100,
            },
            Tier {
                threshold: 1_000,
                rate_bps: 50,
            },
            Tier {
                threshold: 10_000,
                rate_bps: 25,
            },
        ];

        for value in [0i128, 1, 500, 999, 1_000, 1_001, 10_000, 50_000] {
            assert_eq!(
                RewardsContract::calculate_tiered_rate(value, tiers.clone()),
                shared_types::rate_curve::calculate_tiered_rate(value, &shared_tiers),
            );
        }
    }

    #[test]
    fn empty_tiers_return_zero() {
        let env = Env::default();
        assert_eq!(RewardsContract::calculate_tiered_rate(5_000, Vec::new(&env)), 0);
    }
}
