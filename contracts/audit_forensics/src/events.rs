//! Event emission helpers for the Audit Forensics contract.
//!
//! Centralizes event topic constants for audit logging, automated
//! analysis, and compliance reporting.

use soroban_sdk::{symbol_short, Env, Symbol};

const TOPIC_PREFIX: &str = "AUDIT_FORENSICS";

/// Emit when an audit rule is configured.
pub fn emit_rule_configured(env: &Env, rule_id: u64) {
    env.events().publish(
        (Symbol::new(env, TOPIC_PREFIX), symbol_short!("RULE_CFG")),
        (rule_id,),
    );
}

/// Emit when an automated audit execution completes.
pub fn emit_audit_completed(env: &Env, execution_id: u64, passed: bool) {
    env.events().publish(
        (Symbol::new(env, TOPIC_PREFIX), symbol_short!("AUDIT_DONE")),
        (execution_id, passed),
    );
}

/// Emit when a compliance report is generated.
pub fn emit_compliance_report(env: &Env, start: u64, end: u64) {
    env.events().publish(
        (Symbol::new(env, TOPIC_PREFIX), symbol_short!("COMPL_RPT")),
        (start, end),
    );
}
