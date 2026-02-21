use super::UpgradeError;
use soroban_sdk::{BytesN, Env};

pub trait Migratable {
    /// Function called after an upgrade to perform data migration
    fn migrate(env: &Env, from_version: u32) -> Result<(), UpgradeError>;

    /// Function called to verify state integrity (pre and post migration)
    fn verify_integrity(env: &Env) -> Result<BytesN<32>, UpgradeError>;
}

pub fn execute_migration<T: Migratable>(env: &Env, from_version: u32) -> Result<(), UpgradeError> {
    T::migrate(env, from_version)?;
    T::verify_integrity(env).map(|_| ())
}
