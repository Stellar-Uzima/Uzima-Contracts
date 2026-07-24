//! Security telemetry and alerting for suspicious access patterns.
//!
//! This module provides structured tracking and alerting for security-relevant
//! access patterns including failed authentication attempts, suspicious
//! address behavior, rate limit violations, and unauthorized access attempts.

#![no_std]

use soroban_sdk::{
    contracttype, symbol_short, Address, BytesN, Env, Symbol,
};

use super::MonitoringError;

// ==================== Types ====================

#[derive(Clone, Copy, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[contracttype]
pub enum SecurityEventType {
    AuthFailure = 1,
    UnauthorizedAccess = 2,
    RateLimitExceeded = 3,
    SuspiciousPattern = 4,
    BruteForceAttempt = 5,
    PrivilegeEscalation = 6,
    DataAccessViolation = 7,
}

#[derive(Clone)]
#[contracttype]
pub struct SecurityAlert {
    pub event_type: SecurityEventType,
    pub source_address: Address,
    pub target_function: soroban_sdk::String,
    pub severity: u32,
    pub timestamp: u64,
    pub occurrence_count: u32,
}

#[derive(Clone)]
#[contracttype]
pub struct SecurityAlertConfig {
    pub max_auth_failures_per_window: u32,
    pub max_rate_limit_violations: u32,
    pub window_duration_seconds: u64,
    pub auto_lock_threshold: u32,
}

#[derive(Clone)]
#[contracttype]
pub struct SecuritySnapshot {
    pub total_auth_failures: u64,
    pub total_unauthorized_access: u64,
    pub total_rate_limit_violations: u64,
    pub total_suspicious_patterns: u64,
    pub total_brute_force_attempts: u64,
    pub active_alerts: u32,
    pub locked_addresses: u32,
    pub snapshot_at: u64,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[contracttype]
pub enum SecurityDataKey {
    AuthFailureCount(Address),
    UnauthorizedAccessCount(Address),
    RateLimitCount(Address),
    SuspiciousPatternCount(Address),
    BruteForceCount(Address),
    LockStatus(Address),
    AlertConfig,
    TotalAuthFailures,
    TotalUnauthorizedAccess,
    TotalRateLimitViolations,
    TotalSuspiciousPatterns,
    TotalBruteForceAttempts,
    ActiveAlertCount,
    LockedAddressCount,
    SecurityAlert(u32),
}

// ==================== Functions ====================

pub fn initialize_security_config(
    env: &Env,
    config: &SecurityAlertConfig,
) {
    env.storage()
        .instance()
        .set(&SecurityDataKey::AlertConfig, config);
    env.storage()
        .instance()
        .set(&SecurityDataKey::TotalAuthFailures, &0u64);
    env.storage()
        .instance()
        .set(&SecurityDataKey::TotalUnauthorizedAccess, &0u64);
    env.storage()
        .instance()
        .set(&SecurityDataKey::TotalRateLimitViolations, &0u64);
    env.storage()
        .instance()
        .set(&SecurityDataKey::TotalSuspiciousPatterns, &0u64);
    env.storage()
        .instance()
        .set(&SecurityDataKey::TotalBruteForceAttempts, &0u64);
    env.storage()
        .instance()
        .set(&SecurityDataKey::ActiveAlertCount, &0u32);
    env.storage()
        .instance()
        .set(&SecurityDataKey::LockedAddressCount, &0u32);
}

pub fn record_security_event(
    env: &Env,
    event_type: SecurityEventType,
    source: &Address,
    function_name: &soroban_sdk::String,
) -> Result<(), MonitoringError> {
    let config: SecurityAlertConfig = env
        .storage()
        .instance()
        .get(&SecurityDataKey::AlertConfig)
        .ok_or(MonitoringError::NotInitialized)?;

    if is_address_locked(env, source) {
        return Err(MonitoringError::Unauthorized);
    }

    let (count_key, total_key) = match event_type {
        SecurityEventType::AuthFailure => (
            SecurityDataKey::AuthFailureCount(source.clone()),
            SecurityDataKey::TotalAuthFailures,
        ),
        SecurityEventType::UnauthorizedAccess | SecurityEventType::DataAccessViolation => (
            SecurityDataKey::UnauthorizedAccessCount(source.clone()),
            SecurityDataKey::TotalUnauthorizedAccess,
        ),
        SecurityEventType::RateLimitExceeded => (
            SecurityDataKey::RateLimitCount(source.clone()),
            SecurityDataKey::TotalRateLimitViolations,
        ),
        SecurityEventType::SuspiciousPattern => (
            SecurityDataKey::SuspiciousPatternCount(source.clone()),
            SecurityDataKey::TotalSuspiciousPatterns,
        ),
        SecurityEventType::BruteForceAttempt => (
            SecurityDataKey::BruteForceCount(source.clone()),
            SecurityDataKey::TotalBruteForceAttempts,
        ),
        SecurityEventType::PrivilegeEscalation => (
            SecurityDataKey::UnauthorizedAccessCount(source.clone()),
            SecurityDataKey::TotalUnauthorizedAccess,
        ),
    };

    let count: u64 = env.storage().instance().get(&count_key).unwrap_or(0);
    let new_count = count.saturating_add(1);
    env.storage().instance().set(&count_key, &new_count);

    let total: u64 = env.storage().instance().get(&total_key).unwrap_or(0);
    env.storage()
        .instance()
        .set(&total_key, &total.saturating_add(1));

    let alert_threshold = match event_type {
        SecurityEventType::AuthFailure => config.max_auth_failures_per_window,
        SecurityEventType::BruteForceAttempt => config.auto_lock_threshold / 2,
        _ => config.max_rate_limit_violations,
    };

    if alert_threshold > 0 && (new_count as u32) >= alert_threshold {
        emit_security_alert(
            env,
            event_type,
            source,
            function_name,
            new_count as u32,
        );

        if config.auto_lock_threshold > 0
            && event_type == SecurityEventType::BruteForceAttempt
            && (new_count as u32) >= config.auto_lock_threshold
        {
            lock_address(env, source);
        }
    }

    emit_security_event(env, event_type, source, function_name);

    Ok(())
}

pub fn is_address_locked(env: &Env, address: &Address) -> bool {
    env.storage()
        .persistent()
        .get(&SecurityDataKey::LockStatus(address.clone()))
        .unwrap_or(false)
}

pub fn lock_address(env: &Env, address: &Address) {
    let was_locked = is_address_locked(env, address);
    env.storage().persistent().set(
        &SecurityDataKey::LockStatus(address.clone()),
        &true,
    );

    if !was_locked {
        let count: u32 = env
            .storage()
            .instance()
            .get(&SecurityDataKey::LockedAddressCount)
            .unwrap_or(0);
        env.storage()
            .instance()
            .set(&SecurityDataKey::LockedAddressCount, &count.saturating_add(1));

        env.events().publish(
            (symbol_short!("SEC"), symbol_short!("LOCK")),
            (address.clone(), env.ledger().timestamp()),
        );
    }
}

pub fn unlock_address(
    env: &Env,
    admin: &Address,
    address: &Address,
) -> Result<(), MonitoringError> {
    let stored_admin: Address = env
        .storage()
        .instance()
        .get(&super::DataKey::Admin)
        .ok_or(MonitoringError::NotInitialized)?;

    if *admin != stored_admin {
        return Err(MonitoringError::Unauthorized);
    }

    let was_locked = is_address_locked(env, address);
    env.storage().persistent().set(
        &SecurityDataKey::LockStatus(address.clone()),
        &false,
    );

    if was_locked {
        let count: u32 = env
            .storage()
            .instance()
            .get(&SecurityDataKey::LockedAddressCount)
            .unwrap_or(0);
        if count > 0 {
            env.storage()
                .instance()
                .set(&SecurityDataKey::LockedAddressCount, &count - 1);
        }

        env.events().publish(
            (symbol_short!("SEC"), symbol_short!("UNLOCK")),
            (address.clone(), env.ledger().timestamp()),
        );
    }

    Ok(())
}

pub fn get_security_snapshot(env: &Env) -> Result<SecuritySnapshot, MonitoringError> {
    if !env.storage().instance().has(&SecurityDataKey::AlertConfig) {
        return Err(MonitoringError::NotInitialized);
    }

    let total_auth_failures: u64 = env
        .storage()
        .instance()
        .get(&SecurityDataKey::TotalAuthFailures)
        .unwrap_or(0);
    let total_unauthorized_access: u64 = env
        .storage()
        .instance()
        .get(&SecurityDataKey::TotalUnauthorizedAccess)
        .unwrap_or(0);
    let total_rate_limit_violations: u64 = env
        .storage()
        .instance()
        .get(&SecurityDataKey::TotalRateLimitViolations)
        .unwrap_or(0);
    let total_suspicious_patterns: u64 = env
        .storage()
        .instance()
        .get(&SecurityDataKey::TotalSuspiciousPatterns)
        .unwrap_or(0);
    let total_brute_force_attempts: u64 = env
        .storage()
        .instance()
        .get(&SecurityDataKey::TotalBruteForceAttempts)
        .unwrap_or(0);
    let active_alerts: u32 = env
        .storage()
        .instance()
        .get(&SecurityDataKey::ActiveAlertCount)
        .unwrap_or(0);
    let locked_addresses: u32 = env
        .storage()
        .instance()
        .get(&SecurityDataKey::LockedAddressCount)
        .unwrap_or(0);

    Ok(SecuritySnapshot {
        total_auth_failures,
        total_unauthorized_access,
        total_rate_limit_violations,
        total_suspicious_patterns,
        total_brute_force_attempts,
        active_alerts,
        locked_addresses,
        snapshot_at: env.ledger().timestamp(),
    })
}

fn emit_security_alert(
    env: &Env,
    event_type: SecurityEventType,
    source: &Address,
    function_name: &soroban_sdk::String,
    count: u32,
) {
    let alert_id: u32 = env
        .storage()
        .instance()
        .get(&SecurityDataKey::ActiveAlertCount)
        .unwrap_or(0);

    let severity = match event_type {
        SecurityEventType::BruteForceAttempt => 4u32,
        SecurityEventType::PrivilegeEscalation => 4u32,
        SecurityEventType::UnauthorizedAccess => 3u32,
        SecurityEventType::DataAccessViolation => 3u32,
        SecurityEventType::AuthFailure => 2u32,
        SecurityEventType::RateLimitExceeded => 2u32,
        SecurityEventType::SuspiciousPattern => 2u32,
    };

    let alert = SecurityAlert {
        event_type,
        source_address: source.clone(),
        target_function: function_name.clone(),
        severity,
        timestamp: env.ledger().timestamp(),
        occurrence_count: count,
    };

    env.storage()
        .instance()
        .set(&SecurityDataKey::SecurityAlert(alert_id), &alert);
    env.storage()
        .instance()
        .set(&SecurityDataKey::ActiveAlertCount, &(alert_id + 1));

    env.events().publish(
        (
            symbol_short!("SEC"),
            symbol_short!("ALERT"),
        ),
        (event_type_num(event_type), source.clone(), count),
    );
}

fn emit_security_event(
    env: &Env,
    event_type: SecurityEventType,
    source: &Address,
    function_name: &soroban_sdk::String,
) {
    env.events().publish(
        (
            symbol_short!("SEC"),
            symbol_short!("EVT"),
        ),
        (
            event_type_num(event_type),
            source.clone(),
            function_name.clone(),
            env.ledger().timestamp(),
        ),
    );
}

fn event_type_num(event_type: SecurityEventType) -> u32 {
    event_type as u32
}

pub fn get_security_alert(
    env: &Env,
    alert_id: u32,
) -> Option<SecurityAlert> {
    env.storage()
        .instance()
        .get(&SecurityDataKey::SecurityAlert(alert_id))
}

pub fn get_address_failure_count(
    env: &Env,
    address: &Address,
    event_type: SecurityEventType,
) -> u64 {
    let key = match event_type {
        SecurityEventType::AuthFailure => SecurityDataKey::AuthFailureCount(address.clone()),
        SecurityEventType::UnauthorizedAccess | SecurityEventType::DataAccessViolation => {
            SecurityDataKey::UnauthorizedAccessCount(address.clone())
        }
        SecurityEventType::RateLimitExceeded => SecurityDataKey::RateLimitCount(address.clone()),
        SecurityEventType::SuspiciousPattern => {
            SecurityDataKey::SuspiciousPatternCount(address.clone())
        }
        SecurityEventType::BruteForceAttempt => SecurityDataKey::BruteForceCount(address.clone()),
        SecurityEventType::PrivilegeEscalation => {
            SecurityDataKey::UnauthorizedAccessCount(address.clone())
        }
    };
    env.storage().instance().get(&key).unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use soroban_sdk::testutils::Address as _;

    fn default_security_config() -> SecurityAlertConfig {
        SecurityAlertConfig {
            max_auth_failures_per_window: 5,
            max_rate_limit_violations: 10,
            window_duration_seconds: 3600,
            auto_lock_threshold: 10,
        }
    }

    #[test]
    fn test_record_auth_failure() {
        let env = Env::default();
        let contract_id = env.register_contract(None, super::super::ContractMonitoring);
        let client = super::super::ContractMonitoringClient::new(&env, &contract_id);

        let admin = Address::generate(&env);
        let alert_config = super::super::AlertConfig {
            max_error_rate_pct: 10,
            max_gas_per_window: 1_000_000,
            max_storage_entries: 10_000,
        };
        env.mock_all_auths();
        client.initialize(&admin, &alert_config);

        let security_config = default_security_config();
        initialize_security_config(&env, &security_config);

        let source = Address::generate(&env);
        let fn_name = soroban_sdk::String::from_str(&env, "write_record");

        for _ in 0..3 {
            let _ = record_security_event(
                &env,
                SecurityEventType::AuthFailure,
                &source,
                &fn_name,
            );
        }

        let count = get_address_failure_count(
            &env,
            &source,
            SecurityEventType::AuthFailure,
        );
        assert_eq!(count, 3);
    }

    #[test]
    fn test_lock_address_on_brute_force() {
        let env = Env::default();
        let contract_id = env.register_contract(None, super::super::ContractMonitoring);
        let client = super::super::ContractMonitoringClient::new(&env, &contract_id);

        let admin = Address::generate(&env);
        let alert_config = super::super::AlertConfig {
            max_error_rate_pct: 10,
            max_gas_per_window: 1_000_000,
            max_storage_entries: 10_000,
        };
        env.mock_all_auths();
        client.initialize(&admin, &alert_config);

        let security_config = SecurityAlertConfig {
            max_auth_failures_per_window: 5,
            max_rate_limit_violations: 10,
            window_duration_seconds: 3600,
            auto_lock_threshold: 3,
        };
        initialize_security_config(&env, &security_config);

        let source = Address::generate(&env);
        let fn_name = soroban_sdk::String::from_str(&env, "transfer_tokens");

        for _ in 0..3 {
            let _ = record_security_event(
                &env,
                SecurityEventType::BruteForceAttempt,
                &source,
                &fn_name,
            );
        }

        assert!(is_address_locked(&env, &source));
    }

    #[test]
    fn test_unlock_address() {
        let env = Env::default();
        let contract_id = env.register_contract(None, super::super::ContractMonitoring);
        let client = super::super::ContractMonitoringClient::new(&env, &contract_id);

        let admin = Address::generate(&env);
        let alert_config = super::super::AlertConfig {
            max_error_rate_pct: 10,
            max_gas_per_window: 1_000_000,
            max_storage_entries: 10_000,
        };
        env.mock_all_auths();
        client.initialize(&admin, &alert_config);

        let security_config = default_security_config();
        initialize_security_config(&env, &security_config);

        let source = Address::generate(&env);
        lock_address(&env, &source);
        assert!(is_address_locked(&env, &source));

        let _ = unlock_address(&env, &admin, &source);
        assert!(!is_address_locked(&env, &source));
    }

    #[test]
    fn test_security_snapshot() {
        let env = Env::default();
        let contract_id = env.register_contract(None, super::super::ContractMonitoring);
        let client = super::super::ContractMonitoringClient::new(&env, &contract_id);

        let admin = Address::generate(&env);
        let alert_config = super::super::AlertConfig {
            max_error_rate_pct: 10,
            max_gas_per_window: 1_000_000,
            max_storage_entries: 10_000,
        };
        env.mock_all_auths();
        client.initialize(&admin, &alert_config);

        let security_config = default_security_config();
        initialize_security_config(&env, &security_config);

        let source = Address::generate(&env);
        let fn_name = soroban_sdk::String::from_str(&env, "read_record");

        let _ = record_security_event(
            &env,
            SecurityEventType::AuthFailure,
            &source,
            &fn_name,
        );

        let snapshot = get_security_snapshot(&env).unwrap();
        assert_eq!(snapshot.total_auth_failures, 1);
    }
}
