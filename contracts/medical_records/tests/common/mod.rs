use medical_records::{MedicalRecordsContract, MedicalRecordsContractClient, Role};
use soroban_sdk::{testutils::Address as _, Address, Env};

// Added <'a> to struct definition
pub struct UzimaTest<'a> {
    pub client: MedicalRecordsContractClient<'a>, // Client needs lifetime
    pub admin1: Address,
    #[allow(dead_code)]
    pub admin2: Address,
    pub doctor: Address,
    pub patient: Address,
}

// Added <'a> to function and argument
pub fn setup_uzima<'a>(env: &'a Env) -> UzimaTest<'a> {
    env.mock_all_auths();

    let admin1 = Address::generate(env);
    let admin2 = Address::generate(env);
    let doctor = Address::generate(env);
    let patient = Address::generate(env);

    let contract_id = env.register_contract(None, MedicalRecordsContract);
    let client = MedicalRecordsContractClient::new(env, &contract_id);

    // Initialize with first admin
    client.initialize(&admin1);

    // Make the second admin an Admin (for multisig tests)
    client.manage_user(&admin1, &admin2, &Role::Admin);

    // IMPORTANT FIX: Add the Patient to the USERS map!
    client.manage_user(&admin1, &patient, &Role::Patient);

    // IMPORTANT FIX: Add the Doctor to the USERS map (required for link_did_to_user to find a profile)
    client.manage_user(&admin1, &doctor, &Role::Doctor);

    UzimaTest {
        client,
        admin1,
        admin2,
        doctor,
        patient,
    }
}
