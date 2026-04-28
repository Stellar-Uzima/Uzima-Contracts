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

// ==================== Diagnostic Events ====================

/// Emitted when a function is entered (DEBUG level)
pub fn diag_fn_enter(env: &Env, fn_name: &'static str) {
    env.events().publish(
        (symbol_short!("DIAG"), symbol_short!("ENTER")),
        soroban_sdk::String::from_str(env, fn_name),
    );
}

/// Emitted when a function exits successfully (DEBUG level)
pub fn diag_fn_exit(env: &Env, fn_name: &'static str) {
    env.events().publish(
        (symbol_short!("DIAG"), symbol_short!("EXIT")),
        soroban_sdk::String::from_str(env, fn_name),
    );
}

/// Emitted on state change (INFO level)
pub fn diag_state_change(env: &Env, appointment_id: u64, old_status: u32, new_status: u32) {
    env.events().publish(
        (symbol_short!("DIAG"), symbol_short!("STATE")),
        (appointment_id, old_status, new_status),
    );
}

/// Emitted on validation failure (WARN level)
pub fn diag_validation_fail(env: &Env, fn_name: &'static str, reason: &'static str) {
    env.events().publish(
        (symbol_short!("DIAG"), symbol_short!("VALFAIL")),
        (
            soroban_sdk::String::from_str(env, fn_name),
            soroban_sdk::String::from_str(env, reason),
        ),
    );
}

/// Emitted on authorization check failure (WARN level)
pub fn diag_auth_fail(env: &Env, fn_name: &'static str) {
    env.events().publish(
        (symbol_short!("DIAG"), symbol_short!("AUTHFAIL")),
        soroban_sdk::String::from_str(env, fn_name),
    );
}

/// Emitted on error condition (ERROR level)
pub fn diag_error(env: &Env, fn_name: &'static str, error_code: u32) {
    env.events().publish(
        (symbol_short!("DIAG"), symbol_short!("ERR")),
        (soroban_sdk::String::from_str(env, fn_name), error_code),
    );
}
