//! # FIDO2 / WebAuthn Authenticator Contract
//!
//! Implements FIDO2 Level 2 / WebAuthn device registry on Soroban (Stellar).
//! Supports biometric platform authenticators (fingerprint, face ID), hardware
//! security keys (YubiKey, etc.), and multi-device passkeys.
//!
//! ## Algorithm support
//! | Algorithm | COSE ID | On-chain verification |
//! |-----------|---------|----------------------|
//! | EdDSA (Ed25519) | -8 | Direct via `env.crypto().ed25519_verify()` |
//! | ES256 (ECDSA P-256) | -7 | ZK proof submitted by a trusted verifier |
//!
//! ## FIDO2 ceremony flow
//! 1. **Registration** — `issue_registration_challenge` → (client creates credential) →
//!    `register_device`
//! 2. **Authentication** — `issue_auth_challenge` → (client signs challenge) →
//!    `verify_ed25519_assertion` *or* `verify_zk_assertion`
//!
//! ## Identity Registry integration
//! When the identity registry address is configured, `register_device` binds
//! each new credential as a FIDO2 verification method in the user's DID document
//! via `add_fido2_device` on the identity registry.

#![no_std]
#![allow(clippy::too_many_arguments)]
#![allow(clippy::arithmetic_side_effects)]

use soroban_sdk::xdr::ToXdr;
use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype, Address, Bytes, BytesN, Env, String, Vec,
};

// ═══════════════════════════════════════════════════════════════════════════
// Errors
// ═══════════════════════════════════════════════════════════════════════════

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    AlreadyInitialized = 1,
    NotInitialized = 2,
    NotAuthorized = 3,
    DeviceNotFound = 4,
    DeviceAlreadyRegistered = 5,
    MaxDevicesReached = 6,
    DeviceInactive = 7,
    InvalidPublicKey = 8,
    /// Signature or ZK proof verification failed.
    InvalidSignature = 9,
    /// `authenticatorData` is malformed or too short.
    InvalidAuthenticatorData = 10,
    /// The pending challenge has expired (> 5 minutes old).
    ChallengeExpired = 11,
    /// Authentication attempted without first issuing a challenge.
    NoChallengeIssued = 12,
    /// Sign count did not increase — possible credential clone detected.
    SignCountRegression = 13,
    InvalidDeviceName = 14,
    InvalidCredentialIdHash = 15,
    /// `verify_zk_assertion` called but no ZK verifier contract is configured.
    ZkVerifierNotSet = 16,
    /// ZK proof nullifier has already been used (replay attack).
    NullifierAlreadyUsed = 17,
    /// `authenticatorData` rpIdHash does not match the contract's configured RP ID.
    RpIdMismatch = 18,
    /// FIDO2 User Presence (UP) flag is not set in `authenticatorData`.
    UserPresenceNotVerified = 19,
    InvalidRevocationReason = 20,
    /// `register_device` called with an algorithm mismatched to the public key size.
    AlgorithmKeyMismatch = 21,
}

// ═══════════════════════════════════════════════════════════════════════════
// Data types
// ═══════════════════════════════════════════════════════════════════════════

/// FIDO2 / COSE public-key algorithm identifier.
#[contracttype]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum PublicKeyAlgorithm {
    /// Ed25519 (COSE algorithm -8).  Verified on-chain.
    EdDSA,
    /// ECDSA P-256 (COSE algorithm -7).  Verified via ZK proof.
    ES256,
}

/// Physical or logical transport mechanism of the authenticator.
#[contracttype]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum AuthenticatorTransport {
    /// USB hardware security key (e.g., YubiKey 5 series).
    Usb,
    /// NFC-capable hardware security key.
    Nfc,
    /// Bluetooth Low-Energy hardware security key.
    Ble,
    /// Built-in platform authenticator — fingerprint sensor, Face ID, Windows Hello.
    Internal,
    /// Hybrid / passkey-synced credential (cross-device authentication via phone).
    Hybrid,
}

/// Whether the authenticator is a built-in device or an external roaming key.
#[contracttype]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum AuthenticatorAttachment {
    /// Built-in authenticator (Touch ID, Face ID, Windows Hello).
    Platform,
    /// External / roaming hardware security key.
    CrossPlatform,
}

/// A registered FIDO2 credential bound to a user address.
#[contracttype]
#[derive(Clone, Debug)]
pub struct Fido2Device {
    /// SHA-256 of the opaque credential ID returned by the authenticator.
    /// Used as an on-chain lookup key; the raw credential ID stays off-chain.
    pub credential_id_hash: BytesN<32>,
    /// Raw public key bytes:
    /// - Ed25519 → 32 bytes
    /// - P-256 uncompressed → 65 bytes (0x04 || x || y)
    pub public_key: Bytes,
    /// Signing algorithm for this credential.
    pub algorithm: PublicKeyAlgorithm,
    /// Monotonic signature counter from the authenticator (FIDO2 clone-detection).
    pub sign_count: u32,
    /// 16-byte Authenticator Attestation GUID identifying the authenticator model.
    pub aaguid: BytesN<16>,
    /// User-assigned friendly name (e.g., "iPhone 15 Pro", "YubiKey 5C NFC").
    pub device_name: String,
    /// Whether this is a platform or cross-platform authenticator.
    pub attachment: AuthenticatorAttachment,
    /// Transport mechanisms supported by this authenticator.
    pub transports: Vec<AuthenticatorTransport>,
    /// Ledger timestamp when this device was registered.
    pub created_at: u64,
    /// Ledger timestamp of the most recent successful assertion. 0 = never used.
    pub last_used_at: u64,
    /// `false` if the device has been revoked.
    pub is_active: bool,
    /// Whether the credential is eligible for backup (passkey / multi-device).
    pub backup_eligible: bool,
    /// Whether the credential is currently backed up to another device.
    pub backup_state: bool,
}

/// One-time challenge issued to the client for a registration or auth ceremony.
#[contracttype]
#[derive(Clone, Debug)]
pub struct PendingChallenge {
    /// 32 pseudorandom bytes to embed in `clientDataJSON`.
    pub challenge: BytesN<32>,
    /// Ledger timestamp when issued.
    pub created_at: u64,
    /// Ledger timestamp after which the challenge must be rejected.
    pub expires_at: u64,
}

/// Result returned after a successful FIDO2 assertion.
#[contracttype]
#[derive(Clone, Debug)]
pub struct AssertionResult {
    /// Credential that was used for authentication.
    pub credential_id_hash: BytesN<32>,
    /// Updated monotonic counter reported by the authenticator.
    pub new_sign_count: u32,
    /// Friendly name of the authenticating device.
    pub device_name: String,
    /// Attachment type of the authenticating device.
    pub attachment: AuthenticatorAttachment,
    /// Ledger timestamp at which authentication succeeded.
    pub verified_at: u64,
}

/// Audit record for a device revocation.
#[contracttype]
#[derive(Clone, Debug)]
pub struct RevocationRecord {
    /// Hash of the revoked credential ID.
    pub credential_id_hash: BytesN<32>,
    /// Device name at time of revocation.
    pub device_name: String,
    /// Ledger timestamp of revocation.
    pub revoked_at: u64,
    /// Address that triggered the revocation (user or admin).
    pub revoked_by: Address,
    /// Human-readable revocation reason.
    pub reason: String,
}

// ═══════════════════════════════════════════════════════════════════════════
// Storage keys
// ═══════════════════════════════════════════════════════════════════════════

#[contracttype]
pub enum DataKey {
    Admin,
    Initialized,
    /// Optional: address of the identity_registry contract.
    IdentityRegistry,
    /// Optional: address of the ZK verifier contract (required for ES256).
    ZkVerifier,
    /// SHA-256 of the relying party ID string (e.g., `sha256("uzima.health")`).
    RpIdHash,
    /// All registered devices for a user (active + revoked).
    UserDevices(Address),
    /// Outstanding registration or authentication challenge for a user.
    PendingChallenge(Address),
    /// Nullifiers consumed by ZK assertions (replay-attack prevention).
    UsedNullifier(BytesN<32>),
    /// Revocation audit log per user.
    RevocationHistory(Address),
}

// ═══════════════════════════════════════════════════════════════════════════
// Constants
// ═══════════════════════════════════════════════════════════════════════════

/// Maximum active devices per user (satisfies "5+" requirement; capped for gas).
const MAX_DEVICES: u32 = 10;

/// Challenge validity window in seconds (5 minutes per WebAuthn recommendation).
const CHALLENGE_TTL_SECS: u64 = 300;

/// Minimum authenticatorData byte length per FIDO2 spec:
/// rpIdHash (32) + flags (1) + signCount (4) = 37.
const MIN_AUTH_DATA_LEN: u32 = 37;

/// User Presence (UP) flag bitmask in the authenticatorData flags byte (index 32).
const FLAG_UP: u8 = 0x01;

/// Maximum friendly device name length in UTF-8 code units.
const MAX_DEVICE_NAME_LEN: u32 = 64;

/// Maximum revocation reason length.
const MAX_REASON_LEN: u32 = 256;

/// Ed25519 public key size in bytes.
const ED25519_KEY_LEN: u32 = 32;

/// P-256 uncompressed public key size in bytes (0x04 || x || y).
const P256_UNCOMPRESSED_KEY_LEN: u32 = 65;

/// P-256 compressed public key size in bytes (0x02/0x03 || x).
const P256_COMPRESSED_KEY_LEN: u32 = 33;

// ═══════════════════════════════════════════════════════════════════════════
// External contract client — ZK verifier
// ═══════════════════════════════════════════════════════════════════════════

#[soroban_sdk::contractclient(name = "ZkVerifierClient")]
pub trait ZkVerifierContract {
    fn verify_proof(
        env: Env,
        vk_version: u32,
        public_inputs_hash: BytesN<32>,
        proof: Bytes,
    ) -> bool;
}

// ═══════════════════════════════════════════════════════════════════════════
// External contract client — Identity Registry
// ═══════════════════════════════════════════════════════════════════════════

/// Minimal client interface for the `identity_registry` contract.
/// Calls `add_fido2_device` which binds the credential to the user's DID document.
#[soroban_sdk::contractclient(name = "IdentityRegistryClient")]
pub trait IdentityRegistryContract {
    fn add_fido2_device(
        env: Env,
        subject: Address,
        device_name: String,
        algorithm_tag: u32,
        public_key_hash: BytesN<32>,
    );
}

// ═══════════════════════════════════════════════════════════════════════════
// Contract implementation
// ═══════════════════════════════════════════════════════════════════════════

#[contract]
pub struct Fido2AuthenticatorContract;

#[contractimpl]
impl Fido2AuthenticatorContract {
    // ─────────────────────────────── Lifecycle ───────────────────────────────

    /// Initializes the contract.  Must be called exactly once.
    ///
    /// * `admin`      — address authorised to call administrative functions.
    /// * `rp_id_hash` — SHA-256 of the relying party identifier string
    ///                  (e.g., `sha256(b"uzima.health")`).
    pub fn initialize(env: Env, admin: Address, rp_id_hash: BytesN<32>) -> Result<(), Error> {
        admin.require_auth();
        if env.storage().instance().has(&DataKey::Initialized) {
            return Err(Error::AlreadyInitialized);
        }
        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage()
            .instance()
            .set(&DataKey::RpIdHash, &rp_id_hash);
        env.storage().instance().set(&DataKey::Initialized, &true);
        Ok(())
    }

    /// Configures the identity registry contract address.
    /// When set, `register_device` will bind new credentials to the caller's DID.
    pub fn set_identity_registry(
        env: Env,
        caller: Address,
        contract_id: Address,
    ) -> Result<(), Error> {
        caller.require_auth();
        Self::require_admin(&env, &caller)?;
        env.storage()
            .instance()
            .set(&DataKey::IdentityRegistry, &contract_id);
        Ok(())
    }

    /// Configures the ZK verifier contract used for ES256 (P-256) assertions.
    pub fn set_zk_verifier(env: Env, caller: Address, contract_id: Address) -> Result<(), Error> {
        caller.require_auth();
        Self::require_admin(&env, &caller)?;
        env.storage()
            .instance()
            .set(&DataKey::ZkVerifier, &contract_id);
        Ok(())
    }

    // ─────────────────────────── Device registration ─────────────────────────

    /// Issues a registration challenge for `user`.
    ///
    /// The 32-byte challenge must be embedded in `clientDataJSON.challenge` during
    /// the FIDO2 attestation ceremony.  Valid for 5 minutes.
    pub fn issue_registration_challenge(env: Env, user: Address) -> Result<BytesN<32>, Error> {
        user.require_auth();
        Self::require_initialized(&env)?;
        let challenge = Self::generate_challenge(&env, &user);
        let now = env.ledger().timestamp();
        env.storage().persistent().set(
            &DataKey::PendingChallenge(user),
            &PendingChallenge {
                challenge: challenge.clone(),
                created_at: now,
                expires_at: now + CHALLENGE_TTL_SECS,
            },
        );
        Ok(challenge)
    }

    /// Completes device registration after the FIDO2 attestation ceremony.
    ///
    /// Attestation statement verification is performed off-chain by a trusted
    /// relayer before calling this function.  The contract validates:
    /// - A non-expired challenge was issued for `user`.
    /// - The public key size matches the declared algorithm.
    /// - The credential has not been registered before.
    /// - `MAX_DEVICES` has not been reached.
    ///
    /// When the identity registry is configured the credential is also bound to
    /// the user's DID document as a FIDO2 verification method.
    ///
    /// Returns the zero-based device index.
    pub fn register_device(
        env: Env,
        user: Address,
        credential_id_hash: BytesN<32>,
        public_key: Bytes,
        algorithm: PublicKeyAlgorithm,
        device_name: String,
        attachment: AuthenticatorAttachment,
        transports: Vec<AuthenticatorTransport>,
        initial_sign_count: u32,
        aaguid: BytesN<16>,
        backup_eligible: bool,
    ) -> Result<u32, Error> {
        user.require_auth();
        Self::require_initialized(&env)?;

        // One-time challenge consumption (validates + removes the pending challenge).
        Self::consume_challenge(&env, &user)?;

        // Validate public key size against declared algorithm.
        Self::validate_public_key_size(&public_key, algorithm)?;

        // Validate device name length.
        if device_name.is_empty() || device_name.len() > MAX_DEVICE_NAME_LEN {
            return Err(Error::InvalidDeviceName);
        }

        // Load existing device list.
        let mut devices: Vec<Fido2Device> = env
            .storage()
            .persistent()
            .get(&DataKey::UserDevices(user.clone()))
            .unwrap_or_else(|| Vec::new(&env));

        // Enforce per-user device cap.
        if devices.len() >= MAX_DEVICES {
            return Err(Error::MaxDevicesReached);
        }

        // Reject duplicate credential IDs.
        for i in 0..devices.len() {
            if let Some(d) = devices.get(i) {
                if d.credential_id_hash == credential_id_hash {
                    return Err(Error::DeviceAlreadyRegistered);
                }
            }
        }

        let now = env.ledger().timestamp();
        let device_index = devices.len();

        // Compute public key hash for DID binding (SHA-256 of raw key bytes).
        let pk_hash: BytesN<32> = env.crypto().sha256(&public_key).into();

        devices.push_back(Fido2Device {
            credential_id_hash: credential_id_hash.clone(),
            public_key: public_key.clone(),
            algorithm,
            sign_count: initial_sign_count,
            aaguid,
            device_name: device_name.clone(),
            attachment,
            transports,
            created_at: now,
            last_used_at: 0,
            is_active: true,
            backup_eligible,
            backup_state: false,
        });

        env.storage()
            .persistent()
            .set(&DataKey::UserDevices(user.clone()), &devices);

        // Bind to DID document if identity registry is configured.
        let maybe_registry: Option<Address> =
            env.storage().instance().get(&DataKey::IdentityRegistry);
        if let Some(registry_addr) = maybe_registry {
            let algorithm_tag: u32 = match algorithm {
                PublicKeyAlgorithm::EdDSA => 1,
                PublicKeyAlgorithm::ES256 => 2,
            };
            let client = IdentityRegistryClient::new(&env, &registry_addr);
            // Best-effort: ignore errors so registration is not blocked if DID does not exist yet.
            let _ = client.try_add_fido2_device(&user, &device_name, &algorithm_tag, &pk_hash);
        }

        Ok(device_index)
    }

    // ──────────────────── Authentication — Ed25519 (EdDSA) ───────────────────

    /// Issues a one-time authentication challenge for `user`.
    pub fn issue_auth_challenge(env: Env, user: Address) -> Result<BytesN<32>, Error> {
        user.require_auth();
        Self::require_initialized(&env)?;
        let challenge = Self::generate_challenge(&env, &user);
        let now = env.ledger().timestamp();
        env.storage().persistent().set(
            &DataKey::PendingChallenge(user),
            &PendingChallenge {
                challenge: challenge.clone(),
                created_at: now,
                expires_at: now + CHALLENGE_TTL_SECS,
            },
        );
        Ok(challenge)
    }

    /// Verifies a FIDO2 assertion signed with Ed25519 (EdDSA).
    ///
    /// The signed payload per FIDO2 Level 2 spec is:
    /// `authenticatorData || SHA-256(clientDataJSON)`
    ///
    /// # Arguments
    /// * `user`               — authenticating user address.
    /// * `credential_id_hash` — SHA-256 of the credential ID.
    /// * `authenticator_data` — raw `authenticatorData` bytes (≥ 37 bytes).
    /// * `client_data_hash`   — `SHA-256(clientDataJSON)`.
    /// * `signature`          — 64-byte Ed25519 signature.
    /// * `new_sign_count`     — monotonic counter value from the authenticator.
    ///
    /// The transaction is aborted (host trap) if the Ed25519 signature is invalid.
    pub fn verify_ed25519_assertion(
        env: Env,
        user: Address,
        credential_id_hash: BytesN<32>,
        authenticator_data: Bytes,
        client_data_hash: BytesN<32>,
        signature: BytesN<64>,
        new_sign_count: u32,
    ) -> Result<AssertionResult, Error> {
        user.require_auth();
        Self::require_initialized(&env)?;

        // Consume the pending challenge.
        Self::consume_challenge(&env, &user)?;

        // Load device list and find the device.
        let mut devices: Vec<Fido2Device> = env
            .storage()
            .persistent()
            .get(&DataKey::UserDevices(user.clone()))
            .ok_or(Error::DeviceNotFound)?;

        let idx = Self::find_device_index(&devices, &credential_id_hash)?;
        let mut device = devices.get(idx).ok_or(Error::DeviceNotFound)?;

        if !device.is_active {
            return Err(Error::DeviceInactive);
        }
        if device.algorithm != PublicKeyAlgorithm::EdDSA {
            return Err(Error::AlgorithmKeyMismatch);
        }

        // Validate authenticatorData structure (length, rpIdHash, UP flag).
        Self::validate_authenticator_data(&env, &authenticator_data)?;

        // Build signed message: authenticatorData || clientDataHash.
        let mut message = authenticator_data.clone();
        let hash_bytes: Bytes = client_data_hash.into();
        message.append(&hash_bytes);

        // Extract 32-byte Ed25519 public key.
        let pub_key = Self::bytes_to_ed25519_key(&env, &device.public_key)?;

        // Verify — panics (host trap) if signature is invalid.
        env.crypto().ed25519_verify(&pub_key, &message, &signature);

        // FIDO2 clone-detection: sign count must strictly increase when non-zero.
        if new_sign_count > 0 && new_sign_count <= device.sign_count {
            return Err(Error::SignCountRegression);
        }

        let now = env.ledger().timestamp();
        device.sign_count = new_sign_count;
        device.last_used_at = now;
        devices.set(idx, device.clone());
        env.storage()
            .persistent()
            .set(&DataKey::UserDevices(user), &devices);

        Ok(AssertionResult {
            credential_id_hash,
            new_sign_count,
            device_name: device.device_name,
            attachment: device.attachment,
            verified_at: now,
        })
    }

    // ─────────────────── Authentication — ES256 via ZK proof ─────────────────

    /// Verifies a FIDO2 assertion for a P-256 (ES256) credential using a ZK proof.
    ///
    /// Because Soroban does not natively support P-256 ECDSA verification, the
    /// caller submits a ZK proof generated by a trusted off-chain prover that
    /// attests to a valid P-256 signature over `authenticatorData || clientDataHash`.
    ///
    /// The proof also enables privacy-preserving authentication: the `nullifier`
    /// and `commitment` allow proving key ownership without disclosing the exact
    /// device on every authentication.
    ///
    /// # Arguments
    /// * `credential_id_hash` — identifies which registered P-256 device is used.
    /// * `nullifier`          — unique value preventing proof replay.
    /// * `commitment`         — public commitment included in the ZK circuit.
    /// * `proof`              — ZK proof bytes forwarded to the verifier contract.
    /// * `new_sign_count`     — monotonic counter value from the authenticator.
    /// * `vk_version`         — verifying key version for the ZK circuit.
    pub fn verify_zk_assertion(
        env: Env,
        user: Address,
        credential_id_hash: BytesN<32>,
        nullifier: BytesN<32>,
        commitment: BytesN<32>,
        proof: Bytes,
        new_sign_count: u32,
        vk_version: u32,
    ) -> Result<AssertionResult, Error> {
        user.require_auth();
        Self::require_initialized(&env)?;

        let zk_verifier: Address = env
            .storage()
            .instance()
            .get(&DataKey::ZkVerifier)
            .ok_or(Error::ZkVerifierNotSet)?;

        // Replay protection: nullifier must not have been used before.
        if env
            .storage()
            .persistent()
            .has(&DataKey::UsedNullifier(nullifier.clone()))
        {
            return Err(Error::NullifierAlreadyUsed);
        }

        // Consume challenge.
        Self::consume_challenge(&env, &user)?;

        // Load device.
        let mut devices: Vec<Fido2Device> = env
            .storage()
            .persistent()
            .get(&DataKey::UserDevices(user.clone()))
            .ok_or(Error::DeviceNotFound)?;

        let idx = Self::find_device_index(&devices, &credential_id_hash)?;
        let mut device = devices.get(idx).ok_or(Error::DeviceNotFound)?;

        if !device.is_active {
            return Err(Error::DeviceInactive);
        }
        if device.algorithm != PublicKeyAlgorithm::ES256 {
            return Err(Error::AlgorithmKeyMismatch);
        }

        // Build public inputs hash: commitment || credential_id_hash || nullifier.
        let mut inputs: Bytes = commitment.clone().into();
        let cred_bytes: Bytes = credential_id_hash.clone().into();
        let null_bytes: Bytes = nullifier.clone().into();
        inputs.append(&cred_bytes);
        inputs.append(&null_bytes);
        let public_inputs_hash: BytesN<32> = env.crypto().sha256(&inputs).into();

        // Delegate proof verification to the ZK verifier contract.
        let client = ZkVerifierClient::new(&env, &zk_verifier);
        if !client.verify_proof(&vk_version, &public_inputs_hash, &proof) {
            return Err(Error::InvalidSignature);
        }

        // Clone detection.
        if new_sign_count > 0 && new_sign_count <= device.sign_count {
            return Err(Error::SignCountRegression);
        }

        // Permanently consume the nullifier.
        env.storage()
            .persistent()
            .set(&DataKey::UsedNullifier(nullifier), &true);

        let now = env.ledger().timestamp();
        device.sign_count = new_sign_count;
        device.last_used_at = now;
        devices.set(idx, device.clone());
        env.storage()
            .persistent()
            .set(&DataKey::UserDevices(user), &devices);

        Ok(AssertionResult {
            credential_id_hash,
            new_sign_count,
            device_name: device.device_name,
            attachment: device.attachment,
            verified_at: now,
        })
    }

    // ─────────────────────────── Device management ───────────────────────────

    /// Revokes a device, preventing it from being used for future authentications.
    ///
    /// Both the device owner (`user`) and the contract admin may revoke devices.
    /// A `RevocationRecord` is appended to the user's audit log.
    pub fn revoke_device(
        env: Env,
        caller: Address,
        user: Address,
        credential_id_hash: BytesN<32>,
        reason: String,
    ) -> Result<(), Error> {
        caller.require_auth();
        Self::require_initialized(&env)?;

        // Only the user or admin may revoke.
        let admin: Address = env
            .storage()
            .instance()
            .get(&DataKey::Admin)
            .ok_or(Error::NotInitialized)?;
        if caller != user && caller != admin {
            return Err(Error::NotAuthorized);
        }
        if reason.is_empty() || reason.len() > MAX_REASON_LEN {
            return Err(Error::InvalidRevocationReason);
        }

        let mut devices: Vec<Fido2Device> = env
            .storage()
            .persistent()
            .get(&DataKey::UserDevices(user.clone()))
            .ok_or(Error::DeviceNotFound)?;

        let idx = Self::find_device_index(&devices, &credential_id_hash)?;
        let mut device = devices.get(idx).ok_or(Error::DeviceNotFound)?;

        if !device.is_active {
            return Err(Error::DeviceInactive);
        }

        // Append revocation record to audit log.
        let record = RevocationRecord {
            credential_id_hash: credential_id_hash.clone(),
            device_name: device.device_name.clone(),
            revoked_at: env.ledger().timestamp(),
            revoked_by: caller,
            reason,
        };
        let mut history: Vec<RevocationRecord> = env
            .storage()
            .persistent()
            .get(&DataKey::RevocationHistory(user.clone()))
            .unwrap_or_else(|| Vec::new(&env));
        history.push_back(record);
        env.storage()
            .persistent()
            .set(&DataKey::RevocationHistory(user.clone()), &history);

        // Deactivate the device.
        device.is_active = false;
        devices.set(idx, device);
        env.storage()
            .persistent()
            .set(&DataKey::UserDevices(user), &devices);

        Ok(())
    }

    /// Updates the user-assigned friendly name of a registered device.
    pub fn update_device_name(
        env: Env,
        user: Address,
        credential_id_hash: BytesN<32>,
        new_name: String,
    ) -> Result<(), Error> {
        user.require_auth();
        Self::require_initialized(&env)?;

        if new_name.is_empty() || new_name.len() > MAX_DEVICE_NAME_LEN {
            return Err(Error::InvalidDeviceName);
        }

        let mut devices: Vec<Fido2Device> = env
            .storage()
            .persistent()
            .get(&DataKey::UserDevices(user.clone()))
            .ok_or(Error::DeviceNotFound)?;

        let idx = Self::find_device_index(&devices, &credential_id_hash)?;
        let mut device = devices.get(idx).ok_or(Error::DeviceNotFound)?;
        device.device_name = new_name;
        devices.set(idx, device);
        env.storage()
            .persistent()
            .set(&DataKey::UserDevices(user), &devices);

        Ok(())
    }

    /// Returns all devices registered for `user` (active and revoked).
    ///
    /// Only the user or the admin may call this function.
    pub fn list_devices(
        env: Env,
        caller: Address,
        user: Address,
    ) -> Result<Vec<Fido2Device>, Error> {
        caller.require_auth();
        Self::require_initialized(&env)?;

        let admin: Address = env
            .storage()
            .instance()
            .get(&DataKey::Admin)
            .ok_or(Error::NotInitialized)?;
        if caller != user && caller != admin {
            return Err(Error::NotAuthorized);
        }

        let devices: Vec<Fido2Device> = env
            .storage()
            .persistent()
            .get(&DataKey::UserDevices(user))
            .unwrap_or_else(|| Vec::new(&env));
        Ok(devices)
    }

    /// Returns the total device count (active + revoked) for `user`.
    pub fn get_device_count(env: Env, user: Address) -> u32 {
        env.storage()
            .persistent()
            .get::<_, Vec<Fido2Device>>(&DataKey::UserDevices(user))
            .map(|v| v.len())
            .unwrap_or(0)
    }

    /// Returns the number of active (non-revoked) devices for `user`.
    pub fn get_active_device_count(env: Env, user: Address) -> u32 {
        let devices: Vec<Fido2Device> = env
            .storage()
            .persistent()
            .get(&DataKey::UserDevices(user))
            .unwrap_or_else(|| Vec::new(&env));
        let mut count = 0u32;
        for i in 0..devices.len() {
            if let Some(d) = devices.get(i) {
                if d.is_active {
                    count += 1;
                }
            }
        }
        count
    }

    /// Returns `true` if `credential_id_hash` is registered and active for `user`.
    pub fn is_device_registered(env: Env, user: Address, credential_id_hash: BytesN<32>) -> bool {
        let devices: Vec<Fido2Device> = env
            .storage()
            .persistent()
            .get(&DataKey::UserDevices(user))
            .unwrap_or_else(|| Vec::new(&env));
        for i in 0..devices.len() {
            if let Some(d) = devices.get(i) {
                if d.credential_id_hash == credential_id_hash && d.is_active {
                    return true;
                }
            }
        }
        false
    }

    /// Returns the full revocation audit history for `user`.
    ///
    /// Only the user or the admin may call this function.
    pub fn get_revocation_history(
        env: Env,
        caller: Address,
        user: Address,
    ) -> Result<Vec<RevocationRecord>, Error> {
        caller.require_auth();
        Self::require_initialized(&env)?;

        let admin: Address = env
            .storage()
            .instance()
            .get(&DataKey::Admin)
            .ok_or(Error::NotInitialized)?;
        if caller != user && caller != admin {
            return Err(Error::NotAuthorized);
        }

        let history: Vec<RevocationRecord> = env
            .storage()
            .persistent()
            .get(&DataKey::RevocationHistory(user))
            .unwrap_or_else(|| Vec::new(&env));
        Ok(history)
    }

    // ─────────────────────────────── Helpers ─────────────────────────────────

    fn require_initialized(env: &Env) -> Result<(), Error> {
        if !env.storage().instance().has(&DataKey::Initialized) {
            return Err(Error::NotInitialized);
        }
        Ok(())
    }

    fn require_admin(env: &Env, caller: &Address) -> Result<(), Error> {
        let admin: Address = env
            .storage()
            .instance()
            .get(&DataKey::Admin)
            .ok_or(Error::NotInitialized)?;
        if *caller != admin {
            return Err(Error::NotAuthorized);
        }
        Ok(())
    }

    /// Derives a 32-byte pseudorandom challenge from ledger state and the user address.
    ///
    /// Not cryptographically random in the full sense (deterministic per block), but
    /// sufficient as a FIDO2 challenge because the ledger timestamp + sequence already
    /// make the value unpredictable to the authenticator before it is issued.
    fn generate_challenge(env: &Env, user: &Address) -> BytesN<32> {
        let mut data = user.clone().to_xdr(env);
        let mut state: Vec<u64> = Vec::new(env);
        state.push_back(env.ledger().timestamp());
        state.push_back(env.ledger().sequence() as u64);
        data.append(&state.to_xdr(env));
        env.crypto().sha256(&data).into()
    }

    /// Validates that a pending challenge exists for `user`, is not expired,
    /// and removes it from storage (one-time use).
    fn consume_challenge(env: &Env, user: &Address) -> Result<(), Error> {
        let key = DataKey::PendingChallenge(user.clone());
        let pending: PendingChallenge = env
            .storage()
            .persistent()
            .get(&key)
            .ok_or(Error::NoChallengeIssued)?;

        env.storage().persistent().remove(&key);

        if env.ledger().timestamp() > pending.expires_at {
            return Err(Error::ChallengeExpired);
        }
        Ok(())
    }

    /// Validates `authenticatorData` structure per FIDO2 Level 2 §6.1:
    /// - Minimum length 37 bytes.
    /// - First 32 bytes (rpIdHash) must match the contract's configured RP ID hash.
    /// - Byte 32 (flags) must have the User Presence (UP) bit set.
    fn validate_authenticator_data(env: &Env, auth_data: &Bytes) -> Result<(), Error> {
        if auth_data.len() < MIN_AUTH_DATA_LEN {
            return Err(Error::InvalidAuthenticatorData);
        }

        // Validate rpIdHash (bytes 0–31).
        let rp_id_hash: BytesN<32> = env
            .storage()
            .instance()
            .get(&DataKey::RpIdHash)
            .ok_or(Error::NotInitialized)?;

        let mut rp_arr = [0u8; 32];
        for (i, byte) in rp_arr.iter_mut().enumerate() {
            *byte = auth_data.get(i as u32).unwrap_or(0);
        }
        let auth_rp_hash = BytesN::<32>::from_array(env, &rp_arr);
        if auth_rp_hash != rp_id_hash {
            return Err(Error::RpIdMismatch);
        }

        // Validate User Presence flag (byte 32).
        let flags = auth_data.get(32).unwrap_or(0);
        if flags & FLAG_UP == 0 {
            return Err(Error::UserPresenceNotVerified);
        }

        Ok(())
    }

    /// Validates that the public key byte length is consistent with `algorithm`.
    fn validate_public_key_size(
        public_key: &Bytes,
        algorithm: PublicKeyAlgorithm,
    ) -> Result<(), Error> {
        let len = public_key.len();
        match algorithm {
            PublicKeyAlgorithm::EdDSA => {
                if len != ED25519_KEY_LEN {
                    return Err(Error::AlgorithmKeyMismatch);
                }
            }
            PublicKeyAlgorithm::ES256 => {
                if len != P256_UNCOMPRESSED_KEY_LEN && len != P256_COMPRESSED_KEY_LEN {
                    return Err(Error::AlgorithmKeyMismatch);
                }
            }
        }
        if len == 0 {
            return Err(Error::InvalidPublicKey);
        }
        Ok(())
    }

    /// Converts a `Bytes` buffer of exactly 32 bytes into a `BytesN<32>`.
    fn bytes_to_ed25519_key(env: &Env, key_bytes: &Bytes) -> Result<BytesN<32>, Error> {
        if key_bytes.len() != ED25519_KEY_LEN {
            return Err(Error::InvalidPublicKey);
        }
        let mut arr = [0u8; 32];
        for (i, byte) in arr.iter_mut().enumerate() {
            *byte = key_bytes.get(i as u32).unwrap_or(0);
        }
        Ok(BytesN::<32>::from_array(env, &arr))
    }

    /// Returns the index of the device with the given `credential_id_hash`,
    /// or `DeviceNotFound` if no match exists.
    fn find_device_index(
        devices: &Vec<Fido2Device>,
        credential_id_hash: &BytesN<32>,
    ) -> Result<u32, Error> {
        for i in 0..devices.len() {
            if let Some(d) = devices.get(i) {
                if d.credential_id_hash == *credential_id_hash {
                    return Ok(i);
                }
            }
        }
        Err(Error::DeviceNotFound)
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// Tests
// ═══════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]
    #![allow(clippy::expect_used)]

    use super::*;
    use soroban_sdk::{
        testutils::{Address as _, Ledger},
        Bytes, BytesN, Env, String, Vec,
    };

    // ─────────────────────────── Test helpers ────────────────────────────────

    /// SHA-256 of the ASCII string "uzima.health" — used as the test RP ID hash.
    const TEST_RP_ID_HASH: [u8; 32] = [
        0x27, 0x08, 0x6a, 0x75, 0x68, 0x88, 0xde, 0x5c, 0xd7, 0x93, 0x04, 0x4d, 0x4b, 0x79, 0x3c,
        0x21, 0x4a, 0x4e, 0x8c, 0x7c, 0x86, 0xc3, 0xd4, 0x7e, 0x36, 0xaf, 0xbc, 0xd3, 0x3e, 0x0b,
        0xed, 0x9c,
    ];

    fn setup_contract(env: &Env) -> (Fido2AuthenticatorContractClient<'_>, Address) {
        let admin = Address::generate(env);
        let contract_id = env.register_contract(None, Fido2AuthenticatorContract);
        let client = Fido2AuthenticatorContractClient::new(env, &contract_id);
        let rp_id_hash = BytesN::from_array(env, &TEST_RP_ID_HASH);
        client.initialize(&admin, &rp_id_hash);
        (client, admin)
    }

    /// Builds a minimal valid authenticatorData (37 bytes) with the test RP ID hash,
    /// UP flag set, and the given sign count encoded big-endian at bytes 33-36.
    fn make_auth_data(env: &Env, sign_count: u32) -> Bytes {
        let mut data = [0u8; 37];
        // bytes 0-31: rpIdHash
        data[..32].copy_from_slice(&TEST_RP_ID_HASH);
        // byte 32: flags — UP bit set
        data[32] = FLAG_UP;
        // bytes 33-36: sign count (big-endian)
        data[33] = ((sign_count >> 24) & 0xff) as u8;
        data[34] = ((sign_count >> 16) & 0xff) as u8;
        data[35] = ((sign_count >> 8) & 0xff) as u8;
        data[36] = (sign_count & 0xff) as u8;
        Bytes::from_array(env, &data)
    }

    /// Registers a dummy Ed25519 device for `user`, returning the credential_id_hash.
    fn register_dummy_device(
        client: &Fido2AuthenticatorContractClient,
        env: &Env,
        user: &Address,
    ) -> BytesN<32> {
        client.issue_registration_challenge(user);
        let credential_id_hash = BytesN::from_array(env, &[0x01u8; 32]);
        let public_key = Bytes::from_array(env, &[0x02u8; 32]); // 32-byte Ed25519 key
        let device_name = String::from_str(env, "Test Device");
        let transports: Vec<AuthenticatorTransport> = Vec::new(env);
        client.register_device(
            user,
            &credential_id_hash,
            &public_key,
            &PublicKeyAlgorithm::EdDSA,
            &device_name,
            &AuthenticatorAttachment::Platform,
            &transports,
            &0u32,
            &BytesN::from_array(env, &[0u8; 16]),
            &false,
        );
        credential_id_hash
    }

    // ─────────────────────────── Lifecycle ───────────────────────────────────

    #[test]
    fn test_initialize_success() {
        let env = Env::default();
        env.mock_all_auths();
        let (_, _admin) = setup_contract(&env);
        // Passes if no panic.
    }

    #[test]
    fn test_double_initialize_fails() {
        let env = Env::default();
        env.mock_all_auths();
        let (client, admin) = setup_contract(&env);
        let rp_id_hash = BytesN::from_array(&env, &TEST_RP_ID_HASH);
        let result = client.try_initialize(&admin, &rp_id_hash);
        assert_eq!(result, Err(Ok(Error::AlreadyInitialized)));
    }

    // ─────────────────────────── Registration ────────────────────────────────

    #[test]
    fn test_register_device_success() {
        let env = Env::default();
        env.mock_all_auths();
        env.ledger().with_mut(|l| l.timestamp = 1000);
        let (client, _) = setup_contract(&env);
        let user = Address::generate(&env);

        let cred_hash = register_dummy_device(&client, &env, &user);

        assert!(client.is_device_registered(&user, &cred_hash));
        assert_eq!(client.get_device_count(&user), 1);
        assert_eq!(client.get_active_device_count(&user), 1);
    }

    #[test]
    fn test_register_without_challenge_fails() {
        let env = Env::default();
        env.mock_all_auths();
        let (client, _) = setup_contract(&env);
        let user = Address::generate(&env);

        let cred_hash = BytesN::from_array(&env, &[0x01u8; 32]);
        let pub_key = Bytes::from_array(&env, &[0x02u8; 32]);
        let name = String::from_str(&env, "Device");
        let transports: Vec<AuthenticatorTransport> = Vec::new(&env);
        let result = client.try_register_device(
            &user,
            &cred_hash,
            &pub_key,
            &PublicKeyAlgorithm::EdDSA,
            &name,
            &AuthenticatorAttachment::Platform,
            &transports,
            &0u32,
            &BytesN::from_array(&env, &[0u8; 16]),
            &false,
        );
        assert_eq!(result, Err(Ok(Error::NoChallengeIssued)));
    }

    #[test]
    fn test_duplicate_device_rejected() {
        let env = Env::default();
        env.mock_all_auths();
        env.ledger().with_mut(|l| l.timestamp = 1000);
        let (client, _) = setup_contract(&env);
        let user = Address::generate(&env);

        register_dummy_device(&client, &env, &user);

        // Attempt to register the same credential_id_hash again.
        client.issue_registration_challenge(&user);
        let cred_hash = BytesN::from_array(&env, &[0x01u8; 32]);
        let pub_key = Bytes::from_array(&env, &[0x02u8; 32]);
        let name = String::from_str(&env, "Duplicate");
        let transports: Vec<AuthenticatorTransport> = Vec::new(&env);
        let result = client.try_register_device(
            &user,
            &cred_hash,
            &pub_key,
            &PublicKeyAlgorithm::EdDSA,
            &name,
            &AuthenticatorAttachment::Platform,
            &transports,
            &0u32,
            &BytesN::from_array(&env, &[0u8; 16]),
            &false,
        );
        assert_eq!(result, Err(Ok(Error::DeviceAlreadyRegistered)));
    }

    #[test]
    fn test_max_devices_limit_enforced() {
        let env = Env::default();
        env.mock_all_auths();
        env.ledger().with_mut(|l| l.timestamp = 1000);
        let (client, _) = setup_contract(&env);
        let user = Address::generate(&env);

        // Register MAX_DEVICES devices.
        for i in 0..MAX_DEVICES {
            client.issue_registration_challenge(&user);
            let mut cred = [0u8; 32];
            cred[0] = i as u8;
            let cred_hash = BytesN::from_array(&env, &cred);
            let pub_key = Bytes::from_array(&env, &[i as u8; 32]);
            let name = String::from_str(&env, "Device");
            let transports: Vec<AuthenticatorTransport> = Vec::new(&env);
            client.register_device(
                &user,
                &cred_hash,
                &pub_key,
                &PublicKeyAlgorithm::EdDSA,
                &name,
                &AuthenticatorAttachment::Platform,
                &transports,
                &0u32,
                &BytesN::from_array(&env, &[0u8; 16]),
                &false,
            );
        }

        // The (MAX_DEVICES + 1)-th registration must fail.
        client.issue_registration_challenge(&user);
        let overflow_hash = BytesN::from_array(&env, &[0xffu8; 32]);
        let pub_key = Bytes::from_array(&env, &[0xffu8; 32]);
        let name = String::from_str(&env, "Overflow");
        let transports: Vec<AuthenticatorTransport> = Vec::new(&env);
        let result = client.try_register_device(
            &user,
            &overflow_hash,
            &pub_key,
            &PublicKeyAlgorithm::EdDSA,
            &name,
            &AuthenticatorAttachment::Platform,
            &transports,
            &0u32,
            &BytesN::from_array(&env, &[0u8; 16]),
            &false,
        );
        assert_eq!(result, Err(Ok(Error::MaxDevicesReached)));
    }

    #[test]
    fn test_wrong_key_size_rejected() {
        let env = Env::default();
        env.mock_all_auths();
        env.ledger().with_mut(|l| l.timestamp = 1000);
        let (client, _) = setup_contract(&env);
        let user = Address::generate(&env);

        client.issue_registration_challenge(&user);
        let cred_hash = BytesN::from_array(&env, &[0x01u8; 32]);
        // 16-byte key is not valid for any supported algorithm.
        let bad_key = Bytes::from_array(&env, &[0xAAu8; 16]);
        let name = String::from_str(&env, "Bad Key Device");
        let transports: Vec<AuthenticatorTransport> = Vec::new(&env);
        let result = client.try_register_device(
            &user,
            &cred_hash,
            &bad_key,
            &PublicKeyAlgorithm::EdDSA,
            &name,
            &AuthenticatorAttachment::Platform,
            &transports,
            &0u32,
            &BytesN::from_array(&env, &[0u8; 16]),
            &false,
        );
        assert_eq!(result, Err(Ok(Error::AlgorithmKeyMismatch)));
    }

    // ─────────────────────── Challenge expiry ────────────────────────────────

    #[test]
    fn test_expired_challenge_rejected() {
        let env = Env::default();
        env.mock_all_auths();
        env.ledger().with_mut(|l| l.timestamp = 1000);
        let (client, _) = setup_contract(&env);
        let user = Address::generate(&env);

        client.issue_registration_challenge(&user);

        // Advance time past the TTL.
        env.ledger()
            .with_mut(|l| l.timestamp = 1000 + CHALLENGE_TTL_SECS + 1);

        let cred_hash = BytesN::from_array(&env, &[0x01u8; 32]);
        let pub_key = Bytes::from_array(&env, &[0x02u8; 32]);
        let name = String::from_str(&env, "Late Device");
        let transports: Vec<AuthenticatorTransport> = Vec::new(&env);
        let result = client.try_register_device(
            &user,
            &cred_hash,
            &pub_key,
            &PublicKeyAlgorithm::EdDSA,
            &name,
            &AuthenticatorAttachment::Platform,
            &transports,
            &0u32,
            &BytesN::from_array(&env, &[0u8; 16]),
            &false,
        );
        assert_eq!(result, Err(Ok(Error::ChallengeExpired)));
    }

    // ─────────────────────────── Device management ───────────────────────────

    #[test]
    fn test_revoke_device_success() {
        let env = Env::default();
        env.mock_all_auths();
        env.ledger().with_mut(|l| l.timestamp = 1000);
        let (client, _) = setup_contract(&env);
        let user = Address::generate(&env);

        let cred_hash = register_dummy_device(&client, &env, &user);
        assert!(client.is_device_registered(&user, &cred_hash));

        let reason = String::from_str(&env, "Lost device");
        client.revoke_device(&user, &user, &cred_hash, &reason);

        assert!(!client.is_device_registered(&user, &cred_hash));
        assert_eq!(client.get_active_device_count(&user), 0);
        // Total count includes revoked devices.
        assert_eq!(client.get_device_count(&user), 1);
    }

    #[test]
    fn test_revoke_already_revoked_fails() {
        let env = Env::default();
        env.mock_all_auths();
        env.ledger().with_mut(|l| l.timestamp = 1000);
        let (client, _) = setup_contract(&env);
        let user = Address::generate(&env);

        let cred_hash = register_dummy_device(&client, &env, &user);
        let reason = String::from_str(&env, "Stolen");
        client.revoke_device(&user, &user, &cred_hash, &reason);

        let result = client.try_revoke_device(&user, &user, &cred_hash, &reason);
        assert_eq!(result, Err(Ok(Error::DeviceInactive)));
    }

    #[test]
    fn test_unauthorized_revocation_fails() {
        let env = Env::default();
        env.mock_all_auths();
        env.ledger().with_mut(|l| l.timestamp = 1000);
        let (client, _) = setup_contract(&env);
        let user = Address::generate(&env);
        let attacker = Address::generate(&env);

        let cred_hash = register_dummy_device(&client, &env, &user);
        let reason = String::from_str(&env, "Unauthorized");
        let result = client.try_revoke_device(&attacker, &user, &cred_hash, &reason);
        assert_eq!(result, Err(Ok(Error::NotAuthorized)));
    }

    #[test]
    fn test_admin_can_revoke() {
        let env = Env::default();
        env.mock_all_auths();
        env.ledger().with_mut(|l| l.timestamp = 1000);
        let (client, admin) = setup_contract(&env);
        let user = Address::generate(&env);

        let cred_hash = register_dummy_device(&client, &env, &user);
        let reason = String::from_str(&env, "Security policy");
        client.revoke_device(&admin, &user, &cred_hash, &reason);

        assert!(!client.is_device_registered(&user, &cred_hash));
    }

    #[test]
    fn test_update_device_name() {
        let env = Env::default();
        env.mock_all_auths();
        env.ledger().with_mut(|l| l.timestamp = 1000);
        let (client, _) = setup_contract(&env);
        let user = Address::generate(&env);

        let cred_hash = register_dummy_device(&client, &env, &user);
        let new_name = String::from_str(&env, "My YubiKey 5C NFC");
        client.update_device_name(&user, &cred_hash, &new_name);

        let devices = client.list_devices(&user, &user);
        assert_eq!(devices.get(0).unwrap().device_name, new_name);
    }

    #[test]
    fn test_list_devices_unauthorized() {
        let env = Env::default();
        env.mock_all_auths();
        env.ledger().with_mut(|l| l.timestamp = 1000);
        let (client, _) = setup_contract(&env);
        let user = Address::generate(&env);
        let attacker = Address::generate(&env);

        register_dummy_device(&client, &env, &user);

        let result = client.try_list_devices(&attacker, &user);
        assert!(matches!(result, Err(Ok(Error::NotAuthorized))));
    }

    #[test]
    fn test_revocation_history_recorded() {
        let env = Env::default();
        env.mock_all_auths();
        env.ledger().with_mut(|l| l.timestamp = 1000);
        let (client, _) = setup_contract(&env);
        let user = Address::generate(&env);

        let cred_hash = register_dummy_device(&client, &env, &user);
        let reason = String::from_str(&env, "Replaced with newer device");
        client.revoke_device(&user, &user, &cred_hash, &reason);

        let history = client.get_revocation_history(&user, &user);
        assert_eq!(history.len(), 1);
        let record = history.get(0).unwrap();
        assert_eq!(record.credential_id_hash, cred_hash);
        assert_eq!(record.reason, reason);
        assert_eq!(record.revoked_by, user);
    }

    #[test]
    fn test_multiple_devices_per_user() {
        let env = Env::default();
        env.mock_all_auths();
        env.ledger().with_mut(|l| l.timestamp = 1000);
        let (client, _) = setup_contract(&env);
        let user = Address::generate(&env);

        // Register 5 different devices.
        for i in 0u8..5 {
            client.issue_registration_challenge(&user);
            let mut cred = [0u8; 32];
            cred[0] = i;
            let cred_hash = BytesN::from_array(&env, &cred);
            let pub_key = Bytes::from_array(&env, &[i; 32]);
            let name = String::from_str(&env, "Device");
            let transports: Vec<AuthenticatorTransport> = Vec::new(&env);
            client.register_device(
                &user,
                &cred_hash,
                &pub_key,
                &PublicKeyAlgorithm::EdDSA,
                &name,
                &AuthenticatorAttachment::Platform,
                &transports,
                &0u32,
                &BytesN::from_array(&env, &[0u8; 16]),
                &false,
            );
        }

        assert_eq!(client.get_active_device_count(&user), 5);
        assert_eq!(client.get_device_count(&user), 5);
    }

    // ──────────────────── Ed25519 assertion verification ─────────────────────

    #[test]
    fn test_ed25519_assertion_valid_signature() {
        use ed25519_dalek::{Signer, SigningKey};
        use rand::rngs::OsRng;

        let env = Env::default();
        env.mock_all_auths();
        env.ledger().with_mut(|l| l.timestamp = 1000);
        let (client, _) = setup_contract(&env);
        let user = Address::generate(&env);

        // Generate a real Ed25519 key pair.
        let mut rng = OsRng;
        let signing_key = SigningKey::generate(&mut rng);
        let verifying_key_bytes = signing_key.verifying_key().to_bytes();

        // Register the device with the real public key.
        client.issue_registration_challenge(&user);
        let cred_hash = BytesN::from_array(&env, &[0xA1u8; 32]);
        let public_key = Bytes::from_array(&env, &verifying_key_bytes);
        let name = String::from_str(&env, "iPhone 15 Pro");
        let transports: Vec<AuthenticatorTransport> = Vec::new(&env);
        client.register_device(
            &user,
            &cred_hash,
            &public_key,
            &PublicKeyAlgorithm::EdDSA,
            &name,
            &AuthenticatorAttachment::Platform,
            &transports,
            &0u32,
            &BytesN::from_array(&env, &[0u8; 16]),
            &true, // backup_eligible (passkey)
        );

        // Issue an auth challenge.
        client.issue_auth_challenge(&user);

        // Build authenticatorData with sign count = 1.
        let auth_data_bytes = make_auth_data(&env, 1);

        // Build clientDataHash (any 32-byte hash; represents SHA-256(clientDataJSON)).
        let client_data_hash = BytesN::from_array(&env, &[0x42u8; 32]);

        // Build the message the authenticator signs: authenticatorData (37 bytes) || clientDataHash (32 bytes).
        let mut message = [0u8; 69];
        for (i, byte) in message.iter_mut().enumerate().take(37) {
            *byte = auth_data_bytes.get(i as u32).unwrap_or(0);
        }
        message[37..69].copy_from_slice(&[0x42u8; 32]);

        // Sign the message.
        let sig_bytes = signing_key.sign(&message).to_bytes();
        let signature = BytesN::from_array(&env, &sig_bytes);

        // Verify the assertion on-chain.
        let result = client.verify_ed25519_assertion(
            &user,
            &cred_hash,
            &auth_data_bytes,
            &client_data_hash,
            &signature,
            &1u32,
        );
        assert_eq!(result.new_sign_count, 1);
        assert_eq!(result.device_name, name);
    }

    #[test]
    fn test_sign_count_regression_detected() {
        use ed25519_dalek::{Signer, SigningKey};
        use rand::rngs::OsRng;

        let env = Env::default();
        env.mock_all_auths();
        env.ledger().with_mut(|l| l.timestamp = 1000);
        let (client, _) = setup_contract(&env);
        let user = Address::generate(&env);

        let mut rng = OsRng;
        let signing_key = SigningKey::generate(&mut rng);
        let vk_bytes = signing_key.verifying_key().to_bytes();

        // Register device with initial sign_count = 5.
        client.issue_registration_challenge(&user);
        let cred_hash = BytesN::from_array(&env, &[0xB1u8; 32]);
        let public_key = Bytes::from_array(&env, &vk_bytes);
        let name = String::from_str(&env, "YubiKey");
        let transports: Vec<AuthenticatorTransport> = Vec::new(&env);
        client.register_device(
            &user,
            &cred_hash,
            &public_key,
            &PublicKeyAlgorithm::EdDSA,
            &name,
            &AuthenticatorAttachment::CrossPlatform,
            &transports,
            &5u32, // already at count 5
            &BytesN::from_array(&env, &[0u8; 16]),
            &false,
        );

        // Attempt assertion with a sign count of 3 (< 5 → clone detected).
        client.issue_auth_challenge(&user);
        let auth_data = make_auth_data(&env, 3);
        let client_data_hash = BytesN::from_array(&env, &[0x11u8; 32]);

        let mut msg = [0u8; 69];
        for (i, byte) in msg.iter_mut().enumerate().take(37) {
            *byte = auth_data.get(i as u32).unwrap_or(0);
        }
        msg[37..69].copy_from_slice(&[0x11u8; 32]);
        let sig = BytesN::from_array(&env, &signing_key.sign(&msg).to_bytes());

        let result = client.try_verify_ed25519_assertion(
            &user,
            &cred_hash,
            &auth_data,
            &client_data_hash,
            &sig,
            &3u32,
        );
        assert!(matches!(result, Err(Ok(Error::SignCountRegression))));
    }

    #[test]
    fn test_rp_id_mismatch_rejected() {
        let env = Env::default();
        env.mock_all_auths();
        env.ledger().with_mut(|l| l.timestamp = 1000);
        let (client, _) = setup_contract(&env);
        let user = Address::generate(&env);

        let cred_hash = register_dummy_device(&client, &env, &user);
        client.issue_auth_challenge(&user);

        // Build authenticatorData with a WRONG rpIdHash.
        let mut bad_auth_data_arr = [0u8; 37];
        bad_auth_data_arr[..32].copy_from_slice(&[0xDEu8; 32]); // wrong RP hash
        bad_auth_data_arr[32] = FLAG_UP;
        bad_auth_data_arr[36] = 1;
        let bad_auth_data = Bytes::from_array(&env, &bad_auth_data_arr);
        let client_data_hash = BytesN::from_array(&env, &[0x42u8; 32]);
        let signature = BytesN::from_array(&env, &[0u8; 64]);

        let result = client.try_verify_ed25519_assertion(
            &user,
            &cred_hash,
            &bad_auth_data,
            &client_data_hash,
            &signature,
            &1u32,
        );
        assert!(matches!(result, Err(Ok(Error::RpIdMismatch))));
    }

    #[test]
    fn test_zk_nullifier_replay_prevented() {
        let env = Env::default();
        env.mock_all_auths();
        env.ledger().with_mut(|l| l.timestamp = 1000);
        let (client, admin) = setup_contract(&env);

        // Set a mock ZK verifier (non-existent address; ZK call would fail, but
        // nullifier replay is checked first).
        let fake_verifier = Address::generate(&env);
        client.set_zk_verifier(&admin, &fake_verifier);

        // Pre-mark a nullifier as used.
        let nullifier = BytesN::from_array(&env, &[0xABu8; 32]);
        env.as_contract(&client.address, || {
            env.storage()
                .persistent()
                .set(&DataKey::UsedNullifier(nullifier.clone()), &true);
        });

        let user = Address::generate(&env);
        client.issue_auth_challenge(&user);

        let cred_hash = BytesN::from_array(&env, &[0x01u8; 32]);
        let commitment = BytesN::from_array(&env, &[0x02u8; 32]);
        let proof = Bytes::from_array(&env, &[0u8; 64]);

        let result = client.try_verify_zk_assertion(
            &user,
            &cred_hash,
            &nullifier,
            &commitment,
            &proof,
            &1u32,
            &1u32,
        );
        assert!(matches!(result, Err(Ok(Error::NullifierAlreadyUsed))));
    }
}
