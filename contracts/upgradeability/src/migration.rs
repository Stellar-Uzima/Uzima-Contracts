use super::UpgradeError;
use soroban_sdk::{contracttype, symbol_short, BytesN, Env, Symbol, Vec};

#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub struct UpgradeValidation {
    pub state_compatible: bool,
    pub api_compatible: bool,
    pub storage_layout_valid: bool,
    pub tests_passed: bool,
    pub gas_impact: i128,
    pub report: Vec<Symbol>,
}

pub trait Migratable {
    /// Function called after an upgrade to perform data migration
    fn migrate(env: &Env, from_version: u32) -> Result<(), UpgradeError>;

    /// Function called to verify state integrity (pre and post migration)
    fn verify_integrity(env: &Env) -> Result<BytesN<32>, UpgradeError>;

    /// Function called to validate the upgrade safety before execution
    fn validate(_env: &Env, _new_wasm_hash: &BytesN<32>) -> Result<UpgradeValidation, UpgradeError> {
        Ok(UpgradeValidation {
            state_compatible: true,
            api_compatible: true,
            storage_layout_valid: true,
            tests_passed: true,
            gas_impact: 0,
            report: Vec::new(_env),
        })
    }
}

pub fn execute_migration<T: Migratable>(env: &Env, from_version: u32) -> Result<(), UpgradeError> {
    T::migrate(env, from_version)?;
    T::verify_integrity(env).map(|_| ())
}
