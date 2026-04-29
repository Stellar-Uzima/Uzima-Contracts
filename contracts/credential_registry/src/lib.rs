#![no_std]
#![allow(clippy::too_many_arguments)]

use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype, symbol_short, Address, BytesN, Env,
};

const MAX_METADATA_HASH_SIZE: u32 = 32;

#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub struct CredentialRootRecord {
    pub version: u32,
    pub root: BytesN<32>,
    pub metadata_hash: BytesN<32>,
    pub updated_at: u64,
    pub expiry: u64,
    pub signature: BytesN<64>,
    pub revoked: bool,
}

#[contracttype]
pub enum DataKey {
    Initialized,
    Admin,
    IssuerAdmin(Address),
    ActiveVersion(Address),
    ActiveRoot(Address),
    RootRecord(Address, u32),
    RevocationRoot(Address),
    RootToVersion(Address, BytesN<32>), // index: root bytes → version number
}

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    AlreadyInitialized = 1,
    NotInitialized = 2,
    NotAuthorized = 3,
    IssuerNotFound = 4,
    RootVersionNotFound = 5,
    InvalidCredentialId = 6,
    InvalidExpiry = 7,
    InvalidMetadata = 8,
    InvalidSignature = 9,
}

#[contract]
pub struct CredentialRegistryContract;

#[contractimpl]
impl CredentialRegistryContract {
    pub fn initialize(env: Env, admin: Address) -> Result<(), Error> {
        admin.require_auth();
        if env.storage().instance().has(&DataKey::Initialized) {
            return Err(Error::AlreadyInitialized);
        }
        env.storage().instance().set(&DataKey::Initialized, &true);
        env.storage().instance().set(&DataKey::Admin, &admin);

        env.events()
            .publish((symbol_short!("CREDREG"), symbol_short!("INIT")), admin);
        Ok(())
    }

    pub fn set_issuer_admin(
        env: Env,
        caller: Address,
        issuer: Address,
        issuer_admin: Address,
    ) -> Result<bool, Error> {
        caller.require_auth();
        Self::require_initialized(&env)?;
        Self::require_global_admin(&env, &caller)?;

        env.storage()
            .persistent()
            .set(&DataKey::IssuerAdmin(issuer.clone()), &issuer_admin.clone());
        env.events().publish(
            (symbol_short!("CREDREG"), symbol_short!("IADMIN")),
            (issuer, issuer_admin),
        );
        Ok(true)
    }

    pub fn get_issuer_admin(env: Env, issuer: Address) -> Option<Address> {
        env.storage()
            .persistent()
            .get(&DataKey::IssuerAdmin(issuer))
    }

    pub fn set_credential_root(
        env: Env,
        caller: Address,
        issuer: Address,
        root: BytesN<32>,
        metadata_hash: BytesN<32>,
        expiry: u64,
        signature: BytesN<64>,
    ) -> Result<u32, Error> {
        caller.require_auth();
        Self::require_initialized(&env)?;
        Self::require_issuer_manager(&env, &caller, &issuer)?;

        Self::validate_credential_id(&root)?;
        Self::validate_expiry(&env, expiry)?;
        Self::validate_metadata_hash(&metadata_hash)?;
        Self::validate_signature(&signature)?;

        let current: u32 = env
            .storage()
            .persistent()
            .get(&DataKey::ActiveVersion(issuer.clone()))
            .unwrap_or(0);
        let next = current.saturating_add(1);

        let rec = CredentialRootRecord {
            version: next,
            root: root.clone(),
            metadata_hash,
            updated_at: env.ledger().timestamp(),
            expiry,
            signature,
            revoked: false,
        };
        env.storage()
            .persistent()
            .set(&DataKey::RootRecord(issuer.clone(), next), &rec);
        env.storage()
            .persistent()
            .set(&DataKey::ActiveVersion(issuer.clone()), &next);
        env.storage()
            .persistent()
            .set(&DataKey::ActiveRoot(issuer.clone()), &root.clone());
        env.storage()
            .persistent()
            .set(&DataKey::RootToVersion(issuer.clone(), root.clone()), &next);

        env.events().publish(
            (symbol_short!("CREDREG"), symbol_short!("ROOT")),
            (issuer, next),
        );
        Ok(next)
    }

    pub fn revoke_root(
        env: Env,
        caller: Address,
        issuer: Address,
        version: u32,
    ) -> Result<bool, Error> {
        caller.require_auth();
        Self::require_initialized(&env)?;
        Self::require_issuer_manager(&env, &caller, &issuer)?;

        let key = DataKey::RootRecord(issuer.clone(), version);
        let mut rec: CredentialRootRecord = env
            .storage()
            .persistent()
            .get(&key)
            .ok_or(Error::RootVersionNotFound)?;
        if rec.revoked {
            return Ok(false);
        }
        rec.revoked = true;
        env.storage().persistent().set(&key, &rec);

        let active_version: u32 = env
            .storage()
            .persistent()
            .get(&DataKey::ActiveVersion(issuer.clone()))
            .unwrap_or(0);
        if active_version == version {
            env.storage()
                .persistent()
                .remove(&DataKey::ActiveRoot(issuer));
        }

        env.events()
            .publish((symbol_short!("CREDREG"), symbol_short!("REVOKE")), version);
        Ok(true)
    }

    pub fn set_revocation_root(
        env: Env,
        caller: Address,
        issuer: Address,
        revocation_root: BytesN<32>,
    ) -> Result<bool, Error> {
        caller.require_auth();
        Self::require_initialized(&env)?;
        Self::require_issuer_manager(&env, &caller, &issuer)?;

        env.storage()
            .persistent()
            .set(&DataKey::RevocationRoot(issuer.clone()), &revocation_root);
        env.events()
            .publish((symbol_short!("CREDREG"), symbol_short!("REVROOT")), issuer);
        Ok(true)
    }

    pub fn get_active_root(env: Env, issuer: Address) -> Option<BytesN<32>> {
        env.storage().persistent().get(&DataKey::ActiveRoot(issuer))
    }

    pub fn get_active_version(env: Env, issuer: Address) -> u32 {
        env.storage()
            .persistent()
            .get(&DataKey::ActiveVersion(issuer))
            .unwrap_or(0)
    }

    pub fn get_root(env: Env, issuer: Address, version: u32) -> Option<CredentialRootRecord> {
        env.storage()
            .persistent()
            .get(&DataKey::RootRecord(issuer, version))
    }

    pub fn get_revocation_root(env: Env, issuer: Address) -> Option<BytesN<32>> {
        env.storage()
            .persistent()
            .get(&DataKey::RevocationRoot(issuer))
    }

    pub fn is_root_revoked(env: Env, issuer: Address, root: BytesN<32>) -> bool {
        let version: Option<u32> = env
            .storage()
            .persistent()
            .get(&DataKey::RootToVersion(issuer.clone(), root.clone()));
        let Some(v) = version else {
            return false;
        };
        env.storage()
            .persistent()
            .get::<_, CredentialRootRecord>(&DataKey::RootRecord(issuer, v))
            .map(|rec| rec.revoked)
            .unwrap_or(false)
    }

    pub fn batch_set_credential_roots(
        env: Env,
        caller: Address,
        issuer: Address,
        roots: soroban_sdk::Vec<BytesN<32>>,
        metadata_hashes: soroban_sdk::Vec<BytesN<32>>,
        expiries: soroban_sdk::Vec<u64>,
        signatures: soroban_sdk::Vec<BytesN<64>>,
    ) -> Result<soroban_sdk::Vec<u32>, Error> {
        caller.require_auth();
        Self::require_initialized(&env)?;
        Self::require_issuer_manager(&env, &caller, &issuer)?;

        let len = roots.len();
        if len != metadata_hashes.len() || len != expiries.len() || len != signatures.len() {
            return Err(Error::InvalidCredentialId);
        }

        let now = env.ledger().timestamp();
        let mut current: u32 = env
            .storage()
            .persistent()
            .get(&DataKey::ActiveVersion(issuer.clone()))
            .unwrap_or(0);

        let mut versions = soroban_sdk::Vec::new(&env);

        for i in 0..len {
            let root = roots.get(i).unwrap();
            let metadata_hash = metadata_hashes.get(i).unwrap();
            let expiry = expiries.get(i).unwrap();
            let signature = signatures.get(i).unwrap();

            Self::validate_credential_id(&root)?;
            if expiry <= now {
                return Err(Error::InvalidExpiry);
            }
            Self::validate_metadata_hash(&metadata_hash)?;
            Self::validate_signature(&signature)?;

            current = current.saturating_add(1);
            let rec = CredentialRootRecord {
                version: current,
                root: root.clone(),
                metadata_hash,
                updated_at: now,
                expiry,
                signature,
                revoked: false,
            };
            env.storage()
                .persistent()
                .set(&DataKey::RootRecord(issuer.clone(), current), &rec);
            env.storage().persistent().set(
                &DataKey::RootToVersion(issuer.clone(), root.clone()),
                &current,
            );
            versions.push_back(current);
        }

        env.storage()
            .persistent()
            .set(&DataKey::ActiveVersion(issuer.clone()), &current);
        if let Some(last_root) = roots.get(len - 1) {
            env.storage()
                .persistent()
                .set(&DataKey::ActiveRoot(issuer.clone()), &last_root);
        }

        env.events().publish(
            (symbol_short!("CREDREG"), symbol_short!("BROOT")),
            (issuer, current),
        );
        Ok(versions)
    }

    pub fn has_active_root(env: Env, issuer: Address) -> bool {
        env.storage().persistent().has(&DataKey::ActiveRoot(issuer))
    }

    fn validate_credential_id(root: &BytesN<32>) -> Result<(), Error> {
        if root.to_array() == [0u8; 32] {
            return Err(Error::InvalidCredentialId);
        }
        Ok(())
    }

    fn validate_expiry(env: &Env, expiry: u64) -> Result<(), Error> {
        if expiry <= env.ledger().timestamp() {
            return Err(Error::InvalidExpiry);
        }
        Ok(())
    }

    fn validate_metadata_hash(metadata_hash: &BytesN<32>) -> Result<(), Error> {
        if metadata_hash.to_array() == [0u8; MAX_METADATA_HASH_SIZE as usize] {
            return Err(Error::InvalidMetadata);
        }
        Ok(())
    }

    fn validate_signature(signature: &BytesN<64>) -> Result<(), Error> {
        if signature.to_array() == [0u8; 64] {
            return Err(Error::InvalidSignature);
        }
        Ok(())
    }

    fn require_initialized(env: &Env) -> Result<(), Error> {
        if env.storage().instance().has(&DataKey::Initialized) {
            Ok(())
        } else {
            Err(Error::NotInitialized)
        }
    }

    fn require_global_admin(env: &Env, caller: &Address) -> Result<(), Error> {
        let admin: Address = env
            .storage()
            .instance()
            .get(&DataKey::Admin)
            .ok_or(Error::NotInitialized)?;
        if admin == *caller {
            Ok(())
        } else {
            Err(Error::NotAuthorized)
        }
    }

    fn require_issuer_manager(env: &Env, caller: &Address, issuer: &Address) -> Result<(), Error> {
        if Self::is_global_admin(env, caller) {
            return Ok(());
        }
        if *caller == *issuer {
            return Ok(());
        }
        let issuer_admin: Option<Address> = env
            .storage()
            .persistent()
            .get(&DataKey::IssuerAdmin(issuer.clone()));
        if issuer_admin == Some(caller.clone()) {
            Ok(())
        } else {
            Err(Error::NotAuthorized)
        }
    }

    fn is_global_admin(env: &Env, caller: &Address) -> bool {
        let admin: Option<Address> = env.storage().instance().get(&DataKey::Admin);
        admin == Some(caller.clone())
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]

    use super::*;
    use soroban_sdk::testutils::{Address as _, Ledger};

    fn setup_env() -> (
        Env,
        CredentialRegistryContractClient<'static>,
        Address,
        Address,
    ) {
        let env = Env::default();
        env.mock_all_auths();
        env.ledger().set_timestamp(1_000_000);

        let admin = Address::generate(&env);
        let issuer = Address::generate(&env);
        let contract_id = env.register_contract(None, CredentialRegistryContract);
        let client = CredentialRegistryContractClient::new(&env, &contract_id);
        client.initialize(&admin);
        (env, client, admin, issuer)
    }

    #[test]
    fn test_root_lifecycle() {
        let (env, client, admin, issuer) = setup_env();

        let root_1 = BytesN::from_array(&env, &[11u8; 32]);
        let meta_1 = BytesN::from_array(&env, &[12u8; 32]);
        let sig_1 = BytesN::from_array(&env, &[13u8; 64]);
        let expiry = 2_000_000u64;

        let v1 = client.set_credential_root(&admin, &issuer, &root_1, &meta_1, &expiry, &sig_1);
        assert_eq!(v1, 1);
        assert_eq!(client.get_active_root(&issuer), Some(root_1.clone()));

        let rec = client.get_root(&issuer, &1).unwrap();
        assert_eq!(rec.expiry, expiry);
        assert_eq!(rec.signature, sig_1);

        assert!(client.revoke_root(&admin, &issuer, &1));
        assert!(client.is_root_revoked(&issuer, &root_1));
    }

    #[test]
    fn test_invalid_credential_id_zero_root() {
        let (env, client, admin, issuer) = setup_env();
        let zero_root = BytesN::from_array(&env, &[0u8; 32]);
        let meta = BytesN::from_array(&env, &[12u8; 32]);
        let sig = BytesN::from_array(&env, &[13u8; 64]);
        let result =
            client.try_set_credential_root(&admin, &issuer, &zero_root, &meta, &2_000_000, &sig);
        assert_eq!(result, Err(Ok(Error::InvalidCredentialId)));
    }

    #[test]
    fn test_invalid_expiry_in_past() {
        let (env, client, admin, issuer) = setup_env();
        let root = BytesN::from_array(&env, &[11u8; 32]);
        let meta = BytesN::from_array(&env, &[12u8; 32]);
        let sig = BytesN::from_array(&env, &[13u8; 64]);
        let past_expiry = 500_000u64;
        let result =
            client.try_set_credential_root(&admin, &issuer, &root, &meta, &past_expiry, &sig);
        assert_eq!(result, Err(Ok(Error::InvalidExpiry)));
    }

    #[test]
    fn test_invalid_metadata_zero_hash() {
        let (env, client, admin, issuer) = setup_env();
        let root = BytesN::from_array(&env, &[11u8; 32]);
        let zero_meta = BytesN::from_array(&env, &[0u8; 32]);
        let sig = BytesN::from_array(&env, &[13u8; 64]);
        let result =
            client.try_set_credential_root(&admin, &issuer, &root, &zero_meta, &2_000_000, &sig);
        assert_eq!(result, Err(Ok(Error::InvalidMetadata)));
    }

    #[test]
    fn test_invalid_signature_zero_bytes() {
        let (env, client, admin, issuer) = setup_env();
        let root = BytesN::from_array(&env, &[11u8; 32]);
        let meta = BytesN::from_array(&env, &[12u8; 32]);
        let zero_sig = BytesN::from_array(&env, &[0u8; 64]);
        let result =
            client.try_set_credential_root(&admin, &issuer, &root, &meta, &2_000_000, &zero_sig);
        assert_eq!(result, Err(Ok(Error::InvalidSignature)));
    }
}
