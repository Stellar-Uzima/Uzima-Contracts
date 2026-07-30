#![no_std]

use soroban_sdk::{contracterror, contracttype, Address, BytesN, Env, String};

/// Errors returned by feature flag operations.
#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum FeatureFlagError {
    /// The requested feature flag does not exist for this contract.
    FlagNotFound = 1,
    /// The rollout percentage is invalid (must be 0–100).
    InvalidRolloutPercentage = 2,
    /// An environment override could not be resolved.
    EnvironmentOverrideError = 3,
}

/// The rollout stage of a feature flag.
#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FlagStage {
    Disabled,
    Canary,
    Beta,
    Ga,
}

/// A single feature flag definition.
#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FeatureFlag {
    pub name: String,
    pub enabled: bool,
    pub rollout_percentage: u32,
    pub stage: FlagStage,
}

/// Storage keys used by the feature-flag library.
/// Stored in the calling contract's persistent storage namespace.
#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DataKey {
    /// Stores a `FeatureFlag` by name.
    Flag(String),
    /// Stores a `u32` override percentage for a specific caller address.
    CallerOverride(Address),
    /// Stores the environment name (e.g. "testnet", "mainnet").
    Environment,
}

/// Look up a feature flag by name from the calling contract's storage.
///
/// Returns `Ok(FeatureFlag)` if found, or `Err(FlagNotFound)` otherwise.
pub fn get_flag(env: &Env, flag_name: &str) -> Result<FeatureFlag, FeatureFlagError> {
    let key = DataKey::Flag(String::from_str(env, flag_name));
    env.storage()
        .persistent()
        .get::<DataKey, FeatureFlag>(&key)
        .ok_or(FeatureFlagError::FlagNotFound)
}

/// Store a feature flag in the calling contract's persistent storage.
pub fn set_flag(env: &Env, flag: &FeatureFlag) {
    let key = DataKey::Flag(flag.name.clone());
    env.storage().persistent().set(&key, flag);
}

/// Simple boolean check: is this flag enabled at all?
///
/// Returns `true` only if the flag exists and its `enabled` field is `true`.
/// Does **not** consider rollout percentage — use [`evaluate_rollout`] for that.
pub fn is_enabled(env: &Env, flag_name: &str) -> bool {
    match get_flag(env, flag_name) {
        Ok(flag) => flag.enabled,
        Err(_) => false,
    }
}

/// Evaluate whether a specific caller should see this flag enabled,
/// taking rollout percentage and environment overrides into account.
///
/// # Algorithm
///
/// 1. Check environment overrides — if `all_flags_enabled` is set for the
///    current environment, return `true`.
/// 2. Look up the flag — if not found or `enabled == false`, return `false`.
/// 3. If `rollout_percentage == 100`, return `true`.
/// 4. If `rollout_percentage == 0`, return `false`.
/// 5. For 1–99%, compute a deterministic hash of `(flag_name + caller_address)`
///    modulo 100 and compare against the percentage threshold.
///
/// This ensures the same caller always gets the same result for a given flag,
/// and callers are distributed uniformly across the rollout range.
pub fn evaluate_rollout(
    env: &Env,
    flag_name: &str,
    caller: &Address,
) -> Result<bool, FeatureFlagError> {
    // 1. Check environment overrides
    if is_all_flags_enabled_for_env(env) {
        return Ok(true);
    }

    // 2. Look up flag
    let flag = get_flag(env, flag_name)?;

    if !flag.enabled {
        return Ok(false);
    }

    // 3. Full rollout
    if flag.rollout_percentage >= 100 {
        return Ok(true);
    }

    // 4. No rollout
    if flag.rollout_percentage == 0 {
        return Ok(false);
    }

    // 5. Deterministic percentage-based evaluation
    let hash = deterministic_hash(env, flag_name, caller);
    let bucket = hash % 100;
    Ok(bucket < flag.rollout_percentage)
}

/// Set an environment override that forces all flags enabled (or disabled).
///
/// This is intended for testnet (force all on) or emergency shut-off (force all off).
pub fn set_environment_override(env: &Env, all_enabled: bool) {
    let key = DataKey::Environment;
    env.storage()
        .persistent()
        .set(&key, &if all_enabled { 1u32 } else { 0u32 });
}

/// Get the current environment name (e.g. "testnet", "mainnet").
pub fn get_environment_name(env: &Env) -> Option<String> {
    // Check if we have a stored environment override for all-flags-enabled
    // The environment name itself is informational
    let key = DataKey::Environment;
    env.storage()
        .persistent()
        .get::<DataKey, String>(&key)
}

/// Set the environment name label for this deployment.
pub fn set_environment_name(env: &Env, name: &str) {
    let key = DataKey::Environment;
    env.storage()
        .persistent()
        .set(&key, &String::from_str(env, name));
}

/// Convenience: read the `Environment` key and check if all-flags-enabled
/// override is active.
fn is_all_flags_enabled_for_env(env: &Env) -> bool {
    let key = DataKey::Environment;
    env.storage()
        .persistent()
        .get::<DataKey, u32>(&key)
        .map(|v| v == 1)
        .unwrap_or(false)
}

/// Deterministic hash of `(flag_name || caller_address)` modulo 2^32.
///
/// Uses a simple FNV-1a-like mixing on the raw bytes. This is not
/// cryptographic — it only needs to be deterministic and uniformly distributed.
fn deterministic_hash(env: &Env, flag_name: &str, caller: &Address) -> u32 {
    let mut hasher: u32 = 0x811c9dc5; // FNV offset basis

    for byte in flag_name.as_bytes().iter() {
        hasher = hasher ^ (*byte as u32);
        hasher = hasher.wrapping_mul(0x01000193); // FNV prime
    }

    let caller_bytes = caller.to_bytes();
    for i in 0..caller_bytes.len() {
        let byte = caller_bytes.get(i as u32);
        hasher = hasher ^ (byte as u32);
        hasher = hasher.wrapping_mul(0x01000193);
    }

    hasher
}

#[cfg(test)]
mod tests;
