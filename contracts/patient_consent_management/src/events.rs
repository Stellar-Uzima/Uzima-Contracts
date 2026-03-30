use soroban_sdk::{symbol_short, Address, Env};

pub fn publish_consent_granted(env: &Env, patient: &Address, provider: &Address, timestamp: u64) {
    env.events().publish(
        (symbol_short!("CONSENT"), symbol_short!("GRANT")),
        (patient, provider, timestamp),
    );
}

pub fn publish_consent_revoked(env: &Env, patient: &Address, provider: &Address, timestamp: u64) {
    env.events().publish(
        (symbol_short!("CONSENT"), symbol_short!("REVOKE")),
        (patient, provider, timestamp),
    );
}

pub fn publish_initialization(env: &Env, admin: &Address) {
    env.events()
        .publish((symbol_short!("CONSENT"), symbol_short!("INIT")), admin);
}

pub fn publish_unauthorized_attempt(env: &Env, caller: &Address, patient: &Address, timestamp: u64) {
    env.events().publish(
        (symbol_short!("CONSENT"), symbol_short!("UNAUTH")),
        (caller, patient, timestamp),
    );
}

pub fn publish_consent_checked(env: &Env, patient: &Address, provider: &Address, has_consent: bool) {
    env.events().publish(
        (symbol_short!("CONSENT"), symbol_short!("CHECK")),
        (patient, provider, has_consent),
    );
}
