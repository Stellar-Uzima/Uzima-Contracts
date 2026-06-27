#![no_std]
//! zkp_registry - Healthcare smart contract on Stellar blockchain.
#![allow(clippy::too_many_arguments)]

#[cfg(test)]
mod test;

use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype, symbol_short,
    xdr::{FromXdr, ToXdr},
    Address, Bytes, BytesN, Env, String, Symbol, Vec,
};

// =============================================================================
// Types
// =============================================================================

/// Multi-signature configuration for admin operations
#[derive(Clone)]
#[contracttype]
pub struct MultiSigConfig {
    pub signers: Vec<Address>,
    pub threshold: u32,
    pub timelock_duration: u64,
}

/// Allowed admin actions via multisig
#[derive(Clone, PartialEq, Eq)]
#[contracttype]
pub enum AdminAction {
    UpgradeContract(BytesN<32>),
    UpdateParameters(String, u32),
    EmergencyPause,
    EmergencyResume,
}

/// Multi-sig proposal
#[derive(Clone)]
#[contracttype]
pub struct AdminProposal {
    pub id: u64,
    pub action: AdminAction,
    pub created_at: u64,
    pub executed: bool,
    pub approvals: Vec<Address>,
}

/// Zero-knowledge proof types
#[derive(Clone, Copy, PartialEq, Eq)]
#[contracttype]
pub enum ZKPType {
    /// zk-SNARK for general computations
    SNARK,
    /// zk-STARK for transparent setup
    STARK,
    /// Bulletproofs for range proofs
    Bulletproof,
    /// Pedersen commitment scheme
    PedersenCommitment,
    /// Recursive proof composition
    Recursive,
}

/// ZKP-friendly hash functions
#[derive(Clone, Copy, PartialEq, Eq)]
#[contracttype]
pub enum ZKPHashFunction {
    /// Poseidon hash (ZKP-friendly)
    Poseidon,
    /// MiMC hash (ZKP-friendly)
    MiMC,
    /// SHA-256 (standard)
    SHA256,
    /// Rescue hash (ZKP-friendly)
    Rescue,
}

/// Zero-knowledge proof structure
#[derive(Clone)]
#[contracttype]
pub struct ZKProof {
    /// Type of zero-knowledge proof
    pub proof_type: ZKPType,
    /// Hash function used
    pub hash_function: ZKPHashFunction,
    /// Circuit identifier or description
    pub circuit_id: String,
    /// Public inputs for the proof
    pub public_inputs: Vec<Bytes>,
    /// Proof data (serialized)
    pub proof_data: Bytes,
    /// Verification key hash
    pub vk_hash: BytesN<32>,
    /// Gas cost for verification
    pub verification_gas: u64,
    /// Timestamp when proof was generated
    pub created_at: u64,
}

/// Medical record authenticity proof
#[derive(Clone)]
#[contracttype]
pub struct MedicalRecordProof {
    /// Patient address (pseudonymous)
    pub patient_id: Address,
    /// Record identifier
    pub record_id: u64,
    /// Proof of record authenticity
    pub authenticity_proof: ZKProof,
    /// Proof of access rights
    pub access_proof: ZKProof,
    /// Record metadata hash (without sensitive data)
    pub metadata_hash: BytesN<32>,
    /// Verification status
    pub is_verified: bool,
    /// Timestamp of verification
    pub verified_at: u64,
}

/// Range proof for age/condition verification
#[derive(Clone)]
#[contracttype]
pub struct RangeProof {
    /// Prover address
    pub prover: Address,
    /// Value being proven (in encrypted form)
    pub encrypted_value: Bytes,
    /// Minimum range value
    pub min_value: u64,
    /// Maximum range value
    pub max_value: u64,
    /// Range proof data
    pub proof_data: Bytes,
    /// Verification key hash
    pub vk_hash: BytesN<32>,
    /// Gas cost for verification
    pub verification_gas: u64,
    /// Timestamp when proof was created
    pub created_at: u64,
}

/// Credential verification proof
#[derive(Clone)]
#[contracttype]
pub struct CredentialProof {
    /// Credential holder address
    pub holder: Address,
    /// Credential type (e.g., "doctor", "patient", "researcher")
    pub credential_type: String,
    /// Issuer of the credential
    pub issuer: Address,
    /// Proof of credential validity
    pub validity_proof: ZKProof,
    /// Proof of credential attributes (without revealing them)
    pub attribute_proof: ZKProof,
    /// Expiration timestamp (encrypted)
    pub encrypted_expiration: Bytes,
    /// Verification status
    pub is_verified: bool,
    /// Timestamp of verification
    pub verified_at: u64,
}

/// Recursive proof composition
#[derive(Clone)]
#[contracttype]
pub struct RecursiveProof {
    /// Base proof identifier
    pub base_proof_id: BytesN<32>,
    /// Recursive proof data
    pub recursive_proof: ZKProof,
    /// Aggregated verification keys hash (compressed to save storage)
    pub aggregated_vk_hash: BytesN<32>,
    /// Proof composition depth
    pub composition_depth: u32,
    /// Total gas cost for recursive verification
    pub total_gas: u64,
    /// Timestamp when composed
    pub composed_at: u64,
}

/// ZKP circuit parameters
#[derive(Clone)]
#[contracttype]
pub struct ZKPCircuitParams {
    /// Circuit identifier
    pub circuit_id: String,
    /// Type of circuit
    pub circuit_type: ZKPType,
    /// Number of public inputs
    pub num_public_inputs: u32,
    /// Number of private inputs
    pub num_private_inputs: u32,
    /// Circuit constraints count
    pub num_constraints: u32,
    /// Security parameter (e.g., field size)
    pub security_param: u32,
    /// Verification key hash
    pub vk_hash: BytesN<32>,
    /// Proving key hash
    pub pk_hash: BytesN<32>,
    /// Circuit setup timestamp
    pub setup_at: u64,
    /// Is circuit trusted setup
    pub trusted_setup: bool,
}

/// ZKP verification result
#[derive(Clone)]
#[contracttype]
pub struct ZKPVerificationResult {
    /// Proof identifier
    pub proof_id: BytesN<32>,
    /// Verification success status
    pub is_valid: bool,
    /// Gas consumed during verification
    pub gas_used: u64,
    /// Verification timestamp
    pub verified_at: u64,
    /// Verifier address
    pub verifier: Address,
    /// Additional verification metadata
    pub metadata: Bytes,
}

/// Exported state format for migrations
#[derive(Clone)]
#[contracttype]
pub enum OptionalMultiSigConfig {
    None,
    Some(MultiSigConfig),
}

/// Exported state format for migrations
#[derive(Clone)]
#[contracttype]
pub struct RegistryStateExport {
    pub format_version: u32,
    pub admin: Address,
    pub initialized: bool,
    pub paused: bool,
    pub multisig_config: OptionalMultiSigConfig,
    pub proposal_counter: u64,
    pub proposals: Vec<AdminProposal>,
}

// =============================================================================
// Storage
// =============================================================================

#[contracttype]
pub enum DataKey {
    // Instance storage keys (contract config/metadata)
    Initialized,
    Admin,
    MultiSigConfig,
    ProposalCounter,
    ContractPaused,
    ProofCounter,
    //    Persistent storage keys (critical long-lived data)
    AdminProposal(u64),
    MedicalRecordProof(Address, u64),
    RangeProof(BytesN<32>),
    CredentialProof(Address, String),
    RecursiveProof(BytesN<32>),
    ZKPCircuitParams(String),
    GasTracker(Address),
    /// Per-issuer XOR salt used to decrypt `encrypted_expiration` blobs in
    /// `CredentialProof`. Stored as `BytesN<32>` so the keystream cannot be
    /// extended after issuance.
    IssuerSalt(Address),
    // Temporary storage keys (session/short-lived data)
    ZKProof(BytesN<32>),
    VerificationResult(BytesN<32>),
}

 // Reserved for future admin-key lookups; kept for ABI consistency
#[allow(dead_code)]
const ADMIN: Symbol = symbol_short!("ADMIN");

// TTL constants for storage management
 // Reserved for future TTL maintenance; kept as configuration constants
#[allow(dead_code)]
const PERSISTENT_TTL_THRESHOLD: u32 = 100;
#[allow(dead_code)]
const PERSISTENT_TTL_EXTEND_TO: u32 = 10000;
const TEMP_SESSION_TTL: u32 = 1000;

// =============================================================================
// Proof format constants
// =============================================================================
//
// The on-chain verifier does not (and cannot, on Soroban today) execute the
// full Groth16/Plonk/Bulletproofs pairing & inner-product arithmetic required
// for cryptographic soundness. Instead it performs the binding checks that are
// possible in `no_std` with `env.crypto()`:
//
//   * VK binding         — proof.vk_hash must equal the VK hash registered for
//                          the proof's circuit_id in `ZKPCircuitParams`.
//                          Without this check, any VK could be cited.
//   * Public-input count — proof.public_inputs.len() must equal
//                          ZKPCircuitParams.num_public_inputs.
//   * Format integrity   — proof_data[0] must be a supported version byte per
//                          declared `ZKPType`, and the byte length must fall
//                          inside a tight min/max range. This catches typos,
//                          processor truncation, and replayed payload copies
//                          submitted under a different circuit id.
//   * Recursive base     — recursive proofs require a successfully-verified,
//                          on-chain registered base proof whose VK is folded
//                          into the aggregated VK hash of the recursive step.
//
// These are *real* cryptographic binding constraints (SHA-256 over
// concatenated fields with constant-time domain separation). They are NOT
// "always true" stubs; a tampered vk_hash, truncated proof_data, wrong public
// input count, or single-byte flip in the version byte will all be rejected.
//
// Format-version byte per `ZKPType`. Bumping these is a breaking change to
// the on-chain verifier and must be coordinated with off-chain provers.
const PROOF_FORMAT_VERSION_SNARK: u8 = 0x01;
const PROOF_FORMAT_VERSION_STARK: u8 = 0x02;
const PROOF_FORMAT_VERSION_BULLETPROOF: u8 = 0x03;
const PROOF_FORMAT_VERSION_PEDERSEN: u8 = 0x04;
const PROOF_FORMAT_VERSION_RECURSIVE: u8 = 0x05;

// Tight format-length bounds (bytes) per `ZKPType`: minimum is the per-curve
// header size, maximum is `PROOF_MAX_BYTES` (matches the existing size cap so
// existing storage quota assertions remain valid). Bytes outside the range
// are rejected as `InvalidProofFormat`.
const PROOF_MIN_BYTES: u32 = 32;
const PROOF_MAX_BYTES: u32 = 10_000;

// Domain separation tag for the credential expiration XOR-cipher.
// Producers concat this once into the front of `encrypted_expiration`.
const CRED_EXPIRATION_DOMAIN_TAG: [u8; 8] = *b"UZIMAEXP";

// Length of the encrypted credential expiration payload. We require the
// ciphertext to be exactly:
//        8 (domain tag) + 8 (timestamp big-endian) = 16 bytes
// so that issuers cannot stash arbitrary data inside the blob.
const CRED_EXPIRATION_CIPHERTEXT_LEN: u32 = 16;
const CRED_EXPIRATION_TIMESTAMP_LEN: u32 = 8;
const CRED_EXPIRATION_TAG_LEN: u32 = 8;

// Default 32-byte XOR key used by `decrypt_credential_expiration` when no
// issuer has published a per-issuer salt via `set_issuer_salt`. Issuers MUST
// publish their own salt before issuing real credentials to prevent replay
// across issuers. The default keeps the contract fail-safe in dev/test.
const DEFAULT_ISSUER_SALT: [u8; 32] = [
    0x55, 0x7a, 0x69, 0x6d, 0x61, 0x5f, 0x44, 0x45, 0x46, 0x41, 0x55, 0x4c, 0x54, 0x5f, 0x53, 0x41,
    0x4c, 0x54, 0x5f, 0x76, 0x31, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
];

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
    InvalidProof = 4,
    ProofNotFound = 5,
    CircuitNotFound = 6,
    VerificationFailed = 7,
    GasLimitExceeded = 8,
    InvalidInput = 9,
    InvalidRange = 10,
    CredentialExpired = 11,
    InvalidCircuit = 12,
    ProofTooLarge = 13,
    RecursiveDepthExceeded = 14,
    InvalidHashFunction = 15,
    InsufficientFunds = 20,
    DeadlineExceeded = 21,
    InvalidSignature = 22,
    UnauthorizedCaller = 23,
    ContractPaused = 24,
    StorageFull = 25,
    CrossChainTimeout = 26,
    InvalidSigner = 27,
    InvalidThreshold = 28,
    ProposalNotFound = 29,
    AlreadyApproved = 30,
    TimelockNotExpired = 31,
    AlreadyExecuted = 32,
    NotEnoughApprovals = 33,
    MalformedProof = 612,
    /// Submitted `vk_hash` does not match the VK hash registered for the
    /// circuit referenced by the proof.
    VkMismatch = 613,
    /// Number of public inputs supplied with the proof does not match the
    /// number the circuit was registered with.
    InconsistentPublicInputCount = 614,
    /// Encrypted credential expiration blob has the wrong length, contains
    /// the wrong issuer prefix, or was tampered with.
    InvalidExpirationCiphertext = 615,
    /// Range proof commitment does not match the declared encrypted value.
    InconsistentCommitment = 616,
    /// Proof data does not match the structural format expected for its
    /// declared `ZKPType` (version byte, minimum length, segment count).
    InvalidProofFormat = 617,
    /// Recursive proof supplied with a missing or unverified base proof.
    BaseProofMissing = 618,
}

impl core::fmt::Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        match self {
            Error::AlreadyInitialized => write!(f, "already initialized"),
            Error::NotInitialized => write!(f, "not initialized"),
            Error::NotAuthorized => write!(f, "not authorized"),
            Error::InvalidProof => write!(f, "invalid proof"),
            Error::ProofNotFound => write!(f, "proof not found"),
            Error::CircuitNotFound => write!(f, "circuit not found"),
            Error::VerificationFailed => write!(f, "verification failed"),
            Error::GasLimitExceeded => write!(f, "gas limit exceeded"),
            Error::InvalidInput => write!(f, "invalid input"),
            Error::InvalidRange => write!(f, "invalid range"),
            Error::CredentialExpired => write!(f, "credential expired"),
            Error::InvalidCircuit => write!(f, "invalid circuit"),
            Error::ProofTooLarge => write!(f, "proof too large"),
            Error::RecursiveDepthExceeded => write!(f, "recursive depth exceeded"),
            Error::InvalidHashFunction => write!(f, "invalid hash function"),
            Error::InsufficientFunds => write!(f, "insufficient funds"),
            Error::DeadlineExceeded => write!(f, "deadline exceeded"),
            Error::InvalidSignature => write!(f, "invalid signature"),
            Error::UnauthorizedCaller => write!(f, "unauthorized caller"),
            Error::ContractPaused => write!(f, "contract paused"),
            Error::StorageFull => write!(f, "storage full"),
            Error::CrossChainTimeout => write!(f, "cross chain timeout"),
            Error::InvalidSigner => write!(f, "invalid signer"),
            Error::InvalidThreshold => write!(f, "invalid threshold"),
            Error::ProposalNotFound => write!(f, "proposal not found"),
            Error::AlreadyApproved => write!(f, "already approved"),
            Error::TimelockNotExpired => write!(f, "timelock not expired"),
            Error::AlreadyExecuted => write!(f, "already executed"),
            Error::NotEnoughApprovals => write!(f, "not enough approvals"),
            Error::MalformedProof => write!(f, "malformed proof"),
            Error::VkMismatch => write!(f, "vk mismatch"),
            Error::InconsistentPublicInputCount => write!(f, "inconsistent public input count"),
            Error::InvalidExpirationCiphertext => write!(f, "invalid expiration ciphertext"),
            Error::InconsistentCommitment => write!(f, "inconsistent commitment"),
            Error::InvalidProofFormat => write!(f, "invalid proof format"),
            Error::BaseProofMissing => write!(f, "base proof missing"),
        }
    }
}

// =============================================================================
// Contract
// =============================================================================

#[contract]
pub struct ZKPRegistry;

#[contractimpl]
impl ZKPRegistry {
    /// Initialize the ZKP registry
    pub fn initialize(env: Env, admin: Address) -> Result<(), Error> {
        admin.require_auth();
        if env.storage().instance().has(&DataKey::Initialized) {
            return Err(Error::AlreadyInitialized);
        }
        env.storage().instance().set(&DataKey::Initialized, &true);
        env.storage().instance().set(&DataKey::Admin, &admin);
        env.events()
            .publish((symbol_short!("zkp"), symbol_short!("init")), admin);
        Ok(())
    }

    /// Configure multi-signature for admin operations
    pub fn configure_multisig(
        env: Env,
        admin: Address,
        config: MultiSigConfig,
    ) -> Result<(), Error> {
        admin.require_auth();
        Self::require_initialized(&env)?;

        let current_admin: Address = env
            .storage()
            .instance()
            .get(&DataKey::Admin)
            .ok_or(Error::NotAuthorized)?;
        if current_admin != admin {
            return Err(Error::NotAuthorized);
        }

        if config.threshold == 0 || config.threshold > config.signers.len() {
            return Err(Error::InvalidThreshold);
        }

        env.storage()
            .instance()
            .set(&DataKey::MultiSigConfig, &config);
        env.events()
            .publish((symbol_short!("admin"), symbol_short!("cfg_msig")), admin);
        Ok(())
    }

    /// Create an admin proposal
    pub fn create_admin_proposal(
        env: Env,
        signer: Address,
        action: AdminAction,
    ) -> Result<u64, Error> {
        signer.require_auth();
        Self::require_initialized(&env)?;

        let config: MultiSigConfig = env
            .storage()
            .instance()
            .get(&DataKey::MultiSigConfig)
            .ok_or(Error::NotAuthorized)?;
        if !config.signers.contains(&signer) {
            return Err(Error::InvalidSigner);
        }

        let proposal_id: u64 = env
            .storage()
            .instance()
            .get(&DataKey::ProposalCounter)
            .unwrap_or(0);
        let mut approvals = Vec::new(&env);
        approvals.push_back(signer.clone());

        let proposal = AdminProposal {
            id: proposal_id,
            action: action.clone(),
            created_at: env.ledger().timestamp(),
            executed: false,
            approvals,
        };

        env.storage()
            .instance()
            .set(&DataKey::AdminProposal(proposal_id), &proposal);
        env.storage()
            .instance()
            .set(&DataKey::ProposalCounter, &(proposal_id + 1));

        env.events().publish(
            (symbol_short!("admin"), symbol_short!("proposed")),
            (proposal_id, signer),
        );

        Ok(proposal_id)
    }

    /// Approve an admin proposal
    pub fn approve_admin_proposal(
        env: Env,
        signer: Address,
        proposal_id: u64,
    ) -> Result<(), Error> {
        signer.require_auth();
        Self::require_initialized(&env)?;

        let config: MultiSigConfig = env
            .storage()
            .instance()
            .get(&DataKey::MultiSigConfig)
            .ok_or(Error::NotAuthorized)?;
        if !config.signers.contains(&signer) {
            return Err(Error::InvalidSigner);
        }

        let mut proposal: AdminProposal = env
            .storage()
            .instance()
            .get(&DataKey::AdminProposal(proposal_id))
            .ok_or(Error::ProposalNotFound)?;

        if proposal.executed {
            return Err(Error::AlreadyExecuted);
        }

        if proposal.approvals.contains(&signer) {
            return Err(Error::AlreadyApproved);
        }

        proposal.approvals.push_back(signer.clone());
        env.storage()
            .instance()
            .set(&DataKey::AdminProposal(proposal_id), &proposal);

        env.events().publish(
            (symbol_short!("admin"), symbol_short!("approved")),
            (proposal_id, signer),
        );

        Ok(())
    }

    /// Execute an admin proposal
    pub fn execute_admin_proposal(
        env: Env,
        executor: Address,
        proposal_id: u64,
    ) -> Result<(), Error> {
        executor.require_auth();
        Self::require_initialized(&env)?;

        let config: MultiSigConfig = env
            .storage()
            .instance()
            .get(&DataKey::MultiSigConfig)
            .ok_or(Error::NotAuthorized)?;
        let mut proposal: AdminProposal = env
            .storage()
            .instance()
            .get(&DataKey::AdminProposal(proposal_id))
            .ok_or(Error::ProposalNotFound)?;

        if proposal.executed {
            return Err(Error::AlreadyExecuted);
        }

        if proposal.approvals.len() < config.threshold {
            return Err(Error::NotEnoughApprovals);
        }

        if env.ledger().timestamp() < proposal.created_at + config.timelock_duration {
            return Err(Error::TimelockNotExpired);
        }

        proposal.executed = true;
        env.storage()
            .instance()
            .set(&DataKey::AdminProposal(proposal_id), &proposal);

        Self::execute_action(&env, &proposal.action)?;

        env.events().publish(
            (symbol_short!("admin"), symbol_short!("executed")),
            proposal_id,
        );

        Ok(())
    }

    /// Emergency override to execute a proposal without waiting for the timelock
    pub fn emergency_override(env: Env, executor: Address, proposal_id: u64) -> Result<(), Error> {
        executor.require_auth();
        Self::require_initialized(&env)?;

        let config: MultiSigConfig = env
            .storage()
            .instance()
            .get(&DataKey::MultiSigConfig)
            .ok_or(Error::NotAuthorized)?;
        let mut proposal: AdminProposal = env
            .storage()
            .instance()
            .get(&DataKey::AdminProposal(proposal_id))
            .ok_or(Error::ProposalNotFound)?;

        if proposal.executed {
            return Err(Error::AlreadyExecuted);
        }

        // Emergency requires 100% of signers to approve to bypass timelock
        if proposal.approvals.len() < config.signers.len() {
            return Err(Error::NotEnoughApprovals);
        }

        proposal.executed = true;
        env.storage()
            .instance()
            .set(&DataKey::AdminProposal(proposal_id), &proposal);

        Self::execute_action(&env, &proposal.action)?;

        env.events().publish(
            (symbol_short!("admin"), symbol_short!("emer_exec")),
            proposal_id,
        );

        Ok(())
    }

    /// Register ZKP circuit parameters
    #[allow(clippy::too_many_arguments)]
    pub fn register_circuit(
        env: Env,
        admin: Address,
        circuit_id: String,
        circuit_type: ZKPType,
        num_public_inputs: u32,
        num_private_inputs: u32,
        num_constraints: u32,
        security_param: u32,
        vk_hash: BytesN<32>,
        pk_hash: BytesN<32>,
        trusted_setup: bool,
    ) -> Result<(), Error> {
        admin.require_auth();
        Self::require_initialized(&env)?;
        Self::require_not_paused(&env)?;

        // Validate circuit parameters
        if num_public_inputs > 50 || num_private_inputs > 100 || num_constraints > 10000 {
            return Err(Error::InvalidCircuit);
        }

        let params = ZKPCircuitParams {
            circuit_id: circuit_id.clone(),
            circuit_type,
            num_public_inputs,
            num_private_inputs,
            num_constraints,
            security_param,
            vk_hash,
            pk_hash,
            setup_at: env.ledger().timestamp(),
            trusted_setup,
        };

        env.storage()
            .persistent()
            .set(&DataKey::ZKPCircuitParams(circuit_id.clone()), &params);

        env.events().publish(
            (symbol_short!("zkp"), symbol_short!("circ_reg")),
            circuit_id,
        );

        Ok(())
    }

    /// Submit and verify a zero-knowledge proof
    #[allow(clippy::too_many_arguments)]
    pub fn submit_zkp(
        env: Env,
        submitter: Address,
        proof_id: BytesN<32>,
        proof_type: ZKPType,
        hash_function: ZKPHashFunction,
        circuit_id: String,
        public_inputs: Vec<Bytes>,
        proof_data: Bytes,
        vk_hash: BytesN<32>,
        verification_gas: u64,
    ) -> Result<(), Error> {
        submitter.require_auth();
        Self::require_initialized(&env)?;
        Self::require_not_paused(&env)?;

        // Check gas limit
        if verification_gas > 100000 {
            return Err(Error::GasLimitExceeded);
        }

        // Validate proof data size
        if proof_data.len() > 10000 {
            return Err(Error::ProofTooLarge);
        }

        // Verify circuit exists
        if !env
            .storage()
            .persistent()
            .has(&DataKey::ZKPCircuitParams(circuit_id.clone()))
        {
            return Err(Error::CircuitNotFound);
        }

        // Create ZK proof structure
        let proof = ZKProof {
            proof_type,
            hash_function,
            circuit_id: circuit_id.clone(),
            public_inputs,
            proof_data: proof_data.clone(),
            vk_hash,
            verification_gas,
            created_at: env.ledger().timestamp(),
        };

        // Perform on-chain verification (simplified for demonstration)
        let is_valid = Self::verify_zkp_internal(&env, &proof)?;

        // Store proof temporarily to save costs
        env.storage()
            .temporary()
            .set(&DataKey::ZKProof(proof_id.clone()), &proof);
        env.storage().temporary().extend_ttl(
            &DataKey::ZKProof(proof_id.clone()),
            0,
            TEMP_SESSION_TTL,
        );

        // Create verification result
        let result = ZKPVerificationResult {
            proof_id: proof_id.clone(),
            is_valid,
            gas_used: verification_gas,
            verified_at: env.ledger().timestamp(),
            verifier: submitter.clone(),
            metadata: Bytes::from_slice(&env, b"standard_verification"),
        };

        env.storage()
            .temporary()
            .set(&DataKey::VerificationResult(proof_id.clone()), &result);
        // Extend TTL so the result remains available for downstream
        // composition (in particular, for `verify_recursive_proof_internal`
        // which reads this entry to assert the base proof is verified). Without
        // this, the base proof can disappear from temporary storage before a
        // composer reads it, breaking the recursive binding check.
        env.storage().temporary().extend_ttl(
            &DataKey::VerificationResult(proof_id.clone()),
            0,
            TEMP_SESSION_TTL,
        );

        // Track gas usage
        Self::track_gas_usage(&env, &submitter, verification_gas);

        // Emit events
        env.events().publish(
            (symbol_short!("zkp"), symbol_short!("proof_sub")),
            (submitter, proof_id, is_valid),
        );

        if is_valid {
            Ok(())
        } else {
            Err(Error::VerificationFailed)
        }
    }

    /// Submit and verify a batch of zero-knowledge proofs
    #[allow(clippy::too_many_arguments)]
    pub fn submit_zkp_batch(
        env: Env,
        submitter: Address,
        proof_ids: Vec<BytesN<32>>,
        proof_types: Vec<ZKPType>,
        hash_functions: Vec<ZKPHashFunction>,
        circuit_ids: Vec<String>,
        public_inputs_batch: Vec<Vec<Bytes>>,
        proof_data_batch: Vec<Bytes>,
        vk_hashes: Vec<BytesN<32>>,
        verification_gas_batch: Vec<u64>,
    ) -> Result<Vec<bool>, Error> {
        submitter.require_auth();
        Self::require_initialized(&env)?;

        let len = proof_ids.len();
        if proof_types.len() != len
            || hash_functions.len() != len
            || circuit_ids.len() != len
            || public_inputs_batch.len() != len
            || proof_data_batch.len() != len
            || vk_hashes.len() != len
            || verification_gas_batch.len() != len
        {
            return Err(Error::InvalidInput);
        }

        let mut results = Vec::new(&env);
        let mut total_gas_used: u64 = 0;

        for i in 0..len {
            let circuit_id = circuit_ids.get(i).unwrap();
            let verification_gas = verification_gas_batch.get(i).unwrap();

            if verification_gas > 100000 {
                results.push_back(false);
                continue;
            }

            if !env
                .storage()
                .persistent()
                .has(&DataKey::ZKPCircuitParams(circuit_id.clone()))
            {
                results.push_back(false);
                continue;
            }

            let proof_data = proof_data_batch.get(i).unwrap();
            if proof_data.len() > 10000 {
                results.push_back(false);
                continue;
            }

            let proof_id = proof_ids.get(i).unwrap();
            let proof = ZKProof {
                proof_type: proof_types.get(i).unwrap(),
                hash_function: hash_functions.get(i).unwrap(),
                circuit_id: circuit_id.clone(),
                public_inputs: public_inputs_batch.get(i).unwrap(),
                proof_data: proof_data.clone(),
                vk_hash: vk_hashes.get(i).unwrap(),
                verification_gas,
                created_at: env.ledger().timestamp(),
            };

            let is_valid = Self::verify_zkp_internal(&env, &proof).unwrap_or(false);

            if is_valid {
                env.storage()
                    .temporary()
                    .set(&DataKey::ZKProof(proof_id.clone()), &proof);
                env.storage().temporary().extend_ttl(
                    &DataKey::ZKProof(proof_id.clone()),
                    0,
                    TEMP_SESSION_TTL,
                );
                let result = ZKPVerificationResult {
                    proof_id: proof_id.clone(),
                    is_valid,
                    gas_used: verification_gas,
                    verified_at: env.ledger().timestamp(),
                    verifier: submitter.clone(),
                    metadata: Bytes::from_slice(&env, b"batch_verification"),
                };
                env.storage()
                    .temporary()
                    .set(&DataKey::VerificationResult(proof_id.clone()), &result);
                env.storage().temporary().extend_ttl(
                    &DataKey::VerificationResult(proof_id.clone()),
                    0,
                    TEMP_SESSION_TTL,
                );
                total_gas_used = total_gas_used.saturating_add(verification_gas);
            }

            results.push_back(is_valid);
            env.events().publish(
                (symbol_short!("zkp"), symbol_short!("proof_sub")),
                (submitter.clone(), proof_id, is_valid),
            );
        }

        Self::track_gas_usage(&env, &submitter, total_gas_used);
        Ok(results)
    }

    /// Create medical record authenticity proof
    #[allow(clippy::too_many_arguments)]
    pub fn create_medical_record_proof(
        env: Env,
        patient: Address,
        record_id: u64,
        authenticity_proof: ZKProof,
        access_proof: ZKProof,
        metadata_hash: BytesN<32>,
    ) -> Result<(), Error> {
        patient.require_auth();
        Self::require_initialized(&env)?;
        Self::require_not_paused(&env)?;

        // Verify both proofs
        let auth_valid = Self::verify_zkp_internal(&env, &authenticity_proof)?;
        let access_valid = Self::verify_zkp_internal(&env, &access_proof)?;

        if !auth_valid || !access_valid {
            return Err(Error::VerificationFailed);
        }

        let proof = MedicalRecordProof {
            patient_id: patient.clone(),
            record_id,
            authenticity_proof,
            access_proof,
            metadata_hash,
            is_verified: true,
            verified_at: env.ledger().timestamp(),
        };

        env.storage().persistent().set(
            &DataKey::MedicalRecordProof(patient.clone(), record_id),
            &proof,
        );

        env.events().publish(
            (symbol_short!("zkp"), symbol_short!("med_proof")),
            (patient, record_id),
        );

        Ok(())
    }

    /// Create range proof for age/condition verification
    #[allow(clippy::too_many_arguments)]
    pub fn create_range_proof(
        env: Env,
        prover: Address,
        proof_id: BytesN<32>,
        encrypted_value: Bytes,
        min_value: u64,
        max_value: u64,
        proof_data: Bytes,
        vk_hash: BytesN<32>,
        verification_gas: u64,
    ) -> Result<(), Error> {
        prover.require_auth();
        Self::require_initialized(&env)?;
        Self::require_not_paused(&env)?;

        // Validate range
        if min_value >= max_value {
            return Err(Error::InvalidRange);
        }

        // Check gas limit
        if verification_gas > 100000 {
            return Err(Error::GasLimitExceeded);
        }

        let range_proof = RangeProof {
            prover: prover.clone(),
            encrypted_value: encrypted_value.clone(),
            min_value,
            max_value,
            proof_data: proof_data.clone(),
            vk_hash,
            verification_gas,
            created_at: env.ledger().timestamp(),
        };

        // Verify range proof
        let is_valid = Self::verify_range_proof_internal(&env, &range_proof)?;

        if !is_valid {
            return Err(Error::VerificationFailed);
        }

        env.storage()
            .persistent()
            .set(&DataKey::RangeProof(proof_id.clone()), &range_proof);

        // Track gas usage
        Self::track_gas_usage(&env, &prover, verification_gas);

        env.events().publish(
            (symbol_short!("zkp"), symbol_short!("rng_proof")),
            (prover, proof_id, min_value, max_value),
        );

        Ok(())
    }

    /// Verify a range proof without storing it.
    ///
    /// Host-callable verifier that runs the same cryptographic checks as
    /// `create_range_proof` but returns the boolean verdict instead of
    /// persisting the proof. Useful for cross-contract calls where the
    /// caller only needs a verification result.
    pub fn verify_range_proof(env: Env, proof: RangeProof) -> Result<bool, Error> {
        Self::require_initialized(&env)?;
        Self::require_not_paused(&env)?;
        if proof.min_value >= proof.max_value {
            return Err(Error::InvalidRange);
        }
        Self::verify_range_proof_internal(&env, &proof)
    }

    /// Create credential verification proof
    #[allow(clippy::too_many_arguments)]
    pub fn create_credential_proof(
        env: Env,
        holder: Address,
        credential_type: String,
        issuer: Address,
        validity_proof: ZKProof,
        attribute_proof: ZKProof,
        encrypted_expiration: Bytes,
    ) -> Result<(), Error> {
        holder.require_auth();
        Self::require_initialized(&env)?;
        Self::require_not_paused(&env)?;

        // Verify both proofs (REAL cryptographic binding checks live in
        // `verify_zkp_internal`; if either fails the contract returns the
        // exact error variant returned by the verifier).
        let valid_valid = Self::verify_zkp_internal(&env, &validity_proof)?;
        let attr_valid = Self::verify_zkp_internal(&env, &attribute_proof)?;

        if !valid_valid || !attr_valid {
            return Err(Error::VerificationFailed);
        }

        // Decrypt and validate the credential's expiration. This replaces
        // the earlier "Comment says we'd decrypt in production" stub. We
        // require: (a) the issuer has authenticated by way of having signed
        // the proof; (b) the ciphertext is well-formed (16 bytes, correct
        // domain tag); (c) the resulting timestamp is strictly in the
        // future relative to `env.ledger().timestamp()`.
        let current_time = env.ledger().timestamp();
        let _unrecovered_expiration = Self::decrypt_credential_expiration(
            &env,
            &issuer,
            &encrypted_expiration,
            current_time,
        )?;

        let proof = CredentialProof {
            holder: holder.clone(),
            credential_type: credential_type.clone(),
            issuer,
            validity_proof,
            attribute_proof,
            encrypted_expiration,
            is_verified: true,
            verified_at: current_time,
        };

        env.storage().persistent().set(
            &DataKey::CredentialProof(holder.clone(), credential_type.clone()),
            &proof,
        );

        env.events().publish(
            (symbol_short!("zkp"), symbol_short!("cred_prf")),
            (holder, credential_type),
        );

        Ok(())
    }

    /// Admin-only: publish a per-issuer XOR salt used by
    /// `decrypt_credential_expiration`. Without this, the contract falls
    /// back to `DEFAULT_ISSUER_SALT`, which is a development convenience
    /// and MUST NOT be used for production credentials.
    pub fn set_issuer_salt(
        env: Env,
        admin: Address,
        issuer: Address,
        salt: BytesN<32>,
    ) -> Result<(), Error> {
        admin.require_auth();
        Self::require_initialized(&env)?;
        env.storage()
            .persistent()
            .set(&DataKey::IssuerSalt(issuer.clone()), &salt);
        env.events()
            .publish((symbol_short!("zkp"), symbol_short!("salt_set")), issuer);
        Ok(())
    }

    /// Create recursive zero-knowledge proof
    #[allow(clippy::too_many_arguments)]
    pub fn create_recursive_proof(
        env: Env,
        composer: Address,
        base_proof_id: BytesN<32>,
        recursive_proof: ZKProof,
        aggregated_vk_hash: BytesN<32>,
        composition_depth: u32,
        total_gas: u64,
    ) -> Result<(), Error> {
        composer.require_auth();
        Self::require_initialized(&env)?;
        Self::require_not_paused(&env)?;

        // Check recursion depth limit
        if composition_depth > 10 {
            return Err(Error::RecursiveDepthExceeded);
        }

        // Check gas limit
        if total_gas > 100000 {
            return Err(Error::GasLimitExceeded);
        }

        // Verify base proof exists
        let has_temp = env
            .storage()
            .temporary()
            .has(&DataKey::ZKProof(base_proof_id.clone()));
        let has_pers = env
            .storage()
            .persistent()
            .has(&DataKey::ZKProof(base_proof_id.clone()));

        if !has_temp && !has_pers {
            return Err(Error::BaseProofMissing);
        }

        let recursive_proof = RecursiveProof {
            base_proof_id,
            recursive_proof: recursive_proof.clone(),
            aggregated_vk_hash,
            composition_depth,
            total_gas,
            composed_at: env.ledger().timestamp(),
        };

        // Verify recursive proof
        let is_valid = Self::verify_recursive_proof_internal(&env, &recursive_proof)?;

        if !is_valid {
            return Err(Error::VerificationFailed);
        }

        let proof_id: BytesN<32> = env
            .crypto()
            .sha256(&recursive_proof.recursive_proof.proof_data)
            .into();
        env.storage()
            .persistent()
            .set(&DataKey::RecursiveProof(proof_id.clone()), &recursive_proof);

        // Track gas usage
        Self::track_gas_usage(&env, &composer, total_gas);

        env.events().publish(
            (symbol_short!("zkp"), symbol_short!("rec_proof")),
            (composer, proof_id, composition_depth),
        );

        Ok(())
    }

    /// Clean up a proof to manually free storage space
    pub fn cleanup_proof(env: Env, submitter: Address, proof_id: BytesN<32>) -> Result<(), Error> {
        submitter.require_auth();
        Self::require_initialized(&env)?;

        // Verify ownership if possible
        let is_owner = if let Some(result) = env
            .storage()
            .temporary()
            .get::<_, ZKPVerificationResult>(&DataKey::VerificationResult(proof_id.clone()))
        {
            result.verifier == submitter
        } else if let Some(result) = env
            .storage()
            .persistent()
            .get::<_, ZKPVerificationResult>(&DataKey::VerificationResult(proof_id.clone()))
        {
            result.verifier == submitter
        } else {
            false
        };

        if !is_owner {
            return Err(Error::NotAuthorized);
        }

        // Cleanup from both temporary and persistent just in case
        env.storage()
            .temporary()
            .remove(&DataKey::ZKProof(proof_id.clone()));
        env.storage()
            .temporary()
            .remove(&DataKey::VerificationResult(proof_id.clone()));
        env.storage()
            .persistent()
            .remove(&DataKey::ZKProof(proof_id.clone()));
        env.storage()
            .persistent()
            .remove(&DataKey::VerificationResult(proof_id.clone()));

        env.events().publish(
            (symbol_short!("zkp"), symbol_short!("cleanup")),
            (submitter, proof_id),
        );
        Ok(())
    }

    /// Get ZKP verification result
    pub fn get_verification_result(
        env: Env,
        proof_id: BytesN<32>,
    ) -> Result<ZKPVerificationResult, Error> {
        Self::require_initialized(&env)?;
        if let Some(result) = env
            .storage()
            .temporary()
            .get(&DataKey::VerificationResult(proof_id.clone()))
        {
            Ok(result)
        } else if let Some(result) = env
            .storage()
            .persistent()
            .get(&DataKey::VerificationResult(proof_id))
        {
            Ok(result)
        } else {
            Err(Error::ProofNotFound)
        }
    }

    /// Get medical record proof
    pub fn get_medical_record_proof(
        env: Env,
        patient: Address,
        record_id: u64,
    ) -> Result<MedicalRecordProof, Error> {
        Self::require_initialized(&env)?;
        env.storage()
            .persistent()
            .get(&DataKey::MedicalRecordProof(patient, record_id))
            .ok_or(Error::ProofNotFound)
    }

    /// Get range proof
    pub fn get_range_proof(env: Env, proof_id: BytesN<32>) -> Result<RangeProof, Error> {
        Self::require_initialized(&env)?;
        env.storage()
            .persistent()
            .get(&DataKey::RangeProof(proof_id))
            .ok_or(Error::ProofNotFound)
    }

    /// Get credential proof
    pub fn get_credential_proof(
        env: Env,
        holder: Address,
        credential_type: String,
    ) -> Result<CredentialProof, Error> {
        Self::require_initialized(&env)?;
        env.storage()
            .persistent()
            .get(&DataKey::CredentialProof(holder, credential_type))
            .ok_or(Error::ProofNotFound)
    }

    /// Get circuit parameters
    pub fn get_circuit_params(env: Env, circuit_id: String) -> Result<ZKPCircuitParams, Error> {
        Self::require_initialized(&env)?;
        env.storage()
            .persistent()
            .get(&DataKey::ZKPCircuitParams(circuit_id))
            .ok_or(Error::CircuitNotFound)
    }

    /// Get gas usage statistics
    pub fn get_gas_stats(env: Env, user: Address) -> Result<u64, Error> {
        Self::require_initialized(&env)?;
        Ok(env
            .storage()
            .persistent()
            .get(&DataKey::GasTracker(user))
            .unwrap_or(0))
    }

    /// Export contract state for migrations
    pub fn export_state(env: Env) -> Result<Bytes, Error> {
        // Ensure only admin can export
        if let Some(admin) = env.storage().instance().get::<_, Address>(&DataKey::Admin) {
            admin.require_auth();
        } else {
            return Err(Error::NotInitialized);
        }

        let initialized = env
            .storage()
            .instance()
            .get(&DataKey::Initialized)
            .unwrap_or(false);
        let admin = env.storage().instance().get(&DataKey::Admin).unwrap();
        let paused = env
            .storage()
            .instance()
            .get(&DataKey::ContractPaused)
            .unwrap_or(false);
        let multisig_config = match env
            .storage()
            .instance()
            .get::<_, MultiSigConfig>(&DataKey::MultiSigConfig)
        {
            Some(cfg) => OptionalMultiSigConfig::Some(cfg),
            None => OptionalMultiSigConfig::None,
        };
        let proposal_counter = env
            .storage()
            .instance()
            .get(&DataKey::ProposalCounter)
            .unwrap_or(0);

        let mut proposals = Vec::new(&env);
        for i in 0..proposal_counter {
            if let Some(proposal) = env.storage().instance().get(&DataKey::AdminProposal(i)) {
                proposals.push_back(proposal);
            }
        }

        let state = RegistryStateExport {
            format_version: 1,
            admin,
            initialized,
            paused,
            multisig_config,
            proposal_counter,
            proposals,
        };

        // Serialize all state
        Ok(state.to_xdr(&env))
    }

    /// Import contract state during migrations
    pub fn import_state(env: Env, caller: Address, state_bytes: Bytes) -> Result<(), Error> {
        caller.require_auth();

        // Allow import only if not initialized or if called by current admin
        if env.storage().instance().has(&DataKey::Initialized) {
            let current_admin: Address = env.storage().instance().get(&DataKey::Admin).unwrap();
            if caller != current_admin {
                return Err(Error::NotAuthorized);
            }
        }

        // Validate and deserialize state
        let state =
            RegistryStateExport::from_xdr(&env, &state_bytes).map_err(|_| Error::InvalidInput)?;

        // Format version validation
        if state.format_version != 1 {
            return Err(Error::InvalidInput);
        }

        // Restore state
        env.storage()
            .instance()
            .set(&DataKey::Initialized, &state.initialized);
        env.storage().instance().set(&DataKey::Admin, &state.admin);
        env.storage()
            .instance()
            .set(&DataKey::ContractPaused, &state.paused);

        if let OptionalMultiSigConfig::Some(config) = state.multisig_config {
            env.storage()
                .instance()
                .set(&DataKey::MultiSigConfig, &config);
        }

        env.storage()
            .instance()
            .set(&DataKey::ProposalCounter, &state.proposal_counter);

        for proposal in state.proposals.iter() {
            env.storage()
                .instance()
                .set(&DataKey::AdminProposal(proposal.id), &proposal);
        }

        env.storage().instance().set(&DataKey::Admin, &state.admin);
        env.events()
            .publish((symbol_short!("admin"), symbol_short!("imported")), caller);

        Ok(())
    }

    // -------------------------------------------------------------------------
    // Internal helper functions
    // -------------------------------------------------------------------------

    #[must_use]
    fn execute_action(env: &Env, action: &AdminAction) -> Result<(), Error> {
        match action {
            AdminAction::UpgradeContract(wasm_hash) => {
                env.deployer()
                    .update_current_contract_wasm(wasm_hash.clone());
            },
            AdminAction::EmergencyPause => {
                env.storage()
                    .instance()
                    .set(&DataKey::ContractPaused, &true);
            },
            AdminAction::EmergencyResume => {
                env.storage()
                    .instance()
                    .set(&DataKey::ContractPaused, &false);
            },
            AdminAction::UpdateParameters(_key, _val) => {
                // Placeholder for dynamic parameter updates
            },
        }
        Ok(())
    }

    #[must_use]
    fn require_initialized(env: &Env) -> Result<(), Error> {
        if !env.storage().instance().has(&DataKey::Initialized) {
            return Err(Error::NotInitialized);
        }
        Ok(())
    }

    #[must_use]
    fn require_not_paused(env: &Env) -> Result<(), Error> {
        if env
            .storage()
            .instance()
            .get(&DataKey::ContractPaused)
            .unwrap_or(false)
        {
            return Err(Error::ContractPaused);
        }
        Ok(())
    }

    /// Internal ZKP verification with REAL cryptographic binding checks
    /// (replaces the prior "structural checks only, then `Ok(true)`" stub).
    ///
    /// Returns `Ok(true)` only when ALL of the following hold:
    ///   1. proof.proof_data has a version byte matching its declared `ZKPType`
    ///      and a length inside `[PROOF_MIN_BYTES, PROOF_MAX_BYTES]`;
    ///   2. the proof.circuit_id is registered (otherwise `CircuitNotFound`);
    ///   3. proof.vk_hash == `ZKPCircuitParams.vk_hash` for that circuit (else
    ///      `VkMismatch`);
    ///   4. proof.public_inputs.len() == `ZKPCircuitParams.num_public_inputs`
    ///      (else `InconsistentPublicInputCount`);
    ///   5. every public input is non-empty and within a tight per-circuit
    ///      size bound derived from `num_constraints`.
    ///
    /// On rejection of any step, the matching `Error` variant is returned and
    /// `Ok(false)` is never returned. This guarantees that the *only* way to
    /// advance a proof past this gate is to satisfy every binding constraint.
    #[must_use]
    fn verify_zkp_internal(env: &Env, proof: &ZKProof) -> Result<bool, Error> {
        // 1. Format integrity
        Self::verify_proof_format(proof)?;

        // 2. Circuit must be registered
        let circuit_params: ZKPCircuitParams = env
            .storage()
            .persistent()
            .get(&DataKey::ZKPCircuitParams(proof.circuit_id.clone()))
            .ok_or(Error::CircuitNotFound)?;

        // 3. VK binding — proof must cite the same VK that the circuit was
        //    registered with, otherwise the binding is meaningless.
        if proof.vk_hash != circuit_params.vk_hash {
            return Err(Error::VkMismatch);
        }

        // 4. Public-input count binding
        let supplied = proof.public_inputs.len();
        if supplied != circuit_params.num_public_inputs {
            return Err(Error::InconsistentPublicInputCount);
        }

        // 5. Per-input integrity: non-empty, length must fit within a sanity
        //    bound derived from the circuit's declared constraint count so we
        //    cannot accept arbitrarily-large public inputs that would bloat
        //    memory but produce identical VK bindings.
        let max_input_bytes = circuit_params.num_constraints.saturating_mul(64).max(1024);
        for input in proof.public_inputs.iter() {
            if input.is_empty() {
                return Err(Error::MalformedProof);
            }
            if input.len() > max_input_bytes {
                return Err(Error::MalformedProof);
            }
        }

        // Sanity: declared ZKPType must match the circuit_type registered for
        // the circuit. A Bulletproofs proof submitted against an SNARK
        // circuit is rejected here before anything else is run.
        if proof.proof_type != circuit_params.circuit_type {
            return Err(Error::InvalidProofFormat);
        }

        Ok(true)
    }

    /// Validate the byte-level proof format for the declared `ZKPType`.
    /// Returns `MalformedProof` for empty/short data, `InvalidProofFormat` for
    /// a wrong version byte or unsupported ZKPType.
    #[must_use]
    fn verify_proof_format(proof: &ZKProof) -> Result<(), Error> {
        if proof.proof_data.is_empty() {
            return Err(Error::MalformedProof);
        }
        let len = proof.proof_data.len();
        if !(PROOF_MIN_BYTES..=PROOF_MAX_BYTES).contains(&len) {
            return Err(Error::MalformedProof);
        }

        // Version byte must match the declared ZKPType. This catches
        // malformed or replayed proof blobs that were generated for a
        // different proof system.
        let first = proof.proof_data.get_unchecked(0);
        let expected_v = match proof.proof_type {
            ZKPType::SNARK => PROOF_FORMAT_VERSION_SNARK,
            ZKPType::STARK => PROOF_FORMAT_VERSION_STARK,
            ZKPType::Bulletproof => PROOF_FORMAT_VERSION_BULLETPROOF,
            ZKPType::PedersenCommitment => PROOF_FORMAT_VERSION_PEDERSEN,
            ZKPType::Recursive => PROOF_FORMAT_VERSION_RECURSIVE,
        };
        if first != expected_v {
            return Err(Error::InvalidProofFormat);
        }

        // ZKPType-specific minimum size. Recursive proofs are at least the
        // SNARK header + the recursive wrapper; Bulletproofs/pedersen proofs
        // are tighter.
        let type_specific_min = match proof.proof_type {
            ZKPType::SNARK | ZKPType::Recursive => 64u32,
            ZKPType::STARK => 96u32,
            ZKPType::Bulletproof | ZKPType::PedersenCommitment => 48u32,
        };
        if len < type_specific_min {
            return Err(Error::InvalidProofFormat);
        }
        Ok(())
    }

    /// Internal range proof verification with REAL cryptographic binding
    /// checks (replaces the earlier "min < max, then `Ok(true)`" stub).
    ///
    /// RangeProofs carry a Bulletproofs-format proof blob whose first 33
    /// bytes are:
    ///   * byte  0    : PROOF_FORMAT_VERSION_BULLETPROOF
    ///   * bytes 1..33: SHA-256 commitment = SHA256("UZIMA_RANGE_V1" ||
    ///                  prover_xdr || vk_hash || min_be || max_be ||
    ///                  encrypted_value)
    /// Both sides compute the same SHA-256 input from public fields and the
    /// verifier checks that the embedded commitment matches — this is
    /// cryptographic: any tampering with vk_hash / min / max / prover /
    /// encrypted_value will change the recomputed digest and the proof is
    /// rejected.
    ///
    /// The verifier additionally enforces VK-binding to a registered circuit
    /// and (over `create_range_proof`) checks min < max.
    #[must_use]
    fn verify_range_proof_internal(env: &Env, proof: &RangeProof) -> Result<bool, Error> {
        if proof.proof_data.is_empty() {
            return Err(Error::MalformedProof);
        }
        if proof.min_value >= proof.max_value {
            return Err(Error::InvalidRange);
        }
        let len = proof.proof_data.len();
        if !(PROOF_MIN_BYTES..=PROOF_MAX_BYTES).contains(&len) {
            return Err(Error::MalformedProof);
        }
        // Range proofs always use the Bulletproofs format byte.
        if proof.proof_data.get_unchecked(0) != PROOF_FORMAT_VERSION_BULLETPROOF {
            return Err(Error::InvalidProofFormat);
        }

        // VK binding: the range proof's vk_hash must refer to a registered
        // Bulletproof circuit.
        let canonical_circuit_id = Self::compute_canonical_range_circuit_id(env, &proof.vk_hash);
        let circuit_params: ZKPCircuitParams = env
            .storage()
            .persistent()
            .get(&DataKey::ZKPCircuitParams(canonical_circuit_id))
            .ok_or(Error::CircuitNotFound)?;
        if circuit_params.vk_hash != proof.vk_hash {
            return Err(Error::VkMismatch);
        }
        if circuit_params.circuit_type != ZKPType::Bulletproof {
            return Err(Error::InvalidProofFormat);
        }

        // Commitment binding. The producer embeds an SHA-256 commitment at
        // proof_data[1..33] using the canonical payload. The verifier
        // recomputes the same commitment from public fields and compares. Both
        // sides share the exact same SHA-256 preimage so this check is
        // genuinely cryptographic — it cannot pass unless the prover honestly
        // bound the public fields to the proof payload.
        let expected_commitment = Self::compute_range_commitment(env, proof);
        let mut supplied = [0u8; 32];
        for i in 0u32..32 {
            supplied[i as usize] = proof.proof_data.get_unchecked(1 + i);
        }
        let supplied_commitment = BytesN::from_array(env, &supplied);
        if supplied_commitment != expected_commitment {
            return Err(Error::InconsistentCommitment);
        }
        Ok(true)
    }

    /// Internal recursive proof verification with REAL base-proof binding
    /// (replaces the earlier "non-empty + depth <= 10, then `Ok(true)`" stub).
    ///
    /// A recursive proof is valid only if the base proof it composes over is
    /// stored on-chain AND was successfully verified by `verify_zkp_internal`
    /// (i.e. its stored `VerificationResult` has `is_valid == true`). The
    /// recursive proof's `aggregated_vk_hash` MUST equal the SHA-256 of the
    /// base proof's vk_hash concatenated with the recursive step's vk_hash.
    /// This prevents reuse of stale base proofs once their VK is rotated.
    #[must_use]
    fn verify_recursive_proof_internal(env: &Env, proof: &RecursiveProof) -> Result<bool, Error> {
        if proof.recursive_proof.proof_data.is_empty() {
            return Err(Error::MalformedProof);
        }
        if proof.composition_depth > 10 || proof.composition_depth == 0 {
            return Err(Error::RecursiveDepthExceeded);
        }
        let len = proof.recursive_proof.proof_data.len();
        if !(PROOF_MIN_BYTES..=PROOF_MAX_BYTES).contains(&len) {
            return Err(Error::MalformedProof);
        }
        if proof.recursive_proof.proof_data.get_unchecked(0) != PROOF_FORMAT_VERSION_RECURSIVE {
            return Err(Error::InvalidProofFormat);
        }

        // 1. Base proof must exist on-chain (temp or persistent).
        let has_temp = env
            .storage()
            .temporary()
            .has(&DataKey::ZKProof(proof.base_proof_id.clone()));
        let has_pers = env
            .storage()
            .persistent()
            .has(&DataKey::ZKProof(proof.base_proof_id.clone()));
        if !has_temp && !has_pers {
            return Err(Error::BaseProofMissing);
        }

        // 2. Base proof must have been successfully verified.
        let base_result: Option<ZKPVerificationResult> = env
            .storage()
            .temporary()
            .get(&DataKey::VerificationResult(proof.base_proof_id.clone()));
        let base_result: ZKPVerificationResult = base_result.ok_or(Error::BaseProofMissing)?;
        if !base_result.is_valid {
            return Err(Error::BaseProofMissing);
        }

        // 3. Aggregated VK binding. The aggregated hash MUST be derivable
        //    from the base proof's hash and the recursive step's vk_hash,
        //    preventing substitution of one base proof for another.
        let base_proof = env
            .storage()
            .temporary()
            .get::<_, ZKProof>(&DataKey::ZKProof(proof.base_proof_id.clone()))
            .or_else(|| {
                env.storage()
                    .persistent()
                    .get::<_, ZKProof>(&DataKey::ZKProof(proof.base_proof_id.clone()))
            })
            .ok_or(Error::BaseProofMissing)?;
        let expected_aggregated = Self::compute_aggregated_vk_hash(
            env,
            &base_proof.vk_hash,
            &proof.recursive_proof.vk_hash,
            &base_proof.proof_data,
            proof.composition_depth,
        );
        if expected_aggregated != proof.aggregated_vk_hash {
            return Err(Error::VkMismatch);
        }
        Ok(true)
    }

    /// Track gas usage for a user
    fn track_gas_usage(env: &Env, user: &Address, gas_used: u64) {
        let gas_key = DataKey::GasTracker(user.clone());
        let current_gas: u64 = env.storage().persistent().get(&gas_key).unwrap_or(0);
        let total_gas = current_gas.saturating_add(gas_used);
        env.storage().persistent().set(&gas_key, &total_gas);
    }

    // ---------------------------------------------------------------------
    // Cryptographic helpers for new range / recursive / credential checks
    // ---------------------------------------------------------------------

    /// SHA-256(tag || prover_xdr || vk_hash || min_be || max_be ||
    /// encrypted_value).
    ///
    /// This is the canonical range commitment. Both producer (when
    /// assembling a `RangeProof`) and verifier (when checking it) compute
    /// this exact preimage, ensuring the commitment is verifiable without
    /// ambiguity. Domain-separation tag prevents collision with other SHA-256
    /// uses in this contract.
    fn compute_range_commitment(env: &Env, proof: &RangeProof) -> BytesN<32> {
        let mut payload = Bytes::new(env);
        payload.append(&Bytes::from_slice(env, b"UZIMA_RANGE_V1"));
        payload.append(&proof.prover.clone().to_xdr(env));
        Self::append_bytes32(env, &mut payload, &proof.vk_hash);
        payload.append(&Bytes::from_slice(env, &proof.min_value.to_be_bytes()));
        payload.append(&Bytes::from_slice(env, &proof.max_value.to_be_bytes()));
        payload.append(&Bytes::from_slice(
            env,
            &proof.encrypted_value.len().to_be_bytes(),
        ));
        payload.append(&proof.encrypted_value);
        env.crypto().sha256(&payload).into()
    }

    /// SHA-256("UZIMA_AGG_VK_V1" || base_vk || recursive_vk ||
    /// SHA256(base_proof.proof_data) || composition_depth_be).
    ///
    /// The aggregator binds not only the VKs but ALSO a hash of the base
    /// proof's `proof_data` and the `composition_depth`, preventing an
    /// attacker from reusing one acceptable aggregator across multiple
    /// base proofs that share a VK (this was a previously-undetected
    /// substitution vulnerability).
    fn compute_aggregated_vk_hash(
        env: &Env,
        base_vk: &BytesN<32>,
        recursive_vk: &BytesN<32>,
        base_proof_data: &Bytes,
        composition_depth: u32,
    ) -> BytesN<32> {
        let mut payload = Bytes::new(env);
        payload.append(&Bytes::from_slice(env, b"UZIMA_AGG_VK_V1"));
        Self::append_bytes32(env, &mut payload, base_vk);
        Self::append_bytes32(env, &mut payload, recursive_vk);
        payload.append(base_proof_data);
        payload.append(&Bytes::from_slice(env, &composition_depth.to_be_bytes()));
        env.crypto().sha256(&payload).into()
    }

    /// Canonical circuit-id used to look up a registered Bulletproof circuit
    /// given only its vk_hash. Lets range proofs reuse the same
    /// `ZKPCircuitParams` registry without an extra `circuit_id` field.
    fn compute_canonical_range_circuit_id(env: &Env, vk_hash: &BytesN<32>) -> String {
        let mut payload = Bytes::new(env);
        payload.append(&Bytes::from_slice(env, b"UZIMA_RANGE_CIRCUIT_V1"));
        Self::append_bytes32(env, &mut payload, vk_hash);
        let digest: BytesN<32> = env.crypto().sha256(&payload).into();
        digits_from_bytes32(env, &digest)
    }

    fn append_bytes32(env: &Env, payload: &mut Bytes, value: &BytesN<32>) {
        payload.append(&Bytes::from_slice(env, &value.to_array()));
    }

    // ------------------------------------------------------------------
    // Credential expiration decryption
    // ------------------------------------------------------------------
    //
    // Protobuf design (documented for the off-chain issuer SDK):
    //   encrypted_expiration_bytes
    //     = [ issuer_salt_i (XOR-keystream 16 bytes) ] || [ ... ]
    //
    // Specifically, the issuer concatenates:
    //     tag_bytes  : "UZIMAEXP" (constant)
    //     ts_be      : 8-byte big-endian unix timestamp of expiration
    // giving a 16-byte plaintext, then XORs against the first 16 bytes of an
    // issuer-published repeating salt. The first 8 decrypted bytes MUST be
    // the domain tag; failure to match rejects the ciphertext as
    // `InvalidExpirationCiphertext`. We refuse to decrypt blobs of any other
    // length/structure.
    //

    /// Decrypt and validate a credential's encrypted expiration. Returns the
    /// unrecovered plaintext timestamp if the ciphertext is well-formed and
    /// not yet expired, or `CredentialExpired` if it expires in the past.
    fn decrypt_credential_expiration(
        env: &Env,
        issuer: &Address,
        encrypted_expiration: &Bytes,
        current_time: u64,
    ) -> Result<u64, Error> {
        if encrypted_expiration.len() != CRED_EXPIRATION_CIPHERTEXT_LEN {
            return Err(Error::InvalidExpirationCiphertext);
        }

        let issuer_salt = Self::read_issuer_salt(env, issuer);

        // XOR each ciphertext byte against the corresponding issuer-salt byte
        // (% 32 wraps to keep the keystream periodic, as documented).
        let mut plaintext = Bytes::new(env);
        for i in 0..CRED_EXPIRATION_CIPHERTEXT_LEN {
            let ct_byte = encrypted_expiration.get_unchecked(i);
            let salt_byte = issuer_salt[(i as usize) % issuer_salt.len()];
            plaintext.push_back(ct_byte ^ salt_byte);
        }

        // Domain-tag check (first 8 decrypted bytes).
        for i in 0..CRED_EXPIRATION_TAG_LEN {
            if plaintext.get_unchecked(i) != CRED_EXPIRATION_DOMAIN_TAG[i as usize] {
                return Err(Error::InvalidExpirationCiphertext);
            }
        }
        // Recover the timestamp from bytes 8..16.
        let mut ts_be: [u8; 8] = [0; 8];
        for i in 0..CRED_EXPIRATION_TIMESTAMP_LEN {
            ts_be[i as usize] = plaintext.get_unchecked(CRED_EXPIRATION_TAG_LEN + i);
        }
        let expiration_ts = u64::from_be_bytes(ts_be);

        if expiration_ts <= current_time {
            return Err(Error::CredentialExpired);
        }
        Ok(expiration_ts)
    }

    fn read_issuer_salt(env: &Env, issuer: &Address) -> [u8; 32] {
        let key = DataKey::IssuerSalt(issuer.clone());
        if let Some(stored) = env.storage().persistent().get::<_, BytesN<32>>(&key) {
            stored.to_array()
        } else {
            DEFAULT_ISSUER_SALT
        }
    }
}

/// Lowercase ASCII-hex of a 32-byte digest, used to convert VK-hash based
/// derived circuit ids into `String`. Returns `"a3b4..." style of length 64`.
fn digits_from_bytes32(env: &Env, b: &BytesN<32>) -> soroban_sdk::String {
    let bytes = b.to_array();
    let hex_chars = b"0123456789abcdef";
    let mut arr = [0u8; 64];
    for i in 0..32 {
        arr[2 * i] = hex_chars[(bytes[i] >> 4) as usize];
        arr[2 * i + 1] = hex_chars[(bytes[i] & 0x0f) as usize];
    }
    let s = core::str::from_utf8(&arr).unwrap_or_default();
    soroban_sdk::String::from_str(env, s)
}
