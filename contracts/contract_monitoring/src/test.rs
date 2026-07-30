#![cfg(test)]

use super::*;
use soroban_sdk::{testutils::Events, Env, Symbol, BytesN};

#[test]
fn test_telemetry_event_emission() {
    let env = Env::default();
    let contract_id = BytesN::from_array(&env, &[0u8; 32]);
    let function_name = Symbol::new(&env, "process_record");

    TelemetryLogger::emit_execution_telemetry(
        &env,
        contract_id.clone(),
        function_name.clone(),
        1_200_000,
        512_000,
        ExecutionStatus::Success,
        0,
        0,
    );

    let events = env.events().all();
    assert_eq!(events.len(), 1);

    let event = events.get(0).unwrap();
    assert_eq!(
        event.0,
        (
            Symbol::new(&env, "telemetry"),
            function_name,
            ExecutionStatus::Success,
        )
    );
}