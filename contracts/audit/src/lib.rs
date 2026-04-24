#![no_std]

pub mod errors;
pub mod querying;
pub mod storage;
pub mod types;
pub mod verification;

#[cfg(test)]
mod test;

use crate::errors::Error;
use crate::types::{AuditConfig, AuditRecord, AuditSummary, AuditType, DataKey};
use soroban_sdk::{
    contract, contractimpl, symbol_short, Address, Bytes, BytesN, Env, Map, String, Symbol, Vec,
};

#[contract]
pub struct AuditTrail;

#[contractimpl]
impl AuditTrail {
    pub fn initialize(env: Env, admin: Address, config: AuditConfig) -> Result<(), Error> {
        if env.storage().instance().has(&DataKey::Admin) {
            return Err(Error::AlreadyInitialized);
        }
        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage().instance().set(&DataKey::Config, &config);
        env.storage().instance().set(&DataKey::RecordCount, &0u64);
        env.storage()
            .instance()
            .set(&DataKey::RollingHash, &BytesN::from_array(&env, &[0u8; 32]));
        Ok(())
    }

    pub fn record_event(
        env: Env,
        actor: Address,
        audit_type: AuditType,
        target: Option<Address>,
        action_data: Bytes,
        previous_hash: Option<BytesN<32>>,
        current_hash: BytesN<32>,
        metadata: Map<String, String>,
    ) -> Result<u64, Error> {
        if !env.storage().instance().has(&DataKey::Admin) {
            return Err(Error::NotInitialized);
        }
        actor.require_auth();

        let id = Self::next_id(&env, &DataKey::RecordCount);
        let action_hash = env.crypto().sha256(&action_data).into();

        let record = AuditRecord {
            id,
            timestamp: env.ledger().timestamp(),
            actor: actor.clone(),
            audit_type,
            target_contract: target.clone(),
            action_hash,
            previous_state_hash: previous_hash,
            current_state_hash: current_hash,
            metadata,
        };

        env.storage()
            .persistent()
            .set(&DataKey::Record(id), &record);

        Self::update_rolling_hash(&env, &record);
        Self::save_index(&env, &actor, target, id);

        env.events().publish(
            (symbol_short!("AUDIT"), symbol_short!("LOG")),
            (id, record.audit_type, record.actor),
        );

        Ok(id)
    }

    pub fn get_record(env: Env, id: u64) -> Result<AuditRecord, Error> {
        env.storage()
            .persistent()
            .get(&DataKey::Record(id))
            .ok_or(Error::RecordNotFound)
    }

    pub fn verify_integrity(env: Env) -> BytesN<32> {
        env.storage()
            .instance()
            .get(&DataKey::RollingHash)
            .unwrap_or(BytesN::from_array(&env, &[0u8; 32]))
    }

    pub fn generate_summary(env: Env, start: u64, end: u64) -> AuditSummary {
        let count = env
            .storage()
            .instance()
            .get(&DataKey::RecordCount)
            .unwrap_or(0u64);
        let mut total = 0u64;
        let mut events = 0u32;
        let mut admins = 0u32;

        for i in 0..count {
            if let Some(record) = env
                .storage()
                .persistent()
                .get::<DataKey, AuditRecord>(&DataKey::Record(i))
            {
                if record.timestamp >= start && record.timestamp <= end {
                    total += 1;
                    match record.audit_type {
                        AuditType::Event => events += 1,
                        AuditType::AdminAction => admins += 1,
                        _ => {}
                    }
                }
            }
        }

        AuditSummary {
            start_time: start,
            end_time: end,
            total_records: total,
            event_count: events,
            admin_action_count: admins,
            root_hash: Self::verify_integrity(env),
        }
    }

    fn update_rolling_hash(env: &Env, record: &AuditRecord) {
        let mut current_rolling: BytesN<32> =
            env.storage().instance().get(&DataKey::RollingHash).unwrap();

        let mut buffer = soroban_sdk::Bytes::new(env);
        buffer.append(&soroban_sdk::Bytes::from_slice(env, &current_rolling.to_array()));
        buffer.append(&soroban_sdk::Bytes::from_slice(env, &record.id.to_be_bytes()));
        buffer.append(&soroban_sdk::Bytes::from_slice(env, &record.action_hash.to_array()));

        let new_hash: BytesN<32> = env.crypto().sha256(&buffer).into();
        env.storage()
            .instance()
            .set(&DataKey::RollingHash, &new_hash);
    }

    fn save_index(env: &Env, user: &Address, contract: Option<Address>, id: u64) {
        let mut user_list: Vec<u64> = env
            .storage()
            .persistent()
            .get(&DataKey::UserAudits(user.clone()))
            .unwrap_or(Vec::new(env));
        user_list.push_back(id);
        env.storage()
            .persistent()
            .set(&DataKey::UserAudits(user.clone()), &user_list);

        if let Some(c) = contract {
            let mut contract_list: Vec<u64> = env
                .storage()
                .persistent()
                .get(&DataKey::ContractAudits(c.clone()))
                .unwrap_or(Vec::new(env));
            contract_list.push_back(id);
            env.storage()
                .persistent()
                .set(&DataKey::ContractAudits(c), &contract_list);
        }
    }

    fn next_id(env: &Env, key: &DataKey) -> u64 {
        let current: u64 = env.storage().instance().get(key).unwrap_or(0);
        let next = current.saturating_add(1);
        env.storage().instance().set(key, &next);
        next
    }
}
