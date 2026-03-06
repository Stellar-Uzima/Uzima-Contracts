#![cfg(test)]

use soroban_sdk::{
    testutils::{Address as _, Events},
    Address, Env, IntoVal, String, Vec,
};
use crate::{
    types::*,
    MentalHealthPlatform, MentalHealthPlatformClient,
};

#[test]
fn test_initialization() {
    let env = Env::default();
    let admin = Address::generate(&env);

    let contract_id = env.register_contract(None, MentalHealthPlatform);
    let client = MentalHealthPlatformClient::new(&env, &contract_id);

    client.initialize(&admin);

    // Check that initialization event was emitted
    let events = env.events().all();
    assert_eq!(events.len(), 1);
}

#[test]
fn test_user_registration() {
    let env = Env::default();
    let admin = Address::generate(&env);
    let user = Address::generate(&env);

    let contract_id = env.register_contract(None, MentalHealthPlatform);
    let client = MentalHealthPlatformClient::new(&env, &contract_id);

    client.initialize(&admin);
    client.register_user(&user, &UserType::Patient, &true);

    // Check that user registration event was emitted
    let events = env.events().all();
    assert_eq!(events.len(), 2); // init + registration
}

#[test]
fn test_mood_tracking() {
    let env = Env::default();
    let admin = Address::generate(&env);
    let patient = Address::generate(&env);

    let contract_id = env.register_contract(None, MentalHealthPlatform);
    let client = MentalHealthPlatformClient::new(&env, &contract_id);

    client.initialize(&admin);
    client.register_user(&patient, &UserType::Patient, &true);

    let emotions = Vec::from_array(&env, [String::from_str(&env, "sad")]);
    let triggers = Vec::from_array(&env, [String::from_str(&env, "stress")]);

    let mood_id = client.record_mood(
        &patient,
        &-3,
        &emotions,
        &triggers,
        &String::from_str(&env, "Feeling down"),
        &Some(String::from_str(&env, "home")),
    );

    assert_eq!(mood_id, 1); // First mood entry should have ID 1
}

#[test]
fn test_assessment_creation() {
    let env = Env::default();
    let admin = Address::generate(&env);
    let patient = Address::generate(&env);
    let therapist = Address::generate(&env);

    let contract_id = env.register_contract(None, MentalHealthPlatform);
    let client = MentalHealthPlatformClient::new(&env, &contract_id);

    client.initialize(&admin);
    client.register_user(&patient, &UserType::Patient, &true);
    client.register_user(&therapist, &UserType::MentalHealthProfessional, &true);

    let assessment_id = client.create_assessment(
        &patient,
        &AssessmentType::PHQ9,
        &therapist,
    );

    assert_eq!(assessment_id, 1); // First assessment should have ID 1
}

#[test]
fn test_crisis_alert() {
    let env = Env::default();
    let admin = Address::generate(&env);
    let patient = Address::generate(&env);

    let contract_id = env.register_contract(None, MentalHealthPlatform);
    let client = MentalHealthPlatformClient::new(&env, &contract_id);

    client.initialize(&admin);
    client.register_user(&patient, &UserType::Patient, &true);

    let alert_id = client.create_crisis_alert(
        &patient,
        &CrisisType::SevereDepression,
        &CrisisSeverity::High,
        &String::from_str(&env, "Patient reporting severe depressive symptoms"),
        &Some(String::from_str(&env, "home")),
    );

    assert_eq!(alert_id, 1); // First alert should have ID 1
}

#[test]
fn test_medication_plan() {
    let env = Env::default();
    let admin = Address::generate(&env);
    let patient = Address::generate(&env);
    let doctor = Address::generate(&env);

    let contract_id = env.register_contract(None, MentalHealthPlatform);
    let client = MentalHealthPlatformClient::new(&env, &contract_id);

    client.initialize(&admin);
    client.register_user(&patient, &UserType::Patient, &true);
    client.register_user(&doctor, &UserType::MentalHealthProfessional, &true);

    let side_effects = Vec::from_array(&env, [String::from_str(&env, "drowsiness")]);

    let plan_id = client.create_medication_plan(
        &patient,
        &String::from_str(&env, "Sertraline"),
        &String::from_str(&env, "50mg"),
        &String::from_str(&env, "Once daily"),
        &env.ledger().timestamp(),
        &Some(env.ledger().timestamp() + 90 * 24 * 60 * 60), // 90 days
        &doctor,
        &side_effects,
    );

    assert_eq!(plan_id, 1); // First plan should have ID 1
}

#[test]
fn test_peer_group() {
    let env = Env::default();
    let admin = Address::generate(&env);
    let moderator = Address::generate(&env);
    let user = Address::generate(&env);

    let contract_id = env.register_contract(None, MentalHealthPlatform);
    let client = MentalHealthPlatformClient::new(&env, &contract_id);

    client.initialize(&admin);
    client.register_user(&moderator, &UserType::MentalHealthProfessional, &true);
    client.register_user(&user, &UserType::Patient, &true);

    let group_id = String::from_str(&env, "anxiety_support");

    client.create_peer_group(
        &group_id,
        &String::from_str(&env, "Anxiety Support Group"),
        &String::from_str(&env, "Support for anxiety-related concerns"),
        &String::from_str(&env, "Anxiety"),
        &moderator,
        &20,
        &GroupPrivacy::Private,
    );

    client.join_peer_group(&group_id, &user);

    // Test messaging
    let message_id = client.post_peer_message(
        &group_id,
        &user,
        &String::from_str(&env, "Hello everyone, looking for support"),
        &MessageType::Text,
    );

    assert_eq!(message_id, 1); // First message should have ID 1
}

#[test]
fn test_suicide_prevention() {
    let env = Env::default();
    let admin = Address::generate(&env);
    let patient = Address::generate(&env);

    let contract_id = env.register_contract(None, MentalHealthPlatform);
    let client = MentalHealthPlatformClient::new(&env, &contract_id);

    client.initialize(&admin);
    client.register_user(&patient, &UserType::Patient, &true);

    let indicators = Vec::from_array(&env, [String::from_str(&env, "suicidal_ideation")]);
    let context_data = soroban_sdk::Map::new(&env);

    let risk_assessment = client.detect_suicide_risk(&patient, &indicators, &context_data);

    assert!(risk_assessment.risk_score > 0.8); // Should detect high risk
    assert_eq!(risk_assessment.risk_level, SuicideRiskLevel::High);
}