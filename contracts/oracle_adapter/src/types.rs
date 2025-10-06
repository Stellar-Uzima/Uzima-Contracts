use soroban_sdk::{contracttype, Address};

#[derive(Clone)]
#[contracttype]
pub struct OracleReading {
    pub value: i128,
    pub timestamp: u64,
}

#[derive(Clone)]
#[contracttype]
pub struct AdapterConfig {
    pub owner: Address,
    pub ttl_secs: u64,
    pub providers: Vec<Address>,
    pub min_required: u32,
    pub fallback_to_last_good: bool,
    pub ema_bps: u32, // 0..=10_000; smoothing: new = (ema_bps*prev + (10000-ema_bps)*curr)/10000
}

#[derive(Clone)]
#[contracttype]
pub struct LastGood {
    pub value: i128,
    pub timestamp: u64,
}

#[contracttype]
pub enum DataKey {
    Config,
    LastGood,
}
