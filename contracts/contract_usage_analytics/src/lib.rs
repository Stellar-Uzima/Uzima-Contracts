#![no_std]

use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype, symbol_short, Address, Env, String, Vec, Map,
};

#[derive(Clone)]
#[contracttype]
pub struct FunctionMetric {
    pub name: String,
    pub call_count: u64,
    pub total_cpu_usage: u64,
    pub total_ram_usage: u64,
    pub error_count: u64,
    pub avg_latency_ms: u64,
    pub last_called: u64,
}

#[derive(Clone)]
#[contracttype]
pub struct UserMetric {
    pub user: Address,
    pub total_calls: u64,
    pub last_active: u64,
}

#[derive(Clone)]
#[contracttype]
pub struct UsageSnapshot {
    pub timestamp: u64,
    pub total_calls: u64,
    pub active_users: u32,
    pub error_rate_bps: u32, // Basis points (1/100th of a percent)
}

#[derive(Clone)]
#[contracttype]
pub enum DataKey {
    Admin,
    FunctionMetric(String),
    UserMetric(Address),
    Snapshots,
    AllFunctions,
    ActiveUsers(u64), // Day-based key for active users
}

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    NotAuthorized = 1,
    AlreadyInitialized = 2,
    NotInitialized = 3,
    InvalidInput = 4,
}

#[contract]
pub struct ContractUsageAnalytics;

#[contractimpl]
impl ContractUsageAnalytics {
    pub fn initialize(env: Env, admin: Address) -> Result<(), Error> {
        if env.storage().instance().has(&DataKey::Admin) {
            return Err(Error::AlreadyInitialized);
        }
        env.storage().instance().set(&DataKey::Admin, &admin);
        Ok(())
    }

    pub fn record_call(
        env: Env,
        function_name: String,
        user: Address,
        cpu_usage: u64,
        ram_usage: u64,
        success: bool,
        latency_ms: u64,
    ) -> Result<(), Error> {
        // In a real scenario, we might want to restrict who can call this, 
        // or let any contract report its own usage.
        
        let timestamp = env.ledger().timestamp();
        
        // 1. Update Function Metrics
        let mut f_metric = env.storage().instance().get(&DataKey::FunctionMetric(function_name.clone()))
            .unwrap_or(FunctionMetric {
                name: function_name.clone(),
                call_count: 0,
                total_cpu_usage: 0,
                total_ram_usage: 0,
                error_count: 0,
                avg_latency_ms: 0,
                last_called: 0,
            });

        let new_count = f_metric.call_count.saturating_add(1);
        f_metric.total_cpu_usage = f_metric.total_cpu_usage.saturating_add(cpu_usage);
        f_metric.total_ram_usage = f_metric.total_ram_usage.saturating_add(ram_usage);
        if !success {
            f_metric.error_count = f_metric.error_count.saturating_add(1);
        }
        
        // Rolling average for latency
        let prev_total_latency = f_metric.avg_latency_ms.saturating_mul(f_metric.call_count);
        f_metric.avg_latency_ms = prev_total_latency.saturating_add(latency_ms).checked_div(new_count).unwrap_or(0);
        
        f_metric.call_count = new_count;
        f_metric.last_called = timestamp;
        
        env.storage().instance().set(&DataKey::FunctionMetric(function_name.clone()), &f_metric);

        // Track function name if new
        let mut all_functions: Vec<String> = env.storage().instance().get(&DataKey::AllFunctions).unwrap_or(Vec::new(&env));
        let mut exists = false;
        for f in all_functions.iter() {
            if f == function_name {
                exists = true;
                break;
            }
        }
        if !exists {
            all_functions.push_back(function_name.clone());
            env.storage().instance().set(&DataKey::AllFunctions, &all_functions);
        }

        // 2. Update User Metrics
        let mut u_metric = env.storage().instance().get(&DataKey::UserMetric(user.clone()))
            .unwrap_or(UserMetric {
                user: user.clone(),
                total_calls: 0,
                last_active: 0,
            });
        
        u_metric.total_calls = u_metric.total_calls.saturating_add(1);
        u_metric.last_active = timestamp;
        env.storage().instance().set(&DataKey::UserMetric(user.clone()), &u_metric);

        // 3. Track daily active users
        let day_id = timestamp / 86400;
        let mut active_users_today: Vec<Address> = env.storage().instance().get(&DataKey::ActiveUsers(day_id)).unwrap_or(Vec::new(&env));
        let mut user_exists = false;
        for u in active_users_today.iter() {
            if u == user {
                user_exists = true;
                break;
            }
        }
        if !user_exists {
            active_users_today.push_back(user.clone());
            env.storage().instance().set(&DataKey::ActiveUsers(day_id), &active_users_today);
        }

        // 4. Publish Event
        env.events().publish(
            (symbol_short!("usage"), function_name),
            (user, success, cpu_usage, ram_usage)
        );

        Ok(())
    }

    pub fn take_snapshot(env: Env) -> Result<UsageSnapshot, Error> {
        let timestamp = env.ledger().timestamp();
        let day_id = timestamp / 86400;
        
        let active_users_today: Vec<Address> = env.storage().instance().get(&DataKey::ActiveUsers(day_id)).unwrap_or(Vec::new(&env));
        let all_functions: Vec<String> = env.storage().instance().get(&DataKey::AllFunctions).unwrap_or(Vec::new(&env));
        
        let mut total_calls = 0;
        let mut total_errors = 0;
        
        for f_name in all_functions.iter() {
            if let Some(metric) = env.storage().instance().get::<_, FunctionMetric>(&DataKey::FunctionMetric(f_name)) {
                total_calls = total_calls.saturating_add(metric.call_count);
                total_errors = total_errors.saturating_add(metric.error_count);
            }
        }
        
        let error_rate_bps = if total_calls > 0 {
            (total_errors.saturating_mul(10000)).checked_div(total_calls).unwrap_or(0) as u32
        } else {
            0
        };
        
        let snapshot = UsageSnapshot {
            timestamp,
            total_calls,
            active_users: active_users_today.len(),
            error_rate_bps,
        };
        
        let mut snapshots: Vec<UsageSnapshot> = env.storage().instance().get(&DataKey::Snapshots).unwrap_or(Vec::new(&env));
        snapshots.push_back(snapshot.clone());
        
        // Keep only last 30 snapshots
        if snapshots.len() > 30 {
            snapshots.remove(0);
        }
        
        env.storage().instance().set(&DataKey::Snapshots, &snapshots);
        
        Ok(snapshot)
    }

    pub fn get_function_metrics(env: Env, function_name: String) -> Option<FunctionMetric> {
        env.storage().instance().get(&DataKey::FunctionMetric(function_name))
    }

    pub fn get_user_metrics(env: Env, user: Address) -> Option<UserMetric> {
        env.storage().instance().get(&DataKey::UserMetric(user))
    }

    pub fn get_all_functions(env: Env) -> Vec<String> {
        env.storage().instance().get(&DataKey::AllFunctions).unwrap_or(Vec::new(&env))
    }

    pub fn get_snapshots(env: Env) -> Vec<UsageSnapshot> {
        env.storage().instance().get(&DataKey::Snapshots).unwrap_or(Vec::new(&env))
    }
}
