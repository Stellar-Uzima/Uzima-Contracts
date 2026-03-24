#![no_std]
#![allow(clippy::arithmetic_side_effects)]

use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype, symbol_short, Address, BytesN, Env,
    String, Vec,
};

// Error types for Healthcare Reputation System
#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    AlreadyInitialized = 1,
    NotInitialized = 2,
    NotAuthorized = 3,
    ProviderNotFound = 4,
    CredentialNotFound = 5,
    InvalidCredentialType = 6,
    CredentialExpired = 7,
    CredentialRevoked = 8,
    DuplicateCredential = 9,
    InvalidRating = 10,
    FeedbackNotFound = 11,
    DisputeNotFound = 12,
    InsufficientReputation = 13,
    NotVerifiedProvider = 14,
    InvalidConductEntry = 15,
    ConductEntryNotFound = 16,
}

// Credential types for healthcare providers
#[contracttype]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum CredentialType {
    MedicalLicense = 0,
    BoardCertification = 1,
    Specialization = 2,
    DEARegistration = 3,
    StateLicense = 4,
    HospitalPrivileges = 5,
    InsuranceCredentials = 6,
    ContinuingEducation = 7,
}

// Provider credential information
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProviderCredential {
    pub credential_id: BytesN<32>,
    pub provider: Address,
    pub credential_type: CredentialType,
    pub issuer: Address,
    pub issue_date: u64,
    pub expiration_date: u64,
    pub credential_hash: BytesN<32>,
    pub is_active: bool,
    pub verification_status: VerificationStatus,
}

// Verification status for credentials
#[contracttype]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum VerificationStatus {
    Pending = 0,
    Verified = 1,
    Rejected = 2,
    Expired = 3,
    Revoked = 4,
}

// Patient feedback structure
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PatientFeedback {
    pub feedback_id: BytesN<32>,
    pub provider: Address,
    pub patient: Address,
    pub rating: u32, // 1-5 stars
    pub comment: String,
    pub timestamp: u64,
    pub is_verified: bool,
    pub feedback_type: FeedbackType,
}

// Types of patient feedback
#[contracttype]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum FeedbackType {
    General = 0,
    Treatment = 1,
    Communication = 2,
    BedsideManner = 3,
    WaitTime = 4,
    Facility = 5,
}

// Professional conduct tracking
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ConductEntry {
    pub entry_id: BytesN<32>,
    pub provider: Address,
    pub conduct_type: ConductType,
    pub description: String,
    pub severity: u32, // 1-10
    pub reporter: Address,
    pub timestamp: u64,
    pub is_verified: bool,
    pub action_taken: String,
}

// Types of professional conduct
#[contracttype]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum ConductType {
    Positive = 0,
    Complaint = 1,
    Malpractice = 2,
    EthicsViolation = 3,
    ProfessionalAchievement = 4,
    CommunityService = 5,
}

// Reputation dispute structure
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReputationDispute {
    pub dispute_id: BytesN<32>,
    pub provider: Address,
    pub challenger: Address,
    pub dispute_type: DisputeType,
    pub description: String,
    pub evidence: Vec<String>,
    pub timestamp: u64,
    pub status: DisputeStatus,
    pub resolution: String,
}

// Types of reputation disputes
#[contracttype]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum DisputeType {
    Credential = 0,
    Feedback = 1,
    Conduct = 2,
    Score = 3,
}

// Dispute status
#[contracttype]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum DisputeStatus {
    Pending = 0,
    UnderReview = 1,
    Resolved = 2,
    Rejected = 3,
}

// Reputation score components
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReputationComponents {
    pub credential_score: u32, // 40% weight
    pub feedback_score: u32,   // 30% weight
    pub conduct_score: u32,    // 20% weight
    pub experience_score: u32, // 10% weight
    pub total_score: u32,      // Weighted total
}

// Storage keys
#[contracttype]
pub enum DataKey {
    Admin,
    Initialized,
    ProviderCredential(Address, BytesN<32>),
    ProviderCredentials(Address), // Vec<BytesN<32>>
    PatientFeedback(BytesN<32>),
    ProviderFeedback(Address), // Vec<BytesN<32>>
    ConductEntry(BytesN<32>),
    ProviderConduct(Address), // Vec<BytesN<32>>
    ReputationDispute(BytesN<32>),
    ProviderDisputes(Address), // Vec<BytesN<32>>
    ReputationScore(Address),
    ReputationComponents(Address),
    CredentialVerification(BytesN<32>),
    ExpirationNotification(Address, u64),
}

#[contract]
pub struct HealthcareReputationSystem;

#[contractimpl]
impl HealthcareReputationSystem {
    // Initialize the healthcare reputation system
    pub fn initialize(env: Env, admin: Address) -> Result<(), Error> {
        if env.storage().instance().has(&DataKey::Initialized) {
            return Err(Error::AlreadyInitialized);
        }

        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage().instance().set(&DataKey::Initialized, &true);

        env.events()
            .publish((symbol_short!("HLTHREP"), symbol_short!("INIT")), admin);
        Ok(())
    }

    // Add provider credential
    #[allow(clippy::too_many_arguments)]
    pub fn add_credential(
        env: Env,
        provider: Address,
        credential_id: BytesN<32>,
        credential_type: CredentialType,
        issuer: Address,
        issue_date: u64,
        expiration_date: u64,
        credential_hash: BytesN<32>,
    ) -> Result<(), Error> {
        provider.require_auth();
        Self::require_initialized(&env)?;

        // Check if credential already exists
        if env.storage().persistent().has(&DataKey::ProviderCredential(
            provider.clone(),
            credential_id.clone(),
        )) {
            return Err(Error::DuplicateCredential);
        }

        let credential = ProviderCredential {
            credential_id: credential_id.clone(),
            provider: provider.clone(),
            credential_type,
            issuer,
            issue_date,
            expiration_date,
            credential_hash,
            is_active: true,
            verification_status: VerificationStatus::Pending,
        };

        // Store credential
        env.storage().persistent().set(
            &DataKey::ProviderCredential(provider.clone(), credential_id.clone()),
            &credential,
        );

        // Update provider's credential list
        let mut credentials: Vec<BytesN<32>> = env
            .storage()
            .persistent()
            .get(&DataKey::ProviderCredentials(provider.clone()))
            .unwrap_or(Vec::new(&env));
        credentials.push_back(credential_id.clone());
        env.storage().persistent().set(
            &DataKey::ProviderCredentials(provider.clone()),
            &credentials,
        );

        // Schedule expiration notification
        env.storage().persistent().set(
            &DataKey::ExpirationNotification(provider.clone(), expiration_date),
            &credential_id,
        );

        env.events().publish(
            (symbol_short!("HLTHREP"), symbol_short!("CRED_ADD")),
            (provider, credential_id),
        );

        Ok(())
    }

    // Verify provider credential
    pub fn verify_credential(
        env: Env,
        admin: Address,
        provider: Address,
        credential_id: BytesN<32>,
        verified: bool,
    ) -> Result<(), Error> {
        let admin_clone = admin.clone();
        admin.require_auth();
        Self::require_admin(&env, &admin_clone)?;

        let mut credential: ProviderCredential = env
            .storage()
            .persistent()
            .get(&DataKey::ProviderCredential(
                provider.clone(),
                credential_id.clone(),
            ))
            .ok_or(Error::CredentialNotFound)?;

        credential.verification_status = if verified {
            VerificationStatus::Verified
        } else {
            VerificationStatus::Rejected
        };

        env.storage().persistent().set(
            &DataKey::ProviderCredential(provider.clone(), credential_id.clone()),
            &credential,
        );

        // Update reputation score
        Self::update_reputation_score(&env, provider.clone())?;

        env.events().publish(
            (symbol_short!("HLTHREP"), symbol_short!("CRED_VER")),
            (provider, credential_id, verified),
        );

        Ok(())
    }

    // Add patient feedback
    pub fn add_feedback(
        env: Env,
        provider: Address,
        patient: Address,
        rating: u32,
        comment: String,
        feedback_type: FeedbackType,
    ) -> Result<(), Error> {
        patient.require_auth();
        Self::require_initialized(&env)?;

        if !(1..=5).contains(&rating) {
            return Err(Error::InvalidRating);
        }

        let feedback_id = BytesN::from_array(
            &env,
            &[
                (env.ledger().timestamp() >> 24) as u8,
                (env.ledger().timestamp() >> 16) as u8,
                (env.ledger().timestamp() >> 8) as u8,
                env.ledger().timestamp() as u8,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
            ],
        );

        let feedback = PatientFeedback {
            feedback_id: feedback_id.clone(),
            provider: provider.clone(),
            patient,
            rating,
            comment,
            timestamp: env.ledger().timestamp(),
            is_verified: false,
            feedback_type,
        };

        // Store feedback
        env.storage()
            .persistent()
            .set(&DataKey::PatientFeedback(feedback_id.clone()), &feedback);

        // Update provider's feedback list
        let mut feedback_list: Vec<BytesN<32>> = env
            .storage()
            .persistent()
            .get(&DataKey::ProviderFeedback(provider.clone()))
            .unwrap_or(Vec::new(&env));
        feedback_list.push_back(feedback_id.clone());
        env.storage()
            .persistent()
            .set(&DataKey::ProviderFeedback(provider.clone()), &feedback_list);

        // Update reputation score
        Self::update_reputation_score(&env, provider.clone())?;

        env.events().publish(
            (symbol_short!("HLTHREP"), symbol_short!("FEEDBACK")),
            (provider, feedback_id, rating),
        );

        Ok(())
    }

    // Add professional conduct entry
    pub fn add_conduct_entry(
        env: Env,
        reporter: Address,
        provider: Address,
        conduct_type: ConductType,
        description: String,
        severity: u32,
        action_taken: String,
    ) -> Result<(), Error> {
        reporter.require_auth();
        Self::require_initialized(&env)?;

        if !(1..=10).contains(&severity) {
            return Err(Error::InvalidConductEntry);
        }

        let entry_id = BytesN::from_array(
            &env,
            &[
                (env.ledger().timestamp() >> 24) as u8,
                (env.ledger().timestamp() >> 16) as u8,
                (env.ledger().timestamp() >> 8) as u8,
                env.ledger().timestamp() as u8,
                1,
                1,
                1,
                1,
                1,
                1,
                1,
                1,
                1,
                1,
                1,
                1,
                1,
                1,
                1,
                1,
                1,
                1,
                1,
                1,
                1,
                1,
                1,
                1,
                1,
                1,
                1,
                1,
            ],
        );

        let conduct_entry = ConductEntry {
            entry_id: entry_id.clone(),
            provider: provider.clone(),
            conduct_type,
            description,
            severity,
            reporter,
            timestamp: env.ledger().timestamp(),
            is_verified: false,
            action_taken,
        };

        // Store conduct entry
        env.storage()
            .persistent()
            .set(&DataKey::ConductEntry(entry_id.clone()), &conduct_entry);

        // Update provider's conduct list
        let mut conduct_list: Vec<BytesN<32>> = env
            .storage()
            .persistent()
            .get(&DataKey::ProviderConduct(provider.clone()))
            .unwrap_or(Vec::new(&env));
        conduct_list.push_back(entry_id.clone());
        env.storage()
            .persistent()
            .set(&DataKey::ProviderConduct(provider.clone()), &conduct_list);

        // Update reputation score
        Self::update_reputation_score(&env, provider.clone())?;

        env.events().publish(
            (symbol_short!("HLTHREP"), symbol_short!("CONDUCT")),
            (provider, entry_id, conduct_type),
        );

        Ok(())
    }

    // Create reputation dispute
    pub fn create_dispute(
        env: Env,
        challenger: Address,
        provider: Address,
        dispute_type: DisputeType,
        description: String,
        evidence: Vec<String>,
    ) -> Result<(), Error> {
        challenger.require_auth();
        Self::require_initialized(&env)?;

        let dispute_id = BytesN::from_array(
            &env,
            &[
                (env.ledger().timestamp() >> 24) as u8,
                (env.ledger().timestamp() >> 16) as u8,
                (env.ledger().timestamp() >> 8) as u8,
                env.ledger().timestamp() as u8,
                2,
                2,
                2,
                2,
                2,
                2,
                2,
                2,
                2,
                2,
                2,
                2,
                2,
                2,
                2,
                2,
                2,
                2,
                2,
                2,
                2,
                2,
                2,
                2,
                2,
                2,
                2,
                2,
            ],
        );

        let dispute = ReputationDispute {
            dispute_id: dispute_id.clone(),
            provider: provider.clone(),
            challenger,
            dispute_type,
            description,
            evidence,
            timestamp: env.ledger().timestamp(),
            status: DisputeStatus::Pending,
            resolution: String::from_str(&env, ""),
        };

        // Store dispute
        env.storage()
            .persistent()
            .set(&DataKey::ReputationDispute(dispute_id.clone()), &dispute);

        // Update provider's dispute list
        let mut dispute_list: Vec<BytesN<32>> = env
            .storage()
            .persistent()
            .get(&DataKey::ProviderDisputes(provider.clone()))
            .unwrap_or(Vec::new(&env));
        dispute_list.push_back(dispute_id.clone());
        env.storage()
            .persistent()
            .set(&DataKey::ProviderDisputes(provider.clone()), &dispute_list);

        env.events().publish(
            (symbol_short!("HLTHREP"), symbol_short!("DISPUTE")),
            (provider, dispute_id, dispute_type),
        );

        Ok(())
    }

    // Resolve reputation dispute
    pub fn resolve_dispute(
        env: Env,
        admin: Address,
        dispute_id: BytesN<32>,
        approved: bool,
        resolution: String,
    ) -> Result<(), Error> {
        admin.require_auth();
        Self::require_admin(&env, &admin)?;

        let mut dispute: ReputationDispute = env
            .storage()
            .persistent()
            .get(&DataKey::ReputationDispute(dispute_id.clone()))
            .ok_or(Error::DisputeNotFound)?;

        dispute.status = if approved {
            DisputeStatus::Resolved
        } else {
            DisputeStatus::Rejected
        };
        dispute.resolution = resolution;

        env.storage()
            .persistent()
            .set(&DataKey::ReputationDispute(dispute_id.clone()), &dispute);

        // Update reputation score if dispute was resolved in favor of challenger
        if approved {
            Self::update_reputation_score(&env, dispute.provider.clone())?;
        }

        env.events().publish(
            (symbol_short!("HLTHREP"), symbol_short!("DISP_RES")),
            (dispute_id, approved),
        );

        Ok(())
    }

    // Calculate and update reputation score
    fn update_reputation_score(env: &Env, provider: Address) -> Result<(), Error> {
        let components = Self::calculate_reputation_components(env, provider.clone())?;
        let total_score = components.credential_score * 40 / 100
            + components.feedback_score * 30 / 100
            + components.conduct_score * 20 / 100
            + components.experience_score * 10 / 100;

        // Store components and total score
        env.storage().persistent().set(
            &DataKey::ReputationComponents(provider.clone()),
            &components,
        );
        env.storage()
            .persistent()
            .set(&DataKey::ReputationScore(provider), &total_score);

        Ok(())
    }

    // Calculate reputation components
    fn calculate_reputation_components(
        env: &Env,
        provider: Address,
    ) -> Result<ReputationComponents, Error> {
        let credential_score = Self::calculate_credential_score(env, provider.clone())?;
        let feedback_score = Self::calculate_feedback_score(env, provider.clone())?;
        let conduct_score = Self::calculate_conduct_score(env, provider.clone())?;
        let experience_score = Self::calculate_experience_score(env, provider)?;

        Ok(ReputationComponents {
            credential_score,
            feedback_score,
            conduct_score,
            experience_score,
            total_score: 0, // Will be calculated in update_reputation_score
        })
    }

    // Calculate credential score (0-100)
    fn calculate_credential_score(env: &Env, provider: Address) -> Result<u32, Error> {
        let credentials: Vec<BytesN<32>> = env
            .storage()
            .persistent()
            .get(&DataKey::ProviderCredentials(provider.clone()))
            .unwrap_or(Vec::new(env));

        if credentials.is_empty() {
            return Ok(0);
        }

        let mut verified_count = 0;
        let mut total_weight = 0;

        for credential_id in credentials.iter() {
            if let Some(credential) = env
                .storage()
                .persistent()
                .get::<DataKey, ProviderCredential>(&DataKey::ProviderCredential(
                    provider.clone(),
                    credential_id.clone(),
                ))
            {
                let weight = match credential.credential_type {
                    CredentialType::MedicalLicense => 30,
                    CredentialType::BoardCertification => 25,
                    CredentialType::Specialization => 20,
                    CredentialType::DEARegistration => 15,
                    CredentialType::StateLicense => 10,
                    _ => 5,
                };

                total_weight += weight;
                if credential.verification_status == VerificationStatus::Verified {
                    verified_count += weight;
                }
            }
        }

        if total_weight == 0 {
            Ok(0)
        } else {
            Ok((verified_count * 100) / total_weight)
        }
    }

    // Calculate feedback score (0-100)
    fn calculate_feedback_score(env: &Env, provider: Address) -> Result<u32, Error> {
        let feedback_list: Vec<BytesN<32>> = env
            .storage()
            .persistent()
            .get(&DataKey::ProviderFeedback(provider.clone()))
            .unwrap_or(Vec::new(env));

        if feedback_list.is_empty() {
            return Ok(50); // Neutral score for no feedback
        }

        let mut total_rating = 0;
        let mut count = 0;

        for feedback_id in feedback_list.iter() {
            if let Some(feedback) = env
                .storage()
                .persistent()
                .get::<DataKey, PatientFeedback>(&DataKey::PatientFeedback(feedback_id.clone()))
            {
                total_rating += feedback.rating;
                count += 1;
            }
        }

        if count == 0 {
            Ok(50)
        } else {
            // Convert 1-5 scale to 0-100 scale
            Ok((total_rating * 20) / count)
        }
    }

    // Calculate conduct score (0-100)
    fn calculate_conduct_score(env: &Env, provider: Address) -> Result<u32, Error> {
        let conduct_list: Vec<BytesN<32>> = env
            .storage()
            .persistent()
            .get(&DataKey::ProviderConduct(provider.clone()))
            .unwrap_or(Vec::new(env));

        if conduct_list.is_empty() {
            return Ok(70); // Good baseline for no conduct issues
        }

        let mut score: u32 = 70; // Start with good baseline

        for entry_id in conduct_list.iter() {
            if let Some(entry) = env
                .storage()
                .persistent()
                .get::<DataKey, ConductEntry>(&DataKey::ConductEntry(entry_id.clone()))
            {
                match entry.conduct_type {
                    ConductType::Positive | ConductType::ProfessionalAchievement => {
                        score = score.saturating_add(5);
                    }
                    ConductType::Complaint => {
                        score = score.saturating_sub(entry.severity);
                    }
                    ConductType::Malpractice => {
                        score = score.saturating_sub(entry.severity * 2);
                    }
                    ConductType::EthicsViolation => {
                        score = score.saturating_sub(entry.severity * 3);
                    }
                    _ => {}
                }
            }
        }

        Ok(score.clamp(0, 100))
    }

    // Calculate experience score (0-100)
    fn calculate_experience_score(env: &Env, _provider: Address) -> Result<u32, Error> {
        // Get provider profile from provider directory if available
        // For now, use a simple time-based calculation
        let current_time = env.ledger().timestamp();

        // Assume provider joined 1 year ago for this example
        // In real implementation, this would come from provider directory
        let join_time = current_time.saturating_sub(365 * 24 * 60 * 60);
        let years_experience = (current_time.saturating_sub(join_time)) / (365 * 24 * 60 * 60);

        // Cap at 20 years for maximum score
        let experience_score = (years_experience.min(20) * 5).min(100);
        Ok(experience_score as u32)
    }

    // Get provider reputation score
    pub fn get_reputation_score(env: Env, provider: Address) -> Result<u32, Error> {
        env.storage()
            .persistent()
            .get(&DataKey::ReputationScore(provider))
            .ok_or(Error::ProviderNotFound)
    }

    // Get provider reputation components
    pub fn get_reputation_components(
        env: Env,
        provider: Address,
    ) -> Result<ReputationComponents, Error> {
        env.storage()
            .persistent()
            .get(&DataKey::ReputationComponents(provider))
            .ok_or(Error::ProviderNotFound)
    }

    // Check if provider meets minimum reputation threshold
    pub fn check_reputation_threshold(
        env: Env,
        provider: Address,
        threshold: u32,
    ) -> Result<bool, Error> {
        let score = Self::get_reputation_score(env, provider)?;
        Ok(score >= threshold)
    }

    // Get provider credentials
    pub fn get_provider_credentials(
        env: Env,
        provider: Address,
    ) -> Result<Vec<ProviderCredential>, Error> {
        let credential_ids: Vec<BytesN<32>> = env
            .storage()
            .persistent()
            .get(&DataKey::ProviderCredentials(provider.clone()))
            .unwrap_or(Vec::new(&env));

        let mut credentials = Vec::new(&env);
        for credential_id in credential_ids.iter() {
            if let Some(credential) = env
                .storage()
                .persistent()
                .get::<DataKey, ProviderCredential>(&DataKey::ProviderCredential(
                    provider.clone(),
                    credential_id.clone(),
                ))
            {
                credentials.push_back(credential);
            }
        }

        Ok(credentials)
    }

    // Get provider feedback
    pub fn get_provider_feedback(
        env: Env,
        provider: Address,
    ) -> Result<Vec<PatientFeedback>, Error> {
        let feedback_ids: Vec<BytesN<32>> = env
            .storage()
            .persistent()
            .get(&DataKey::ProviderFeedback(provider.clone()))
            .unwrap_or(Vec::new(&env));

        let mut feedback = Vec::new(&env);
        for feedback_id in feedback_ids.iter() {
            if let Some(fb) = env
                .storage()
                .persistent()
                .get::<DataKey, PatientFeedback>(&DataKey::PatientFeedback(feedback_id.clone()))
            {
                feedback.push_back(fb);
            }
        }

        Ok(feedback)
    }

    // Check for expired credentials
    pub fn check_expired_credentials(
        env: Env,
        provider: Address,
    ) -> Result<Vec<BytesN<32>>, Error> {
        let credentials = Self::get_provider_credentials(env.clone(), provider.clone())?;
        let current_time = env.ledger().timestamp();
        let mut expired = Vec::new(&env);

        for credential in credentials.iter() {
            if credential.expiration_date < current_time && credential.is_active {
                expired.push_back(credential.credential_id);
            }
        }

        Ok(expired)
    }

    // Helper functions
    fn require_initialized(env: &Env) -> Result<(), Error> {
        if env.storage().instance().has(&DataKey::Initialized) {
            Ok(())
        } else {
            Err(Error::NotInitialized)
        }
    }

    fn require_admin(env: &Env, caller: &Address) -> Result<(), Error> {
        let admin: Address = env
            .storage()
            .instance()
            .get(&DataKey::Admin)
            .ok_or(Error::NotInitialized)?;
        if admin == *caller {
            Ok(())
        } else {
            Err(Error::NotAuthorized)
        }
    }
}
