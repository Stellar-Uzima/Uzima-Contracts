use soroban_sdk::{contracttype, Address, Env, String, Vec};
use crate::{types::*, errors::Error, events::*};

pub struct WellnessManager;

impl WellnessManager {
    pub fn create_wellness_program(
        env: &Env,
        name: String,
        description: String,
        category: WellnessCategory,
        duration_weeks: u32,
        modules: Vec<WellnessModule>,
    ) -> Result<u64, Error> {
        let program_id = env.ledger().timestamp() as u64;

        let program = WellnessProgram {
            program_id,
            name,
            description,
            category,
            duration_weeks,
            enrolled_users: Vec::new(env),
            modules,
            completion_rate: 0.0,
            effectiveness_score: 0.0,
        };

        env.storage().instance().set(&program_id, &program);

        env.events().publish(
            (Symbol::new(env, "wellness_program_created"),),
            program_id,
        );

        Ok(program_id)
    }

    pub fn enroll_in_program(
        env: &Env,
        program_id: u64,
        user_id: Address,
    ) -> Result<(), Error> {
        let mut program: WellnessProgram = env.storage().instance()
            .get(&program_id)
            .ok_or(Error::ProgramNotFound)?;

        // Check if already enrolled
        for enrolled_user in program.enrolled_users.iter() {
            if enrolled_user == user_id {
                return Err(Error::InvalidInput); // Already enrolled
            }
        }

        program.enrolled_users.push_back(user_id.clone());
        env.storage().instance().set(&program_id, &program);

        // Initialize user progress
        let progress = UserWellnessProgress {
            user_id: user_id.clone(),
            program_id,
            enrolled_date: env.ledger().timestamp(),
            completed_modules: Vec::new(env),
            current_streak: 0,
            total_sessions: 0,
            last_activity: env.ledger().timestamp(),
            progress_percentage: 0.0,
        };

        let progress_key = String::from_str(env, "user_program_progress");
        env.storage().instance().set(&progress_key, &progress);

        env.events().publish(
            (Symbol::new(env, "user_enrolled_program"),),
            (user_id, program_id),
        );

        Ok(())
    }

    pub fn complete_module(
        env: &Env,
        program_id: u64,
        user_id: Address,
        module_id: u64,
        session_duration: u32,
    ) -> Result<(), Error> {
        let progress_key = String::from_str(env, "user_program_progress");
        let mut progress: UserWellnessProgress = env.storage().instance()
            .get(&progress_key)
            .ok_or(Error::ProgramNotFound)?;

        // Check if module already completed
        for completed_module in progress.completed_modules.iter() {
            if *completed_module == module_id {
                return Err(Error::InvalidInput); // Already completed
            }
        }

        progress.completed_modules.push_back(module_id);
        progress.total_sessions += 1;
        progress.last_activity = env.ledger().timestamp();

        // Calculate progress percentage
        let program: WellnessProgram = env.storage().instance()
            .get(&program_id)
            .unwrap();
        progress.progress_percentage = progress.completed_modules.len() as f32 / program.modules.len() as f32 * 100.0;

        // Update streak (simplified - assumes daily activity)
        let days_since_last = (env.ledger().timestamp() - progress.last_activity) / (24 * 60 * 60);
        if days_since_last <= 1 {
            progress.current_streak += 1;
        } else {
            progress.current_streak = 1;
        }

        env.storage().instance().set(&key, &progress);

        env.events().publish(
            (Symbol::new(env, "module_completed"),),
            WellnessProgressEvent {
                user_id,
                program_id,
                progress_percentage: progress.progress_percentage,
                timestamp: env.ledger().timestamp(),
            },
        );

        Ok(())
    }

    pub fn get_user_progress(
        env: &Env,
        program_id: u64,
        user_id: Address,
    ) -> Result<UserWellnessProgress, Error> {
        let progress_key = String::from_str(env, "user_program_progress");
        env.storage().instance()
            .get(&progress_key)
            .ok_or(Error::ProgramNotFound)
    }

    pub fn get_available_programs(env: &Env, category: Option<WellnessCategory>) -> Vec<WellnessProgram> {
        // In a real implementation, this would query all programs
        // For now, return empty vec
        Vec::new(env)
    }

    pub fn submit_wellness_feedback(
        env: &Env,
        program_id: u64,
        user_id: Address,
        overall_rating: u32,
        helpful_modules: Vec<u64>,
        suggestions: String,
    ) -> Result<(), Error> {
        if overall_rating > 10 {
            return Err(Error::InvalidInput);
        }

        let feedback = WellnessFeedback {
            user_id,
            program_id,
            overall_rating,
            helpful_modules,
            suggestions,
            submitted_timestamp: env.ledger().timestamp(),
        };

        let feedback_key = String::from_str(env, "program_feedback");
        let mut feedbacks: Vec<WellnessFeedback> = env.storage().instance()
            .get(&feedback_key)
            .unwrap_or(Vec::new(env));
        feedbacks.push_back(feedback);
        env.storage().instance().set(&feedback_key, &feedbacks);

        // Update program effectiveness score
        Self::update_program_effectiveness(env, program_id);

        env.events().publish(
            (Symbol::new(env, "wellness_feedback_submitted"),),
            (user_id, program_id, overall_rating),
        );

        Ok(())
    }

    pub fn get_program_feedback(
        env: &Env,
        program_id: u64,
    ) -> Vec<WellnessFeedback> {
        let feedback_key = String::from_str(env, "program_feedback");
        env.storage().instance()
            .get(&feedback_key)
            .unwrap_or(Vec::new(env))
    }

    pub fn generate_personalized_recommendations(
        env: &Env,
        user_id: Address,
    ) -> Vec<WellnessRecommendation> {
        let mut recommendations = Vec::new(env);

        // Get user's mood history and current state
        // This is simplified - in reality, would analyze comprehensive data

        // Check recent mood entries
        let mood_entries: Vec<MoodEntry> = env.storage().instance()
            .get(&user_id)
            .unwrap_or(Vec::new(env));

        if !mood_entries.is_empty() {
            let recent_mood = mood_entries.get(mood_entries.len() - 1).unwrap();

            if recent_mood.mood_score < -2 {
                recommendations.push_back(WellnessRecommendation {
                    recommendation_type: String::from_str(env, "mindfulness"),
                    title: String::from_str(env, "Guided Meditation"),
                    description: String::from_str(env, "Daily 10-minute mindfulness meditation to reduce stress"),
                    priority: 9,
                });

                recommendations.push_back(WellnessRecommendation {
                    recommendation_type: String::from_str(env, "exercise"),
                    title: String::from_str(env, "Light Walking"),
                    description: String::from_str(env, "Gentle 20-minute walks in nature"),
                    priority: 8,
                });
            }

            // Check for sleep-related concerns
            let sleep_issues = recent_mood.triggers.iter()
                .any(|trigger| trigger.to_lowercase().contains("sleep") ||
                            trigger.to_lowercase().contains("tired"));

            if sleep_issues {
                recommendations.push_back(WellnessRecommendation {
                    recommendation_type: String::from_str(env, "sleep"),
                    title: String::from_str(env, "Sleep Hygiene Program"),
                    description: String::from_str(env, "Establish healthy sleep routines"),
                    priority: 10,
                });
            }
        }

        // Default recommendations if no specific data
        if recommendations.is_empty() {
            recommendations.push_back(WellnessRecommendation {
                recommendation_type: String::from_str(env, "general"),
                title: String::from_str(env, "Daily Wellness Check-in"),
                description: String::from_str(env, "Regular mood and wellness tracking"),
                priority: 5,
            });
        }

        recommendations
    }

    pub fn track_wellness_activity(
        env: &Env,
        user_id: Address,
        activity_type: String,
        duration_minutes: u32,
        notes: String,
    ) -> Result<(), Error> {
        let activity = WellnessActivity {
            user_id,
            activity_type,
            duration_minutes,
            timestamp: env.ledger().timestamp(),
            notes,
        };

        let activities_key = String::from_str(env, "user_activities");
        let mut activities: Vec<WellnessActivity> = env.storage().instance()
            .get(&activities_key)
            .unwrap_or(Vec::new(env));
        activities.push_back(activity);
        env.storage().instance().set(&activities_key, &activities);

        env.events().publish(
            (Symbol::new(env, "wellness_activity_tracked"),),
            (user_id, duration_minutes),
        );

        Ok(())
    }

    pub fn get_wellness_activities(
        env: &Env,
        user_id: Address,
        days: u32,
    ) -> Vec<WellnessActivity> {
        let activities_key = String::from_str(env, "user_activities");
        let activities: Vec<WellnessActivity> = env.storage().instance()
            .get(&activities_key)
            .unwrap_or(Vec::new(env));

        let cutoff_time = env.ledger().timestamp() - (days as u64 * 24 * 60 * 60);

        activities.iter()
            .filter(|activity| activity.timestamp >= cutoff_time)
            .collect()
    }

    fn update_program_effectiveness(env: &Env, program_id: u64) {
        let feedbacks = Self::get_program_feedback(env, program_id);

        if feedbacks.is_empty() {
            return;
        }

        let mut total_rating = 0.0;
        for feedback in feedbacks.iter() {
            total_rating += feedback.overall_rating as f32;
        }

        let average_rating = total_rating / feedbacks.len() as f32;

        let mut program: WellnessProgram = env.storage().instance()
            .get(&program_id)
            .unwrap();
        program.effectiveness_score = average_rating;
        env.storage().instance().set(&program_id, &program);
    }
}

#[contracttype]
#[derive(Clone)]
pub struct WellnessFeedback {
    pub user_id: Address,
    pub program_id: u64,
    pub overall_rating: u32,
    pub helpful_modules: Vec<u64>,
    pub suggestions: String,
    pub submitted_timestamp: u64,
}

#[contracttype]
#[derive(Clone)]
pub struct WellnessRecommendation {
    pub recommendation_type: String,
    pub title: String,
    pub description: String,
    pub priority: u32,
}

#[contracttype]
#[derive(Clone)]
pub struct WellnessActivity {
    pub user_id: Address,
    pub activity_type: String,
    pub duration_minutes: u32,
    pub timestamp: u64,
    pub notes: String,
}