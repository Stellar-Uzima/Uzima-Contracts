use soroban_sdk::{symbol_short, Address, Env};

pub fn publish_appointment_booked(
    env: &Env,
    appointment_id: u64,
    patient: &Address,
    provider: &Address,
    amount: i128,
    timestamp: u64,
) {
    env.events().publish(
        (symbol_short!("APPT"), symbol_short!("BOOK")),
        (appointment_id, patient, provider, amount, timestamp),
    );
}

pub fn publish_appointment_confirmed(
    env: &Env,
    appointment_id: u64,
    provider: &Address,
    timestamp: u64,
) {
    env.events().publish(
        (symbol_short!("APPT"), symbol_short!("CONF")),
        (appointment_id, provider, timestamp),
    );
}

pub fn publish_appointment_refunded(
    env: &Env,
    appointment_id: u64,
    patient: &Address,
    amount: i128,
    timestamp: u64,
) {
    env.events().publish(
        (symbol_short!("APPT"), symbol_short!("REFUND")),
        (appointment_id, patient, amount, timestamp),
    );
}

pub fn publish_funds_released(
    env: &Env,
    appointment_id: u64,
    provider: &Address,
    amount: i128,
    timestamp: u64,
) {
    env.events().publish(
        (symbol_short!("APPT"), symbol_short!("RELEASE")),
        (appointment_id, provider, amount, timestamp),
    );
}

pub fn publish_initialization(env: &Env, admin: &Address) {
    env.events()
        .publish((symbol_short!("APPT"), symbol_short!("INIT")), admin);
}
