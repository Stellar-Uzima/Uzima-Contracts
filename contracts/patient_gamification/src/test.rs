#![cfg(test)]

use super::*;
use soroban_sdk::{testutils::Address as _, Address, Env, String};

#[test]
fn test_initialize() {
    let env = Env::default();
    let contract_id = env.register_contract(None, PatientGamificationContract);
    let client = PatientGamificationContractClient::new(&env, &contract_id);

    let admin = Address::random(&env);

    let result = client.initialize(
        &admin,
        &100u32,  // points_per_achievement
        &50u32,   // points_per_challenge
        &10u32,   // points_per_streak_day
        &1000u32, // max_daily_points
        &5u32,    // privacy_threshold
    );

    assert!(result.is_ok());
    assert!(result.unwrap());
}

#[test]
fn test_create_achievement() {
    let env = Env::default();
    let contract_id = env.register_contract(None, PatientGamificationContract);
    let client = PatientGamificationContractClient::new(&env, &contract_id);

    let admin = Address::random(&env);

    client.initialize(
        &admin,
        &100u32,
        &50u32,
        &10u32,
        &1000u32,
        &5u32,
    );

    let achievement_id = client.create_achievement(
        &admin,
        &String::from_str(&env, "First Steps"),
        &String::from_str(&env, "Complete your first health metric"),
        &String::from_str(&env, "Onboarding"),
        &100u32,
        &String::from_str(&env, "ipfs://badge1"),
        &String::from_str(&env, "metrics_recorded"),
        &1u32,
    );

    assert!(achievement_id.is_ok());
    assert_eq!(achievement_id.unwrap(), 1);

    let achievement = client.get_achievement(&1u64);
    assert!(achievement.is_ok());
    let achievement = achievement.unwrap();
    assert_eq!(achievement.name, String::from_str(&env, "First Steps"));
    assert_eq!(achievement.points_reward, 100);
    assert!(achievement.is_active);
}

#[test]
fn test_achievement_progress() {
    let env = Env::default();
    let contract_id = env.register_contract(None, PatientGamificationContract);
    let client = PatientGamificationContractClient::new(&env, &contract_id);

    let admin = Address::random(&env);
    let patient = Address::random(&env);

    client.initialize(
        &admin,
        &100u32,
        &50u32,
        &10u32,
        &1000u32,
        &5u32,
    );

    client.create_achievement(
        &admin,
        &String::from_str(&env, "First Steps"),
        &String::from_str(&env, "Complete your first health metric"),
        &String::from_str(&env, "Onboarding"),
        &100u32,
        &String::from_str(&env, "ipfs://badge1"),
        &String::from_str(&env, "metrics_recorded"),
        &1u32,
    );

    // Update progress to complete achievement
    let result = client.update_achievement_progress(
        &patient,
        &patient,
        &1u64,
        &1u32,
    );

    assert!(result.is_ok());
    assert!(result.unwrap());

    // Check points were awarded
    let points = client.get_reward_points(&patient);
    assert_eq!(points.total_points, 100);
    assert_eq!(points.available_points, 100);
}

#[test]
fn test_create_challenge() {
    let env = Env::default();
    let contract_id = env.register_contract(None, PatientGamificationContract);
    let client = PatientGamificationContractClient::new(&env, &contract_id);

    let admin = Address::random(&env);

    client.initialize(
        &admin,
        &100u32,
        &50u32,
        &10u32,
        &1000u32,
        &5u32,
    );

    let challenge_id = client.create_challenge(
        &admin,
        &String::from_str(&env, "30 Day Step Challenge"),
        &String::from_str(&env, "Walk 10,000 steps daily for 30 days"),
        &String::from_str(&env, "steps"),
        &String::from_str(&env, "daily_steps"),
        &10000u32,
        &1000u64,  // start_time
        &2680000u64, // end_time (30 days later)
        &500u32,
        &100u32,
    );

    assert!(challenge_id.is_ok());
    assert_eq!(challenge_id.unwrap(), 1);

    let challenge = client.get_challenge(&1u64);
    assert!(challenge.is_ok());
    let challenge = challenge.unwrap();
    assert_eq!(challenge.name, String::from_str(&env, "30 Day Step Challenge"));
    assert_eq!(challenge.points_reward, 500);
    assert!(challenge.is_active);
}

#[test]
fn test_join_challenge() {
    let env = Env::default();
    let contract_id = env.register_contract(None, PatientGamificationContract);
    let client = PatientGamificationContractClient::new(&env, &contract_id);

    let admin = Address::random(&env);
    let patient = Address::random(&env);

    client.initialize(
        &admin,
        &100u32,
        &50u32,
        &10u32,
        &1000u32,
        &5u32,
    );

    client.create_challenge(
        &admin,
        &String::from_str(&env, "30 Day Step Challenge"),
        &String::from_str(&env, "Walk 10,000 steps daily for 30 days"),
        &String::from_str(&env, "steps"),
        &String::from_str(&env, "daily_steps"),
        &10000u32,
        &1000u64,
        &2680000u64,
        &500u32,
        &100u32,
    );

    let result = client.join_challenge(&patient, &1u64);
    assert!(result.is_ok());
    assert!(result.unwrap());

    let participant = client.get_challenge_participant(&1u64, &patient);
    assert!(participant.is_ok());
    let participant = participant.unwrap();
    assert_eq!(participant.patient_id, patient);
    assert!(!participant.is_completed);
}

#[test]
fn test_reward_points() {
    let env = Env::default();
    let contract_id = env.register_contract(None, PatientGamificationContract);
    let client = PatientGamificationContractClient::new(&env, &contract_id);

    let admin = Address::random(&env);
    let patient = Address::random(&env);

    client.initialize(
        &admin,
        &100u32,
        &50u32,
        &10u32,
        &1000u32,
        &5u32,
    );

    // Create and complete achievement
    client.create_achievement(
        &admin,
        &String::from_str(&env, "First Steps"),
        &String::from_str(&env, "Complete your first health metric"),
        &String::from_str(&env, "Onboarding"),
        &100u32,
        &String::from_str(&env, "ipfs://badge1"),
        &String::from_str(&env, "metrics_recorded"),
        &1u32,
    );

    client.update_achievement_progress(
        &patient,
        &patient,
        &1u64,
        &1u32,
    );

    // Check points
    let points = client.get_reward_points(&patient);
    assert_eq!(points.total_points, 100);
    assert_eq!(points.available_points, 100);

    // Redeem points
    let result = client.redeem_points(&patient, &50u64);
    assert!(result.is_ok());
    assert!(result.unwrap());

    // Check updated points
    let points = client.get_reward_points(&patient);
    assert_eq!(points.total_points, 100);
    assert_eq!(points.available_points, 50);
}

#[test]
fn test_social_profile() {
    let env = Env::default();
    let contract_id = env.register_contract(None, PatientGamificationContract);
    let client = PatientGamificationContractClient::new(&env, &contract_id);

    let admin = Address::random(&env);
    let patient = Address::random(&env);

    client.initialize(
        &admin,
        &100u32,
        &50u32,
        &10u32,
        &1000u32,
        &5u32,
    );

    let result = client.create_social_profile(
        &patient,
        &String::from_str(&env, "John Doe"),
        &String::from_str(&env, "Health enthusiast"),
        &String::from_str(&env, "ipfs://avatar1"),
        &true,
        &true,
        &true,
        &true,
    );

    assert!(result.is_ok());
    assert!(result.unwrap());

    let profile = client.get_social_profile(&patient);
    assert!(profile.is_ok());
    let profile = profile.unwrap();
    assert_eq!(profile.display_name, String::from_str(&env, "John Doe"));
    assert!(profile.is_public);
}

#[test]
fn test_leaderboard() {
    let env = Env::default();
    let contract_id = env.register_contract(None, PatientGamificationContract);
    let client = PatientGamificationContractClient::new(&env, &contract_id);

    let admin = Address::random(&env);
    let patient1 = Address::random(&env);
    let patient2 = Address::random(&env);

    client.initialize(
        &admin,
        &100u32,
        &50u32,
        &10u32,
        &1000u32,
        &5u32,
    );

    // Create achievements
    client.create_achievement(
        &admin,
        &String::from_str(&env, "First Steps"),
        &String::from_str(&env, "Complete your first health metric"),
        &String::from_str(&env, "Onboarding"),
        &100u32,
        &String::from_str(&env, "ipfs://badge1"),
        &String::from_str(&env, "metrics_recorded"),
        &1u32,
    );

    // Patient 1 completes achievement
    client.update_achievement_progress(
        &patient1,
        &patient1,
        &1u64,
        &1u32,
    );

    // Patient 2 completes achievement
    client.update_achievement_progress(
        &patient2,
        &patient2,
        &1u64,
        &1u32,
    );

    // Create another achievement with more points
    client.create_achievement(
        &admin,
        &String::from_str(&env, "Week Warrior"),
        &String::from_str(&env, "Maintain a 7-day streak"),
        &String::from_str(&env, "Streaks"),
        &200u32,
        &String::from_str(&env, "ipfs://badge2"),
        &String::from_str(&env, "streak_days"),
        &7u32,
    );

    // Patient 1 completes second achievement
    client.update_achievement_progress(
        &patient1,
        &patient1,
        &2u64,
        &7u32,
    );

    // Check leaderboard
    let leaderboard = client.get_leaderboard(&10u32);
    assert!(leaderboard.is_ok());
    let leaderboard = leaderboard.unwrap();
    assert_eq!(leaderboard.len(), 2);

    // Patient 1 should be first with 300 points
    let first = leaderboard.get(0).unwrap();
    assert_eq!(first.patient_id, patient1);
    assert_eq!(first.points, 300);
    assert_eq!(first.rank, 1);

    // Patient 2 should be second with 100 points
    let second = leaderboard.get(1).unwrap();
    assert_eq!(second.patient_id, patient2);
    assert_eq!(second.points, 100);
    assert_eq!(second.rank, 2);
}

#[test]
fn test_health_metrics() {
    let env = Env::default();
    let contract_id = env.register_contract(None, PatientGamificationContract);
    let client = PatientGamificationContractClient::new(&env, &contract_id);

    let admin = Address::random(&env);
    let patient = Address::random(&env);

    client.initialize(
        &admin,
        &100u32,
        &50u32,
        &10u32,
        &1000u32,
        &5u32,
    );

    let result = client.record_health_metric(
        &patient,
        &patient,
        &String::from_str(&env, "blood_pressure"),
        &120u32,
        &String::from_str(&env, "mmHg"),
        &String::from_str(&env, "manual"),
    );

    assert!(result.is_ok());
    assert!(result.unwrap());

    let metrics = client.get_health_metrics(
        &patient,
        &String::from_str(&env, "blood_pressure"),
    );
    assert!(metrics.is_ok());
    let metrics = metrics.unwrap();
    assert_eq!(metrics.len(), 1);
}

#[test]
fn test_daily_streak() {
    let env = Env::default();
    let contract_id = env.register_contract(None, PatientGamificationContract);
    let client = PatientGamificationContractClient::new(&env, &contract_id);

    let admin = Address::random(&env);
    let patient = Address::random(&env);

    client.initialize(
        &admin,
        &100u32,
        &50u32,
        &10u32,
        &1000u32,
        &5u32,
    );

    // Record health metric to trigger streak update
    client.record_health_metric(
        &patient,
        &patient,
        &String::from_str(&env, "steps"),
        &5000u32,
        &String::from_str(&env, "steps"),
        &String::from_str(&env, "device"),
    );

    let streak = client.get_daily_streak(&patient);
    assert_eq!(streak.current_streak, 1);
    assert_eq!(streak.total_active_days, 1);
}

#[test]
fn test_privacy_preserving() {
    let env = Env::default();
    let contract_id = env.register_contract(None, PatientGamificationContract);
    let client = PatientGamificationContractClient::new(&env, &contract_id);

    let admin = Address::random(&env);
    let patient = Address::random(&env);

    client.initialize(
        &admin,
        &100u32,
        &50u32,
        &10u32,
        &1000u32,
        &5u32,
    );

    // Create private profile
    client.create_social_profile(
        &patient,
        &String::from_str(&env, "Anonymous"),
        &String::from_str(&env, ""),
        &String::from_str(&env, ""),
        &false, // is_public
        &false, // show_achievements
        &false, // show_challenges
        &false, // show_points
    );

    let profile = client.get_social_profile(&patient);
    assert!(profile.is_ok());
    let profile = profile.unwrap();
    assert!(!profile.is_public);
    assert!(!profile.show_achievements);
    assert!(!profile.show_challenges);
    assert!(!profile.show_points);
}

#[test]
fn test_error_handling() {
    let env = Env::default();
    let contract_id = env.register_contract(None, PatientGamificationContract);
    let client = PatientGamificationContractClient::new(&env, &contract_id);

    let admin = Address::random(&env);
    let patient = Address::random(&env);

    // Try to use before initialization
    let result = client.get_achievement(&1u64);
    assert!(result.is_err());

    client.initialize(
        &admin,
        &100u32,
        &50u32,
        &10u32,
        &1000u32,
        &5u32,
    );

    // Try to get non-existent achievement
    let result = client.get_achievement(&999u64);
    assert!(result.is_err());

    // Try to join non-existent challenge
    let result = client.join_challenge(&patient, &999u64);
    assert!(result.is_err());

    // Try to redeem more points than available
    let result = client.redeem_points(&patient, &1000u64);
    assert!(result.is_err());
}
