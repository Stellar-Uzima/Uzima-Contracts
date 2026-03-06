use soroban_sdk::{contracttype, Address, Env, String, Vec};
use crate::{types::*, errors::Error, events::*};

pub struct CrisisInterventionManager;

impl CrisisInterventionManager {
    pub fn create_crisis_alert(
        env: &Env,
        patient_id: Address,
        alert_type: CrisisType,
        severity: CrisisSeverity,
        description: String,
        location: Option<String>,
    ) -> Result<u64, Error> {
        let alert_id = env.ledger().timestamp() as u64;

        let alert = CrisisAlert {
            alert_id,
            patient_id: patient_id.clone(),
            alert_type,
            severity,
            timestamp: env.ledger().timestamp(),
            description,
            location,
            immediate_actions_taken: Vec::new(env),
            emergency_contacts_notified: Vec::new(env),
            resolution_status: CrisisResolution::Ongoing,
            follow_up_required: true,
        };

        // Store alert
        let mut alerts: Vec<CrisisAlert> = env.storage().instance()
            .get(&patient_id)
            .unwrap_or(Vec::new(env));
        alerts.push_back(alert);
        env.storage().instance().set(&patient_id, &alerts);

        // Trigger immediate response based on severity
        Self::trigger_emergency_response(env, patient_id.clone(), alert_type, severity);

        // Emit event
        env.events().publish(
            (Symbol::new(env, "crisis_alert_created"),),
            CrisisAlertEvent {
                alert_id,
                patient_id,
                alert_type: String::from_str(env, "crisis"),
                severity: String::from_str(env, "high"),
                timestamp: env.ledger().timestamp(),
            },
        );

        Ok(alert_id)
    }

    pub fn update_crisis_resolution(
        env: &Env,
        alert_id: u64,
        patient_id: Address,
        resolution_status: CrisisResolution,
        actions_taken: Vec<String>,
        follow_up_required: bool,
    ) -> Result<(), Error> {
        let mut alerts: Vec<CrisisAlert> = env.storage().instance()
            .get(&patient_id)
            .ok_or(Error::InvalidInput)?;

        for i in 0..alerts.len() {
            let mut alert = alerts.get(i).unwrap();
            if alert.alert_id == alert_id {
                alert.resolution_status = resolution_status;
                alert.immediate_actions_taken = actions_taken;
                alert.follow_up_required = follow_up_required;

                alerts.set(i, alert);
                env.storage().instance().set(&patient_id, &alerts);

                env.events().publish(
                    (Symbol::new(env, "crisis_resolution_updated"),),
                    (alert_id, patient_id, resolution_status),
                );

                return Ok(());
            }
        }

        Err(Error::InvalidInput)
    }

    pub fn get_patient_crisis_history(
        env: &Env,
        patient_id: Address,
    ) -> Vec<CrisisAlert> {
        env.storage().instance()
            .get(&patient_id)
            .unwrap_or(Vec::new(env))
    }

    pub fn notify_emergency_contacts(
        env: &Env,
        alert_id: u64,
        patient_id: Address,
        contacts: Vec<Address>,
    ) -> Result<(), Error> {
        let mut alerts: Vec<CrisisAlert> = env.storage().instance()
            .get(&patient_id)
            .ok_or(Error::InvalidInput)?;

        for i in 0..alerts.len() {
            let mut alert = alerts.get(i).unwrap();
            if alert.alert_id == alert_id {
                alert.emergency_contacts_notified = contacts.clone();
                alerts.set(i, alert);
                env.storage().instance().set(&patient_id, &alerts);

                // In a real implementation, this would trigger actual notifications
                env.events().publish(
                    (Symbol::new(env, "emergency_contacts_notified"),),
                    (alert_id, patient_id, contacts),
                );

                return Ok(());
            }
        }

        Err(Error::InvalidInput)
    }

    pub fn assess_crisis_risk(
        env: &Env,
        patient_id: Address,
        indicators: Vec<String>,
    ) -> CrisisRiskAssessment {
        let mut risk_score = 0.0;
        let mut risk_factors = Vec::new(env);
        let mut recommended_actions = Vec::new(env);

        // Analyze risk indicators
        for indicator in indicators.iter() {
            match indicator.as_str() {
                "suicidal_ideation" => {
                    risk_score += 0.9;
                    risk_factors.push_back(String::from_str(env, "Suicidal thoughts"));
                    recommended_actions.push_back(String::from_str(env, "Immediate emergency intervention"));
                }
                "self_harm" => {
                    risk_score += 0.8;
                    risk_factors.push_back(String::from_str(env, "Self-harm risk"));
                    recommended_actions.push_back(String::from_str(env, "Crisis counseling"));
                }
                "severe_depression" => {
                    risk_score += 0.6;
                    risk_factors.push_back(String::from_str(env, "Severe depression"));
                    recommended_actions.push_back(String::from_str(env, "Professional evaluation"));
                }
                "severe_anxiety" => {
                    risk_score += 0.5;
                    risk_factors.push_back(String::from_str(env, "Severe anxiety"));
                    recommended_actions.push_back(String::from_str(env, "Anxiety management support"));
                }
                "ptsd" => {
                    risk_score += 0.7;
                    risk_factors.push_back(String::from_str(env, "PTSD symptoms"));
                    recommended_actions.push_back(String::from_str(env, "Trauma-informed care"));
                }
                _ => {
                    risk_score += 0.1;
                }
            }
        }

        // Check recent crisis history
        let crisis_history = Self::get_patient_crisis_history(env, patient_id);
        let recent_crises = crisis_history.iter()
            .filter(|alert| env.ledger().timestamp() - alert.timestamp < 30 * 24 * 60 * 60) // Last 30 days
            .count();

        if recent_crises > 0 {
            risk_score += 0.2 * recent_crises as f32;
            risk_factors.push_back(String::from_str(env, "Recent crisis history"));
        }

        // Determine risk level
        let risk_level = if risk_score >= 0.8 {
            CrisisSeverity::Critical
        } else if risk_score >= 0.6 {
            CrisisSeverity::High
        } else if risk_score >= 0.4 {
            CrisisSeverity::Moderate
        } else {
            CrisisSeverity::Low
        };

        CrisisRiskAssessment {
            risk_score,
            risk_level,
            risk_factors,
            recommended_actions,
            assessment_timestamp: env.ledger().timestamp(),
        }
    }

    fn trigger_emergency_response(
        env: &Env,
        patient_id: Address,
        alert_type: CrisisType,
        severity: CrisisSeverity,
    ) {
        match severity {
            CrisisSeverity::Critical => {
                // Immediate emergency response
                Self::activate_emergency_protocol(env, patient_id, alert_type);
            }
            CrisisSeverity::High => {
                // Urgent response within hours
                Self::schedule_urgent_response(env, patient_id);
            }
            CrisisSeverity::Moderate => {
                // Response within 24 hours
                Self::schedule_follow_up(env, patient_id);
            }
            CrisisSeverity::Low => {
                // Monitor and follow up
                Self::schedule_monitoring(env, patient_id);
            }
        }
    }

    fn activate_emergency_protocol(env: &Env, patient_id: Address, alert_type: CrisisType) {
        // Get user's emergency contacts
        let profile: UserProfile = env.storage().instance().get(&patient_id).unwrap();

        // Notify all emergency contacts
        for contact in profile.emergency_contacts.iter() {
            // In real implementation, send actual notifications
            env.events().publish(
                (Symbol::new(env, "emergency_notification_sent"),),
                (patient_id.clone(), contact.name.clone()),
            );
        }

        // Trigger suicide prevention protocol if applicable
        if alert_type == CrisisType::SuicidalIdeation {
            Self::activate_suicide_prevention(env, patient_id);
        }

        env.events().publish(
            (Symbol::new(env, "emergency_protocol_activated"),),
            (patient_id, alert_type),
        );
    }

    fn schedule_urgent_response(env: &Env, patient_id: Address) {
        env.events().publish(
            (Symbol::new(env, "urgent_response_scheduled"),),
            patient_id,
        );
    }

    fn schedule_follow_up(env: &Env, patient_id: Address) {
        env.events().publish(
            (Symbol::new(env, "follow_up_scheduled"),),
            patient_id,
        );
    }

    fn schedule_monitoring(env: &Env, patient_id: Address) {
        env.events().publish(
            (Symbol::new(env, "monitoring_scheduled"),),
            patient_id,
        );
    }

    fn activate_suicide_prevention(env: &Env, patient_id: Address) {
        // This would integrate with suicide prevention module
        env.events().publish(
            (Symbol::new(env, "suicide_prevention_activated"),),
            patient_id,
        );
    }
}

#[contracttype]
#[derive(Clone)]
pub struct CrisisRiskAssessment {
    pub risk_score: f32,
    pub risk_level: CrisisSeverity,
    pub risk_factors: Vec<String>,
    pub recommended_actions: Vec<String>,
    pub assessment_timestamp: u64,
}