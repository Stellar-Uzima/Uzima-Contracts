use crate::utils::IntegrationTestEnv;
use medical_records::Role;
use soroban_sdk::{vec, String, Env, Address};
use serde_json::Value;

#[test]
fn test_record_created_event_schema() {
    let test_env = IntegrationTestEnv::new();
    let env = &test_env.env;

    let (records_id, records_client) = test_env.register_medical_records();

    let admin = &test_env.team.admin.address;
    let doctor = &test_env.team.doctors[0].address;
    let patient = &test_env.team.patients[0].address;

    records_client.initialize(admin);
    records_client.manage_user(admin, doctor, &Role::Doctor);
    records_client.manage_user(admin, patient, &Role::Patient);

    let diagnosis = String::from_str(env, "Hypertension");
    let treatment = String::from_str(env, "ACE inhibitor");

    let record_id = records_client.add_record(
        doctor,
        patient,
        &diagnosis,
        &treatment,
        &false,
        &vec![env, String::from_str(env, "herbal")],
        &String::from_str(env, "Traditional"),
        &String::from_str(env, "Herbal Therapy"),
        &String::from_str(env, "QmHash"),
    );

    let events = test_env.get_events();
    let event = events.iter().find(|(id, topics, _)| {
        id == &records_id && topics.contains(&test_env.topics(&["record_created"]))
    });

    assert!(event.is_some());

    let (_, _, data) = event.unwrap();
    let event_json: Value = serde_json::from_str(&data).unwrap();

    let schema_str = include_str!("../../schemas/events/record_created_event.schema.json");
    let schema: Value = serde_json::from_str(schema_str).unwrap();

    let compiled_schema = jsonschema::JSONSchema::compile(&schema).unwrap();
    let result = compiled_schema.validate(&event_json);

    assert!(result.is_ok());
}