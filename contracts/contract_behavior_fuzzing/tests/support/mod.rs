#![allow(dead_code)]

use contract_behavior_fuzzing::TraceEvent;
use soroban_sdk::{testutils::Events as _, xdr::ToXdr, BytesN, Env, String, Symbol, TryFromVal};

pub fn bytes32(env: &Env, seed: u8) -> BytesN<32> {
    BytesN::from_array(env, &[seed; 32])
}

pub fn event_count(env: &Env) -> usize {
    env.events().all().len() as usize
}

pub fn s(env: &Env, value: &str) -> String {
    String::from_str(env, value)
}

/// Captures the ordered event stream published on `env` so far as `TraceEvent`
/// entries. Each event's topics keep only the `Symbol` values (converted to
/// their UTF-8 text); the data payload is stored as its canonical XDR bytes so
/// it survives re-encoding and off-chain replay unchanged.
pub fn captured_trace(env: &Env) -> Vec<TraceEvent> {
    env.events()
        .all()
        .iter()
        .map(|(_, topics, data)| {
            let topic_names = topics
                .iter()
                .filter_map(|value| Symbol::try_from_val(env, value).ok())
                .map(|symbol| symbol.to_string().into_bytes())
                .collect::<Vec<_>>();
            let payload = (*data).to_xdr(env).to_alloc_vec();
            TraceEvent::new(topic_names, payload)
        })
        .collect()
}
