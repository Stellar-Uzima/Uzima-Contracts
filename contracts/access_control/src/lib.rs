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
#![forbid(alloc)]

use soroban_sdk::{contracttype, symbol_short, Address, Env, Symbol};

// ── Storage layout constants ──────────────────────────────────────────────────

/// Number of reserved storage-key slots kept in [`DataKey`] for future upgrades.
///
/// New storage keys introduced by a future contract version must consume a
/// reserved slot (via [`DataKey::Reserved`]) rather than inserting variants in
/// front of existing ones.  This preserves the on-disk encoding of every key
/// already written by the current version, so upgraded code continues to read
/// and write the same data.
pub const STORAGE_GAP: u32 = 16;

/// Storage-layout version stamped into instance storage at initialisation.
///
/// Increment this constant when the [`DataKey`] enum changes shape.  Future
/// migration code can compare the stored value against `STORAGE_VERSION` to
/// decide whether a migration pass is required after an upgrade.
pub const STORAGE_VERSION: u32 = 1;

// ── Storage keys ──────────────────────────────────────────────────────────────

/// Storage keys used by the access-control module.
///
/// Each variant maps to a distinct piece of on-chain state:
/// the admin pointer, a per-address role assignment, or a
/// per-address/per-permission grant flag.
#[derive(Clone, PartialEq, Eq)]
#[contracttype]
pub enum DataKey {
    /// Instance-storage key holding the current admin [`Address`].
    Admin,
    /// Persistent-storage key holding the [`Role`] assigned to an address.
    Role(Address),
    /// Persistent-storage key holding a boolean flag for a fine-grained
    /// [`Permission`] granted to an address.  The `u32` is the discriminant
    /// of the permission variant.
    Permission(Address, u32),
    /// Version marker written once by [`AccessControlImpl::init`].
    ///
    /// Stores a `u32` equal to [`STORAGE_VERSION`].
    StorageVersion,
    /// Reserved storage slots for future upgrades.
    ///
    /// Each index in `0..STORAGE_GAP` is unused by the current version.
    /// A future upgrade may assign meaning to a slot by matching a specific
    /// index (e.g. `DataKey::Reserved(0)`); existing slots must never be
    /// renumbered or reused for a different purpose.
    Reserved(u32),
}

// ── Domain types ──────────────────────────────────────────────────────────────

/// Roles available across all contracts that use this module.
///
/// Roles are stored as a single value per address; assigning a new role
/// overwrites the previous one.  Use fine-grained [`Permission`]s when an
/// address needs multiple independent capabilities.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[contracttype]
#[repr(u32)]
pub enum Role {
    /// Full administrative control, including role and permission management.
    Admin = 0,
    /// A healthcare provider authorised to create and read medical records.
    Doctor = 1,
    /// A data subject whose own records may be accessed.
    Patient = 2,
    /// General hospital staff with limited record access.
    Staff = 3,
    /// An insurance provider authorised for billing-related reads.
    Insurer = 4,
    /// A researcher authorised for anonymised data access.
    Researcher = 5,
    /// A read-only auditor authorised for compliance checks.
    Auditor = 6,
    /// A machine-to-machine service account.
    Service = 7,
}

/// Fine-grained permissions that can be granted independently of roles.
///
/// Permissions are additive flags stored per address; an address may hold
/// any number of them simultaneously.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[contracttype]
#[repr(u32)]
pub enum Permission {
    /// Create, update, and remove user accounts.
    ManageUsers = 1,
    /// Perform system-level administration tasks.
    ManageSystem = 2,
    /// Create a new medical record.
    CreateRecord = 10,
    /// Read the contents of a medical record.
    ReadRecord = 11,
    /// Update an existing medical record.
    UpdateRecord = 12,
    /// Delete a medical record.
    DeleteRecord = 13,
    /// Read records classified as confidential.
    ReadConfidential = 20,
    /// Grant record access to another address.
    GrantAccess = 30,
    /// Revoke previously granted record access.
    RevokeAccess = 31,
}

/// Errors returned by access-control helpers.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[contracttype]
#[repr(u32)]
pub enum AccessError {
    /// The caller is not authorised to perform the operation.
    Unauthorized = 1,
    /// The module has not been initialised with an admin.
    NotInitialized = 2,
    /// Initialisation was attempted more than once.
    AlreadyInitialized = 3,
    /// The supplied role value does not map to a known variant.
    InvalidRole = 4,
}

impl core::fmt::Display for AccessError {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        match self {
            AccessError::Unauthorized => write!(f, "unauthorized"),
            AccessError::NotInitialized => write!(f, "not initialized"),
            AccessError::AlreadyInitialized => write!(f, "already initialized"),
            AccessError::InvalidRole => write!(f, "invalid role"),
        }
    }
}

// ── AccessControl trait ───────────────────────────────────────────────────────

/// Trait that any contract can implement to gain standardised access control.
///
/// The default implementations delegate to [`AccessControlImpl`], so a contract
/// only needs to call the associated functions – no boilerplate required.
pub trait AccessControl {
    /// Panics (via `require_auth`) unless the caller is the stored admin.
    #[must_use]
    fn require_admin(env: &Env) -> Result<(), AccessError> {
        AccessControlImpl::require_admin(env)
    }

    /// Panics unless the given address holds `role`.
    #[must_use]
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
///
/// All methods take an [`Env`] reference and operate on the calling contract's
/// storage.  Call [`AccessControlImpl::init`] exactly once during contract
/// initialisation before using any other method.
pub struct AccessControlImpl;

impl AccessControlImpl {
    // ── Initialisation ────────────────────────────────────────────────────────

    /// Store the initial admin.  Must be called once during contract `initialize`.
    ///
    /// Also stamps [`STORAGE_VERSION`] into instance storage so that future
    /// upgrade migrations can detect the storage layout they are running against.
    ///
    /// # Errors
    /// Fails if the module has already been initialised.
    pub fn init(env: &Env, admin: &Address) {
        governance_commons::init_guard(env);
        env.storage().instance().set(&DataKey::Admin, admin);
        env.storage()
            .instance()
            .set(&DataKey::StorageVersion, &STORAGE_VERSION);
    }

    /// Return the storage-layout version stamped at initialisation.
    ///
    /// Returns `0` when no version marker is present (i.e. the contract was
    /// initialised by a version that predates the storage-gap change).
    pub fn storage_version(env: &Env) -> u32 {
        env.storage()
            .instance()
            .get::<DataKey, u32>(&DataKey::StorageVersion)
            .unwrap_or(0)
    }

    // ── Admin helpers ─────────────────────────────────────────────────────────

    /// Return the stored admin address.
    ///
    /// # Panics
    /// Panics when the module has not been initialised.
    pub fn get_admin(env: &Env) -> Address {
        env.storage()
            .instance()
            .get::<DataKey, Address>(&DataKey::Admin)
            .expect("access_control: not initialised")
    }

    /// Require that the transaction was authorised by the admin.
    ///
    /// # Errors
    /// Returns [`AccessError::Unauthorized`] when `require_auth` fails.
    #[must_use]
    pub fn require_admin(env: &Env) -> Result<(), AccessError> {
        let admin = Self::get_admin(env);
        admin.require_auth();
        Ok(())
    }

    /// Transfer admin rights to a new address.
    ///
    /// Emits an `AC ADMIN` event carrying the new admin address.
    ///
    /// # Errors
    /// Returns [`AccessError::Unauthorized`] when the caller is not the
    /// current admin.
    #[must_use]
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
    ///
    /// Overwrites any previously assigned role and emits an `AC GRANT` event.
    ///
    /// # Errors
    /// Returns [`AccessError::Unauthorized`] when the caller is not the admin.
    #[must_use]
    pub fn grant_role(env: &Env, address: &Address, role: Role) -> Result<(), AccessError> {
        Self::require_admin(env)?;
        env.storage()
            .persistent()
            .set(&DataKey::Role(address.clone()), &role);
        Self::emit_role_event(env, symbol_short!("GRANT"), address, role);
        Ok(())
    }

    /// Remove the role from `address`.  Caller must be admin.
    ///
    /// Emits an `AC REVOKE` event.
    ///
    /// # Errors
    /// Returns [`AccessError::Unauthorized`] when the caller is not the admin.
    #[must_use]
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
    ///
    /// # Errors
    /// Returns [`AccessError::Unauthorized`] when `address` does not hold
    /// the requested role.
    #[must_use]
    pub fn require_role(env: &Env, address: &Address, role: Role) -> Result<(), AccessError> {
        if Self::has_role(env, address, role) {
            Ok(())
        } else {
            Err(AccessError::Unauthorized)
        }
    }

    // ── Permission helpers ────────────────────────────────────────────────────

    /// Grant a fine-grained `permission` to `address`.  Caller must be admin.
    ///
    /// # Errors
    /// Returns [`AccessError::Unauthorized`] when the caller is not the admin.
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
    ///
    /// # Errors
    /// Returns [`AccessError::Unauthorized`] when the caller is not the admin.
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
        env.events().publish(
            (symbol_short!("AC"), action),
            (address.clone(), role as u32),
        );
    }

    // ── Batch Operations (Issue #1164) ───────────────────────────────────

    /// Batch grant roles to multiple addresses.
    /// Caller must be admin. Returns the count of successfully granted roles.
    ///
    /// # Errors
    /// Returns [`AccessError::Unauthorized`] when the caller is not the admin.
    pub fn batch_grant_roles(
        env: &Env,
        assignments: Vec<(Address, Role)>,
    ) -> Result<u32, AccessError> {
        Self::require_admin(env)?;
        let mut granted: u32 = 0;
        for i in 0..assignments.len() {
            if let Some((address, role)) = assignments.get(i) {
                env.storage()
                    .persistent()
                    .set(&DataKey::Role(address.clone()), &role);
                Self::emit_role_event(env, symbol_short!("GRANT"), &address, role);
                granted = granted.saturating_add(1);
            }
        }
        Ok(granted)
    }

    /// Batch revoke roles from multiple addresses.
    /// Caller must be admin. Returns the count of successfully revoked roles.
    ///
    /// # Errors
    /// Returns [`AccessError::Unauthorized`] when the caller is not the admin.
    pub fn batch_revoke_roles(
        env: &Env,
        addresses: Vec<Address>,
    ) -> Result<u32, AccessError> {
        Self::require_admin(env)?;
        let mut revoked: u32 = 0;
        for i in 0..addresses.len() {
            if let Some(address) = addresses.get(i) {
                env.storage()
                    .persistent()
                    .remove(&DataKey::Role(address.clone()));
                env.events().publish(
                    (symbol_short!("AC"), symbol_short!("REVOKE")),
                    address.clone(),
                );
                revoked = revoked.saturating_add(1);
            }
        }
        Ok(revoked)
    }

    /// Batch grant permissions to an address.
    /// Caller must be admin. Returns the count of permissions granted.
    ///
    /// # Errors
    /// Returns [`AccessError::Unauthorized`] when the caller is not the admin.
    pub fn batch_grant_permissions(
        env: &Env,
        address: Address,
        permissions: Vec<Permission>,
    ) -> Result<u32, AccessError> {
        Self::require_admin(env)?;
        let mut granted: u32 = 0;
        for i in 0..permissions.len() {
            if let Some(permission) = permissions.get(i) {
                env.storage().persistent().set(
                    &DataKey::Permission(address.clone(), permission as u32),
                    &true,
                );
                granted = granted.saturating_add(1);
            }
        }
        env.events().publish(
            (symbol_short!("AC"), symbol_short!("BATCH_PERM")),
            (address, granted),
        );
        Ok(granted)
    }

    /// Batch revoke permissions from an address.
    /// Caller must be admin. Returns the count of permissions revoked.
    ///
    /// # Errors
    /// Returns [`AccessError::Unauthorized`] when the caller is not the admin.
    pub fn batch_revoke_permissions(
        env: &Env,
        address: Address,
        permissions: Vec<Permission>,
    ) -> Result<u32, AccessError> {
        Self::require_admin(env)?;
        let mut revoked: u32 = 0;
        for i in 0..permissions.len() {
            if let Some(permission) = permissions.get(i) {
                env.storage()
                    .persistent()
                    .remove(&DataKey::Permission(address.clone(), permission as u32));
                revoked = revoked.saturating_add(1);
            }
        }
        env.events().publish(
            (symbol_short!("AC"), symbol_short!("BATCH_REV")),
            (address, revoked),
        );
        Ok(revoked)
    }

    /// Batch grant a single permission to multiple addresses.
    /// Caller must be admin. Returns the count of addresses granted.
    ///
    /// # Errors
    /// Returns [`AccessError::Unauthorized`] when the caller is not the admin.
    pub fn batch_grant_permission_to_many(
        env: &Env,
        permission: Permission,
        addresses: Vec<Address>,
    ) -> Result<u32, AccessError> {
        Self::require_admin(env)?;
        let mut granted: u32 = 0;
        for i in 0..addresses.len() {
            if let Some(address) = addresses.get(i) {
                env.storage().persistent().set(
                    &DataKey::Permission(address.clone(), permission as u32),
                    &true,
                );
                granted = granted.saturating_add(1);
            }
        }
        env.events().publish(
            (symbol_short!("AC"), symbol_short!("BATCH_G")),
            (permission as u32, granted),
        );
        Ok(granted)
    }

    /// Batch revoke a single permission from multiple addresses.
    /// Caller must be admin. Returns the count of addresses revoked.
    ///
    /// # Errors
    /// Returns [`AccessError::Unauthorized`] when the caller is not the admin.
    pub fn batch_revoke_permission_from_many(
        env: &Env,
        permission: Permission,
        addresses: Vec<Address>,
    ) -> Result<u32, AccessError> {
        Self::require_admin(env)?;
        let mut revoked: u32 = 0;
        for i in 0..addresses.len() {
            if let Some(address) = addresses.get(i) {
                env.storage()
                    .persistent()
                    .remove(&DataKey::Permission(address.clone(), permission as u32));
                revoked = revoked.saturating_add(1);
            }
        }
        env.events().publish(
            (symbol_short!("AC"), symbol_short!("BATCH_R")),
            (permission as u32, revoked),
        );
        Ok(revoked)
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

        let contract_id = env.register_contract(None, DummyContract);

        // Split each mutating operation into its own as_contract block
        // to avoid Soroban v21 auth frame conflicts with mock_all_auths().
        env.as_contract(&contract_id, || {
            AccessControlImpl::init(&env, &admin);
            AccessControlImpl::grant_role(&env, &staff, Role::Staff).unwrap();
        });

        env.as_contract(&contract_id, || {
            assert!(AccessControlImpl::has_role(&env, &staff, Role::Staff));
        });

        env.as_contract(&contract_id, || {
            AccessControlImpl::revoke_role(&env, &staff).unwrap();
        });

        env.as_contract(&contract_id, || {
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
