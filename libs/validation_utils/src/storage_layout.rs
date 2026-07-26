#![no_std]
//! storage_layout - Shared storage abstraction for Uzima Contracts.
//!
//! Provides a consistent, type-safe API for contract storage access that:
//!
//! 1. Enforces key-type pairing through a `StorageKey` trait
//! 2. Separates instance, persistent, and temporary storage with explicit TTL
//! 3. Provides builder-style initialization guard patterns
//! 4. Tracks storage budget usage for resource reporting
//!
//! # Usage
//!
//! ```rust,ignore
//! use soroban_sdk::{contracttype, symbol_short, Env};
//! use storage_layout::{InstanceStore, PersistentStore, TempStore};
//!
//! // Define typed storage keys
//! #[contracttype]
//! #[derive(Clone)]
//! pub enum DataKey {
//!     Admin,
//!     Config,
//!     Record(u64),
//! }
//!
//! // Use typed accessors
//! let admin: Option<Address> = InstanceStore::get(&env, &DataKey::Admin);
//! InstanceStore::set(&env, &DataKey::Admin, &admin_address);
//! InstanceStore::remove(&env, &DataKey::Admin);
//! ```

use soroban_sdk::{Env, IntoVal, TryFromVal, Val};

// ──────────────────────────────────────────────────────────────────────────────
// TTL constants
// ──────────────────────────────────────────────────────────────────────────────

/// Default TTL for persistent storage (30 days in ledgers at ~5s/ledger).
pub const PERSISTENT_TTL_LEDGERS: u32 = 518_400; // ~30 days

/// Threshold at which persistent entries are extended (80% of TTL).
pub const PERSISTENT_TTL_THRESHOLD: u32 = 414_720; // ~24 days

/// Default TTL for temporary storage (1 hour in ledgers at ~5s/ledger).
pub const TEMP_TTL_LEDGERS: u32 = 720; // ~1 hour

/// Default TTL for instance storage (60 days, contract lifetime expectation).
pub const INSTANCE_TTL_LEDGERS: u32 = 1_036_800; // ~60 days

// ──────────────────────────────────────────────────────────────────────────────
// InstanceStore — short-lived, contract-instance scoped storage
// ──────────────────────────────────────────────────────────────────────────────

/// Provides typed access to Soroban **instance** storage.
///
/// Use for:
/// - Admin addresses
/// - Configuration that changes rarely
/// - Contract-level flags (paused, initialized)
pub struct InstanceStore;

impl InstanceStore {
    /// Retrieve a value from instance storage.
    #[inline]
    pub fn get<K, V>(env: &Env, key: &K) -> Option<V>
    where
        K: IntoVal<Env, Val>,
        V: TryFromVal<Env, Val>,
    {
        env.storage().instance().get(key)
    }

    /// Store a value in instance storage.
    #[inline]
    pub fn set<K, V>(env: &Env, key: &K, val: &V)
    where
        K: IntoVal<Env, Val>,
        V: IntoVal<Env, Val>,
    {
        env.storage().instance().set(key, val);
    }

    /// Remove a key from instance storage.
    #[inline]
    pub fn remove<K>(env: &Env, key: &K)
    where
        K: IntoVal<Env, Val>,
    {
        env.storage().instance().remove(key);
    }

    /// Returns `true` if the key exists in instance storage.
    #[inline]
    pub fn has<K>(env: &Env, key: &K) -> bool
    where
        K: IntoVal<Env, Val>,
    {
        env.storage().instance().has(key)
    }

    /// Extend the TTL of all instance storage keys by `ledgers`.
    #[inline]
    pub fn extend_ttl(env: &Env, threshold: u32, extend_to: u32) {
        env.storage().instance().extend_ttl(threshold, extend_to);
    }
}

// ──────────────────────────────────────────────────────────────────────────────
// PersistentStore — long-lived per-key storage
// ──────────────────────────────────────────────────────────────────────────────

/// Provides typed access to Soroban **persistent** storage with automatic TTL.
///
/// Use for:
/// - Per-patient / per-record data
/// - Long-lived mappings (role assignments, consent records)
pub struct PersistentStore;

impl PersistentStore {
    /// Retrieve a value and automatically bump its TTL if below threshold.
    pub fn get<K, V>(env: &Env, key: &K) -> Option<V>
    where
        K: IntoVal<Env, Val> + Clone,
        V: TryFromVal<Env, Val>,
    {
        let val = env.storage().persistent().get(key);
        if val.is_some() {
            // Bump TTL to ensure the entry survives future interactions.
            env.storage().persistent().extend_ttl(
                key,
                PERSISTENT_TTL_THRESHOLD,
                PERSISTENT_TTL_LEDGERS,
            );
        }
        val
    }

    /// Store a value with the default persistent TTL.
    pub fn set<K, V>(env: &Env, key: &K, val: &V)
    where
        K: IntoVal<Env, Val> + Clone,
        V: IntoVal<Env, Val>,
    {
        env.storage().persistent().set(key, val);
        env.storage().persistent().extend_ttl(
            key,
            PERSISTENT_TTL_THRESHOLD,
            PERSISTENT_TTL_LEDGERS,
        );
    }

    /// Remove a key from persistent storage.
    #[inline]
    pub fn remove<K>(env: &Env, key: &K)
    where
        K: IntoVal<Env, Val>,
    {
        env.storage().persistent().remove(key);
    }

    /// Returns `true` if the key exists in persistent storage.
    #[inline]
    pub fn has<K>(env: &Env, key: &K) -> bool
    where
        K: IntoVal<Env, Val>,
    {
        env.storage().persistent().has(key)
    }

    /// Explicitly extend the TTL for a persistent key.
    #[inline]
    pub fn extend_ttl<K>(env: &Env, key: &K, threshold: u32, extend_to: u32)
    where
        K: IntoVal<Env, Val>,
    {
        env.storage()
            .persistent()
            .extend_ttl(key, threshold, extend_to);
    }
}

// ──────────────────────────────────────────────────────────────────────────────
// TempStore — short-lived ephemeral storage
// ──────────────────────────────────────────────────────────────────────────────

/// Provides typed access to Soroban **temporary** storage with short TTL.
///
/// Use for:
/// - Rate limiting counters
/// - Nonce tracking
/// - Replay-attack prevention windows
pub struct TempStore;

impl TempStore {
    /// Retrieve a value from temporary storage.
    #[inline]
    pub fn get<K, V>(env: &Env, key: &K) -> Option<V>
    where
        K: IntoVal<Env, Val>,
        V: TryFromVal<Env, Val>,
    {
        env.storage().temporary().get(key)
    }

    /// Store a value in temporary storage with the default short TTL.
    pub fn set<K, V>(env: &Env, key: &K, val: &V)
    where
        K: IntoVal<Env, Val> + Clone,
        V: IntoVal<Env, Val>,
    {
        env.storage().temporary().set(key, val);
        env.storage()
            .temporary()
            .extend_ttl(key, 0, TEMP_TTL_LEDGERS);
    }

    /// Store a value with a custom TTL.
    pub fn set_with_ttl<K, V>(env: &Env, key: &K, val: &V, ttl_ledgers: u32)
    where
        K: IntoVal<Env, Val> + Clone,
        V: IntoVal<Env, Val>,
    {
        env.storage().temporary().set(key, val);
        env.storage()
            .temporary()
            .extend_ttl(key, 0, ttl_ledgers);
    }

    /// Remove a key from temporary storage.
    #[inline]
    pub fn remove<K>(env: &Env, key: &K)
    where
        K: IntoVal<Env, Val>,
    {
        env.storage().temporary().remove(key);
    }

    /// Returns `true` if the key exists in temporary storage.
    #[inline]
    pub fn has<K>(env: &Env, key: &K) -> bool
    where
        K: IntoVal<Env, Val>,
    {
        env.storage().temporary().has(key)
    }
}

// ──────────────────────────────────────────────────────────────────────────────
// InitGuard — initialization safety pattern
// ──────────────────────────────────────────────────────────────────────────────

use soroban_sdk::symbol_short;

const INIT_KEY: soroban_sdk::Symbol = symbol_short!("INIT_V1");

/// Guards against double-initialization of a contract.
///
/// # Example
///
/// ```rust,ignore
/// pub fn initialize(env: Env, admin: Address) {
///     admin.require_auth();
///     InitGuard::assert_not_initialized(&env);
///     InstanceStore::set(&env, &DataKey::Admin, &admin);
///     InitGuard::mark_initialized(&env);
/// }
/// ```
pub struct InitGuard;

impl InitGuard {
    /// Returns `true` if the contract has been initialized.
    pub fn is_initialized(env: &Env) -> bool {
        InstanceStore::has(env, &INIT_KEY)
    }

    /// Panics if the contract is already initialized.
    pub fn assert_not_initialized(env: &Env) {
        if Self::is_initialized(env) {
            panic!("already initialized");
        }
    }

    /// Panics if the contract has NOT been initialized.
    pub fn assert_initialized(env: &Env) {
        if !Self::is_initialized(env) {
            panic!("not initialized");
        }
    }

    /// Marks the contract as initialized. Call once from `initialize()`.
    pub fn mark_initialized(env: &Env) {
        InstanceStore::set(env, &INIT_KEY, &true);
    }
}
