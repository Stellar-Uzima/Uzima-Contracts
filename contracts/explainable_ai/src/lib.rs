// Explainable AI Contract - XAI explanations and bias auditing
#![no_std]
#![allow(clippy::too_many_arguments)]
#![allow(clippy::arithmetic_side_effects)]
#![allow(clippy::panic)]
#![allow(clippy::unwrap_used)]
#![allow(dead_code)]

use soroban_sdk::{
    contract, contractimpl, contracttype, symbol_short, Address, BytesN, Env, Map, String, Symbol,
    Vec,
};

#[derive(Clone)]
#[contracttype]
pub struct ExplanationRequest {
    pub request_id: u64,
    pub patient: Address,
    pub ai_insight_id: u64,
    pub requested_at: u64,
    pub fulfilled_at: Option<u64>,
    pub explanation_ref: Option<String>,
    pub status: ExplanationStatus,
}

#[derive(Clone)]
#[contracttype]
pub struct FeatureImportance {
    pub feature_name: String,
    pub importance_bps: u32,   // Importance in basis points (0-10000)
    pub normalized_value: u32, // Normalized value for this feature (0-10000)
}

#[derive(Clone)]
#[contracttype]
pub struct ExplanationMetadata {
    pub insight_id: u64,
    pub model_id: BytesN<32>,
    pub patient: Address,
    pub explanation_method: String, // e.g., "SHAP", "LIME", "attention_weights"
    pub feature_importance: Vec<FeatureImportance>,
    pub primary_factors: Vec<String>, // Top contributing factors
    pub confidence_impact: u32,       // How much this factor impacted confidence (in bps)
    pub created_at: u64,
    pub explanation_ref: String, // Off-chain reference to detailed explanation
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[contracttype]
pub enum ExplanationStatus {
    Pending,
    Processing,
    Completed,
    Failed,
}

#[derive(Clone)]
#[contracttype]
pub struct BiasAudit {
    pub audit_id: u64,
    pub model_id: BytesN<32>,
    pub audit_date: u64,
    pub demographic_fairness_metrics: Map<String, u32>, // Group -> disparity metric
    pub equalized_odds: bool,
    pub calibration_by_group: Map<String, u32>, // Group -> calibration metric
    pub audit_summary: String,
    pub recommendations: Vec<String>,
}

const REQUESTS: Symbol = symbol_short!("REQUESTS");
const AUDITS: Symbol = symbol_short!("AUDITS");
const ADMIN: Symbol = symbol_short!("ADMIN");
const REQ_COUNT: Symbol = symbol_short!("REQ_CNT");
const AUDIT_COUNT: Symbol = symbol_short!("AUD_CNT");

#[contract]
pub struct ExplainableAIContract;

#[contractimpl]
impl ExplainableAIContract {
    pub fn initialize(env: Env, admin: Address) {
        if env.storage().persistent().has(&ADMIN) {
            panic!("Already initialized");
        }

        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage().instance().set(&REQUEST_COUNTER, &0u64);
        env.storage().instance().set(&EXPLANATION_COUNTER, &0u64);
        env.storage().instance().set(&AUDIT_COUNTER, &0u64);
        true
    }

    fn ensure_admin(env: &Env, caller: &Address) {
        let admin: Address = env
            .storage()
            .instance()
            .get(&DataKey::Admin)
            .unwrap_or_else(|| panic!("Explainable AI admin not set"));

        if admin != *caller {
            panic!("Not authorized: caller is not admin");
        }
    }

    fn next_request_id(env: &Env) -> u64 {
        let current: u64 = env.storage().instance().get(&REQUEST_COUNTER).unwrap_or(0);
        let next = current + 1;
        env.storage().instance().set(&REQUEST_COUNTER, &next);
        next
    }

    fn next_explanation_id(env: &Env) -> u64 {
        let current: u64 = env
            .storage()
            .instance()
            .get(&EXPLANATION_COUNTER)
            .unwrap_or(0);
        let next = current + 1;
        env.storage().instance().set(&EXPLANATION_COUNTER, &next);
        next
    }

    fn next_audit_id(env: &Env) -> u64 {
        let current: u64 = env.storage().instance().get(&AUDIT_COUNTER).unwrap_or(0);
        let next = current + 1;
        env.storage().instance().set(&AUDIT_COUNTER, &next);
        next
    }

    pub fn request_explanation(env: Env, caller: Address, ai_insight_id: u64) -> u64 {
        caller.require_auth();

        // Only patient, admin, or authorized healthcare provider can request explanation
        // For simplicity in this example, we'll just allow anyone to request
        // In a real implementation, access controls would be more restrictive

        let request_id = Self::next_request_id(&env);
        let timestamp = env.ledger().timestamp();

        let request = ExplanationRequest {
            request_id: req_id,
            requester,
            model_id,
            input_data_hash,
            created_at: env.ledger().timestamp(),
            status: ExplanationStatus::Pending,
            result_ipfs_hash: String::from_str(&env, ""),
        };

        env.storage()
            .instance()
            .set(&DataKey::Request(request_id), &request);

        env.events().publish(
            (symbol_short!("ExpReq"),),
            (request_id, ai_insight_id, caller),
        );

        req_id
    }

    pub fn fulfill_explanation_request(
        env: Env,
        admin: Address,
        request_id: u64,
        model_id: BytesN<32>,
        explanation_method: String,
        feature_importance: Vec<FeatureImportance>,
        primary_factors: Vec<String>,
        confidence_impact: u32,
        explanation_ref: String,
    ) -> Result<bool, Error> {
        caller.require_auth();
        Self::ensure_admin(&env, &caller);

        let mut request: ExplanationRequest = env
            .storage()
            .instance()
            .get(&DataKey::Request(request_id))
            .ok_or(Error::RequestNotFound)?;

        // Validate feature importance values
        for feature in feature_importance.iter() {
            if feature.importance_bps > 10_000 {
                return Err(Error::InvalidImportance);
            }
            if feature.normalized_value > 10_000 {
                return Err(Error::InvalidBPSValue);
            }
        }

        // Validate confidence impact
        if confidence_impact > 10_000 {
            return Err(Error::InvalidBPSValue);
        }

        let explanation_id = Self::next_explanation_id(&env);
        let timestamp = env.ledger().timestamp();

        // Create explanation metadata
        let explanation = ExplanationMetadata {
            insight_id: request.ai_insight_id,
            model_id,
            patient: request.patient.clone(),
            explanation_method,
            feature_importance,
            primary_factors,
            confidence_impact,
            created_at: timestamp,
            explanation_ref,
        };

        // Save explanation
        env.storage()
            .instance()
            .set(&DataKey::Explanation(explanation_id), &explanation);

        // Update request status
        request.status = ExplanationStatus::Completed;
        request.fulfilled_at = Some(timestamp);
        request.explanation_ref = Some(explanation.explanation_ref.clone());

        env.storage()
            .instance()
            .set(&DataKey::Request(request_id), &request);

        env.events().publish(
            (symbol_short!("ExpFull"),),
            (request_id, explanation_id, request.patient),
        );

        Ok(true)
    }

    pub fn get_explanation_request(env: Env, request_id: u64) -> Option<ExplanationRequest> {
        env.storage().instance().get(&DataKey::Request(request_id))
    }

    pub fn get_explanation(env: Env, explanation_id: u64) -> Option<ExplanationMetadata> {
        env.storage()
            .instance()
            .get(&DataKey::Explanation(explanation_id))
    }

    pub fn get_explanations_for_patient(
        env: Env,
        _patient: Address,
        _page: u32,
        _page_size: u32,
    ) -> Vec<ExplanationMetadata> {
        // This is a simplified implementation
        // In a real contract, we'd need a way to track explanations by patient
        // For now, we'll return an empty vector
        Vec::new(&env)
    }

    pub fn submit_bias_audit(
        env: Env,
        auditor: Address,
        model_id: BytesN<32>,
        audit_summary_hash: BytesN<32>,
        recommendations: Vec<String>,
    ) -> u64 {
        auditor.require_auth();

        let audit_id = env.storage().persistent().get(&AUDIT_COUNT).unwrap_or(0u64) + 1;
        env.storage().persistent().set(&AUDIT_COUNT, &audit_id);

        let mut calibration_by_group: Map<String, u32> = Map::new(&env);
        calibration_by_group.set(String::from_str(&env, "age_young"), 9700u32);
        calibration_by_group.set(String::from_str(&env, "age_middle"), 9550u32);
        calibration_by_group.set(String::from_str(&env, "age_elderly"), 9400u32);

        let audit_result = BiasAuditResult {
            model_id: model_id.clone(),
            audit_date: timestamp,
            demographic_fairness_metrics: demographic_fairness,
            equalized_odds: false, // Simplified for example
            calibration_by_group,
            audit_summary,
            recommendations,
        };

        env.storage()
            .instance()
            .set(&DataKey::BiasAudit(model_id.clone()), &audit_result);

        env.events()
            .publish((symbol_short!("BiasAudit"),), (audit_id, model_id));

        Ok(audit_id)
    }

    pub fn get_bias_audit(env: Env, model_id: BytesN<32>) -> Option<BiasAuditResult> {
        env.storage().instance().get(&DataKey::BiasAudit(model_id))
    }

    pub fn run_fairness_metrics(
        env: Env,
        caller: Address,
        _model_id: BytesN<32>,
        _protected_attribute: String,
        _privileged_group: String,
        _unprivileged_group: String,
    ) -> Result<(u32, u32, u32), Error> {
        // Returns (demographic_parity_diff, equalized_odds_diff, calibration_diff)
        caller.require_auth();
        Self::ensure_admin(&env, &caller);

        // Simulate calculation of fairness metrics
        // In a real implementation, this would analyze model predictions across groups
        let demographic_parity_diff = 150u32; // Difference in positive prediction rates (in bps)
        let equalized_odds_diff = 200u32; // Difference in true positive rates (in bps)
        let calibration_diff = 100u32; // Difference in calibration (in bps)

        Ok((
            demographic_parity_diff,
            equalized_odds_diff,
            calibration_diff,
        ))
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use soroban_sdk::testutils::Address as _;
    use soroban_sdk::vec;

    #[test]
    fn test_explanation_flow() {
        let env = Env::default();
        env.mock_all_auths();

        let contract_id = env.register_contract(None, ExplainableAIContract);
        let client = ExplainableAIContractClient::new(&env, &contract_id);

        let admin = Address::generate(&env);
        let patient = Address::generate(&env);

        client.mock_all_auths().initialize(&admin);

        // Request an explanation
        let request_id = client
            .mock_all_auths()
            .request_explanation(&patient, &123u64);
        assert_eq!(request_id, 1u64);

        let request = client.get_request(&req_id);
        assert_eq!(request.status, ExplanationStatus::Pending);

        // Fulfill the explanation request
        let model_id = BytesN::from_array(&env, &[1; 32]);
        let explanation_method = String::from_str(&env, "SHAP");

        let feature_importance = vec![
            &env,
            FeatureImportance {
                feature_name: String::from_str(&env, "age"),
                importance_bps: 8000u32,
                normalized_value: 7500u32,
            },
            FeatureImportance {
                feature_name: String::from_str(&env, "bmi"),
                importance_bps: 6500u32,
                normalized_value: 8200u32,
            },
        ];

        let primary_factors = vec![
            &env,
            String::from_str(&env, "age"),
            String::from_str(&env, "bmi"),
        ];

        let explanation_ref = String::from_str(&env, "ipfs://explanation-details-123");

        assert!(client.mock_all_auths().fulfill_explanation_request(
            &admin,
            &request_id,
            &model_id,
            &explanation_method,
            &feature_importance,
            &primary_factors,
            &5000u32,
            &explanation_ref,
        ));

        // Verify the request is now completed
        let updated_request = client.get_explanation_request(&request_id).unwrap();
        assert_eq!(updated_request.status, ExplanationStatus::Completed);
        assert!(updated_request.fulfilled_at.is_some());

        // Get the explanation
        let explanation = client.get_explanation(&1u64).unwrap(); // First explanation
        assert_eq!(explanation.model_id, model_id);
        assert_eq!(explanation.patient, patient);
        assert_eq!(explanation.explanation_method, explanation_method);
        assert_eq!(explanation.feature_importance.len(), 2);
    }

    #[test]
    fn test_bias_audit() {
        let env = Env::default();
        env.mock_all_auths();

        let contract_id = env.register_contract(None, ExplainableAIContract);
        let client = ExplainableAIContractClient::new(&env, &contract_id);

        let admin = Address::generate(&env);
        client.initialize(&admin); // Initialize first

        client.mock_all_auths().initialize(&admin);

        // Submit a bias audit
        let audit_summary = String::from_str(&env, "Initial bias audit for model v1.0");
        let recommendations = vec![
            &env,
            String::from_str(&env, "Collect more diverse training data"),
            String::from_str(&env, "Adjust model weights for underrepresented groups"),
        ];

        let audit_id = client.mock_all_auths().submit_bias_audit(
            &admin,
            &model_id,
            &audit_summary,
            &recommendations,
        );

        assert_eq!(audit_id, 1u64);

        // Get the bias audit
        let audit = client.get_bias_audit(&model_id).unwrap();
        assert_eq!(audit.model_id, model_id);
        assert_eq!(audit.audit_summary, audit_summary);
        assert_eq!(audit.recommendations.len(), 2);

        // Run fairness metrics
        let (dp_diff, eo_diff, cal_diff) = client.mock_all_auths().run_fairness_metrics(
            &admin,
            &model_id,
            &String::from_str(&env, "gender"),
            &String::from_str(&env, "male"),
            &String::from_str(&env, "female"),
        );

        assert_eq!(dp, 500);
        assert_eq!(eo, 200);
        assert_eq!(cal, 100);
    }
}
