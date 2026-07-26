#![cfg(test)]

//! Property-based tests for the ZK access-control invariants defined in
//! `docs/zk/formal-notes.md` (Issue #832).
//!
//! Each `proptest!` block exercises one of the five core invariants from the
//! formal notes with randomly-generated inputs to maximise coverage of edge
//! cases and degenerate inputs (`nullifier == stored_commitment`, zeroed bytes,
//! minimum/maximum timestamps, etc.).
//!
//! All strategies produce plain Rust primitives (`u8`, `u64`, `[u8; 32]`,
//! `bool`) so the Soroban Bound API types (`BytesN<32>`, `Address`, `Env`)
//! are constructed inside each test body — this avoids the lifetime
//! gymnastics required to embed `Env`-bound values inside a `'static`
//! `Strategy`.

mod common;

// external crates
use credential_registry::{CredentialRegistryContract, CredentialRegistryContractClient};
use medical_records::{Error, ZkPublicInputs};
use proptest::prelude::*;
use soroban_sdk::xdr::ToXdr;
use soroban_sdk::{
    testutils::{Address as _, Ledger},
    Address, Bytes, BytesN, Env, String,
};
use zk_verifier::{ZkVerifierContract, ZkVerifierContractClient};

// test helpers
use common::setup_uzima;

// =============================================================================
// Local helpers (mirror a subset of `zk_access_tests.rs` to keep this file
// self-contained and the test bools small)
// =============================================================================

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

#[allow(clippy::too_many_arguments)]
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
        &2_000_000_000u64,
        &BytesN::from_array(env, &[0xAA; 64]),
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

fn add_base_record(env: &Env, t: &common::UzimaTest<'_>) -> u64 {
    t.client.add_record(
        &t.doctor,
        &t.patient,
        &String::from_str(env, "SECRET_DIAGNOSIS"),
        &String::from_str(env, "SECRET_TREATMENT"),
        &true,
        &soroban_sdk::vec![env, String::from_str(env, "secure-tag")],
        &String::from_str(env, "Modern"),
        &String::from_str(env, "Medication"),
        &String::from_str(env, "ipfs://secure-ref-1234567890"),
    )
}

fn proof_bytes(env: &Env, tag: u32) -> Bytes {
    // 32-byte payload keeps the structural validator in `zk_verifier` happy
    // (proof_data must be >= 32 bytes); the tag ensures uniqueness across
    // replay-style property tests.
    let mut buf = [0u8; 32];
    buf[0..4].copy_from_slice(&tag.to_be_bytes());
    Bytes::from_slice(env, &buf)
}

// =============================================================================
// Property-Based Tests
// =============================================================================
//
// Each test exercises one of the five Core Invariants from
// `docs/zk/formal-notes.md`. Tests are configured with a small number of
// cases (16) so the full suite stays under a few seconds in CI; shrinking is
// disabled because the input space for each invariant is structurally simple.

proptest! {
    #![proptest_config(ProptestConfig {
        cases: 16,
        max_shrink_iters: 0,
        .. ProptestConfig::default()
    })]

    /// Invariant 3: Nullifier Uniqueness.
    /// For any valid proof envelope with nullifier `n`, a second submission
    /// with the same `n` must fail with `Error::CredentialRevoked`.
    #[test]
    fn prop_nullifier_uniqueness_replay_always_fails(
        nullifier_bytes in any::<[u8; 32]>(),
        first_proof_tag in any::<u32>(),
        second_proof_tag in any::<u32>(),
    ) {
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
        let nullifier = BytesN::from_array(&env, &nullifier_bytes);

        // First submission: must succeed and register the nullifier.
        let first_inputs = build_public_inputs(
            &env, &t, &issuer, &credential_root, record_id,
            &t.patient, nullifier.clone(), vk_version,
        );
        let first_proof = proof_bytes(&env, first_proof_tag);
        attest(&env, &zk_verifier, &attestor, &first_inputs, &first_proof, true);
        prop_assert_eq!(
            t.client.try_submit_zk_access_proof(
                &t.patient, &record_id,
                &String::from_str(&env, "clinical_review"),
                &first_inputs, &first_proof,
            ),
            Ok(Ok(true)),
            "first submission with fresh nullifier must succeed"
        );

        // Second submission reusing the same nullifier: must fail.
        let second_inputs = build_public_inputs(
            &env, &t, &issuer, &credential_root, record_id,
            &t.patient, nullifier, vk_version,
        );
        let second_proof = proof_bytes(&env, second_proof_tag);
        attest(&env, &zk_verifier, &attestor, &second_inputs, &second_proof, true);
        prop_assert_eq!(
            t.client.try_submit_zk_access_proof(
                &t.patient, &record_id,
                &String::from_str(&env, "clinical_review"),
                &second_inputs, &second_proof,
            ),
            Err(Ok(Error::CredentialRevoked)),
            "second submission with reused nullifier must fail with CredentialRevoked"
        );
    }
}

proptest! {
    #![proptest_config(ProptestConfig {
        cases: 16,
        max_shrink_iters: 0,
        .. ProptestConfig::default()
    })]

    /// Invariant 4: Root Consistency.
    /// For any submitted `credential_root` that differs from the active
    /// issuer root, submission must fail with `Error::InvalidCredential`.
    #[test]
    fn prop_root_mismatch_always_fails(
        // Bit-flip seed: XOR'd into a clone of the stored root to produce
        // a guaranteed-different wrong root regardless of proptest input.
        bit_position in 0u8..8,
        bit_mask in any::<u8>(),
        proof_tag in any::<u32>(),
    ) {
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

        // Build a wrong root that is guaranteed to differ from the stored
        // credential root: rotate a single-bit flip across byte 0's bit
        // positions (proptest-provided `bit_position`) XOR an arbitrary
        // mask into byte 1 (proptest-provided `bit_mask`). Together this
        // always changes at least byte 0 of the stored root.
        let mut wrong_arr = credential_root.to_array();
        wrong_arr[0] ^= 1u8 << bit_position;
        wrong_arr[1] ^= bit_mask;
        let wrong_root = BytesN::from_array(&env, &wrong_arr);

        let inputs = build_public_inputs(
            &env, &t, &issuer, &wrong_root, record_id,
            &t.patient, BytesN::from_array(&env, &[0xB1; 32]), vk_version,
        );
        let proof = proof_bytes(&env, proof_tag);
        attest(&env, &zk_verifier, &attestor, &inputs, &proof, true);

        prop_assert_eq!(
            t.client.try_submit_zk_access_proof(
                &t.patient, &record_id,
                &String::from_str(&env, "clinical_review"),
                &inputs, &proof,
            ),
            Err(Ok(Error::InvalidCredential)),
            "credential_root mismatch must fail with InvalidCredential"
        );
    }
}

proptest! {
    #![proptest_config(ProptestConfig {
        cases: 16,
        max_shrink_iters: 0,
        .. ProptestConfig::default()
    })]

    /// Invariant 5: Commitment Binding.
    /// For any submitted `record_commitment` that differs from the stored
    /// commitment, submission must fail with `Error::InvalidCredential`.
    #[test]
    fn prop_record_commitment_mismatch_always_fails(
        // XOR mask — guarantees at least one bit flip vs the stored
        // commitment for any non-zero input.
        xor_byte0 in any::<u8>(),
        xor_byte1 in any::<u8>(),
        proof_tag in any::<u32>(),
    ) {
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

        // XOR mask — `mask0` always has bit 0 set (forced via `| 0x01`)
        // so byte 0 is always flipped, guaranteeing the wrong commitment
        // differs from the stored one for every proptest input without
        // needing a `prop_assume!` filter.
        let mask0 = xor_byte0 | 0x01;
        let mask1 = xor_byte1;

        let record_id = add_base_record(&env, &t);
        let stored_commitment = t
            .client
            .get_record_commitment(&record_id)
            .unwrap_or(t.client.get_record_metadata(&record_id).record_hash);

        // Mismatched commitment: XOR byte 0 (always) and byte 1 (optionally).
        let mut wrong_arr = stored_commitment.to_array();
        wrong_arr[0] ^= mask0;
        wrong_arr[1] ^= mask1;
        prop_assert_ne!(&wrong_arr, &stored_commitment.to_array());
        let wrong_commitment = BytesN::from_array(&env, &wrong_arr);

        let mut inputs = build_public_inputs(
            &env, &t, &issuer, &credential_root, record_id,
            &t.patient, BytesN::from_array(&env, &[0xC1; 32]), vk_version,
        );
        inputs.record_commitment = wrong_commitment;

        let proof = proof_bytes(&env, proof_tag);
        attest(&env, &zk_verifier, &attestor, &inputs, &proof, true);

        prop_assert_eq!(
            t.client.try_submit_zk_access_proof(
                &t.patient, &record_id,
                &String::from_str(&env, "clinical_review"),
                &inputs, &proof,
            ),
            Err(Ok(Error::InvalidCredential)),
            "record_commitment mismatch must fail with InvalidCredential"
        );
    }
}

proptest! {
    #![proptest_config(ProptestConfig {
        cases: 8,
        max_shrink_iters: 0,
        .. ProptestConfig::default()
    })]

    /// Invariant 4 (Enforcement toggle) + Invariant 1 (Access Safety).
    /// For any boolean `enforce`:
    ///   - If `zk_enforced = true`, a user with passing ACL but no ZK grant
    ///     cannot read the record (`InvalidCredential`).
    ///   - If `zk_enforced = false`, ACL-only access succeeds without any
    ///     ZK grant (matches the formal note that ZK enforcement is a
    ///     toggle layered on top of the existing ACL).
    #[test]
    fn prop_enforcement_toggle_preserves_expected_behavior(
        enforce in any::<bool>(),
    ) {
        let env = Env::default();
        env.mock_all_auths();
        let t = setup_uzima(&env);
        let (
            credential_registry_id,
            zk_verifier_id,
            _credential_registry,
            _zk_verifier,
            _issuer,
            _attestor,
            _vk_version,
            _credential_root,
        ) = setup_zk_gate(&env, &t.admin1);
        t.client
            .set_credential_registry_contract(&t.admin1, &credential_registry_id);
        t.client
            .set_zk_verifier_contract(&t.admin1, &zk_verifier_id);
        t.client.set_zk_enforced(&t.admin1, &enforce);

        let record_id = add_base_record(&env, &t);
        // Important: do NOT submit a proof; only the ACL matters.

        let read_result = t.client.try_get_record(&t.patient, &record_id);

        if enforce {
            // ACL passes (patient owns record), but ZK is enforced and no
            // grant exists => must fail with InvalidCredential.
            prop_assert_eq!(
                read_result,
                Err(Ok(Error::InvalidCredential)),
                "with zk_enforced=true and no grant, patient read must fail"
            );
        } else {
            // ACL passes and ZK enforcement is bypassed => inner Ok(record).
            // We must unwrap both layers because `try_get_record` returns
            // `Result<Result<MedicalRecord, Error>, Result<Error, ContractError>>`
            // — only checking the outer Result hides a contract regression
            // where the inner Result becomes an error.
            prop_assert!(
                matches!(read_result, Ok(Ok(_))),
                "with zk_enforced=false and no grant, patient read must succeed at the contract level (got {:?})",
                read_result,
            );
        }
    }
}

proptest! {
    #![proptest_config(ProptestConfig {
        cases: 8,
        max_shrink_iters: 0,
        .. ProptestConfig::default()
    })]

    /// Invariant 5 (Grant expiration): any grant with
    /// `expires_at <= ledger_timestamp` must be invalid.
    ///
    /// We configure a tiny TTL via `set_zk_grant_ttl`, submit a valid proof
    /// (which materialises a grant with `expires_at = now + ttl`), then
    /// advance the ledger past `expires_at`, and assert the read is denied.
    #[test]
    fn prop_grant_expiration_invalidates_after_ttl(
        ttl_secs in 1u64..30,
        proof_tag in any::<u32>(),
    ) {
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

        // Configure a short TTL so the test can cheaply move past it.
        // ttl_secs is in 1..30 which is always valid (contract cap is 3600),
        // but unwrap so a future tightening of validation surfaces here.
        t.client
            .set_zk_grant_ttl(&t.admin1, &ttl_secs)
            .expect("test setup: short TTL must be accepted by the contract");

        let record_id = add_base_record(&env, &t);
        let proof = proof_bytes(&env, proof_tag);
        let inputs = build_public_inputs(
            &env, &t, &issuer, &credential_root, record_id,
            &t.patient, BytesN::from_array(&env, &[0xD1; 32]), vk_version,
        );
        attest(&env, &zk_verifier, &attestor, &inputs, &proof, true);
        prop_assert!(t.client.submit_zk_access_proof(
            &t.patient, &record_id,
            &String::from_str(&env, "clinical_review"),
            &inputs, &proof,
        ));

        // Immediately after submission the grant is valid.
        let now = env.ledger().timestamp();
        prop_assert!(
            t.client.has_valid_zk_access_grant(&t.patient, &record_id),
            "grant must be valid immediately after submission"
        );
        prop_assert!(t.client.try_get_record(&t.patient, &record_id).is_ok());

        // Advance the ledger past expires_at and re-check.
        env.ledger().set_timestamp(now.saturating_add(ttl_secs).saturating_add(1));

        prop_assert!(
            !t.client.has_valid_zk_access_grant(&t.patient, &record_id),
            "grant must be invalid once ledger ts >= expires_at"
        );
        prop_assert_eq!(
            t.client.try_get_record(&t.patient, &record_id),
            Err(Ok(Error::InvalidCredential)),
            "read after expiration must fail with InvalidCredential"
        );
    }
}
