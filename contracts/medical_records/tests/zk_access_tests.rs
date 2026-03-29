#![cfg(test)]

mod common;

use common::setup_uzima;
use credential_registry::{CredentialRegistryContract, CredentialRegistryContractClient};
use medical_records::{Error, ZkAuditRecord, ZkPublicInputs};
use soroban_sdk::xdr::ToXdr;
use soroban_sdk::{
    symbol_short,
    testutils::{Address as _, Events, Ledger},
    Address, Bytes, BytesN, Env, String, Symbol, TryFromVal,
};
use std::time::Instant;
use zk_verifier::{ZkVerifierContract, ZkVerifierContractClient};

fn append_bytes32(env: &Env, payload: &mut Bytes, value: &BytesN<32>) {
    payload.append(&Bytes::from_slice(env, &value.to_array()));
}

fn hash_address(env: &Env, address: &Address) -> BytesN<32> {
    env.crypto().sha256(&address.to_xdr(env)).into()
}

fn compute_pseudonym(
    env: &Env,
    requester: &Address,
    issuer: &Address,
    record_id: u64,
) -> BytesN<32> {
    let mut payload = Bytes::new(env);
    payload.append(&requester.to_xdr(env));
    payload.append(&issuer.to_xdr(env));
    payload.append(&Bytes::from_slice(env, &record_id.to_be_bytes()));
    payload.append(&Bytes::from_slice(env, b"UZIMA_ZK_PSEUDONYM_V1"));
    env.crypto().sha256(&payload).into()
}

fn hash_public_inputs(env: &Env, public_inputs: &ZkPublicInputs) -> BytesN<32> {
    let mut payload = Bytes::new(env);
    payload.append(&Bytes::from_slice(
        env,
        &public_inputs.record_id.to_be_bytes(),
    ));
    append_bytes32(env, &mut payload, &public_inputs.record_commitment);
    append_bytes32(env, &mut payload, &public_inputs.credential_root);
    payload.append(&public_inputs.issuer.clone().to_xdr(env));
    append_bytes32(env, &mut payload, &public_inputs.requester_commitment);
    append_bytes32(env, &mut payload, &public_inputs.provider_commitment);
    append_bytes32(env, &mut payload, &public_inputs.claim_commitment);
    payload.append(&Bytes::from_slice(
        env,
        &public_inputs.min_timestamp.to_be_bytes(),
    ));
    payload.append(&Bytes::from_slice(
        env,
        &public_inputs.max_timestamp.to_be_bytes(),
    ));
    append_bytes32(env, &mut payload, &public_inputs.nullifier);
    append_bytes32(env, &mut payload, &public_inputs.pseudonym);
    payload.append(&Bytes::from_slice(
        env,
        &public_inputs.vk_version.to_be_bytes(),
    ));
    env.crypto().sha256(&payload).into()
}

fn setup_zk_gate<'a>(
    env: &'a Env,
    admin: &Address,
) -> (
    Address,
    Address,
    CredentialRegistryContractClient<'a>,
    ZkVerifierContractClient<'a>,
    Address,
    Address,
    u32,
    BytesN<32>,
) {
    let credential_registry_id = env.register_contract(None, CredentialRegistryContract);
    let credential_registry = CredentialRegistryContractClient::new(env, &credential_registry_id);
    credential_registry.initialize(admin);

    let zk_verifier_id = env.register_contract(None, ZkVerifierContract);
    let zk_verifier = ZkVerifierContractClient::new(env, &zk_verifier_id);
    zk_verifier.initialize(admin, &600u64);

    let issuer = Address::generate(env);
    let attestor = Address::generate(env);
    let credential_root = BytesN::from_array(env, &[0x42; 32]);
    credential_registry.set_credential_root(
        admin,
        &issuer,
        &credential_root,
        &BytesN::from_array(env, &[0x11; 32]),
    );

    let vk_version = zk_verifier.register_verifying_key(
        admin,
        &BytesN::from_array(env, &[0xAA; 32]),
        &BytesN::from_array(env, &[0xBB; 32]),
        &attestor,
        &BytesN::from_array(env, &[0xCC; 32]),
    );

    (
        credential_registry_id,
        zk_verifier_id,
        credential_registry,
        zk_verifier,
        issuer,
        attestor,
        vk_version,
        credential_root,
    )
}

fn add_base_record(env: &Env, t: &common::UzimaTest<'_>) -> u64 {
    t.client.add_record(
        &t.doctor,
        &t.patient,
        &String::from_str(env, "SECRET_DIAGNOSIS_VALUE"),
        &String::from_str(env, "SECRET_TREATMENT_VALUE"),
        &true,
        &soroban_sdk::vec![env, String::from_str(env, "secure-tag")],
        &String::from_str(env, "Modern"),
        &String::from_str(env, "Medication"),
        &String::from_str(env, "ipfs://secure-ref-1234567890"),
    )
}

#[allow(clippy::too_many_arguments)]
fn build_public_inputs(
    env: &Env,
    t: &common::UzimaTest<'_>,
    issuer: &Address,
    credential_root: &BytesN<32>,
    record_id: u64,
    requester: &Address,
    nullifier: BytesN<32>,
    vk_version: u32,
) -> ZkPublicInputs {
    let meta = t.client.get_record_metadata(&record_id);
    let record_commitment = t
        .client
        .get_record_commitment(&record_id)
        .unwrap_or(meta.record_hash.clone());

    ZkPublicInputs {
        record_id,
        record_commitment,
        credential_root: credential_root.clone(),
        issuer: issuer.clone(),
        requester_commitment: hash_address(env, requester),
        provider_commitment: hash_address(env, &t.doctor),
        claim_commitment: BytesN::from_array(env, &[0x33; 32]),
        min_timestamp: meta.timestamp.saturating_sub(5),
        max_timestamp: meta.timestamp.saturating_add(5),
        nullifier,
        pseudonym: compute_pseudonym(env, requester, issuer, record_id),
        vk_version,
    }
}

fn attest(
    env: &Env,
    zk_verifier: &ZkVerifierContractClient<'_>,
    attestor: &Address,
    public_inputs: &ZkPublicInputs,
    proof: &Bytes,
    verified: bool,
) {
    let pi_hash = hash_public_inputs(env, public_inputs);
    let proof_hash: BytesN<32> = env.crypto().sha256(proof).into();
    zk_verifier.submit_attestation(
        attestor,
        &public_inputs.vk_version,
        &pi_hash,
        &proof_hash,
        &verified,
        &300u64,
    );
}

#[test]
fn test_zk_success_path_and_access_gate() {
    let env = Env::default();
    env.mock_all_auths();
    let t = setup_uzima(&env);

    let (
        credential_registry_id,
        zk_verifier_id,
        _credential_registry,
        zk_verifier,
        issuer,
        attestor,
        vk_version,
        credential_root,
    ) = setup_zk_gate(&env, &t.admin1);
    t.client
        .set_credential_registry_contract(&t.admin1, &credential_registry_id);
    t.client
        .set_zk_verifier_contract(&t.admin1, &zk_verifier_id);
    t.client.set_zk_enforced(&t.admin1, &true);

    let record_id = add_base_record(&env, &t);

    let public_inputs = build_public_inputs(
        &env,
        &t,
        &issuer,
        &credential_root,
        record_id,
        &t.patient,
        BytesN::from_array(&env, &[0xA1; 32]),
        vk_version,
    );
    let proof = Bytes::from_slice(&env, b"valid-proof-bytes-v1");
    attest(&env, &zk_verifier, &attestor, &public_inputs, &proof, true);

    assert!(t.client.submit_zk_access_proof(
        &t.patient,
        &record_id,
        &String::from_str(&env, "clinical_review"),
        &public_inputs,
        &proof,
    ));

    let record = t.client.get_record(&t.patient, &record_id);
    assert_eq!(record.patient_id, t.patient);
}

#[test]
fn test_zk_failure_wrong_record_commitment() {
    let env = Env::default();
    env.mock_all_auths();
    let t = setup_uzima(&env);
    let (
        credential_registry_id,
        zk_verifier_id,
        _credential_registry,
        zk_verifier,
        issuer,
        attestor,
        vk_version,
        credential_root,
    ) = setup_zk_gate(&env, &t.admin1);

    t.client
        .set_credential_registry_contract(&t.admin1, &credential_registry_id);
    t.client
        .set_zk_verifier_contract(&t.admin1, &zk_verifier_id);
    t.client.set_zk_enforced(&t.admin1, &true);

    let record_id = add_base_record(&env, &t);
    let mut public_inputs = build_public_inputs(
        &env,
        &t,
        &issuer,
        &credential_root,
        record_id,
        &t.patient,
        BytesN::from_array(&env, &[0xA2; 32]),
        vk_version,
    );
    public_inputs.record_commitment = BytesN::from_array(&env, &[0x99; 32]);

    let proof = Bytes::from_slice(&env, b"proof-bad-commitment");
    attest(&env, &zk_verifier, &attestor, &public_inputs, &proof, true);

    let res = t.client.try_submit_zk_access_proof(
        &t.patient,
        &record_id,
        &String::from_str(&env, "clinical_review"),
        &public_inputs,
        &proof,
    );
    assert_eq!(res, Err(Ok(Error::InvalidCredential)));
}

#[test]
fn test_zk_failure_wrong_credential_root() {
    let env = Env::default();
    env.mock_all_auths();
    let t = setup_uzima(&env);
    let (
        credential_registry_id,
        zk_verifier_id,
        _credential_registry,
        zk_verifier,
        issuer,
        attestor,
        vk_version,
        credential_root,
    ) = setup_zk_gate(&env, &t.admin1);

    t.client
        .set_credential_registry_contract(&t.admin1, &credential_registry_id);
    t.client
        .set_zk_verifier_contract(&t.admin1, &zk_verifier_id);
    t.client.set_zk_enforced(&t.admin1, &true);

    let record_id = add_base_record(&env, &t);
    let wrong_root = BytesN::from_array(&env, &[0x55; 32]);
    assert_ne!(wrong_root, credential_root);

    let public_inputs = build_public_inputs(
        &env,
        &t,
        &issuer,
        &wrong_root,
        record_id,
        &t.patient,
        BytesN::from_array(&env, &[0xA3; 32]),
        vk_version,
    );
    let proof = Bytes::from_slice(&env, b"proof-wrong-root");
    attest(&env, &zk_verifier, &attestor, &public_inputs, &proof, true);

    let res = t.client.try_submit_zk_access_proof(
        &t.patient,
        &record_id,
        &String::from_str(&env, "clinical_review"),
        &public_inputs,
        &proof,
    );
    assert_eq!(res, Err(Ok(Error::InvalidCredential)));
}

#[test]
fn test_zk_failure_replay_nullifier_reused() {
    let env = Env::default();
    env.mock_all_auths();
    let t = setup_uzima(&env);
    let (
        credential_registry_id,
        zk_verifier_id,
        _credential_registry,
        zk_verifier,
        issuer,
        attestor,
        vk_version,
        credential_root,
    ) = setup_zk_gate(&env, &t.admin1);

    t.client
        .set_credential_registry_contract(&t.admin1, &credential_registry_id);
    t.client
        .set_zk_verifier_contract(&t.admin1, &zk_verifier_id);
    t.client.set_zk_enforced(&t.admin1, &true);

    let record_id = add_base_record(&env, &t);
    let nullifier = BytesN::from_array(&env, &[0xA4; 32]);

    let public_inputs_first = build_public_inputs(
        &env,
        &t,
        &issuer,
        &credential_root,
        record_id,
        &t.patient,
        nullifier.clone(),
        vk_version,
    );
    let proof_first = Bytes::from_slice(&env, b"proof-first");
    attest(
        &env,
        &zk_verifier,
        &attestor,
        &public_inputs_first,
        &proof_first,
        true,
    );
    assert!(t.client.submit_zk_access_proof(
        &t.patient,
        &record_id,
        &String::from_str(&env, "clinical_review"),
        &public_inputs_first,
        &proof_first,
    ));

    let public_inputs_second = build_public_inputs(
        &env,
        &t,
        &issuer,
        &credential_root,
        record_id,
        &t.patient,
        nullifier,
        vk_version,
    );
    let proof_second = Bytes::from_slice(&env, b"proof-second");
    attest(
        &env,
        &zk_verifier,
        &attestor,
        &public_inputs_second,
        &proof_second,
        true,
    );

    let replay = t.client.try_submit_zk_access_proof(
        &t.patient,
        &record_id,
        &String::from_str(&env, "clinical_review"),
        &public_inputs_second,
        &proof_second,
    );
    assert_eq!(replay, Err(Ok(Error::CredentialRevoked)));
}

#[test]
fn test_zk_failure_malformed_proof() {
    let env = Env::default();
    env.mock_all_auths();
    let t = setup_uzima(&env);
    let (
        credential_registry_id,
        zk_verifier_id,
        _credential_registry,
        zk_verifier,
        issuer,
        attestor,
        vk_version,
        credential_root,
    ) = setup_zk_gate(&env, &t.admin1);

    t.client
        .set_credential_registry_contract(&t.admin1, &credential_registry_id);
    t.client
        .set_zk_verifier_contract(&t.admin1, &zk_verifier_id);
    t.client.set_zk_enforced(&t.admin1, &true);

    let record_id = add_base_record(&env, &t);
    let public_inputs = build_public_inputs(
        &env,
        &t,
        &issuer,
        &credential_root,
        record_id,
        &t.patient,
        BytesN::from_array(&env, &[0xA5; 32]),
        vk_version,
    );
    let good_proof = Bytes::from_slice(&env, b"proof-good-shape");
    attest(
        &env,
        &zk_verifier,
        &attestor,
        &public_inputs,
        &good_proof,
        true,
    );
    let malformed = Bytes::from_slice(&env, b"proof-MALFORMED");

    let res = t.client.try_submit_zk_access_proof(
        &t.patient,
        &record_id,
        &String::from_str(&env, "clinical_review"),
        &public_inputs,
        &malformed,
    );
    assert_eq!(res, Err(Ok(Error::InvalidCredential)));
}

#[test]
fn test_acl_and_zk_gate_interplay() {
    let env = Env::default();
    env.mock_all_auths();
    let t = setup_uzima(&env);
    let (
        credential_registry_id,
        zk_verifier_id,
        _credential_registry,
        zk_verifier,
        issuer,
        attestor,
        vk_version,
        credential_root,
    ) = setup_zk_gate(&env, &t.admin1);

    t.client
        .set_credential_registry_contract(&t.admin1, &credential_registry_id);
    t.client
        .set_zk_verifier_contract(&t.admin1, &zk_verifier_id);
    t.client.set_zk_enforced(&t.admin1, &true);

    let record_id = add_base_record(&env, &t);

    // ACL passes for patient, but no ZK grant yet -> blocked.
    let no_grant = t.client.try_get_record(&t.patient, &record_id);
    assert!(matches!(no_grant, Err(Ok(Error::InvalidCredential))));

    // Unauthorized user can even submit proof, but ACL still blocks final read.
    let intruder = Address::generate(&env);
    let public_inputs_intruder = build_public_inputs(
        &env,
        &t,
        &issuer,
        &credential_root,
        record_id,
        &intruder,
        BytesN::from_array(&env, &[0xA6; 32]),
        vk_version,
    );
    let intruder_proof = Bytes::from_slice(&env, b"proof-intruder");
    attest(
        &env,
        &zk_verifier,
        &attestor,
        &public_inputs_intruder,
        &intruder_proof,
        true,
    );
    assert!(t.client.submit_zk_access_proof(
        &intruder,
        &record_id,
        &String::from_str(&env, "clinical_review"),
        &public_inputs_intruder,
        &intruder_proof,
    ));
    let intruder_read = t.client.try_get_record(&intruder, &record_id);
    assert!(matches!(intruder_read, Err(Ok(Error::NotAuthorized))));

    // Patient with valid proof succeeds.
    let public_inputs_patient = build_public_inputs(
        &env,
        &t,
        &issuer,
        &credential_root,
        record_id,
        &t.patient,
        BytesN::from_array(&env, &[0xA7; 32]),
        vk_version,
    );
    let patient_proof = Bytes::from_slice(&env, b"proof-patient");
    attest(
        &env,
        &zk_verifier,
        &attestor,
        &public_inputs_patient,
        &patient_proof,
        true,
    );
    assert!(t.client.submit_zk_access_proof(
        &t.patient,
        &record_id,
        &String::from_str(&env, "clinical_review"),
        &public_inputs_patient,
        &patient_proof,
    ));
    let patient_record = t.client.get_record(&t.patient, &record_id);
    assert_eq!(patient_record.patient_id, t.patient);
}

#[test]
fn test_audit_event_privacy_and_performance() {
    let env = Env::default();
    env.mock_all_auths();
    env.ledger().with_mut(|li| {
        li.timestamp = 1_700_000_000;
        li.sequence_number = 99;
    });

    let t = setup_uzima(&env);
    let (
        credential_registry_id,
        zk_verifier_id,
        _credential_registry,
        zk_verifier,
        issuer,
        attestor,
        vk_version,
        credential_root,
    ) = setup_zk_gate(&env, &t.admin1);

    t.client
        .set_credential_registry_contract(&t.admin1, &credential_registry_id);
    t.client
        .set_zk_verifier_contract(&t.admin1, &zk_verifier_id);
    t.client.set_zk_enforced(&t.admin1, &true);

    let record_id = add_base_record(&env, &t);
    let public_inputs = build_public_inputs(
        &env,
        &t,
        &issuer,
        &credential_root,
        record_id,
        &t.patient,
        BytesN::from_array(&env, &[0xA8; 32]),
        vk_version,
    );
    let proof = Bytes::from_slice(&env, b"proof-audit");
    attest(&env, &zk_verifier, &attestor, &public_inputs, &proof, true);

    let started = Instant::now();
    assert!(t.client.submit_zk_access_proof(
        &t.patient,
        &record_id,
        &String::from_str(&env, "clinical_review"),
        &public_inputs,
        &proof,
    ));
    let elapsed = started.elapsed();
    assert!(elapsed.as_millis() < 5_000);

    let _ = t.client.get_record(&t.patient, &record_id);

    let mut saw_zk_audit = false;
    for event in env.events().all().iter() {
        if event.1.len() < 2 {
            continue;
        }
        let Some(topic) = event.1.get(1) else {
            continue;
        };
        if Symbol::try_from_val(&env, &topic) == Ok(symbol_short!("ZK_AUD")) {
            let data = event.2;
            if ZkAuditRecord::try_from_val(&env, &data).is_ok() {
                saw_zk_audit = true;
            }
        }
    }
    assert!(saw_zk_audit);

    let event_dump = format!("{:?}", env.events().all());
    assert!(!event_dump.contains("SECRET_DIAGNOSIS_VALUE"));
    assert!(!event_dump.contains("SECRET_TREATMENT_VALUE"));
}
