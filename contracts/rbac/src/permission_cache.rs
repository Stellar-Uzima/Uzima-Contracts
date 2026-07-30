#![no_std]
//! permission_cache - Temporary-storage cache for frequent RBAC lookups.
//!
//! Caches role assignments in temporary storage (TTL: 1 ledger) to avoid
//! repeated persistent storage reads within a single transaction context.

use soroban_sdk::{contracttype, Address, Env, Symbol};

/// Cache key for a (caller, role) permission lookup.
#[derive(Clone)]
#[contracttype]
pub struct PermCacheKey {
    pub caller: Address,
    pub role: Symbol,
}

/// TTL for cached permission results (1 ledger — within-transaction cache).
pub const PERM_CACHE_TTL: u32 = 1;

/// Cached RBAC permission lookup.
pub struct PermissionCache;

impl PermissionCache {
    /// Look up a cached permission. Returns `None` if not cached.
    pub fn get(env: &Env, caller: &Address, role: &Symbol) -> Option<bool> {
        let key = PermCacheKey { caller: caller.clone(), role: role.clone() };
        env.storage().temporary().get(&key)
    }

    /// Store a permission result in the cache.
    pub fn set(env: &Env, caller: &Address, role: &Symbol, has_role: bool) {
        let key = PermCacheKey { caller: caller.clone(), role: role.clone() };
        env.storage().temporary().set(&key, &has_role);
        env.storage().temporary().extend_ttl(&key, 0, PERM_CACHE_TTL);
    }

    /// Invalidate the cache for a caller (e.g. after role change).
    pub fn invalidate(env: &Env, caller: &Address, role: &Symbol) {
        let key = PermCacheKey { caller: caller.clone(), role: role.clone() };
        env.storage().temporary().remove(&key);
    }

    /// Check permission with cache-through: reads cache first, falls back to
    /// the provided `lookup_fn`, and populates the cache on miss.
    pub fn check_with_cache<F>(
        env: &Env,
        caller: &Address,
        role: &Symbol,
        lookup_fn: F,
    ) -> bool
    where
        F: Fn() -> bool,
    {
        if let Some(cached) = Self::get(env, caller, role) {
            return cached;
        }
        let result = lookup_fn();
        Self::set(env, caller, role, result);
        result
    }
}
