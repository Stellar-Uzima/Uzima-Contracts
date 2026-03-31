#![no_std]
use soroban_sdk::{
    contract, contractimpl, contracttype, Address, Env, String, Vec, 
    symbol_short as ss,
};

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RecommendationType {
    DrugInteraction = 0,
    TreatmentOptimization = 1,
    PathwayAdjustment = 2,
    PreventativeCare = 3,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FHIRCode {
    pub system: String,
    pub code: String,
    pub display: String,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Recommendation {
    pub rec_id: String,
    pub patient_id: String,
    pub rec_type: RecommendationType,
    pub content: String,
    pub confidence_score: u32, // Represented as basis points (e.g., 9800 = 98.00%)
    pub urgency: u32,         // 0: Routine, 1: High, 2: Critical
    pub timestamp: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ClinicalGuideline {
    pub condition_code: String,
    pub recommended_action: String,
    pub evidence_level: String,
    pub min_confidence: u32,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DataKey {
    Admin,
    Oracle,
    MedicalRecordsContract,
    Guideline(String),      // Map condition code to guideline
    Interaction(Vec<String>), // Map drug code pair to severity
    Outcome(String),        // Track clinical outcomes for learning
}

#[contract]
pub struct ClinicalDecisionSupport;

#[contractimpl]
impl ClinicalDecisionSupport {
    /// Initialize the CDSS contract with necessary integration addresses.
    pub fn initialize(env: Env, admin: Address, oracle: Address, medical_records: Address) {
        if env.storage().persistent().has(&DataKey::Admin) {
            panic!("Already initialized");
        }
        env.storage().persistent().set(&DataKey::Admin, &admin);
        env.storage().persistent().set(&DataKey::Oracle, &oracle);
        env.storage().persistent().set(&DataKey::MedicalRecordsContract, &medical_records);
    }

    /// Checks for drug-drug interactions in real-time.
    /// Returns a list of alerts if interactions are found.
    pub fn check_drug_interactions(
        env: Env,
        patient_id: String,
        new_medication_code: String,
        current_medications: Vec<String>,
    ) -> Vec<Recommendation> {
        let mut alerts = Vec::new(&env);
        
        for existing_med in current_medications.iter() {
            // Create a deterministic key for the interaction pair (sorted alphabetically)
            let mut key_vec = Vec::new(&env);
            if new_medication_code < existing_med {
                key_vec.push_back(new_medication_code.clone());
                key_vec.push_back(existing_med.clone());
            } else {
                key_vec.push_back(existing_med.clone());
                key_vec.push_back(new_medication_code.clone());
            };

            if let Some(severity) = env.storage().persistent().get::<DataKey, String>(&DataKey::Interaction(key_vec)) {
                alerts.push_back(Recommendation {
                    rec_id: String::from_str(&env, "alert-interaction"),
                    patient_id: patient_id.clone(),
                    rec_type: RecommendationType::DrugInteraction,
                    content: severity,
                    confidence_score: 9999, // Static high accuracy for known interactions
                    urgency: 2, // Critical
                    timestamp: env.ledger().timestamp(),
                });
            }
        }
        alerts
    }

    /// Provides personalized treatment recommendations based on patient conditions and guidelines.
    pub fn get_treatment_recommendation(
        env: Env,
        patient_id: String,
        condition_codes: Vec<String>,
    ) -> Vec<Recommendation> {
        let mut recommendations = Vec::new(&env);

        for code in condition_codes.iter() {
            if let Some(guideline) = env.storage().persistent().get::<DataKey, ClinicalGuideline>(&DataKey::Guideline(code.clone())) {
                // Simulate AI Confidence Calculation based on historical outcome data
                let outcome_factor = Self::calculate_learning_factor(&env, &code);
                let adjusted_confidence = (9500 + outcome_factor).min(9999);

                recommendations.push_back(Recommendation {
                    rec_id: String::from_str(&env, "rec-treatment"),
                    patient_id: patient_id.clone(),
                    rec_type: RecommendationType::TreatmentOptimization,
                    content: guideline.recommended_action,
                    confidence_score: adjusted_confidence,
                    urgency: 1, // High
                    timestamp: env.ledger().timestamp(),
                });
            }
        }
        recommendations
    }

    /// Optimizes clinical pathways by suggesting adjustments based on real-time data.
    pub fn optimize_pathway(
        env: Env,
        patient_id: String,
        current_pathway_step: u32,
        vitals_trend: i32, // -1: Declining, 0: Stable, 1: Improving
    ) -> Recommendation {
        let (action, urgency, confidence) = if vitals_trend < 0 {
            (String::from_str(&env, "Escalate care: Shift to high-intensity monitoring pathway"), 2, 9750)
        } else if vitals_trend > 0 && current_pathway_step > 5 {
            (String::from_str(&env, "Optimize pathway: Evaluate for early discharge or step-down"), 0, 9600)
        } else {
            (String::from_str(&env, "Continue current pathway: Patient stable"), 0, 9900)
        };

        Recommendation {
            rec_id: String::from_str(&env, "rec-pathway"),
            patient_id,
            rec_type: RecommendationType::PathwayAdjustment,
            content: action,
            confidence_score: confidence,
            urgency,
            timestamp: env.ledger().timestamp(),
        }
    }

    /// Records clinical outcomes to enable continuous learning for the CDSS AI.
    pub fn record_outcome(
        env: Env,
        condition_code: String,
        was_successful: bool,
    ) {
        let key = DataKey::Outcome(condition_code);
        let mut stats = env.storage().persistent().get::<DataKey, (u32, u32)>(&key).unwrap_or((0, 0));
        
        stats.0 += 1; // Total attempts
        if was_successful {
            stats.1 += 1; // Successes
        }
        
        env.storage().persistent().set(&key, &stats);
        
        env.events().publish(
            (ss!("cdss"), ss!("learning_update")),
            (condition_code, was_successful, stats)
        );
    }

    /// Administrative function to update medical guidelines from the Oracle.
    pub fn update_guideline(env: Env, oracle: Address, guideline: ClinicalGuideline) {
        oracle.require_auth();
        let stored_oracle = env.storage().persistent().get::<DataKey, Address>(&DataKey::Oracle).unwrap();
        if oracle != stored_oracle {
            panic!("Unauthorized oracle");
        }

        env.storage().persistent().set(&DataKey::Guideline(guideline.condition_code.clone()), &guideline);
    }

    /// Administrative function to update the drug interaction database.
    pub fn set_interaction(env: Env, admin: Address, drug_a: String, drug_b: String, severity: String) {
        admin.require_auth();
        let stored_admin = env.storage().persistent().get::<DataKey, Address>(&DataKey::Admin).unwrap();
        if admin != stored_admin {
            panic!("Unauthorized admin");
        }

        let mut key_vec = Vec::new(&env);
        if drug_a < drug_b {
            key_vec.push_back(drug_a);
            key_vec.push_back(drug_b);
        } else {
            key_vec.push_back(drug_b);
            key_vec.push_back(drug_a);
        };

        env.storage().persistent().set(&DataKey::Interaction(key_vec), &severity);
    }

    // Internal helper to calculate the "Learning Factor" from historical outcomes.
    fn calculate_learning_factor(env: &Env, condition_code: &String) -> u32 {
        match env.storage().persistent().get::<DataKey, (u32, u32)>(&DataKey::Outcome(condition_code.clone())) {
            Some((total, success)) if total > 10 => {
                let success_rate = (success * 10000) / total;
                if success_rate > 9000 {
                    200 // Add 2% confidence
                } else if success_rate < 5000 {
                    0 // No confidence boost for poorly performing guidelines
                } else {
                    100 // Add 1% confidence
                }
            },
            _ => 0, // Not enough data to learn yet
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use soroban_sdk::testutils::Address as _;

    #[test]
    fn test_drug_interaction_alert() {
        let env = Env::default();
        let contract_id = env.register_contract(None, ClinicalDecisionSupport);
        let client = ClinicalDecisionSupportClient::new(&env, &contract_id);
        
        let admin = Address::generate(&env);
        let oracle = Address::generate(&env);
        let medical_records = Address::generate(&env);
        
        client.initialize(&admin, &oracle, &medical_records);

        let drug_a = String::from_str(&env, "Rx001");
        let drug_b = String::from_str(&env, "Rx002");
        let severity = String::from_str(&env, "Critical: Risk of Serotonin Syndrome");
        
        // Setup interaction database
        env.mock_all_auths();
        client.set_interaction(&admin, &drug_a, &drug_b, &severity);

        let mut current_meds = Vec::new(&env);
        current_meds.push_back(drug_a.clone());

        let alerts = client.check_drug_interactions(&String::from_str(&env, "P1"), &drug_b, &current_meds);
        
        assert_eq!(alerts.len(), 1);
        let alert = alerts.get(0).unwrap();
        assert_eq!(alert.rec_type, RecommendationType::DrugInteraction);
        assert_eq!(alert.content, severity);
    }
}