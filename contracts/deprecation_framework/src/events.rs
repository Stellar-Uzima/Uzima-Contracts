use crate::types::{DeprecationStatus, MigrationGuide, SunsetTimeline};
use soroban_sdk::{symbol_short, Address, Env, String};

pub fn publish_initialization(env: &Env, admin: &Address) {
    env.events()
        .publish((symbol_short!("DEPREC"), symbol_short!("INIT")), admin);
}

pub fn publish_deprecation_marked(env: &Env, status: &DeprecationStatus) {
    env.events().publish(
        (symbol_short!("DEPREC"), symbol_short!("MARKED")),
        (status.contract_id.clone(), status.contract_name.clone()),
    );
}

pub fn publish_sunset_timeline_set(env: &Env, timeline: &SunsetTimeline) {
    env.events().publish(
        (symbol_short!("DEPREC"), symbol_short!("TIMELINE")),
        (timeline.contract_id.clone(), timeline.removal_date),
    );
}

pub fn publish_migration_guide_added(env: &Env, guide: &MigrationGuide) {
    env.events().publish(
        (symbol_short!("DEPREC"), symbol_short!("GUIDE")),
        (guide.contract_id.clone(), guide.guide_title.clone()),
    );
}

pub fn publish_phase_updated(env: &Env, status: &DeprecationStatus) {
    env.events().publish(
        (symbol_short!("DEPREC"), symbol_short!("PHASE")),
        (status.contract_id.clone(), status.phase as u32),
    );
}

pub fn publish_communication_sent(env: &Env, contract_id: &String, comm_id: u64) {
    env.events().publish(
        (symbol_short!("DEPREC"), symbol_short!("COMM")),
        (contract_id.clone(), comm_id),
    );
}

pub fn publish_removal_checklist_created(env: &Env, contract_id: &String) {
    env.events().publish(
        (symbol_short!("DEPREC"), symbol_short!("CHECKLIST")),
        contract_id.clone(),
    );
}

pub fn publish_checklist_item_completed(env: &Env, contract_id: &String, item_index: u32) {
    env.events().publish(
        (symbol_short!("DEPREC"), symbol_short!("DONE")),
        (contract_id.clone(), item_index),
    );
}
