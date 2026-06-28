use soroban_sdk::{testutils::Address as _, Address, BytesN, Env, String, Vec};

use crate::{
    AppointmentStatus, PatientPortalContract, PatientPortalContractClient, PortalAppointment,
};

fn setup(env: &Env) -> (PatientPortalContractClient<'_>, Address) {
    let contract_id = Address::generate(env);
    env.register_contract(&contract_id, PatientPortalContract);
    let client = PatientPortalContractClient::new(env, &contract_id);
    let admin = Address::generate(env);
    env.mock_all_auths();
    client.try_initialize(&admin).unwrap().unwrap();
    (client, admin)
}

#[test]
fn register_and_integrations() {
    let env = Env::default();
    let (client, admin) = setup(&env);
    let med = Address::generate(&env);
    let ident = Address::generate(&env);
    client
        .try_set_integration_contracts(&admin, &med, &ident)
        .unwrap()
        .unwrap();
    assert_eq!(
        client.try_get_medical_records_contract().unwrap().unwrap(),
        Some(med)
    );
    assert_eq!(
        client
            .try_get_identity_registry_contract()
            .unwrap()
            .unwrap(),
        Some(ident)
    );

    let patient = Address::generate(&env);
    client
        .try_register(
            &patient,
            &BytesN::from_array(&env, &[9u8; 32]),
            &String::from_str(&env, "en"),
        )
        .unwrap()
        .unwrap();
    let p = client.try_get_profile(&patient).unwrap().unwrap();
    assert_eq!(p.patient, patient);
    assert_eq!(p.locale, String::from_str(&env, "en"));
}

#[test]
fn appointments_and_adherence_and_export() {
    let env = Env::default();
    let (client, _admin) = setup(&env);
    let patient = Address::generate(&env);
    let provider = Address::generate(&env);
    client
        .try_register(
            &patient,
            &BytesN::from_array(&env, &[0u8; 32]),
            &String::from_str(&env, "en"),
        )
        .unwrap()
        .unwrap();

    let start: u64 = 100;
    let end: u64 = 200;
    let appt_id = client
        .try_schedule_appointment(
            &patient,
            &provider,
            &start,
            &end,
            &BytesN::from_array(&env, &[0u8; 32]),
            &String::from_str(&env, "checkup"),
        )
        .unwrap()
        .unwrap();
    let appt: PortalAppointment = client.try_get_appointment(&appt_id).unwrap().unwrap();
    assert_eq!(appt.status, AppointmentStatus::Requested);
    client
        .try_set_appointment_status(&patient, &appt_id, &AppointmentStatus::Confirmed)
        .unwrap()
        .unwrap();
    let ids = client
        .try_list_my_appointment_ids(&patient)
        .unwrap()
        .unwrap();
    assert_eq!(ids.len(), 1);

    client
        .try_log_medication_event(
            &patient,
            &String::from_str(&env, "metformin-500"),
            &300u64,
            &true,
        )
        .unwrap()
        .unwrap();
    let adh_ids = client.try_list_my_adherence_ids(&patient).unwrap().unwrap();
    assert_eq!(adh_ids.len(), 1);

    let mut ids_vec = Vec::new(&env);
    ids_vec.push_back(1u64);
    ids_vec.push_back(2u64);
    client
        .try_request_phr_export(&patient, &ids_vec, &BytesN::from_array(&env, &[7u8; 32]))
        .unwrap()
        .unwrap();
}
