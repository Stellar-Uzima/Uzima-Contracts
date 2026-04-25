//! API Versioning Strategy (Issue #418)
//!
//! Provides semantic versioning for Soroban contracts, API version tracking,
//! backward compatibility guarantees, and deprecation timeline management.

use soroban_sdk::{contracttype, symbol_short, Env, Symbol, Vec};

// ---------------------------------------------------------------------------
// Version constants
// ---------------------------------------------------------------------------

/// Current API version. Increment on every breaking change.
pub const API_VERSION: u32 = 2;

/// Minimum supported API version for backward compatibility.
pub const MIN_SUPPORTED_API_VERSION: u32 = 1;

/// API version at which features were deprecated (for sunset tracking).
pub const DEPRECATED_SINCE_VERSION: u32 = 1;

// ---------------------------------------------------------------------------
// Storage keys
// ---------------------------------------------------------------------------

const API_VER_KEY: Symbol = symbol_short!("API_VER");
const COMPAT_KEY: Symbol = symbol_short!("COMPAT");

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

/// Semantic version representation (major.minor.patch).
#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub struct SemanticVersion {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
}

/// Compatibility status between two API versions.
#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub enum CompatibilityStatus {
    /// Fully compatible – no changes required.
    Compatible,
    /// Deprecated but still functional – migration recommended.
    Deprecated,
    /// Incompatible – client must upgrade.
    Incompatible,
}

/// Result of a version negotiation between client and contract.
#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub struct VersionNegotiation {
    pub requested_version: u32,
    pub supported_version: u32,
    pub status: CompatibilityStatus,
    pub deprecation_notice: bool,
    pub sunset_version: u32,
}

// ---------------------------------------------------------------------------
// Core API versioning functions
// ---------------------------------------------------------------------------

/// Returns the current API version of the contract.
///
/// Clients should call this before interacting with the contract to ensure
/// compatibility.
pub fn get_api_version(_env: &Env) -> u32 {
    API_VERSION
}

/// Stores the API version in contract instance storage for on-chain queries.
pub fn initialize_api_version(env: &Env) {
    env.storage().instance().set(&API_VER_KEY, &API_VERSION);
}

/// Retrieves the stored API version from instance storage.
pub fn get_stored_api_version(env: &Env) -> u32 {
    env.storage()
        .instance()
        .get(&API_VER_KEY)
        .unwrap_or(API_VERSION)
}

/// Negotiates compatibility between a client's requested version and the
/// contract's current API version.
///
/// Returns a `VersionNegotiation` describing whether the client can proceed,
/// should migrate, or must upgrade.
pub fn negotiate_version(env: &Env, client_version: u32) -> VersionNegotiation {
    let current = get_api_version(env);

    let (status, deprecation_notice) = if client_version > current {
        // Client is ahead – incompatible (contract needs upgrade)
        (CompatibilityStatus::Incompatible, false)
    } else if client_version < MIN_SUPPORTED_API_VERSION {
        // Client is too old – incompatible
        (CompatibilityStatus::Incompatible, false)
    } else if client_version < current && client_version >= DEPRECATED_SINCE_VERSION {
        // Client is on a deprecated but still-supported version
        (CompatibilityStatus::Deprecated, true)
    } else {
        // Exact match or within supported range
        (CompatibilityStatus::Compatible, false)
    };

    VersionNegotiation {
        requested_version: client_version,
        supported_version: current,
        status,
        deprecation_notice,
        // Sunset is 2 major versions ahead of the deprecated version
        sunset_version: DEPRECATED_SINCE_VERSION + 2,
    }
}

/// Checks whether a given API version is still supported.
pub fn is_version_supported(_env: &Env, version: u32) -> bool {
    version >= MIN_SUPPORTED_API_VERSION && version <= API_VERSION
}

/// Returns a list of all supported API versions.
pub fn get_supported_versions(env: &Env) -> Vec<u32> {
    let mut versions = Vec::new(env);
    let mut v = MIN_SUPPORTED_API_VERSION;
    while v <= API_VERSION {
        versions.push_back(v);
        v += 1;
    }
    versions
}

/// Records a compatibility matrix entry in instance storage.
/// Used to track which client versions have been tested against this contract.
pub fn record_compatibility(env: &Env, client_version: u32, compatible: bool) {
    let key = (COMPAT_KEY, client_version);
    env.storage().instance().set(&key, &compatible);
}

/// Retrieves the recorded compatibility for a given client version.
pub fn get_compatibility(env: &Env, client_version: u32) -> Option<bool> {
    let key = (COMPAT_KEY, client_version);
    env.storage().instance().get(&key)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use soroban_sdk::Env;

    #[test]
    fn test_get_api_version_returns_current() {
        let env = Env::default();
        assert_eq!(get_api_version(&env), API_VERSION);
    }

    #[test]
    fn test_initialize_and_retrieve_stored_version() {
        let env = Env::default();
        initialize_api_version(&env);
        assert_eq!(get_stored_api_version(&env), API_VERSION);
    }

    #[test]
    fn test_negotiate_compatible_version() {
        let env = Env::default();
        let result = negotiate_version(&env, API_VERSION);
        assert_eq!(result.status, CompatibilityStatus::Compatible);
        assert!(!result.deprecation_notice);
    }

    #[test]
    fn test_negotiate_deprecated_version() {
        let env = Env::default();
        // Version 1 is deprecated (below current API_VERSION=2) but >= MIN_SUPPORTED
        let result = negotiate_version(&env, 1);
        assert_eq!(result.status, CompatibilityStatus::Deprecated);
        assert!(result.deprecation_notice);
        assert_eq!(result.sunset_version, DEPRECATED_SINCE_VERSION + 2);
    }

    #[test]
    fn test_negotiate_incompatible_old_version() {
        let env = Env::default();
        // Version 0 is below MIN_SUPPORTED_API_VERSION
        let result = negotiate_version(&env, 0);
        assert_eq!(result.status, CompatibilityStatus::Incompatible);
    }

    #[test]
    fn test_negotiate_incompatible_future_version() {
        let env = Env::default();
        // Client claims version 99 – contract can't support it
        let result = negotiate_version(&env, 99);
        assert_eq!(result.status, CompatibilityStatus::Incompatible);
    }

    #[test]
    fn test_is_version_supported() {
        let env = Env::default();
        assert!(is_version_supported(&env, API_VERSION));
        assert!(is_version_supported(&env, MIN_SUPPORTED_API_VERSION));
        assert!(!is_version_supported(&env, 0));
        assert!(!is_version_supported(&env, 999));
    }

    #[test]
    fn test_get_supported_versions() {
        let env = Env::default();
        let versions = get_supported_versions(&env);
        assert!(versions.len() >= 1);
        // Must include current version
        let mut found = false;
        for i in 0..versions.len() {
            if versions.get(i).unwrap() == API_VERSION {
                found = true;
                break;
            }
        }
        assert!(found, "Current API version must be in supported list");
    }

    #[test]
    fn test_record_and_get_compatibility() {
        let env = Env::default();
        record_compatibility(&env, 1, true);
        assert_eq!(get_compatibility(&env, 1), Some(true));
        record_compatibility(&env, 0, false);
        assert_eq!(get_compatibility(&env, 0), Some(false));
    }
}
