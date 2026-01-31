use super::UpgradeError;
use soroban_sdk::Env;

pub trait Migratable {
    /// Function called after an upgrade to perform data migration
    fn migrate(env: &Env, from_version: u32) -> Result<(), UpgradeError>;
}

pub fn execute_migration<T: Migratable>(env: &Env, from_version: u32) -> Result<(), UpgradeError> {
    T::migrate(env, from_version)
}
