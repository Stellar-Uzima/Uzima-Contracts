use soroban_sdk::{contract, contractimpl, contractmeta, symbol_short, Address, Env, Symbol, Vec as SVec};

use crate::storage::{get_config, get_last_good, require_owner, set_config, set_last_good, validate_providers};
use crate::types::{AdapterConfig, LastGood, OracleReading};

// Interface for provider contracts: fn latest() -> (i128, u64)
pub trait OracleProviderInterface {
    fn latest(env: &Env, contract: &Address) -> OracleReading;
}

fn call_latest(env: &Env, addr: &Address) -> Option<OracleReading> {
    // Expect provider implements a function named "latest" returning (i128, u64)
    // Use a raw invocation to avoid generating client code. If the call traps, the whole tx reverts.
    let sym: Symbol = symbol_short!("latest");
    let tup: (i128, u64) = env.invoke_contract(addr, &sym, soroban_sdk::vec![&env]);
    Some(OracleReading { value: tup.0, timestamp: tup.1 })
}

fn median_of_sorted(values: &SVec<i128>) -> i128 {
    let n = values.len();
    if n % 2 == 1 {
        values.get_unchecked(n/2)
    } else {
        let a = values.get_unchecked(n/2 - 1);
        let b = values.get_unchecked(n/2);
        // average rounding toward zero
        (a + b) / 2
    }
}

fn sort_in_place(env: &Env, mut arr: SVec<i128>) -> SVec<i128> {
    // Simple insertion sort, small N expected
    for i in 1..arr.len() {
        let key = arr.get_unchecked(i);
        let mut j: i32 = i as i32 - 1;
        while j >= 0 && arr.get_unchecked(j as u32) > key {
            let prev = arr.get_unchecked(j as u32);
            arr.set((j+1) as u32, prev);
            j -= 1;
        }
        arr.set((j+1) as u32, key);
    }
    arr
}

fn apply_ema(prev: Option<i128>, curr: i128, ema_bps: u32) -> i128 {
    if ema_bps == 0 { return curr; }
    let p = prev.unwrap_or(curr);
    // new = (ema_bps*p + (10000-ema_bps)*curr)/10000
    let a = (ema_bps as i128) * p;
    let b = ((10_000u32 - ema_bps) as i128) * curr;
    (a + b) / 10_000i128
}

#[contract]
pub struct OracleAdapterContract;

// Metadata that is added on to every WASM custom section
contractmeta!(
    key = "Description",
    val = "Pluggable Oracle Adapter with Median and TTL"
);

#[contractimpl]
impl OracleAdapterContract {
    pub fn init(env: Env, owner: Address, ttl_secs: u64, providers: SVec<Address>, min_required: u32, fallback_to_last_good: bool, ema_bps: u32) {
        assert!(env.storage().instance().has(&crate::types::DataKey::Config) == false, "inited");
        assert!(ema_bps <= 10_000, "ema_bps<=10000");
        assert!(ttl_secs > 0, "ttl>0");
        owner.require_auth();
        validate_providers(&env, &providers, min_required);
        let cfg = AdapterConfig { owner: owner.clone(), ttl_secs, providers, min_required, fallback_to_last_good, ema_bps };
        set_config(&env, &cfg);
    }

    // Admin
    pub fn set_ttl(env: Env, ttl_secs: u64) { require_owner(&env); assert!(ttl_secs>0, "ttl>0"); let mut c = get_config(&env); c.ttl_secs = ttl_secs; set_config(&env,&c); }
    pub fn set_min_required(env: Env, min_required: u32) { require_owner(&env); let mut c = get_config(&env); validate_providers(&env, &c.providers, min_required); c.min_required = min_required; set_config(&env,&c); }
    pub fn set_providers(env: Env, providers: SVec<Address>) { require_owner(&env); let mut c = get_config(&env); validate_providers(&env, &providers, c.min_required); c.providers = providers; set_config(&env,&c); }
    pub fn set_fallback(env: Env, enabled: bool) { require_owner(&env); let mut c = get_config(&env); c.fallback_to_last_good = enabled; set_config(&env,&c); }
    pub fn set_ema_bps(env: Env, ema_bps: u32) { require_owner(&env); assert!(ema_bps <= 10_000, "ema_bps<=10000"); let mut c = get_config(&env); c.ema_bps = ema_bps; set_config(&env,&c); }

    // Read latest aggregated data
    pub fn get_latest_data(env: Env) -> (i128, u64) {
        let cfg = get_config(&env);
        let now = env.ledger().timestamp();
        let min_ts = now.saturating_sub(cfg.ttl_secs);

        let mut fresh_values: SVec<i128> = SVec::new(&env);
        let mut newest_ts: u64 = 0;
        let mut fresh_count: u32 = 0;

        for i in 0..cfg.providers.len() {
            let addr = cfg.providers.get_unchecked(i);
            let r = call_latest(&env, &addr).expect("provider call failed");
            if r.timestamp >= min_ts && r.timestamp <= now {
                fresh_values.push_back(r.value);
                fresh_count += 1;
                if r.timestamp > newest_ts { newest_ts = r.timestamp; }
            }
        }

        if fresh_count < cfg.min_required {
            if cfg.fallback_to_last_good {
                if let Some(last) = get_last_good(&env) {
                    return (last.value, last.timestamp);
                }
            }
            panic!("STALE_OR_INSUFFICIENT_SOURCES");
        }

        // sort and median
        let sorted = sort_in_place(&env, fresh_values);
        let median = median_of_sorted(&sorted);
        let smoothed = apply_ema(get_last_good(&env).map(|l| l.value), median, cfg.ema_bps);
        set_last_good(&env, &LastGood { value: smoothed, timestamp: newest_ts });
        (smoothed, newest_ts)
    }

    // Views
    pub fn get_config_view(env: Env) -> AdapterConfig { get_config(&env) }
    pub fn get_last_good_view(env: Env) -> Option<LastGood> { get_last_good(&env) }
}
