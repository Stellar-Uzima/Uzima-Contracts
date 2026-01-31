#![no_std]
#![allow(unused_variables)]
#![allow(dead_code)] // <--- NEW LINE 1
#![allow(clippy::enum_variant_names)] // <--- NEW LINE 2
#![allow(clippy::arithmetic_side_effects)]
#![allow(clippy::needless_borrow)]
#![allow(clippy::too_many_arguments)]

mod events;
mod rate_limiting;
mod validation;
};
use upgradeability::storage::{ADMIN as UPGRADE_ADMIN, VERSION};

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

#[derive(Clone)]
#[contracttype]
}

// ==================== Constants ====================

const APPROVAL_THRESHOLD: u32 = 2;
const TIMELOCK_SECS: u64 = 86_400;

const CHAIN_LIST_LEN: usize = 6;

// ==================== Contract ====================

#[contract]
pub struct MedicalRecordsContract;

#[contractimpl]
impl MedicalRecordsContract {
        let mut users: Map<Address, UserProfile> = Map::new(&env);
        users.set(
            admin.clone(),
            UserProfile {
                role: Role::Admin,
                active: true,
                did_reference: None,
            },
        );
    }

    pub fn manage_user(
        env: Env,
        caller: Address,
        user: Address,
        role: Role,
    ) -> Result<bool, Error> {
        caller.require_auth();

    pub fn unpause(env: Env, admin: Address) -> Result<bool, Error> {
        admin.require_auth();
        env.storage().persistent().set(&PAUSED_FLAG, &false);
        Ok(true)
    }

    }

    pub fn link_did_to_user(
        env: Env,

    pub fn get_user_did(env: Env, user: Address) -> Option<String> {
        env.storage().persistent().get(&DataKey::Did(user))
    }

    pub fn get_ai_config(env: Env) -> Option<AIConfig> {
        env.storage().persistent().get(&DataKey::AIConfig)
    }

    pub fn set_ai_config(
        env: Env,
        admin: Address,
        ai_coord: Address,
        dp: u32,
        min: u32,
    ) -> Result<bool, Error> {
        admin.require_auth();
        env.storage().persistent().set(
            &DataKey::AIConfig,
            &AIConfig {
                ai_coordinator: ai_coord,
                dp_epsilon: dp,
                min_participants: min,
            },
        );
        Ok(true)
    }

        env: Env,
        caller: Address,
        _patient: Address,
        _record_id: u64,
        _model_id: BytesN<32>,
        _score: u32,
        _cat: String,
        _ver: String,
        _report: String,
        _feat: Vec<(String, u32)>,
    ) -> Result<bool, Error> {
        caller.require_auth();
    }

    pub fn version(env: Env) -> u32 {
        env.storage().instance().get(&VERSION).unwrap_or(0)
    }

    // ---------------------------------------------------------------------
    // Internal helpers
    // ---------------------------------------------------------------------

    fn require_initialized(env: &Env) -> Result<(), Error> {
        if env.storage().instance().has(&UPGRADE_ADMIN) {
            Ok(())
        } else {
            Err(Error::NotInitialized)
        }
    }

    fn require_not_paused(env: &Env) -> Result<(), Error> {
        let paused: bool = env
            .storage()
            .persistent()
            .get(&DataKey::Paused)
            .unwrap_or(false);
        if paused {
            Err(Error::ContractPaused)
        } else {
            Ok(())
        }
    }

    fn read_users(env: &Env) -> Map<Address, UserProfile> {
        env.storage()
            .persistent()
            .get(&DataKey::Users)
            .unwrap_or(Map::new(env))
    }

    fn is_admin(env: &Env, address: &Address) -> bool {
        match Self::read_users(env).get(address.clone()) {
            Some(profile) => matches!(profile.role, Role::Admin) && profile.active,
            None => false,
        }
    }

    fn is_active_doctor(env: &Env, address: &Address) -> bool {
        match Self::read_users(env).get(address.clone()) {
            Some(profile) => matches!(profile.role, Role::Doctor) && profile.active,
            None => false,
        }
    }

    fn is_active_patient(env: &Env, address: &Address) -> bool {
        match Self::read_users(env).get(address.clone()) {
            Some(profile) => matches!(profile.role, Role::Patient) && profile.active,
            None => false,
        }
    }

    fn require_admin(env: &Env, caller: &Address) -> Result<(), Error> {
        if Self::is_admin(env, caller) {
            Ok(())
        } else {
            Err(Error::NotAuthorized)
        }
    }

    fn require_active_doctor(env: &Env, caller: &Address) -> Result<(), Error> {
        if Self::is_active_doctor(env, caller) {
            Ok(())
        } else {
            Err(Error::NotAuthorized)
        }
    }

    fn require_active_patient(env: &Env, patient: &Address) -> Result<(), Error> {
        if Self::is_active_patient(env, patient) {
            Ok(())
        } else {
            Err(Error::NotAuthorized)
        }
    }

    fn next_id(env: &Env) -> u64 {
        let current: u64 = env
            .storage()
            .persistent()
            .get(&DataKey::NextId)
            .unwrap_or(0);
        let next = current.saturating_add(1);
        env.storage().persistent().set(&DataKey::NextId, &next);
        next
    }

    fn increment_record_count(env: &Env) {
        let current: u64 = env
            .storage()
            .persistent()
            .get(&DataKey::RecordCount)
            .unwrap_or(0);
        env.storage()
            .persistent()
            .set(&DataKey::RecordCount, &current.saturating_add(1));
    }

    fn store_record(
        env: &Env,
        record_id: u64,
        record: &MedicalRecord,
        category: &String,
        is_confidential: bool,
    ) {
        env.storage()
            .persistent()
            .set(&DataKey::Record(record_id), record);

        // Lightweight hash anchor: unique per record id (sufficient for tests; off-chain can use stronger binding).
        let mut payload = Bytes::new(env);
        payload.append(&Bytes::from_slice(env, &record_id.to_be_bytes()));
        payload.append(&Bytes::from_slice(env, &record.timestamp.to_be_bytes()));
        let record_hash: BytesN<32> = env.crypto().sha256(&payload).into();

        let meta = RecordMetadata {
            record_id,
            patient_id: record.patient_id.clone(),
            timestamp: record.timestamp,
            category: category.clone(),
            is_confidential,
            record_hash,
        };
        env.storage()
            .persistent()
            .set(&DataKey::RecordMeta(record_id), &meta);
    }

    fn append_patient_record(env: &Env, patient: &Address, record_id: u64) {
        let mut ids: Vec<u64> = env
            .storage()
            .persistent()
            .get(&DataKey::PatientRecords(patient.clone()))
            .unwrap_or(Vec::new(env));
        ids.push_back(record_id);
        env.storage()
            .persistent()
            .set(&DataKey::PatientRecords(patient.clone()), &ids);
    }

    fn has_emergency_access_internal(
        env: &Env,
        grantee: &Address,
        patient: &Address,
        record_id: u64,
    ) -> bool {
        let now = env.ledger().timestamp();
        let grants: Map<Address, EmergencyAccess> = env
            .storage()
            .persistent()
            .get(&DataKey::PatientEmergencyGrants(patient.clone()))
            .unwrap_or(Map::new(env));
        let grant = match grants.get(grantee.clone()) {
            Some(g) => g,
            None => return false,
        };
        if !grant.is_active {
            return false;
        }
        if grant.expires_at <= now {
            return false;
        }
        if grant.record_scope.is_empty() {
            return true;
        }
        grant.record_scope.contains(record_id)
    }

    fn can_view_record(
        env: &Env,
        caller: &Address,
        record: &MedicalRecord,
        record_id: u64,
    ) -> bool {
        if Self::is_admin(env, caller) {
            return true;
        }
        if *caller == record.patient_id {
            return true;
        }
        if *caller == record.doctor_id {
            return true;
        }
        if Self::has_emergency_access_internal(env, caller, &record.patient_id, record_id) {
            return true;
        }
        if record.is_confidential {
            Self::check_permission(env, caller, Permission::ReadConfidential)
        } else {
            Self::check_permission(env, caller, Permission::ReadRecord)
        }
    }

    fn log_access(
        env: &Env,
        patient: &Address,
        record_id: u64,
        requester: &Address,
        purpose: &String,
        granted: bool,
    ) {
        let now = env.ledger().timestamp();

        let current: u64 = env
            .storage()
            .persistent()
            .get(&DataKey::AccessLogCount)
            .unwrap_or(0);
        let next = current.saturating_add(1);
        env.storage()
            .persistent()
            .set(&DataKey::AccessLogCount, &next);

        let entry = AccessRequest {
            requester: requester.clone(),
            patient: patient.clone(),
            record_id,
            purpose: purpose.clone(),
            timestamp: now,
            granted,
        };
        env.storage()
            .persistent()
            .set(&DataKey::AccessLog(next), &entry);

        let pc: u64 = env
            .storage()
            .persistent()
            .get(&DataKey::PatientAccessLogCount(patient.clone()))
            .unwrap_or(0);
        let pnext = pc.saturating_add(1);
        env.storage()
            .persistent()
            .set(&DataKey::PatientAccessLogCount(patient.clone()), &pnext);
        env.storage()
            .persistent()
            .set(&DataKey::PatientAccessLog(patient.clone(), pnext), &next);
    }

    fn require_cross_chain_contracts(env: &Env) -> Result<Address, Error> {
        let bridge: Address = env
            .storage()
            .persistent()
            .get(&DataKey::BridgeContract)
            .ok_or(Error::CrossChainContractsNotSet)?;
        if !env
            .storage()
            .persistent()
            .has(&DataKey::CrossChainIdentityContract)
        {
            return Err(Error::CrossChainContractsNotSet);
        }
        if !env
            .storage()
            .persistent()
            .has(&DataKey::CrossChainAccessContract)
        {
            return Err(Error::CrossChainContractsNotSet);
        }
        Ok(bridge)
    }

    fn require_crypto_registry(env: &Env) -> Result<Address, Error> {
        env.storage()
            .persistent()
            .get(&DataKey::CryptoRegistry)
            .ok_or(Error::CryptoRegistryNotSet)
    }

    fn is_encryption_required_internal(env: &Env) -> bool {
        env.storage()
            .persistent()
            .get(&DataKey::EncryptionRequired)
            .unwrap_or(false)
    }

    fn is_require_pq_envelopes_internal(env: &Env) -> bool {
        env.storage()
            .persistent()
            .get(&DataKey::RequirePqEnvelopes)
            .unwrap_or(false)
    }

    fn can_view_encrypted_record(
        env: &Env,
        caller: &Address,
        record: &EncryptedRecord,
        record_id: u64,
    ) -> bool {
        if Self::is_admin(env, caller) {
            return true;
        }
        if *caller == record.patient_id {
            return true;
        }
        if *caller == record.doctor_id {
            return true;
        }
        if Self::is_active_doctor(env, caller) && !record.is_confidential {
            return true;
        }
        if Self::has_emergency_access_internal(env, caller, &record.patient_id, record_id) {
            return true;
        }
        false
    }

    fn log_crypto_event(
        env: &Env,
        actor: &Address,
        action: CryptoAuditAction,
        record_id: Option<u64>,
        details_hash: BytesN<32>,
        details_ref: Option<String>,
    ) {
        let current: u64 = env
            .storage()
            .persistent()
            .get(&DataKey::CryptoAuditCount)
            .unwrap_or(0);
        let next = current.saturating_add(1);
        env.storage()
            .persistent()
            .set(&DataKey::CryptoAuditCount, &next);

        let entry = CryptoAuditEntry {
            id: next,
            timestamp: env.ledger().timestamp(),
            actor: actor.clone(),
            action,
            record_id,
            details_hash,
            details_ref,
        };
        env.storage()
            .persistent()
            .set(&DataKey::CryptoAudit(next), &entry);
    }
}
