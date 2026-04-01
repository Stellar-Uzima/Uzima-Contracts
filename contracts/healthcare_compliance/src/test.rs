extern crate std;

use soroban_sdk::Env;

#[test]
fn test_placeholder() {
    let _env = Env::default();
}

use crate::HealthcareComplianceContractClient;
use soroban_sdk::{Address, Env, BytesN, String, Vec};

fn setup_contract(env: &Env) -> (HealthcareComplianceContractClient<'_>, Address) {
    env.mock_all_auths();
    let contract_id = env.register_contract(None, crate::HealthcareComplianceContract);
    let client = HealthcareComplianceContractClient::new(env, &contract_id);

    let admin = Address::generate(env);
    client.initialize(&admin);
    (client, admin)
}

#[test]
fn test_submit_and_get_compliance_report() {
    let env = Env::default();
    let (client, _admin) = setup_contract(&env);

    let reporter = Address::generate(&env);
    let report_id = String::from_str(&env, "report-1");
    let report_hash = BytesN::from_array(&env, &[1u8; 32]);
    let uri = String::from_str(&env, "ipfs://report-1");

    let r = client.submit_compliance_report(&reporter, &report_id, &report_hash, &uri);
    assert_eq!(r, ());

    let rec = client.get_compliance_report(&report_id).expect("report should exist");
    assert_eq!(rec.report_id, report_id);
    assert_eq!(rec.reporter, reporter);
    assert_eq!(rec.report_hash, report_hash);
    assert_eq!(rec.uri, uri);
}
