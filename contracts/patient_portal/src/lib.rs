//! On-chain patient portal coordinator for personal health records (PHR), scheduling,
//! and medication adherence. Integrates with [`medical_records`] and [`identity_registry`]
//! contract addresses configured at deploy time (responsive UI, WCAG 2.1, sub-second
//! reads, and tiered scaling for high concurrency are implemented in off-chain services
//! that call this contract).
//!
//! [`medical_records`]: ../../medical_records/src/lib.rs
//! [`identity_registry`]: ../../identity_registry/src/lib.rs

#![no_std]
#![allow(clippy::too_many_arguments)]

#[cfg(test)]
mod test;

use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype, log, Address, BytesN, Env, String, Vec,
};

// --- Limits (gas / storage bounds; supports large user bases via many ledger entries) ---

const MAX_INDEXED_ITEMS: u32 = 48;
const MAX_EXPORT_RECORD_IDS: u32 = 64;
const MAX_MEDICATION_REF_LEN: u32 = 128;
const MAX_NOTE_LEN: u32 = 256;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum PatientPortalError {
    AlreadyInitialized = 1,
    NotInitialized = 2,
    NotAdmin = 3,
    Paused = 4,
    AlreadyRegistered = 5,
    NotRegistered = 6,
    AppointmentNotFound = 7,
    ExportTooManyRecords = 8,
    InvalidInput = 9,
    NotAppointmentOwner = 10,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[contracttype]
pub enum AppointmentStatus {
    Requested,
    Confirmed,
    Completed,
    Cancelled,
}

#[derive(Clone)]
#[contracttype]
pub struct PortalProfile {
    pub patient: Address,
    pub registered_at: u64,
    /// Commitment linking to `identity_registry`; all-zero means not linked.
    pub identity_commitment: BytesN<32>,
    pub locale: String,
}

#[derive(Clone)]
#[contracttype]
pub struct PortalAppointment {
    pub id: u64,
    pub patient: Address,
    pub provider: Address,
    pub start_ts: u64,
    pub end_ts: u64,
    pub status: AppointmentStatus,
    /// Correlates to `telemedicine` `AppointmentSlot.appointment_id`; all-zero if none.
    pub telemedicine_appointment_id: BytesN<32>,
    pub notes: String,
}

#[derive(Clone)]
#[contracttype]
pub struct MedicationAdherenceEvent {
    pub id: u64,
    pub patient: Address,
    pub medication_ref: String,
    pub scheduled_for: u64,
    pub taken: bool,
    pub logged_at: u64,
}

#[derive(Clone)]
#[contracttype]
pub struct PhrExportManifest {
    pub id: u64,
    pub patient: Address,
    pub record_ids: Vec<u64>,
    pub requested_at: u64,
    /// Commitment to the off-chain export bundle (encrypted PHR package).
    pub manifest_hash: BytesN<32>,
}

#[contracttype]
pub enum DataKey {
    Initialized,
    Admin,
    Paused,
    MedicalRecords,
    IdentityRegistry,
    NextAppointmentId,
    Appointment(u64),
    NextAdherenceId,
    Adherence(u64),
    NextExportId,
    Export(u64),
    Profile(Address),
    PatientAppointmentIds(Address),
    PatientAdherenceIds(Address),
    PatientExportIds(Address),
}

#[contract]
pub struct PatientPortalContract;

#[contractimpl]
impl PatientPortalContract {
    pub fn initialize(env: Env, admin: Address) -> Result<(), PatientPortalError> {
        if env.storage().instance().has(&DataKey::Initialized) {
            return Err(PatientPortalError::AlreadyInitialized);
        }
        admin.require_auth();
        env.storage().instance().set(&DataKey::Initialized, &true);
        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage().instance().set(&DataKey::Paused, &false);
        log!(&env, "patient_portal: initialized");
        Ok(())
    }

    /// Wire `medical_records` and `identity_registry` for integrators (viewing uses those contracts).
    pub fn set_integration_contracts(
        env: Env,
        caller: Address,
        medical_records: Address,
        identity_registry: Address,
    ) -> Result<(), PatientPortalError> {
        Self::require_admin(&env, &caller)?;
        env.storage()
            .instance()
            .set(&DataKey::MedicalRecords, &medical_records);
        env.storage()
            .instance()
            .set(&DataKey::IdentityRegistry, &identity_registry);
        Ok(())
    }

    pub fn get_medical_records_contract(env: Env) -> Result<Option<Address>, PatientPortalError> {
        Self::require_init(&env)?;
        Ok(env.storage().instance().get(&DataKey::MedicalRecords))
    }

    pub fn get_identity_registry_contract(env: Env) -> Result<Option<Address>, PatientPortalError> {
        Self::require_init(&env)?;
        Ok(env.storage().instance().get(&DataKey::IdentityRegistry))
    }

    pub fn pause(env: Env, caller: Address) -> Result<(), PatientPortalError> {
        Self::require_admin(&env, &caller)?;
        env.storage().instance().set(&DataKey::Paused, &true);
        Ok(())
    }

    pub fn unpause(env: Env, caller: Address) -> Result<(), PatientPortalError> {
        Self::require_admin(&env, &caller)?;
        env.storage().instance().set(&DataKey::Paused, &false);
        Ok(())
    }

    /// Patient-signed registration for the portal (pairs with secure auth in the dApp).
    pub fn register(
        env: Env,
        patient: Address,
        identity_commitment: BytesN<32>,
        locale: String,
    ) -> Result<(), PatientPortalError> {
        patient.require_auth();
        Self::require_not_paused(&env)?;
        if env
            .storage()
            .persistent()
            .has(&DataKey::Profile(patient.clone()))
        {
            return Err(PatientPortalError::AlreadyRegistered);
        }
        Self::validate_locale(&locale)?;
        let profile = PortalProfile {
            patient: patient.clone(),
            registered_at: env.ledger().timestamp(),
            identity_commitment,
            locale,
        };
        env.storage()
            .persistent()
            .set(&DataKey::Profile(patient), &profile);
        Ok(())
    }

    pub fn get_profile(env: Env, patient: Address) -> Result<PortalProfile, PatientPortalError> {
        Self::require_init(&env)?;
        env.storage()
            .persistent()
            .get(&DataKey::Profile(patient))
            .ok_or(PatientPortalError::NotRegistered)
    }

    /// Audit trail for PHR download / export (actual ciphertext lives off-chain).
    pub fn request_phr_export(
        env: Env,
        patient: Address,
        record_ids: Vec<u64>,
        manifest_hash: BytesN<32>,
    ) -> Result<u64, PatientPortalError> {
        patient.require_auth();
        Self::require_not_paused(&env)?;
        Self::require_profile(&env, &patient)?;
        if record_ids.is_empty() || record_ids.len() > MAX_EXPORT_RECORD_IDS {
            return Err(PatientPortalError::ExportTooManyRecords);
        }
        let id = Self::bump_export_id(&env);
        let manifest = PhrExportManifest {
            id,
            patient: patient.clone(),
            record_ids,
            requested_at: env.ledger().timestamp(),
            manifest_hash,
        };
        env.storage()
            .persistent()
            .set(&DataKey::Export(id), &manifest);
        Self::push_index(&env, &DataKey::PatientExportIds(patient.clone()), id)?;
        Ok(id)
    }

    pub fn get_export(env: Env, id: u64) -> Result<PhrExportManifest, PatientPortalError> {
        Self::require_init(&env)?;
        env.storage()
            .persistent()
            .get(&DataKey::Export(id))
            .ok_or(PatientPortalError::InvalidInput)
    }

    /// Book or request an appointment; link to telemedicine appointment id when available.
    pub fn schedule_appointment(
        env: Env,
        patient: Address,
        provider: Address,
        start_ts: u64,
        end_ts: u64,
        telemedicine_appointment_id: BytesN<32>,
        notes: String,
    ) -> Result<u64, PatientPortalError> {
        patient.require_auth();
        Self::require_not_paused(&env)?;
        Self::require_profile(&env, &patient)?;
        if start_ts >= end_ts {
            return Err(PatientPortalError::InvalidInput);
        }
        Self::validate_notes(&notes)?;
        let id = Self::bump_appointment_id(&env);
        let appt = PortalAppointment {
            id,
            patient: patient.clone(),
            provider,
            start_ts,
            end_ts,
            status: AppointmentStatus::Requested,
            telemedicine_appointment_id,
            notes,
        };
        env.storage()
            .persistent()
            .set(&DataKey::Appointment(id), &appt);
        Self::push_index(&env, &DataKey::PatientAppointmentIds(patient.clone()), id)?;
        Ok(id)
    }

    pub fn set_appointment_status(
        env: Env,
        patient: Address,
        appointment_id: u64,
        status: AppointmentStatus,
    ) -> Result<(), PatientPortalError> {
        patient.require_auth();
        Self::require_not_paused(&env)?;
        let key = DataKey::Appointment(appointment_id);
        let mut appt: PortalAppointment = env
            .storage()
            .persistent()
            .get(&key)
            .ok_or(PatientPortalError::AppointmentNotFound)?;
        if appt.patient != patient {
            return Err(PatientPortalError::NotAppointmentOwner);
        }
        appt.status = status;
        env.storage().persistent().set(&key, &appt);
        Ok(())
    }

    pub fn get_appointment(
        env: Env,
        appointment_id: u64,
    ) -> Result<PortalAppointment, PatientPortalError> {
        Self::require_init(&env)?;
        env.storage()
            .persistent()
            .get(&DataKey::Appointment(appointment_id))
            .ok_or(PatientPortalError::AppointmentNotFound)
    }

    pub fn list_my_appointment_ids(
        env: Env,
        patient: Address,
    ) -> Result<Vec<u64>, PatientPortalError> {
        Self::require_init(&env)?;
        Ok(Self::read_index(
            &env,
            &DataKey::PatientAppointmentIds(patient),
        ))
    }

    /// Medication adherence tracking (references prescriptions / meds off-chain or in EMR).
    pub fn log_medication_event(
        env: Env,
        patient: Address,
        medication_ref: String,
        scheduled_for: u64,
        taken: bool,
    ) -> Result<u64, PatientPortalError> {
        patient.require_auth();
        Self::require_not_paused(&env)?;
        Self::require_profile(&env, &patient)?;
        Self::validate_med_ref(&medication_ref)?;
        let id = Self::bump_adherence_id(&env);
        let ev = MedicationAdherenceEvent {
            id,
            patient: patient.clone(),
            medication_ref,
            scheduled_for,
            taken,
            logged_at: env.ledger().timestamp(),
        };
        env.storage().persistent().set(&DataKey::Adherence(id), &ev);
        Self::push_index(&env, &DataKey::PatientAdherenceIds(patient), id)?;
        Ok(id)
    }

    pub fn get_adherence_event(
        env: Env,
        id: u64,
    ) -> Result<MedicationAdherenceEvent, PatientPortalError> {
        Self::require_init(&env)?;
        env.storage()
            .persistent()
            .get(&DataKey::Adherence(id))
            .ok_or(PatientPortalError::InvalidInput)
    }

    pub fn list_my_adherence_ids(
        env: Env,
        patient: Address,
    ) -> Result<Vec<u64>, PatientPortalError> {
        Self::require_init(&env)?;
        Ok(Self::read_index(
            &env,
            &DataKey::PatientAdherenceIds(patient),
        ))
    }
}

impl PatientPortalContract {
    fn require_init(env: &Env) -> Result<(), PatientPortalError> {
        if !env.storage().instance().has(&DataKey::Initialized) {
            return Err(PatientPortalError::NotInitialized);
        }
        Ok(())
    }

    fn require_not_paused(env: &Env) -> Result<(), PatientPortalError> {
        if env
            .storage()
            .instance()
            .get::<_, bool>(&DataKey::Paused)
            .unwrap_or(false)
        {
            return Err(PatientPortalError::Paused);
        }
        Ok(())
    }

    fn require_admin(env: &Env, caller: &Address) -> Result<(), PatientPortalError> {
        Self::require_init(env)?;
        let admin: Address = env
            .storage()
            .instance()
            .get(&DataKey::Admin)
            .ok_or(PatientPortalError::NotInitialized)?;
        if caller != &admin {
            return Err(PatientPortalError::NotAdmin);
        }
        caller.require_auth();
        Ok(())
    }

    fn require_profile(env: &Env, patient: &Address) -> Result<(), PatientPortalError> {
        if !env
            .storage()
            .persistent()
            .has(&DataKey::Profile(patient.clone()))
        {
            return Err(PatientPortalError::NotRegistered);
        }
        Ok(())
    }

    fn bump_appointment_id(env: &Env) -> u64 {
        let cur: u64 = env
            .storage()
            .instance()
            .get::<_, u64>(&DataKey::NextAppointmentId)
            .unwrap_or(0);
        let next = cur.saturating_add(1);
        env.storage()
            .instance()
            .set(&DataKey::NextAppointmentId, &next);
        next
    }

    fn bump_adherence_id(env: &Env) -> u64 {
        let cur: u64 = env
            .storage()
            .instance()
            .get::<_, u64>(&DataKey::NextAdherenceId)
            .unwrap_or(0);
        let next = cur.saturating_add(1);
        env.storage()
            .instance()
            .set(&DataKey::NextAdherenceId, &next);
        next
    }

    fn bump_export_id(env: &Env) -> u64 {
        let cur: u64 = env
            .storage()
            .instance()
            .get::<_, u64>(&DataKey::NextExportId)
            .unwrap_or(0);
        let next = cur.saturating_add(1);
        env.storage().instance().set(&DataKey::NextExportId, &next);
        next
    }

    fn read_index(env: &Env, key: &DataKey) -> Vec<u64> {
        env.storage()
            .persistent()
            .get(key)
            .unwrap_or_else(|| Vec::new(env))
    }

    fn push_index(env: &Env, key: &DataKey, id: u64) -> Result<(), PatientPortalError> {
        let mut v = Self::read_index(env, key);
        if v.len() >= MAX_INDEXED_ITEMS {
            let mut shifted = Vec::new(env);
            let skip = v.len().saturating_sub(MAX_INDEXED_ITEMS.saturating_sub(1));
            let mut i = skip;
            while i < v.len() {
                shifted.push_back(v.get(i).unwrap());
                i = i.saturating_add(1);
            }
            v = shifted;
        }
        v.push_back(id);
        env.storage().persistent().set(key, &v);
        Ok(())
    }

    fn validate_locale(s: &String) -> Result<(), PatientPortalError> {
        if s.is_empty() || s.len() > 16 {
            return Err(PatientPortalError::InvalidInput);
        }
        Ok(())
    }

    fn validate_notes(s: &String) -> Result<(), PatientPortalError> {
        if s.len() > MAX_NOTE_LEN {
            return Err(PatientPortalError::InvalidInput);
        }
        Ok(())
    }

    fn validate_med_ref(s: &String) -> Result<(), PatientPortalError> {
        if s.is_empty() || s.len() > MAX_MEDICATION_REF_LEN {
            return Err(PatientPortalError::InvalidInput);
        }
        Ok(())
    }
}
