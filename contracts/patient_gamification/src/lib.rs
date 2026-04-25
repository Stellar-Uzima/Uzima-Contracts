#![no_std]
#![allow(clippy::too_many_arguments)]

use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype, symbol_short, Address,
    Bytes, BytesN, Env, String, Vec,
};
use soroban_sdk::xdr::ToXdr;

const MIN_RANDOMNESS_LEDGER_DELAY: u32 = 2;
const RANDOMNESS_REVEAL_WINDOW: u32 = 20;

// ============================================================================
// DATA STRUCTURES
// ============================================================================

#[derive(Clone)]
#[contracttype]
pub struct GamificationConfig {
    pub admin: Address,
    pub points_per_achievement: u32,
    pub points_per_challenge: u32,
    pub points_per_streak_day: u32,
    pub max_daily_points: u32,
    pub privacy_threshold: u32,
    pub enabled: bool,
}

#[derive(Clone)]
#[contracttype]
pub struct Achievement {
    pub id: u64,
    pub name: String,
    pub description: String,
    pub category: String,
    pub points_reward: u32,
    pub badge_uri: String,
    pub requirement_type: String,
    pub requirement_value: u32,
    pub is_active: bool,
    pub created_at: u64,
}

#[derive(Clone)]
#[contracttype]
pub struct PatientAchievement {
    pub patient_id: Address,
    pub achievement_id: u64,
    pub earned_at: u64,
    pub progress: u32,
    pub is_completed: bool,
}

#[derive(Clone)]
#[contracttype]
pub struct HealthChallenge {
    pub id: u64,
    pub name: String,
    pub description: String,
    pub challenge_type: String,
    pub metric_name: String,
    pub target_value: u32,
    pub start_time: u64,
    pub end_time: u64,
    pub points_reward: u32,
    pub max_participants: u32,
    pub current_participants: u32,
    pub is_active: bool,
    pub created_at: u64,
}

#[derive(Clone)]
#[contracttype]
pub struct ChallengeParticipant {
    pub patient_id: Address,
    pub challenge_id: u64,
    pub current_value: u32,
    pub joined_at: u64,
    pub completed_at: Option<u64>,
    pub is_completed: bool,
}

#[derive(Clone)]
#[contracttype]
pub struct RewardPoints {
    pub patient_id: Address,
    pub total_points: u64,
    pub available_points: u64,
    pub lifetime_points: u64,
    pub last_updated: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub struct RandomBonusCommitment {
    pub patient_id: Address,
    pub reveal_hash: BytesN<32>,
    pub target_ledger: u32,
    pub expires_at_ledger: u32,
    pub max_bonus_points: u32,
    pub committed_at: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub struct RandomBonusOutcome {
    pub patient_id: Address,
    pub random_value: u64,
    pub bonus_points: u32,
    pub target_ledger: u32,
    pub revealed_at: u64,
}

#[derive(Clone)]
#[contracttype]
pub struct LeaderboardEntry {
    pub patient_id: Address,
    pub display_name: String,
    pub points: u64,
    pub achievements_count: u32,
    pub challenges_completed: u32,
    pub rank: u32,
    pub last_updated: u64,
}

#[derive(Clone)]
#[contracttype]
pub struct SocialProfile {
    pub patient_id: Address,
    pub display_name: String,
    pub bio: String,
    pub avatar_uri: String,
    pub is_public: bool,
    pub show_achievements: bool,
    pub show_challenges: bool,
    pub show_points: bool,
    pub created_at: u64,
    pub last_active: u64,
}

#[derive(Clone)]
#[contracttype]
pub struct HealthMetric {
    pub patient_id: Address,
    pub metric_name: String,
    pub value: u32,
    pub unit: String,
    pub recorded_at: u64,
    pub source: String,
}

#[derive(Clone)]
#[contracttype]
pub struct DailyStreak {
    pub patient_id: Address,
    pub current_streak: u32,
    pub longest_streak: u32,
    pub last_activity_date: u64,
    pub total_active_days: u32,
}

#[derive(Clone)]
#[contracttype]
pub enum DataKey {
    Config,
    AchievementCounter,
    Achievement(u64),
    PatientAchievement(Address, u64),
    PatientAchievements(Address),
    ChallengeCounter,
    Challenge(u64),
    ChallengeParticipant(u64, Address),
    ChallengeParticipants(u64),
    RewardPoints(Address),
    RandomBonusCommitment(Address),
    Leaderboard,
    SocialProfile(Address),
    HealthMetric(Address, String, u64),
    HealthMetrics(Address, String),
    DailyStreak(Address),
    Admin(Address),
}

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    NotAuthorized = 1,
    AlreadyInitialized = 2,
    NotInitialized = 3,
    InvalidInput = 4,
    AchievementNotFound = 5,
    ChallengeNotFound = 6,
    ChallengeFull = 7,
    ChallengeEnded = 8,
    AlreadyParticipating = 9,
    NotParticipating = 10,
    InsufficientPoints = 11,
    PrivacyThresholdNotMet = 12,
    InvalidTimeRange = 13,
    AlreadyCompleted = 14,
    RandomnessAlreadyCommitted = 15,
    RandomnessCommitNotFound = 16,
    RandomnessRevealTooEarly = 17,
    RandomnessRevealMismatch = 18,
    RandomnessCommitExpired = 19,
}

// ============================================================================
// CONTRACT IMPLEMENTATION
// ============================================================================

#[contract]
pub struct PatientGamificationContract;

#[contractimpl]
impl PatientGamificationContract {
    // -------------------------------------------------------------------------
    // INITIALIZATION
    // -------------------------------------------------------------------------

    pub fn initialize(
        env: Env,
        admin: Address,
        points_per_achievement: u32,
        points_per_challenge: u32,
        points_per_streak_day: u32,
        max_daily_points: u32,
        privacy_threshold: u32,
    ) -> Result<bool, Error> {
        admin.require_auth();

        if env.storage().instance().has(&DataKey::Config) {
            return Err(Error::AlreadyInitialized);
        }

        if privacy_threshold == 0 || max_daily_points == 0 {
            return Err(Error::InvalidInput);
        }

        let config = GamificationConfig {
            admin: admin.clone(),
            points_per_achievement,
            points_per_challenge,
            points_per_streak_day,
            max_daily_points,
            privacy_threshold,
            enabled: true,
        };

        env.storage().instance().set(&DataKey::Config, &config);
        env.storage()
            .instance()
            .set(&DataKey::AchievementCounter, &0u64);
        env.storage()
            .instance()
            .set(&DataKey::ChallengeCounter, &0u64);
        env.storage()
            .instance()
            .set(&DataKey::Leaderboard, &Vec::<LeaderboardEntry>::new(&env));

        env.events().publish((symbol_short!("GamInit"),), true);
        Ok(true)
    }

    fn load_config(env: &Env) -> Result<GamificationConfig, Error> {
        env.storage()
            .instance()
            .get(&DataKey::Config)
            .ok_or(Error::NotInitialized)
    }

    fn ensure_admin(env: &Env, caller: &Address) -> Result<GamificationConfig, Error> {
        let config = Self::load_config(env)?;
        if config.admin != *caller {
            return Err(Error::NotAuthorized);
        }
        Ok(config)
    }

    fn next_counter(env: &Env, key: &DataKey) -> u64 {
        let current: u64 = env.storage().instance().get(key).unwrap_or(0);
        let next = current.saturating_add(1);
        env.storage().instance().set(key, &next);
        next
    }

    // -------------------------------------------------------------------------
    // ACHIEVEMENT AND BADGE SYSTEM
    // -------------------------------------------------------------------------

    pub fn create_achievement(
        env: Env,
        caller: Address,
        name: String,
        description: String,
        category: String,
        points_reward: u32,
        badge_uri: String,
        requirement_type: String,
        requirement_value: u32,
    ) -> Result<u64, Error> {
        caller.require_auth();
        Self::ensure_admin(&env, &caller)?;

        if name.is_empty() || requirement_value == 0 {
            return Err(Error::InvalidInput);
        }

        let achievement_id = Self::next_counter(&env, &DataKey::AchievementCounter);
        let timestamp = env.ledger().timestamp();

        let achievement = Achievement {
            id: achievement_id,
            name: name.clone(),
            description,
            category,
            points_reward,
            badge_uri,
            requirement_type,
            requirement_value,
            is_active: true,
            created_at: timestamp,
        };

        env.storage()
            .instance()
            .set(&DataKey::Achievement(achievement_id), &achievement);

        env.events().publish(
            (symbol_short!("AchCreate"),),
            (achievement_id, name, points_reward),
        );
        Ok(achievement_id)
    }

    pub fn get_achievement(env: Env, achievement_id: u64) -> Result<Achievement, Error> {
        env.storage()
            .instance()
            .get(&DataKey::Achievement(achievement_id))
            .ok_or(Error::AchievementNotFound)
    }

    pub fn update_achievement_progress(
        env: Env,
        caller: Address,
        patient_id: Address,
        achievement_id: u64,
        progress: u32,
    ) -> Result<bool, Error> {
        caller.require_auth();

        let achievement: Achievement = env
            .storage()
            .instance()
            .get(&DataKey::Achievement(achievement_id))
            .ok_or(Error::AchievementNotFound)?;

        if !achievement.is_active {
            return Err(Error::InvalidInput);
        }

        let key = DataKey::PatientAchievement(patient_id.clone(), achievement_id);
        let mut patient_achievement: PatientAchievement = env
            .storage()
            .instance()
            .get(&key)
            .unwrap_or(PatientAchievement {
                patient_id: patient_id.clone(),
                achievement_id,
                earned_at: 0,
                progress: 0,
                is_completed: false,
            });

        if patient_achievement.is_completed {
            return Err(Error::AlreadyCompleted);
        }

        patient_achievement.progress = progress;
        let timestamp = env.ledger().timestamp();

        if progress >= achievement.requirement_value {
            patient_achievement.is_completed = true;
            patient_achievement.earned_at = timestamp;

            // Award points
            Self::award_points_internal(&env, &patient_id, achievement.points_reward)?;

            // Update achievements list
            let mut achievements: Vec<u64> = env
                .storage()
                .instance()
                .get(&DataKey::PatientAchievements(patient_id.clone()))
                .unwrap_or(Vec::new(&env));
            achievements.push_back(achievement_id);
            env.storage()
                .instance()
                .set(&DataKey::PatientAchievements(patient_id.clone()), &achievements);

            env.events().publish(
                (symbol_short!("AchEarn"),),
                (patient_id, achievement_id, achievement.points_reward),
            );
        }

        env.storage().instance().set(&key, &patient_achievement);
        Ok(patient_achievement.is_completed)
    }

    pub fn get_patient_achievement(
        env: Env,
        patient_id: Address,
        achievement_id: u64,
    ) -> Result<PatientAchievement, Error> {
        env.storage()
            .instance()
            .get(&DataKey::PatientAchievement(patient_id, achievement_id))
            .ok_or(Error::AchievementNotFound)
    }

    pub fn get_patient_achievements(env: Env, patient_id: Address) -> Result<Vec<u64>, Error> {
        Ok(env
            .storage()
            .instance()
            .get(&DataKey::PatientAchievements(patient_id))
            .unwrap_or(Vec::new(&env)))
    }

    // -------------------------------------------------------------------------
    // HEALTH CHALLENGES AND COMPETITIONS
    // -------------------------------------------------------------------------

    pub fn create_challenge(
        env: Env,
        caller: Address,
        name: String,
        description: String,
        challenge_type: String,
        metric_name: String,
        target_value: u32,
        start_time: u64,
        end_time: u64,
        points_reward: u32,
        max_participants: u32,
    ) -> Result<u64, Error> {
        caller.require_auth();
        Self::ensure_admin(&env, &caller)?;

        if name.is_empty() || target_value == 0 || start_time >= end_time {
            return Err(Error::InvalidInput);
        }

        let challenge_id = Self::next_counter(&env, &DataKey::ChallengeCounter);
        let timestamp = env.ledger().timestamp();

        let challenge = HealthChallenge {
            id: challenge_id,
            name: name.clone(),
            description,
            challenge_type,
            metric_name,
            target_value,
            start_time,
            end_time,
            points_reward,
            max_participants,
            current_participants: 0,
            is_active: true,
            created_at: timestamp,
        };

        env.storage()
            .instance()
            .set(&DataKey::Challenge(challenge_id), &challenge);

        env.events().publish(
            (symbol_short!("ChalCrt"),),
            (challenge_id, name, points_reward),
        );
        Ok(challenge_id)
    }

    pub fn get_challenge(env: Env, challenge_id: u64) -> Result<HealthChallenge, Error> {
        env.storage()
            .instance()
            .get(&DataKey::Challenge(challenge_id))
            .ok_or(Error::ChallengeNotFound)
    }

    pub fn join_challenge(
        env: Env,
        patient_id: Address,
        challenge_id: u64,
    ) -> Result<bool, Error> {
        patient_id.require_auth();

        let mut challenge: HealthChallenge = env
            .storage()
            .instance()
            .get(&DataKey::Challenge(challenge_id))
            .ok_or(Error::ChallengeNotFound)?;

        let timestamp = env.ledger().timestamp();

        if !challenge.is_active || timestamp > challenge.end_time {
            return Err(Error::ChallengeEnded);
        }

        if challenge.current_participants >= challenge.max_participants {
            return Err(Error::ChallengeFull);
        }

        let key = DataKey::ChallengeParticipant(challenge_id, patient_id.clone());
        if env.storage().instance().has(&key) {
            return Err(Error::AlreadyParticipating);
        }

        let participant = ChallengeParticipant {
            patient_id: patient_id.clone(),
            challenge_id,
            current_value: 0,
            joined_at: timestamp,
            completed_at: None,
            is_completed: false,
        };

        env.storage().instance().set(&key, &participant);

        challenge.current_participants = challenge.current_participants.saturating_add(1);
        env.storage()
            .instance()
            .set(&DataKey::Challenge(challenge_id), &challenge);

        // Update participants list
        let mut participants: Vec<Address> = env
            .storage()
            .instance()
            .get(&DataKey::ChallengeParticipants(challenge_id))
            .unwrap_or(Vec::new(&env));
        participants.push_back(patient_id.clone());
        env.storage()
            .instance()
            .set(&DataKey::ChallengeParticipants(challenge_id), &participants);

        env.events().publish(
            (symbol_short!("ChalJoin"),),
            (patient_id, challenge_id),
        );
        Ok(true)
    }

    pub fn update_challenge_progress(
        env: Env,
        caller: Address,
        patient_id: Address,
        challenge_id: u64,
        value: u32,
    ) -> Result<bool, Error> {
        caller.require_auth();

        let challenge: HealthChallenge = env
            .storage()
            .instance()
            .get(&DataKey::Challenge(challenge_id))
            .ok_or(Error::ChallengeNotFound)?;

        let timestamp = env.ledger().timestamp();

        if !challenge.is_active || timestamp > challenge.end_time {
            return Err(Error::ChallengeEnded);
        }

        let key = DataKey::ChallengeParticipant(challenge_id, patient_id.clone());
        let mut participant: ChallengeParticipant = env
            .storage()
            .instance()
            .get(&key)
            .ok_or(Error::NotParticipating)?;

        if participant.is_completed {
            return Err(Error::AlreadyCompleted);
        }

        participant.current_value = value;

        if value >= challenge.target_value {
            participant.is_completed = true;
            participant.completed_at = Some(timestamp);

            // Award points
            Self::award_points_internal(&env, &patient_id, challenge.points_reward)?;

            env.events().publish(
                (symbol_short!("ChalComp"),),
                (patient_id, challenge_id, challenge.points_reward),
            );
        }

        env.storage().instance().set(&key, &participant);
        Ok(participant.is_completed)
    }

    pub fn get_challenge_participant(
        env: Env,
        challenge_id: u64,
        patient_id: Address,
    ) -> Result<ChallengeParticipant, Error> {
        env.storage()
            .instance()
            .get(&DataKey::ChallengeParticipant(challenge_id, patient_id))
            .ok_or(Error::NotParticipating)
    }

    pub fn get_challenge_participants(
        env: Env,
        challenge_id: u64,
    ) -> Result<Vec<Address>, Error> {
        Ok(env
            .storage()
            .instance()
            .get(&DataKey::ChallengeParticipants(challenge_id))
            .unwrap_or(Vec::new(&env)))
    }

    // -------------------------------------------------------------------------
    // REWARD POINTS SYSTEM
    // -------------------------------------------------------------------------

    fn award_points_internal(
        env: &Env,
        patient_id: &Address,
        points: u32,
    ) -> Result<(), Error> {
        let _config = Self::load_config(env)?;
        let timestamp = env.ledger().timestamp();

        let mut reward_points: RewardPoints = env
            .storage()
            .instance()
            .get(&DataKey::RewardPoints(patient_id.clone()))
            .unwrap_or(RewardPoints {
                patient_id: patient_id.clone(),
                total_points: 0,
                available_points: 0,
                lifetime_points: 0,
                last_updated: timestamp,
            });

        let points_u64 = u64::from(points);
        reward_points.total_points = reward_points.total_points.saturating_add(points_u64);
        reward_points.available_points = reward_points.available_points.saturating_add(points_u64);
        reward_points.lifetime_points = reward_points.lifetime_points.saturating_add(points_u64);
        reward_points.last_updated = timestamp;

        env.storage()
            .instance()
            .set(&DataKey::RewardPoints(patient_id.clone()), &reward_points);

        // Update leaderboard
        Self::update_leaderboard_internal(env, patient_id, reward_points.total_points)?;

        Ok(())
    }

    pub fn get_reward_points(env: Env, patient_id: Address) -> Result<RewardPoints, Error> {
        Ok(env
            .storage()
            .instance()
            .get(&DataKey::RewardPoints(patient_id.clone()))
            .unwrap_or(RewardPoints {
                patient_id,
                total_points: 0,
                available_points: 0,
                lifetime_points: 0,
                last_updated: 0,
            }))
    }

    pub fn redeem_points(
        env: Env,
        patient_id: Address,
        points: u64,
    ) -> Result<bool, Error> {
        patient_id.require_auth();

        let mut reward_points: RewardPoints = env
            .storage()
            .instance()
            .get(&DataKey::RewardPoints(patient_id.clone()))
            .ok_or(Error::InsufficientPoints)?;

        if reward_points.available_points < points {
            return Err(Error::InsufficientPoints);
        }

        reward_points.available_points = reward_points.available_points.saturating_sub(points);
        reward_points.last_updated = env.ledger().timestamp();

        env.storage()
            .instance()
            .set(&DataKey::RewardPoints(patient_id.clone()), &reward_points);

        env.events().publish(
            (symbol_short!("PtsRedeem"),),
            (patient_id, points),
        );
        Ok(true)
    }

    pub fn commit_random_bonus(
        env: Env,
        patient_id: Address,
        reveal_hash: BytesN<32>,
        target_ledger: u32,
        max_bonus_points: u32,
    ) -> Result<bool, Error> {
        patient_id.require_auth();

        if max_bonus_points == 0 {
            return Err(Error::InvalidInput);
        }

        let current_ledger = env.ledger().sequence();
        if target_ledger < current_ledger.saturating_add(MIN_RANDOMNESS_LEDGER_DELAY) {
            return Err(Error::InvalidInput);
        }

        let key = DataKey::RandomBonusCommitment(patient_id.clone());
        if let Some(existing) = env
            .storage()
            .instance()
            .get::<DataKey, RandomBonusCommitment>(&key)
        {
            if current_ledger <= existing.expires_at_ledger {
                return Err(Error::RandomnessAlreadyCommitted);
            }
        }

        let commitment = RandomBonusCommitment {
            patient_id: patient_id.clone(),
            reveal_hash,
            target_ledger,
            expires_at_ledger: target_ledger.saturating_add(RANDOMNESS_REVEAL_WINDOW),
            max_bonus_points,
            committed_at: env.ledger().timestamp(),
        };

        env.storage().instance().set(&key, &commitment);
        env.events().publish(
            (symbol_short!("RndCmt"),),
            (patient_id, target_ledger, max_bonus_points),
        );

        Ok(true)
    }

    pub fn reveal_random_bonus(
        env: Env,
        patient_id: Address,
        reveal: BytesN<32>,
    ) -> Result<RandomBonusOutcome, Error> {
        patient_id.require_auth();

        let key = DataKey::RandomBonusCommitment(patient_id.clone());
        let commitment: RandomBonusCommitment = env
            .storage()
            .instance()
            .get(&key)
            .ok_or(Error::RandomnessCommitNotFound)?;

        let current_ledger = env.ledger().sequence();
        if current_ledger < commitment.target_ledger {
            return Err(Error::RandomnessRevealTooEarly);
        }

        if current_ledger > commitment.expires_at_ledger {
            env.storage().instance().remove(&key);
            return Err(Error::RandomnessCommitExpired);
        }

        let expected_reveal_hash = Self::hash_random_bonus_reveal(&env, &patient_id, &reveal);
        if expected_reveal_hash != commitment.reveal_hash {
            return Err(Error::RandomnessRevealMismatch);
        }

        let random_value = Self::derive_random_bonus_value(&env, &commitment, &reveal);
        let bonus_points = (random_value % u64::from(commitment.max_bonus_points))
            .saturating_add(1) as u32;

        Self::award_points_internal(&env, &patient_id, bonus_points)?;
        env.storage().instance().remove(&key);

        let outcome = RandomBonusOutcome {
            patient_id: patient_id.clone(),
            random_value,
            bonus_points,
            target_ledger: commitment.target_ledger,
            revealed_at: env.ledger().timestamp(),
        };

        env.events().publish(
            (symbol_short!("RndRvl"),),
            (patient_id, bonus_points, commitment.target_ledger),
        );

        Ok(outcome)
    }

    pub fn get_random_bonus_commitment(
        env: Env,
        patient_id: Address,
    ) -> Result<RandomBonusCommitment, Error> {
        env.storage()
            .instance()
            .get(&DataKey::RandomBonusCommitment(patient_id))
            .ok_or(Error::RandomnessCommitNotFound)
    }

    // -------------------------------------------------------------------------
    // SOCIAL FEATURES AND LEADERBOARDS
    // -------------------------------------------------------------------------

    pub fn create_social_profile(
        env: Env,
        patient_id: Address,
        display_name: String,
        bio: String,
        avatar_uri: String,
        is_public: bool,
        show_achievements: bool,
        show_challenges: bool,
        show_points: bool,
    ) -> Result<bool, Error> {
        patient_id.require_auth();

        if display_name.is_empty() {
            return Err(Error::InvalidInput);
        }

        let timestamp = env.ledger().timestamp();

        let profile = SocialProfile {
            patient_id: patient_id.clone(),
            display_name,
            bio,
            avatar_uri,
            is_public,
            show_achievements,
            show_challenges,
            show_points,
            created_at: timestamp,
            last_active: timestamp,
        };

        env.storage()
            .instance()
            .set(&DataKey::SocialProfile(patient_id.clone()), &profile);

        env.events().publish(
            (symbol_short!("ProfCrt"),),
            patient_id,
        );
        Ok(true)
    }

    pub fn get_social_profile(env: Env, patient_id: Address) -> Result<SocialProfile, Error> {
        env.storage()
            .instance()
            .get(&DataKey::SocialProfile(patient_id))
            .ok_or(Error::NotInitialized)
    }

    pub fn update_social_profile(
        env: Env,
        patient_id: Address,
        display_name: String,
        bio: String,
        avatar_uri: String,
        is_public: bool,
        show_achievements: bool,
        show_challenges: bool,
        show_points: bool,
    ) -> Result<bool, Error> {
        patient_id.require_auth();

        let mut profile: SocialProfile = env
            .storage()
            .instance()
            .get(&DataKey::SocialProfile(patient_id.clone()))
            .ok_or(Error::NotInitialized)?;

        if !display_name.is_empty() {
            profile.display_name = display_name;
        }
        profile.bio = bio;
        profile.avatar_uri = avatar_uri;
        profile.is_public = is_public;
        profile.show_achievements = show_achievements;
        profile.show_challenges = show_challenges;
        profile.show_points = show_points;
        profile.last_active = env.ledger().timestamp();

        env.storage()
            .instance()
            .set(&DataKey::SocialProfile(patient_id), &profile);

        Ok(true)
    }

    fn update_leaderboard_internal(
        env: &Env,
        patient_id: &Address,
        points: u64,
    ) -> Result<(), Error> {
        let leaderboard: Vec<LeaderboardEntry> = env
            .storage()
            .instance()
            .get(&DataKey::Leaderboard)
            .unwrap_or(Vec::new(env));

        let timestamp = env.ledger().timestamp();

        // Find existing entry or create new one
        let mut found = false;
        let mut new_leaderboard = Vec::new(env);

        for entry in leaderboard.iter() {
            if entry.patient_id == *patient_id {
                found = true;
                let updated_entry = LeaderboardEntry {
                    patient_id: patient_id.clone(),
                    display_name: entry.display_name.clone(),
                    points,
                    achievements_count: entry.achievements_count,
                    challenges_completed: entry.challenges_completed,
                    rank: 0,
                    last_updated: timestamp,
                };
                new_leaderboard.push_back(updated_entry);
            } else {
                new_leaderboard.push_back(entry.clone());
            }
        }

        if !found {
            let profile = env
                .storage()
                .instance()
                .get(&DataKey::SocialProfile(patient_id.clone()))
                .unwrap_or(SocialProfile {
                    patient_id: patient_id.clone(),
                    display_name: String::from_str(env, "Anonymous"),
                    bio: String::from_str(env, ""),
                    avatar_uri: String::from_str(env, ""),
                    is_public: false,
                    show_achievements: true,
                    show_challenges: true,
                    show_points: true,
                    created_at: timestamp,
                    last_active: timestamp,
                });

            let achievements: Vec<u64> = env
                .storage()
                .instance()
                .get(&DataKey::PatientAchievements(patient_id.clone()))
                .unwrap_or(Vec::new(env));

            let entry = LeaderboardEntry {
                patient_id: patient_id.clone(),
                display_name: profile.display_name,
                points,
                achievements_count: achievements.len() as u32,
                challenges_completed: 0,
                rank: 0,
                last_updated: timestamp,
            };
            new_leaderboard.push_back(entry);
        }

        // Sort by points (descending)
        let mut sorted_leaderboard = Vec::new(env);
        let mut temp_vec: Vec<LeaderboardEntry> = Vec::new(env);
        for entry in new_leaderboard.iter() {
            temp_vec.push_back(entry.clone());
        }

        // Simple bubble sort for on-chain sorting
        let len = temp_vec.len();
        for i in 0..len {
            for j in 0..len.saturating_sub(i).saturating_sub(1) {
                if j + 1 < len {
                    let entry_j = temp_vec.get(j).unwrap();
                    let entry_j1 = temp_vec.get(j + 1).unwrap();
                    if entry_j.points < entry_j1.points {
                        let temp = entry_j.clone();
                        temp_vec.set(j, entry_j1.clone());
                        temp_vec.set(j + 1, temp);
                    }
                }
            }
        }

        // Assign ranks
        let mut rank = 1u32;
        for entry in temp_vec.iter() {
            let ranked_entry = LeaderboardEntry {
                patient_id: entry.patient_id.clone(),
                display_name: entry.display_name.clone(),
                points: entry.points,
                achievements_count: entry.achievements_count,
                challenges_completed: entry.challenges_completed,
                rank,
                last_updated: entry.last_updated,
            };
            sorted_leaderboard.push_back(ranked_entry);
            rank = rank.saturating_add(1);
        }

        env.storage()
            .instance()
            .set(&DataKey::Leaderboard, &sorted_leaderboard);

        Ok(())
    }

    pub fn get_leaderboard(env: Env, limit: u32) -> Result<Vec<LeaderboardEntry>, Error> {
        let leaderboard: Vec<LeaderboardEntry> = env
            .storage()
            .instance()
            .get(&DataKey::Leaderboard)
            .unwrap_or(Vec::new(&env));

        let mut result = Vec::new(&env);
        let mut count = 0u32;

        for entry in leaderboard.iter() {
            if count >= limit {
                break;
            }
            result.push_back(entry.clone());
            count = count.saturating_add(1);
        }

        Ok(result)
    }

    pub fn get_patient_rank(env: Env, patient_id: Address) -> Result<u32, Error> {
        let leaderboard: Vec<LeaderboardEntry> = env
            .storage()
            .instance()
            .get(&DataKey::Leaderboard)
            .unwrap_or(Vec::new(&env));

        for entry in leaderboard.iter() {
            if entry.patient_id == patient_id {
                return Ok(entry.rank);
            }
        }

        Ok(0)
    }

    // -------------------------------------------------------------------------
    // HEALTH METRICS TRACKING
    // -------------------------------------------------------------------------

    pub fn record_health_metric(
        env: Env,
        caller: Address,
        patient_id: Address,
        metric_name: String,
        value: u32,
        unit: String,
        source: String,
    ) -> Result<bool, Error> {
        caller.require_auth();

        if metric_name.is_empty() {
            return Err(Error::InvalidInput);
        }

        let timestamp = env.ledger().timestamp();

        let metric = HealthMetric {
            patient_id: patient_id.clone(),
            metric_name: metric_name.clone(),
            value,
            unit,
            recorded_at: timestamp,
            source,
        };

        // Store metric with timestamp as key for historical tracking
        let key = DataKey::HealthMetric(patient_id.clone(), metric_name.clone(), timestamp);
        env.storage().instance().set(&key, &metric);

        // Update metrics list
        let mut metrics: Vec<u64> = env
            .storage()
            .instance()
            .get(&DataKey::HealthMetrics(patient_id.clone(), metric_name.clone()))
            .unwrap_or(Vec::new(&env));
        metrics.push_back(timestamp);
        env.storage()
            .instance()
            .set(&DataKey::HealthMetrics(patient_id.clone(), metric_name.clone()), &metrics);

        // Update daily streak
        Self::update_daily_streak_internal(&env, &patient_id)?;

        env.events().publish(
            (symbol_short!("MetricRec"),),
            (patient_id, metric_name, value),
        );
        Ok(true)
    }

    pub fn get_health_metric(
        env: Env,
        patient_id: Address,
        metric_name: String,
        timestamp: u64,
    ) -> Result<HealthMetric, Error> {
        env.storage()
            .instance()
            .get(&DataKey::HealthMetric(patient_id, metric_name, timestamp))
            .ok_or(Error::InvalidInput)
    }

    pub fn get_health_metrics(
        env: Env,
        patient_id: Address,
        metric_name: String,
    ) -> Result<Vec<u64>, Error> {
        Ok(env
            .storage()
            .instance()
            .get(&DataKey::HealthMetrics(patient_id, metric_name))
            .unwrap_or(Vec::new(&env)))
    }

    // -------------------------------------------------------------------------
    // DAILY STREAKS
    // -------------------------------------------------------------------------

    fn update_daily_streak_internal(env: &Env, patient_id: &Address) -> Result<(), Error> {
        let timestamp = env.ledger().timestamp();
        let day_seconds = 86400u64;
        let current_day = timestamp / day_seconds;

        let mut streak: DailyStreak = env
            .storage()
            .instance()
            .get(&DataKey::DailyStreak(patient_id.clone()))
            .unwrap_or(DailyStreak {
                patient_id: patient_id.clone(),
                current_streak: 0,
                longest_streak: 0,
                last_activity_date: 0,
                total_active_days: 0,
            });

        let last_day = streak.last_activity_date / day_seconds;

        if current_day > last_day {
            if current_day == last_day + 1 {
                // Consecutive day
                streak.current_streak = streak.current_streak.saturating_add(1);
            } else if current_day > last_day + 1 {
                // Streak broken
                streak.current_streak = 1;
            }
            // If same day, don't update streak

            if current_day != last_day {
                streak.total_active_days = streak.total_active_days.saturating_add(1);
                streak.last_activity_date = timestamp;

                if streak.current_streak > streak.longest_streak {
                    streak.longest_streak = streak.current_streak;
                }

                // Award streak points
                let config = Self::load_config(env)?;
                let streak_points = u32::from(streak.current_streak)
                    .saturating_mul(config.points_per_streak_day)
                    .min(config.max_daily_points);

                if streak_points > 0 {
                    Self::award_points_internal(env, patient_id, streak_points)?;
                }
            }
        }

        env.storage()
            .instance()
            .set(&DataKey::DailyStreak(patient_id.clone()), &streak);

        Ok(())
    }

    pub fn get_daily_streak(env: Env, patient_id: Address) -> Result<DailyStreak, Error> {
        Ok(env
            .storage()
            .instance()
            .get(&DataKey::DailyStreak(patient_id.clone()))
            .unwrap_or(DailyStreak {
                patient_id,
                current_streak: 0,
                longest_streak: 0,
                last_activity_date: 0,
                total_active_days: 0,
            }))
    }

    // -------------------------------------------------------------------------
    // ADMIN FUNCTIONS
    // -------------------------------------------------------------------------

    pub fn update_config(
        env: Env,
        caller: Address,
        points_per_achievement: u32,
        points_per_challenge: u32,
        points_per_streak_day: u32,
        max_daily_points: u32,
        privacy_threshold: u32,
        enabled: bool,
    ) -> Result<bool, Error> {
        caller.require_auth();
        let mut config = Self::ensure_admin(&env, &caller)?;

        if privacy_threshold == 0 || max_daily_points == 0 {
            return Err(Error::InvalidInput);
        }

        config.points_per_achievement = points_per_achievement;
        config.points_per_challenge = points_per_challenge;
        config.points_per_streak_day = points_per_streak_day;
        config.max_daily_points = max_daily_points;
        config.privacy_threshold = privacy_threshold;
        config.enabled = enabled;

        env.storage().instance().set(&DataKey::Config, &config);

        env.events().publish(
            (symbol_short!("ConfigUpd"),),
            enabled,
        );
        Ok(true)
    }

    pub fn deactivate_achievement(
        env: Env,
        caller: Address,
        achievement_id: u64,
    ) -> Result<bool, Error> {
        caller.require_auth();
        Self::ensure_admin(&env, &caller)?;

        let mut achievement: Achievement = env
            .storage()
            .instance()
            .get(&DataKey::Achievement(achievement_id))
            .ok_or(Error::AchievementNotFound)?;

        achievement.is_active = false;
        env.storage()
            .instance()
            .set(&DataKey::Achievement(achievement_id), &achievement);

        Ok(true)
    }

    pub fn deactivate_challenge(
        env: Env,
        caller: Address,
        challenge_id: u64,
    ) -> Result<bool, Error> {
        caller.require_auth();
        Self::ensure_admin(&env, &caller)?;

        let mut challenge: HealthChallenge = env
            .storage()
            .instance()
            .get(&DataKey::Challenge(challenge_id))
            .ok_or(Error::ChallengeNotFound)?;

        challenge.is_active = false;
        env.storage()
            .instance()
            .set(&DataKey::Challenge(challenge_id), &challenge);

        Ok(true)
    }

    // -------------------------------------------------------------------------
    // VIEW FUNCTIONS
    // -------------------------------------------------------------------------

    pub fn get_config(env: Env) -> Result<GamificationConfig, Error> {
        Self::load_config(&env)
    }

    pub fn get_total_achievements(env: Env) -> Result<u64, Error> {
        Ok(env
            .storage()
            .instance()
            .get(&DataKey::AchievementCounter)
            .unwrap_or(0))
    }

    pub fn get_total_challenges(env: Env) -> Result<u64, Error> {
        Ok(env
            .storage()
            .instance()
            .get(&DataKey::ChallengeCounter)
            .unwrap_or(0))
    }

    fn hash_random_bonus_reveal(
        env: &Env,
        patient_id: &Address,
        reveal: &BytesN<32>,
    ) -> BytesN<32> {
        let mut payload = patient_id.clone().to_xdr(env);
        Self::append_bytes32(env, &mut payload, reveal);
        env.crypto().sha256(&payload).into()
    }

    fn derive_random_bonus_value(
        env: &Env,
        commitment: &RandomBonusCommitment,
        reveal: &BytesN<32>,
    ) -> u64 {
        let mut payload = commitment.patient_id.clone().to_xdr(env);
        payload.append(&commitment.target_ledger.to_xdr(env));
        payload.append(&commitment.max_bonus_points.to_xdr(env));
        payload.append(&env.current_contract_address().to_xdr(env));
        Self::append_bytes32(env, &mut payload, &commitment.reveal_hash);
        Self::append_bytes32(env, &mut payload, reveal);

        let digest: BytesN<32> = env.crypto().sha256(&payload).into();
        let digest_bytes = digest.to_array();
        let mut prefix = [0u8; 8];
        prefix.copy_from_slice(&digest_bytes[..8]);
        u64::from_be_bytes(prefix)
    }

    fn append_bytes32(env: &Env, payload: &mut Bytes, value: &BytesN<32>) {
        payload.append(&Bytes::from_slice(env, &value.to_array()));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use soroban_sdk::{
        testutils::{Address as _, Ledger},
        Address, BytesN, Env,
    };

    #[test]
    fn random_bonus_commit_and_reveal() {
        let env = Env::default();
        env.mock_all_auths();
        env.ledger().with_mut(|l| {
            l.timestamp = 1_000;
            l.sequence_number = 10;
        });

        let contract_id = env.register_contract(None, PatientGamificationContract);
        let client = PatientGamificationContractClient::new(&env, &contract_id);

        let admin = Address::generate(&env);
        let patient = Address::generate(&env);

        assert!(client.initialize(&admin, &100u32, &50u32, &10u32, &1000u32, &5u32));

        let reveal = BytesN::from_array(&env, &[7u8; 32]);
        let reveal_hash =
            PatientGamificationContract::hash_random_bonus_reveal(&env, &patient, &reveal);

        assert!(client.commit_random_bonus(&patient, &reveal_hash, &12u32, &25u32));

        let pending = client.get_random_bonus_commitment(&patient);
        assert_eq!(pending.target_ledger, 12);
        assert_eq!(pending.max_bonus_points, 25);

        let early = client.try_reveal_random_bonus(&patient, &reveal);
        assert_eq!(early, Err(Ok(Error::RandomnessRevealTooEarly)));

        env.ledger().with_mut(|l| {
            l.timestamp = 1_200;
            l.sequence_number = 12;
        });

        let outcome = client.reveal_random_bonus(&patient, &reveal);
        assert_eq!(outcome.patient_id, patient);
        assert_eq!(outcome.target_ledger, 12);
        assert!(outcome.bonus_points >= 1);
        assert!(outcome.bonus_points <= 25);

        let points = client.get_reward_points(&patient);
        assert_eq!(points.total_points, u64::from(outcome.bonus_points));
        assert_eq!(points.available_points, u64::from(outcome.bonus_points));

        let missing = client.try_get_random_bonus_commitment(&patient);
        assert_eq!(missing, Err(Ok(Error::RandomnessCommitNotFound)));
    }

    #[test]
    fn random_bonus_rejects_mismatched_reveal_and_duplicate_commit() {
        let env = Env::default();
        env.mock_all_auths();
        env.ledger().with_mut(|l| {
            l.timestamp = 2_000;
            l.sequence_number = 20;
        });

        let contract_id = env.register_contract(None, PatientGamificationContract);
        let client = PatientGamificationContractClient::new(&env, &contract_id);

        let admin = Address::generate(&env);
        let patient = Address::generate(&env);

        assert!(client.initialize(&admin, &100u32, &50u32, &10u32, &1000u32, &5u32));

        let reveal = BytesN::from_array(&env, &[9u8; 32]);
        let reveal_hash =
            PatientGamificationContract::hash_random_bonus_reveal(&env, &patient, &reveal);

        assert!(client.commit_random_bonus(&patient, &reveal_hash, &22u32, &10u32));

        let duplicate = client.try_commit_random_bonus(&patient, &reveal_hash, &23u32, &10u32);
        assert_eq!(duplicate, Err(Ok(Error::RandomnessAlreadyCommitted)));

        env.ledger().with_mut(|l| {
            l.timestamp = 2_200;
            l.sequence_number = 22;
        });

        let wrong_reveal = BytesN::from_array(&env, &[10u8; 32]);
        let mismatch = client.try_reveal_random_bonus(&patient, &wrong_reveal);
        assert_eq!(mismatch, Err(Ok(Error::RandomnessRevealMismatch)));
    }

    #[test]
    fn random_bonus_commit_expires() {
        let env = Env::default();
        env.mock_all_auths();
        env.ledger().with_mut(|l| {
            l.timestamp = 3_000;
            l.sequence_number = 30;
        });

        let contract_id = env.register_contract(None, PatientGamificationContract);
        let client = PatientGamificationContractClient::new(&env, &contract_id);

        let admin = Address::generate(&env);
        let patient = Address::generate(&env);

        assert!(client.initialize(&admin, &100u32, &50u32, &10u32, &1000u32, &5u32));

        let reveal = BytesN::from_array(&env, &[11u8; 32]);
        let reveal_hash =
            PatientGamificationContract::hash_random_bonus_reveal(&env, &patient, &reveal);

        assert!(client.commit_random_bonus(&patient, &reveal_hash, &32u32, &15u32));

        env.ledger().with_mut(|l| {
            l.timestamp = 3_400;
            l.sequence_number = 53;
        });

        let expired = client.try_reveal_random_bonus(&patient, &reveal);
        assert_eq!(expired, Err(Ok(Error::RandomnessCommitExpired)));

        let next_reveal = BytesN::from_array(&env, &[12u8; 32]);
        let next_hash =
            PatientGamificationContract::hash_random_bonus_reveal(&env, &patient, &next_reveal);
        let recommit = client.commit_random_bonus(&patient, &next_hash, &56u32, &20u32);
        assert!(recommit);
    }
}
