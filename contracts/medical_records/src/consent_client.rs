//! Cross-contract interface for checking patient consent.
//!
//! Keep this interface local instead of linking the implementation crate:
//! Soroban contract crates export the same WASM entry-point symbols and cannot
//! safely be linked together into one contract binary.

use soroban_sdk::{contractclient, Address, Env};

#[contractclient(name = "PatientConsentManagementClient")]
pub trait PatientConsentManagement {
    fn check_consent(env: Env, patient: Address, provider: Address) -> bool;
}
