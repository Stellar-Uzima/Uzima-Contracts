#![no_main]

use libfuzzer_sys::fuzz_target;
use soroban_sdk::{Env, Bytes};

fuzz_target!(|data: &[u8]| {
    let env = Env::default();
    let _bytes = Bytes::from_slice(&env, data);
});