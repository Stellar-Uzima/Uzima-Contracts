#![no_std]
#![allow(clippy::too_many_arguments)]
#![allow(clippy::len_zero)]
#![allow(clippy::unwrap_used)]
#![allow(clippy::arithmetic_side_effects)]

mod admin;
mod disputes;
mod oracles;
mod submissions;
#[cfg(test)]
mod test;
mod types;
mod utils;

use soroban_sdk::{contract, contractimpl, Address, Env, String, Vec};

pub use types::{
    AggregationRound, ClinicalTrialData, Config, ConsensusRecord, DataKey, Dispute, DisputeStatus,
    DrugPriceData, Error, FeedKey, FeedKind, FeedPayload, OracleNode, RegulatoryAuthority,
    RegulatoryStatus, RegulatoryUpdateData, SourceType, TreatmentOutcomeData,
};

#[contract]
pub struct HealthcareOracleNetwork;

#[contractimpl]
impl HealthcareOracleNetwork {
    pub fn initialize(
        env: Env,
        admin: Address,
        arbiters: Vec<Address>,
        min_submissions: u32,
    ) -> Result<(), Error> {
        admin::initialize(env, admin, arbiters, min_submissions)
    }

    pub fn register_oracle(
        env: Env,
        operator: Address,
        endpoint: String,
        source_type: SourceType,
    ) -> Result<(), Error> {
        oracles::register_oracle(env, operator, endpoint, source_type)
    }

    pub fn verify_oracle(
        env: Env,
        admin: Address,
        operator: Address,
        verified: bool,
        active: bool,
    ) -> Result<(), Error> {
        oracles::verify_oracle(env, admin, operator, verified, active)
    }

    pub fn update_oracle_endpoint(
        env: Env,
        operator: Address,
        endpoint: String,
    ) -> Result<(), Error> {
        oracles::update_oracle_endpoint(env, operator, endpoint)
    }

    pub fn update_config(
        env: Env,
        admin: Address,
        min_submissions: u32,
        min_reputation: i128,
        max_drug_price_minor: i128,
        max_availability_units: u32,
    ) -> Result<(), Error> {
        admin::update_config(
            env,
            admin,
            min_submissions,
            min_reputation,
            max_drug_price_minor,
            max_availability_units,
        )
    }

    pub fn add_arbiter(env: Env, admin: Address, arbiter: Address) -> Result<(), Error> {
        admin::add_arbiter(env, admin, arbiter)
    }

    pub fn submit_drug_price(
        env: Env,
        operator: Address,
        feed_id: String,
        ndc_code: String,
        currency: String,
        price_minor: i128,
        availability_units: u32,
        observed_at: u64,
    ) -> Result<u64, Error> {
        submissions::submit_drug_price(
            env,
            operator,
            feed_id,
            ndc_code,
            currency,
            price_minor,
            availability_units,
            observed_at,
        )
    }

    pub fn submit_clinical_trial(
        env: Env,
        operator: Address,
        trial_id: String,
        phase: u32,
        enrolled: u32,
        success_rate_bps: u32,
        adverse_event_rate_bps: u32,
        result_hash: String,
        published_at: u64,
    ) -> Result<u64, Error> {
        submissions::submit_clinical_trial(
            env,
            operator,
            trial_id,
            phase,
            enrolled,
            success_rate_bps,
            adverse_event_rate_bps,
            result_hash,
            published_at,
        )
    }

    pub fn submit_regulatory_update(
        env: Env,
        operator: Address,
        regulation_id: String,
        authority: RegulatoryAuthority,
        status: RegulatoryStatus,
        title: String,
        details_hash: String,
        effective_at: u64,
    ) -> Result<u64, Error> {
        submissions::submit_regulatory_update(
            env,
            operator,
            regulation_id,
            authority,
            status,
            title,
            details_hash,
            effective_at,
        )
    }

    pub fn submit_treatment_outcome(
        env: Env,
        operator: Address,
        outcome_id: String,
        condition_code: String,
        treatment_code: String,
        improvement_rate_bps: u32,
        readmission_rate_bps: u32,
        mortality_rate_bps: u32,
        sample_size: u32,
        reported_at: u64,
    ) -> Result<u64, Error> {
        submissions::submit_treatment_outcome(
            env,
            operator,
            outcome_id,
            condition_code,
            treatment_code,
            improvement_rate_bps,
            readmission_rate_bps,
            mortality_rate_bps,
            sample_size,
            reported_at,
        )
    }

    pub fn finalize_feed(
        env: Env,
        kind: FeedKind,
        feed_id: String,
    ) -> Result<ConsensusRecord, Error> {
        submissions::finalize_feed(env, kind, feed_id)
    }

    pub fn raise_dispute(
        env: Env,
        challenger: Address,
        kind: FeedKind,
        feed_id: String,
        reason: String,
    ) -> Result<u64, Error> {
        disputes::raise_dispute(env, challenger, kind, feed_id, reason)
    }

    pub fn resolve_dispute(
        env: Env,
        resolver: Address,
        dispute_id: u64,
        valid_dispute: bool,
        ruling: String,
        penalized_oracle: Option<Address>,
    ) -> Result<(), Error> {
        disputes::resolve_dispute(
            env,
            resolver,
            dispute_id,
            valid_dispute,
            ruling,
            penalized_oracle,
        )
    }

    pub fn get_consensus(env: Env, kind: FeedKind, feed_id: String) -> Option<ConsensusRecord> {
        submissions::get_consensus(env, kind, feed_id)
    }

    pub fn get_oracle(env: Env, operator: Address) -> Option<OracleNode> {
        oracles::get_oracle(env, operator)
    }

    pub fn get_dispute(env: Env, dispute_id: u64) -> Option<Dispute> {
        disputes::get_dispute(env, dispute_id)
    }

    pub fn get_config(env: Env) -> Option<Config> {
        admin::get_config(env)
    }
}
