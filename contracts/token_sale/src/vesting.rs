use soroban_sdk::{contracttype, symbol_short, Address, Env, Map, Symbol};

const VESTING_SCHEDULES: Symbol = symbol_short!("vestsched");

#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub struct VestingSchedule {
    pub total_amount: i128,
    pub start_time: u64,
    pub duration: u64,
    pub cliff: u64,
    pub released_amount: i128,
}

pub fn create_vesting_schedule(
    env: &Env,
    beneficiary: Address,
    total_amount: i128,
    start_time: u64,
    duration: u64,
    cliff: u64,
) {
    if total_amount <= 0 || duration == 0 {
        panic!("Invalid vesting parameters");
    }

    let schedule = VestingSchedule {
        total_amount,
        start_time,
        duration,
        cliff,
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

pub fn get_vesting_schedule(env: &Env, beneficiary: Address) -> Option<VestingSchedule> {
    let schedules: Map<Address, VestingSchedule> = env
        .storage()
        .persistent()
        .get(&VESTING_SCHEDULES)
        .unwrap_or(Map::new(env));
    schedules.get(beneficiary)
}