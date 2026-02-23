#![no_std]
#![allow(clippy::too_many_arguments)]

#[cfg(test)]
mod test;

use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype, symbol_short, Address, Bytes, BytesN, Env,
    String, Symbol, Vec,
};

// =============================================================================
// Types
// =============================================================================

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
    /// Aggregated verification keys
    pub aggregated_vk: Bytes,
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

// =============================================================================
// Storage
// =============================================================================

#[contracttype]
pub enum DataKey {
    Initialized,
    Admin,
    ZKProof(BytesN<32>),
    MedicalRecordProof(Address, u64),
    RangeProof(BytesN<32>),
    CredentialProof(Address, String),
    RecursiveProof(BytesN<32>),
    ZKPCircuitParams(String),
    VerificationResult(BytesN<32>),
    ProofCounter,
    GasTracker(Address),
}

const ADMIN: Symbol = symbol_short!("ADMIN");

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
        env.storage().persistent().set(&ADMIN, &admin);
        env.events()
            .publish((symbol_short!("zkp"), symbol_short!("init")), admin);
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

        // Store proof
        env.storage()
            .persistent()
            .set(&DataKey::ZKProof(proof_id.clone()), &proof);

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
            .persistent()
            .set(&DataKey::VerificationResult(proof_id.clone()), &result);

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

        // Verify both proofs
        let valid_valid = Self::verify_zkp_internal(&env, &validity_proof)?;
        let attr_valid = Self::verify_zkp_internal(&env, &attribute_proof)?;

        if !valid_valid || !attr_valid {
            return Err(Error::VerificationFailed);
        }

        // Check expiration (simplified - in production would decrypt and check)
        let current_time = env.ledger().timestamp();
        // Note: In production, decrypt encrypted_expiration and compare with current_time

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

    /// Create recursive zero-knowledge proof
    #[allow(clippy::too_many_arguments)]
    pub fn create_recursive_proof(
        env: Env,
        composer: Address,
        base_proof_id: BytesN<32>,
        recursive_proof: ZKProof,
        aggregated_vk: Bytes,
        composition_depth: u32,
        total_gas: u64,
    ) -> Result<(), Error> {
        composer.require_auth();
        Self::require_initialized(&env)?;

        // Check recursion depth limit
        if composition_depth > 10 {
            return Err(Error::RecursiveDepthExceeded);
        }

        // Check gas limit
        if total_gas > 100000 {
            return Err(Error::GasLimitExceeded);
        }

        // Verify base proof exists
        if !env
            .storage()
            .persistent()
            .has(&DataKey::ZKProof(base_proof_id.clone()))
        {
            return Err(Error::ProofNotFound);
        }

        let recursive_proof = RecursiveProof {
            base_proof_id,
            recursive_proof: recursive_proof.clone(),
            aggregated_vk: aggregated_vk.clone(),
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

    /// Get ZKP verification result
    pub fn get_verification_result(
        env: Env,
        proof_id: BytesN<32>,
    ) -> Result<ZKPVerificationResult, Error> {
        Self::require_initialized(&env)?;
        env.storage()
            .persistent()
            .get(&DataKey::VerificationResult(proof_id))
            .ok_or(Error::ProofNotFound)
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

    // -------------------------------------------------------------------------
    // Internal helper functions
    // -------------------------------------------------------------------------

    fn require_initialized(env: &Env) -> Result<(), Error> {
        if !env.storage().instance().has(&DataKey::Initialized) {
            return Err(Error::NotInitialized);
        }
        Ok(())
    }

    /// Internal ZKP verification (simplified for demonstration)
    fn verify_zkp_internal(_env: &Env, proof: &ZKProof) -> Result<bool, Error> {
        // In production, this would perform actual cryptographic verification
        // For demonstration, we do basic validation

        // Check proof data is not empty
        if proof.proof_data.is_empty() {
            return Ok(false);
        }

        // Check public inputs are reasonable
        if proof.public_inputs.len() > 50 {
            return Ok(false);
        }

        // Simulate verification based on proof type and hash function
        let verification_cost = match proof.proof_type {
            ZKPType::SNARK => match proof.hash_function {
                ZKPHashFunction::Poseidon => 50000,
                ZKPHashFunction::MiMC => 45000,
                ZKPHashFunction::SHA256 => 80000,
                ZKPHashFunction::Rescue => 55000,
            },
            ZKPType::STARK => 90000,
            ZKPType::Bulletproof => 30000,
            ZKPType::PedersenCommitment => 20000,
            ZKPType::Recursive => 95000,
        };

        // Check if verification cost is within acceptable range
        Ok(verification_cost <= 100000)
    }

    /// Internal range proof verification
    fn verify_range_proof_internal(_env: &Env, proof: &RangeProof) -> Result<bool, Error> {
        // In production, this would perform actual cryptographic range proof verification
        // For demonstration, we do basic validation

        // Check proof data is not empty
        if proof.proof_data.is_empty() {
            return Ok(false);
        }

        // Check range validity
        if proof.min_value >= proof.max_value {
            return Ok(false);
        }

        // Simulate range proof verification
        Ok(true)
    }

    /// Internal recursive proof verification
    fn verify_recursive_proof_internal(_env: &Env, proof: &RecursiveProof) -> Result<bool, Error> {
        // In production, this would perform actual recursive proof verification
        // For demonstration, we do basic validation

        // Check proof data is not empty
        if proof.recursive_proof.proof_data.is_empty() {
            return Ok(false);
        }

        // Check composition depth
        if proof.composition_depth > 10 {
            return Ok(false);
        }

        // Simulate recursive verification
        Ok(true)
    }

    /// Track gas usage for a user
    fn track_gas_usage(env: &Env, user: &Address, gas_used: u64) {
        let gas_key = DataKey::GasTracker(user.clone());
        let current_gas: u64 = env.storage().persistent().get(&gas_key).unwrap_or(0);
        let total_gas = current_gas.saturating_add(gas_used);
        env.storage().persistent().set(&gas_key, &total_gas);
    }
}
