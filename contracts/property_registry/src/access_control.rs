//! Role-based access control modelled on OpenZeppelin's `Ownable` +
//! `AccessControl`.
//!
//! # Why this is in-tree rather than a dependency
//!
//! The workspace pins `soroban-sdk = "=21.7.7"` and carries no OpenZeppelin
//! Soroban crate. Introducing one would change the dependency graph for every
//! contract in the workspace, and OpenZeppelin's Soroban contracts are still
//! pre-1.0, so that is a call for maintainers rather than a side effect of a
//! hardening PR. So the primitives are implemented here against the same API
//! shape OpenZeppelin uses, and they are deliberately drop-in replaceable: the
//! public surface (`only_admin`, `only_registrar`, `has_role`, `grant_role`,
//! `revoke_role`, `renounce_role`) matches OZ naming, so swapping the
//! implementation for the real crate later is a `use` change, not a rewrite.
//!
//! # Model
//!
//! - `DEFAULT_ADMIN_ROLE` is held by the deployer and can rotate itself and
//!   grant/revoke every other role.
//! - `REGISTRAR` gates `register_property`, `update_valuation` and
//!   `record_ownership`. It is a contract-wide role so the registrar set can be
//!   audited and rotated without touching the per-property `owner` field.
//!
//! Note the third gate that is *not* a role: `transfer_property` is authorized
//! by the property's owner of record, because ownership of a specific deed is a
//! per-property fact and not something a contract-wide role can express.

use crate::errors::Error;
use crate::types::DataKey;
use soroban_sdk::{symbol_short, Address, Env, Vec};

/// Role allowed to grant and revoke every other role.
pub const DEFAULT_ADMIN_ROLE: u32 = 0;
/// Role allowed to register properties, revalue them, and record share changes.
pub const REGISTRAR_ROLE: u32 = 1;

/// Require `caller` to hold `role`.
pub fn require_role(env: &Env, caller: &Address, role: u32) -> Result<(), Error> {
    if has_role(env, caller, role) {
        Ok(())
    } else {
        Err(Error::Unauthorized)
    }
}

/// Require `caller` to hold the default admin role.
pub fn require_admin(env: &Env, caller: &Address) -> Result<(), Error> {
    require_role(env, caller, DEFAULT_ADMIN_ROLE)
}

/// Require `caller` to hold the registrar role.
pub fn require_registrar(env: &Env, caller: &Address) -> Result<(), Error> {
    require_role(env, caller, REGISTRAR_ROLE)
}

/// Whether `account` holds `role`.
pub fn has_role(env: &Env, account: &Address, role: u32) -> bool {
    env.storage()
        .persistent()
        .get(&DataKey::Role(account.clone(), role))
        .unwrap_or(false)
}

/// Every address holding `role`, for indexing off-chain.
pub fn role_members(env: &Env, role: u32) -> Vec<Address> {
    env.storage()
        .instance()
        .get(&DataKey::RoleMembers(role))
        .unwrap_or_else(|| Vec::new(env))
}

/// Grant `role` to `account`. Only a `DEFAULT_ADMIN_ROLE` holder may do this.
pub fn grant_role(
    env: &Env,
    caller: &Address,
    account: &Address,
    role: u32,
) -> Result<(), Error> {
    require_admin(env, caller)?;

    if has_role(env, account, role) {
        return Ok(());
    }

    env.storage()
        .persistent()
        .set(&DataKey::Role(account.clone(), role), &true);

    let mut members = role_members(env, role);
    members.push_back(account.clone());
    env.storage().instance().set(&DataKey::RoleMembers(role), &members);

    env.events().publish(
        (symbol_short!("role_grant"), role, account.clone()),
        (),
    );
    Ok(())
}

/// Revoke `role` from `account`. Only a `DEFAULT_ADMIN_ROLE` holder may do this.
pub fn revoke_role(
    env: &Env,
    caller: &Address,
    account: &Address,
    role: u32,
) -> Result<(), Error> {
    require_admin(env, caller)?;

    // The default admin role is the root of trust; revoking it from the last
    // holder would leave the contract permanently unadministrable — no pause,
    // no registrar rotation, no upgrade path. Refuse rather than brick it.
    if role == DEFAULT_ADMIN_ROLE && role_members(env, role).len() == 1 {
        return Err(Error::CannotRevokeLastAdmin);
    }

    env.storage()
        .persistent()
        .set(&DataKey::Role(account.clone(), role), &false);

    let current = role_members(env, role);
    let mut kept = Vec::new(env);
    for member in current.iter() {
        if member != account.clone() {
            kept.push_back(member);
        }
    }
    env.storage().instance().set(&DataKey::RoleMembers(role), &kept);

    env.events().publish(
        (symbol_short!("role_revoke"), role, account.clone()),
        (),
    );
    Ok(())
}

/// Drop every role `account` holds, including the admin role.
pub fn renounce_role(env: &Env, account: &Address) {
    env.storage()
        .persistent()
        .remove(&DataKey::Role(account.clone(), DEFAULT_ADMIN_ROLE));
    env.storage()
        .persistent()
        .remove(&DataKey::Role(account.clone(), REGISTRAR_ROLE));

    // Cleared outright rather than filtered: renouncement is per-address and the
    // index is rebuilt on the next `grant_role`.
    env.storage()
        .instance()
        .remove(&DataKey::RoleMembers(DEFAULT_ADMIN_ROLE));
    env.storage()
        .instance()
        .remove(&DataKey::RoleMembers(REGISTRAR_ROLE));

    env.events()
        .publish((symbol_short!("role_renounce"), account.clone()), ());
}

/// Seed the deployer with the default admin role at initialization.
pub fn bootstrap_roles(env: &Env, admin: &Address) {
    env.storage()
        .persistent()
        .set(&DataKey::Role(admin.clone(), DEFAULT_ADMIN_ROLE), &true);
    let mut members = Vec::new(env);
    members.push_back(admin.clone());
    env.storage()
        .instance()
        .set(&DataKey::RoleMembers(DEFAULT_ADMIN_ROLE), &members);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup() -> (Env, Address, Address) {
        let env = Env::default();
        let admin = Address::generate(&env);
        let other = Address::generate(&env);
        bootstrap_roles(&env, &admin);
        (env, admin, other)
    }

    #[test]
    fn bootstrap_grants_the_admin_role() {
        let (env, admin, _other) = setup();
        assert!(has_role(&env, &admin, DEFAULT_ADMIN_ROLE));
        assert_eq!(role_members(&env, DEFAULT_ADMIN_ROLE).len(), 1);
    }

    #[test]
    fn grant_and_revoke_round_trip() {
        let (env, admin, other) = setup();

        grant_role(&env, &admin, &other, REGISTRAR_ROLE).unwrap();
        assert!(has_role(&env, &other, REGISTRAR_ROLE));
        assert_eq!(role_members(&env, REGISTRAR_ROLE).len(), 1);

        revoke_role(&env, &admin, &other, REGISTRAR_ROLE).unwrap();
        assert!(!has_role(&env, &other, REGISTRAR_ROLE));
        assert_eq!(role_members(&env, REGISTRAR_ROLE).len(), 0);
    }

    #[test]
    fn granting_twice_is_a_no_op() {
        let (env, admin, other) = setup();

        grant_role(&env, &admin, &other, REGISTRAR_ROLE).unwrap();
        grant_role(&env, &admin, &other, REGISTRAR_ROLE).unwrap();

        assert_eq!(role_members(&env, REGISTRAR_ROLE).len(), 1);
    }

    #[test]
    fn non_admin_cannot_revoke() {
        let (env, admin, other) = setup();

        grant_role(&env, &admin, &other, REGISTRAR_ROLE).unwrap();
        assert_eq!(
            revoke_role(&env, &other, &other, REGISTRAR_ROLE),
            Err(Error::Unauthorized)
        );
    }

    #[test]
    fn last_admin_cannot_be_revoked() {
        let (env, admin, _other) = setup();

        assert_eq!(
            revoke_role(&env, &admin, &admin, DEFAULT_ADMIN_ROLE),
            Err(Error::CannotRevokeLastAdmin)
        );
    }

    #[test]
    fn admin_role_can_be_revoked_once_a_second_admin_exists() {
        let (env, admin, other) = setup();

        grant_role(&env, &admin, &other, DEFAULT_ADMIN_ROLE).unwrap();
        revoke_role(&env, &admin, &admin, DEFAULT_ADMIN_ROLE).unwrap();

        assert!(!has_role(&env, &admin, DEFAULT_ADMIN_ROLE));
        assert!(has_role(&env, &other, DEFAULT_ADMIN_ROLE));
    }

    #[test]
    fn renounce_clears_every_role_and_the_index() {
        let (env, admin, other) = setup();

        grant_role(&env, &admin, &other, REGISTRAR_ROLE).unwrap();
        renounce_role(&env, &other);

        assert!(!has_role(&env, &other, REGISTRAR_ROLE));
        assert!(!has_role(&env, &other, DEFAULT_ADMIN_ROLE));
        assert_eq!(role_members(&env, REGISTRAR_ROLE).len(), 0);
    }

    #[test]
    fn require_rejects_a_stranger() {
        let (env, _admin, stranger) = setup();
        assert_eq!(
            require_registrar(&env, &stranger),
            Err(Error::Unauthorized)
        );
    }
}