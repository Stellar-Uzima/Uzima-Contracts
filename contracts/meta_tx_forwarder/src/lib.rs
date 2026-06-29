#![no_std]

//! # Meta-Transaction Forwarder (ERC-2771 Compatible)
//!
//! This contract enables gasless transactions by allowing a relayer to submit
//! transactions on behalf of users. The forwarder verifies Ed25519 signatures
//! and manages per-user nonces to prevent replay attacks, then invokes the
//! target contract on the user's behalf via `env.invoke_contract`.
//!
//! ## Key Features
//! - ERC-2771-compatible execution flow (original sender is appended as the
//!   first positional argument of every forwarded invocation).
//! - Ed25519 signature verification of the `ForwardRequest` payload using
//!   `env.crypto().ed25519_verify`.
//! - Per-user, monotonically increasing nonces for replay protection.
//! - Deadline enforcement against stale / front-run requests.
//! - Pluggable, registered-active relayer set.
//! - Optional batch execution.
//!
//! ## Soroban-specific ERC-2771 mapping
//! Unlike the EVM "append 20-byte sender to calldata" variant, Soroban forwards
//! invoke_contract calls with structured `(Symbol, Vec<Val>)` argument lists.
//! This contract therefore prepends the original `from` `Address` as the first
//! positional argument so target contracts can extract it from their
//! `env.invoker()` lineage or, more reliably, accept it as `arg 0`.

pub mod erc2771_context;

use soroban_sdk::xdr::ToXdr;
use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype, symbol_short, Address, Bytes, BytesN, Env,
    IntoVal, Symbol, Val, Vec,
};

// ============================================================================
// Error Definitions
// ============================================================================

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    /// The provided Ed25519 signature failed cryptographic verification.
    InvalidSignature = 1,
    /// The request nonce does not match the user's current nonce.
    InvalidNonce = 2,
    /// The request deadline is in the past.
    RequestExpired = 3,
    /// The target contract invocation returned an error.
    ExecutionFailed = 4,
    /// The caller is not permitted to perform this action.
    Unauthorized = 5,
    /// The contract has already been initialized.
    AlreadyInitialized = 6,
    /// No owner has been set on the contract.
    OwnerNotSet = 7,
    /// Batch `requests` and `signatures` vectors have different lengths.
    BatchLengthMismatch = 8,
    /// No public key is registered for the request's `from` address.
    PubKeyNotRegistered = 9,
    /// The supplied `fee_percentage` exceeds the 100% maximum (10_000 bps).
    InvalidFeePercentage = 10,
}

impl core::fmt::Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        match self {
            Error::InvalidSignature => write!(f, "invalid signature"),
            Error::InvalidNonce => write!(f, "invalid nonce"),
            Error::RequestExpired => write!(f, "request expired"),
            Error::ExecutionFailed => write!(f, "execution failed"),
            Error::Unauthorized => write!(f, "unauthorized"),
            Error::AlreadyInitialized => write!(f, "already initialized"),
            Error::OwnerNotSet => write!(f, "owner not set"),
            Error::BatchLengthMismatch => write!(f, "batch length mismatch"),
            Error::PubKeyNotRegistered => write!(f, "pub key not registered"),
            Error::InvalidFeePercentage => write!(f, "invalid fee percentage"),
        }
    }
}

// ============================================================================
// Data Structures
// ============================================================================

/// Forward request structure containing all necessary data for a meta-transaction.
///
/// The signed payload is the deterministic XDR serialization of this struct
/// (produced via `request.to_xdr()`). Signers must produce the Ed25519 signature
/// over these exact bytes after application of the user's public-key.
///
/// `target_fn` identifies the Soroban contract function to invoke on `to`,
/// and `target_args` are the positional arguments to that function *excluding
/// the original sender* — the forwarder automatically prepends `from` as
/// the first positional argument so target contracts recognise the original
/// signer (the ERC-2771 pattern adapted to Soroban).
///
/// `data` is preserved for backwards compatibility and may carry arbitrary
/// opaque bytes (currently unused by the on-chain verifier).
#[derive(Clone)]
#[contracttype]
pub struct ForwardRequest {
    /// Address of the original signer (whose public key was registered).
    pub from: Address,
    /// Target contract address that will be invoked.
    pub to: Address,
    /// Value to transfer (currently informational; Soroban token payments are
    /// dispatched separately by the caller, so this should typically be 0).
    pub value: i128,
    /// Optional gas limit hint (informational; the Soroban host enforces its
    /// own CPU budget).
    pub gas: u32,
    /// Per-user nonce. Must equal the user's current on-chain nonce.
    pub nonce: u64,
    /// Unix timestamp deadline after which the request is no longer valid.
    pub deadline: u64,
    /// Target contract function symbol to invoke on `to`.
    pub target_fn: Symbol,
    /// Optional positional arguments for `target_fn`. The original `from`
    /// address is always prepended automatically.
    pub target_args: Vec<Val>,
}

/// Relayer configuration
#[derive(Clone)]
#[contracttype]
pub struct RelayerConfig {
    pub address: Address,
    pub is_active: bool,
    pub fee_percentage: u32, // Fee in basis points (e.g., 100 = 1%)
}

/// Storage keys
#[contracttype]
pub enum DataKey {
    Owner,
    /// Per-user monotonically increasing nonce (replay protection).
    Nonce(Address),
    /// Registered relayer configuration.
    Relayer(Address),
    /// This contract's own address (the trusted forwarder for ERC-2771).
    TrustedForwarder,
    /// Address that receives relay fees.
    FeeCollector,
    /// Minimum stake required to be a relayer (informational; not enforced).
    MinRelayerStake,
    /// Per-user Ed25519 public key (32 bytes). Required for sig verification.
    UserPubKey(Address),
}

/// Forwarding outcome returned by `execute_*`.
#[derive(Clone)]
#[contracttype]
pub struct ForwardOutcome {
    /// XDR-encoded `Val` returned by the target contract.
    pub result: Bytes,
    /// Nonce of the user *after* this execution (== old nonce + 1).
    pub new_nonce: u64,
    /// Ledger timestamp at execution time.
    pub executed_at: u64,
}

// ============================================================================
// Domain separator
// ============================================================================

/// Domain separator prefix prepended to every signed payload.
///
/// **Test / SDK consumers MUST use the same byte sequence when constructing
/// signatures.** A change to this constant is a hard fork of the
/// signature format — old signed requests will fail verification.
///
/// The chosen layout (`"UZM-MTX-v1"` + 6 null padding bytes) deliberately
/// matches an off-chain EIP-712-style domain and binds signatures to the
/// `MetaTxForwarder` v1. Whenever the contract is upgraded and the
/// signature format changes, bump the trailing ASCII character.
pub const DOMAIN_PREFIX: [u8; 16] = [
    b'U', b'Z', b'M', b'-', b'M', b'T', b'X', b'-', b'v', b'1', 0, 0, 0, 0, 0, 0,
];

// ============================================================================
// Contract Implementation
// ============================================================================

#[contract]
pub struct MetaTxForwarder;

#[contractimpl]
impl MetaTxForwarder {
    // ========================================================================
    // Initialization
    // ========================================================================

    /// Initialize the forwarder contract.
    ///
    /// # Arguments
    /// * `owner` - Contract owner address
    /// * `fee_collector` - Address to receive relay fees (informational)
    /// * `min_relayer_stake` - Minimum stake required for relayers (informational)
    pub fn initialize(
        env: Env,
        owner: Address,
        fee_collector: Address,
        min_relayer_stake: i128,
    ) -> Result<(), Error> {
        governance_commons::try_init_guard(&env).map_err(|_| Error::AlreadyInitialized)?;

        owner.require_auth();

        env.storage().instance().set(&DataKey::Owner, &owner);
        env.storage()
            .instance()
            .set(&DataKey::FeeCollector, &fee_collector);
        env.storage()
            .instance()
            .set(&DataKey::MinRelayerStake, &min_relayer_stake);
        env.storage()
            .instance()
            .set(&DataKey::TrustedForwarder, &env.current_contract_address());

        env.events().publish(
            (symbol_short!("init"),),
            (owner.clone(), fee_collector.clone(), min_relayer_stake),
        );

        Ok(())
    }

    // ========================================================================
    // User -> Public Key registration
    // ========================================================================

    /// Register an Ed25519 public key (32 bytes) for a user.
    ///
    /// The public key is required before the user can sign `ForwardRequest`s.
    /// One-time registration — re-registering overwrites the previous key.
    pub fn register_user_pub_key(
        env: Env,
        user: Address,
        pub_key: BytesN<32>,
    ) -> Result<(), Error> {
        user.require_auth();
        env.storage()
            .persistent()
            .set(&DataKey::UserPubKey(user.clone()), &pub_key);
        env.events().publish(
            (symbol_short!("reg_key"),),
            (user, Bytes::from_slice(&env, &pub_key.to_array())),
        );
        Ok(())
    }

    /// Returns the registered Ed25519 public key for `user`, if any.
    pub fn get_user_pub_key(env: Env, user: Address) -> Option<BytesN<32>> {
        env.storage().persistent().get(&DataKey::UserPubKey(user))
    }

    // ========================================================================
    // Core Forwarding Functions
    // ========================================================================

    /// Execute a meta-transaction on behalf of a user.
    pub fn execute(
        env: Env,
        relayer: Address,
        request: ForwardRequest,
        signature: BytesN<64>,
    ) -> Result<Bytes, Error> {
        relayer.require_auth();
        Self::require_active_relayer(&env, &relayer)?;
        Self::execute_one(&env, request, signature, &relayer)
    }

    /// Execute multiple meta-transactions in a batch.
    ///
    /// All requests must be valid individually; on first failure, the
    /// already-completed requests have incremented their nonces and
    /// committed state, and the rest are not executed.
    pub fn execute_batch(
        env: Env,
        relayer: Address,
        requests: Vec<ForwardRequest>,
        signatures: Vec<BytesN<64>>,
    ) -> Result<Vec<Bytes>, Error> {
        relayer.require_auth();
        Self::require_active_relayer(&env, &relayer)?;

        if requests.len() != signatures.len() {
            return Err(Error::BatchLengthMismatch);
        }

        let mut results = Vec::new(&env);
        for i in 0..requests.len() {
            let request = requests.get(i).ok_or(Error::InvalidSignature)?;
            let signature = signatures.get(i).ok_or(Error::InvalidSignature)?;
            results.push_back(Self::execute_one(&env, request, signature, &relayer)?);
        }
        Ok(results)
    }

    // ========================================================================
    // Relayer Management
    // ========================================================================

    /// Register a new relayer (owner-only).
    pub fn register_relayer(
        env: Env,
        owner: Address,
        relayer: Address,
        fee_percentage: u32,
    ) -> Result<(), Error> {
        owner.require_auth();
        Self::require_owner(&env, &owner)?;

        if fee_percentage > 10_000 {
            return Err(Error::InvalidFeePercentage);
        }

        let config = RelayerConfig {
            address: relayer.clone(),
            is_active: true,
            fee_percentage,
        };
        env.storage()
            .instance()
            .set(&DataKey::Relayer(relayer.clone()), &config);
        env.events()
            .publish((symbol_short!("reg_relay"),), (relayer, fee_percentage));
        Ok(())
    }

    /// Deactivate a relayer (owner-only).
    pub fn deactivate_relayer(env: Env, owner: Address, relayer: Address) -> Result<(), Error> {
        owner.require_auth();
        Self::require_owner(&env, &owner)?;

        let mut config: RelayerConfig = env
            .storage()
            .instance()
            .get(&DataKey::Relayer(relayer.clone()))
            .unwrap_or(RelayerConfig {
                address: relayer.clone(),
                is_active: false,
                fee_percentage: 0,
            });
        config.is_active = false;
        env.storage()
            .instance()
            .set(&DataKey::Relayer(relayer.clone()), &config);
        env.events().publish((symbol_short!("deact_rel"),), relayer);
        Ok(())
    }

    // ========================================================================
    // View Functions
    // ========================================================================

    /// Get the current nonce for a user.
    pub fn get_nonce(env: Env, user: Address) -> u64 {
        env.storage()
            .persistent()
            .get(&DataKey::Nonce(user))
            .unwrap_or(0)
    }

    /// Check if an address is an active relayer.
    pub fn is_relayer(env: Env, relayer: Address) -> bool {
        let config: Option<RelayerConfig> =
            env.storage().instance().get(&DataKey::Relayer(relayer));
        matches!(config, Some(cfg) if cfg.is_active)
    }

    /// Get relayer configuration.
    pub fn get_relayer_config(env: Env, relayer: Address) -> Option<RelayerConfig> {
        env.storage().instance().get(&DataKey::Relayer(relayer))
    }

    /// Get the trusted forwarder address (this contract).
    pub fn get_trusted_forwarder(env: Env) -> Address {
        env.storage()
            .instance()
            .get(&DataKey::TrustedForwarder)
            .unwrap_or_else(|| env.current_contract_address())
    }

    // ========================================================================
    // Internal helpers
    // ========================================================================

    /// Single-request execution: deadline + nonce + signature check, then
    /// forward. Relayer authorization must have already been performed by
    /// the caller.
    fn execute_one(
        env: &Env,
        request: ForwardRequest,
        signature: BytesN<64>,
        relayer: &Address,
    ) -> Result<Bytes, Error> {
        let now = env.ledger().timestamp();
        if now > request.deadline {
            return Err(Error::RequestExpired);
        }
        Self::verify_nonce(env, &request.from, request.nonce)?;
        Self::verify_signature(env, &request, &signature)?;
        let result = Self::forward_call(env, &request)?;
        Self::increment_nonce(env, &request.from);
        env.events().publish(
            (symbol_short!("fwd"),),
            (
                relayer.clone(),
                request.from.clone(),
                request.to.clone(),
                request.nonce,
            ),
        );
        Ok(result)
    }

    /// Verify user nonce without incrementing.
    #[must_use]
    fn verify_nonce(env: &Env, user: &Address, expected_nonce: u64) -> Result<(), Error> {
        let current: u64 = env
            .storage()
            .persistent()
            .get(&DataKey::Nonce(user.clone()))
            .unwrap_or(0);
        if current != expected_nonce {
            return Err(Error::InvalidNonce);
        }
        Ok(())
    }

    /// Increment user nonce.
    fn increment_nonce(env: &Env, user: &Address) {
        let current: u64 = env
            .storage()
            .persistent()
            .get(&DataKey::Nonce(user.clone()))
            .unwrap_or(0);
        env.storage()
            .persistent()
            .set(&DataKey::Nonce(user.clone()), &(current.saturating_add(1)));
    }

    /// Verify Ed25519 signature over the typed payload of `request`.
    ///
    /// The signed message is `DOMAIN_PREFIX || forwarder_address || request.to_xdr()`.
    /// The user's 32-byte Ed25519 public key is looked up from storage.
    ///
    /// Traps (host panic) on signature verification failure by design — Soroban
    /// `ed25519_verify` is fail-fast and there is no soft return path;
    /// `Error::PubKeyNotRegistered` is surfaced up-front when the user has
    /// not registered their key.
    fn verify_signature(
        env: &Env,
        request: &ForwardRequest,
        signature: &BytesN<64>,
    ) -> Result<(), Error> {
        let pub_key: BytesN<32> = env
            .storage()
            .persistent()
            .get(&DataKey::UserPubKey(request.from.clone()))
            .ok_or(Error::PubKeyNotRegistered)?;

        let mut message = Bytes::new(env);
        // Domain prefix (16 bytes) — binds signatures to MetaTxForwarder v1.
        message.append(&Bytes::from_slice(env, &DOMAIN_PREFIX));
        // Forwarder address — prevents cross-forwarder replay.
        let forwarder_addr = Self::get_trusted_forwarder(env.clone());
        message.append(&forwarder_addr.to_xdr(env));
        // Request body — full deterministic XDR of the typed ForwardRequest.
        // The soroban-sdk 21.7.7 contracttype-derive impl takes `self` by
        // value, so we clone the borrowed request before serializing it.
        let req_xdr: soroban_sdk::Bytes = request.clone().to_xdr(env);
        message.append(&req_xdr);

        // Verify Ed25519 signature. Traps on cryptographic failure.
        env.crypto().ed25519_verify(&pub_key, &message, signature);
        Ok(())
    }

    /// Forward the call to the target contract.
    ///
    /// Builds `Vec<Val>` of `[from.into_val(), ...request.target_args]` and
    /// invokes `request.to.request.target_fn(...)`. Returns the XDR-encoded
    /// return value of the target.
    #[must_use]
    fn forward_call(env: &Env, request: &ForwardRequest) -> Result<Bytes, Error> {
        // Build the positional arg list passed to the target. The forwarder
        // always prepends the original `from` so target contracts have
        // first-argument access to the original signer.
        let mut args: Vec<Val> = Vec::new(env);
        args.push_back(request.from.clone().into_val(env));
        for arg in request.target_args.iter() {
            args.push_back(arg);
        }

        let result: Val = env.invoke_contract(&request.to, &request.target_fn, args);
        // The auto-derived `ToXdr` for `Val` consumes self by value in 21.x
        // (returns `Bytes`). `result` is freshly owned, so no clone needed.
        Ok(result.to_xdr(env))
    }

    /// Require that the caller is the contract owner.
    #[must_use]
    fn require_owner(env: &Env, caller: &Address) -> Result<(), Error> {
        let stored: Address = env
            .storage()
            .instance()
            .get(&DataKey::Owner)
            .ok_or(Error::OwnerNotSet)?;
        if *caller != stored {
            return Err(Error::Unauthorized);
        }
        Ok(())
    }

    /// Require that the relayer is registered and active.
    #[must_use]
    fn require_active_relayer(env: &Env, relayer: &Address) -> Result<(), Error> {
        let config: Option<RelayerConfig> = env
            .storage()
            .instance()
            .get(&DataKey::Relayer(relayer.clone()));
        match config {
            Some(cfg) if cfg.is_active => Ok(()),
            _ => Err(Error::Unauthorized),
        }
    }

    // ========================================================================
    // Re-export helpers
    // ========================================================================

    /// Returns the canonical domain separator string used in the signed
    /// message. Exposed for off-chain clients that need to reproduce the
    /// exact prefix when constructing signatures.
    pub fn domain_separator(env: Env) -> Bytes {
        Bytes::from_slice(&env, &DOMAIN_PREFIX)
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod test;
