//! Event emission helpers for the Cross-Chain Bridge contract.
//!
//! Centralizes event topic constants for cross-chain bridge operations.
//! Primary event emission happens inline in `lib.rs`; this module provides
//! shared helpers for events requiring consistent topic formatting.

use soroban_sdk::{symbol_short, Env, Symbol};

const TOPIC_PREFIX: &str = "XBRIDGE";

/// Emit when a jurisdiction check is performed for cross-border data transfer.
pub fn emit_jurisdiction_check(env: &Env, jurisdiction: &str) {
    env.events().publish(
        (Symbol::new(env, TOPIC_PREFIX), symbol_short!("JUR_CHECK")),
        (symbol_short!(jurisdiction),),
    );
}
