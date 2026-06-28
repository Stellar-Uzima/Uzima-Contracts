#![no_std]
//! clinical_trial - Healthcare smart contract on Stellar blockchain.
#![allow(dead_code)]
#![allow(clippy::manual_range_contains)]
#![allow(clippy::absurd_extreme_comparisons)]

use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype, Address, Env, String, Symbol, Vec,
};

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    ProtocolNotFound = 1,
    TrialFull = 2,
    InvalidTitle = 3,
    InvalidMetadataRef = 4,
    InvalidName = 5,
    InvalidConsentRef = 6,
    InvalidMaxParticipants = 7,
    InvalidDescriptionRef = 8,
    InvalidSeverity = 9,
}

// ==================== VALIDATION CONSTANTS ====================

/// Minimum length for protocol title
const MIN_TITLE_LENGTH: u32 = 3;
/// Maximum length for protocol title
const MAX_TITLE_LENGTH: u32 = 256;

/// Minimum length for site name
const MIN_NAME_LENGTH: u32 = 2;
/// Maximum length for site name
const MAX_NAME_LENGTH: u32 = 128;

/// Minimum length for reference strings (IPFS CID or similar)
const MIN_REF_LENGTH: u32 = 10;
/// Maximum length for reference strings
const MAX_REF_LENGTH: u32 = 256;

/// Minimum severity level (0)
const MIN_SEVERITY: u32 = 0;
/// Maximum severity level (10)
const MAX_SEVERITY: u32 = 10;

// ==================== VALIDATION FUNCTIONS ====================

/// Validates that a title string has appropriate length
fn validate_title(title: &String) -> Result<(), Error> {
    let len = title.len();
    if len == 0 || len < MIN_TITLE_LENGTH || len > MAX_TITLE_LENGTH {
        return Err(Error::InvalidTitle);
    }
    Ok(())
}

/// Validates that a name string has appropriate length
fn validate_name(name: &String) -> Result<(), Error> {
    let len = name.len();
    if len == 0 || len < MIN_NAME_LENGTH || len > MAX_NAME_LENGTH {
        return Err(Error::InvalidName);
    }
    Ok(())
}

/// Validates that a reference string (IPFS CID, etc.) has appropriate length
fn validate_reference(reference: &String, error_type: Error) -> Result<(), Error> {
    let len = reference.len();
    if len == 0 || len < MIN_REF_LENGTH || len > MAX_REF_LENGTH {
        return Err(error_type);
    }
    Ok(())
}

/// Validates that max_participants is positive
fn validate_max_participants(max_participants: u64) -> Result<(), Error> {
    if max_participants == 0 {
        return Err(Error::InvalidMaxParticipants);
    }
    Ok(())
}

/// Validates severity level is within acceptable range
fn validate_severity(severity: u32) -> Result<(), Error> {
    if severity < MIN_SEVERITY || severity > MAX_SEVERITY {
        return Err(Error::InvalidSeverity);
    }
    Ok(())
}

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
    pub max_participants: u64,
    pub current_participants: u64,
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
    ProtocolEnrollmentCount(u64),
}

// ------------------ Contract ------------------

#[contract]
pub struct ClinicalTrial;

#[allow(clippy::too_many_arguments)] // Contract API functions require all parameters individually per Soroban ABI
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
            .publish((Symbol::new(&env, "initialized"),), (admin,));
    }

    // Create or version a trial protocol
    pub fn create_protocol(
        env: Env,
        proposer: Address,
        title: String,
        metadata_ref: String,
        max_participants: u64,
    ) -> Result<u64, Error> {
        proposer.require_auth();

        // Validate inputs
        validate_title(&title)?;
        validate_reference(&metadata_ref, Error::InvalidMetadataRef)?;
        validate_max_participants(max_participants)?;

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
            max_participants,
            current_participants: 0,
        };
        env.storage()
            .persistent()
            .set(&DataKey::Protocol(id), &proto);
        env.storage()
            .instance()
            .set(&DataKey::ProtocolNextId, &next.saturating_add(1));
        env.events()
            .publish((Symbol::new(&env, "protocol_created"),), (id, proposer));
        id
    }

    pub fn get_protocol(env: Env, id: u64) -> Option<Protocol> {
        env.storage().persistent().get(&DataKey::Protocol(id))
    }

    pub fn register_site(env: Env, registrar: Address, name: String) -> Result<u64, Error> {
        registrar.require_auth();

        // Validate inputs
        validate_name(&name)?;

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
            .publish((Symbol::new(&env, "site_registered"),), (id, registrar));
        id
    }

    // Patient recruitment / eligibility with enrollment cap enforcement
    pub fn recruit_patient(
        env: Env,
        site: Address,
        patient: Address,
        protocol_id: u64,
    ) -> Result<(), Error> {
        site.require_auth();

        // Check protocol exists and has capacity
        let mut protocol: Protocol = env
            .storage()
            .persistent()
            .get(&DataKey::Protocol(protocol_id))
            .ok_or(Error::ProtocolNotFound)?;

        if protocol.max_participants > 0
            && protocol.current_participants >= protocol.max_participants
        {
            return Err(Error::TrialFull);
        }

        // Store recruitment state
        let key = DataKey::ParticipantRecords(patient.clone());
        let mut v: Vec<u64> = env
            .storage()
            .persistent()
            .get(&key)
            .unwrap_or(Vec::new(&env));
        v.push_back(protocol_id);
        env.storage().persistent().set(&key, &v);

        // Update enrollment count
        protocol.current_participants = protocol.current_participants.saturating_add(1);
        env.storage()
            .persistent()
            .set(&DataKey::Protocol(protocol_id), &protocol);

        // Check if trial is now full and emit event
        if protocol.max_participants > 0
            && protocol.current_participants >= protocol.max_participants
        {
            env.events().publish(
                (Symbol::new(&env, "TrialCapacityReached"),),
                (protocol_id, protocol.max_participants),
            );
        }

        env.events().publish(
            (Symbol::new(&env, "patient_recruited"),),
            (patient, protocol_id, site),
        );

        Ok(())
    }

    pub fn record_consent(
        env: Env,
        patient: Address,
        protocol_id: u64,
        consent_ref: String,
    ) -> Result<u64, Error> {
        patient.require_auth();

        // Validate inputs
        validate_reference(&consent_ref, Error::InvalidConsentRef)?;

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
            (Symbol::new(&env, "consent_recorded"),),
            (id, patient, protocol_id),
        );
        Ok(id)
    }

    #[allow(clippy::too_many_arguments)] // All parameters are individually required by the Soroban contract ABI
    pub fn report_adverse_event(
        env: Env,
        reporter: Address,
        patient: Address,
        protocol_id: u64,
        site_id: u64,
        severity: u32,
        description_ref: String,
    ) -> Result<u64, Error> {
        reporter.require_auth();

        // Validate inputs
        validate_severity(severity)?;
        validate_reference(&description_ref, Error::InvalidDescriptionRef)?;

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
            (Symbol::new(&env, "adverse_event"),),
            (id, patient, protocol_id, site_id, severity),
        );
        Ok(id)
    }

    pub fn get_trial_status(env: Env, protocol_id: u64) -> Result<(u64, u64, u64), Error> {
        let proto: Protocol = env
            .storage()
            .persistent()
            .get(&DataKey::Protocol(protocol_id))
            .ok_or(Error::ProtocolNotFound)?;
        Ok((proto.id, proto.current_participants, proto.max_participants))
    }

    /// Enroll a participant in a clinical trial.
    ///
    /// Enforces the `max_participants` cap: if the trial is already at capacity
    /// this returns `Err(Error::TrialFull)`.  When the last available slot is
    /// filled a `TrialCapacityReached` event is emitted in addition to the
    /// standard `ParticipantEnrolled` event.
    pub fn enroll_participant(
        env: Env,
        site: Address,
        participant: Address,
        protocol_id: u64,
    ) -> Result<(), Error> {
        site.require_auth();

        // Load the protocol; fail if it doesn't exist.
        let mut protocol: Protocol = env
            .storage()
            .persistent()
            .get(&DataKey::Protocol(protocol_id))
            .ok_or(Error::ProtocolNotFound)?;

        // Enforce enrollment cap (max_participants == 0 means unlimited).
        if protocol.max_participants > 0
            && protocol.current_participants >= protocol.max_participants
        {
            return Err(Error::TrialFull);
        }

        // Record participant enrollment.
        let key = DataKey::ParticipantRecords(participant.clone());
        let mut enrolled: Vec<u64> = env
            .storage()
            .persistent()
            .get(&key)
            .unwrap_or(Vec::new(&env));
        enrolled.push_back(protocol_id);
        env.storage().persistent().set(&key, &enrolled);

        // Increment the protocol's enrollment counter.
        protocol.current_participants = protocol.current_participants.saturating_add(1);
        env.storage()
            .persistent()
            .set(&DataKey::Protocol(protocol_id), &protocol);

        // Emit TrialCapacityReached when the last slot is now filled.
        if protocol.max_participants > 0
            && protocol.current_participants >= protocol.max_participants
        {
            env.events().publish(
                (Symbol::new(&env, "TrialCapacityReached"),),
                (protocol_id, protocol.max_participants),
            );
        }

        env.events().publish(
            (Symbol::new(&env, "ParticipantEnrolled"),),
            (participant, protocol_id, site),
        );

        Ok(())
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

// ==================== Tests ====================

#[cfg(test)]
mod tests {
    use super::*;
    use soroban_sdk::testutils::Address as _;
    use soroban_sdk::{Env, String};

    fn setup(env: &Env) -> (ClinicalTrialClient<'_>, Address, Address) {
        let admin = Address::generate(env);
        let contract_id = env.register_contract(None, ClinicalTrial);
        let client = ClinicalTrialClient::new(env, &contract_id);
        client.initialize(&admin);
        (client, contract_id, admin)
    }

    #[test]
    fn test_enroll_up_to_max_succeeds() {
        let env = Env::default();
        env.mock_all_auths();
        let (client, _contract_id, admin) = setup(&env);

        let protocol_id = client.create_protocol(
            &admin,
            &String::from_str(&env, "Phase I Trial"),
            &String::from_str(&env, "QmMetadataRef1234567890ABCDEF"),
            &3u64, // max 3 participants
        );

        let site = Address::generate(&env);
        let p1 = Address::generate(&env);
        let p2 = Address::generate(&env);
        let p3 = Address::generate(&env);

        // Enrolling up to capacity should all succeed
        client.enroll_participant(&site, &p1, &protocol_id);
        client.enroll_participant(&site, &p2, &protocol_id);
        client.enroll_participant(&site, &p3, &protocol_id);

        let (_, enrolled, max) = client.get_trial_status(&protocol_id);
        assert_eq!(enrolled, 3);
        assert_eq!(max, 3);
    }

    #[test]
    fn test_enroll_beyond_max_fails() {
        let env = Env::default();
        env.mock_all_auths();
        let (client, _contract_id, admin) = setup(&env);

        let protocol_id = client.create_protocol(
            &admin,
            &String::from_str(&env, "Phase II Trial"),
            &String::from_str(&env, "QmMetadataRef1234567890ABCDEF"),
            &2u64, // max 2 participants
        );

        let site = Address::generate(&env);
        let p1 = Address::generate(&env);
        let p2 = Address::generate(&env);
        let p3 = Address::generate(&env); // would exceed cap

        client.enroll_participant(&site, &p1, &protocol_id);
        client.enroll_participant(&site, &p2, &protocol_id);

        // Third enrollment must return TrialFull
        let result = client.try_enroll_participant(&site, &p3, &protocol_id);
        assert_eq!(result, Err(Ok(Error::TrialFull)));

        // Enrollment count must remain at 2
        let (_, enrolled, _) = client.get_trial_status(&protocol_id);
        assert_eq!(enrolled, 2);
    }
}
