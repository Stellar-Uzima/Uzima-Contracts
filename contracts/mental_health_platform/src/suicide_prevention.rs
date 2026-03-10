use crate::{errors::Error, events::*, types::*};
use soroban_sdk::{contracttype, Address, Env, Map, String, Symbol, Vec, vec};

pub struct SuicidePreventionManager;

impl SuicidePreventionManager {
    pub fn create_prevention_protocol(
        env: &Env,
        name: String,
        triggers: Vec<String>,
        risk_factors: Vec<String>,
        intervention_steps: Vec<String>,
        emergency_contacts: Vec<EmergencyContact>,
        resources: Vec<String>,
    ) -> Result<u64, Error> {
        let protocol_id = env.ledger().timestamp() as u64;

        let protocol = SuicidePreventionProtocol {
            protocol_id,
            name,
            triggers,
            risk_factors,
            intervention_steps,
            emergency_contacts,
            resources,
            success_rate: 0, // Will be updated based on outcomes
        };

        env.storage().instance().set(&protocol_id, &protocol);

        env.events().publish(
            (Symbol::new(env, "prevention_protocol_created"),),
            protocol_id,
        );

        Ok(protocol_id)
    }

    pub fn detect_suicide_risk(
        env: &Env,
        patient_id: Address,
        indicators: Vec<String>,
        context_data: Map<String, String>,
    ) -> SuicideRiskAssessment {
        let mut risk_score: u32 = 0;
        let mut risk_factors = Vec::new(env);
        let mut recommended_actions = Vec::new(env);

        // Analyze indicators
        for indicator in indicators.iter() {
            if indicator == String::from_str(env, "suicidal_ideation") {
                risk_score += 90;
                risk_factors.push_back(String::from_str(env, "Active suicidal ideation"));
                recommended_actions
                    .push_back(String::from_str(env, "Immediate crisis intervention"));
            } else if indicator == String::from_str(env, "suicide_plan") {
                risk_score += 100;
                risk_factors.push_back(String::from_str(env, "Suicide plan present"));
                recommended_actions
                    .push_back(String::from_str(env, "Emergency psychiatric evaluation"));
            } else if indicator == String::from_str(env, "suicide_attempt_history") {
                risk_score += 70;
                risk_factors.push_back(String::from_str(env, "History of suicide attempts"));
                recommended_actions
                    .push_back(String::from_str(env, "Close monitoring required"));
            } else if indicator == String::from_str(env, "severe_hopelessness") {
                risk_score += 60;
                risk_factors.push_back(String::from_str(env, "Severe hopelessness"));
                recommended_actions
                    .push_back(String::from_str(env, "Immediate therapeutic intervention"));
            } else if indicator == String::from_str(env, "social_isolation") {
                risk_score += 40;
                risk_factors.push_back(String::from_str(env, "Extreme social isolation"));
                recommended_actions
                    .push_back(String::from_str(env, "Social support intervention"));
            } else {
                risk_score += 10;
            }
        }

        // Check context data
        if let Some(mood_score_str) = context_data.get(String::from_str(env, "recent_mood_score")) {
            // Simplified: assuming it's a small integer string for now
            // In reality would need a safe parse to i32/u32
            if mood_score_str.len() > 0 { 
                risk_score += 30;
                risk_factors.push_back(String::from_str(env, "Recent low mood indicators"));
            }
        }

        // Check recent crisis history
        let crisis_history = Self::get_patient_prevention_alerts(env, patient_id.clone());
        let recent_alerts = crisis_history
            .iter()
            .filter(|alert| env.ledger().timestamp() - alert.timestamp < 30 * 24 * 60 * 60)
            .count();

        if recent_alerts > 0 {
            risk_score += 20 * recent_alerts as u32;
            risk_factors.push_back(String::from_str(env, "Recent prevention alerts"));
        }

        // Determine risk level and protocol
        let (risk_level, protocol_id) = if risk_score >= 80 {
            (
                SuicideRiskLevel::High,
                Some(Self::get_high_risk_protocol(env)),
            )
        } else if risk_score >= 50 {
            (
                SuicideRiskLevel::Moderate,
                Some(Self::get_moderate_risk_protocol(env)),
            )
        } else if risk_score >= 20 {
            (
                SuicideRiskLevel::Low,
                Some(Self::get_low_risk_protocol(env)),
            )
        } else {
            (SuicideRiskLevel::None, None)
        };

        SuicideRiskAssessment {
            patient_id,
            risk_score,
            risk_level,
            risk_factors,
            recommended_actions,
            protocol_id,
            assessment_timestamp: env.ledger().timestamp(),
        }
    }

    pub fn activate_prevention_alert(
        env: &Env,
        patient_id: Address,
        protocol_id: u64,
        trigger_reason: String,
        risk_score: u32,
    ) -> Result<u64, Error> {
        let alert_id = env.ledger().timestamp() as u64;

        let alert = PreventionAlert {
            alert_id,
            patient_id: patient_id.clone(),
            protocol_id,
            trigger_reason,
            risk_score,
            timestamp: env.ledger().timestamp(),
            actions_taken: Vec::new(env),
            outcome: PreventionOutcome::OngoingMonitoring,
        };

        // Store alert
        let key = String::from_str(env, "prevention_alerts");
        let mut alerts: Vec<PreventionAlert> =
            env.storage().instance().get(&key).unwrap_or(Vec::new(env));
        alerts.push_back(alert);
        env.storage().instance().set(&key, &alerts);

        // Trigger immediate actions based on risk
        Self::execute_prevention_protocol(env, patient_id.clone(), protocol_id);

        env.events().publish(
            (Symbol::new(env, "prevention_alert_activated"),),
            PreventionAlertEvent {
                alert_id,
                patient_id,
                protocol_id,
                risk_score,
                timestamp: env.ledger().timestamp(),
            },
        );

        Ok(alert_id)
    }

    pub fn update_prevention_outcome(
        env: &Env,
        alert_id: u64,
        patient_id: Address,
        actions_taken: Vec<String>,
        outcome: PreventionOutcome,
    ) -> Result<(), Error> {
        let key = String::from_str(env, "prevention_alerts");
        let mut alerts: Vec<PreventionAlert> = env
            .storage()
            .instance()
            .get(&key)
            .ok_or(Error::InvalidInput)?;

        for i in 0..alerts.len() {
            let mut alert = alerts.get(i).unwrap();
            if alert.alert_id == alert_id {
                alert.actions_taken = actions_taken;
                alert.outcome = outcome;
                alerts.set(i, alert.clone());
                env.storage().instance().set(&key, &alerts);

                // Update protocol success rate
                Self::update_protocol_success_rate(env, alert.protocol_id, outcome);

                env.events().publish(
                    (Symbol::new(env, "prevention_outcome_updated"),),
                    (alert_id, patient_id, outcome),
                );

                return Ok(());
            }
        }

        Err(Error::InvalidInput)
    }

    pub fn get_suicide_hotlines(env: &Env) -> Vec<SuicideHotline> {
        vec![
            env,
            SuicideHotline {
                name: String::from_str(env, "National Suicide Prevention Lifeline"),
                number: String::from_str(env, "988"),
                country: String::from_str(env, "US"),
                languages: vec![
                    env,
                    String::from_str(env, "English"),
                    String::from_str(env, "Spanish"),
                ],
                available_24_7: true,
                crisis_text_available: true,
                text_number: Some(String::from_str(env, "988")),
            },
            SuicideHotline {
                name: String::from_str(env, "International Association for Suicide Prevention"),
                number: String::from_str(env, "+1-202-237-2280"),
                country: String::from_str(env, "International"),
                languages: vec![env, String::from_str(env, "Multiple")],
                available_24_7: false,
                crisis_text_available: false,
                text_number: None,
            },
        ]
    }

    pub fn get_crisis_resources(env: &Env, location: Option<String>) -> Vec<CrisisResource> {
        let mut resources = Vec::new(env);

        // Add general resources
        resources.push_back(CrisisResource {
            name: String::from_str(env, "Crisis Text Line"),
            contact: String::from_str(env, "Text HOME to 741741"),
            resource_type: String::from_str(env, "Text Support"),
            description: String::from_str(env, "24/7 crisis counseling via text"),
            languages: vec![env, String::from_str(env, "English")],
        });

        resources.push_back(CrisisResource {
            name: String::from_str(env, "Mental Health America"),
            contact: String::from_str(env, "1-800-950-6264"),
            resource_type: String::from_str(env, "Helpline"),
            description: String::from_str(env, "Mental health screening and referrals"),
            languages: vec![env, String::from_str(env, "English")],
        });

        // Location-specific resources would be added here
        if let Some(_location) = location {
            // Location-specific resources: in production check location value
            resources.push_back(CrisisResource {
                name: String::from_str(env, "Veterans Crisis Line"),
                contact: String::from_str(env, "988 then press 1"),
                resource_type: String::from_str(env, "Veterans Support"),
                description: String::from_str(env, "Support for veterans in crisis"),
                languages: vec![env, String::from_str(env, "English")],
            });
        }

        resources
    }

    pub fn create_safety_plan(
        env: &Env,
        patient_id: Address,
        warning_signs: Vec<String>,
        coping_strategies: Vec<String>,
        reasons_to_live: Vec<String>,
        support_contacts: Vec<EmergencyContact>,
        professional_contacts: Vec<Address>,
    ) -> Result<u64, Error> {
        let plan_id = env.ledger().timestamp() as u64;

        let plan = SafetyPlan {
            plan_id,
            patient_id,
            warning_signs,
            coping_strategies,
            reasons_to_live,
            support_contacts,
            professional_contacts,
            created_timestamp: env.ledger().timestamp(),
            last_updated: env.ledger().timestamp(),
        };

        env.storage().instance().set(&plan_id, &plan);

        env.events().publish(
            (Symbol::new(env, "safety_plan_created"),),
            (plan_id, patient_id.clone()),
        );

        Ok(plan_id)
    }

    pub fn get_patient_prevention_alerts(env: &Env, _patient_id: Address) -> Vec<PreventionAlert> {
        let key = String::from_str(env, "prevention_alerts");
        env.storage().instance().get(&key).unwrap_or(Vec::new(env))
    }

    fn execute_prevention_protocol(env: &Env, patient_id: Address, protocol_id: u64) {
        // Get the protocol
        if let Some(protocol) = env.storage().instance().get(&protocol_id) {
            let protocol: SuicidePreventionProtocol = protocol;

            // Execute intervention steps (simplified)
            for step in protocol.intervention_steps.iter() {
                env.events().publish(
                    (Symbol::new(env, "intervention_step_executed"),),
                    (patient_id.clone(), step.clone()),
                );
            }

            // Notify emergency contacts
            for contact in protocol.emergency_contacts.iter() {
                env.events().publish(
                    (Symbol::new(env, "emergency_contact_notified"),),
                    (patient_id.clone(), contact.name.clone()),
                );
            }
        }
    }

    fn update_protocol_success_rate(env: &Env, protocol_id: u64, outcome: PreventionOutcome) {
        let mut protocol: SuicidePreventionProtocol =
            env.storage().instance().get(&protocol_id).unwrap();

        // Simple success rate calculation (would be more sophisticated in reality)
        let success_increment = match outcome {
            PreventionOutcome::InterventionSuccessful => 100,
            PreventionOutcome::OngoingMonitoring => 50,
            _ => 0,
        };

        // This is a simplified calculation - in reality would track multiple outcomes
        protocol.success_rate = (protocol.success_rate + success_increment) / 2;

        env.storage().instance().set(&protocol_id, &protocol);
    }

    fn get_high_risk_protocol(_env: &Env) -> u64 {
        // Return a default high-risk protocol ID
        // In reality, this would be a predefined protocol
        1
    }

    fn get_moderate_risk_protocol(_env: &Env) -> u64 {
        2
    }

    fn get_low_risk_protocol(_env: &Env) -> u64 {
        3
    }
}

#[contracttype]
#[derive(Clone)]
pub struct SuicideRiskAssessment {
    pub patient_id: Address,
    pub risk_score: u32,
    pub risk_level: SuicideRiskLevel,
    pub risk_factors: Vec<String>,
    pub recommended_actions: Vec<String>,
    pub protocol_id: Option<u64>,
    pub assessment_timestamp: u64,
}

#[derive(Clone, Copy, PartialEq, Eq)]
#[contracttype]
pub enum SuicideRiskLevel {
    None,
    Low,
    Moderate,
    High,
}

#[contracttype]
#[derive(Clone)]
pub struct SuicideHotline {
    pub name: String,
    pub number: String,
    pub country: String,
    pub languages: Vec<String>,
    pub available_24_7: bool,
    pub crisis_text_available: bool,
    pub text_number: Option<String>,
}

#[contracttype]
#[derive(Clone)]
pub struct CrisisResource {
    pub name: String,
    pub contact: String,
    pub resource_type: String,
    pub description: String,
    pub languages: Vec<String>,
}

#[contracttype]
#[derive(Clone)]
pub struct SafetyPlan {
    pub plan_id: u64,
    pub patient_id: Address,
    pub warning_signs: Vec<String>,
    pub coping_strategies: Vec<String>,
    pub reasons_to_live: Vec<String>,
    pub support_contacts: Vec<EmergencyContact>,
    pub professional_contacts: Vec<Address>,
    pub created_timestamp: u64,
    pub last_updated: u64,
}
