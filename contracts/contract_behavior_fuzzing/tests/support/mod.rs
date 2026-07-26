#![allow(dead_code)]

use soroban_sdk::{testutils::Events as _, BytesN, Env, String};

pub fn bytes32(env: &Env, seed: u8) -> BytesN<32> {
    BytesN::from_array(env, &[seed; 32])
}

pub fn event_count(env: &Env) -> usize {
    env.events().all().len() as usize
}

pub fn s(env: &Env, value: &str) -> String {
    String::from_str(env, value)
}
