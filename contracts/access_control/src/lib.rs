//! # AccessControl Module
//!
//! Implements a reusable access control inheritance pattern for Soroban contracts.
//! Resolves issue #434: eliminates duplicated access control logic across contracts
//! by providing a shared trait and storage helpers.
//!
//! ## Usage
//! Import and use `AccessControlImpl` in any contract to get consistent
//! `require_admin`, `require_role`, and `has_permission` behaviour.

#![no_std]

use soroban_sdk::{contracttype, symbol_short, Address, Env, Symbol};

// ── Storage keys ──────────────────────────────────────────────────────────────

#[derive(Clone, PartialEq, Eq)]
#[contracttype]
pub enum DataKey {
    Admin,
    Role(Address),
    Permission(Address, u32),
}

// ── Domain types ──────────────────────────────────────────────────────────────

/// Roles available across all contracts that use this module.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[contracttype]
#[repr(u32)]
pub enum Role {
    Admin = 0,
    Doctor = 1,
    Patient = 2,
    Staff = 3,
    Insurer = 4,
    Researcher = 5,
    Auditor = 6,
    Service = 7,
}

/// Fine-grained permissions that can be granted independently of roles.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[contracttype]
#[repr(u32)]
pub enum Permission {
    ManageUsers = 1,
    ManageSystem = 2,
    CreateRecord = 10,
    ReadRecord = 11,
    UpdateRecord = 12,
    DeleteRecord = 13,
    ReadConfidential = 20,
    GrantAccess = 30,
    RevokeAccess = 31,
}

/// Errors returned by access-control helpers.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[contracttype]
#[repr(u32)]
pub enum AccessError {
    Unauthorized = 1,
    NotInitialized = 2,
    AlreadyInitialized = 3,
    InvalidRole = 4,
}

// ── AccessControl trait ───────────────────────────────────────────────────────

/// Trait that any contract can implement to gain standardised access control.
///
/// The default implementations delegate to [`AccessControlImpl`], so a contract
/// only needs to call the associated functions – no boilerplate required.
pub trait AccessControl {
    /// Panics (via `require_auth`) unless the caller is the stored admin.
    fn require_admin(env: &Env) -> Result<(), AccessError> {
        AccessControlImpl::require_admin(env)
    }

    /// Panics unless the given address holds `role`.
    fn require_role(env: &Env, address: &Address, role: Role) -> Result<(), AccessError> {
        AccessControlImpl::require_role(env, address, role)
    }

    /// Returns `true` when `address` has been granted `permission`.
    fn has_permission(env: &Env, address: &Address, permission: Permission) -> bool {
        AccessControlImpl::has_permission(env, address, permission)
    }
}

// ── Concrete implementation ───────────────────────────────────────────────────

/// Stateless helper that reads/writes access-control state from contract storage.
pub struct AccessControlImpl;

impl AccessControlImpl {
    // ── Initialisation ────────────────────────────────────────────────────────

    /// Store the initial admin.  Must be called once during contract `initialize`.
    pub fn init(env: &Env, admin: &Address) {
        env.storage().instance().set(&DataKey::Admin, admin);
    }

    // ── Admin helpers ─────────────────────────────────────────────────────────

    /// Return the stored admin address.
    pub fn get_admin(env: &Env) -> Address {
        env.storage()
            .instance()
            .get::<DataKey, Address>(&DataKey::Admin)
            .expect("access_control: not initialised")
    }

    /// Require that the transaction was authorised by the admin.
    pub fn require_admin(env: &Env) -> Result<(), AccessError> {
        let admin = Self::get_admin(env);
        admin.require_auth();
        Ok(())
    }

    /// Transfer admin rights to a new address.
    pub fn transfer_admin(env: &Env, new_admin: &Address) -> Result<(), AccessError> {
        Self::require_admin(env)?;
        env.storage().instance().set(&DataKey::Admin, new_admin);
        env.events().publish(
            (symbol_short!("AC"), symbol_short!("ADMIN")),
            new_admin.clone(),
        );
        Ok(())
    }

    // ── Role helpers ──────────────────────────────────────────────────────────

    /// Assign `role` to `address`.  Caller must be admin.
    pub fn grant_role(env: &Env, address: &Address, role: Role) -> Result<(), AccessError> {
        Self::require_admin(env)?;
        env.storage()
            .persistent()
            .set(&DataKey::Role(address.clone()), &role);
        Self::emit_role_event(env, symbol_short!("GRANT"), address, role);
        Ok(())
    }

    /// Remove the role from `address`.  Caller must be admin.
    pub fn revoke_role(env: &Env, address: &Address) -> Result<(), AccessError> {
        Self::require_admin(env)?;
        env.storage()
            .persistent()
            .remove(&DataKey::Role(address.clone()));
        env.events().publish(
            (symbol_short!("AC"), symbol_short!("REVOKE")),
            address.clone(),
        );
        Ok(())
    }

    /// Return the role assigned to `address`, if any.
    pub fn get_role(env: &Env, address: &Address) -> Option<Role> {
        env.storage()
            .persistent()
            .get::<DataKey, Role>(&DataKey::Role(address.clone()))
    }

    /// Return `true` when `address` holds exactly `role`.
    pub fn has_role(env: &Env, address: &Address, role: Role) -> bool {
        Self::get_role(env, address)
            .map(|r| r == role)
            .unwrap_or(false)
    }

    /// Require that `address` holds `role`, otherwise return `Unauthorized`.
    pub fn require_role(env: &Env, address: &Address, role: Role) -> Result<(), AccessError> {
        if Self::has_role(env, address, role) {
            Ok(())
        } else {
            Err(AccessError::Unauthorized)
        }
    }

    // ── Permission helpers ────────────────────────────────────────────────────

    /// Grant a fine-grained `permission` to `address`.  Caller must be admin.
    pub fn grant_permission(
        env: &Env,
        address: &Address,
        permission: Permission,
    ) -> Result<(), AccessError> {
        Self::require_admin(env)?;
        env.storage().persistent().set(
            &DataKey::Permission(address.clone(), permission as u32),
            &true,
        );
        Ok(())
    }

    /// Revoke a fine-grained `permission` from `address`.  Caller must be admin.
    pub fn revoke_permission(
        env: &Env,
        address: &Address,
        permission: Permission,
    ) -> Result<(), AccessError> {
        Self::require_admin(env)?;
        env.storage()
            .persistent()
            .remove(&DataKey::Permission(address.clone(), permission as u32));
        Ok(())
    }

    /// Return `true` when `address` has been explicitly granted `permission`.
    pub fn has_permission(env: &Env, address: &Address, permission: Permission) -> bool {
        env.storage()
            .persistent()
            .get::<DataKey, bool>(&DataKey::Permission(address.clone(), permission as u32))
            .unwrap_or(false)
    }

    // ── Internal ──────────────────────────────────────────────────────────────

    fn emit_role_event(env: &Env, action: Symbol, address: &Address, role: Role) {
        env.events()
            .publish((symbol_short!("AC"), action), (address.clone(), role as u32));
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod test {
    use super::*;
    use soroban_sdk::{testutils::Address as _, Env};

    #[test]
    fn test_init_and_admin() {
        let env = Env::default();
        env.mock_all_auths();
        let admin = Address::generate(&env);

        env.as_contract(&env.register_contract(None, DummyContract), || {
            AccessControlImpl::init(&env, &admin);
            assert_eq!(AccessControlImpl::get_admin(&env), admin);
        });
    }

    #[test]
    fn test_grant_and_require_role() {
        let env = Env::default();
        env.mock_all_auths();
        let admin = Address::generate(&env);
        let doctor = Address::generate(&env);

        env.as_contract(&env.register_contract(None, DummyContract), || {
            AccessControlImpl::init(&env, &admin);
            AccessControlImpl::grant_role(&env, &doctor, Role::Doctor).unwrap();
            assert!(AccessControlImpl::has_role(&env, &doctor, Role::Doctor));
            assert!(AccessControlImpl::require_role(&env, &doctor, Role::Doctor).is_ok());
            assert_eq!(
                AccessControlImpl::require_role(&env, &doctor, Role::Admin),
                Err(AccessError::Unauthorized)
            );
        });
    }

    #[test]
    fn test_grant_and_has_permission() {
        let env = Env::default();
        env.mock_all_auths();
        let admin = Address::generate(&env);
        let user = Address::generate(&env);

        env.as_contract(&env.register_contract(None, DummyContract), || {
            AccessControlImpl::init(&env, &admin);
            AccessControlImpl::grant_permission(&env, &user, Permission::ReadRecord).unwrap();
            assert!(AccessControlImpl::has_permission(
                &env,
                &user,
                Permission::ReadRecord
            ));
            assert!(!AccessControlImpl::has_permission(
                &env,
                &user,
                Permission::DeleteRecord
            ));
        });
    }

    #[test]
    fn test_revoke_role() {
        let env = Env::default();
        env.mock_all_auths();
        let admin = Address::generate(&env);
        let staff = Address::generate(&env);

        env.as_contract(&env.register_contract(None, DummyContract), || {
            AccessControlImpl::init(&env, &admin);
            AccessControlImpl::grant_role(&env, &staff, Role::Staff).unwrap();
            assert!(AccessControlImpl::has_role(&env, &staff, Role::Staff));
            AccessControlImpl::revoke_role(&env, &staff).unwrap();
            assert!(!AccessControlImpl::has_role(&env, &staff, Role::Staff));
        });
    }

    // Minimal contract needed to provide a contract context for storage calls.
    use soroban_sdk::{contract, contractimpl};

    #[contract]
    struct DummyContract;

    #[contractimpl]
    impl DummyContract {}
}
