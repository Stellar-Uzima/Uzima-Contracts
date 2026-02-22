#![no_std]

use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype, symbol_short, Address, Env, String, Vec,
};

#[cfg(test)]
mod test;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    AlreadyInitialized = 1,
    NotInitialized = 2,
    Unauthorized = 3,
    OracleAlreadyRegistered = 4,
    OracleNotFound = 5,
    OracleNotVerified = 6,
    OracleInactive = 7,
    InvalidData = 8,
    SubmissionAlreadyExists = 9,
    RoundNotFound = 10,
    InsufficientSubmissions = 11,
    ConsensusAlreadyFinalized = 12,
    ConsensusNotFound = 13,
    DisputeNotFound = 14,
    DisputeAlreadyResolved = 15,
    InvalidDisputeState = 16,
    InvalidFeedType = 17,
    ArbiterExists = 18,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[contracttype]
pub enum FeedKind {
    DrugPricing = 1,
    ClinicalTrial = 2,
    RegulatoryUpdate = 3,
    TreatmentOutcome = 4,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[contracttype]
pub enum SourceType {
    PharmaSupplier = 1,
    ClinicalRegistry = 2,
    RegulatoryBody = 3,
    MarketAggregator = 4,
    HospitalNetwork = 5,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[contracttype]
pub enum RegulatoryAuthority {
    FDA = 1,
    EMA = 2,
    MHRA = 3,
    PMDA = 4,
    WHO = 5,
    CDSCO = 6,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[contracttype]
pub enum RegulatoryStatus {
    Approved = 1,
    SafetyWarning = 2,
    Recall = 3,
    GuidelineUpdate = 4,
    TrialHold = 5,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[contracttype]
pub enum DisputeStatus {
    Open = 1,
    ResolvedValid = 2,
    ResolvedInvalid = 3,
}

#[derive(Clone)]
#[contracttype]
pub struct FeedKey {
    pub kind: FeedKind,
    pub feed_id: String,
}

#[derive(Clone)]
#[contracttype]
pub struct DrugPriceData {
    pub ndc_code: String,
    pub currency: String,
    pub price_minor: i128,
    pub availability_units: u32,
    pub observed_at: u64,
}

#[derive(Clone)]
#[contracttype]
pub struct ClinicalTrialData {
    pub trial_id: String,
    pub phase: u32,
    pub enrolled: u32,
    pub success_rate_bps: u32,
    pub adverse_event_rate_bps: u32,
    pub result_hash: String,
    pub published_at: u64,
}

#[derive(Clone)]
#[contracttype]
pub struct TreatmentOutcomeData {
    pub outcome_id: String,
    pub condition_code: String,
    pub treatment_code: String,
    pub improvement_rate_bps: u32,
    pub readmission_rate_bps: u32,
    pub mortality_rate_bps: u32,
    pub sample_size: u32,
    pub reported_at: u64,
}

#[derive(Clone)]
#[contracttype]
pub struct RegulatoryUpdateData {
    pub regulation_id: String,
    pub authority: RegulatoryAuthority,
    pub status: RegulatoryStatus,
    pub title: String,
    pub details_hash: String,
    pub effective_at: u64,
}

#[derive(Clone)]
#[contracttype]
pub enum FeedPayload {
    DrugPrice(DrugPriceData),
    ClinicalTrial(ClinicalTrialData),
    RegulatoryUpdate(RegulatoryUpdateData),
    TreatmentOutcome(TreatmentOutcomeData),
}

#[derive(Clone)]
#[contracttype]
pub struct OracleNode {
    pub operator: Address,
    pub endpoint: String,
    pub source_type: SourceType,
    pub verified: bool,
    pub active: bool,
    pub reputation: i128,
    pub submissions: u32,
    pub disputes: u32,
    pub last_seen: u64,
}

#[derive(Clone)]
#[contracttype]
pub struct Config {
    pub admin: Address,
    pub arbiters: Vec<Address>,
    pub min_submissions: u32,
    pub min_reputation: i128,
    pub max_drug_price_minor: i128,
    pub max_availability_units: u32,
}

#[derive(Clone)]
#[contracttype]
pub struct AggregationRound {
    pub id: u64,
    pub started_at: u64,
    pub finalized: bool,
    pub submissions: u32,
}

#[derive(Clone)]
#[contracttype]
pub struct ConsensusRecord {
    pub key: FeedKey,
    pub payload: FeedPayload,
    pub round_id: u64,
    pub finalized_at: u64,
    pub submitters: Vec<Address>,
    pub confidence_bps: u32,
    pub disputed: bool,
}

#[derive(Clone)]
#[contracttype]
pub struct Dispute {
    pub id: u64,
    pub key: FeedKey,
    pub round_id: u64,
    pub challenger: Address,
    pub reason: String,
    pub status: DisputeStatus,
    pub opened_at: u64,
    pub resolved_at: Option<u64>,
    pub resolver: Option<Address>,
    pub ruling: Option<String>,
}

#[derive(Clone)]
#[contracttype]
pub enum DataKey {
    Config,
    Oracle(Address),
    OracleList,
    RoundCounter(FeedKey),
    Round(FeedKey, u64),
    Submission(FeedKey, u64, Address),
    Consensus(FeedKey),
    DisputeCount,
    Dispute(u64),
}

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
        if env.storage().instance().has(&DataKey::Config) {
            return Err(Error::AlreadyInitialized);
        }

        if min_submissions == 0 {
            return Err(Error::InvalidData);
        }

        let cfg = Config {
            admin,
            arbiters,
            min_submissions,
            min_reputation: 0,
            max_drug_price_minor: 1_000_000_000,
            max_availability_units: 5_000_000,
        };

        env.storage().instance().set(&DataKey::Config, &cfg);
        env.storage()
            .instance()
            .set(&DataKey::OracleList, &Vec::<Address>::new(&env));
        env.storage().instance().set(&DataKey::DisputeCount, &0u64);
        Ok(())
    }

    pub fn register_oracle(
        env: Env,
        operator: Address,
        endpoint: String,
        source_type: SourceType,
    ) -> Result<(), Error> {
        operator.require_auth();
        Self::require_initialized(&env)?;

        if endpoint.len() == 0 {
            return Err(Error::InvalidData);
        }

        if env
            .storage()
            .persistent()
            .has(&DataKey::Oracle(operator.clone()))
        {
            return Err(Error::OracleAlreadyRegistered);
        }

        let mut oracles: Vec<Address> = env
            .storage()
            .instance()
            .get(&DataKey::OracleList)
            .unwrap_or(Vec::new(&env));
        oracles.push_back(operator.clone());

        let node = OracleNode {
            operator: operator.clone(),
            endpoint,
            source_type,
            verified: false,
            active: true,
            reputation: 50,
            submissions: 0,
            disputes: 0,
            last_seen: env.ledger().timestamp(),
        };

        env.storage()
            .persistent()
            .set(&DataKey::Oracle(operator), &node);
        env.storage().instance().set(&DataKey::OracleList, &oracles);
        Ok(())
    }

    pub fn verify_oracle(
        env: Env,
        admin: Address,
        operator: Address,
        verified: bool,
        active: bool,
    ) -> Result<(), Error> {
        Self::require_admin(&env, admin)?;

        let mut node = Self::read_oracle(&env, operator.clone())?;
        node.verified = verified;
        node.active = active;
        node.last_seen = env.ledger().timestamp();
        env.storage()
            .persistent()
            .set(&DataKey::Oracle(operator), &node);
        Ok(())
    }

    pub fn update_oracle_endpoint(
        env: Env,
        operator: Address,
        endpoint: String,
    ) -> Result<(), Error> {
        operator.require_auth();
        if endpoint.len() == 0 {
            return Err(Error::InvalidData);
        }
        let mut node = Self::read_oracle(&env, operator.clone())?;
        node.endpoint = endpoint;
        node.last_seen = env.ledger().timestamp();
        env.storage()
            .persistent()
            .set(&DataKey::Oracle(operator), &node);
        Ok(())
    }

    pub fn update_config(
        env: Env,
        admin: Address,
        min_submissions: u32,
        min_reputation: i128,
        max_drug_price_minor: i128,
        max_availability_units: u32,
    ) -> Result<(), Error> {
        Self::require_admin(&env, admin)?;

        if min_submissions == 0 || max_drug_price_minor <= 0 || max_availability_units == 0 {
            return Err(Error::InvalidData);
        }

        let mut cfg: Config = env
            .storage()
            .instance()
            .get(&DataKey::Config)
            .ok_or(Error::NotInitialized)?;

        cfg.min_submissions = min_submissions;
        cfg.min_reputation = min_reputation;
        cfg.max_drug_price_minor = max_drug_price_minor;
        cfg.max_availability_units = max_availability_units;

        env.storage().instance().set(&DataKey::Config, &cfg);
        Ok(())
    }

    pub fn add_arbiter(env: Env, admin: Address, arbiter: Address) -> Result<(), Error> {
        Self::require_admin(&env, admin)?;
        let mut cfg: Config = env
            .storage()
            .instance()
            .get(&DataKey::Config)
            .ok_or(Error::NotInitialized)?;

        if cfg.arbiters.contains(&arbiter) {
            return Err(Error::ArbiterExists);
        }

        cfg.arbiters.push_back(arbiter);
        env.storage().instance().set(&DataKey::Config, &cfg);
        Ok(())
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
        operator.require_auth();
        let cfg = Self::require_verified_oracle(&env, operator.clone())?;

        if feed_id.len() == 0 || ndc_code.len() == 0 || currency.len() == 0 {
            return Err(Error::InvalidData);
        }

        if price_minor <= 0
            || price_minor > cfg.max_drug_price_minor
            || availability_units > cfg.max_availability_units
        {
            return Err(Error::InvalidData);
        }

        let payload = FeedPayload::DrugPrice(DrugPriceData {
            ndc_code,
            currency,
            price_minor,
            availability_units,
            observed_at,
        });

        Self::submit_payload(env, operator, FeedKind::DrugPricing, feed_id, payload, cfg)
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
        operator.require_auth();
        let cfg = Self::require_verified_oracle(&env, operator.clone())?;

        if trial_id.len() == 0 || result_hash.len() == 0 {
            return Err(Error::InvalidData);
        }

        if phase == 0
            || phase > 4
            || enrolled == 0
            || success_rate_bps > 10_000
            || adverse_event_rate_bps > 10_000
        {
            return Err(Error::InvalidData);
        }

        let payload = FeedPayload::ClinicalTrial(ClinicalTrialData {
            trial_id,
            phase,
            enrolled,
            success_rate_bps,
            adverse_event_rate_bps,
            result_hash,
            published_at,
        });

        Self::submit_payload(
            env,
            operator,
            FeedKind::ClinicalTrial,
            payload_feed_id_from_trial(&payload),
            payload,
            cfg,
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
        operator.require_auth();
        let cfg = Self::require_verified_oracle(&env, operator.clone())?;

        if regulation_id.len() == 0 || title.len() == 0 || details_hash.len() == 0 {
            return Err(Error::InvalidData);
        }

        let payload = FeedPayload::RegulatoryUpdate(RegulatoryUpdateData {
            regulation_id: regulation_id.clone(),
            authority,
            status,
            title,
            details_hash,
            effective_at,
        });

        Self::submit_payload(
            env,
            operator,
            FeedKind::RegulatoryUpdate,
            regulation_id,
            payload,
            cfg,
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
        operator.require_auth();
        let cfg = Self::require_verified_oracle(&env, operator.clone())?;

        if outcome_id.len() == 0 || condition_code.len() == 0 || treatment_code.len() == 0 {
            return Err(Error::InvalidData);
        }

        if improvement_rate_bps > 10_000
            || readmission_rate_bps > 10_000
            || mortality_rate_bps > 10_000
            || sample_size == 0
        {
            return Err(Error::InvalidData);
        }

        let payload = FeedPayload::TreatmentOutcome(TreatmentOutcomeData {
            outcome_id,
            condition_code,
            treatment_code,
            improvement_rate_bps,
            readmission_rate_bps,
            mortality_rate_bps,
            sample_size,
            reported_at,
        });

        Self::submit_payload(
            env,
            operator,
            FeedKind::TreatmentOutcome,
            payload_feed_id_from_outcome(&payload),
            payload,
            cfg,
        )
    }

    pub fn finalize_feed(
        env: Env,
        kind: FeedKind,
        feed_id: String,
    ) -> Result<ConsensusRecord, Error> {
        Self::require_initialized(&env)?;
        if feed_id.len() == 0 {
            return Err(Error::InvalidData);
        }

        let key = FeedKey { kind, feed_id };
        let round_id = Self::active_round_id(&env, key.clone())?;
        Self::finalize_round(env, key, round_id)
    }

    pub fn raise_dispute(
        env: Env,
        challenger: Address,
        kind: FeedKind,
        feed_id: String,
        reason: String,
    ) -> Result<u64, Error> {
        challenger.require_auth();
        Self::require_initialized(&env)?;

        if reason.len() == 0 || feed_id.len() == 0 {
            return Err(Error::InvalidData);
        }

        let key = FeedKey { kind, feed_id };
        let consensus: ConsensusRecord = env
            .storage()
            .persistent()
            .get(&DataKey::Consensus(key.clone()))
            .ok_or(Error::ConsensusNotFound)?;

        if consensus.disputed {
            return Err(Error::InvalidDisputeState);
        }

        let dispute_id: u64 = env
            .storage()
            .instance()
            .get(&DataKey::DisputeCount)
            .unwrap_or(0u64)
            .saturating_add(1);

        let dispute = Dispute {
            id: dispute_id,
            key,
            round_id: consensus.round_id,
            challenger,
            reason,
            status: DisputeStatus::Open,
            opened_at: env.ledger().timestamp(),
            resolved_at: None,
            resolver: None,
            ruling: None,
        };

        env.storage()
            .persistent()
            .set(&DataKey::Dispute(dispute_id), &dispute);
        env.storage()
            .instance()
            .set(&DataKey::DisputeCount, &dispute_id);

        env.events()
            .publish((symbol_short!("dispute"),), dispute_id);
        Ok(dispute_id)
    }

    pub fn resolve_dispute(
        env: Env,
        resolver: Address,
        dispute_id: u64,
        valid_dispute: bool,
        ruling: String,
        penalized_oracle: Option<Address>,
    ) -> Result<(), Error> {
        resolver.require_auth();
        let cfg: Config = env
            .storage()
            .instance()
            .get(&DataKey::Config)
            .ok_or(Error::NotInitialized)?;

        if resolver != cfg.admin && !cfg.arbiters.contains(&resolver) {
            return Err(Error::Unauthorized);
        }

        let mut dispute: Dispute = env
            .storage()
            .persistent()
            .get(&DataKey::Dispute(dispute_id))
            .ok_or(Error::DisputeNotFound)?;

        if dispute.status != DisputeStatus::Open {
            return Err(Error::DisputeAlreadyResolved);
        }

        let mut consensus: ConsensusRecord = env
            .storage()
            .persistent()
            .get(&DataKey::Consensus(dispute.key.clone()))
            .ok_or(Error::ConsensusNotFound)?;

        if valid_dispute {
            consensus.disputed = true;
            dispute.status = DisputeStatus::ResolvedValid;

            if let Some(oracle) = penalized_oracle {
                Self::adjust_reputation(&env, oracle, -15, true)?;
            }
        } else {
            dispute.status = DisputeStatus::ResolvedInvalid;
            let mut i = 0;
            while i < consensus.submitters.len() {
                let submitter = consensus.submitters.get(i).unwrap();
                Self::adjust_reputation(&env, submitter, 2, false)?;
                i += 1;
            }
        }

        dispute.resolved_at = Some(env.ledger().timestamp());
        dispute.resolver = Some(resolver);
        dispute.ruling = Some(ruling);

        env.storage()
            .persistent()
            .set(&DataKey::Consensus(dispute.key.clone()), &consensus);
        env.storage()
            .persistent()
            .set(&DataKey::Dispute(dispute_id), &dispute);

        env.events()
            .publish((symbol_short!("resolve"), dispute_id), valid_dispute);

        Ok(())
    }

    pub fn get_consensus(env: Env, kind: FeedKind, feed_id: String) -> Option<ConsensusRecord> {
        let key = FeedKey { kind, feed_id };
        env.storage().persistent().get(&DataKey::Consensus(key))
    }

    pub fn get_oracle(env: Env, operator: Address) -> Option<OracleNode> {
        env.storage().persistent().get(&DataKey::Oracle(operator))
    }

    pub fn get_dispute(env: Env, dispute_id: u64) -> Option<Dispute> {
        env.storage()
            .persistent()
            .get(&DataKey::Dispute(dispute_id))
    }

    pub fn get_config(env: Env) -> Option<Config> {
        env.storage().instance().get(&DataKey::Config)
    }

    fn submit_payload(
        env: Env,
        operator: Address,
        kind: FeedKind,
        feed_id: String,
        payload: FeedPayload,
        cfg: Config,
    ) -> Result<u64, Error> {
        let key = FeedKey { kind, feed_id };
        let round_id = Self::ensure_active_round(&env, key.clone())?;

        let sub_key = DataKey::Submission(key.clone(), round_id, operator.clone());
        if env.storage().persistent().has(&sub_key) {
            return Err(Error::SubmissionAlreadyExists);
        }

        env.storage().persistent().set(&sub_key, &payload);

        let mut round: AggregationRound = env
            .storage()
            .persistent()
            .get(&DataKey::Round(key.clone(), round_id))
            .ok_or(Error::RoundNotFound)?;
        round.submissions = round.submissions.saturating_add(1);
        env.storage()
            .persistent()
            .set(&DataKey::Round(key.clone(), round_id), &round);

        let mut node = Self::read_oracle(&env, operator.clone())?;
        node.submissions = node.submissions.saturating_add(1);
        node.last_seen = env.ledger().timestamp();
        env.storage()
            .persistent()
            .set(&DataKey::Oracle(operator), &node);

        if round.submissions >= cfg.min_submissions {
            let _ = Self::finalize_round(env.clone(), key.clone(), round_id)?;
        }

        Ok(round_id)
    }

    fn finalize_round(env: Env, key: FeedKey, round_id: u64) -> Result<ConsensusRecord, Error> {
        let cfg: Config = env
            .storage()
            .instance()
            .get(&DataKey::Config)
            .ok_or(Error::NotInitialized)?;

        let mut round: AggregationRound = env
            .storage()
            .persistent()
            .get(&DataKey::Round(key.clone(), round_id))
            .ok_or(Error::RoundNotFound)?;

        if round.finalized {
            return Err(Error::ConsensusAlreadyFinalized);
        }

        let all_oracles: Vec<Address> = env
            .storage()
            .instance()
            .get(&DataKey::OracleList)
            .unwrap_or(Vec::new(&env));

        let mut submitters = Vec::<Address>::new(&env);
        let mut payloads = Vec::<FeedPayload>::new(&env);
        let mut weights = Vec::<i128>::new(&env);

        let mut i = 0;
        while i < all_oracles.len() {
            let oracle = all_oracles.get(i).unwrap();
            let node = Self::read_oracle(&env, oracle.clone())?;

            if node.verified && node.active && node.reputation >= cfg.min_reputation {
                let k = DataKey::Submission(key.clone(), round_id, oracle.clone());
                if env.storage().persistent().has(&k) {
                    let payload: FeedPayload = env
                        .storage()
                        .persistent()
                        .get(&k)
                        .ok_or(Error::InvalidData)?;
                    submitters.push_back(oracle);
                    payloads.push_back(payload);
                    weights.push_back(if node.reputation > 0 {
                        node.reputation
                    } else {
                        1
                    });
                }
            }
            i += 1;
        }

        if submitters.len() < cfg.min_submissions {
            return Err(Error::InsufficientSubmissions);
        }

        let aggregated = Self::aggregate_payload(
            &env,
            key.kind,
            payloads.clone(),
            weights.clone(),
            key.feed_id.clone(),
        )?;

        let confidence_bps = Self::compute_confidence_bps(submitters.len(), all_oracles.len());

        let consensus = ConsensusRecord {
            key: key.clone(),
            payload: aggregated,
            round_id,
            finalized_at: env.ledger().timestamp(),
            submitters: submitters.clone(),
            confidence_bps,
            disputed: false,
        };

        env.storage()
            .persistent()
            .set(&DataKey::Consensus(key.clone()), &consensus);

        round.finalized = true;
        env.storage()
            .persistent()
            .set(&DataKey::Round(key, round_id), &round);

        Self::reward_and_slash(&env, &consensus, submitters, payloads)?;

        env.events()
            .publish((symbol_short!("consens"), round_id), confidence_bps);

        Ok(consensus)
    }

    fn aggregate_payload(
        env: &Env,
        kind: FeedKind,
        payloads: Vec<FeedPayload>,
        weights: Vec<i128>,
        feed_id: String,
    ) -> Result<FeedPayload, Error> {
        let mut idx = 0;
        let mut sum_weight = 0i128;
        while idx < weights.len() {
            sum_weight = sum_weight.saturating_add(weights.get(idx).unwrap_or(1));
            idx += 1;
        }

        if sum_weight <= 0 {
            return Err(Error::InvalidData);
        }

        match kind {
            FeedKind::DrugPricing => {
                let mut price_weighted = 0i128;
                let mut avail_weighted = 0i128;
                let mut observed_at = 0u64;
                let mut ndc = String::from_str(env, "");
                let mut ccy = String::from_str(env, "");

                let mut i = 0;
                while i < payloads.len() {
                    let w = weights.get(i).unwrap_or(1);
                    match payloads.get(i).unwrap() {
                        FeedPayload::DrugPrice(v) => {
                            if ndc.len() == 0 {
                                ndc = v.ndc_code.clone();
                                ccy = v.currency.clone();
                            }
                            price_weighted =
                                price_weighted.saturating_add(v.price_minor.saturating_mul(w));
                            avail_weighted = avail_weighted
                                .saturating_add((v.availability_units as i128).saturating_mul(w));
                            if v.observed_at > observed_at {
                                observed_at = v.observed_at;
                            }
                        }
                        _ => return Err(Error::InvalidFeedType),
                    }
                    i += 1;
                }

                Ok(FeedPayload::DrugPrice(DrugPriceData {
                    ndc_code: ndc,
                    currency: ccy,
                    price_minor: price_weighted / sum_weight,
                    availability_units: (avail_weighted / sum_weight) as u32,
                    observed_at,
                }))
            }
            FeedKind::ClinicalTrial => {
                let mut phase_weighted = 0i128;
                let mut enrolled_weighted = 0i128;
                let mut success_weighted = 0i128;
                let mut adverse_weighted = 0i128;
                let mut published_at = 0u64;
                let mut result_hash = String::from_str(env, "");

                let mut i = 0;
                while i < payloads.len() {
                    let w = weights.get(i).unwrap_or(1);
                    match payloads.get(i).unwrap() {
                        FeedPayload::ClinicalTrial(v) => {
                            phase_weighted =
                                phase_weighted.saturating_add((v.phase as i128).saturating_mul(w));
                            enrolled_weighted = enrolled_weighted
                                .saturating_add((v.enrolled as i128).saturating_mul(w));
                            success_weighted = success_weighted
                                .saturating_add((v.success_rate_bps as i128).saturating_mul(w));
                            adverse_weighted = adverse_weighted.saturating_add(
                                (v.adverse_event_rate_bps as i128).saturating_mul(w),
                            );
                            if v.published_at > published_at {
                                published_at = v.published_at;
                                result_hash = v.result_hash.clone();
                            }
                        }
                        _ => return Err(Error::InvalidFeedType),
                    }
                    i += 1;
                }

                Ok(FeedPayload::ClinicalTrial(ClinicalTrialData {
                    trial_id: feed_id,
                    phase: (phase_weighted / sum_weight) as u32,
                    enrolled: (enrolled_weighted / sum_weight) as u32,
                    success_rate_bps: (success_weighted / sum_weight) as u32,
                    adverse_event_rate_bps: (adverse_weighted / sum_weight) as u32,
                    result_hash,
                    published_at,
                }))
            }
            FeedKind::RegulatoryUpdate => {
                let mut best_weight = -1i128;
                let mut picked: Option<RegulatoryUpdateData> = None;
                let mut i = 0;
                while i < payloads.len() {
                    let w = weights.get(i).unwrap_or(1);
                    match payloads.get(i).unwrap() {
                        FeedPayload::RegulatoryUpdate(v) => {
                            if w > best_weight {
                                best_weight = w;
                                picked = Some(v.clone());
                            }
                        }
                        _ => return Err(Error::InvalidFeedType),
                    }
                    i += 1;
                }

                if let Some(mut value) = picked {
                    value.regulation_id = feed_id;
                    Ok(FeedPayload::RegulatoryUpdate(value))
                } else {
                    Err(Error::InvalidData)
                }
            }
            FeedKind::TreatmentOutcome => {
                let mut improvement_weighted = 0i128;
                let mut readmission_weighted = 0i128;
                let mut mortality_weighted = 0i128;
                let mut sample_weighted = 0i128;
                let mut reported_at = 0u64;
                let mut condition_code = String::from_str(env, "");
                let mut treatment_code = String::from_str(env, "");

                let mut i = 0;
                while i < payloads.len() {
                    let w = weights.get(i).unwrap_or(1);
                    match payloads.get(i).unwrap() {
                        FeedPayload::TreatmentOutcome(v) => {
                            if condition_code.len() == 0 {
                                condition_code = v.condition_code.clone();
                                treatment_code = v.treatment_code.clone();
                            }
                            improvement_weighted = improvement_weighted
                                .saturating_add((v.improvement_rate_bps as i128).saturating_mul(w));
                            readmission_weighted = readmission_weighted
                                .saturating_add((v.readmission_rate_bps as i128).saturating_mul(w));
                            mortality_weighted = mortality_weighted
                                .saturating_add((v.mortality_rate_bps as i128).saturating_mul(w));
                            sample_weighted = sample_weighted
                                .saturating_add((v.sample_size as i128).saturating_mul(w));
                            if v.reported_at > reported_at {
                                reported_at = v.reported_at;
                            }
                        }
                        _ => return Err(Error::InvalidFeedType),
                    }
                    i += 1;
                }

                Ok(FeedPayload::TreatmentOutcome(TreatmentOutcomeData {
                    outcome_id: feed_id,
                    condition_code,
                    treatment_code,
                    improvement_rate_bps: (improvement_weighted / sum_weight) as u32,
                    readmission_rate_bps: (readmission_weighted / sum_weight) as u32,
                    mortality_rate_bps: (mortality_weighted / sum_weight) as u32,
                    sample_size: (sample_weighted / sum_weight) as u32,
                    reported_at,
                }))
            }
        }
    }

    fn reward_and_slash(
        env: &Env,
        consensus: &ConsensusRecord,
        submitters: Vec<Address>,
        payloads: Vec<FeedPayload>,
    ) -> Result<(), Error> {
        let mut i = 0;
        while i < submitters.len() {
            let submitter = submitters.get(i).unwrap();
            let payload = payloads.get(i).unwrap();
            let delta = Self::reputation_delta(consensus.payload.clone(), payload);
            Self::adjust_reputation(env, submitter, delta, delta < 0)?;
            i += 1;
        }
        Ok(())
    }

    fn reputation_delta(consensus: FeedPayload, payload: FeedPayload) -> i128 {
        match (consensus, payload) {
            (FeedPayload::DrugPrice(c), FeedPayload::DrugPrice(p)) => {
                if c.price_minor <= 0 {
                    return 1;
                }
                let diff = if p.price_minor > c.price_minor {
                    p.price_minor.saturating_sub(c.price_minor)
                } else {
                    c.price_minor.saturating_sub(p.price_minor)
                };
                let bps = diff.saturating_mul(10_000) / c.price_minor;
                if bps <= 500 {
                    5
                } else {
                    -3
                }
            }
            (FeedPayload::ClinicalTrial(c), FeedPayload::ClinicalTrial(p)) => {
                let diff = if p.success_rate_bps > c.success_rate_bps {
                    p.success_rate_bps.saturating_sub(c.success_rate_bps)
                } else {
                    c.success_rate_bps.saturating_sub(p.success_rate_bps)
                };
                if diff <= 700 {
                    4
                } else {
                    -2
                }
            }
            (FeedPayload::RegulatoryUpdate(c), FeedPayload::RegulatoryUpdate(p)) => {
                if c.status == p.status {
                    4
                } else {
                    -4
                }
            }
            (FeedPayload::TreatmentOutcome(c), FeedPayload::TreatmentOutcome(p)) => {
                let diff = if p.improvement_rate_bps > c.improvement_rate_bps {
                    p.improvement_rate_bps
                        .saturating_sub(c.improvement_rate_bps)
                } else {
                    c.improvement_rate_bps
                        .saturating_sub(p.improvement_rate_bps)
                };
                if diff <= 700 {
                    4
                } else {
                    -2
                }
            }
            _ => -5,
        }
    }

    fn adjust_reputation(
        env: &Env,
        operator: Address,
        delta: i128,
        is_dispute: bool,
    ) -> Result<(), Error> {
        let mut node = Self::read_oracle(env, operator.clone())?;
        node.reputation = node.reputation.saturating_add(delta);
        if node.reputation < 0 {
            node.reputation = 0;
        }
        if is_dispute {
            node.disputes = node.disputes.saturating_add(1);
        }
        node.last_seen = env.ledger().timestamp();
        env.storage()
            .persistent()
            .set(&DataKey::Oracle(operator), &node);
        Ok(())
    }

    fn ensure_active_round(env: &Env, key: FeedKey) -> Result<u64, Error> {
        if let Ok(id) = Self::active_round_id(env, key.clone()) {
            return Ok(id);
        }

        let next_id = env
            .storage()
            .persistent()
            .get(&DataKey::RoundCounter(key.clone()))
            .unwrap_or(0u64)
            .saturating_add(1);

        let round = AggregationRound {
            id: next_id,
            started_at: env.ledger().timestamp(),
            finalized: false,
            submissions: 0,
        };

        env.storage()
            .persistent()
            .set(&DataKey::RoundCounter(key.clone()), &next_id);
        env.storage()
            .persistent()
            .set(&DataKey::Round(key, next_id), &round);

        Ok(next_id)
    }

    fn active_round_id(env: &Env, key: FeedKey) -> Result<u64, Error> {
        let latest: u64 = env
            .storage()
            .persistent()
            .get(&DataKey::RoundCounter(key.clone()))
            .ok_or(Error::RoundNotFound)?;

        let round: AggregationRound = env
            .storage()
            .persistent()
            .get(&DataKey::Round(key, latest))
            .ok_or(Error::RoundNotFound)?;

        if round.finalized {
            return Err(Error::RoundNotFound);
        }

        Ok(latest)
    }

    fn require_initialized(env: &Env) -> Result<(), Error> {
        if !env.storage().instance().has(&DataKey::Config) {
            return Err(Error::NotInitialized);
        }
        Ok(())
    }

    fn require_admin(env: &Env, admin: Address) -> Result<(), Error> {
        admin.require_auth();
        let cfg: Config = env
            .storage()
            .instance()
            .get(&DataKey::Config)
            .ok_or(Error::NotInitialized)?;

        if cfg.admin != admin {
            return Err(Error::Unauthorized);
        }
        Ok(())
    }

    fn require_verified_oracle(env: &Env, operator: Address) -> Result<Config, Error> {
        let cfg: Config = env
            .storage()
            .instance()
            .get(&DataKey::Config)
            .ok_or(Error::NotInitialized)?;

        let node = Self::read_oracle(env, operator)?;
        if !node.verified {
            return Err(Error::OracleNotVerified);
        }
        if !node.active {
            return Err(Error::OracleInactive);
        }
        Ok(cfg)
    }

    fn read_oracle(env: &Env, operator: Address) -> Result<OracleNode, Error> {
        env.storage()
            .persistent()
            .get(&DataKey::Oracle(operator))
            .ok_or(Error::OracleNotFound)
    }

    fn compute_confidence_bps(submitters: u32, total_oracles: u32) -> u32 {
        if total_oracles == 0 {
            return 0;
        }
        let mut score = (submitters.saturating_mul(10_000)) / total_oracles;
        if score > 10_000 {
            score = 10_000;
        }
        score
    }
}

fn payload_feed_id_from_trial(payload: &FeedPayload) -> String {
    match payload {
        FeedPayload::ClinicalTrial(data) => data.trial_id.clone(),
        _ => unreachable!(),
    }
}

fn payload_feed_id_from_outcome(payload: &FeedPayload) -> String {
    match payload {
        FeedPayload::TreatmentOutcome(data) => data.outcome_id.clone(),
        _ => unreachable!(),
    }
}
