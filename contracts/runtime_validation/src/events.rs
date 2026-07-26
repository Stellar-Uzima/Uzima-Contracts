use crate::types::{
    InvariantCheck, PermissionCheck, ResourceTracker, StateConsistencyCheck, ValidationReport,
};
use soroban_sdk::{symbol_short, Address, Env, String};

pub fn publish_initialization(env: &Env, admin: &Address) {
    env.events()
        .publish((symbol_short!("VALID"), symbol_short!("INIT")), admin);
}

pub fn publish_invariant_registered(env: &Env, check: &InvariantCheck) {
    env.events().publish(
        (symbol_short!("VALID"), symbol_short!("INV_REG")),
        (check.check_id.clone(), check.severity),
    );
}

pub fn publish_state_check_registered(env: &Env, check: &StateConsistencyCheck) {
    env.events().publish(
        (symbol_short!("VALID"), symbol_short!("STATE_REG")),
        check.check_id.clone(),
    );
}

pub fn publish_permission_check_registered(env: &Env, check: &PermissionCheck) {
    env.events().publish(
        (symbol_short!("VALID"), symbol_short!("PERM_REG")),
        check.check_id.clone(),
    );
}

pub fn publish_resource_tracker_registered(env: &Env, tracker: &ResourceTracker) {
    env.events().publish(
        (symbol_short!("VALID"), symbol_short!("RES_REG")),
        (tracker.tracker_id.clone(), tracker.max_allocation),
    );
}

pub fn publish_violation_reported(env: &Env, report: &ValidationReport) {
    env.events().publish(
        (symbol_short!("VALID"), symbol_short!("VIOL")),
        (report.violation_id, report.check_id.clone()),
    );
}

pub fn publish_invariant_violation(env: &Env, check_id: &String, value: i128) {
    env.events().publish(
        (symbol_short!("VALID"), symbol_short!("INV_VIOL")),
        (check_id.clone(), value),
    );
}

pub fn publish_state_violation(env: &Env, check_id: &String, state: &String) {
    env.events().publish(
        (symbol_short!("VALID"), symbol_short!("STATE_V")),
        (check_id.clone(), state.clone()),
    );
}

pub fn publish_permission_violation(env: &Env, check_id: &String, role: &String) {
    env.events().publish(
        (symbol_short!("VALID"), symbol_short!("PERM_VIOL")),
        (check_id.clone(), role.clone()),
    );
}

pub fn publish_resource_updated(env: &Env, tracker: &ResourceTracker) {
    env.events().publish(
        (symbol_short!("VALID"), symbol_short!("RES_UPD")),
        (tracker.tracker_id.clone(), tracker.current_usage),
    );
}
