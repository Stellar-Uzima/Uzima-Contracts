use soroban_sdk::{symbol_short, Address, BytesN, Env};

pub fn publish_record_stored(
    env: &Env,
    patient_id: &Address,
    record_hash: &BytesN<32>,
    timestamp: u64,
) {
    env.events().publish(
        (symbol_short!("MEDREG"), symbol_short!("STORE")),
        (patient_id, record_hash.clone(), timestamp),
    );
}

pub fn publish_record_verified(
    env: &Env,
    patient_id: &Address,
    record_hash: &BytesN<32>,
    verified: bool,
) {
    env.events().publish(
        (symbol_short!("MEDREG"), symbol_short!("VERIFY")),
        (patient_id, record_hash.clone(), verified),
    );
}

pub fn publish_initialization(env: &Env, admin: &Address) {
    env.events()
        .publish((symbol_short!("MEDREG"), symbol_short!("INIT")), admin);
}

pub fn publish_duplicate_rejected(env: &Env, patient_id: &Address, record_hash: &BytesN<32>) {
    env.events().publish(
        (symbol_short!("MEDREG"), symbol_short!("DUP")),
        (patient_id, record_hash.clone()),
    );
}
