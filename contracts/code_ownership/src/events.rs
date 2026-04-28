use crate::types::{ModuleOwnership, ReviewRoute};
use soroban_sdk::{symbol_short, Address, Env};

pub fn publish_initialization(env: &Env, admin: &Address) {
    env.events().publish(
        (symbol_short!("OWNER"), symbol_short!("INIT")),
        admin,
    );
}

pub fn publish_module_registered(env: &Env, ownership: &ModuleOwnership) {
    env.events().publish(
        (symbol_short!("OWNER"), symbol_short!("REG")),
        (ownership.module_id.clone(), ownership.primary_owner.clone()),
    );
}

pub fn publish_ownership_updated(env: &Env, ownership: &ModuleOwnership) {
    env.events().publish(
        (symbol_short!("OWNER"), symbol_short!("UPD")),
        (ownership.module_id.clone(), ownership.primary_owner.clone()),
    );
}

pub fn publish_review_route_configured(env: &Env, route: &ReviewRoute) {
    env.events().publish(
        (symbol_short!("OWNER"), symbol_short!("ROUTE")),
        (route.module_id.clone(), route.required_reviewers),
    );
}
