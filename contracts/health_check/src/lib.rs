#![no_std]

use soroban_sdk::{contract, contractimpl, contracttype, Env, String, Vec};

/// Contract version constant
const VERSION: &str = "1.0.0";

/// Comprehensive health check result
#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub struct ContractHealth {
    /// Contract version
    pub version: String,
    /// Whether the contract is paused
    pub is_paused: bool,
    /// Storage usage in bytes
    pub storage_usage: u64,
    /// Last activity timestamp
    pub last_activity: u64,
    /// Total operations count
    pub total_operations: u64,
    /// Failed operations count
    pub failed_operations: u64,
    /// Success rate (percentage * 100)
    pub success_rate: u32,
}

/// Detailed monitoring metrics
#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub struct MonitoringMetrics {
    /// Contract version
    pub version: String,
    /// Pause status
    pub is_paused: bool,
    /// Storage utilization in bytes
    pub storage_usage: u64,
    /// Recent activity timestamp
    pub last_activity: u64,
    /// Total error count
    pub error_count: u64,
    /// Average gas usage per operation
    pub avg_gas_usage: u64,
    /// Peak gas usage recorded
    pub peak_gas_usage: u64,
    /// Total operations processed
    pub total_operations: u64,
}

/// Gas usage statistics
#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub struct GasMetrics {
    /// Current operation gas usage
    pub current_usage: u64,
    /// Average gas usage
    pub average_usage: u64,
    /// Peak gas usage
    pub peak_usage: u64,
    /// Total gas consumed
    pub total_consumed: u64,
    /// Number of operations measured
    pub operation_count: u64,
}

/// Error rate tracking
#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub struct ErrorMetrics {
    /// Total errors
    pub total_errors: u64,
    /// Errors in last hour
    pub recent_errors: u64,
    /// Error rate (errors per 1000 operations)
    pub error_rate: u32,
    /// Last error timestamp
    pub last_error_time: u64,
    /// Most common error code
    pub common_error_code: u32,
}

/// Storage key enumeration
#[derive(Clone)]
#[contracttype]
pub enum DataKey {
    Initialized,
    Paused,
    Admin,
    LastActivity,
    TotalOperations,
    FailedOperations,
    TotalGasUsed,
    PeakGasUsage,
    TotalErrors,
    RecentErrors,
    LastErrorTime,
    CommonErrorCode,
}

#[contract]
pub struct HealthCheckContract;

#[contractimpl]
impl HealthCheckContract {
    /// Initialize the health check contract
    pub fn initialize(env: Env) -> bool {
        if env.storage().instance().has(&DataKey::Initialized) {
            return false;
        }

        env.storage().instance().set(&DataKey::Initialized, &true);
        env.storage().instance().set(&DataKey::Paused, &false);
        env.storage().instance().set(&DataKey::LastActivity, &env.ledger().timestamp());
        env.storage().instance().set(&DataKey::TotalOperations, &0u64);
        env.storage().instance().set(&DataKey::FailedOperations, &0u64);
        env.storage().instance().set(&DataKey::TotalGasUsed, &0u64);
        env.storage().instance().set(&DataKey::PeakGasUsage, &0u64);
        env.storage().instance().set(&DataKey::TotalErrors, &0u64);
        env.storage().instance().set(&DataKey::RecentErrors, &0u64);
        env.storage().instance().set(&DataKey::LastErrorTime, &0u64);
        env.storage().instance().set(&DataKey::CommonErrorCode, &0u32);

        true
    }

    /// Get comprehensive health check
    pub fn health_check(env: Env) -> ContractHealth {
        let version = String::from_str(&env, VERSION);
        let is_paused = Self::is_paused(&env);
        let storage_usage = Self::storage_usage(&env);
        let last_activity = Self::last_activity(&env);
        let total_operations = Self::get_total_operations(&env);
        let failed_operations = Self::get_failed_operations(&env);
        
        let success_rate = if total_operations > 0 {
            let successful = total_operations.saturating_sub(failed_operations);
            ((successful * 10000) / total_operations) as u32
        } else {
            10000u32
        };

        ContractHealth {
            version,
            is_paused,
            storage_usage,
            last_activity,
            total_operations,
            failed_operations,
            success_rate,
        }
    }

    /// Get detailed monitoring metrics
    pub fn get_monitoring_metrics(env: Env) -> MonitoringMetrics {
        let version = String::from_str(&env, VERSION);
        let total_ops = Self::get_total_operations(&env);
        let total_gas = Self::get_total_gas_used(&env);
        
        let avg_gas_usage = if total_ops > 0 {
            total_gas / total_ops
        } else {
            0
        };

        MonitoringMetrics {
            version,
            is_paused: Self::is_paused(&env),
            storage_usage: Self::storage_usage(&env),
            last_activity: Self::last_activity(&env),
            error_count: Self::get_total_errors(&env),
            avg_gas_usage,
            peak_gas_usage: Self::get_peak_gas_usage(&env),
            total_operations: total_ops,
        }
    }

    /// Get gas usage metrics
    pub fn get_gas_metrics(env: Env) -> GasMetrics {
        let total_ops = Self::get_total_operations(&env);
        let total_gas = Self::get_total_gas_used(&env);
        
        let avg_usage = if total_ops > 0 {
            total_gas / total_ops
        } else {
            0
        };

        GasMetrics {
            current_usage: 0, // Would be set during operation
            average_usage: avg_usage,
            peak_usage: Self::get_peak_gas_usage(&env),
            total_consumed: total_gas,
            operation_count: total_ops,
        }
    }

    /// Get error rate metrics
    pub fn get_error_metrics(env: Env) -> ErrorMetrics {
        let total_ops = Self::get_total_operations(&env);
        let total_errors = Self::get_total_errors(&env);
        
        let error_rate = if total_ops > 0 {
            ((total_errors * 1000) / total_ops) as u32
        } else {
            0
        };

        ErrorMetrics {
            total_errors,
            recent_errors: Self::get_recent_errors(&env),
            error_rate,
            last_error_time: Self::get_last_error_time(&env),
            common_error_code: Self::get_common_error_code(&env),
        }
    }

    /// Record an operation
    pub fn record_operation(env: Env, gas_used: u64, success: bool) {
        Self::update_last_activity(&env);
        Self::increment_total_operations(&env);
        Self::update_gas_metrics(&env, gas_used);
        
        if !success {
            Self::increment_failed_operations(&env);
        }
    }

    /// Record an error
    pub fn record_error(env: Env, error_code: u32) {
        Self::increment_total_errors(&env);
        Self::increment_recent_errors(&env);
        env.storage().instance().set(&DataKey::LastErrorTime, &env.ledger().timestamp());
        env.storage().instance().set(&DataKey::CommonErrorCode, &error_code);
    }

    /// Set pause status
    pub fn set_paused(env: Env, paused: bool) {
        env.storage().instance().set(&DataKey::Paused, &paused);
    }

    /// Check if contract is paused
    pub fn is_paused(env: &Env) -> bool {
        env.storage()
            .instance()
            .get(&DataKey::Paused)
            .unwrap_or(false)
    }

    /// Get storage usage estimate
    pub fn storage_usage(env: &Env) -> u64 {
        // Estimate based on stored data
        let base_size = 1024u64; // Base contract size
        let operations = Self::get_total_operations(env);
        base_size + (operations * 32) // Rough estimate
    }

    /// Get last activity timestamp
    pub fn last_activity(env: &Env) -> u64 {
        env.storage()
            .instance()
            .get(&DataKey::LastActivity)
            .unwrap_or(0)
    }

    /// Update last activity timestamp
    fn update_last_activity(env: &Env) {
        env.storage()
            .instance()
            .set(&DataKey::LastActivity, &env.ledger().timestamp());
    }

    /// Get total operations
    fn get_total_operations(env: &Env) -> u64 {
        env.storage()
            .instance()
            .get(&DataKey::TotalOperations)
            .unwrap_or(0)
    }

    /// Increment total operations
    fn increment_total_operations(env: &Env) {
        let current = Self::get_total_operations(env);
        env.storage()
            .instance()
            .set(&DataKey::TotalOperations, &(current + 1));
    }

    /// Get failed operations
    fn get_failed_operations(env: &Env) -> u64 {
        env.storage()
            .instance()
            .get(&DataKey::FailedOperations)
            .unwrap_or(0)
    }

    /// Increment failed operations
    fn increment_failed_operations(env: &Env) {
        let current = Self::get_failed_operations(env);
        env.storage()
            .instance()
            .set(&DataKey::FailedOperations, &(current + 1));
    }

    /// Get total gas used
    fn get_total_gas_used(env: &Env) -> u64 {
        env.storage()
            .instance()
            .get(&DataKey::TotalGasUsed)
            .unwrap_or(0)
    }

    /// Get peak gas usage
    fn get_peak_gas_usage(env: &Env) -> u64 {
        env.storage()
            .instance()
            .get(&DataKey::PeakGasUsage)
            .unwrap_or(0)
    }

    /// Update gas metrics
    fn update_gas_metrics(env: &Env, gas_used: u64) {
        let total = Self::get_total_gas_used(env);
        env.storage()
            .instance()
            .set(&DataKey::TotalGasUsed, &(total + gas_used));

        let peak = Self::get_peak_gas_usage(env);
        if gas_used > peak {
            env.storage().instance().set(&DataKey::PeakGasUsage, &gas_used);
        }
    }

    /// Get total errors
    fn get_total_errors(env: &Env) -> u64 {
        env.storage()
            .instance()
            .get(&DataKey::TotalErrors)
            .unwrap_or(0)
    }

    /// Increment total errors
    fn increment_total_errors(env: &Env) {
        let current = Self::get_total_errors(env);
        env.storage()
            .instance()
            .set(&DataKey::TotalErrors, &(current + 1));
    }

    /// Get recent errors
    fn get_recent_errors(env: &Env) -> u64 {
        env.storage()
            .instance()
            .get(&DataKey::RecentErrors)
            .unwrap_or(0)
    }

    /// Increment recent errors
    fn increment_recent_errors(env: &Env) {
        let current = Self::get_recent_errors(env);
        env.storage()
            .instance()
            .set(&DataKey::RecentErrors, &(current + 1));
    }

    /// Get last error time
    fn get_last_error_time(env: &Env) -> u64 {
        env.storage()
            .instance()
            .get(&DataKey::LastErrorTime)
            .unwrap_or(0)
    }

    /// Get common error code
    fn get_common_error_code(env: &Env) -> u32 {
        env.storage()
            .instance()
            .get(&DataKey::CommonErrorCode)
            .unwrap_or(0)
    }

    /// Reset recent errors (for hourly cleanup)
    pub fn reset_recent_errors(env: Env) {
        env.storage().instance().set(&DataKey::RecentErrors, &0u64);
    }

    /// Get alert thresholds status
    pub fn check_alert_thresholds(env: Env) -> Vec<String> {
        let mut alerts = Vec::new(&env);
        
        let health = Self::health_check(env.clone());
        
        // Check error rate threshold (> 5%)
        if health.success_rate < 9500 {
            alerts.push_back(String::from_str(&env, "High error rate detected"));
        }
        
        // Check storage usage threshold (> 80% of estimate)
        if health.storage_usage > 8192 {
            alerts.push_back(String::from_str(&env, "High storage usage"));
        }
        
        // Check if paused
        if health.is_paused {
            alerts.push_back(String::from_str(&env, "Contract is paused"));
        }
        
        // Check inactivity (> 1 hour)
        let current_time = env.ledger().timestamp();
        if current_time > health.last_activity + 3600 {
            alerts.push_back(String::from_str(&env, "No recent activity"));
        }
        
        alerts
    }
}

#[cfg(test)]
mod test;
