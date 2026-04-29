//! # Contract Verification Metadata
//!
//! Resolves issue #438: publishes on-chain verification metadata so that block
//! explorers (e.g. Stellar Expert) can display source-code provenance, build
//! reproducibility information, and ABI details.
//!
//! ## What is stored
//! * `ContractMetadata` – human-readable name, version, source URL, licence.
//! * `BuildInfo` – compiler version, optimisation flags, WASM hash.
//! * `AbiEntry` – lightweight ABI description for each public function.
//!
//! ## Explorer integration
//! After deployment, call `publish_metadata` once (admin-only).  Block explorers
//! that index Soroban events will pick up the `VERIFY/META` event and display
//! the information alongside the contract.

#![no_std]

use soroban_sdk::{
    contract, contractimpl, contracttype, symbol_short, vec, Address, BytesN, Env, String, Vec,
};

// ── Types ─────────────────────────────────────────────────────────────────────

/// Human-readable contract metadata for block-explorer display.
#[derive(Clone)]
#[contracttype]
pub struct ContractMetadata {
    /// Contract name, e.g. "MedicalRecords".
    pub name: String,
    /// Semantic version string, e.g. "1.2.0".
    pub version: String,
    /// URL to the public source repository.
    pub source_url: String,
    /// SPDX licence identifier, e.g. "MIT".
    pub license: String,
    /// Short description shown in explorer UI.
    pub description: String,
    /// Ledger timestamp when metadata was published.
    pub published_at: u64,
    /// Address of the publisher (must be admin).
    pub publisher: Address,
}

/// Reproducible-build information.
#[derive(Clone)]
#[contracttype]
pub struct BuildInfo {
    /// Rust toolchain, e.g. "1.78.0".
    pub rust_version: String,
    /// Soroban SDK version, e.g. "21.7.7".
    pub sdk_version: String,
    /// Optimisation profile, e.g. "release".
    pub build_profile: String,
    /// SHA-256 hash of the deployed WASM binary.
    pub wasm_hash: BytesN<32>,
    /// Git commit SHA at build time.
    pub commit_sha: String,
}

/// Lightweight ABI entry for a single public function.
#[derive(Clone)]
#[contracttype]
pub struct AbiEntry {
    /// Function name.
    pub name: String,
    /// Comma-separated parameter types, e.g. "Address, u64, String".
    pub params: String,
    /// Return type, e.g. "Result<u64, Error>".
    pub returns: String,
    /// One-line description.
    pub doc: String,
}

#[derive(Clone, PartialEq, Eq)]
#[contracttype]
pub enum DataKey {
    Admin,
    Metadata,
    BuildInfo,
    AbiEntries,
    IsVerified,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[contracttype]
#[repr(u32)]
pub enum VerificationError {
    NotInitialized = 1,
    AlreadyInitialized = 2,
    Unauthorized = 3,
    MetadataNotFound = 4,
}

// ── Contract ──────────────────────────────────────────────────────────────────

#[contract]
pub struct ContractVerification;

#[contractimpl]
impl ContractVerification {
    /// Initialise the verification registry with an admin address.
    pub fn initialize(env: Env, admin: Address) -> Result<(), VerificationError> {
        if env.storage().instance().has(&DataKey::Admin) {
            return Err(VerificationError::AlreadyInitialized);
        }
        admin.require_auth();
        env.storage().instance().set(&DataKey::Admin, &admin);
        Ok(())
    }

    /// Publish contract metadata.  Must be called by the admin.
    ///
    /// Emits a `(VERIFY, META)` event that block explorers can index.
    pub fn publish_metadata(
        env: Env,
        name: String,
        version: String,
        source_url: String,
        license: String,
        description: String,
    ) -> Result<(), VerificationError> {
        let admin = Self::get_admin(&env)?;
        admin.require_auth();

        let metadata = ContractMetadata {
            name: name.clone(),
            version: version.clone(),
            source_url,
            license,
            description,
            published_at: env.ledger().timestamp(),
            publisher: admin,
        };

        env.storage()
            .instance()
            .set(&DataKey::Metadata, &metadata);

        // Emit event for block-explorer indexing.
        env.events().publish(
            (symbol_short!("VERIFY"), symbol_short!("META")),
            (name, version),
        );

        Ok(())
    }

    /// Publish build reproducibility information.
    pub fn publish_build_info(
        env: Env,
        rust_version: String,
        sdk_version: String,
        build_profile: String,
        wasm_hash: BytesN<32>,
        commit_sha: String,
    ) -> Result<(), VerificationError> {
        let admin = Self::get_admin(&env)?;
        admin.require_auth();

        let build_info = BuildInfo {
            rust_version,
            sdk_version,
            build_profile,
            wasm_hash: wasm_hash.clone(),
            commit_sha,
        };

        env.storage()
            .instance()
            .set(&DataKey::BuildInfo, &build_info);

        env.events().publish(
            (symbol_short!("VERIFY"), symbol_short!("BUILD")),
            wasm_hash,
        );

        Ok(())
    }

    /// Publish the ABI for all public functions.
    pub fn publish_abi(env: Env, entries: Vec<AbiEntry>) -> Result<(), VerificationError> {
        let admin = Self::get_admin(&env)?;
        admin.require_auth();

        env.storage()
            .instance()
            .set(&DataKey::AbiEntries, &entries);

        env.events().publish(
            (symbol_short!("VERIFY"), symbol_short!("ABI")),
            entries.len(),
        );

        Ok(())
    }

    /// Mark the contract as fully verified (metadata + build + ABI all present).
    pub fn mark_verified(env: Env) -> Result<(), VerificationError> {
        let admin = Self::get_admin(&env)?;
        admin.require_auth();

        if !env.storage().instance().has(&DataKey::Metadata)
            || !env.storage().instance().has(&DataKey::BuildInfo)
            || !env.storage().instance().has(&DataKey::AbiEntries)
        {
            return Err(VerificationError::MetadataNotFound);
        }

        env.storage().instance().set(&DataKey::IsVerified, &true);

        env.events().publish(
            (symbol_short!("VERIFY"), symbol_short!("OK")),
            env.ledger().timestamp(),
        );

        Ok(())
    }

    // ── Queries ───────────────────────────────────────────────────────────────

    pub fn get_metadata(env: Env) -> Result<ContractMetadata, VerificationError> {
        env.storage()
            .instance()
            .get(&DataKey::Metadata)
            .ok_or(VerificationError::MetadataNotFound)
    }

    pub fn get_build_info(env: Env) -> Result<BuildInfo, VerificationError> {
        env.storage()
            .instance()
            .get(&DataKey::BuildInfo)
            .ok_or(VerificationError::MetadataNotFound)
    }

    pub fn get_abi(env: Env) -> Result<Vec<AbiEntry>, VerificationError> {
        env.storage()
            .instance()
            .get(&DataKey::AbiEntries)
            .ok_or(VerificationError::MetadataNotFound)
    }

    pub fn is_verified(env: Env) -> bool {
        env.storage()
            .instance()
            .get::<DataKey, bool>(&DataKey::IsVerified)
            .unwrap_or(false)
    }

    // ── Internal ──────────────────────────────────────────────────────────────

    fn get_admin(env: &Env) -> Result<Address, VerificationError> {
        env.storage()
            .instance()
            .get(&DataKey::Admin)
            .ok_or(VerificationError::NotInitialized)
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod test {
    use super::*;
    use soroban_sdk::{testutils::Address as _, BytesN, Env};

    fn setup(env: &Env) -> (ContractVerificationClient, Address) {
        let contract_id = env.register_contract(None, ContractVerification);
        let client = ContractVerificationClient::new(env, &contract_id);
        let admin = Address::generate(env);
        env.mock_all_auths();
        client.initialize(&admin);
        (client, admin)
    }

    #[test]
    fn test_publish_and_get_metadata() {
        let env = Env::default();
        let (client, _) = setup(&env);

        client.publish_metadata(
            &String::from_str(&env, "MedicalRecords"),
            &String::from_str(&env, "1.0.0"),
            &String::from_str(&env, "https://github.com/Stellar-Uzima/Uzima-Contracts"),
            &String::from_str(&env, "MIT"),
            &String::from_str(&env, "Decentralised medical records on Stellar"),
        );

        let meta = client.get_metadata();
        assert_eq!(meta.version, String::from_str(&env, "1.0.0"));
    }

    #[test]
    fn test_publish_build_info() {
        let env = Env::default();
        let (client, _) = setup(&env);

        let wasm_hash = BytesN::from_array(&env, &[0u8; 32]);
        client.publish_build_info(
            &String::from_str(&env, "1.78.0"),
            &String::from_str(&env, "21.7.7"),
            &String::from_str(&env, "release"),
            &wasm_hash,
            &String::from_str(&env, "abc123"),
        );

        let build = client.get_build_info();
        assert_eq!(build.rust_version, String::from_str(&env, "1.78.0"));
    }

    #[test]
    fn test_mark_verified_requires_all_data() {
        let env = Env::default();
        let (client, _) = setup(&env);

        // Should fail – metadata not yet published.
        assert_eq!(
            client.try_mark_verified(),
            Err(Ok(VerificationError::MetadataNotFound))
        );
    }

    #[test]
    fn test_full_verification_flow() {
        let env = Env::default();
        let (client, _) = setup(&env);

        client.publish_metadata(
            &String::from_str(&env, "TestContract"),
            &String::from_str(&env, "0.1.0"),
            &String::from_str(&env, "https://example.com"),
            &String::from_str(&env, "MIT"),
            &String::from_str(&env, "Test"),
        );

        let wasm_hash = BytesN::from_array(&env, &[1u8; 32]);
        client.publish_build_info(
            &String::from_str(&env, "1.78.0"),
            &String::from_str(&env, "21.7.7"),
            &String::from_str(&env, "release"),
            &wasm_hash,
            &String::from_str(&env, "deadbeef"),
        );

        let abi_entries: Vec<AbiEntry> = vec![
            &env,
            AbiEntry {
                name: String::from_str(&env, "initialize"),
                params: String::from_str(&env, "Address"),
                returns: String::from_str(&env, "Result<(), Error>"),
                doc: String::from_str(&env, "Initialise the contract"),
            },
        ];
        client.publish_abi(&abi_entries);

        assert!(!client.is_verified());
        client.mark_verified();
        assert!(client.is_verified());
    }

    #[test]
    fn test_double_initialize_fails() {
        let env = Env::default();
        let (client, _) = setup(&env);
        let admin2 = Address::generate(&env);
        assert_eq!(
            client.try_initialize(&admin2),
            Err(Ok(VerificationError::AlreadyInitialized))
        );
    }
}
