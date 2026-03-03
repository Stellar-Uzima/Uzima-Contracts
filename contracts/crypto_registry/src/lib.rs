#![no_std]

#[cfg(test)]
mod test;

use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype, symbol_short, Address, Bytes, BytesN, Env,
    Symbol,
};

// =============================================================================
// Types
// =============================================================================

/// Cryptographic algorithm identifier.
///
/// Notes:
/// - This contract stores public keys and metadata only. Cryptographic operations
///   are performed off-chain (E2E encryption, PQC, HE, MPC).
#[derive(Clone, Copy, PartialEq, Eq)]
#[contracttype]
pub enum KeyAlgorithm {
    // Classical
    X25519,
    Ed25519,
    Secp256k1,

    // Post-quantum preparations (stored for hybrid / future upgrade)
    Kyber768,
    Dilithium3,
    Falcon512,

    // For forward-compatibility
    Custom(u32),
}

#[derive(Clone)]
#[contracttype]
pub struct PublicKey {
    pub algorithm: KeyAlgorithm,
    /// Raw public key bytes (format depends on `algorithm`).
    pub key: Bytes,
}

#[derive(Clone)]
#[contracttype]
pub struct KeyBundle {
    /// Monotonic version per account.
    pub version: u32,
    /// Timestamp at registration/rotation.
    pub created_at: u64,
    /// Whether this key bundle has been revoked.
    pub revoked: bool,

    /// Primary encryption key (recommended: X25519).
    pub encryption_key: PublicKey,
    /// Optional post-quantum encryption public key (e.g., Kyber).
    pub has_pq_encryption_key: bool,
    pub pq_encryption_key: PublicKey,
    /// Optional signing public key (recommended: Ed25519).
    pub has_signing_key: bool,
    pub signing_key: PublicKey,

    /// Deterministic fingerprint of the bundle (sha256 over normalized encoding).
    pub bundle_id: BytesN<32>,
}

// =============================================================================
// Storage
// =============================================================================

#[contracttype]
pub enum DataKey {
    Initialized,
    Admin,
    CurrentVersion(Address),
    Bundle(Address, u32),
}

const INIT: Symbol = symbol_short!("INIT");

// =============================================================================
// Errors
// =============================================================================

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    AlreadyInitialized = 1,
    NotInitialized = 2,
    NotAuthorized = 3,
    InvalidKey = 4,
    KeyNotFound = 5,
    KeyAlreadyRevoked = 6,
    InvalidKeyLength = 7,
}

// =============================================================================
// Contract
// =============================================================================

#[contract]
pub struct CryptoRegistry;

#[contractimpl]
impl CryptoRegistry {
    /// Initialize the registry with an admin address for policy upgrades.
    /// Key registration/rotation is always self-authorized by the account.
    pub fn initialize(env: Env, admin: Address) -> Result<(), Error> {
        admin.require_auth();
        if env.storage().instance().has(&DataKey::Initialized) {
            return Err(Error::AlreadyInitialized);
        }
        env.storage().instance().set(&DataKey::Initialized, &true);
        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage().persistent().set(&INIT, &true);

        env.events()
            .publish((symbol_short!("crypto"), symbol_short!("init")), admin);
        Ok(())
    }

    /// Register (or rotate) the caller's key bundle.
    ///
    /// Returns the newly assigned version.
    pub fn register_key_bundle(
        env: Env,
        owner: Address,
        encryption_key: PublicKey,
        pq_encryption_key: PublicKey,
        has_pq_encryption_key: bool,
        signing_key: PublicKey,
        has_signing_key: bool,
    ) -> Result<u32, Error> {
        owner.require_auth();
        Self::require_initialized(&env)?;

        Self::validate_public_key(&encryption_key)?;
        let pq_key = if has_pq_encryption_key {
            Self::validate_public_key(&pq_encryption_key)?;
            pq_encryption_key
        } else {
            PublicKey {
                algorithm: KeyAlgorithm::Custom(0),
                key: Bytes::new(&env),
            }
        };
        let sig_key = if has_signing_key {
            Self::validate_public_key(&signing_key)?;
            signing_key
        } else {
            PublicKey {
                algorithm: KeyAlgorithm::Custom(0),
                key: Bytes::new(&env),
            }
        };

        let next_version = Self::next_version(&env, &owner);
        let bundle_id = Self::compute_bundle_id(
            &env,
            next_version,
            &encryption_key,
            has_pq_encryption_key,
            &pq_key,
            has_signing_key,
            &sig_key,
        );

        let bundle = KeyBundle {
            version: next_version,
            created_at: env.ledger().timestamp(),
            revoked: false,
            encryption_key,
            has_pq_encryption_key,
            pq_encryption_key: pq_key,
            has_signing_key,
            signing_key: sig_key,
            bundle_id,
        };

        env.storage()
            .persistent()
            .set(&DataKey::Bundle(owner.clone(), next_version), &bundle);
        env.storage()
            .persistent()
            .set(&DataKey::CurrentVersion(owner.clone()), &next_version);

        env.events().publish(
            (symbol_short!("crypto"), symbol_short!("bundle")),
            (owner, next_version),
        );

        Ok(next_version)
    }

    /// Revoke a specific key bundle version.
    pub fn revoke_key_bundle(env: Env, owner: Address, version: u32) -> Result<(), Error> {
        owner.require_auth();
        Self::require_initialized(&env)?;

        let key = DataKey::Bundle(owner.clone(), version);
        let mut bundle: KeyBundle = env
            .storage()
            .persistent()
            .get(&key)
            .ok_or(Error::KeyNotFound)?;
        if bundle.revoked {
            return Err(Error::KeyAlreadyRevoked);
        }
        bundle.revoked = true;
        env.storage().persistent().set(&key, &bundle);

        env.events().publish(
            (symbol_short!("crypto"), symbol_short!("revoke")),
            (owner, version),
        );

        Ok(())
    }

    pub fn get_current_version(env: Env, owner: Address) -> Result<u32, Error> {
        Self::require_initialized(&env)?;
        Ok(env
            .storage()
            .persistent()
            .get(&DataKey::CurrentVersion(owner))
            .unwrap_or(0))
    }

    pub fn get_current_key_bundle(env: Env, owner: Address) -> Result<Option<KeyBundle>, Error> {
        Self::require_initialized(&env)?;
        let v: u32 = env
            .storage()
            .persistent()
            .get(&DataKey::CurrentVersion(owner.clone()))
            .unwrap_or(0);
        if v == 0 {
            return Ok(None);
        }
        Ok(env.storage().persistent().get(&DataKey::Bundle(owner, v)))
    }

    pub fn get_key_bundle(
        env: Env,
        owner: Address,
        version: u32,
    ) -> Result<Option<KeyBundle>, Error> {
        Self::require_initialized(&env)?;
        Ok(env
            .storage()
            .persistent()
            .get(&DataKey::Bundle(owner, version)))
    }

    // -------------------------------------------------------------------------
    // Internal helpers
    // -------------------------------------------------------------------------

    fn require_initialized(env: &Env) -> Result<(), Error> {
        if env.storage().instance().has(&DataKey::Initialized) {
            Ok(())
        } else {
            Err(Error::NotInitialized)
        }
    }

    fn next_version(env: &Env, owner: &Address) -> u32 {
        let current: u32 = env
            .storage()
            .persistent()
            .get(&DataKey::CurrentVersion(owner.clone()))
            .unwrap_or(0);
        current.saturating_add(1)
    }

    fn validate_public_key(key: &PublicKey) -> Result<(), Error> {
        // Enforce minimal and maximal length to prevent pathological storage use.
        let len = key.key.len();
        if len == 0 {
            return Err(Error::InvalidKey);
        }
        if len > 2048 {
            return Err(Error::InvalidKeyLength);
        }

        // Lightweight checks for common algorithms.
        match key.algorithm {
            KeyAlgorithm::X25519 | KeyAlgorithm::Ed25519 => {
                if len != 32 {
                    return Err(Error::InvalidKeyLength);
                }
            }
            KeyAlgorithm::Secp256k1 => {
                // Compressed: 33, uncompressed: 65. Allow either.
                if len != 33 && len != 65 {
                    return Err(Error::InvalidKeyLength);
                }
            }
            _ => {}
        }

        Ok(())
    }

    fn compute_bundle_id(
        env: &Env,
        version: u32,
        encryption_key: &PublicKey,
        has_pq_encryption_key: bool,
        pq_encryption_key: &PublicKey,
        has_signing_key: bool,
        signing_key: &PublicKey,
    ) -> BytesN<32> {
        let mut payload = Bytes::new(env);
        payload.append(&Bytes::from_slice(env, &version.to_be_bytes()));
        payload.append(&Self::encode_algorithm(env, &encryption_key.algorithm));
        payload.append(&encryption_key.key);
        payload.append(&Bytes::from_slice(env, &[has_pq_encryption_key as u8]));
        payload.append(&Self::encode_algorithm(env, &pq_encryption_key.algorithm));
        payload.append(&pq_encryption_key.key);
        payload.append(&Bytes::from_slice(env, &[has_signing_key as u8]));
        payload.append(&Self::encode_algorithm(env, &signing_key.algorithm));
        payload.append(&signing_key.key);
        env.crypto().sha256(&payload).into()
    }

    fn encode_algorithm(env: &Env, algo: &KeyAlgorithm) -> Bytes {
        // Stable encoding: 4-byte tag + optional 4-byte custom value.
        let (tag, custom) = match algo {
            KeyAlgorithm::X25519 => (1u32, None),
            KeyAlgorithm::Ed25519 => (2u32, None),
            KeyAlgorithm::Secp256k1 => (3u32, None),
            KeyAlgorithm::Kyber768 => (101u32, None),
            KeyAlgorithm::Dilithium3 => (102u32, None),
            KeyAlgorithm::Falcon512 => (103u32, None),
            KeyAlgorithm::Custom(v) => (10_000u32, Some(*v)),
        };

        let mut out = Bytes::new(env);
        out.append(&Bytes::from_slice(env, &tag.to_be_bytes()));
        if let Some(v) = custom {
            out.append(&Bytes::from_slice(env, &v.to_be_bytes()));
        }
        out
    }
}
