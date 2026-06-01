//! # Credential Notifications Contract
//!
//! ## Access Control Model
//!
//! - **Admin**: Set at initialization. Can add/remove authorized notifiers and transfer admin role.
//! - **Authorized Notifiers**: Addresses explicitly granted permission to send credential notifications.
//!   Only notifiers (or the admin) may call `send_notification`.
//! - **Unauthorized callers**: All other callers are rejected with `Error::Unauthorized`.
#![no_std]

use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype, symbol_short, Address, Env, String,
    Symbol,
};

// ── Storage keys ──────────────────────────────────────────────────────────────

const ADMIN: Symbol = symbol_short!("ADMIN");

#[contracttype]
pub enum DataKey {
    Notifier(Address), // authorized notifier -> bool
}

// ── Errors ────────────────────────────────────────────────────────────────────

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    AlreadyInitialized = 1,
    NotInitialized = 2,
    Unauthorized = 3,
    NotifierNotFound = 4,
}

// ── Contract ──────────────────────────────────────────────────────────────────

#[contract]
pub struct CredentialNotificationsContract;

#[contractimpl]
impl CredentialNotificationsContract {
    /// Initialize the contract with an admin address.
    /// Can only be called once.
    pub fn initialize(env: Env, admin: Address) -> Result<(), Error> {
        admin.require_auth();
        if env.storage().instance().has(&ADMIN) {
            return Err(Error::AlreadyInitialized);
        }
        env.storage().instance().set(&ADMIN, &admin);
        env.events()
            .publish((symbol_short!("CRED"), symbol_short!("INIT")), &admin);
        Ok(())
    }

    /// Grant notification permission to an address. Admin only.
    pub fn add_notifier(env: Env, caller: Address, notifier: Address) -> Result<(), Error> {
        caller.require_auth();
        Self::require_admin(&env, &caller)?;
        env.storage()
            .persistent()
            .set(&DataKey::Notifier(notifier.clone()), &true);
        env.events().publish(
            (symbol_short!("CRED"), symbol_short!("ADD_NTF")),
            &notifier,
        );
        Ok(())
    }

    /// Revoke notification permission from an address. Admin only.
    pub fn remove_notifier(env: Env, caller: Address, notifier: Address) -> Result<(), Error> {
        caller.require_auth();
        Self::require_admin(&env, &caller)?;
        if !env
            .storage()
            .persistent()
            .has(&DataKey::Notifier(notifier.clone()))
        {
            return Err(Error::NotifierNotFound);
        }
        env.storage()
            .persistent()
            .remove(&DataKey::Notifier(notifier.clone()));
        env.events().publish(
            (symbol_short!("CRED"), symbol_short!("RM_NTF")),
            &notifier,
        );
        Ok(())
    }

    /// Send a credential notification. Only authorized notifiers or admin may call this.
    pub fn send_notification(
        env: Env,
        caller: Address,
        recipient: Address,
        credential_id: String,
        message: String,
    ) -> Result<(), Error> {
        caller.require_auth();
        Self::require_notifier_or_admin(&env, &caller)?;
        env.events().publish(
            (symbol_short!("CRED"), symbol_short!("NOTIFY")),
            (caller, recipient, credential_id, message),
        );
        Ok(())
    }

    /// Check whether an address is an authorized notifier.
    pub fn is_notifier(env: Env, notifier: Address) -> bool {
        env.storage()
            .persistent()
            .get(&DataKey::Notifier(notifier))
            .unwrap_or(false)
    }

    /// Return the current admin address.
    pub fn get_admin(env: Env) -> Result<Address, Error> {
        env.storage()
            .instance()
            .get(&ADMIN)
            .ok_or(Error::NotInitialized)
    }

    // ── Internal helpers ──────────────────────────────────────────────────────

    fn require_admin(env: &Env, caller: &Address) -> Result<(), Error> {
        let admin: Address = env
            .storage()
            .instance()
            .get(&ADMIN)
            .ok_or(Error::NotInitialized)?;
        if &admin != caller {
            return Err(Error::Unauthorized);
        }
        Ok(())
    }

    fn require_notifier_or_admin(env: &Env, caller: &Address) -> Result<(), Error> {
        if Self::require_admin(env, caller).is_ok() {
            return Ok(());
        }
        let authorized: bool = env
            .storage()
            .persistent()
            .get(&DataKey::Notifier(caller.clone()))
            .unwrap_or(false);
        if !authorized {
            return Err(Error::Unauthorized);
        }
        Ok(())
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use soroban_sdk::testutils::Address as _;
    use soroban_sdk::{Address, Env, String};

    fn setup() -> (Env, Address, soroban_sdk::Address) {
        let env = Env::default();
        let contract_id = env.register_contract(None, CredentialNotificationsContract);
        let client = CredentialNotificationsContractClient::new(&env, &contract_id);
        let admin = Address::generate(&env);
        client.mock_all_auths().initialize(&admin);
        (env, contract_id, admin)
    }

    #[test]
    fn test_initialize_sets_admin() {
        let (env, contract_id, admin) = setup();
        let client = CredentialNotificationsContractClient::new(&env, &contract_id);
        assert_eq!(client.get_admin(), admin);
    }

    #[test]
    fn test_double_initialize_fails() {
        let (env, contract_id, admin) = setup();
        let client = CredentialNotificationsContractClient::new(&env, &contract_id);
        let result = client.mock_all_auths().try_initialize(&admin);
        assert!(result.is_err());
    }

    #[test]
    fn test_add_and_remove_notifier() {
        let (env, contract_id, admin) = setup();
        let client = CredentialNotificationsContractClient::new(&env, &contract_id);
        let notifier = Address::generate(&env);

        assert!(!client.is_notifier(&notifier));
        client.mock_all_auths().add_notifier(&admin, &notifier);
        assert!(client.is_notifier(&notifier));
        client.mock_all_auths().remove_notifier(&admin, &notifier);
        assert!(!client.is_notifier(&notifier));
    }

    #[test]
    fn test_unauthorized_cannot_add_notifier() {
        let (env, contract_id, _admin) = setup();
        let client = CredentialNotificationsContractClient::new(&env, &contract_id);
        let attacker = Address::generate(&env);
        let result = client
            .mock_all_auths()
            .try_add_notifier(&attacker, &attacker);
        assert!(result.is_err());
    }

    #[test]
    fn test_authorized_notifier_can_send_notification() {
        let (env, contract_id, admin) = setup();
        let client = CredentialNotificationsContractClient::new(&env, &contract_id);
        let notifier = Address::generate(&env);
        let recipient = Address::generate(&env);

        client.mock_all_auths().add_notifier(&admin, &notifier);
        let result = client.mock_all_auths().try_send_notification(
            &notifier,
            &recipient,
            &String::from_str(&env, "CRED-001"),
            &String::from_str(&env, "Your credential is ready"),
        );
        assert!(result.is_ok());
    }

    #[test]
    fn test_unauthorized_cannot_send_notification() {
        let (env, contract_id, _admin) = setup();
        let client = CredentialNotificationsContractClient::new(&env, &contract_id);
        let attacker = Address::generate(&env);
        let recipient = Address::generate(&env);

        let result = client.mock_all_auths().try_send_notification(
            &attacker,
            &recipient,
            &String::from_str(&env, "CRED-001"),
            &String::from_str(&env, "Forged notification"),
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_admin_can_send_notification_directly() {
        let (env, contract_id, admin) = setup();
        let client = CredentialNotificationsContractClient::new(&env, &contract_id);
        let recipient = Address::generate(&env);

        let result = client.mock_all_auths().try_send_notification(
            &admin,
            &recipient,
            &String::from_str(&env, "CRED-002"),
            &String::from_str(&env, "Admin notification"),
        );
        assert!(result.is_ok());
    }
}
