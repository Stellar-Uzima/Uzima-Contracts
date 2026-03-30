//! Mental health support on-chain module: mood and symptom commitments, teletherapy
//! session references (see `telemedicine`), crisis events and queueing for integrators to
//! pair with `notification_system`, peer-support memberships, and HIPAA-oriented minimal payloads
//! (hashes and scores; clinical narrative stays off-chain). Emergency routing and 24/7
//! operations are implemented by integrators reacting to notifications and events.

#![no_std]
#![allow(clippy::too_many_arguments)]

#[cfg(test)]
mod test;

use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype, log, Address, BytesN, Env, String, Vec,
};

const MAX_COMMUNITY_NAME_LEN: u32 = 64;
const MAX_MODALITY_LEN: u32 = 64;
const MAX_CRISIS_QUEUE: u32 = 100;
const MAX_INDEXED: u32 = 64;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum MentalHealthError {
    AlreadyInitialized = 1,
    NotInitialized = 2,
    NotAdmin = 3,
    Paused = 4,
    NotEnrolled = 5,
    CommunityNotFound = 6,
    AlreadyMember = 7,
    InvalidInput = 8,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[contracttype]
pub enum CrisisSeverity {
    Elevated,
    High,
    Imminent,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[contracttype]
pub enum TherapyModality {
    Cbt,
    Dbt,
    Psychodynamic,
    Group,
    Family,
    MedicationManagement,
    Other,
}

#[derive(Clone)]
#[contracttype]
pub struct MoodEntry {
    pub id: u64,
    pub patient: Address,
    pub recorded_at: u64,
    pub mood_score: u32,
    /// Hash of encrypted symptom / PHQ-style payload (HIPAA minimization on-chain).
    pub symptom_blob_hash: BytesN<32>,
}

#[derive(Clone)]
#[contracttype]
pub struct TeletherapyBooking {
    pub id: u64,
    pub patient: Address,
    pub modality: TherapyModality,
    pub telemedicine_session_id: BytesN<32>,
    pub scheduled_at: u64,
    pub notes: String,
}

#[derive(Clone)]
#[contracttype]
pub struct CrisisIntervention {
    pub id: u64,
    pub patient: Address,
    pub severity: CrisisSeverity,
    pub detail_hash: BytesN<32>,
    pub created_at: u64,
    pub notification_id: Option<u64>,
}

#[derive(Clone)]
#[contracttype]
pub struct PeerCommunity {
    pub id: u64,
    pub name: String,
    pub created_at: u64,
}

#[contracttype]
pub enum DataKey {
    Initialized,
    Admin,
    Paused,
    Telemedicine,
    Notification,
    EmergencyMetaHash,
    NextMoodId,
    Mood(u64),
    NextBookingId,
    Booking(u64),
    NextCrisisId,
    Crisis(u64),
    NextCommunityId,
    Community(u64),
    CommunityMembers(u64),
    Enrolled(Address),
    PatientMoodIds(Address),
    PatientBookingIds(Address),
    PatientCrisisIds(Address),
    PatientCommunities(Address),
    OpenCrisisQueue,
}

#[contract]
pub struct MentalHealthSupportContract;

#[contractimpl]
impl MentalHealthSupportContract {
    pub fn initialize(env: Env, admin: Address) -> Result<(), MentalHealthError> {
        if env.storage().instance().has(&DataKey::Initialized) {
            return Err(MentalHealthError::AlreadyInitialized);
        }
        admin.require_auth();
        env.storage().instance().set(&DataKey::Initialized, &true);
        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage().instance().set(&DataKey::Paused, &false);
        env.storage()
            .persistent()
            .set(&DataKey::OpenCrisisQueue, &Vec::<u64>::new(&env));
        log!(&env, "mental_health_support: initialized");
        Ok(())
    }

    pub fn set_integration_contracts(
        env: Env,
        caller: Address,
        telemedicine: Address,
        notification: Address,
    ) -> Result<(), MentalHealthError> {
        Self::require_admin(&env, &caller)?;
        env.storage()
            .instance()
            .set(&DataKey::Telemedicine, &telemedicine);
        env.storage()
            .instance()
            .set(&DataKey::Notification, &notification);
        Ok(())
    }

    pub fn set_emergency_routing_commitment(
        env: Env,
        caller: Address,
        meta_hash: BytesN<32>,
    ) -> Result<(), MentalHealthError> {
        Self::require_admin(&env, &caller)?;
        env.storage()
            .instance()
            .set(&DataKey::EmergencyMetaHash, &meta_hash);
        Ok(())
    }

    pub fn get_telemedicine_contract(env: Env) -> Result<Option<Address>, MentalHealthError> {
        Self::require_init(&env)?;
        Ok(env.storage().instance().get(&DataKey::Telemedicine))
    }

    pub fn get_notification_contract(env: Env) -> Result<Option<Address>, MentalHealthError> {
        Self::require_init(&env)?;
        Ok(env.storage().instance().get(&DataKey::Notification))
    }

    pub fn pause(env: Env, caller: Address) -> Result<(), MentalHealthError> {
        Self::require_admin(&env, &caller)?;
        env.storage().instance().set(&DataKey::Paused, &true);
        Ok(())
    }

    pub fn unpause(env: Env, caller: Address) -> Result<(), MentalHealthError> {
        Self::require_admin(&env, &caller)?;
        env.storage().instance().set(&DataKey::Paused, &false);
        Ok(())
    }

    pub fn enroll(env: Env, patient: Address) -> Result<(), MentalHealthError> {
        patient.require_auth();
        Self::require_not_paused(&env)?;
        if env
            .storage()
            .persistent()
            .has(&DataKey::Enrolled(patient.clone()))
        {
            return Ok(());
        }
        env.storage()
            .persistent()
            .set(&DataKey::Enrolled(patient), &true);
        Ok(())
    }

    pub fn is_enrolled(env: Env, patient: Address) -> Result<bool, MentalHealthError> {
        Self::require_init(&env)?;
        Ok(env
            .storage()
            .persistent()
            .get::<_, bool>(&DataKey::Enrolled(patient))
            .unwrap_or(false))
    }

    pub fn log_mood(
        env: Env,
        patient: Address,
        mood_score: u32,
        symptom_blob_hash: BytesN<32>,
    ) -> Result<u64, MentalHealthError> {
        patient.require_auth();
        Self::require_not_paused(&env)?;
        Self::require_enrolled(&env, &patient)?;
        if mood_score > 10 {
            return Err(MentalHealthError::InvalidInput);
        }
        let id = Self::bump_mood_id(&env);
        let entry = MoodEntry {
            id,
            patient: patient.clone(),
            recorded_at: env.ledger().timestamp(),
            mood_score,
            symptom_blob_hash,
        };
        env.storage().persistent().set(&DataKey::Mood(id), &entry);
        Self::push_bounded_id(&env, &DataKey::PatientMoodIds(patient), id, MAX_INDEXED)?;
        Ok(id)
    }

    pub fn get_mood(env: Env, id: u64) -> Result<MoodEntry, MentalHealthError> {
        Self::require_init(&env)?;
        env.storage()
            .persistent()
            .get(&DataKey::Mood(id))
            .ok_or(MentalHealthError::InvalidInput)
    }

    pub fn book_teletherapy(
        env: Env,
        patient: Address,
        modality: TherapyModality,
        telemedicine_session_id: BytesN<32>,
        scheduled_at: u64,
        notes: String,
    ) -> Result<u64, MentalHealthError> {
        patient.require_auth();
        Self::require_not_paused(&env)?;
        Self::require_enrolled(&env, &patient)?;
        if notes.len() > MAX_MODALITY_LEN {
            return Err(MentalHealthError::InvalidInput);
        }
        let id = Self::bump_booking_id(&env);
        let b = TeletherapyBooking {
            id,
            patient: patient.clone(),
            modality,
            telemedicine_session_id,
            scheduled_at,
            notes,
        };
        env.storage().persistent().set(&DataKey::Booking(id), &b);
        Self::push_bounded_id(&env, &DataKey::PatientBookingIds(patient), id, MAX_INDEXED)?;
        Ok(id)
    }

    pub fn get_booking(env: Env, id: u64) -> Result<TeletherapyBooking, MentalHealthError> {
        Self::require_init(&env)?;
        env.storage()
            .persistent()
            .get(&DataKey::Booking(id))
            .ok_or(MentalHealthError::InvalidInput)
    }

    /// Records crisis state and enqueues for triage. Use [`Self::get_notification_contract`] from a
    /// relayer to call `notification_system` (with this contract as authorised sender).
    pub fn report_crisis(
        env: Env,
        patient: Address,
        severity: CrisisSeverity,
        detail_hash: BytesN<32>,
    ) -> Result<u64, MentalHealthError> {
        patient.require_auth();
        Self::require_not_paused(&env)?;
        Self::require_enrolled(&env, &patient)?;
        let id = Self::bump_crisis_id(&env);

        let crisis = CrisisIntervention {
            id,
            patient: patient.clone(),
            severity,
            detail_hash,
            created_at: env.ledger().timestamp(),
            notification_id: None,
        };
        env.storage()
            .persistent()
            .set(&DataKey::Crisis(id), &crisis);
        Self::push_bounded_id(&env, &DataKey::PatientCrisisIds(patient), id, MAX_INDEXED)?;
        Self::enqueue_crisis(&env, id)?;
        log!(&env, "mental_health_support: crisis recorded");
        Ok(id)
    }

    pub fn get_crisis(env: Env, id: u64) -> Result<CrisisIntervention, MentalHealthError> {
        Self::require_init(&env)?;
        env.storage()
            .persistent()
            .get(&DataKey::Crisis(id))
            .ok_or(MentalHealthError::InvalidInput)
    }

    pub fn create_peer_community(
        env: Env,
        admin: Address,
        name: String,
    ) -> Result<u64, MentalHealthError> {
        Self::require_admin(&env, &admin)?;
        if name.is_empty() || name.len() > MAX_COMMUNITY_NAME_LEN {
            return Err(MentalHealthError::InvalidInput);
        }
        let id = Self::bump_community_id(&env);
        let c = PeerCommunity {
            id,
            name,
            created_at: env.ledger().timestamp(),
        };
        env.storage().persistent().set(&DataKey::Community(id), &c);
        env.storage()
            .persistent()
            .set(&DataKey::CommunityMembers(id), &Vec::<Address>::new(&env));
        Ok(id)
    }

    pub fn join_peer_community(
        env: Env,
        patient: Address,
        community_id: u64,
    ) -> Result<(), MentalHealthError> {
        patient.require_auth();
        Self::require_not_paused(&env)?;
        Self::require_enrolled(&env, &patient)?;
        if !env
            .storage()
            .persistent()
            .has(&DataKey::Community(community_id))
        {
            return Err(MentalHealthError::CommunityNotFound);
        }
        let key = DataKey::CommunityMembers(community_id);
        let mut members: Vec<Address> = env
            .storage()
            .persistent()
            .get(&key)
            .unwrap_or_else(|| Vec::new(&env));
        if members.contains(&patient) {
            return Err(MentalHealthError::AlreadyMember);
        }
        members.push_back(patient.clone());
        env.storage().persistent().set(&key, &members);

        let pc_key = DataKey::PatientCommunities(patient.clone());
        let pc = Self::read_u64_vec(&env, &pc_key);
        let updated = Self::push_u64_bounded(&env, pc, community_id, MAX_INDEXED)?;
        env.storage()
            .persistent()
            .set(&DataKey::PatientCommunities(patient), &updated);
        Ok(())
    }

    pub fn list_community_members(
        env: Env,
        community_id: u64,
    ) -> Result<Vec<Address>, MentalHealthError> {
        Self::require_init(&env)?;
        env.storage()
            .persistent()
            .get(&DataKey::CommunityMembers(community_id))
            .ok_or(MentalHealthError::CommunityNotFound)
    }

    pub fn open_crisis_queue(env: Env) -> Result<Vec<u64>, MentalHealthError> {
        Self::require_init(&env)?;
        Ok(env
            .storage()
            .persistent()
            .get::<_, Vec<u64>>(&DataKey::OpenCrisisQueue)
            .unwrap_or_else(|| Vec::new(&env)))
    }
}

impl MentalHealthSupportContract {
    fn require_init(env: &Env) -> Result<(), MentalHealthError> {
        if !env.storage().instance().has(&DataKey::Initialized) {
            return Err(MentalHealthError::NotInitialized);
        }
        Ok(())
    }

    fn require_not_paused(env: &Env) -> Result<(), MentalHealthError> {
        if env
            .storage()
            .instance()
            .get::<_, bool>(&DataKey::Paused)
            .unwrap_or(false)
        {
            return Err(MentalHealthError::Paused);
        }
        Ok(())
    }

    fn require_admin(env: &Env, caller: &Address) -> Result<(), MentalHealthError> {
        Self::require_init(env)?;
        let admin: Address = env
            .storage()
            .instance()
            .get(&DataKey::Admin)
            .ok_or(MentalHealthError::NotInitialized)?;
        if caller != &admin {
            return Err(MentalHealthError::NotAdmin);
        }
        caller.require_auth();
        Ok(())
    }

    fn require_enrolled(env: &Env, patient: &Address) -> Result<(), MentalHealthError> {
        if !env
            .storage()
            .persistent()
            .get::<_, bool>(&DataKey::Enrolled(patient.clone()))
            .unwrap_or(false)
        {
            return Err(MentalHealthError::NotEnrolled);
        }
        Ok(())
    }

    fn bump_mood_id(env: &Env) -> u64 {
        let cur = env
            .storage()
            .instance()
            .get::<_, u64>(&DataKey::NextMoodId)
            .unwrap_or(0);
        let next = cur.saturating_add(1);
        env.storage().instance().set(&DataKey::NextMoodId, &next);
        next
    }

    fn bump_booking_id(env: &Env) -> u64 {
        let cur = env
            .storage()
            .instance()
            .get::<_, u64>(&DataKey::NextBookingId)
            .unwrap_or(0);
        let next = cur.saturating_add(1);
        env.storage().instance().set(&DataKey::NextBookingId, &next);
        next
    }

    fn bump_crisis_id(env: &Env) -> u64 {
        let cur = env
            .storage()
            .instance()
            .get::<_, u64>(&DataKey::NextCrisisId)
            .unwrap_or(0);
        let next = cur.saturating_add(1);
        env.storage().instance().set(&DataKey::NextCrisisId, &next);
        next
    }

    fn bump_community_id(env: &Env) -> u64 {
        let cur = env
            .storage()
            .instance()
            .get::<_, u64>(&DataKey::NextCommunityId)
            .unwrap_or(0);
        let next = cur.saturating_add(1);
        env.storage()
            .instance()
            .set(&DataKey::NextCommunityId, &next);
        next
    }

    fn read_u64_vec(env: &Env, key: &DataKey) -> Vec<u64> {
        env.storage()
            .persistent()
            .get(key)
            .unwrap_or_else(|| Vec::new(env))
    }

    fn push_u64_bounded(
        env: &Env,
        mut v: Vec<u64>,
        id: u64,
        max: u32,
    ) -> Result<Vec<u64>, MentalHealthError> {
        if v.len() >= max {
            let mut shifted = Vec::new(env);
            let skip = v.len().saturating_sub(max.saturating_sub(1));
            let mut i = skip;
            while i < v.len() {
                shifted.push_back(v.get(i).unwrap());
                i = i.saturating_add(1);
            }
            v = shifted;
        }
        v.push_back(id);
        Ok(v)
    }

    fn push_bounded_id(
        env: &Env,
        key: &DataKey,
        id: u64,
        max: u32,
    ) -> Result<(), MentalHealthError> {
        let v = Self::read_u64_vec(env, key);
        let updated = Self::push_u64_bounded(env, v, id, max)?;
        env.storage().persistent().set(key, &updated);
        Ok(())
    }

    fn enqueue_crisis(env: &Env, crisis_id: u64) -> Result<(), MentalHealthError> {
        let mut q: Vec<u64> = env
            .storage()
            .persistent()
            .get(&DataKey::OpenCrisisQueue)
            .unwrap_or_else(|| Vec::new(env));
        if q.len() >= MAX_CRISIS_QUEUE {
            let mut shifted = Vec::new(env);
            let skip = q.len().saturating_sub(MAX_CRISIS_QUEUE.saturating_sub(1));
            let mut i = skip;
            while i < q.len() {
                shifted.push_back(q.get(i).unwrap());
                i = i.saturating_add(1);
            }
            q = shifted;
        }
        q.push_back(crisis_id);
        env.storage()
            .persistent()
            .set(&DataKey::OpenCrisisQueue, &q);
        Ok(())
    }
}
