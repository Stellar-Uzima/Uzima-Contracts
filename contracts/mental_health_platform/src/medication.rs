use crate::{errors::Error, events::*, types::*};
use soroban_sdk::{contracttype, Address, Env, String, Vec};

pub struct MedicationManager;

impl MedicationManager {
    pub fn create_medication_plan(
        env: &Env,
        patient_id: Address,
        medication_name: String,
        dosage: String,
        frequency: String,
        start_date: u64,
        end_date: Option<u64>,
        prescribed_by: Address,
        side_effects: Vec<String>,
    ) -> Result<u64, Error> {
        let plan_id = env.ledger().timestamp() as u64;

        let plan = MedicationPlan {
            plan_id,
            patient_id: patient_id.clone(),
            medication_name,
            dosage,
            frequency,
            start_date,
            end_date,
            prescribed_by,
            side_effects,
            adherence_tracking: Vec::new(env),
            effectiveness_rating: None,
            notes: String::from_str(env, ""),
        };

        // Store plan
        let mut plans: Vec<MedicationPlan> = env
            .storage()
            .instance()
            .get(&patient_id)
            .unwrap_or(Vec::new(env));
        plans.push_back(plan);
        env.storage().instance().set(&patient_id, &plans);

        env.events().publish(
            (Symbol::new(env, "medication_plan_created"),),
            (plan_id, patient_id, prescribed_by),
        );

        Ok(plan_id)
    }

    pub fn record_adherence(
        env: &Env,
        plan_id: u64,
        patient_id: Address,
        taken: bool,
        dosage_taken: Option<String>,
        side_effects_experienced: Vec<String>,
        notes: String,
    ) -> Result<(), Error> {
        let mut plans: Vec<MedicationPlan> = env
            .storage()
            .instance()
            .get(&patient_id)
            .ok_or(Error::MedicationPlanNotFound)?;

        for i in 0..plans.len() {
            let mut plan = plans.get(i).unwrap();
            if plan.plan_id == plan_id {
                let entry = AdherenceEntry {
                    timestamp: env.ledger().timestamp(),
                    taken,
                    dosage_taken,
                    side_effects_experienced,
                    notes,
                };

                plan.adherence_tracking.push_back(entry);
                plans.set(i, plan);
                env.storage().instance().set(&patient_id, &plans);

                // Emit event
                env.events().publish(
                    (Symbol::new(env, "medication_adherence_recorded"),),
                    MedicationAdherenceEvent {
                        plan_id,
                        patient_id,
                        taken,
                        timestamp: env.ledger().timestamp(),
                    },
                );

                // Check for adherence issues
                Self::check_adherence_patterns(env, patient_id.clone(), plan);

                return Ok(());
            }
        }

        Err(Error::MedicationPlanNotFound)
    }

    pub fn update_effectiveness_rating(
        env: &Env,
        plan_id: u64,
        patient_id: Address,
        rating: u32,
    ) -> Result<(), Error> {
        if rating > 10 {
            return Err(Error::InvalidInput);
        }

        let mut plans: Vec<MedicationPlan> = env
            .storage()
            .instance()
            .get(&patient_id)
            .ok_or(Error::MedicationPlanNotFound)?;

        for i in 0..plans.len() {
            let mut plan = plans.get(i).unwrap();
            if plan.plan_id == plan_id {
                plan.effectiveness_rating = Some(rating);
                plans.set(i, plan);
                env.storage().instance().set(&patient_id, &plans);

                return Ok(());
            }
        }

        Err(Error::MedicationPlanNotFound)
    }

    pub fn get_patient_medication_plans(env: &Env, patient_id: Address) -> Vec<MedicationPlan> {
        env.storage()
            .instance()
            .get(&patient_id)
            .unwrap_or(Vec::new(env))
    }

    pub fn calculate_adherence_rate(
        env: &Env,
        plan_id: u64,
        patient_id: Address,
        days: u32,
    ) -> Result<f32, Error> {
        let plans = Self::get_patient_medication_plans(env, patient_id);

        for plan in plans.iter() {
            if plan.plan_id == plan_id {
                let recent_entries: Vec<AdherenceEntry> = plan
                    .adherence_tracking
                    .iter()
                    .filter(|entry| {
                        env.ledger().timestamp() - entry.timestamp < days * 24 * 60 * 60
                    })
                    .collect();

                if recent_entries.is_empty() {
                    return Ok(0.0);
                }

                let taken_count = recent_entries.iter().filter(|entry| entry.taken).count();

                return Ok(taken_count as f32 / recent_entries.len() as f32);
            }
        }

        Err(Error::MedicationPlanNotFound)
    }

    pub fn analyze_medication_effectiveness(
        env: &Env,
        plan_id: u64,
        patient_id: Address,
    ) -> Result<MedicationEffectivenessAnalysis, Error> {
        let plans = Self::get_patient_medication_plans(env, patient_id);

        for plan in plans.iter() {
            if plan.plan_id == plan_id {
                let adherence_rate = Self::calculate_adherence_rate(env, plan_id, patient_id, 30)?;
                let effectiveness_rating = plan.effectiveness_rating.unwrap_or(5);

                let mut side_effects_frequency = Vec::new(env);
                let mut side_effect_counts = Map::new(env);

                // Count side effects
                for entry in plan.adherence_tracking.iter() {
                    for side_effect in entry.side_effects_experienced.iter() {
                        let count = side_effect_counts.get(side_effect.clone()).unwrap_or(0u32) + 1;
                        side_effect_counts.set(side_effect, count);
                    }
                }

                // Get most common side effects
                for (side_effect, count) in side_effect_counts.iter() {
                    side_effects_frequency.push_back(String::from_str(env, "Side effect recorded"));
                }

                let recommendations = Self::generate_medication_recommendations(
                    env,
                    adherence_rate,
                    effectiveness_rating,
                    side_effects_frequency.len(),
                );

                return Ok(MedicationEffectivenessAnalysis {
                    adherence_rate,
                    effectiveness_rating: effectiveness_rating as f32,
                    side_effects_frequency,
                    recommendations,
                    analysis_timestamp: env.ledger().timestamp(),
                });
            }
        }

        Err(Error::MedicationPlanNotFound)
    }

    fn check_adherence_patterns(env: &Env, patient_id: Address, plan: MedicationPlan) {
        let adherence_rate =
            Self::calculate_adherence_rate(env, plan.plan_id, patient_id, 7).unwrap_or(0.0);

        if adherence_rate < 0.7 {
            // Low adherence detected
            env.events().publish(
                (Symbol::new(env, "low_adherence_alert"),),
                (plan.plan_id, patient_id, adherence_rate),
            );
        }

        // Check for side effect patterns
        let recent_side_effects: Vec<String> = plan
            .adherence_tracking
            .iter()
            .filter(|entry| env.ledger().timestamp() - entry.timestamp < 7 * 24 * 60 * 60)
            .flat_map(|entry| entry.side_effects_experienced.clone())
            .collect();

        if recent_side_effects.len() > 5 {
            env.events().publish(
                (Symbol::new(env, "side_effects_alert"),),
                (plan.plan_id, patient_id, recent_side_effects.len()),
            );
        }
    }

    fn generate_medication_recommendations(
        env: &Env,
        adherence_rate: f32,
        effectiveness: f32,
        side_effects_count: usize,
    ) -> Vec<String> {
        let mut recommendations = Vec::new(env);

        if adherence_rate < 0.8 {
            recommendations.push_back(String::from_str(
                env,
                "Consider adherence support strategies",
            ));
            recommendations.push_back(String::from_str(
                env,
                "Discuss adherence barriers with healthcare provider",
            ));
        }

        if effectiveness < 5.0 {
            recommendations.push_back(String::from_str(
                env,
                "Discuss alternative medications with prescriber",
            ));
            recommendations.push_back(String::from_str(env, "Consider dosage adjustment"));
        }

        if side_effects_count > 3 {
            recommendations.push_back(String::from_str(
                env,
                "Report side effects to healthcare provider",
            ));
            recommendations.push_back(String::from_str(
                env,
                "Consider side effect management strategies",
            ));
        }

        if recommendations.is_empty() {
            recommendations.push_back(String::from_str(env, "Continue current medication regimen"));
        }

        recommendations
    }
}

#[contracttype]
#[derive(Clone)]
pub struct MedicationEffectivenessAnalysis {
    pub adherence_rate: f32,
    pub effectiveness_rating: f32,
    pub side_effects_frequency: Vec<String>,
    pub recommendations: Vec<String>,
    pub analysis_timestamp: u64,
}
