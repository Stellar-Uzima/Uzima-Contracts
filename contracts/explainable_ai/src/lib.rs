#![no_std]

use soroban_sdk::{
    contract, contractimpl, contracttype, symbol_short, Address, BytesN, Env, Map, String, Symbol,
    Vec,
};

#[derive(Clone)]
#[contracttype]
pub struct ExplanationRequest {
    pub request_id: u64,
    pub requester: Address,
    pub model_id: BytesN<32>,
    pub input_data_hash: BytesN<32>,
    pub created_at: u64,
    pub status: ExplanationStatus,
    pub result_ipfs_hash: String,
}

#[derive(Clone, Copy, PartialEq, Debug, Eq)] // FIXED: Added PartialEq and Debug
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
    pub auditor: Address,
    pub timestamp: u64,
    pub audit_summary_hash: BytesN<32>, // IPFS hash of report
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
        env.storage().persistent().set(&ADMIN, &admin);
        env.storage().persistent().set(&REQ_COUNT, &0u64);
        env.storage().persistent().set(&AUDIT_COUNT, &0u64);
    }

    pub fn request_explanation(
        env: Env,
        requester: Address,
        model_id: BytesN<32>,
        input_data_hash: BytesN<32>,
    ) -> u64 {
        requester.require_auth();

        let req_id = env.storage().persistent().get(&REQ_COUNT).unwrap_or(0u64) + 1;
        env.storage().persistent().set(&REQ_COUNT, &req_id);

        let request = ExplanationRequest {
            request_id: req_id,
            requester,
            model_id,
            input_data_hash,
            created_at: env.ledger().timestamp(),
            status: ExplanationStatus::Pending,
            result_ipfs_hash: String::from_str(&env, ""),
        };

        let mut requests: Map<u64, ExplanationRequest> = env
            .storage()
            .persistent()
            .get(&REQUESTS)
            .unwrap_or(Map::new(&env));
        requests.set(req_id, request);
        env.storage().persistent().set(&REQUESTS, &requests);

        req_id
    }

    pub fn fulfill_explanation_request(
        env: Env,
        admin: Address,
        request_id: u64,
        result_ipfs_hash: String,
    ) {
        admin.require_auth();
        // Check admin
        let stored_admin: Address = env.storage().persistent().get(&ADMIN).unwrap();
        if admin != stored_admin {
            panic!("Not authorized");
        }

        let mut requests: Map<u64, ExplanationRequest> = env
            .storage()
            .persistent()
            .get(&REQUESTS)
            .unwrap_or(Map::new(&env));

        let mut req = requests.get(request_id).expect("Request not found");
        req.status = ExplanationStatus::Completed;
        req.result_ipfs_hash = result_ipfs_hash;

        requests.set(request_id, req);
        env.storage().persistent().set(&REQUESTS, &requests);
    }

    pub fn get_request(env: Env, request_id: u64) -> ExplanationRequest {
        let requests: Map<u64, ExplanationRequest> = env
            .storage()
            .persistent()
            .get(&REQUESTS)
            .unwrap_or(Map::new(&env));
        requests.get(request_id).expect("Request not found")
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

        let audit = BiasAudit {
            audit_id,
            model_id,
            auditor,
            timestamp: env.ledger().timestamp(),
            audit_summary_hash,
            recommendations,
        };

        let mut audits: Map<u64, BiasAudit> = env
            .storage()
            .persistent()
            .get(&AUDITS)
            .unwrap_or(Map::new(&env));
        audits.set(audit_id, audit);
        env.storage().persistent().set(&AUDITS, &audits);

        audit_id
    }

    // New Function: Run Fairness Metrics (Simulated)
    // Returns a tuple of (Demographic Parity Diff, Equal Opportunity Diff, Calibration Diff) scaled by 10000
    pub fn run_fairness_metrics(
        _env: Env,
        _admin: Address,
        _model_id: BytesN<32>,
        _dataset_hash: BytesN<32>,
    ) -> (u32, u32, u32) {
        // In a real system, this would trigger an off-chain oracle or complex computation.
        // Here we simulate returning "good" fairness metrics.
        // 0 means perfect fairness. 1000 = 0.1 difference.

        (500, 200, 100) // Simulated values
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use soroban_sdk::testutils::Address as _;
    use soroban_sdk::{vec, BytesN, Env}; // FIXED: Added vec import

    #[test]
    fn test_explanation_flow() {
        let env = Env::default();
        env.mock_all_auths();

        let contract_id = env.register_contract(None, ExplainableAIContract);
        let client = ExplainableAIContractClient::new(&env, &contract_id);

        let admin = Address::generate(&env);
        let requester = Address::generate(&env);
        client.initialize(&admin);

        let model_id = BytesN::from_array(&env, &[1u8; 32]);
        let input_hash = BytesN::from_array(&env, &[2u8; 32]);

        // Request
        // FIXED: Removed .unwrap()
        let req_id = client.request_explanation(&requester, &model_id, &input_hash);
        assert_eq!(req_id, 1);

        let request = client.get_request(&req_id);
        assert_eq!(request.status, ExplanationStatus::Pending);

        // Fulfill
        let ipfs_hash = String::from_str(&env, "QmHash");
        // FIXED: Removed .is_ok()
        client.fulfill_explanation_request(&admin, &req_id, &ipfs_hash);

        let updated_request = client.get_request(&req_id);
        assert_eq!(updated_request.status, ExplanationStatus::Completed);
        assert_eq!(updated_request.result_ipfs_hash, ipfs_hash);
    }

    #[test]
    fn test_bias_audit() {
        let env = Env::default();
        env.mock_all_auths();

        let contract_id = env.register_contract(None, ExplainableAIContract);
        let client = ExplainableAIContractClient::new(&env, &contract_id);

        let admin = Address::generate(&env);
        client.initialize(&admin); // Initialize first

        let auditor = Address::generate(&env);
        let model_id = BytesN::from_array(&env, &[3u8; 32]);
        let audit_summary = BytesN::from_array(&env, &[4u8; 32]);
        let recommendations = vec![&env, String::from_str(&env, "Fix bias")];

        // FIXED: Removed .unwrap()
        let audit_id =
            client.submit_bias_audit(&auditor, &model_id, &audit_summary, &recommendations);
        assert_eq!(audit_id, 1);
    }

    #[test]
    fn test_fairness_metrics() {
        let env = Env::default();
        env.mock_all_auths();

        let contract_id = env.register_contract(None, ExplainableAIContract);
        let client = ExplainableAIContractClient::new(&env, &contract_id);
        let admin = Address::generate(&env);

        let model_id = BytesN::from_array(&env, &[5u8; 32]);
        let dataset_hash = BytesN::from_array(&env, &[6u8; 32]);

        // FIXED: Removed .unwrap()
        let (dp, eo, cal) = client.run_fairness_metrics(&admin, &model_id, &dataset_hash);

        assert_eq!(dp, 500);
        assert_eq!(eo, 200);
        assert_eq!(cal, 100);
    }
}
