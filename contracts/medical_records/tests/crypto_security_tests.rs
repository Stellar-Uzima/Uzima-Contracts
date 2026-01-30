#![cfg(test)]

mod common;

use common::setup_uzima;
use medical_records::{CryptoAuditAction, EnvelopeAlgorithm, Error, KeyEnvelope};
use soroban_sdk::testutils::{Address as _, Ledger};
use soroban_sdk::{Bytes, BytesN, String, Vec};

fn make_envelopes(env: &soroban_sdk::Env, patient: &soroban_sdk::Address, doctor: &soroban_sdk::Address) -> Vec<KeyEnvelope> {
    let mut envs = Vec::new(env);

    let patient_env = KeyEnvelope {
        recipient: patient.clone(),
        key_version: 1,
        algorithm: EnvelopeAlgorithm::X25519,
        wrapped_key: Bytes::from_slice(env, &[1, 2, 3, 4]),
        pq_wrapped_key: None,
    };
    envs.push_back(patient_env);

    let doctor_env = KeyEnvelope {
        recipient: doctor.clone(),
        key_version: 1,
        algorithm: EnvelopeAlgorithm::X25519,
        wrapped_key: Bytes::from_slice(env, &[9, 9, 9]),
        pq_wrapped_key: None,
    };
    envs.push_back(doctor_env);

    envs
}

#[test]
fn test_encrypted_record_requires_crypto_registry() {
    let env = soroban_sdk::Env::default();
    let t = setup_uzima(&env);

    let mut tags = Vec::new(&env);
    tags.push_back(String::from_str(&env, "tag"));

    let ciphertext_ref = String::from_str(&env, "ipfs://ciphertextcid001");
    let ciphertext_hash = BytesN::from_array(&env, &[7u8; 32]);
    let envelopes = make_envelopes(&env, &t.patient, &t.doctor);

    let res = t.client.try_add_encrypted_record(
        &t.doctor,
        &t.patient,
        &true,
        &tags,
        &String::from_str(&env, "Modern"),
        &String::from_str(&env, "Medication"),
        &ciphertext_ref,
        &ciphertext_hash,
        &envelopes,
    );

    assert_eq!(res, Err(Ok(Error::CryptoRegistryNotSet.into())));
}

#[test]
fn test_plaintext_add_record_blocked_when_encryption_required() {
    let env = soroban_sdk::Env::default();
    let t = setup_uzima(&env);

    // Enable encryption-only mode.
    assert!(t.client.set_encryption_required(&t.admin1, &true));

    let res = t.client.try_add_record(
        &t.doctor,
        &t.patient,
        &String::from_str(&env, "Diagnosis"),
        &String::from_str(&env, "Treatment"),
        &false,
        &Vec::new(&env),
        &String::from_str(&env, "Modern"),
        &String::from_str(&env, "Medication"),
        &String::from_str(&env, "ipfs://plaintextcid0001"),
    );

    assert_eq!(res, Err(Ok(Error::EncryptionRequired.into())));
}

#[test]
fn test_encrypted_record_access_control_and_envelope_scoping() {
    let env = soroban_sdk::Env::default();
    let t = setup_uzima(&env);

    // Configure crypto registry (address is metadata hook; registry contract validates versions off-chain).
    let registry = soroban_sdk::Address::generate(&env);
    assert!(t.client.set_crypto_registry(&t.admin1, &registry));

    let mut tags = Vec::new(&env);
    tags.push_back(String::from_str(&env, "tag"));

    let ciphertext_ref = String::from_str(&env, "ipfs://ciphertextcid002");
    let ciphertext_hash = BytesN::from_array(&env, &[3u8; 32]);
    let envelopes = make_envelopes(&env, &t.patient, &t.doctor);

    let record_id = t.client.add_encrypted_record(
        &t.doctor,
        &t.patient,
        &true,
        &tags,
        &String::from_str(&env, "Modern"),
        &String::from_str(&env, "Medication"),
        &ciphertext_ref,
        &ciphertext_hash,
        &envelopes,
    );

    // Patient can view header and get their own envelope.
    let header = t
        .client
        .get_encrypted_record_header(&t.patient, &record_id)
        .unwrap();
    assert_eq!(header.record_id, record_id);
    assert_eq!(header.ciphertext_hash, ciphertext_hash);

    let patient_env = t
        .client
        .get_encrypted_record_envelope(&t.patient, &record_id)
        .unwrap();
    assert_eq!(patient_env.recipient, t.patient);

    // Doctor can fetch their envelope; patient cannot see doctor's envelope via this API.
    let doctor_env = t
        .client
        .get_encrypted_record_envelope(&t.doctor, &record_id)
        .unwrap();
    assert_eq!(doctor_env.recipient, t.doctor);

    let intruder = soroban_sdk::Address::generate(&env);
    let denied = t.client.try_get_encrypted_record_header(&intruder, &record_id);
    assert!(denied.is_err());
    let inner = denied.err().unwrap().unwrap();
    assert_eq!(inner, Error::NotAuthorized.into());
}

#[test]
fn test_envelope_update_and_crypto_audit_log() {
    let env = soroban_sdk::Env::default();
    let t = setup_uzima(&env);

    let registry = soroban_sdk::Address::generate(&env);
    assert!(t.client.set_crypto_registry(&t.admin1, &registry));

    let mut tags = Vec::new(&env);
    tags.push_back(String::from_str(&env, "tag"));

    let ciphertext_ref = String::from_str(&env, "ipfs://ciphertextcid003");
    let ciphertext_hash = BytesN::from_array(&env, &[5u8; 32]);
    let envelopes = make_envelopes(&env, &t.patient, &t.doctor);

    let record_id = t.client.add_encrypted_record(
        &t.doctor,
        &t.patient,
        &true,
        &tags,
        &String::from_str(&env, "Modern"),
        &String::from_str(&env, "Medication"),
        &ciphertext_ref,
        &ciphertext_hash,
        &envelopes,
    );

    let updated = KeyEnvelope {
        recipient: t.doctor.clone(),
        key_version: 2,
        algorithm: EnvelopeAlgorithm::X25519,
        wrapped_key: Bytes::from_slice(&env, &[1, 1, 1, 1, 1]),
        pq_wrapped_key: None,
    };
    assert!(t
        .client
        .upsert_encrypted_record_envelope(&t.doctor, &record_id, &updated));

    let fetched = t
        .client
        .get_encrypted_record_envelope(&t.doctor, &record_id)
        .unwrap();
    assert_eq!(fetched.key_version, 2);

    let logs = t.client.get_crypto_audit_logs(&t.admin1, &0u32, &50u32);
    assert!(logs.len() > 0);
    assert!(logs.iter().any(|e| e.action == CryptoAuditAction::EnvelopeUpdated));
}

#[test]
fn test_crypto_config_threshold_proposal_flow() {
    let env = soroban_sdk::Env::default();
    let t = setup_uzima(&env);

    let proposal_id = t.client.propose_crypto_config_update(
        &t.admin1,
        &None,
        &None,
        &None,
        &Some(true),
        &Some(true),
    );

    // Timelock not elapsed.
    let early = t
        .client
        .try_execute_crypto_config_update(&t.admin1, &proposal_id);
    assert_eq!(early, Err(Ok(Error::TimelockNotElasped.into())));

    // Fast forward past timelock.
    env.ledger()
        .set_timestamp(env.ledger().timestamp() + 86_400 + 1);

    // Not enough approvals.
    let insufficient = t
        .client
        .try_execute_crypto_config_update(&t.admin1, &proposal_id);
    assert_eq!(insufficient, Err(Ok(Error::NotEnoughApproval.into())));

    // Approve by second admin and execute.
    assert!(t
        .client
        .approve_crypto_config_update(&t.admin2, &proposal_id));
    assert!(t
        .client
        .execute_crypto_config_update(&t.admin1, &proposal_id));

    assert!(t.client.is_encryption_required());
    assert!(t.client.is_require_pq_envelopes());

    let proposal = t
        .client
        .get_crypto_config_proposal(&t.admin1, &proposal_id)
        .unwrap();
    assert!(proposal.executed);
}

#[test]
fn test_require_pq_envelopes_gates_encrypted_records() {
    let env = soroban_sdk::Env::default();
    let t = setup_uzima(&env);

    let registry = soroban_sdk::Address::generate(&env);
    assert!(t.client.set_crypto_registry(&t.admin1, &registry));
    assert!(t.client.set_require_pq_envelopes(&t.admin1, &true));

    let mut tags = Vec::new(&env);
    tags.push_back(String::from_str(&env, "tag"));

    let ciphertext_ref = String::from_str(&env, "ipfs://ciphertextcid004");
    let ciphertext_hash = BytesN::from_array(&env, &[9u8; 32]);

    // Missing PQ wrapped keys should fail.
    let envelopes = make_envelopes(&env, &t.patient, &t.doctor);
    let res = t.client.try_add_encrypted_record(
        &t.doctor,
        &t.patient,
        &true,
        &tags,
        &String::from_str(&env, "Modern"),
        &String::from_str(&env, "Medication"),
        &ciphertext_ref,
        &ciphertext_hash,
        &envelopes,
    );
    assert_eq!(res, Err(Ok(Error::InvalidInput.into())));

    // Provide PQ wrapped keys and retry.
    let mut pq_envs = Vec::new(&env);
    pq_envs.push_back(KeyEnvelope {
        recipient: t.patient.clone(),
        key_version: 1,
        algorithm: EnvelopeAlgorithm::HybridX25519Kyber768,
        wrapped_key: Bytes::from_slice(&env, &[1, 2, 3, 4]),
        pq_wrapped_key: Some(Bytes::from_slice(&env, &[8, 8, 8, 8])),
    });
    pq_envs.push_back(KeyEnvelope {
        recipient: t.doctor.clone(),
        key_version: 1,
        algorithm: EnvelopeAlgorithm::HybridX25519Kyber768,
        wrapped_key: Bytes::from_slice(&env, &[9, 9, 9]),
        pq_wrapped_key: Some(Bytes::from_slice(&env, &[7, 7, 7])),
    });

    let record_id = t.client.add_encrypted_record(
        &t.doctor,
        &t.patient,
        &true,
        &tags,
        &String::from_str(&env, "Modern"),
        &String::from_str(&env, "Medication"),
        &ciphertext_ref,
        &ciphertext_hash,
        &pq_envs,
    );
    assert!(record_id > 0);
}
