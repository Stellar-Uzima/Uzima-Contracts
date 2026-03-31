use soroban_sdk::{symbol_short, Address, Env};

pub fn publish_emergency_access_granted(
    env: &Env,
    patient: &Address,
    provider: &Address,
    expiry_at: u64,
    granted_at: u64,
) {
    env.events().publish(
        (symbol_short!("EMER"), symbol_short!("GRANT")),
        (patient, provider, expiry_at, granted_at),
    );
}

pub fn publish_emergency_access_approved(
    env: &Env,
    patient: &Address,
    provider: &Address,
    approver: &Address,
    approved_at: u64,
) {
    env.events().publish(
        (symbol_short!("EMER"), symbol_short!("APPR")),
        (patient, provider, approver, approved_at),
    );
}

pub fn publish_duplicate_approval(
    env: &Env,
    patient: &Address,
    provider: &Address,
    approver: &Address,
    timestamp: u64,
) {
    env.events().publish(
        (symbol_short!("EMER"), symbol_short!("DUPA")),
        (patient, provider, approver, timestamp),
    );
}

pub fn publish_emergency_access_checked(
    env: &Env,
    patient: &Address,
    provider: &Address,
    has_access: bool,
    checked_at: u64,
) {
    env.events().publish(
        (symbol_short!("EMER"), symbol_short!("CHECK")),
        (patient, provider, has_access, checked_at),
    );
}

pub fn publish_emergency_access_revoked(
    env: &Env,
    patient: &Address,
    provider: &Address,
    revoked_at: u64,
) {
    env.events().publish(
        (symbol_short!("EMER"), symbol_short!("REVOKE")),
        (patient, provider, revoked_at),
    );
}

pub fn publish_initialization(env: &Env, admin: &Address) {
    env.events()
        .publish((symbol_short!("EMER"), symbol_short!("INIT")), admin);
}
