#![no_std]
#![allow(dead_code)]

use soroban_sdk::{contract, contractimpl, contracttype, Address, Env, String, Symbol, Vec};

// ------------------ Types ------------------

#[contracttype]
#[derive(Clone)]
pub struct Protocol {
    pub id: u64,
    pub title: String,
    pub version: u32,
    pub sponsor: Address,
    pub created_at: u64,
    pub active: bool,
    pub metadata_ref: String,
}

#[contracttype]
#[derive(Clone)]
pub struct Site {
    pub id: u64,
    pub address: Address,
    pub name: String,
    pub active: bool,
}

#[contracttype]
#[derive(Clone)]
pub struct Consent {
    pub patient: Address,
    pub protocol_id: u64,
    pub version: u32,
    pub timestamp: u64,
    pub consent_ref: String,
}

#[contracttype]
#[derive(Clone)]
pub struct AdverseEvent {
    pub id: u64,
    pub patient: Address,
    pub protocol_id: u64,
    pub site_id: u64,
    pub description_ref: String,
    pub timestamp: u64,
    pub severity: u32,
}

// ------------------ Storage Keys ------------------

#[contracttype]
pub enum DataKey {
    Initialized,
    Protocol(u64),
    ProtocolNextId,
    Site(u64),
    SiteNextId,
    ConsentCount,
    Consent(u64),
    AdverseEventNextId,
    AdverseEvent(u64),
    ParticipantRecords(Address),
}

// ------------------ Contract ------------------

#[contract]
pub struct ClinicalTrial;

#[contractimpl]
impl ClinicalTrial {
    pub fn initialize(env: Env, admin: Address) {
        admin.require_auth();
        if env.storage().instance().has(&DataKey::Initialized) {
            return;
        }
        env.storage().instance().set(&DataKey::Initialized, &true);
        env.storage()
            .instance()
            .set(&DataKey::ProtocolNextId, &1u64);
        env.storage().instance().set(&DataKey::SiteNextId, &1u64);
        env.storage()
            .instance()
            .set(&DataKey::AdverseEventNextId, &1u64);
        env.events()
            .publish((Symbol::new(&env, "Initialized"),), (admin,));
    }

    // Create or version a trial protocol
    pub fn create_protocol(
        env: Env,
        proposer: Address,
        title: String,
        metadata_ref: String,
    ) -> u64 {
        proposer.require_auth();
        let next: u64 = env
            .storage()
            .instance()
            .get(&DataKey::ProtocolNextId)
            .unwrap_or(1u64);
        let id = next;
        let proto = Protocol {
            id,
            title: title.clone(),
            version: 1u32,
            sponsor: proposer.clone(),
            created_at: env.ledger().timestamp(),
            active: true,
            metadata_ref,
        };
        env.storage()
            .persistent()
            .set(&DataKey::Protocol(id), &proto);
        env.storage()
            .instance()
            .set(&DataKey::ProtocolNextId, &next.saturating_add(1));
        env.events()
            .publish((Symbol::new(&env, "ProtocolCreated"),), (id, proposer));
        id
    }

    pub fn get_protocol(env: Env, id: u64) -> Option<Protocol> {
        env.storage().persistent().get(&DataKey::Protocol(id))
    }

    pub fn register_site(env: Env, registrar: Address, name: String) -> u64 {
        registrar.require_auth();
        let next: u64 = env
            .storage()
            .instance()
            .get(&DataKey::SiteNextId)
            .unwrap_or(1u64);
        let id = next;
        let s = Site {
            id,
            address: registrar.clone(),
            name: name.clone(),
            active: true,
        };
        env.storage().persistent().set(&DataKey::Site(id), &s);
        env.storage()
            .instance()
            .set(&DataKey::SiteNextId, &next.saturating_add(1));
        env.events()
            .publish((Symbol::new(&env, "SiteRegistered"),), (id, registrar));
        id
    }

    // Patient recruitment / eligibility (simple verifier placeholder)
    pub fn recruit_patient(env: Env, site: Address, patient: Address, protocol_id: u64) {
        site.require_auth();
        // a real implementation would run eligibility checks and store recruitment state
        let key = DataKey::ParticipantRecords(patient.clone());
        let mut v: Vec<u64> = env
            .storage()
            .persistent()
            .get(&key)
            .unwrap_or(Vec::new(&env));
        v.push_back(protocol_id);
        env.storage().persistent().set(&key, &v);
        env.events().publish(
            (Symbol::new(&env, "PatientRecruited"),),
            (patient, protocol_id, site),
        );
    }

    pub fn record_consent(
        env: Env,
        patient: Address,
        protocol_id: u64,
        consent_ref: String,
    ) -> u64 {
        patient.require_auth();
        let count: u64 = env
            .storage()
            .instance()
            .get(&DataKey::ConsentCount)
            .unwrap_or(0u64);
        let id = count.saturating_add(1);
        let c = Consent {
            patient: patient.clone(),
            protocol_id,
            version: 1u32,
            timestamp: env.ledger().timestamp(),
            consent_ref,
        };
        env.storage().persistent().set(&DataKey::Consent(id), &c);
        env.storage().instance().set(&DataKey::ConsentCount, &id);
        env.events().publish(
            (Symbol::new(&env, "ConsentRecorded"),),
            (id, patient, protocol_id),
        );
        id
    }

    pub fn report_adverse_event(
        env: Env,
        reporter: Address,
        patient: Address,
        protocol_id: u64,
        site_id: u64,
        severity: u32,
        description_ref: String,
    ) -> u64 {
        reporter.require_auth();
        let next: u64 = env
            .storage()
            .instance()
            .get(&DataKey::AdverseEventNextId)
            .unwrap_or(1u64);
        let id = next;
        let ev = AdverseEvent {
            id,
            patient: patient.clone(),
            protocol_id,
            site_id,
            description_ref,
            timestamp: env.ledger().timestamp(),
            severity,
        };
        env.storage()
            .persistent()
            .set(&DataKey::AdverseEvent(id), &ev);
        env.storage()
            .instance()
            .set(&DataKey::AdverseEventNextId, &next.saturating_add(1));
        env.events().publish(
            (Symbol::new(&env, "AdverseEvent"),),
            (id, patient, protocol_id, site_id, severity),
        );
        id
    }

    // Simple audit: return whether a consent exists for a patient/protocol
    pub fn has_consent(env: Env, patient: Address, protocol_id: u64) -> bool {
        let mut i: u64 = 1;
        let count: u64 = env
            .storage()
            .instance()
            .get(&DataKey::ConsentCount)
            .unwrap_or(0u64);
        while i <= count {
            if let Some(c) = env
                .storage()
                .persistent()
                .get::<DataKey, Consent>(&DataKey::Consent(i))
            {
                if c.patient == patient && c.protocol_id == protocol_id {
                    return true;
                }
            }
            i = i.saturating_add(1);
        }
        false
    }
}
