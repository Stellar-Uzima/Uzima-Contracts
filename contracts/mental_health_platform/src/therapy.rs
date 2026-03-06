use soroban_sdk::{contracttype, Address, BytesN, Env, String, Vec};
use crate::{types::*, errors::Error, events::*};

pub struct TherapyManager;

impl TherapyManager {
    pub fn create_session(
        env: &Env,
        patient_id: Address,
        therapist_id: Address,
        session_type: SessionType,
        duration_minutes: u32,
        confidentiality_level: ConfidentialityLevel,
    ) -> Result<u64, Error> {
        // Generate session ID
        let session_id = env.ledger().timestamp() as u64;

        let session = TherapySession {
            session_id,
            patient_id: patient_id.clone(),
            therapist_id: therapist_id.clone(),
            session_type,
            timestamp: env.ledger().timestamp(),
            duration_minutes,
            notes: String::from_str(env, ""),
            recording_hash: None,
            ai_insights: None,
            follow_up_required: false,
            confidentiality_level,
        };

        // Store session
        let mut sessions: Vec<TherapySession> = env.storage().instance()
            .get(&patient_id)
            .unwrap_or(Vec::new(env));
        sessions.push_back(session);
        env.storage().instance().set(&patient_id, &sessions);

        // Emit event
        env.events().publish(
            (Symbol::new(env, "therapy_session_created"),),
            TherapySessionEvent {
                session_id,
                patient_id,
                therapist_id,
                session_type: String::from_str(env, "session"), // Simplified
                timestamp: env.ledger().timestamp(),
            },
        );

        Ok(session_id)
    }

    pub fn record_session_notes(
        env: &Env,
        session_id: u64,
        patient_id: Address,
        notes: String,
        ai_insights: Option<String>,
    ) -> Result<(), Error> {
        let mut sessions: Vec<TherapySession> = env.storage().instance()
            .get(&patient_id)
            .ok_or(Error::SessionNotFound)?;

        for i in 0..sessions.len() {
            let mut session = sessions.get(i).unwrap();
            if session.session_id == session_id {
                session.notes = notes;
                session.ai_insights = ai_insights;
                sessions.set(i, session);
                env.storage().instance().set(&patient_id, &sessions);
                return Ok(());
            }
        }

        Err(Error::SessionNotFound)
    }

    pub fn add_session_recording(
        env: &Env,
        session_id: u64,
        patient_id: Address,
        recording_hash: BytesN<32>,
    ) -> Result<(), Error> {
        let mut sessions: Vec<TherapySession> = env.storage().instance()
            .get(&patient_id)
            .ok_or(Error::SessionNotFound)?;

        for i in 0..sessions.len() {
            let mut session = sessions.get(i).unwrap();
            if session.session_id == session_id {
                session.recording_hash = Some(recording_hash);
                sessions.set(i, session);
                env.storage().instance().set(&patient_id, &sessions);
                return Ok(());
            }
        }

        Err(Error::SessionNotFound)
    }

    pub fn get_patient_sessions(
        env: &Env,
        patient_id: Address,
    ) -> Vec<TherapySession> {
        env.storage().instance()
            .get(&patient_id)
            .unwrap_or(Vec::new(env))
    }

    pub fn analyze_session_patterns(
        env: &Env,
        patient_id: Address,
    ) -> String {
        let sessions = Self::get_patient_sessions(env, patient_id);

        if sessions.is_empty() {
            return String::from_str(env, "No sessions found for analysis");
        }

        // Simple pattern analysis (would be enhanced with AI in production)
        let total_sessions = sessions.len();
        let mut crisis_sessions = 0;
        let mut follow_ups = 0;

        for session in sessions.iter() {
            if session.session_type == SessionType::Crisis {
                crisis_sessions += 1;
            }
            if session.follow_up_required {
                follow_ups += 1;
            }
        }

        let analysis = String::from_str(env, "Session pattern analysis completed");
        analysis
    }
}