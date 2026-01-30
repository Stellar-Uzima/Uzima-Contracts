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

        set_vesting_schedule(&env, &beneficiary, &schedule);

        env.events().publish(
            ("vesting_schedule_created",),
            (beneficiary, cliff_duration, vesting_duration, total_amount),
        );
    }

    /// Release vested tokens to beneficiary
    pub fn release_tokens(env: Env, beneficiary: Address) -> u128 {
        beneficiary.require_auth();

        let releasable = Self::get_releasable_amount(env.clone(), beneficiary.clone());
        assert!(releasable > 0, "No tokens to release");

        // Update released amount
        let mut schedule = get_vesting_schedule(&env, &beneficiary).expect("No vesting schedule");
        schedule.released_amount += releasable;
        set_vesting_schedule(&env, &beneficiary, &schedule);

        // Transfer tokens
        let config = get_config(&env);
        let token_client = token::Client::new(&env, &config.token_address);
        token_client.transfer(
            &env.current_contract_address(),
            &beneficiary,
            &(releasable as i128),
        );

        env.events()
            .publish(("tokens_released",), (beneficiary, releasable));

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
