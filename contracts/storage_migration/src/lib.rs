#![no_std]
#![forbid(alloc)]
//! storage_migration - Storage migration helper for the Uzima Contracts
//! platform. Enables safe, audited migration of contract data between
//! storage key formats during contract upgrades (issue #968).

use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype, symbol_short, Address, Bytes, Env, Vec,
};

#[contracttype]
pub enum DataKey {
    Admin,
    Paused,
    MigrationLog,
    Data(Bytes),
}

#[contracttype]
#[derive(Clone)]
pub struct MigrationEntry {
    pub timestamp: u64,
    pub caller: Address,
    pub source_key: Bytes,
    pub dest_key: Bytes,
    pub items_migrated: u32,
}

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    NotInitialized = 1,
    AlreadyInitialized = 2,
    NotAuthorized = 3,
    Paused = 4,
    SourceEmpty = 5,
    DestNotEmpty = 6,
    VerificationFailed = 7,
}

#[contract]
pub struct StorageMigration;

#[contractimpl]
impl StorageMigration {
    pub fn initialize(env: Env, admin: Address) -> Result<(), Error> {
        admin.require_auth();
        if env.storage().instance().has(&DataKey::Admin) {
            return Err(Error::AlreadyInitialized);
        }
        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage().instance().set(&DataKey::Paused, &false);
        Ok(())
    }

    pub fn set_paused(env: Env, caller: Address, paused: bool) -> Result<(), Error> {
        caller.require_auth();
        Self::require_admin(&env, &caller)?;
        env.storage().instance().set(&DataKey::Paused, &paused);
        Ok(())
    }

    pub fn is_paused(env: Env) -> bool {
        env.storage()
            .instance()
            .get(&DataKey::Paused)
            .unwrap_or(false)
    }

    pub fn store(env: Env, caller: Address, key: Bytes, value: Bytes) -> Result<(), Error> {
        caller.require_auth();
        Self::require_initialized(&env)?;
        Self::require_not_paused(&env)?;
        env.storage().persistent().set(&DataKey::Data(key), &value);
        Ok(())
    }

    pub fn read(env: Env, key: Bytes) -> Option<Bytes> {
        env.storage()
            .persistent()
            .get(&DataKey::Data(key))
    }

    pub fn migrate(
        env: Env,
        caller: Address,
        source_key: Bytes,
        dest_key: Bytes,
        keep_source: bool,
    ) -> Result<u32, Error> {
        caller.require_auth();
        Self::require_initialized(&env)?;
        Self::require_not_paused(&env)?;

        if source_key == dest_key {
            return Err(Error::VerificationFailed);
        }

        let value: Bytes = env
            .storage()
            .persistent()
            .get(&DataKey::Data(source_key.clone()))
            .ok_or(Error::SourceEmpty)?;

        if env
            .storage()
            .persistent()
            .has(&DataKey::Data(dest_key.clone()))
        {
            return Err(Error::DestNotEmpty);
        }

        env.storage()
            .persistent()
            .set(&DataKey::Data(dest_key.clone()), &value);

        if !keep_source {
            env.storage()
                .persistent()
                .remove(&DataKey::Data(source_key.clone()));
        }

        let count = (value.len() as u32).max(1);
        Self::record_migration(&env, &caller, &source_key, &dest_key, count);

        env.events().publish(
            (symbol_short!("MIGRATE"),),
            (caller, source_key, dest_key, count, keep_source),
        );

        Ok(count)
    }

    pub fn migrate_and_verify(
        env: Env,
        caller: Address,
        source_key: Bytes,
        dest_key: Bytes,
        keep_source: bool,
    ) -> Result<u32, Error> {
        caller.require_auth();
        Self::require_initialized(&env)?;
        Self::require_not_paused(&env)?;

        if source_key == dest_key {
            return Err(Error::VerificationFailed);
        }

        let value: Bytes = env
            .storage()
            .persistent()
            .get(&DataKey::Data(source_key.clone()))
            .ok_or(Error::SourceEmpty)?;

        if env
            .storage()
            .persistent()
            .has(&DataKey::Data(dest_key.clone()))
        {
            return Err(Error::DestNotEmpty);
        }

        env.storage()
            .persistent()
            .set(&DataKey::Data(dest_key.clone()), &value);

        let written: Bytes = env
            .storage()
            .persistent()
            .get(&DataKey::Data(dest_key.clone()))
            .unwrap();

        if written != value {
            env.storage()
                .persistent()
                .remove(&DataKey::Data(dest_key));
            return Err(Error::VerificationFailed);
        }

        if !keep_source {
            env.storage()
                .persistent()
                .remove(&DataKey::Data(source_key.clone()));
        }

        let count = (value.len() as u32).max(1);
        Self::record_migration(&env, &caller, &source_key, &dest_key, count);

        env.events().publish(
            (symbol_short!("MIGRATE"),),
            (caller, source_key, dest_key, count, keep_source),
        );

        Ok(count)
    }

    pub fn get_migration_log(env: Env) -> Vec<MigrationEntry> {
        env.storage()
            .persistent()
            .get(&DataKey::MigrationLog)
            .unwrap_or(Vec::new(&env))
    }
}

impl StorageMigration {
    fn require_initialized(env: &Env) -> Result<(), Error> {
        if env.storage().instance().has(&DataKey::Admin) {
            Ok(())
        } else {
            Err(Error::NotInitialized)
        }
    }

    fn require_admin(env: &Env, caller: &Address) -> Result<(), Error> {
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

    fn require_not_paused(env: &Env) -> Result<(), Error> {
        if env
            .storage()
            .instance()
            .get(&DataKey::Paused)
            .unwrap_or(false)
        {
            Err(Error::Paused)
        } else {
            Ok(())
        }
    }

    fn record_migration(
        env: &Env,
        caller: &Address,
        source_key: &Bytes,
        dest_key: &Bytes,
        items_migrated: u32,
    ) {
        let entry = MigrationEntry {
            timestamp: env.ledger().timestamp(),
            caller: caller.clone(),
            source_key: source_key.clone(),
            dest_key: dest_key.clone(),
            items_migrated,
        };
        let mut log: Vec<MigrationEntry> = env
            .storage()
            .persistent()
            .get(&DataKey::MigrationLog)
            .unwrap_or(Vec::new(env));
        log.push_back(entry);
        env.storage().persistent().set(&DataKey::MigrationLog, &log);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use soroban_sdk::{testutils::Address as _, Env};

    fn setup() -> (Env, Address, Address) {
        let env = Env::default();
        env.mock_all_auths();
        let admin = Address::generate(&env);
        let contract_id = env.register_contract(None, StorageMigration);
        (env, admin, contract_id)
    }

    fn init<'a>(
        env: &'a Env,
        admin: &Address,
        contract_id: &'a Address,
    ) -> StorageMigrationClient<'a> {
        let client = StorageMigrationClient::new(env, contract_id);
        client.initialize(admin);
        client
    }

    #[test]
    fn test_initialize_sets_not_paused() {
        let (env, admin, contract_id) = setup();
        let client = init(&env, &admin, &contract_id);
        assert!(!client.is_paused());
    }

    #[test]
    fn test_double_initialize_fails() {
        let (env, admin, contract_id) = setup();
        let client = init(&env, &admin, &contract_id);
        assert_eq!(
            client.try_initialize(&admin),
            Err(Ok(Error::AlreadyInitialized))
        );
    }

    #[test]
    fn test_migration_before_init_fails() {
        let (env, admin, _contract_id) = setup();
        let (_, _, mig_id) = setup();
        let client = StorageMigrationClient::new(&env, &mig_id);
        let key1 = Bytes::from_array(&env, &[1]);
        let key2 = Bytes::from_array(&env, &[2]);
        assert_eq!(
            client.try_migrate(&admin, &key1, &key2, &false),
            Err(Ok(Error::NotInitialized))
        );
    }

    #[test]
    fn test_pause_blocks_migration() {
        let (env, admin, contract_id) = setup();
        let client = init(&env, &admin, &contract_id);
        client.set_paused(&admin, &true);
        let key1 = Bytes::from_array(&env, &[1]);
        let key2 = Bytes::from_array(&env, &[2]);
        assert_eq!(
            client.try_migrate(&admin, &key1, &key2, &false),
            Err(Ok(Error::Paused))
        );
    }

    #[test]
    fn test_store_and_read() {
        let (env, admin, contract_id) = setup();
        let client = init(&env, &admin, &contract_id);
        let key = Bytes::from_array(&env, &[1, 2, 3]);
        let value = Bytes::from_array(&env, &[10, 20, 30]);
        client.store(&admin, &key, &value);
        let result = client.read(&key);
        assert!(result.is_some());
        assert_eq!(result.unwrap(), value);
    }

    #[test]
    fn test_migrate_successful() {
        let (env, admin, contract_id) = setup();
        let client = init(&env, &admin, &contract_id);
        let src = Bytes::from_array(&env, &[1]);
        let dst = Bytes::from_array(&env, &[2]);
        let value = Bytes::from_array(&env, &[42]);
        client.store(&admin, &src, &value);
        let count = client.migrate(&admin, &src, &dst, &false);
        assert_eq!(count, 1);
        assert!(client.read(&src).is_none());
        assert!(client.read(&dst).is_some());
    }

    #[test]
    fn test_migrate_keep_source() {
        let (env, admin, contract_id) = setup();
        let client = init(&env, &admin, &contract_id);
        let src = Bytes::from_array(&env, &[1]);
        let dst = Bytes::from_array(&env, &[2]);
        let value = Bytes::from_array(&env, &[99]);
        client.store(&admin, &src, &value);
        let count = client.migrate(&admin, &src, &dst, &true);
        assert_eq!(count, 1);
        assert!(client.read(&src).is_some());
        assert!(client.read(&dst).is_some());
    }

    #[test]
    fn test_migrate_empty_source_fails() {
        let (env, admin, contract_id) = setup();
        let client = init(&env, &admin, &contract_id);
        let src = Bytes::from_array(&env, &[1]);
        let dst = Bytes::from_array(&env, &[2]);
        assert_eq!(
            client.try_migrate(&admin, &src, &dst, &false),
            Err(Ok(Error::SourceEmpty))
        );
    }

    #[test]
    fn test_migrate_same_key_fails() {
        let (env, admin, contract_id) = setup();
        let client = init(&env, &admin, &contract_id);
        let key = Bytes::from_array(&env, &[1]);
        assert_eq!(
            client.try_migrate(&admin, &key, &key, &false),
            Err(Ok(Error::VerificationFailed))
        );
    }

    #[test]
    fn test_migrate_dest_not_empty_fails() {
        let (env, admin, contract_id) = setup();
        let client = init(&env, &admin, &contract_id);
        let src = Bytes::from_array(&env, &[1]);
        let dst = Bytes::from_array(&env, &[2]);
        client.store(&admin, &src, &Bytes::from_array(&env, &[1]));
        client.store(&admin, &dst, &Bytes::from_array(&env, &[2]));
        assert_eq!(
            client.try_migrate(&admin, &src, &dst, &false),
            Err(Ok(Error::DestNotEmpty))
        );
    }

    #[test]
    fn test_migrate_and_verify_successful() {
        let (env, admin, contract_id) = setup();
        let client = init(&env, &admin, &contract_id);
        let src = Bytes::from_array(&env, &[1]);
        let dst = Bytes::from_array(&env, &[2]);
        let value = Bytes::from_array(&env, &[7, 8, 9]);
        client.store(&admin, &src, &value);
        let count = client.migrate_and_verify(&admin, &src, &dst, &false);
        assert_eq!(count, 1);
        assert!(client.read(&src).is_none());
        assert_eq!(client.read(&dst).unwrap(), value);
    }

    #[test]
    fn test_migration_log_records_entry() {
        let (env, admin, contract_id) = setup();
        let client = init(&env, &admin, &contract_id);
        let src = Bytes::from_array(&env, &[1]);
        let dst = Bytes::from_array(&env, &[2]);
        client.store(&admin, &src, &Bytes::from_array(&env, &[0]));
        client.migrate(&admin, &src, &dst, &false);
        let log = client.get_migration_log();
        assert_eq!(log.len(), 1);
    }

    #[test]
    fn test_migration_log_empty_initially() {
        let (env, _admin, contract_id) = setup();
        let client = StorageMigrationClient::new(&env, &contract_id);
        assert_eq!(client.get_migration_log().len(), 0);
    }
}
