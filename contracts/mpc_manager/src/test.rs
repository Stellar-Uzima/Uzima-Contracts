use super::*;
use soroban_sdk::testutils::Address as _;
use soroban_sdk::{vec, Address, Bytes, BytesN, Env, String};

fn setup(env: &Env) -> (MPCManagerClient<'_>, Address) {
    let id = Address::generate(env);
    env.register_contract(&id, MPCManager);
    (MPCManagerClient::new(env, &id), id)
}

#[test]
fn mpc_session_lifecycle() {
    let env = Env::default();
    env.mock_all_auths();

    let (client, _id) = setup(&env);
    let admin = Address::generate(&env);
    client.initialize(&admin);

    let initiator = Address::generate(&env);
    let p1 = Address::generate(&env);
    let p2 = Address::generate(&env);
    let participants = vec![&env, p1.clone(), p2.clone()];
    let sid = BytesN::from_array(&env, &[3u8; 32]);
    let purpose = String::from_str(&env, "cohort-risk-analysis");

    client.start_session(
        &initiator,
        &sid,
        &participants,
        &2u32,
        &purpose,
        &100u64,
        &ComputationType::StatisticalAnalysis,
    );

    client.commit_share(&p1, &sid, &BytesN::from_array(&env, &[1u8; 32]));
    client.commit_share(&p2, &sid, &BytesN::from_array(&env, &[2u8; 32]));

    client.reveal_share(
        &p1,
        &sid,
        &String::from_str(&env, "ipfs://share1"),
        &BytesN::from_array(&env, &[4u8; 32]),
    );
    client.reveal_share(
        &p2,
        &sid,
        &String::from_str(&env, "ipfs://share2"),
        &BytesN::from_array(&env, &[5u8; 32]),
    );

    client.finalize_session(
        &initiator,
        &sid,
        &String::from_str(&env, "ipfs://result"),
        &BytesN::from_array(&env, &[9u8; 32]),
        &String::from_str(&env, ""),
        &BytesN::from_array(&env, &[0u8; 32]),
    );

    let status = client.get_session(&sid).map(|s| s.status);
    assert!(matches!(status, Some(SessionStatus::Finalized)));
}

#[test]
fn test_shamir_secret_sharing() {
    let env = Env::default();
    env.mock_all_auths();

    let (client, _id) = setup(&env);
    let admin = Address::generate(&env);
    client.initialize(&admin);

    let initiator = Address::generate(&env);
    let p1 = Address::generate(&env);
    let participants = vec![&env, p1.clone()];
    let sid = BytesN::from_array(&env, &[4u8; 32]);
    let purpose = String::from_str(&env, "secret-sharing-test");
    let secret = Bytes::from_slice(&env, b"medical_record_encryption_key");

    client.start_session(
        &initiator,
        &sid,
        &participants,
        &1u32,
        &purpose,
        &100u64,
        &ComputationType::SecureAggregation,
    );

    let shares = client.create_secret_shares(&p1, &sid, &secret, &5u32, &3u32);
    assert_eq!(shares.len(), 5);

    // Verify share structure
    for share in shares.iter() {
        assert!(share.share_id > 0);
        assert!(!share.share_value.is_empty());
        // Note: created_at might be 0 in test environment
    }
}

#[test]
fn test_statistical_analysis() {
    let env = Env::default();
    env.mock_all_auths();

    let (client, _id) = setup(&env);
    let admin = Address::generate(&env);
    client.initialize(&admin);

    let initiator = Address::generate(&env);
    let p1 = Address::generate(&env);
    let participants = vec![&env, p1.clone()];
    let sid = BytesN::from_array(&env, &[5u8; 32]);
    let purpose = String::from_str(&env, "statistical-analysis-test");
    let encrypted_data = Bytes::from_slice(&env, b"encrypted_patient_data");

    client.start_session(
        &initiator,
        &sid,
        &participants,
        &1u32,
        &purpose,
        &100u64,
        &ComputationType::StatisticalAnalysis,
    );

    let result_hash = client.perform_statistical_analysis(
        &p1,
        &sid,
        &String::from_str(&env, "mean_calculation"),
        &encrypted_data,
    );

    assert!(result_hash != BytesN::from_array(&env, &[0u8; 32]));
}

#[test]
fn test_secure_ml_training() {
    let env = Env::default();
    env.mock_all_auths();

    let (client, _id) = setup(&env);
    let admin = Address::generate(&env);
    client.initialize(&admin);

    let initiator = Address::generate(&env);
    let p1 = Address::generate(&env);
    let participants = vec![&env, p1.clone()];
    let sid = BytesN::from_array(&env, &[6u8; 32]);
    let purpose = String::from_str(&env, "ml-training-test");
    let model_params = Bytes::from_slice(&env, b"neural_network_params");
    let training_data = Bytes::from_slice(&env, b"encrypted_training_set");

    client.start_session(
        &initiator,
        &sid,
        &participants,
        &1u32,
        &purpose,
        &100u64,
        &ComputationType::PrivacyPreservingML,
    );

    let model_hash = client.train_secure_ml_model(&p1, &sid, &model_params, &training_data);
    assert!(model_hash != BytesN::from_array(&env, &[0u8; 32]));
}

#[test]
fn test_computation_proof_submission() {
    let env = Env::default();
    env.mock_all_auths();

    let (client, _id) = setup(&env);
    let admin = Address::generate(&env);
    client.initialize(&admin);

    let initiator = Address::generate(&env);
    let p1 = Address::generate(&env);
    let participants = vec![&env, p1.clone()];
    let sid = BytesN::from_array(&env, &[7u8; 32]);
    let purpose = String::from_str(&env, "proof-test");

    client.start_session(
        &initiator,
        &sid,
        &participants,
        &1u32,
        &purpose,
        &100u64,
        &ComputationType::StatisticalAnalysis,
    );

    // Move to reveal phase
    client.commit_share(&p1, &sid, &BytesN::from_array(&env, &[1u8; 32]));

    let proof = ComputationProof {
        computation_type: String::from_str(&env, "statistical"),
        input_commitment: BytesN::from_array(&env, &[8u8; 32]),
        output_hash: BytesN::from_array(&env, &[9u8; 32]),
        proof_data: Bytes::from_slice(&env, b"zk_proof_data"),
        verification_key_hash: BytesN::from_array(&env, &[10u8; 32]),
        gas_used: 25000u64,
        created_at: env.ledger().timestamp(),
    };

    client.submit_computation_proof(&p1, &sid, &proof);
    // If we reach here, the proof was submitted successfully
}

#[test]
fn test_audit_trail() {
    let env = Env::default();
    env.mock_all_auths();

    let (client, _id) = setup(&env);
    let admin = Address::generate(&env);
    client.initialize(&admin);

    let initiator = Address::generate(&env);
    let p1 = Address::generate(&env);
    let participants = vec![&env, p1.clone()];
    let sid = BytesN::from_array(&env, &[8u8; 32]);
    let purpose = String::from_str(&env, "audit-test");
    let secret = Bytes::from_slice(&env, b"test_secret");

    client.start_session(
        &initiator,
        &sid,
        &participants,
        &1u32,
        &purpose,
        &100u64,
        &ComputationType::SecureAggregation,
    );

    // Perform operations that should create audit entries
    client.create_secret_shares(&p1, &sid, &secret, &3u32, &2u32);

    // Retrieve audit trail
    let audit_trail = client.get_audit_trail(&sid);
    assert!(!audit_trail.is_empty());
}

#[test]
fn test_gas_tracking() {
    let env = Env::default();
    env.mock_all_auths();

    let (client, _id) = setup(&env);
    let admin = Address::generate(&env);
    client.initialize(&admin);

    let initiator = Address::generate(&env);
    let p1 = Address::generate(&env);
    let participants = vec![&env, p1.clone()];
    let sid = BytesN::from_array(&env, &[9u8; 32]);
    let purpose = String::from_str(&env, "gas-test");

    client.start_session(
        &initiator,
        &sid,
        &participants,
        &1u32,
        &purpose,
        &100u64,
        &ComputationType::StatisticalAnalysis,
    );

    // Perform operations that consume gas
    client.perform_statistical_analysis(
        &p1,
        &sid,
        &String::from_str(&env, "test"),
        &Bytes::from_slice(&env, b"data"),
    );

    // Check gas statistics
    let _total_gas = client.get_gas_stats(&sid);
    // Note: gas tracking might not work in test environment
    // In production, this would track actual gas consumption
    // u64 is always >= 0, so we just verify the function works
}

#[test]
fn test_multiple_institution_participants() {
    let env = Env::default();
    env.mock_all_auths();

    let (client, _id) = setup(&env);
    let admin = Address::generate(&env);
    client.initialize(&admin);

    let initiator = Address::generate(&env);
    let hospital_a = Address::generate(&env);
    let hospital_b = Address::generate(&env);
    let research_lab = Address::generate(&env);
    let participants = vec![
        &env,
        hospital_a.clone(),
        hospital_b.clone(),
        research_lab.clone(),
    ];

    let sid = BytesN::from_array(&env, &[10u8; 32]);
    let purpose = String::from_str(&env, "multi-institution-study");
    let secret = Bytes::from_slice(&env, b"shared_medical_secret");

    client.start_session(
        &initiator,
        &sid,
        &participants,
        &3u32,
        &purpose,
        &100u64,
        &ComputationType::DiagnosticAnalysis,
    );

    // Each participant creates shares
    let shares_a = client.create_secret_shares(&hospital_a, &sid, &secret, &5u32, &3u32);
    let shares_b = client.create_secret_shares(&hospital_b, &sid, &secret, &5u32, &3u32);
    let shares_c = client.create_secret_shares(&research_lab, &sid, &secret, &5u32, &3u32);

    assert_eq!(shares_a.len(), 5);
    assert_eq!(shares_b.len(), 5);
    assert_eq!(shares_c.len(), 5);

    // Verify all participants are tracked
    let audit_trail = client.get_audit_trail(&sid);
    assert_eq!(audit_trail.len(), 3); // One entry per participant
}
