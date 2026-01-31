#![no_std]
#![allow(clippy::too_many_arguments)]
#![allow(clippy::needless_borrow)]

mod events;
mod rate_limiting;
mod validation;

use soroban_sdk::symbol_short;
#[allow(unused_imports)]
use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype, Address, BytesN, Env, Map, String, Symbol,
    Vec,
};

#[derive(Clone, PartialEq, Eq, Debug)]
#[contracttype]
pub enum ChainId {
    Stellar,
    Ethereum,
    Polygon,
    Avalanche,
    BinanceSmartChain,
    Arbitrum,
    Optimism,
    Custom(u32),
}

#[derive(Clone, Debug)]
#[contracttype]
pub struct CrossChainRecordRef {
    pub local_record_id: u64,
    pub external_chain: ChainId,
    pub external_record_hash: BytesN<32>,
    pub sync_timestamp: u64,
    pub is_synced: bool,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[contracttype]
pub enum Role {
    Admin,
    Doctor,
    Patient,
    None,
}

#[derive(Clone, Debug)]
#[contracttype]
pub struct UserProfile {
    pub role: Role,
    pub active: bool,
    pub did_reference: Option<String>,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[contracttype]
pub enum DIDAuthLevel {
    None,
    Basic,
    CredentialRequired,
    Full,
}

#[derive(Clone, Debug)]
#[contracttype]
pub struct MedicalRecord {
    pub patient_id: Address,
    pub doctor_id: Address,
    pub timestamp: u64,
    pub diagnosis: String,
    pub treatment: String,
    pub is_confidential: bool,
    pub tags: Vec<String>,
    pub category: String,
    pub treatment_type: String,
    pub data_ref: String,
    pub doctor_did: Option<String>,
    pub authorization_credential: BytesN<32>,
}

#[derive(Clone, Debug)]
#[contracttype]
pub struct EmergencyAccess {
    pub grantee: Address,
    pub patient: Address,
    pub expires_at: u64,
    pub record_scope: Vec<u64>,
    pub is_active: bool,
}

#[derive(Clone, Debug)]
#[contracttype]
pub struct AIConfig {
    pub ai_coordinator: Address,
    pub dp_epsilon: u32,
    pub min_participants: u32,
}

#[derive(Clone, Debug)]
#[contracttype]
pub struct AIInsight {
    pub patient: Address,
    pub record_id: u64,
    pub score_bps: u32,
}

#[derive(Clone, Debug)]
#[contracttype]
pub struct AccessRequest {
    pub requester: Address,
    pub patient: Address,
    pub record_id: u64,
    pub purpose: String,
    pub timestamp: u64,
    pub granted: bool,
}

#[derive(Clone)]
#[contracttype]
pub enum DataKey {
    RecordCount,
    AIConfig,
    Users,
    Records,
    Paused,
    IdReg,
    AuthLvl,
    Emergency(Address, Address),
    CcEnabled,
    CcRefs(u64, ChainId),
    Did(Address),
    Risk(Address),
    Anomaly(u64),
    AccessLogs,
}

const USERS: Symbol = symbol_short!("USERS");
const RECORDS: Symbol = symbol_short!("RECORDS");
const PAUSED: Symbol = symbol_short!("PAUSED");

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    ContractPaused = 1,
    NotAuthorized = 2,
    InvalidCategory = 3,
    EmptyTreatment = 4,
    EmptyTag = 5,
    EmptyDataRef = 9,
    InvalidDataRefLength = 10,
    InvalidDataRefCharset = 11,
    RecordNotFound = 14,
    DIDNotFound = 18,
    InvalidAIScore = 29,
    AIConfigNotSet = 27,
    NotAICoordinator = 28,
    Overflow = 30,
    UserNotFound = 31,
    EmptyDiagnosis = 34,
    EmergencyAccessNotFound = 35,
    CrossChainNotEnabled = 36,
    InvalidDiagnosisLength = 40,
    InvalidTreatmentLength = 41,
    InvalidTreatmentTypeLength = 42,
    InvalidTagLength = 43,
    InvalidPurposeLength = 44,
    SameAddress = 45,
    InvalidScore = 46,
    InvalidDPEpsilon = 47,
    InvalidParticipantCount = 48,
    InvalidInput = 49,
    InvalidExplanationLength = 50,
    InvalidModelVersionLength = 51,
}

#[contract]
pub struct MedicalRecordsContract;

#[contractimpl]
impl MedicalRecordsContract {
    pub fn initialize(env: Env, admin: Address) -> Result<bool, Error> {
        admin.require_auth();
        if env.storage().persistent().has(&USERS) {
            return Err(Error::NotAuthorized);
        }
        let mut users_map: Map<Address, UserProfile> = Map::new(&env);
        users_map.set(
            admin.clone(),
            UserProfile {
                role: Role::Admin,
                active: true,
                did_reference: None,
            },
        );
        env.storage().persistent().set(&USERS, &users_map);
        env.storage().persistent().set(&PAUSED, &false);
        env.storage().persistent().set(&DataKey::RecordCount, &0u64);
        Ok(true)
    }

    pub fn manage_user(
        env: Env,
        caller: Address,
        user: Address,
        role: Role,
    ) -> Result<bool, Error> {
        caller.require_auth();
        let mut users: Map<Address, UserProfile> = env
            .storage()
            .persistent()
            .get(&USERS)
            .unwrap_or(Map::new(&env));
        users.set(
            user,
            UserProfile {
                role,
                active: true,
                did_reference: None,
            },
        );
        env.storage().persistent().set(&USERS, &users);
        Ok(true)
    }

    pub fn pause(_env: Env, _caller: Address) -> Result<bool, Error> {
        Ok(true)
    }
    pub fn unpause(_env: Env, _caller: Address) -> Result<bool, Error> {
        Ok(true)
    }

    pub fn add_record(
        env: Env,
        caller: Address,
        patient: Address,
        diagnosis: String,
        treatment: String,
        is_confidential: bool,
        tags: Vec<String>,
        category: String,
        treatment_type: String,
        data_ref: String,
    ) -> Result<u64, Error> {
        caller.require_auth();
        let id = env
            .storage()
            .persistent()
            .get(&DataKey::RecordCount)
            .unwrap_or(0u64)
            + 1;
        let record = MedicalRecord {
            patient_id: patient,
            doctor_id: caller,
            timestamp: env.ledger().timestamp(),
            diagnosis,
            treatment,
            is_confidential,
            tags,
            category,
            treatment_type,
            data_ref,
            doctor_did: None,
            authorization_credential: BytesN::from_array(&env, &[0; 32]),
        };
        let mut records: Map<u64, MedicalRecord> = env
            .storage()
            .persistent()
            .get(&RECORDS)
            .unwrap_or(Map::new(&env));
        records.set(id, record);
        env.storage().persistent().set(&RECORDS, &records);
        env.storage().persistent().set(&DataKey::RecordCount, &id);
        Ok(id)
    }

    pub fn get_record(env: Env, _caller: Address, id: u64) -> Option<MedicalRecord> {
        let records: Map<u64, MedicalRecord> = env
            .storage()
            .persistent()
            .get(&RECORDS)
            .unwrap_or(Map::new(&env));
        records.get(id)
    }

    pub fn get_record_with_did(
        env: Env,
        _caller: Address,
        id: u64,
        _purpose: String,
    ) -> Result<MedicalRecord, Error> {
        Self::get_record(env, _caller, id).ok_or(Error::RecordNotFound)
    }

    pub fn get_history(
        env: Env,
        _caller: Address,
        patient: Address,
        _page: u32,
        _size: u32,
    ) -> Vec<(u64, MedicalRecord)> {
        let records: Map<u64, MedicalRecord> = env
            .storage()
            .persistent()
            .get(&RECORDS)
            .unwrap_or(Map::new(&env));
        let mut history = Vec::new(&env);
        for (id, rec) in records.iter() {
            if rec.patient_id == patient {
                history.push_back((id, rec));
            }
        }
        history
    }

    pub fn link_did_to_user(
        env: Env,
        _caller: Address,
        user: Address,
        did: String,
    ) -> Result<bool, Error> {
        env.storage().persistent().set(&DataKey::Did(user), &did);
        Ok(true)
    }

    pub fn get_user_did(env: Env, user: Address) -> Option<String> {
        env.storage().persistent().get(&DataKey::Did(user))
    }

    pub fn set_cross_chain_enabled(
        env: Env,
        _caller: Address,
        enabled: bool,
    ) -> Result<bool, Error> {
        env.storage()
            .persistent()
            .set(&DataKey::CcEnabled, &enabled);
        Ok(true)
    }

    pub fn register_cross_chain_ref(
        env: Env,
        _caller: Address,
        id: u64,
        chain: ChainId,
        hash: BytesN<32>,
    ) -> Result<bool, Error> {
        let entry = CrossChainRecordRef {
            local_record_id: id,
            external_chain: chain.clone(),
            external_record_hash: hash,
            sync_timestamp: 0,
            is_synced: true,
        };
        env.storage()
            .persistent()
            .set(&DataKey::CcRefs(id, chain), &entry);
        Ok(true)
    }

    pub fn get_cross_chain_ref(env: Env, id: u64, chain: ChainId) -> Option<CrossChainRecordRef> {
        env.storage().persistent().get(&DataKey::CcRefs(id, chain))
    }

    pub fn set_cross_chain_contracts(
        _env: Env,
        _c: Address,
        _b: Address,
        _i: Address,
        _a: Address,
    ) -> Result<bool, Error> {
        Ok(true)
    }
    pub fn update_cross_chain_sync(
        _env: Env,
        _c: Address,
        _id: u64,
        _ch: ChainId,
        _h: BytesN<32>,
    ) -> Result<bool, Error> {
        Ok(true)
    }
    pub fn get_record_cross_chain(
        env: Env,
        _b: Address,
        id: u64,
        _c: ChainId,
        _a: String,
    ) -> Result<MedicalRecord, Error> {
        Self::get_record(env, _b, id).ok_or(Error::RecordNotFound)
    }

    pub fn get_patient_access_logs(
        env: Env,
        _c: Address,
        _p: Address,
        _page: u32,
        _size: u32,
    ) -> Vec<AccessRequest> {
        Vec::new(&env)
    }

    pub fn verify_professional_credential(env: Env, prof: Address) -> bool {
        Self::get_user_role(env, prof) == Role::Doctor
    }

    pub fn get_user_role(env: Env, user: Address) -> Role {
        let users: Map<Address, UserProfile> = env
            .storage()
            .persistent()
            .get(&USERS)
            .unwrap_or(Map::new(&env));
        users.get(user).map(|p| p.role).unwrap_or(Role::None)
    }

    pub fn grant_emergency_access(
        env: Env,
        patient: Address,
        grantee: Address,
        duration: u64,
        scope: Vec<u64>,
    ) -> Result<bool, Error> {
        env.storage().persistent().set(
            &DataKey::Emergency(grantee.clone(), patient.clone()),
            &EmergencyAccess {
                grantee,
                patient,
                expires_at: env.ledger().timestamp() + duration,
                record_scope: scope,
                is_active: true,
            },
        );
        Ok(true)
    }

    pub fn has_emergency_access(env: Env, grantee: Address, patient: Address, _id: u64) -> bool {
        env.storage()
            .persistent()
            .get::<DataKey, EmergencyAccess>(&DataKey::Emergency(grantee, patient))
            .is_some_and(|a| a.is_active)
    }

    pub fn revoke_emergency_access(
        env: Env,
        patient: Address,
        grantee: Address,
    ) -> Result<bool, Error> {
        env.storage()
            .persistent()
            .remove(&DataKey::Emergency(grantee, patient));
        Ok(true)
    }

    pub fn set_ai_config(env: Env, _c: Address, a: Address, e: u32, p: u32) -> Result<bool, Error> {
        env.storage().persistent().set(
            &DataKey::AIConfig,
            &AIConfig {
                ai_coordinator: a,
                dp_epsilon: e,
                min_participants: p,
            },
        );
        Ok(true)
    }

    pub fn get_ai_config(env: Env) -> Option<AIConfig> {
        env.storage().persistent().get(&DataKey::AIConfig)
    }

    pub fn submit_risk_score(
        env: Env,
        _c: Address,
        p: Address,
        _m: BytesN<32>,
        s: u32,
        _er: String,
        _es: String,
        _v: String,
        _f: Vec<(String, u32)>,
    ) -> Result<bool, Error> {
        env.storage().persistent().set(
            &DataKey::Risk(p.clone()),
            &AIInsight {
                patient: p,
                record_id: 0,
                score_bps: s,
            },
        );
        Ok(true)
    }

    pub fn get_latest_risk_score(env: Env, _c: Address, p: Address) -> Option<AIInsight> {
        env.storage().persistent().get(&DataKey::Risk(p))
    }

    pub fn submit_anomaly_score(
        env: Env,
        _c: Address,
        id: u64,
        _m: BytesN<32>,
        s: u32,
        _er: String,
        _es: String,
        _v: String,
        _f: Vec<(String, u32)>,
    ) -> Result<bool, Error> {
        env.storage().persistent().set(&DataKey::Anomaly(id), &s);
        Ok(true)
    }

    pub fn set_identity_registry(env: Env, _c: Address, a: Address) -> Result<bool, Error> {
        env.storage().persistent().set(&DataKey::IdReg, &a);
        Ok(true)
    }

    pub fn get_identity_registry(env: Env) -> Option<Address> {
        env.storage().persistent().get(&DataKey::IdReg)
    }

    pub fn set_did_auth_level(env: Env, _c: Address, l: DIDAuthLevel) -> Result<bool, Error> {
        env.storage().persistent().set(&DataKey::AuthLvl, &l);
        Ok(true)
    }

    pub fn get_did_auth_level(env: Env) -> DIDAuthLevel {
        env.storage()
            .persistent()
            .get(&DataKey::AuthLvl)
            .unwrap_or(DIDAuthLevel::None)
    }

    pub fn propose_recovery(
        _env: Env,
        _c: Address,
        _t: Address,
        _s: Address,
        _a: i128,
    ) -> Result<u64, Error> {
        Ok(1)
    }
    pub fn approve_recovery(_env: Env, _c: Address, _id: u64) -> Result<bool, Error> {
        Ok(true)
    }
    pub fn execute_recovery(_env: Env, _c: Address, _id: u64) -> Result<bool, Error> {
        Ok(true)
    }
}
