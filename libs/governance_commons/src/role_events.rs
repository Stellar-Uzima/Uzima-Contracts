//! Canonical role-change events (issue #1950).
//!
//! Monitoring consumes the schemas in `schemas/events/role_granted_event.schema.json`
//! and `role_revoked_event.schema.json`. Every privileged role grant or revoke
//! must emit one of these events so indexers do not have to understand
//! contract-specific event shapes.

use soroban_sdk::{contracttype, Address, Env, String, Symbol};

#[derive(Clone)]
#[contracttype]
pub struct RoleGrantedEvent {
    pub grantee: Address,
    pub role: String,
    pub granted_by: Address,
    pub ledger: u64,
}

#[derive(Clone)]
#[contracttype]
pub struct RoleRevokedEvent {
    pub revokee: Address,
    pub role: String,
    pub revoked_by: Address,
    pub ledger: u64,
}

pub fn emit_role_granted(env: &Env, grantee: &Address, role: &String, granted_by: &Address) {
    let event = RoleGrantedEvent {
        grantee: grantee.clone(),
        role: role.clone(),
        granted_by: granted_by.clone(),
        ledger: env.ledger().sequence(),
    };
    env.events().publish(
        (Symbol::new(env, "role_granted"), env.current_contract_address()),
        event,
    );
}

pub fn emit_role_revoked(env: &Env, revokee: &Address, role: &String, revoked_by: &Address) {
    let event = RoleRevokedEvent {
        revokee: revokee.clone(),
        role: role.clone(),
        revoked_by: revoked_by.clone(),
        ledger: env.ledger().sequence(),
    };
    env.events().publish(
        (Symbol::new(env, "role_revoked"), env.current_contract_address()),
        event,
    );
}
