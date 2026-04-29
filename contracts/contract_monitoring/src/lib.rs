//! # Contract Monitoring Dashboard
//!
//! Resolves issue #432: provides a centralised on-chain metrics store that
//! aggregates transaction volume, gas usage, error rates, storage utilisation,
//! active users, and function call frequency.
//!
//! ## Architecture
//! * Any contract in the workspace can call `record_call` / `record_error` to
//!   push metrics into this contract.
//! * The `get_dashboard` function returns a `DashboardSnapshot` suitable for
//!   off-chain dashboards (Grafana, custom web UI, etc.).
//! * Alert thresholds are configurable; when breached an `ALERT` event is emitted.

#![no_std]

use soroban_sdk::{contract, contractimpl, contracttype, symbol_short, Address, Env, Map, String};

// ── Types ─────────────────────────────────────────────────────────────────────

/// Alert thresholds.  A value of 0 means "disabled".
#[derive(Clone)]
#[contracttype]
pub struct AlertConfig {
    /// Emit alert when error rate (%) exceeds this value.
    pub max_error_rate_pct: u32,
    /// Emit alert when total gas used in a window exceeds this value.
    pub max_gas_per_window: u64,
    /// Emit alert when storage entries exceed this count.
    pub max_storage_entries: u32,
}

/// Per-function call statistics.
#[derive(Clone)]
#[contracttype]
pub struct FunctionStats {
    pub call_count: u64,
    pub error_count: u64,
    pub total_gas: u64,
    pub last_called_at: u64,
}

/// Top-level dashboard snapshot returned by `get_dashboard`.
#[derive(Clone)]
#[contracttype]
pub struct DashboardSnapshot {
    /// Total calls recorded across all functions.
    pub total_calls: u64,
    /// Total errors recorded.
    pub total_errors: u64,
    /// Error rate as a percentage (0–100).
    pub error_rate_pct: u32,
    /// Cumulative gas used.
    pub total_gas_used: u64,
    /// Number of distinct callers seen.
    pub active_users: u32,
    /// Number of distinct storage keys written.
    pub storage_entries: u32,
    /// Ledger timestamp of the snapshot.
    pub snapshot_at: u64,
    /// Whether any alert threshold is currently breached.
    pub alert_active: bool,
}

#[derive(Clone, PartialEq, Eq)]
#[contracttype]
pub enum DataKey {
    Admin,
    AlertConfig,
    TotalCalls,
    TotalErrors,
    TotalGas,
    ActiveUsers,
    StorageEntries,
    /// Per-function stats keyed by function name.
    FnStats(String),
    /// Tracks whether an address has been seen before.
    SeenUser(Address),
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[contracttype]
#[repr(u32)]
pub enum MonitoringError {
    NotInitialized = 1,
    AlreadyInitialized = 2,
    Unauthorized = 3,
}

// ── Contract ──────────────────────────────────────────────────────────────────

#[contract]
pub struct ContractMonitoring;

#[contractimpl]
impl ContractMonitoring {
    /// Initialise the monitoring contract.
    pub fn initialize(
        env: Env,
        admin: Address,
        alert_config: AlertConfig,
    ) -> Result<(), MonitoringError> {
        if env.storage().instance().has(&DataKey::Admin) {
            return Err(MonitoringError::AlreadyInitialized);
        }
        admin.require_auth();
        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage()
            .instance()
            .set(&DataKey::AlertConfig, &alert_config);
        Ok(())
    }

    /// Record a successful function call.
    ///
    /// `caller` – the address that invoked the function.
    /// `function_name` – name of the function called.
    /// `gas_used` – estimated gas consumed (pass 0 if unknown).
    pub fn record_call(
        env: Env,
        caller: Address,
        function_name: String,
        gas_used: u64,
    ) -> Result<(), MonitoringError> {
        Self::ensure_initialized(&env)?;

        // Update global counters.
        let calls: u64 = env
            .storage()
            .instance()
            .get(&DataKey::TotalCalls)
            .unwrap_or(0);
        env.storage()
            .instance()
            .set(&DataKey::TotalCalls, &(calls + 1));

        let gas: u64 = env
            .storage()
            .instance()
            .get(&DataKey::TotalGas)
            .unwrap_or(0);
        env.storage()
            .instance()
            .set(&DataKey::TotalGas, &(gas + gas_used));

        // Track unique callers.
        if !env
            .storage()
            .persistent()
            .has(&DataKey::SeenUser(caller.clone()))
        {
            env.storage()
                .persistent()
                .set(&DataKey::SeenUser(caller.clone()), &true);
            let users: u32 = env
                .storage()
                .instance()
                .get(&DataKey::ActiveUsers)
                .unwrap_or(0);
            env.storage()
                .instance()
                .set(&DataKey::ActiveUsers, &(users + 1));
        }

        // Update per-function stats.
        let key = DataKey::FnStats(function_name.clone());
        let mut stats: FunctionStats =
            env.storage().instance().get(&key).unwrap_or(FunctionStats {
                call_count: 0,
                error_count: 0,
                total_gas: 0,
                last_called_at: 0,
            });
        stats.call_count += 1;
        stats.total_gas += gas_used;
        stats.last_called_at = env.ledger().timestamp();
        env.storage().instance().set(&key, &stats);

        // Check alert thresholds.
        Self::check_alerts(&env, gas + gas_used);

        Ok(())
    }

    /// Record a failed function call / error.
    pub fn record_error(env: Env, function_name: String) -> Result<(), MonitoringError> {
        Self::ensure_initialized(&env)?;

        let errors: u64 = env
            .storage()
            .instance()
            .get(&DataKey::TotalErrors)
            .unwrap_or(0);
        env.storage()
            .instance()
            .set(&DataKey::TotalErrors, &(errors + 1));

        let key = DataKey::FnStats(function_name);
        let mut stats: FunctionStats =
            env.storage().instance().get(&key).unwrap_or(FunctionStats {
                call_count: 0,
                error_count: 0,
                total_gas: 0,
                last_called_at: 0,
            });
        stats.error_count += 1;
        env.storage().instance().set(&key, &stats);

        // Emit error event.
        env.events()
            .publish((symbol_short!("MON"), symbol_short!("ERROR")), errors + 1);

        Ok(())
    }

    /// Update storage-entry count (call after writes to tracked contracts).
    pub fn update_storage_count(env: Env, count: u32) -> Result<(), MonitoringError> {
        Self::ensure_initialized(&env)?;
        env.storage()
            .instance()
            .set(&DataKey::StorageEntries, &count);

        let config: AlertConfig = env.storage().instance().get(&DataKey::AlertConfig).unwrap();
        if config.max_storage_entries > 0 && count > config.max_storage_entries {
            env.events().publish(
                (symbol_short!("MON"), symbol_short!("ALERT")),
                symbol_short!("STORAGE"),
            );
        }

        Ok(())
    }

    /// Update alert thresholds (admin only).
    pub fn update_alert_config(env: Env, config: AlertConfig) -> Result<(), MonitoringError> {
        let admin = Self::get_admin(&env)?;
        admin.require_auth();
        env.storage().instance().set(&DataKey::AlertConfig, &config);
        Ok(())
    }

    // ── Queries ───────────────────────────────────────────────────────────────

    /// Return a full dashboard snapshot.
    pub fn get_dashboard(env: Env) -> Result<DashboardSnapshot, MonitoringError> {
        Self::ensure_initialized(&env)?;

        let total_calls: u64 = env
            .storage()
            .instance()
            .get(&DataKey::TotalCalls)
            .unwrap_or(0);
        let total_errors: u64 = env
            .storage()
            .instance()
            .get(&DataKey::TotalErrors)
            .unwrap_or(0);
        let total_gas: u64 = env
            .storage()
            .instance()
            .get(&DataKey::TotalGas)
            .unwrap_or(0);
        let active_users: u32 = env
            .storage()
            .instance()
            .get(&DataKey::ActiveUsers)
            .unwrap_or(0);
        let storage_entries: u32 = env
            .storage()
            .instance()
            .get(&DataKey::StorageEntries)
            .unwrap_or(0);

        let error_rate_pct = if total_calls > 0 {
            ((total_errors * 100) / total_calls) as u32
        } else {
            0
        };

        let config: AlertConfig = env.storage().instance().get(&DataKey::AlertConfig).unwrap();

        let alert_active = (config.max_error_rate_pct > 0
            && error_rate_pct > config.max_error_rate_pct)
            || (config.max_gas_per_window > 0 && total_gas > config.max_gas_per_window)
            || (config.max_storage_entries > 0 && storage_entries > config.max_storage_entries);

        Ok(DashboardSnapshot {
            total_calls,
            total_errors,
            error_rate_pct,
            total_gas_used: total_gas,
            active_users,
            storage_entries,
            snapshot_at: env.ledger().timestamp(),
            alert_active,
        })
    }

    /// Return per-function statistics.
    pub fn get_function_stats(
        env: Env,
        function_name: String,
    ) -> Result<FunctionStats, MonitoringError> {
        Self::ensure_initialized(&env)?;
        Ok(env
            .storage()
            .instance()
            .get(&DataKey::FnStats(function_name))
            .unwrap_or(FunctionStats {
                call_count: 0,
                error_count: 0,
                total_gas: 0,
                last_called_at: 0,
            }))
    }

    // ── Internal ──────────────────────────────────────────────────────────────

    fn ensure_initialized(env: &Env) -> Result<(), MonitoringError> {
        if env.storage().instance().has(&DataKey::Admin) {
            Ok(())
        } else {
            Err(MonitoringError::NotInitialized)
        }
    }

    fn get_admin(env: &Env) -> Result<Address, MonitoringError> {
        env.storage()
            .instance()
            .get(&DataKey::Admin)
            .ok_or(MonitoringError::NotInitialized)
    }

    fn check_alerts(env: &Env, total_gas: u64) {
        let config: AlertConfig = match env.storage().instance().get(&DataKey::AlertConfig) {
            Some(c) => c,
            None => return,
        };

        let total_calls: u64 = env
            .storage()
            .instance()
            .get(&DataKey::TotalCalls)
            .unwrap_or(0);
        let total_errors: u64 = env
            .storage()
            .instance()
            .get(&DataKey::TotalErrors)
            .unwrap_or(0);

        let error_rate = if total_calls > 0 {
            ((total_errors * 100) / total_calls) as u32
        } else {
            0
        };

        if config.max_error_rate_pct > 0 && error_rate > config.max_error_rate_pct {
            env.events().publish(
                (symbol_short!("MON"), symbol_short!("ALERT")),
                symbol_short!("ERRRATE"),
            );
        }

        if config.max_gas_per_window > 0 && total_gas > config.max_gas_per_window {
            env.events().publish(
                (symbol_short!("MON"), symbol_short!("ALERT")),
                symbol_short!("GAS"),
            );
        }
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod test {
    use super::*;
    use soroban_sdk::{testutils::Address as _, Env, String};

    fn default_config() -> AlertConfig {
        AlertConfig {
            max_error_rate_pct: 10,
            max_gas_per_window: 1_000_000,
            max_storage_entries: 10_000,
        }
    }

    fn setup(env: &Env) -> (ContractMonitoringClient, Address) {
        let contract_id = env.register_contract(None, ContractMonitoring);
        let client = ContractMonitoringClient::new(env, &contract_id);
        let admin = Address::generate(env);
        env.mock_all_auths();
        client.initialize(&admin, &default_config());
        (client, admin)
    }

    #[test]
    fn test_record_call_increments_counters() {
        let env = Env::default();
        let (client, _) = setup(&env);
        let caller = Address::generate(&env);

        client.record_call(&caller, &String::from_str(&env, "write_record"), &100);
        client.record_call(&caller, &String::from_str(&env, "write_record"), &150);

        let dash = client.get_dashboard();
        assert_eq!(dash.total_calls, 2);
        assert_eq!(dash.total_gas_used, 250);
        assert_eq!(dash.active_users, 1); // same caller
    }

    #[test]
    fn test_unique_user_tracking() {
        let env = Env::default();
        let (client, _) = setup(&env);

        for _ in 0..5 {
            let caller = Address::generate(&env);
            client.record_call(&caller, &String::from_str(&env, "read_record"), &50);
        }

        let dash = client.get_dashboard();
        assert_eq!(dash.active_users, 5);
    }

    #[test]
    fn test_error_rate_calculation() {
        let env = Env::default();
        let (client, _) = setup(&env);
        let caller = Address::generate(&env);

        for _ in 0..9 {
            client.record_call(&caller, &String::from_str(&env, "fn"), &10);
        }
        client.record_error(&String::from_str(&env, "fn"));

        let dash = client.get_dashboard();
        assert_eq!(dash.total_calls, 9);
        assert_eq!(dash.total_errors, 1);
        // error_rate = 1*100/9 = 11 (integer division)
        assert!(dash.error_rate_pct > 0);
    }

    #[test]
    fn test_function_stats() {
        let env = Env::default();
        let (client, _) = setup(&env);
        let caller = Address::generate(&env);
        let fn_name = String::from_str(&env, "initialize");

        client.record_call(&caller, &fn_name, &200);
        client.record_call(&caller, &fn_name, &300);

        let stats = client.get_function_stats(&fn_name);
        assert_eq!(stats.call_count, 2);
        assert_eq!(stats.total_gas, 500);
    }

    #[test]
    fn test_storage_count_alert() {
        let env = Env::default();
        let (client, _) = setup(&env);

        // Exceeds threshold of 10_000.
        client.update_storage_count(&15_000);
        let dash = client.get_dashboard();
        assert!(dash.alert_active);
    }

    #[test]
    fn test_double_initialize_fails() {
        let env = Env::default();
        let (client, _) = setup(&env);
        let admin2 = Address::generate(&env);
        assert_eq!(
            client.try_initialize(&admin2, &default_config()),
            Err(Ok(MonitoringError::AlreadyInitialized))
        );
    }
}
