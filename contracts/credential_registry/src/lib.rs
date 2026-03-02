#![no_std]

use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype, symbol_short, Address, BytesN, Env,
};

#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub struct CredentialRootRecord {
    pub version: u32,
    pub root: BytesN<32>,
    pub metadata_hash: BytesN<32>,
    pub updated_at: u64,
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
    ) -> Result<u32, Error> {
        caller.require_auth();
        Self::require_initialized(&env)?;
        Self::require_issuer_manager(&env, &caller, &issuer)?;

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
            .set(&DataKey::ActiveRoot(issuer.clone()), &root);

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
        let active_version: u32 = env
            .storage()
            .persistent()
            .get(&DataKey::ActiveVersion(issuer.clone()))
            .unwrap_or(0);
        if active_version == 0 {
            return false;
        }

        let mut v = 1u32;
        while v <= active_version {
            if let Some(rec) = env
                .storage()
                .persistent()
                .get::<_, CredentialRootRecord>(&DataKey::RootRecord(issuer.clone(), v))
            {
                if rec.root == root {
                    return rec.revoked;
                }
            }
            v = v.saturating_add(1);
        }
        false
    }

    pub fn has_active_root(env: Env, issuer: Address) -> bool {
        env.storage().persistent().has(&DataKey::ActiveRoot(issuer))
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
    use soroban_sdk::testutils::Address as _;

    #[test]
    fn test_root_lifecycle() {
        let env = Env::default();
        env.mock_all_auths();

        let admin = Address::generate(&env);
        let issuer = Address::generate(&env);
        let contract_id = env.register_contract(None, CredentialRegistryContract);
        let client = CredentialRegistryContractClient::new(&env, &contract_id);
        client.initialize(&admin);

        let root_1 = BytesN::from_array(&env, &[11u8; 32]);
        let meta_1 = BytesN::from_array(&env, &[12u8; 32]);
        let v1 = client.set_credential_root(&admin, &issuer, &root_1, &meta_1);
        assert_eq!(v1, 1);
        assert_eq!(client.get_active_root(&issuer), Some(root_1.clone()));

        assert!(client.revoke_root(&admin, &issuer, &1));
        assert!(client.is_root_revoked(&issuer, &root_1));
    }
}
