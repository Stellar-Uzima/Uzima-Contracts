#![no_std]
#![allow(clippy::too_many_arguments)]

#[cfg(test)]
mod test;

use soroban_sdk::xdr::ToXdr;
use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype, symbol_short, Address, BytesN, Env,
    Symbol, Vec,
};

const ROLE_INDEXER: u32 = 1;
const ROLE_SEARCHER: u32 = 2;
const ROLE_AUDITOR: u32 = 4;
const ROLE_CONFIDENTIAL: u32 = 8;
const ALL_ROLES: u32 = ROLE_INDEXER | ROLE_SEARCHER | ROLE_AUDITOR | ROLE_CONFIDENTIAL;

const ADMIN: Symbol = symbol_short!("ADMIN");
const PAUSED: Symbol = symbol_short!("PAUSED");
const QUERY_ID: Symbol = symbol_short!("QUERY_ID");

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[contracttype]
pub enum ChainId {
    Stellar,
    Ethereum,
    Polygon,
    Avalanche,
    Arbitrum,
    Optimism,
    Custom(u32),
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub struct IndexInput {
    pub record_id: u64,
    pub patient: Address,
    pub network: ChainId,
    pub created_at: u64,
    pub is_confidential: bool,
    pub category_hash: BytesN<32>,
    pub token_hashes: Vec<BytesN<32>>,
    pub attribute_hashes: Vec<BytesN<32>>,
    pub encrypted_ref_hash: BytesN<32>,
    pub quality_score_bps: u32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub struct SearchIndexEntry {
    pub record_id: u64,
    pub indexed_by: Address,
    pub patient: Address,
    pub network: ChainId,
    pub created_at: u64,
    pub is_confidential: bool,
    pub category_hash: BytesN<32>,
    pub token_hashes: Vec<BytesN<32>>,
    pub attribute_hashes: Vec<BytesN<32>>,
    pub encrypted_ref_hash: BytesN<32>,
    pub quality_score_bps: u32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub struct SearchQuery {
    pub required_tokens: Vec<BytesN<32>>,
    pub optional_tokens: Vec<BytesN<32>>,
    pub category_filters: Vec<BytesN<32>>,
    pub attribute_filters: Vec<BytesN<32>>,
    pub network_filters: Vec<ChainId>,
    pub patient_filter: Option<Address>,
    pub from_timestamp: u64,
    pub to_timestamp: u64,
    pub include_confidential: bool,
    pub min_quality_bps: u32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub struct SearchResult {
    pub record_id: u64,
    pub patient: Address,
    pub network: ChainId,
    pub created_at: u64,
    pub encrypted_ref_hash: BytesN<32>,
    pub is_confidential: bool,
    pub score_bps: u32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub struct QueryCacheEntry {
    pub query_hash: BytesN<32>,
    pub created_at: u64,
    pub expires_at: u64,
    pub hit_count: u32,
    pub results: Vec<SearchResult>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub struct CachePolicy {
    pub ttl_seconds: u64,
    pub max_entries: u32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub struct RankingConfig {
    pub required_weight_bps: u32,
    pub optional_weight_bps: u32,
    pub recency_weight_bps: u32,
    pub quality_weight_bps: u32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub struct SearchAuditEntry {
    pub query_id: u64,
    pub caller: Address,
    pub query_hash: BytesN<32>,
    pub timestamp: u64,
    pub result_count: u32,
    pub from_cache: bool,
    pub granted: bool,
}

#[derive(Clone)]
#[contracttype]
pub enum DataKey {
    Roles(Address),
    Index(u64),
    IndexedIds,
    TokenPosting(BytesN<32>),
    CategoryPosting(BytesN<32>),
    AttributePosting(BytesN<32>),
    Cache(BytesN<32>),
    CacheOrder,
    CachePolicy,
    Ranking,
    Audit(u64),
}

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    AlreadyInitialized = 1,
    NotInitialized = 2,
    NotAuthorized = 3,
    ContractPaused = 4,
    InvalidInput = 5,
    RecordNotIndexed = 6,
    QueryTooLarge = 7,
    CacheMiss = 8,
}

#[contract]
pub struct MedicalRecordSearchContract;

#[contractimpl]
impl MedicalRecordSearchContract {
    pub fn initialize(env: Env, admin: Address) -> Result<(), Error> {
        admin.require_auth();
        if env.storage().instance().has(&ADMIN) {
            return Err(Error::AlreadyInitialized);
        }

        env.storage().instance().set(&ADMIN, &admin);
        env.storage().instance().set(&PAUSED, &false);
        env.storage().instance().set(&QUERY_ID, &1u64);
        env.storage()
            .persistent()
            .set(&DataKey::IndexedIds, &Vec::<u64>::new(&env));
        env.storage()
            .persistent()
            .set(&DataKey::CacheOrder, &Vec::<BytesN<32>>::new(&env));
        env.storage().persistent().set(
            &DataKey::CachePolicy,
            &CachePolicy {
                ttl_seconds: 600,
                max_entries: 50,
            },
        );
        env.storage().persistent().set(
            &DataKey::Ranking,
            &RankingConfig {
                required_weight_bps: 4_000,
                optional_weight_bps: 2_000,
                recency_weight_bps: 2_000,
                quality_weight_bps: 2_000,
            },
        );
        Ok(())
    }

    pub fn set_paused(env: Env, caller: Address, paused: bool) -> Result<bool, Error> {
        caller.require_auth();
        Self::require_admin(&env, &caller)?;
        env.storage().instance().set(&PAUSED, &paused);
        Ok(true)
    }

    pub fn assign_role(
        env: Env,
        caller: Address,
        user: Address,
        role_mask: u32,
    ) -> Result<bool, Error> {
        caller.require_auth();
        Self::require_admin(&env, &caller)?;
        env.storage()
            .persistent()
            .set(&DataKey::Roles(user), &(role_mask & ALL_ROLES));
        Ok(true)
    }

    pub fn set_cache_policy(env: Env, caller: Address, policy: CachePolicy) -> Result<bool, Error> {
        caller.require_auth();
        Self::require_admin(&env, &caller)?;
        if policy.ttl_seconds == 0 || policy.max_entries == 0 {
            return Err(Error::InvalidInput);
        }
        env.storage()
            .persistent()
            .set(&DataKey::CachePolicy, &policy);
        Ok(true)
    }

    pub fn set_ranking(env: Env, caller: Address, cfg: RankingConfig) -> Result<bool, Error> {
        caller.require_auth();
        Self::require_admin(&env, &caller)?;
        let total = cfg
            .required_weight_bps
            .saturating_add(cfg.optional_weight_bps)
            .saturating_add(cfg.recency_weight_bps)
            .saturating_add(cfg.quality_weight_bps);
        if total == 0 {
            return Err(Error::InvalidInput);
        }
        env.storage().persistent().set(&DataKey::Ranking, &cfg);
        Ok(true)
    }

    pub fn index_record(env: Env, caller: Address, input: IndexInput) -> Result<bool, Error> {
        caller.require_auth();
        Self::require_role(&env, &caller, ROLE_INDEXER)?;
        Self::require_not_paused(&env)?;
        if input.token_hashes.is_empty() {
            return Err(Error::InvalidInput);
        }

        let entry = SearchIndexEntry {
            record_id: input.record_id,
            indexed_by: caller.clone(),
            patient: input.patient.clone(),
            network: input.network,
            created_at: input.created_at,
            is_confidential: input.is_confidential,
            category_hash: input.category_hash.clone(),
            token_hashes: input.token_hashes.clone(),
            attribute_hashes: input.attribute_hashes.clone(),
            encrypted_ref_hash: input.encrypted_ref_hash,
            quality_score_bps: input.quality_score_bps,
        };

        env.storage()
            .persistent()
            .set(&DataKey::Index(input.record_id), &entry);

        let mut ids: Vec<u64> = env
            .storage()
            .persistent()
            .get(&DataKey::IndexedIds)
            .unwrap_or(Vec::new(&env));
        if !ids.iter().any(|id| id == input.record_id) {
            ids.push_back(input.record_id);
            env.storage().persistent().set(&DataKey::IndexedIds, &ids);
        }

        for token in input.token_hashes.iter() {
            Self::append_posting(&env, DataKey::TokenPosting(token), input.record_id);
        }
        Self::append_posting(
            &env,
            DataKey::CategoryPosting(input.category_hash),
            input.record_id,
        );
        for attr in input.attribute_hashes.iter() {
            Self::append_posting(&env, DataKey::AttributePosting(attr), input.record_id);
        }

        env.events()
            .publish((symbol_short!("SRCH_IDX"),), (input.record_id, caller));
        Ok(true)
    }

    pub fn batch_index_records(
        env: Env,
        caller: Address,
        inputs: Vec<IndexInput>,
    ) -> Result<(u32, u32), Error> {
        caller.require_auth();
        Self::require_role(&env, &caller, ROLE_INDEXER)?;
        if inputs.is_empty() {
            return Err(Error::InvalidInput);
        }
        if inputs.len() > 100 {
            return Err(Error::QueryTooLarge);
        }

        let mut indexed = 0u32;
        let mut failed = 0u32;
        for input in inputs.iter() {
            let res = Self::index_record(env.clone(), caller.clone(), input);
            if res.is_ok() {
                indexed = indexed.saturating_add(1);
            } else {
                failed = failed.saturating_add(1);
            }
        }
        Ok((indexed, failed))
    }

    pub fn search(
        env: Env,
        caller: Address,
        query: SearchQuery,
        page: u32,
        page_size: u32,
    ) -> Result<Vec<SearchResult>, Error> {
        caller.require_auth();
        Self::require_role(&env, &caller, ROLE_SEARCHER)?;
        Self::require_not_paused(&env)?;
        if page_size == 0 || page_size > 100 {
            return Err(Error::InvalidInput);
        }
        if query.required_tokens.len() > 32 || query.optional_tokens.len() > 32 {
            return Err(Error::QueryTooLarge);
        }

        let q_hash = Self::query_hash(&env, &query);
        let now = env.ledger().timestamp();
        if let Some(mut cache) = env
            .storage()
            .persistent()
            .get::<DataKey, QueryCacheEntry>(&DataKey::Cache(q_hash.clone()))
        {
            if now <= cache.expires_at {
                cache.hit_count = cache.hit_count.saturating_add(1);
                env.storage()
                    .persistent()
                    .set(&DataKey::Cache(q_hash.clone()), &cache);
                Self::append_audit(&env, caller, q_hash, cache.results.len(), true, true);
                return Ok(Self::paginate_results(
                    &env,
                    &cache.results,
                    page,
                    page_size,
                ));
            }
            env.storage()
                .persistent()
                .remove(&DataKey::Cache(q_hash.clone()));
        }

        let candidate_ids = Self::candidate_ids(&env, &query);
        let mut ranked = Vec::new(&env);
        for id in candidate_ids.iter() {
            if let Some(entry) = env
                .storage()
                .persistent()
                .get::<DataKey, SearchIndexEntry>(&DataKey::Index(id))
            {
                if !Self::entry_matches(&query, &entry, &caller, &env) {
                    continue;
                }
                let score = Self::compute_score(&env, &query, &entry);
                let result = SearchResult {
                    record_id: entry.record_id,
                    patient: entry.patient.clone(),
                    network: entry.network,
                    created_at: entry.created_at,
                    encrypted_ref_hash: entry.encrypted_ref_hash,
                    is_confidential: entry.is_confidential,
                    score_bps: score,
                };
                Self::insert_ranked(&env, &mut ranked, result);
            }
        }

        Self::upsert_cache(&env, q_hash.clone(), ranked.clone())?;
        Self::append_audit(&env, caller, q_hash, ranked.len(), false, true);
        Ok(Self::paginate_results(&env, &ranked, page, page_size))
    }

    pub fn get_cache_entry(env: Env, query_hash: BytesN<32>) -> Result<QueryCacheEntry, Error> {
        env.storage()
            .persistent()
            .get(&DataKey::Cache(query_hash))
            .ok_or(Error::CacheMiss)
    }

    pub fn invalidate_cache(env: Env, caller: Address) -> Result<bool, Error> {
        caller.require_auth();
        Self::require_admin(&env, &caller)?;
        let order: Vec<BytesN<32>> = env
            .storage()
            .persistent()
            .get(&DataKey::CacheOrder)
            .unwrap_or(Vec::new(&env));
        for key in order.iter() {
            env.storage().persistent().remove(&DataKey::Cache(key));
        }
        env.storage()
            .persistent()
            .set(&DataKey::CacheOrder, &Vec::<BytesN<32>>::new(&env));
        Ok(true)
    }

    pub fn get_audit(env: Env, caller: Address, query_id: u64) -> Result<SearchAuditEntry, Error> {
        caller.require_auth();
        Self::require_role(&env, &caller, ROLE_AUDITOR)?;
        env.storage()
            .persistent()
            .get(&DataKey::Audit(query_id))
            .ok_or(Error::InvalidInput)
    }

    pub fn preview_query_hash(env: Env, query: SearchQuery) -> BytesN<32> {
        Self::query_hash(&env, &query)
    }

    pub fn get_indexed_entry(env: Env, record_id: u64) -> Result<SearchIndexEntry, Error> {
        env.storage()
            .persistent()
            .get(&DataKey::Index(record_id))
            .ok_or(Error::RecordNotIndexed)
    }
}

impl MedicalRecordSearchContract {
    fn append_posting(env: &Env, key: DataKey, record_id: u64) {
        let mut ids: Vec<u64> = env
            .storage()
            .persistent()
            .get(&key)
            .unwrap_or(Vec::new(env));
        if !ids.iter().any(|id| id == record_id) {
            ids.push_back(record_id);
            env.storage().persistent().set(&key, &ids);
        }
    }

    fn candidate_ids(env: &Env, query: &SearchQuery) -> Vec<u64> {
        if !query.required_tokens.is_empty() {
            let Some(first) = query.required_tokens.get(0) else {
                return Vec::new(env);
            };
            let mut ids: Vec<u64> = env
                .storage()
                .persistent()
                .get(&DataKey::TokenPosting(first))
                .unwrap_or(Vec::new(env));

            for i in 1..query.required_tokens.len() {
                let Some(token) = query.required_tokens.get(i) else {
                    continue;
                };
                let posting: Vec<u64> = env
                    .storage()
                    .persistent()
                    .get(&DataKey::TokenPosting(token))
                    .unwrap_or(Vec::new(env));
                let mut intersect = Vec::new(env);
                for id in ids.iter() {
                    if posting.iter().any(|p| p == id) {
                        intersect.push_back(id);
                    }
                }
                ids = intersect;
            }
            ids
        } else if !query.category_filters.is_empty() {
            let mut union = Vec::new(env);
            for cat in query.category_filters.iter() {
                let posting: Vec<u64> = env
                    .storage()
                    .persistent()
                    .get(&DataKey::CategoryPosting(cat))
                    .unwrap_or(Vec::new(env));
                for id in posting.iter() {
                    if !union.iter().any(|x| x == id) {
                        union.push_back(id);
                    }
                }
            }
            union
        } else {
            env.storage()
                .persistent()
                .get(&DataKey::IndexedIds)
                .unwrap_or(Vec::new(env))
        }
    }

    fn entry_matches(
        query: &SearchQuery,
        entry: &SearchIndexEntry,
        caller: &Address,
        env: &Env,
    ) -> bool {
        if entry.quality_score_bps < query.min_quality_bps {
            return false;
        }
        if query.from_timestamp > 0 && entry.created_at < query.from_timestamp {
            return false;
        }
        if query.to_timestamp > 0 && entry.created_at > query.to_timestamp {
            return false;
        }
        if let Some(patient) = query.patient_filter.clone() {
            if patient != entry.patient {
                return false;
            }
        }
        if !query.category_filters.is_empty()
            && !query
                .category_filters
                .iter()
                .any(|x| x == entry.category_hash)
        {
            return false;
        }
        if !query.network_filters.is_empty()
            && !query.network_filters.iter().any(|x| x == entry.network)
        {
            return false;
        }
        for attr in query.attribute_filters.iter() {
            if !entry.attribute_hashes.iter().any(|x| x == attr) {
                return false;
            }
        }
        if entry.is_confidential {
            if !query.include_confidential {
                return false;
            }
            if Self::require_role(env, caller, ROLE_CONFIDENTIAL).is_err() {
                return false;
            }
        }
        true
    }

    fn compute_score(env: &Env, query: &SearchQuery, entry: &SearchIndexEntry) -> u32 {
        let ranking: RankingConfig =
            env.storage()
                .persistent()
                .get(&DataKey::Ranking)
                .unwrap_or(RankingConfig {
                    required_weight_bps: 4_000,
                    optional_weight_bps: 2_000,
                    recency_weight_bps: 2_000,
                    quality_weight_bps: 2_000,
                });
        let total_weight = ranking
            .required_weight_bps
            .saturating_add(ranking.optional_weight_bps)
            .saturating_add(ranking.recency_weight_bps)
            .saturating_add(ranking.quality_weight_bps);
        if total_weight == 0 {
            return 0;
        }

        let required_score = if query.required_tokens.is_empty() {
            10_000
        } else {
            let mut matched = 0u32;
            for t in query.required_tokens.iter() {
                if entry.token_hashes.iter().any(|x| x == t) {
                    matched = matched.saturating_add(1);
                }
            }
            matched
                .saturating_mul(10_000)
                .checked_div(query.required_tokens.len())
                .unwrap_or(0)
        };

        let optional_score = if query.optional_tokens.is_empty() {
            0
        } else {
            let mut matched = 0u32;
            for t in query.optional_tokens.iter() {
                if entry.token_hashes.iter().any(|x| x == t) {
                    matched = matched.saturating_add(1);
                }
            }
            matched
                .saturating_mul(10_000)
                .checked_div(query.optional_tokens.len())
                .unwrap_or(0)
        };

        let age_seconds = env.ledger().timestamp().saturating_sub(entry.created_at);
        let age_days = age_seconds / 86_400;
        let recency_penalty = age_days.saturating_mul(80);
        let recency_score = 10_000u64.saturating_sub(recency_penalty).min(10_000) as u32;

        let quality_score = entry.quality_score_bps.min(10_000);

        let weighted = required_score
            .saturating_mul(ranking.required_weight_bps)
            .saturating_add(optional_score.saturating_mul(ranking.optional_weight_bps))
            .saturating_add(recency_score.saturating_mul(ranking.recency_weight_bps))
            .saturating_add(quality_score.saturating_mul(ranking.quality_weight_bps));
        weighted.checked_div(total_weight).unwrap_or(0)
    }

    fn insert_ranked(env: &Env, ranked: &mut Vec<SearchResult>, candidate: SearchResult) {
        let mut rebuilt = Vec::new(env);
        let mut inserted = false;
        for current in ranked.iter() {
            if !inserted && candidate.score_bps > current.score_bps {
                rebuilt.push_back(candidate.clone());
                inserted = true;
            }
            rebuilt.push_back(current);
        }
        if !inserted {
            rebuilt.push_back(candidate);
        }
        *ranked = rebuilt;
    }

    fn paginate_results(
        env: &Env,
        results: &Vec<SearchResult>,
        page: u32,
        page_size: u32,
    ) -> Vec<SearchResult> {
        let start = page.saturating_mul(page_size);
        let mut out = Vec::new(env);
        let end = start.saturating_add(page_size);
        for i in start..end {
            if let Some(result) = results.get(i) {
                out.push_back(result);
            } else {
                break;
            }
        }
        out
    }

    fn upsert_cache(
        env: &Env,
        query_hash: BytesN<32>,
        results: Vec<SearchResult>,
    ) -> Result<(), Error> {
        if results.is_empty() {
            return Ok(());
        }

        let policy: CachePolicy = env
            .storage()
            .persistent()
            .get(&DataKey::CachePolicy)
            .unwrap_or(CachePolicy {
                ttl_seconds: 600,
                max_entries: 50,
            });
        let now = env.ledger().timestamp();
        let cache = QueryCacheEntry {
            query_hash: query_hash.clone(),
            created_at: now,
            expires_at: now.saturating_add(policy.ttl_seconds),
            hit_count: 0,
            results,
        };
        env.storage()
            .persistent()
            .set(&DataKey::Cache(query_hash.clone()), &cache);

        let mut order: Vec<BytesN<32>> = env
            .storage()
            .persistent()
            .get(&DataKey::CacheOrder)
            .unwrap_or(Vec::new(env));
        let mut trimmed = Vec::new(env);
        for key in order.iter() {
            if key != query_hash {
                trimmed.push_back(key);
            }
        }
        trimmed.push_back(query_hash.clone());
        order = trimmed;

        if order.len() > policy.max_entries {
            let mut to_remove = order.len().saturating_sub(policy.max_entries);
            let mut keep = Vec::new(env);
            for key in order.iter() {
                if to_remove > 0 {
                    env.storage().persistent().remove(&DataKey::Cache(key));
                    to_remove = to_remove.saturating_sub(1);
                } else {
                    keep.push_back(key);
                }
            }
            order = keep;
        }
        env.storage().persistent().set(&DataKey::CacheOrder, &order);
        Ok(())
    }

    fn append_audit(
        env: &Env,
        caller: Address,
        query_hash: BytesN<32>,
        result_count: u32,
        from_cache: bool,
        granted: bool,
    ) {
        let query_id = Self::next_query_id(env);
        let entry = SearchAuditEntry {
            query_id,
            caller,
            query_hash,
            timestamp: env.ledger().timestamp(),
            result_count,
            from_cache,
            granted,
        };
        env.storage()
            .persistent()
            .set(&DataKey::Audit(query_id), &entry);
        env.events().publish(
            (symbol_short!("SRCH_AUD"),),
            (query_id, result_count, from_cache),
        );
    }

    fn query_hash(env: &Env, query: &SearchQuery) -> BytesN<32> {
        env.crypto().sha256(&query.clone().to_xdr(env)).into()
    }

    fn next_query_id(env: &Env) -> u64 {
        let current: u64 = env.storage().instance().get(&QUERY_ID).unwrap_or(1);
        env.storage()
            .instance()
            .set(&QUERY_ID, &current.saturating_add(1));
        current
    }

    fn require_initialized(env: &Env) -> Result<(), Error> {
        if !env.storage().instance().has(&ADMIN) {
            return Err(Error::NotInitialized);
        }
        Ok(())
    }

    fn require_admin(env: &Env, caller: &Address) -> Result<(), Error> {
        Self::require_initialized(env)?;
        let admin: Address = env
            .storage()
            .instance()
            .get(&ADMIN)
            .ok_or(Error::NotInitialized)?;
        if admin != *caller {
            return Err(Error::NotAuthorized);
        }
        Ok(())
    }

    fn require_role(env: &Env, caller: &Address, role: u32) -> Result<(), Error> {
        Self::require_initialized(env)?;
        let admin: Address = env
            .storage()
            .instance()
            .get(&ADMIN)
            .ok_or(Error::NotInitialized)?;
        if admin == *caller {
            return Ok(());
        }
        let mask: u32 = env
            .storage()
            .persistent()
            .get(&DataKey::Roles(caller.clone()))
            .unwrap_or(0u32);
        if (mask & role) == 0 {
            return Err(Error::NotAuthorized);
        }
        Ok(())
    }

    fn require_not_paused(env: &Env) -> Result<(), Error> {
        if env.storage().instance().get(&PAUSED).unwrap_or(false) {
            return Err(Error::ContractPaused);
        }
        Ok(())
    }
}
