use crate::{errors::Error, events::*, types::*};
use soroban_sdk::{contracttype, Address, Env, Map, String, Vec};

pub struct MoodTracker;

impl MoodTracker {
    pub fn record_mood(
        env: &Env,
        patient_id: Address,
        mood_score: i32,
        emotions: Vec<String>,
        triggers: Vec<String>,
        notes: String,
        location_context: Option<String>,
    ) -> Result<u64, Error> {
        // Validate mood score
        if mood_score < -10 || mood_score > 10 {
            return Err(Error::InvalidInput);
        }

        let entry_id = env.ledger().timestamp() as u64;

        // Perform AI analysis (simplified for this implementation)
        let ai_analysis = Self::analyze_mood(env, mood_score, emotions.clone(), triggers.clone());

        let entry = MoodEntry {
            entry_id,
            patient_id: patient_id.clone(),
            timestamp: env.ledger().timestamp(),
            mood_score,
            emotions,
            triggers,
            notes,
            location_context,
            ai_analysis: Some(ai_analysis),
        };

        // Store entry
        let mut entries: Vec<MoodEntry> = env
            .storage()
            .instance()
            .get(&patient_id)
            .unwrap_or(Vec::new(env));
        entries.push_back(entry);
        env.storage().instance().set(&patient_id, &entries);

        // Check for crisis indicators
        if Self::detect_crisis_risk(&ai_analysis.risk_indicators) {
            // Trigger crisis alert
            Self::trigger_crisis_alert(env, patient_id.clone(), ai_analysis.risk_indicators);
        }

        // Emit event
        env.events().publish(
            (Symbol::new(env, "mood_entry_recorded"),),
            MoodEntryEvent {
                entry_id,
                patient_id,
                mood_score,
                timestamp: env.ledger().timestamp(),
            },
        );

        Ok(entry_id)
    }

    pub fn get_mood_history(env: &Env, patient_id: Address, limit: Option<u32>) -> Vec<MoodEntry> {
        let entries: Vec<MoodEntry> = env
            .storage()
            .instance()
            .get(&patient_id)
            .unwrap_or(Vec::new(env));

        if let Some(limit) = limit {
            let start = if entries.len() > limit {
                entries.len() - limit
            } else {
                0
            };
            let mut result = Vec::new(env);
            for i in start..entries.len() {
                result.push_back(entries.get(i).unwrap());
            }
            result
        } else {
            entries
        }
    }

    pub fn analyze_mood_trends(env: &Env, patient_id: Address, days: u32) -> MoodTrendAnalysis {
        let entries = Self::get_mood_history(env, patient_id, Some(days * 24)); // Assuming hourly entries

        if entries.is_empty() {
            return MoodTrendAnalysis {
                average_mood: 0.0,
                trend_direction: String::from_str(env, "insufficient_data"),
                volatility: 0.0,
                dominant_emotions: Vec::new(env),
                risk_trends: Vec::new(env),
                recommendations: Vec::new(env),
            };
        }

        // Calculate average mood
        let mut total_mood = 0.0;
        let mut emotion_counts = Map::new(env);
        let mut risk_indicators = Vec::new(env);

        for entry in entries.iter() {
            total_mood += entry.mood_score as f32;

            // Count emotions
            for emotion in entry.emotions.iter() {
                let count = emotion_counts.get(emotion.clone()).unwrap_or(0u32) + 1;
                emotion_counts.set(emotion, count);
            }

            // Collect risk indicators from AI analysis
            if let Some(analysis) = &entry.ai_analysis {
                for risk in analysis.risk_indicators.iter() {
                    risk_indicators.push_back(risk.clone());
                }
            }
        }

        let average_mood = total_mood / entries.len() as f32;

        // Determine trend direction (simplified)
        let recent_avg = if entries.len() > 10 {
            let mut recent_total = 0.0;
            for i in (entries.len() - 10)..entries.len() {
                recent_total += entries.get(i).unwrap().mood_score as f32;
            }
            recent_total / 10.0
        } else {
            average_mood
        };

        let trend_direction = if recent_avg > average_mood + 0.5 {
            String::from_str(env, "improving")
        } else if recent_avg < average_mood - 0.5 {
            String::from_str(env, "declining")
        } else {
            String::from_str(env, "stable")
        };

        // Calculate volatility (standard deviation)
        let mut variance = 0.0;
        for entry in entries.iter() {
            let diff = entry.mood_score as f32 - average_mood;
            variance += diff * diff;
        }
        let volatility = (variance / entries.len() as f32).sqrt();

        // Get dominant emotions
        let mut dominant_emotions = Vec::new(env);
        let mut max_count = 0u32;
        for (emotion, count) in emotion_counts.iter() {
            if count > max_count {
                max_count = count;
                dominant_emotions = Vec::new(env);
                dominant_emotions.push_back(emotion);
            } else if count == max_count {
                dominant_emotions.push_back(emotion);
            }
        }

        // Generate recommendations
        let recommendations =
            Self::generate_recommendations(env, average_mood, trend_direction.clone(), volatility);

        MoodTrendAnalysis {
            average_mood,
            trend_direction,
            volatility,
            dominant_emotions,
            risk_trends: risk_indicators,
            recommendations,
        }
    }

    fn analyze_mood(
        env: &Env,
        mood_score: i32,
        emotions: Vec<String>,
        triggers: Vec<String>,
    ) -> MoodAnalysis {
        let mut risk_indicators = Vec::new(env);
        let mut recommendations = Vec::new(env);

        // Determine dominant emotion
        let dominant_emotion = if emotions.len() > 0 {
            emotions.get(0).unwrap()
        } else {
            String::from_str(env, "neutral")
        };

        // Risk assessment
        if mood_score <= -7 {
            risk_indicators.push_back(String::from_str(env, "severe_negative_mood"));
        } else if mood_score <= -4 {
            risk_indicators.push_back(String::from_str(env, "negative_mood"));
        }

        // Check for crisis keywords in triggers
        for trigger in triggers.iter() {
            let trigger_lower = trigger.to_lowercase();
            if trigger_lower.contains("suicide")
                || trigger_lower.contains("kill")
                || trigger_lower.contains("end it")
                || trigger_lower.contains("worthless")
            {
                risk_indicators.push_back(String::from_str(env, "suicidal_ideation"));
            }
            if trigger_lower.contains("harm")
                || trigger_lower.contains("cut")
                || trigger_lower.contains("hurt myself")
            {
                risk_indicators.push_back(String::from_str(env, "self_harm"));
            }
        }

        // Generate recommendations
        if mood_score < 0 {
            recommendations.push_back(String::from_str(
                env,
                "Consider reaching out to a mental health professional",
            ));
            recommendations.push_back(String::from_str(
                env,
                "Practice deep breathing or mindfulness exercises",
            ));
        }

        if risk_indicators.len() > 0 {
            recommendations.push_back(String::from_str(
                env,
                "Immediate professional help recommended",
            ));
        }

        MoodAnalysis {
            sentiment_score: mood_score as f32 / 10.0,
            dominant_emotion,
            risk_indicators,
            recommendations,
            trend_analysis: String::from_str(env, "Analysis based on current entry"),
        }
    }

    fn detect_crisis_risk(risk_indicators: &Vec<String>) -> bool {
        for indicator in risk_indicators.iter() {
            if indicator == "suicidal_ideation"
                || indicator == "self_harm"
                || indicator == "severe_negative_mood"
            {
                return true;
            }
        }
        false
    }

    fn trigger_crisis_alert(env: &Env, patient_id: Address, risk_indicators: Vec<String>) {
        // This would integrate with the crisis intervention module
        // For now, just log the event
        env.events().publish(
            (Symbol::new(env, "crisis_risk_detected"),),
            (patient_id, risk_indicators),
        );
    }

    fn generate_recommendations(
        env: &Env,
        average_mood: f32,
        trend: String,
        volatility: f32,
    ) -> Vec<String> {
        let mut recommendations = Vec::new(env);

        if average_mood < -2.0 {
            recommendations.push_back(String::from_str(env, "Consider therapy or counseling"));
        }

        if volatility > 3.0 {
            recommendations.push_back(String::from_str(
                env,
                "Mood swings detected - consider mood stabilizing activities",
            ));
        }

        if trend == "declining" {
            recommendations.push_back(String::from_str(
                env,
                "Trend shows declining mood - reach out for support",
            ));
        }

        recommendations
    }
}

#[contracttype]
#[derive(Clone)]
pub struct MoodTrendAnalysis {
    pub average_mood: f32,
    pub trend_direction: String,
    pub volatility: f32,
    pub dominant_emotions: Vec<String>,
    pub risk_trends: Vec<String>,
    pub recommendations: Vec<String>,
}
