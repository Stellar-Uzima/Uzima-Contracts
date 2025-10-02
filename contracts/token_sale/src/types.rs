use soroban_sdk::{contracttype, Address, Env};

#[derive(Clone)]
#[contracttype]
pub struct SalePhase {
    pub start_time: u64,
    pub end_time: u64,
    pub price_per_token: u128, // Price in payment token units per SUT token
    pub max_tokens: u128,
    pub sold_tokens: u128,
    pub per_address_cap: u128,
    pub is_active: bool,
}

#[derive(Clone)]
#[contracttype]
pub struct VestingSchedule {
    pub cliff_duration: u64,   // Cliff period in seconds
    pub vesting_duration: u64, // Total vesting period in seconds
    pub start_time: u64,
    pub total_amount: u128,
    pub released_amount: u128,
}

#[derive(Clone)]
#[contracttype]
pub struct Contribution {
    pub amount: u128,           // Amount contributed in payment token
    pub tokens_allocated: u128, // SUT tokens allocated
    pub phase_id: u32,
    pub timestamp: u64,
    pub claimed: bool,
}

#[derive(Clone)]
#[contracttype]
pub struct SaleConfig {
    pub token_address: Address, // SUT token contract
    pub treasury: Address,      // Treasury address for funds
    pub soft_cap: u128,         // Minimum raise target
    pub hard_cap: u128,         // Maximum raise target
    pub is_finalized: bool,
    pub refunds_enabled: bool,
}

#[contracttype]
pub enum DataKey {
    Config,
    Owner,
    Paused,
    TotalRaised,
    PhaseCount,
    CurrentPhase,
    Phase(u32),
    Contribution(Address),
    PhaseContribution(Address, u32),
    SupportedToken(Address),
    VestingSchedule(Address),
    VestingContract,
}

pub fn get_ledger_timestamp(env: &Env) -> u64 {
    env.ledger().timestamp()
}
