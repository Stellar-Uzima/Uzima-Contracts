use crate::{errors::Error, events::*, types::*};
use soroban_sdk::{contracttype, Address, Env, Map, String, Symbol, Vec};

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

        // Check for crisis indicators
        let risk_indicators = ai_analysis.risk_indicators.clone();
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

        if Self::detect_crisis_risk(env, &risk_indicators) {
            // Trigger crisis alert
            Self::trigger_crisis_alert(env, patient_id.clone(), risk_indicators);
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
                average_mood: 0,
                trend_direction: String::from_str(env, "insufficient_data"),
                volatility: 0,
                dominant_emotions: Vec::new(env),
                risk_trends: Vec::new(env),
                recommendations: Vec::new(env),
            };
        }

        // Calculate average mood
        let mut total_mood: i32 = 0;
        let mut emotion_counts = Map::new(env);
        let mut risk_indicators = Vec::new(env);

        for entry in entries.iter() {
            total_mood += entry.mood_score;

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

        let average_mood = (total_mood * 100) / (entries.len() as i32);

        // Determine trend direction (simplified)
        let recent_avg = if entries.len() > 10 {
            let mut recent_total: i32 = 0;
            for i in (entries.len() - 10)..entries.len() {
                recent_total += entries.get(i).unwrap().mood_score;
            }
            (recent_total * 100) / 10
        } else {
            average_mood
        };

        let trend_direction = if recent_avg > average_mood + 50 {
            String::from_str(env, "improving")
        } else if recent_avg < average_mood - 50 {
            String::from_str(env, "declining")
        } else {
            String::from_str(env, "stable")
        };

        // Calculate volatility (simplified variance)
        let mut variance = 0;
        for entry in entries.iter() {
            let diff = (entry.mood_score * 100) - average_mood;
            variance += (diff * diff) as u32;
        }
        let volatility = variance / (entries.len() as u32);

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
            average_mood: average_mood as u32,
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
            // Check for crisis-related trigger keywords using soroban String comparison
            if trigger == String::from_str(env, "suicide")
                || trigger == String::from_str(env, "suicidal")
                || trigger == String::from_str(env, "worthless")
            {
                risk_indicators.push_back(String::from_str(env, "suicidal_ideation"));
            }
            if trigger == String::from_str(env, "self_harm")
                || trigger == String::from_str(env, "cutting")
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
            sentiment_score: ((mood_score + 10) * 5) as u32, // scale to 0-100
            dominant_emotion,
            risk_indicators,
            recommendations,
            trend_analysis: String::from_str(env, "Analysis based on current entry"),
        }
}

    fn detect_crisis_risk(env: &Env, risk_indicators: &Vec<String>) -> bool {
        for indicator in risk_indicators.iter() {
            if indicator == String::from_str(env, "suicidal_ideation")
                || indicator == String::from_str(env, "self_harm")
                || indicator == String::from_str(env, "severe_negative_mood")
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
        average_mood: i32,
        trend: String,
        volatility: u32,
    ) -> Vec<String> {
        let mut recommendations = Vec::new(env);

        if average_mood < -200 {
            recommendations.push_back(String::from_str(env, "Consider therapy or counseling"));
        }

        if volatility > 300 {
            recommendations.push_back(String::from_str(
                env,
                "Mood swings detected - consider mood stabilizing activities",
            ));
        }

        if trend == String::from_str(env, "declining") {
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
    pub average_mood: u32,
    pub trend_direction: String,
    pub volatility: u32,
    pub dominant_emotions: Vec<String>,
    pub risk_trends: Vec<String>,
    pub recommendations: Vec<String>,
}
