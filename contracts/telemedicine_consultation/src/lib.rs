#![no_std]

#[cfg(test)]
mod test;

use soroban_sdk::symbol_short;
use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype, vec, Address, BytesN, Env, Map, String,
    Symbol, Vec,
};

// ==================== Telemedicine Consultation Types ====================

/// Consultation Status
#[derive(Clone, Copy, PartialEq, Eq)]
#[contracttype]
pub enum ConsultationStatus {
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
    NoShow,
    Emergency,
}

/// Consultation Type
#[derive(Clone, Copy, PartialEq, Eq)]
#[contracttype]
pub enum ConsultationType {
    Routine,
    FollowUp,
    Emergency,
    SecondOpinion,
    MentalHealth,
    ChronicCare,
    PreOp,
    PostOp,
}

/// Recording Quality
#[derive(Clone, Copy, PartialEq, Eq)]
#[contracttype]
pub enum RecordingQuality {
    Standard,
    High,
    UltraHD,
}

/// Encryption Standard
#[derive(Clone, Copy, PartialEq, Eq)]
#[contracttype]
pub enum EncryptionStandard {
    AES256,
    ChaCha20,
    FIPS140_2,
}

/// Consultation Session
#[derive(Clone)]
#[contracttype]
pub struct ConsultationSession {
    pub session_id: u64,
    pub patient: Address,
    pub provider: Address,
    pub consultation_type: ConsultationType,
    pub scheduled_time: u64,
    pub start_time: u64,
    pub end_time: u64,
    pub duration_minutes: u32,
    pub status: ConsultationStatus,
    pub emergency_level: u32, // 0-5 scale
    pub notes: String,
    pub metadata_uri: String,
}

/// Video Recording Metadata
#[derive(Clone)]
#[contracttype]
pub struct VideoRecording {
    pub recording_id: u64,
    pub session_id: u64,
    pub recording_uri: String, // IPFS hash or secure storage pointer
    pub file_size_bytes: u64,
    pub duration_seconds: u32,
    pub quality: RecordingQuality,
    pub encryption_standard: EncryptionStandard,
    pub encryption_key_hash: BytesN<32>,
    pub recorded_at: u64,
    pub is_archived: bool,
    pub retention_days: u32,
    pub consent_token_id: u64, // Links to consent NFT
}

/// Audio Recording Metadata
#[derive(Clone)]
#[contracttype]
pub struct AudioRecording {
    pub recording_id: u64,
    pub session_id: u64,
    pub recording_uri: String,
    pub file_size_bytes: u64,
    pub duration_seconds: u32,
    pub sample_rate: u32,
    pub channels: u32,
    pub encryption_standard: EncryptionStandard,
    pub encryption_key_hash: BytesN<32>,
    pub recorded_at: u64,
    pub is_archived: bool,
    pub retention_days: u32,
    pub consent_token_id: u64,
}

/// Screen Share Recording
#[derive(Clone)]
#[contracttype]
pub struct ScreenShareRecording {
    pub recording_id: u64,
    pub session_id: u64,
    pub recording_uri: String,
    pub file_size_bytes: u64,
    pub duration_seconds: u32,
    pub resolution: String,
    pub frame_rate: u32,
    pub encryption_standard: EncryptionStandard,
    pub encryption_key_hash: BytesN<32>,
    pub recorded_at: u64,
    pub is_archived: bool,
    pub retention_days: u32,
    pub consent_token_id: u64,
}

/// Consultation Summary
#[derive(Clone)]
#[contracttype]
pub struct ConsultationSummary {
    pub session_id: u64,
    pub chief_complaint: String,
    pub diagnosis_codes: Vec<String>, // ICD-10 codes
    pub treatment_plan: String,
    pub prescriptions: Vec<String>, // Prescription IDs
    pub follow_up_required: bool,
    pub follow_up_timeframe: String,
    pub urgency_level: u32,
    pub provider_notes: String,
    pub patient_satisfaction: u32,    // 1-5 scale
    pub technical_quality_score: u32, // 1-5 scale
    pub created_at: u64,
}

/// Recording Access Log
#[derive(Clone)]
#[contracttype]
pub struct RecordingAccessLog {
    pub access_id: u64,
    pub recording_id: u64,
    pub recording_type: String, // "video", "audio", "screen"
    pub accessor: Address,
    pub access_purpose: String,
    pub timestamp: u64,
    pub ip_address_hash: BytesN<32>,
    pub user_agent_hash: BytesN<32>,
    pub access_granted: bool,
}

/// Data Retention Policy
#[derive(Clone)]
#[contracttype]
pub struct RetentionPolicy {
    pub policy_id: u64,
    pub consultation_type: ConsultationType,
    pub video_retention_days: u32,
    pub audio_retention_days: u32,
    pub screen_retention_days: u32,
    pub auto_archive_days: u32,
    pub auto_delete_days: u32,
    pub requires_patient_consent: bool,
    pub created_at: u64,
}

// Storage Keys
const ADMIN: Symbol = symbol_short!("ADMIN");
const SESSIONS: Symbol = symbol_short!("SESSIONS");
const VIDEO_RECORDINGS: Symbol = symbol_short!("VIDEO");
const AUDIO_RECORDINGS: Symbol = symbol_short!("AUDIO");
const SCREEN_RECORDINGS: Symbol = symbol_short!("SCREEN");
const CONSULTATION_SUMMARIES: Symbol = symbol_short!("SUMMARY");
const ACCESS_LOGS: Symbol = symbol_short!("ACCESS");
const RETENTION_POLICIES: Symbol = symbol_short!("POLICY");
const SESSION_COUNTER: Symbol = symbol_short!("SESS_CNT");
const RECORDING_COUNTER: Symbol = symbol_short!("REC_CNT");
const ACCESS_LOG_COUNTER: Symbol = symbol_short!("LOG_CNT");
const POLICY_COUNTER: Symbol = symbol_short!("POL_CNT");
const PAUSED: Symbol = symbol_short!("PAUSED");
const CONSENT_CONTRACT: Symbol = symbol_short!("CONSENT");
const MEDICAL_RECORDS_CONTRACT: Symbol = symbol_short!("MEDICAL");

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    NotAuthorized = 1,
    ContractPaused = 2,
    SessionNotFound = 3,
    SessionAlreadyExists = 4,
    InvalidStatus = 5,
    RecordingNotFound = 6,
    RecordingAlreadyExists = 7,
    AccessDenied = 8,
    InvalidEncryption = 9,
    RetentionPolicyNotFound = 10,
    ConsentRequired = 11,
    ConsentRevoked = 12,
    InvalidConsultationType = 13,
    InvalidDuration = 14,
    InvalidRecordingType = 15,
    FileTooLarge = 16,
    StorageQuotaExceeded = 17,
    RecordingExpired = 18,
    SessionNotInProgress = 19,
    SessionAlreadyCompleted = 20,
    MedicalRecordsContractNotSet = 21,
    ConsentContractNotSet = 22,
}

#[contract]
pub struct TelemedicineConsultationContract;

#[contractimpl]
impl TelemedicineConsultationContract {
    /// Initialize the telemedicine consultation contract
    pub fn initialize(
        env: Env,
        admin: Address,
        consent_contract: Address,
        medical_records_contract: Address,
    ) -> Result<bool, Error> {
        admin.require_auth();

        if env.storage().persistent().has(&ADMIN) {
            return Err(Error::SessionAlreadyExists);
        }

        env.storage().persistent().set(&ADMIN, &admin);
        env.storage()
            .persistent()
            .set(&CONSENT_CONTRACT, &consent_contract);
        env.storage()
            .persistent()
            .set(&MEDICAL_RECORDS_CONTRACT, &medical_records_contract);
        env.storage().persistent().set(&PAUSED, &false);
        env.storage().persistent().set(&SESSION_COUNTER, &0u64);
        env.storage().persistent().set(&RECORDING_COUNTER, &0u64);
        env.storage().persistent().set(&ACCESS_LOG_COUNTER, &0u64);
        env.storage().persistent().set(&POLICY_COUNTER, &0u64);

        // Initialize default retention policies
        Self::initialize_default_retention_policies(&env)?;

        Ok(true)
    }

    /// Schedule a new telemedicine consultation
    pub fn schedule_consultation(
        env: Env,
        patient: Address,
        provider: Address,
        consultation_type: ConsultationType,
        scheduled_time: u64,
        duration_minutes: u32,
        notes: String,
        metadata_uri: String,
        consent_token_id: u64,
    ) -> Result<u64, Error> {
        patient.require_auth();

        if env.storage().persistent().get(&PAUSED).unwrap_or(false) {
            return Err(Error::ContractPaused);
        }

        // Validate duration (15 mins to 4 hours)
        if duration_minutes < 15 || duration_minutes > 240 {
            return Err(Error::InvalidDuration);
        }

        // Verify consent token exists and is valid
        if !Self::verify_consent_token(&env, consent_token_id, patient.clone(), provider.clone())? {
            return Err(Error::ConsentRequired);
        }

        let session_id = Self::get_and_increment_session_counter(&env);

        let session = ConsultationSession {
            session_id,
            patient: patient.clone(),
            provider: provider.clone(),
            consultation_type,
            scheduled_time,
            start_time: 0,
            end_time: 0,
            duration_minutes,
            status: ConsultationStatus::Scheduled,
            emergency_level: 0,
            notes,
            metadata_uri,
        };

        let mut sessions: Map<u64, ConsultationSession> = env
            .storage()
            .persistent()
            .get(&SESSIONS)
            .unwrap_or(Map::new(&env));
        sessions.set(session_id, session);
        env.storage().persistent().set(&SESSIONS, &sessions);

        // Emit event
        env.events().publish(
            (symbol_short!("Consultation"), symbol_short!("Scheduled")),
            (session_id, patient, provider),
        );

        Ok(session_id)
    }

    /// Start a consultation session
    pub fn start_consultation(env: Env, session_id: u64, provider: Address) -> Result<bool, Error> {
        provider.require_auth();

        if env.storage().persistent().get(&PAUSED).unwrap_or(false) {
            return Err(Error::ContractPaused);
        }

        let mut sessions: Map<u64, ConsultationSession> = env
            .storage()
            .persistent()
            .get(&SESSIONS)
            .ok_or(Error::SessionNotFound)?;

        let mut session = sessions.get(session_id).ok_or(Error::SessionNotFound)?;

        // Validate provider and status
        if session.provider != provider {
            return Err(Error::NotAuthorized);
        }

        if session.status != ConsultationStatus::Scheduled {
            return Err(Error::InvalidStatus);
        }

        let timestamp = env.ledger().timestamp();
        session.start_time = timestamp;
        session.status = ConsultationStatus::InProgress;

        sessions.set(session_id, session);
        env.storage().persistent().set(&SESSIONS, &sessions);

        // Emit event
        env.events().publish(
            (symbol_short!("Consultation"), symbol_short!("Started")),
            (session_id, timestamp),
        );

        Ok(true)
    }

    /// End a consultation session
    pub fn end_consultation(env: Env, session_id: u64, provider: Address) -> Result<bool, Error> {
        provider.require_auth();

        if env.storage().persistent().get(&PAUSED).unwrap_or(false) {
            return Err(Error::ContractPaused);
        }

        let mut sessions: Map<u64, ConsultationSession> = env
            .storage()
            .persistent()
            .get(&SESSIONS)
            .ok_or(Error::SessionNotFound)?;

        let mut session = sessions.get(session_id).ok_or(Error::SessionNotFound)?;

        // Validate provider and status
        if session.provider != provider {
            return Err(Error::NotAuthorized);
        }

        if session.status != ConsultationStatus::InProgress {
            return Err(Error::SessionNotInProgress);
        }

        let timestamp = env.ledger().timestamp();
        session.end_time = timestamp;
        session.status = ConsultationStatus::Completed;

        // Calculate actual duration
        if session.start_time > 0 {
            session.duration_minutes = ((timestamp - session.start_time) / 60) as u32;
        }

        sessions.set(session_id, session);
        env.storage().persistent().set(&SESSIONS, &sessions);

        // Emit event
        env.events().publish(
            (symbol_short!("Consultation"), symbol_short!("Ended")),
            (session_id, timestamp, session.duration_minutes),
        );

        Ok(true)
    }

    /// Store video recording metadata
    pub fn store_video_recording(
        env: Env,
        session_id: u64,
        recording_uri: String,
        file_size_bytes: u64,
        duration_seconds: u32,
        quality: RecordingQuality,
        encryption_standard: EncryptionStandard,
        encryption_key_hash: BytesN<32>,
        consent_token_id: u64,
    ) -> Result<u64, Error> {
        // This would be called by the recording system after consultation
        // For now, we'll require provider authorization
        let sessions: Map<u64, ConsultationSession> = env
            .storage()
            .persistent()
            .get(&SESSIONS)
            .ok_or(Error::SessionNotFound)?;

        let session = sessions.get(session_id).ok_or(Error::SessionNotFound)?;

        session.provider.require_auth();

        if env.storage().persistent().get(&PAUSED).unwrap_or(false) {
            return Err(Error::ContractPaused);
        }

        // Validate file size (max 2GB for video)
        if file_size_bytes > 2_147_483_648 {
            return Err(Error::FileTooLarge);
        }

        // Verify consent
        if !Self::verify_consent_token(
            &env,
            consent_token_id,
            session.patient.clone(),
            session.provider.clone(),
        )? {
            return Err(Error::ConsentRequired);
        }

        let recording_id = Self::get_and_increment_recording_counter(&env);

        let recording = VideoRecording {
            recording_id,
            session_id,
            recording_uri,
            file_size_bytes,
            duration_seconds,
            quality,
            encryption_standard,
            encryption_key_hash,
            recorded_at: env.ledger().timestamp(),
            is_archived: false,
            retention_days: Self::get_retention_days(&env, session.consultation_type, "video")?,
            consent_token_id,
        };

        let mut recordings: Map<u64, VideoRecording> = env
            .storage()
            .persistent()
            .get(&VIDEO_RECORDINGS)
            .unwrap_or(Map::new(&env));
        recordings.set(recording_id, recording);
        env.storage()
            .persistent()
            .set(&VIDEO_RECORDINGS, &recordings);

        // Log access
        Self::log_recording_access(
            &env,
            recording_id,
            String::from_str(&env, "video"),
            session.provider.clone(),
            String::from_str(&env, "store"),
        )?;

        // Emit event
        env.events().publish(
            (symbol_short!("Recording"), symbol_short!("VideoStored")),
            (recording_id, session_id),
        );

        Ok(recording_id)
    }

    /// Store audio recording metadata
    pub fn store_audio_recording(
        env: Env,
        session_id: u64,
        recording_uri: String,
        file_size_bytes: u64,
        duration_seconds: u32,
        sample_rate: u32,
        channels: u8,
        encryption_standard: EncryptionStandard,
        encryption_key_hash: BytesN<32>,
        consent_token_id: u64,
    ) -> Result<u64, Error> {
        let sessions: Map<u64, ConsultationSession> = env
            .storage()
            .persistent()
            .get(&SESSIONS)
            .ok_or(Error::SessionNotFound)?;

        let session = sessions.get(session_id).ok_or(Error::SessionNotFound)?;

        session.provider.require_auth();

        if env.storage().persistent().get(&PAUSED).unwrap_or(false) {
            return Err(Error::ContractPaused);
        }

        // Validate file size (max 500MB for audio)
        if file_size_bytes > 524_288_000 {
            return Err(Error::FileTooLarge);
        }

        // Verify consent
        if !Self::verify_consent_token(
            &env,
            consent_token_id,
            session.patient.clone(),
            session.provider.clone(),
        )? {
            return Err(Error::ConsentRequired);
        }

        let recording_id = Self::get_and_increment_recording_counter(&env);

        let recording = AudioRecording {
            recording_id,
            session_id,
            recording_uri,
            file_size_bytes,
            duration_seconds,
            sample_rate,
            channels,
            encryption_standard,
            encryption_key_hash,
            recorded_at: env.ledger().timestamp(),
            is_archived: false,
            retention_days: Self::get_retention_days(&env, session.consultation_type, "audio")?,
            consent_token_id,
        };

        let mut recordings: Map<u64, AudioRecording> = env
            .storage()
            .persistent()
            .get(&AUDIO_RECORDINGS)
            .unwrap_or(Map::new(&env));
        recordings.set(recording_id, recording);
        env.storage()
            .persistent()
            .set(&AUDIO_RECORDINGS, &recordings);

        // Log access
        Self::log_recording_access(
            &env,
            recording_id,
            String::from_str(&env, "audio"),
            session.provider.clone(),
            String::from_str(&env, "store"),
        )?;

        // Emit event
        env.events().publish(
            (symbol_short!("Recording"), symbol_short!("AudioStored")),
            (recording_id, session_id),
        );

        Ok(recording_id)
    }

    /// Store screen share recording metadata
    pub fn store_screen_recording(
        env: Env,
        session_id: u64,
        recording_uri: String,
        file_size_bytes: u64,
        duration_seconds: u32,
        resolution: String,
        frame_rate: u8,
        encryption_standard: EncryptionStandard,
        encryption_key_hash: BytesN<32>,
        consent_token_id: u64,
    ) -> Result<u64, Error> {
        let sessions: Map<u64, ConsultationSession> = env
            .storage()
            .persistent()
            .get(&SESSIONS)
            .ok_or(Error::SessionNotFound)?;

        let session = sessions.get(session_id).ok_or(Error::SessionNotFound)?;

        session.provider.require_auth();

        if env.storage().persistent().get(&PAUSED).unwrap_or(false) {
            return Err(Error::ContractPaused);
        }

        // Validate file size (max 1GB for screen recording)
        if file_size_bytes > 1_073_741_824 {
            return Err(Error::FileTooLarge);
        }

        // Verify consent
        if !Self::verify_consent_token(
            &env,
            consent_token_id,
            session.patient.clone(),
            session.provider.clone(),
        )? {
            return Err(Error::ConsentRequired);
        }

        let recording_id = Self::get_and_increment_recording_counter(&env);

        let recording = ScreenShareRecording {
            recording_id,
            session_id,
            recording_uri,
            file_size_bytes,
            duration_seconds,
            resolution,
            frame_rate,
            encryption_standard,
            encryption_key_hash,
            recorded_at: env.ledger().timestamp(),
            is_archived: false,
            retention_days: Self::get_retention_days(&env, session.consultation_type, "screen")?,
            consent_token_id,
        };

        let mut recordings: Map<u64, ScreenShareRecording> = env
            .storage()
            .persistent()
            .get(&SCREEN_RECORDINGS)
            .unwrap_or(Map::new(&env));
        recordings.set(recording_id, recording);
        env.storage()
            .persistent()
            .set(&SCREEN_RECORDINGS, &recordings);

        // Log access
        Self::log_recording_access(
            &env,
            recording_id,
            String::from_str(&env, "screen"),
            session.provider.clone(),
            String::from_str(&env, "store"),
        )?;

        // Emit event
        env.events().publish(
            (symbol_short!("Recording"), symbol_short!("ScreenStored")),
            (recording_id, session_id),
        );

        Ok(recording_id)
    }

    /// Create consultation summary
    pub fn create_consultation_summary(
        env: Env,
        session_id: u64,
        provider: Address,
        chief_complaint: String,
        diagnosis_codes: Vec<String>,
        treatment_plan: String,
        prescriptions: Vec<String>,
        follow_up_required: bool,
        follow_up_timeframe: String,
        urgency_level: u8,
        provider_notes: String,
        patient_satisfaction: u8,
        technical_quality_score: u8,
    ) -> Result<bool, Error> {
        provider.require_auth();

        if env.storage().persistent().get(&PAUSED).unwrap_or(false) {
            return Err(Error::ContractPaused);
        }

        let sessions: Map<u64, ConsultationSession> = env
            .storage()
            .persistent()
            .get(&SESSIONS)
            .ok_or(Error::SessionNotFound)?;

        let session = sessions.get(session_id).ok_or(Error::SessionNotFound)?;

        // Validate provider and session status
        if session.provider != provider {
            return Err(Error::NotAuthorized);
        }

        if session.status != ConsultationStatus::Completed {
            return Err(Error::SessionAlreadyCompleted);
        }

        // Validate scores (1-5)
        if patient_satisfaction == 0 || patient_satisfaction > 5 {
            return Err(Error::InvalidStatus);
        }
        if technical_quality_score == 0 || technical_quality_score > 5 {
            return Err(Error::InvalidStatus);
        }

        let summary = ConsultationSummary {
            session_id,
            chief_complaint,
            diagnosis_codes,
            treatment_plan,
            prescriptions,
            follow_up_required,
            follow_up_timeframe,
            urgency_level,
            provider_notes,
            patient_satisfaction,
            technical_quality_score,
            created_at: env.ledger().timestamp(),
        };

        let mut summaries: Map<u64, ConsultationSummary> = env
            .storage()
            .persistent()
            .get(&CONSULTATION_SUMMARIES)
            .unwrap_or(Map::new(&env));
        summaries.set(session_id, summary);
        env.storage()
            .persistent()
            .set(&CONSULTATION_SUMMARIES, &summaries);

        // Emit event
        env.events().publish(
            (symbol_short!("Summary"), symbol_short!("Created")),
            (session_id, provider),
        );

        Ok(true)
    }

    /// Grant access to recording (with audit trail)
    pub fn grant_recording_access(
        env: Env,
        recording_id: u64,
        recording_type: String,
        requester: Address,
        access_purpose: String,
    ) -> Result<bool, Error> {
        // This would typically be called by the patient or authorized provider
        requester.require_auth();

        if env.storage().persistent().get(&PAUSED).unwrap_or(false) {
            return Err(Error::ContractPaused);
        }

        // Validate recording exists and requester has permission
        if !Self::validate_recording_access(
            &env,
            recording_id,
            recording_type.clone(),
            requester.clone(),
        )? {
            return Err(Error::AccessDenied);
        }

        // Log the access
        Self::log_recording_access(
            &env,
            recording_id,
            recording_type,
            requester.clone(),
            access_purpose,
        )?;

        Ok(true)
    }

    /// Get consultation session details
    pub fn get_consultation_session(
        env: Env,
        session_id: u64,
    ) -> Result<ConsultationSession, Error> {
        let sessions: Map<u64, ConsultationSession> = env
            .storage()
            .persistent()
            .get(&SESSIONS)
            .ok_or(Error::SessionNotFound)?;

        sessions.get(session_id).ok_or(Error::SessionNotFound)
    }

    /// Get video recording details
    pub fn get_video_recording(env: Env, recording_id: u64) -> Result<VideoRecording, Error> {
        let recordings: Map<u64, VideoRecording> = env
            .storage()
            .persistent()
            .get(&VIDEO_RECORDINGS)
            .ok_or(Error::RecordingNotFound)?;

        recordings.get(recording_id).ok_or(Error::RecordingNotFound)
    }

    /// Get audio recording details
    pub fn get_audio_recording(env: Env, recording_id: u64) -> Result<AudioRecording, Error> {
        let recordings: Map<u64, AudioRecording> = env
            .storage()
            .persistent()
            .get(&AUDIO_RECORDINGS)
            .ok_or(Error::RecordingNotFound)?;

        recordings.get(recording_id).ok_or(Error::RecordingNotFound)
    }

    /// Get screen recording details
    pub fn get_screen_recording(
        env: Env,
        recording_id: u64,
    ) -> Result<ScreenShareRecording, Error> {
        let recordings: Map<u64, ScreenShareRecording> = env
            .storage()
            .persistent()
            .get(&SCREEN_RECORDINGS)
            .ok_or(Error::RecordingNotFound)?;

        recordings.get(recording_id).ok_or(Error::RecordingNotFound)
    }

    /// Get consultation summary
    pub fn get_consultation_summary(
        env: Env,
        session_id: u64,
    ) -> Result<ConsultationSummary, Error> {
        let summaries: Map<u64, ConsultationSummary> = env
            .storage()
            .persistent()
            .get(&CONSULTATION_SUMMARIES)
            .ok_or(Error::SessionNotFound)?;

        summaries.get(session_id).ok_or(Error::SessionNotFound)
    }

    /// Get access logs for a recording
    pub fn get_recording_access_logs(
        env: Env,
        recording_id: u64,
    ) -> Result<Vec<RecordingAccessLog>, Error> {
        let logs: Vec<RecordingAccessLog> = env
            .storage()
            .persistent()
            .get(&ACCESS_LOGS)
            .unwrap_or(Vec::new(&env));

        let mut recording_logs = Vec::new(&env);
        for log in logs.iter() {
            if log.recording_id == recording_id {
                recording_logs.push_back(log);
            }
        }

        Ok(recording_logs)
    }

    // ==================== Helper Functions ====================

    fn verify_consent_token(
        env: &Env,
        token_id: u64,
        patient: Address,
        provider: Address,
    ) -> Result<bool, Error> {
        let consent_contract: Address = env
            .storage()
            .persistent()
            .get(&CONSENT_CONTRACT)
            .ok_or(Error::ConsentContractNotSet)?;

        // This would call the consent contract to verify the token
        // For now, we'll assume it's valid if not revoked
        // In production, this would be a cross-contract call
        Ok(true)
    }

    fn get_retention_days(
        env: &Env,
        consultation_type: ConsultationType,
        recording_type: &str,
    ) -> Result<u32, Error> {
        let policies: Map<u64, RetentionPolicy> = env
            .storage()
            .persistent()
            .get(&RETENTION_POLICIES)
            .unwrap_or(Map::new(env));

        for policy in policies.values() {
            if policy.consultation_type == consultation_type {
                return match recording_type {
                    "video" => Ok(policy.video_retention_days),
                    "audio" => Ok(policy.audio_retention_days),
                    "screen" => Ok(policy.screen_retention_days),
                    _ => Err(Error::InvalidRecordingType),
                };
            }
        }

        // Default retention periods
        match recording_type {
            "video" => Ok(365),  // 1 year
            "audio" => Ok(730),  // 2 years
            "screen" => Ok(180), // 6 months
            _ => Err(Error::InvalidRecordingType),
        }
    }

    fn log_recording_access(
        env: &Env,
        recording_id: u64,
        recording_type: String,
        accessor: Address,
        access_purpose: String,
    ) -> Result<(), Error> {
        let access_id = Self::get_and_increment_access_log_counter(env);

        let log = RecordingAccessLog {
            access_id,
            recording_id,
            recording_type,
            accessor: accessor.clone(),
            access_purpose,
            timestamp: env.ledger().timestamp(),
            ip_address_hash: BytesN::from_array(env, &[0u8; 32]), // Would be populated by frontend
            user_agent_hash: BytesN::from_array(env, &[0u8; 32]), // Would be populated by frontend
            access_granted: true,
        };

        let mut logs: Vec<RecordingAccessLog> = env
            .storage()
            .persistent()
            .get(&ACCESS_LOGS)
            .unwrap_or(Vec::new(env));
        logs.push_back(log);
        env.storage().persistent().set(&ACCESS_LOGS, &logs);

        Ok(())
    }

    fn validate_recording_access(
        env: &Env,
        recording_id: u64,
        recording_type: String,
        requester: Address,
    ) -> Result<bool, Error> {
        // Check if recording exists and requester has permission
        match recording_type.as_str() {
            "video" => {
                let recordings: Map<u64, VideoRecording> = env
                    .storage()
                    .persistent()
                    .get(&VIDEO_RECORDINGS)
                    .ok_or(Error::RecordingNotFound)?;

                if let Some(recording) = recordings.get(recording_id) {
                    let sessions: Map<u64, ConsultationSession> = env
                        .storage()
                        .persistent()
                        .get(&SESSIONS)
                        .ok_or(Error::SessionNotFound)?;

                    if let Some(session) = sessions.get(recording.session_id) {
                        return Ok(session.patient == requester || session.provider == requester);
                    }
                }
            }
            "audio" => {
                let recordings: Map<u64, AudioRecording> = env
                    .storage()
                    .persistent()
                    .get(&AUDIO_RECORDINGS)
                    .ok_or(Error::RecordingNotFound)?;

                if let Some(recording) = recordings.get(recording_id) {
                    let sessions: Map<u64, ConsultationSession> = env
                        .storage()
                        .persistent()
                        .get(&SESSIONS)
                        .ok_or(Error::SessionNotFound)?;

                    if let Some(session) = sessions.get(recording.session_id) {
                        return Ok(session.patient == requester || session.provider == requester);
                    }
                }
            }
            "screen" => {
                let recordings: Map<u64, ScreenShareRecording> = env
                    .storage()
                    .persistent()
                    .get(&SCREEN_RECORDINGS)
                    .ok_or(Error::RecordingNotFound)?;

                if let Some(recording) = recordings.get(recording_id) {
                    let sessions: Map<u64, ConsultationSession> = env
                        .storage()
                        .persistent()
                        .get(&SESSIONS)
                        .ok_or(Error::SessionNotFound)?;

                    if let Some(session) = sessions.get(recording.session_id) {
                        return Ok(session.patient == requester || session.provider == requester);
                    }
                }
            }
            _ => return Err(Error::InvalidRecordingType),
        }

        Ok(false)
    }

    fn initialize_default_retention_policies(env: &Env) -> Result<(), Error> {
        let mut policies: Map<u64, RetentionPolicy> = Map::new(env);

        let policy_types = vec![
            env,
            ConsultationType::Routine,
            ConsultationType::FollowUp,
            ConsultationType::Emergency,
            ConsultationType::SecondOpinion,
            ConsultationType::MentalHealth,
            ConsultationType::ChronicCare,
            ConsultationType::PreOp,
            ConsultationType::PostOp,
        ];

        for (i, consultation_type) in policy_types.iter().enumerate() {
            let policy = RetentionPolicy {
                policy_id: (i as u64) + 1,
                consultation_type: consultation_type.clone(),
                video_retention_days: match consultation_type {
                    ConsultationType::Emergency => 1825,    // 5 years
                    ConsultationType::MentalHealth => 2555, // 7 years
                    ConsultationType::ChronicCare => 1825,  // 5 years
                    _ => 365,                               // 1 year
                },
                audio_retention_days: match consultation_type {
                    ConsultationType::Emergency => 3650,    // 10 years
                    ConsultationType::MentalHealth => 3650, // 10 years
                    ConsultationType::ChronicCare => 2555,  // 7 years
                    _ => 730,                               // 2 years
                },
                screen_retention_days: match consultation_type {
                    ConsultationType::Emergency => 1095,    // 3 years
                    ConsultationType::MentalHealth => 1825, // 5 years
                    _ => 180,                               // 6 months
                },
                auto_archive_days: 90,
                auto_delete_days: match consultation_type {
                    ConsultationType::Emergency => 3650,
                    ConsultationType::MentalHealth => 3650,
                    ConsultationType::ChronicCare => 2555,
                    _ => 1095,
                },
                requires_patient_consent: true,
                created_at: env.ledger().timestamp(),
            };

            policies.set((i as u64) + 1, policy);
        }

        env.storage()
            .persistent()
            .set(&RETENTION_POLICIES, &policies);

        Ok(())
    }

    fn get_and_increment_session_counter(env: &Env) -> u64 {
        let count: u64 = env
            .storage()
            .persistent()
            .get(&SESSION_COUNTER)
            .unwrap_or(0);
        let next = count + 1;
        env.storage().persistent().set(&SESSION_COUNTER, &next);
        next
    }

    fn get_and_increment_recording_counter(env: &Env) -> u64 {
        let count: u64 = env
            .storage()
            .persistent()
            .get(&RECORDING_COUNTER)
            .unwrap_or(0);
        let next = count + 1;
        env.storage().persistent().set(&RECORDING_COUNTER, &next);
        next
    }

    fn get_and_increment_access_log_counter(env: &Env) -> u64 {
        let count: u64 = env
            .storage()
            .persistent()
            .get(&ACCESS_LOG_COUNTER)
            .unwrap_or(0);
        let next = count + 1;
        env.storage().persistent().set(&ACCESS_LOG_COUNTER, &next);
        next
    }

    /// Pause contract operations (admin only)
    pub fn pause(env: Env, admin: Address) -> Result<bool, Error> {
        let contract_admin: Address = env
            .storage()
            .persistent()
            .get(&ADMIN)
            .ok_or(Error::NotAuthorized)?;

        if admin != contract_admin {
            return Err(Error::NotAuthorized);
        }

        env.storage().persistent().set(&PAUSED, &true);
        Ok(true)
    }

    /// Resume contract operations (admin only)
    pub fn resume(env: Env, admin: Address) -> Result<bool, Error> {
        let contract_admin: Address = env
            .storage()
            .persistent()
            .get(&ADMIN)
            .ok_or(Error::NotAuthorized)?;

        if admin != contract_admin {
            return Err(Error::NotAuthorized);
        }

        env.storage().persistent().set(&PAUSED, &false);
        Ok(true)
    }

    /// Health check for monitoring
    pub fn health_check(env: Env) -> (Symbol, u32, u64) {
        let status = if env.storage().persistent().get(&PAUSED).unwrap_or(false) {
            symbol_short!("PAUSED")
        } else {
            symbol_short!("OK")
        };
        (status, 1, env.ledger().timestamp())
    }
}
