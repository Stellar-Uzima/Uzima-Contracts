use soroban_sdk::{testutils::Address as TestAddress, Address, Env, String};

use contract_registry::{ContractRegistry, ContractRegistryClient};
use medical_records::{MedicalRecordsContract, MedicalRecordsContractClient};
use mental_health_support::{MentalHealthSupportContract, MentalHealthSupportContractClient};
use patient_portal::{PatientPortalContract, PatientPortalContractClient};

fn generate_test_address(env: &Env) -> Address {
    <Address as TestAddress>::generate(env)
}

#[test]
fn test_registry_driven_discovery_across_contracts() {
    let env = Env::default();
    env.ledger().set_protocol_version(1);

    let admin = generate_test_address(&env);
    let registry_id = env.register_contract(None, ContractRegistry);
    let registry_client = ContractRegistryClient::new(&env, &registry_id);
    registry_client.initialize(&admin);

    let rbac_address = generate_test_address(&env);
    let identity_address = generate_test_address(&env);
    let telemedicine_address = generate_test_address(&env);
    let notification_address = generate_test_address(&env);

    registry_client.register_contract(&admin, String::from_str(&env, "rbac"), rbac_address).unwrap();
    registry_client.register_contract(&admin, String::from_str(&env, "identity_registry"), identity_address).unwrap();
    registry_client.register_contract(&admin, String::from_str(&env, "telemedicine"), telemedicine_address).unwrap();
    registry_client.register_contract(&admin, String::from_str(&env, "notification_system"), notification_address).unwrap();

    let medical_records_id = env.register_contract(None, MedicalRecordsContract);
    let medical_records_client = MedicalRecordsContractClient::new(&env, &medical_records_id);
    medical_records_client.initialize(&admin, rbac_address);
    medical_records_client.set_contract_registry(&admin, registry_id).unwrap();

    let portal_id = env.register_contract(None, PatientPortalContract);
    let portal_client = PatientPortalContractClient::new(&env, &portal_id);
    portal_client.initialize(&admin);
    portal_client.set_contract_registry(&admin, registry_id).unwrap();

    let mh_id = env.register_contract(None, MentalHealthSupportContract);
    let mh_client = MentalHealthSupportContractClient::new(&env, &mh_id);
    mh_client.initialize(&admin).unwrap();
    mh_client.set_contract_registry(&admin, registry_id).unwrap();

    assert_eq!(medical_records_client.get_identity_registry(), identity_address);
    assert_eq!(portal_client.get_medical_records_contract().unwrap().unwrap(), rbac_address);
    assert_eq!(portal_client.get_identity_registry_contract().unwrap().unwrap(), identity_address);
    assert_eq!(mh_client.get_telemedicine_contract().unwrap().unwrap(), telemedicine_address);
    assert_eq!(mh_client.get_notification_contract().unwrap().unwrap(), notification_address);
}
