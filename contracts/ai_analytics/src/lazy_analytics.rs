#![no_std]
//! lazy_analytics - Lazy loading for optional analytics and AI metadata.
//!
//! Analytics and AI metadata are optional for most contract operations.
//! This module provides a lazy-load pattern that defers expensive storage
//! reads until the caller explicitly requests analytics data.

use soroban_sdk::{contracttype, symbol_short, Address, Env};

/// Analytics metadata stored alongside a medical record.
/// Only loaded when analytics access is requested.
#[derive(Clone)]
#[contracttype]
pub struct AnalyticsMetadata {
    pub record_id: u64,
    pub ai_risk_score: Option<u32>,
    pub ai_category: Option<soroban_sdk::Symbol>,
    pub model_version: Option<soroban_sdk::Symbol>,
    pub computed_at: u32,
}

/// Storage key for analytics metadata (stored separately from core record).
#[derive(Clone)]
#[contracttype]
pub enum AnalyticsKey {
    /// Per-record analytics (lazy — only written when AI processes the record).
    RecordAnalytics(u64),
    /// Whether analytics are enabled for this contract.
    AnalyticsEnabled,
}

/// Lazy analytics loader — avoids reading analytics storage on paths that
/// don't need it (which is the majority of clinical record operations).
pub struct LazyAnalytics;

impl LazyAnalytics {
    /// Returns analytics metadata for a record, or `None` if not computed yet.
    ///
    /// This is a separate storage read from the record itself, so callers
    /// that don't need analytics pay zero storage cost for this data.
    pub fn get(env: &Env, record_id: u64) -> Option<AnalyticsMetadata> {
        env.storage()
            .persistent()
            .get(&AnalyticsKey::RecordAnalytics(record_id))
    }

    /// Store analytics metadata for a record (called by AI pipeline, not by users).
    pub fn set(env: &Env, metadata: &AnalyticsMetadata) {
        let key = AnalyticsKey::RecordAnalytics(metadata.record_id);
        env.storage().persistent().set(&key, metadata);
        env.storage().persistent().extend_ttl(&key, 0, 518_400); // 30 days

        env.events().publish(
            (symbol_short!("analytics"), symbol_short!("computed")),
            (metadata.record_id, metadata.computed_at),
        );
    }

    /// Returns `true` if analytics are enabled for this deployment.
    pub fn is_enabled(env: &Env) -> bool {
        env.storage()
            .instance()
            .get(&AnalyticsKey::AnalyticsEnabled)
            .unwrap_or(false)
    }

    /// Enable or disable analytics (admin only — caller must have verified auth).
    pub fn set_enabled(env: &Env, enabled: bool) {
        env.storage()
            .instance()
            .set(&AnalyticsKey::AnalyticsEnabled, &enabled);
    }

    /// Delete analytics for a record (e.g. for GDPR erasure).
    pub fn delete(env: &Env, record_id: u64) {
        env.storage()
            .persistent()
            .remove(&AnalyticsKey::RecordAnalytics(record_id));
    }
}
