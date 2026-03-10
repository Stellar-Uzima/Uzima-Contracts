use crate::{errors::Error, events::*, types::*};
use soroban_sdk::{Address, Env, Map, String, Symbol, Vec};

pub struct AssessmentManager;

impl AssessmentManager {
    pub fn create_assessment(
        env: &Env,
        patient_id: Address,
        assessment_type: AssessmentType,
        administered_by: Address,
    ) -> Result<u64, Error> {
        let assessment_id = env.ledger().timestamp() as u64;

        let assessment = Assessment {
            assessment_id,
            patient_id: patient_id.clone(),
            assessment_type,
            timestamp: env.ledger().timestamp(),
            responses: Map::new(env),
            score: None,
            interpretation: String::from_str(env, ""),
            risk_flags: Vec::new(env),
            recommendations: Vec::new(env),
            administered_by,
        };

        // Store assessment
        let mut assessments: Vec<Assessment> = env
            .storage()
            .instance()
            .get(&patient_id)
            .unwrap_or(Vec::new(env));
        assessments.push_back(assessment);
        env.storage().instance().set(&patient_id, &assessments);

        Ok(assessment_id)
    }

    pub fn submit_assessment_responses(
        env: &Env,
        assessment_id: u64,
        patient_id: Address,
        responses: Map<String, String>,
    ) -> Result<(), Error> {
        let mut assessments: Vec<Assessment> = env
            .storage()
            .instance()
            .get(&patient_id)
            .ok_or(Error::AssessmentNotFound)?;

        for i in 0..assessments.len() {
            let mut assessment = assessments.get(i).unwrap();
            if assessment.assessment_id == assessment_id {
                assessment.responses = responses.clone();

                // Calculate score and interpretation
                let (score, interpretation, risk_flags, recommendations) =
                    Self::score_assessment(env, assessment.assessment_type, responses);

                assessment.score = Some(score);
                assessment.interpretation = interpretation;
                assessment.risk_flags = risk_flags.clone();
                assessment.recommendations = recommendations;

                assessments.set(i, assessment.clone());

                env.storage().instance().set(&patient_id, &assessments);

                // Emit event
                env.events().publish(
                    (Symbol::new(env, "assessment_completed"),),
                    AssessmentCompletedEvent {
                        assessment_id,
                        patient_id: patient_id.clone(),
                        assessment_type: Self::assessment_type_to_string(
                            env,
                            assessment.assessment_type,
                        ),
                        score,
                        timestamp: env.ledger().timestamp(),
                    },
                );

                // Check for high-risk results
                if Self::is_high_risk(env, &risk_flags) {
                    Self::trigger_risk_alert(env, patient_id.clone(), risk_flags);
                }

                return Ok(());
            }
        }

        Err(Error::AssessmentNotFound)
    }

    pub fn get_patient_assessments(env: &Env, patient_id: Address) -> Vec<Assessment> {
        env.storage()
            .instance()
            .get(&patient_id)
            .unwrap_or(Vec::new(env))
    }

    pub fn get_assessment_results(
        env: &Env,
        assessment_id: u64,
        patient_id: Address,
    ) -> Result<Assessment, Error> {
        let assessments = Self::get_patient_assessments(env, patient_id);

        for assessment in assessments.iter() {
            if assessment.assessment_id == assessment_id {
                return Ok(assessment);
            }
        }

        Err(Error::AssessmentNotFound)
    }

    fn score_assessment(
        env: &Env,
        assessment_type: AssessmentType,
        responses: Map<String, String>,
    ) -> (u32, String, Vec<String>, Vec<String>) {
        match assessment_type {
            AssessmentType::PHQ9 => Self::score_phq9(env, responses),
            AssessmentType::GAD7 => Self::score_gad7(env, responses),
            AssessmentType::PCL5 => Self::score_pcl5(env, responses),
            _ => Self::score_generic(env, responses),
        }
    }

    fn score_phq9(
        env: &Env,
        responses: Map<String, String>,
    ) -> (u32, String, Vec<String>, Vec<String>) {
        let mut total_score = 0u32;
        let mut risk_flags = Vec::new(env);
        let mut recommendations = Vec::new(env);

        // PHQ-9 scoring (simplified)
        for (_, response) in responses.iter() {
            if response.len() > 0 {
                total_score += 1;
            }
        }

        let interpretation = match total_score {
            0..=4 => String::from_str(env, "Minimal depression"),
            5..=9 => String::from_str(env, "Mild depression"),
            10..=14 => String::from_str(env, "Moderate depression"),
            15..=19 => String::from_str(env, "Moderately severe depression"),
            20..=27 => {
                risk_flags.push_back(String::from_str(env, "severe_depression"));
                String::from_str(env, "Severe depression")
            }
            _ => String::from_str(env, "Invalid score"),
        };

        if total_score >= 15 {
            recommendations.push_back(String::from_str(
                env,
                "Immediate professional evaluation recommended",
            ));
            recommendations.push_back(String::from_str(
                env,
                "Consider intensive treatment options",
            ));
        } else if total_score >= 10 {
            recommendations.push_back(String::from_str(
                env,
                "Professional consultation recommended",
            ));
            recommendations.push_back(String::from_str(env, "Consider therapy or medication"));
        }

        (total_score, interpretation, risk_flags, recommendations)
    }

    fn score_gad7(
        env: &Env,
        responses: Map<String, String>,
    ) -> (u32, String, Vec<String>, Vec<String>) {
        let mut total_score = 0u32;
        let mut risk_flags = Vec::new(env);
        let mut recommendations = Vec::new(env);

        for (_, response) in responses.iter() {
            if response.len() > 0 {
                total_score += 1;
            }
        }

        let interpretation = match total_score {
            0..=4 => String::from_str(env, "Minimal anxiety"),
            5..=9 => String::from_str(env, "Mild anxiety"),
            10..=14 => String::from_str(env, "Moderate anxiety"),
            15..=27 => {
                risk_flags.push_back(String::from_str(env, "severe_anxiety"));
                String::from_str(env, "Severe anxiety")
            }
            _ => String::from_str(env, "Invalid score"),
        };

        if total_score >= 15 {
            recommendations.push_back(String::from_str(
                env,
                "Professional mental health evaluation needed",
            ));
            recommendations.push_back(String::from_str(env, "Consider anxiety-specific therapy"));
        } else if total_score >= 10 {
            recommendations.push_back(String::from_str(
                env,
                "Consultation with mental health professional",
            ));
        }

        (total_score, interpretation, risk_flags, recommendations)
    }

    fn score_pcl5(
        env: &Env,
        responses: Map<String, String>,
    ) -> (u32, String, Vec<String>, Vec<String>) {
        let mut total_score = 0u32;
        let mut risk_flags = Vec::new(env);
        let mut recommendations = Vec::new(env);

        for (_, response) in responses.iter() {
            if response.len() > 0 {
                total_score += 1;
            }
        }

        let interpretation = match total_score {
            0..=30 => String::from_str(env, "Minimal PTSD symptoms"),
            31..=40 => String::from_str(env, "Mild PTSD symptoms"),
            41..=50 => String::from_str(env, "Moderate PTSD symptoms"),
            51..=80 => {
                risk_flags.push_back(String::from_str(env, "ptsd"));
                String::from_str(env, "Severe PTSD symptoms")
            }
            _ => String::from_str(env, "Invalid score"),
        };

        if total_score >= 33 {
            recommendations.push_back(String::from_str(
                env,
                "PTSD evaluation and treatment recommended",
            ));
            recommendations.push_back(String::from_str(env, "Consider trauma-focused therapy"));
        }

        (total_score, interpretation, risk_flags, recommendations)
    }

    fn score_generic(
        env: &Env,
        responses: Map<String, String>,
    ) -> (u32, String, Vec<String>, Vec<String>) {
        let score = responses.len() as u32; // Simple scoring for custom assessments
        let interpretation = String::from_str(env, "Custom assessment completed");
        let risk_flags = Vec::new(env);
        let recommendations = Vec::new(env);

        (score, interpretation, risk_flags, recommendations)
    }

    fn is_high_risk(env: &Env, risk_flags: &Vec<String>) -> bool {
        for flag in risk_flags.iter() {
            if flag == String::from_str(env, "severe_depression")
                || flag == String::from_str(env, "severe_anxiety")
                || flag == String::from_str(env, "ptsd")
                || flag == String::from_str(env, "suicidal_risk")
            {
                return true;
            }
        }
        false
    }

    fn trigger_risk_alert(env: &Env, patient_id: Address, risk_flags: Vec<String>) {
        env.events().publish(
            (Symbol::new(env, "assessment_risk_alert"),),
            (patient_id, risk_flags),
        );
    }

    fn assessment_type_to_string(env: &Env, assessment_type: AssessmentType) -> String {
        match assessment_type {
            AssessmentType::PHQ9 => String::from_str(env, "PHQ9"),
            AssessmentType::GAD7 => String::from_str(env, "GAD7"),
            AssessmentType::PCL5 => String::from_str(env, "PCL5"),
            AssessmentType::AUDIT => String::from_str(env, "AUDIT"),
            AssessmentType::DAST => String::from_str(env, "DAST"),
            AssessmentType::BDI => String::from_str(env, "BDI"),
            AssessmentType::BAI => String::from_str(env, "BAI"),
            AssessmentType::Custom => String::from_str(env, "Custom"),
        }
    }
}
