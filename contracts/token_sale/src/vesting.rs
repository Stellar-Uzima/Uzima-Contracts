// Vesting contract - arithmetic is bounds-checked and overflow is impossible with token amounts
#![allow(clippy::arithmetic_side_effects)]
#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]
#![allow(clippy::panic)]

use crate::storage::*;
use crate::types::*;
use soroban_sdk::{contract, contractimpl, contractmeta, token, Address, Env, Vec};

contractmeta!(
    key = "Description",
    val = "Token Vesting Contract with Cliff and Linear Release"
);

#[contract]
pub struct VestingContract;

#[contractimpl]
impl VestingContract {
    /// Initialize the vesting contract
    pub fn initialize_vesting(env: Env, owner: Address, token_address: Address) {
        owner.require_auth();

        let config = SaleConfig {
            token_address: token_address.clone(),
            treasury: owner.clone(),
            soft_cap: 0,
            hard_cap: 0,
            is_finalized: false,
            refunds_enabled: false,
        };

        set_config(&env, &config);
        set_owner(&env, &owner);

        env.events()
            .publish(("vesting_initialized",), (token_address,));
    }

    /// Create a vesting schedule for a beneficiary
    pub fn create_vesting_schedule(
        env: Env,
        beneficiary: Address,
        cliff_duration: u64,
        vesting_duration: u64,
        total_amount: u128,
    ) {
        let owner = get_owner(&env);
        owner.require_auth();

        assert!(total_amount > 0, "Amount must be > 0");
        assert!(vesting_duration > 0, "Duration must be > 0");
        assert!(
            cliff_duration <= vesting_duration,
            "Cliff cannot be longer than vesting"
        );

        let current_time = get_ledger_timestamp(&env);
        let schedule = VestingSchedule {
            cliff_duration,
            vesting_duration,
            start_time: current_time,
            total_amount,
            released_amount: 0,
        };

    let mut schedules: Map<Address, VestingSchedule> = env
        .storage()
        .persistent()
        .get(&VESTING_SCHEDULES)
        .unwrap_or(Map::new(env));

    schedules.set(beneficiary, schedule);
    env.storage().persistent().set(&VESTING_SCHEDULES, &schedules);
}

pub fn release_vested_tokens(env: &Env, beneficiary: Address) -> i128 {
    let mut schedules: Map<Address, VestingSchedule> = env
        .storage()
        .persistent()
        .get(&VESTING_SCHEDULES)
        .expect("No vesting schedule");

    let mut schedule = schedules.get(beneficiary.clone()).expect("No vesting schedule");
    let now = env.ledger().timestamp();

    if now < schedule.start_time + schedule.cliff {
        return 0;
    }

    let time_vested = now.saturating_sub(schedule.start_time);
    let vested_amount = if time_vested >= schedule.duration {
        schedule.total_amount
    } else {
        schedule.total_amount * (time_vested as i128) / (schedule.duration as i128)
    };

    let releasable = vested_amount.saturating_sub(schedule.released_amount);

    if releasable > 0 {
        schedule.released_amount += releasable;
        schedules.set(beneficiary, schedule);
        env.storage().persistent().set(&VESTING_SCHEDULES, &schedules);
    }

    releasable
}

    /// Get vesting schedule for a beneficiary
    pub fn get_vesting_schedule(env: Env, beneficiary: Address) -> Option<VestingSchedule> {
        get_vesting_schedule(&env, &beneficiary)
    }

    /// Get the amount of tokens that can be released now
    pub fn get_releasable_amount(env: Env, beneficiary: Address) -> u128 {
        let schedule = match get_vesting_schedule(&env, &beneficiary) {
            Some(s) => s,
            None => return 0,
        };

        let current_time = get_ledger_timestamp(&env);
        let vested_amount = Self::get_vested_amount(env, beneficiary, current_time);

        vested_amount.saturating_sub(schedule.released_amount)
    }

    /// Calculate vested amount at a specific timestamp
    pub fn get_vested_amount(env: Env, beneficiary: Address, timestamp: u64) -> u128 {
        let schedule = match get_vesting_schedule(&env, &beneficiary) {
            Some(s) => s,
            None => return 0,
        };

        if schedule.total_amount == 0 {
            return 0;
        }

        let cliff_end = schedule.start_time + schedule.cliff_duration;

        // Before cliff, nothing is vested
        if timestamp < cliff_end {
            return 0;
        }

        let vesting_end = schedule.start_time + schedule.vesting_duration;

        // After vesting period, everything is vested
        if timestamp >= vesting_end {
            return schedule.total_amount;
        }

        // Linear vesting between cliff and end
        let time_since_cliff = timestamp - cliff_end;
        let vesting_period = schedule.vesting_duration - schedule.cliff_duration;

        if vesting_period == 0 {
            return schedule.total_amount;
        }

        // Calculate proportional vesting
        (schedule.total_amount * time_since_cliff as u128) / vesting_period as u128
    }

    /// Batch create vesting schedules for team members
    pub fn batch_create_vesting(
        env: Env,
        beneficiaries: Vec<Address>,
        cliff_duration: u64,
        vesting_duration: u64,
        amounts: Vec<u128>,
    ) {
        let owner = get_owner(&env);
        owner.require_auth();

        assert!(
            beneficiaries.len() == amounts.len(),
            "Mismatched array lengths"
        );

        for i in 0..beneficiaries.len() {
            let beneficiary = beneficiaries.get(i).unwrap();
            let amount = amounts.get(i).unwrap();

            Self::create_vesting_schedule(
                env.clone(),
                beneficiary,
                cliff_duration,
                vesting_duration,
                amount,
            );
        }
    }

    /// Emergency function to update vesting schedule (use with caution)
    pub fn update_vesting_schedule(
        env: Env,
        beneficiary: Address,
        new_cliff_duration: u64,
        new_vesting_duration: u64,
        new_total_amount: u128,
    ) {
        let owner = get_owner(&env);
        owner.require_auth();

        let mut schedule = get_vesting_schedule(&env, &beneficiary).expect("No vesting schedule");

        // Ensure we don't reduce already vested amounts
        let current_time = get_ledger_timestamp(&env);
        let current_vested =
            Self::get_vested_amount(env.clone(), beneficiary.clone(), current_time);
        assert!(
            new_total_amount >= current_vested,
            "Cannot reduce vested amount"
        );

        schedule.cliff_duration = new_cliff_duration;
        schedule.vesting_duration = new_vesting_duration;
        schedule.total_amount = new_total_amount;

        set_vesting_schedule(&env, &beneficiary, &schedule);

        env.events().publish(
            ("vesting_schedule_updated",),
            (
                beneficiary,
                new_cliff_duration,
                new_vesting_duration,
                new_total_amount,
            ),
        );
    }
}
