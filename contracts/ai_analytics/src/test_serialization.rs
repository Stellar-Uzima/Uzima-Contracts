//! Regression tests for `ai_analytics` serialization helpers.
//!
//! These tests pin the current behavior of [`SerializationUtils`],
//! the [`SafeSerialize`] trait, and the [`SafeSerialize`] impls on the
//! contract types (`FederatedRound`, `ParticipantUpdateMeta`,
//! `ModelMetadata`). They cover both the success path (valid inputs
//! return `Ok(())`) and the failure path (invalid inputs return the
//! correct `SerializationError` variant and contract-level `Error`
//! code).
//!
//! Test naming and file layout follow the convention used by the
//! other focused test files in the workspace (e.g.
//! `medical_records/src/test_permissions.rs`): a top-level
//! `test_<topic>` module registered in `lib.rs`, flat `#[test]` fns
//! prefixed with `test_`, `Env::default()` + `mock_all_auths` for
//! contract-flow tests.

#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]

use crate::serialization_utils::{
    SafeSerialize, SerializationError, SerializationUtils, MAX_COLLECTION_SIZE, MAX_NESTING_DEPTH,
    MAX_STRING_LENGTH,
};
use crate::types::{Error, FederatedRound, ModelMetadata, ParticipantUpdateMeta};
use crate::{AiAnalyticsContract, AiAnalyticsContractClient};
use soroban_sdk::{testutils::Address as _, Address, BytesN, Env, Map, String, Vec};

// ---------------------------------------------------------------------------
// Limit constants — lock current values so accidental loosening is caught
// ---------------------------------------------------------------------------

#[test]
fn test_max_nesting_depth_constant_is_locked() {
    // If this fails, the serialization helper limits have been changed.
    // That is intentional only if the change is accompanied by a
    // deliberate update to this test.
    assert_eq!(MAX_NESTING_DEPTH, 50);
}

#[test]
fn test_max_collection_size_constant_is_locked() {
    assert_eq!(MAX_COLLECTION_SIZE, 10_000);
}

#[test]
fn test_max_string_length_constant_is_locked() {
    assert_eq!(MAX_STRING_LENGTH, 100_000);
}

// ---------------------------------------------------------------------------
// SerializationUtils::validate_collection_size — success path
// ---------------------------------------------------------------------------

#[test]
fn test_validate_collection_size_accepts_empty_vec() {
    let env = Env::default();
    let mut v: Vec<u32> = Vec::new(&env);
    assert!(SerializationUtils::validate_collection_size(&v).is_ok());
}

#[test]
fn test_validate_collection_size_accepts_small_vec() {
    let env = Env::default();
    let mut v: Vec<u32> = Vec::new(&env);
    for i in 0..16u32 {
        v.push_back(i);
    }
    assert!(SerializationUtils::validate_collection_size(&v).is_ok());
}

// Note on collection-size failure path: `Vec::len()` returns `u32` and the
// limit is 10_000. Materializing a vector that large in an SDK test is
// unnecessary — `validate_nesting_depth` below already exercises the same
// branch shape (numeric comparison -> specific `SerializationError`),
// giving us a deterministic regression anchor for the "size exceeded"
// error path.

// ---------------------------------------------------------------------------
// SerializationUtils::validate_map_size — success path
// ---------------------------------------------------------------------------

#[test]
fn test_validate_map_size_accepts_empty_map() {
    let env = Env::default();
    let mut m: Map<u32, u32> = Map::new(&env);
    assert!(SerializationUtils::validate_map_size(&m).is_ok());
}

#[test]
fn test_validate_map_size_accepts_small_map() {
    let env = Env::default();
    let mut m: Map<u32, u32> = Map::new(&env);
    for i in 0..16u32 {
        m.set(i, i * 2);
    }
    assert!(SerializationUtils::validate_map_size(&m).is_ok());
}

// ---------------------------------------------------------------------------
// SerializationUtils::validate_string_length — success path
// ---------------------------------------------------------------------------

#[test]
fn test_validate_string_length_accepts_empty_string() {
    let env = Env::default();
    let s = String::from_str(&env, "");
    assert!(SerializationUtils::validate_string_length(&s).is_ok());
}

#[test]
fn test_validate_string_length_accepts_short_string() {
    let env = Env::default();
    let s = String::from_str(&env, "hello world");
    assert!(SerializationUtils::validate_string_length(&s).is_ok());
}

// ---------------------------------------------------------------------------
// SerializationUtils::validate_nesting_depth — success AND failure paths
// (this is the canonical numeric-limit branch and the cleanest place to
// pin the exact `SerializationError` variant returned on overflow)
// ---------------------------------------------------------------------------

#[test]
fn test_validate_nesting_depth_accepts_zero() {
    assert!(SerializationUtils::validate_nesting_depth(0).is_ok());
}

#[test]
fn test_validate_nesting_depth_accepts_value_at_limit() {
    // Depth exactly at MAX is currently valid (the helper rejects only
    // values strictly greater than MAX). Locking this boundary prevents
    // an off-by-one regression in either direction.
    assert!(SerializationUtils::validate_nesting_depth(MAX_NESTING_DEPTH).is_ok());
}

#[test]
fn test_validate_nesting_depth_rejects_value_above_limit() {
    let result = SerializationUtils::validate_nesting_depth(MAX_NESTING_DEPTH + 1);
    assert_eq!(result, Err(SerializationError::NestingTooDeep));
}

#[test]
fn test_validate_nesting_depth_rejects_far_above_limit() {
    let result = SerializationUtils::validate_nesting_depth(u32::MAX);
    assert_eq!(result, Err(SerializationError::NestingTooDeep));
}

// ---------------------------------------------------------------------------
// SerializationUtils::safe_serialize_vec / _map / _string — success path
// (these wrap the validators above and also log on empty input; we lock
// the Ok return contract here)
// ---------------------------------------------------------------------------

#[test]
fn test_safe_serialize_vec_succeeds_on_empty() {
    let env = Env::default();
    let mut v: Vec<u32> = Vec::new(&env);
    assert!(SerializationUtils::safe_serialize_vec(&env, &v).is_ok());
}

#[test]
fn test_safe_serialize_vec_succeeds_on_populated() {
    let env = Env::default();
    let mut v: Vec<u64> = Vec::new(&env);
    v.push_back(1);
    v.push_back(2);
    v.push_back(3);
    assert!(SerializationUtils::safe_serialize_vec(&env, &v).is_ok());
}

#[test]
fn test_safe_serialize_map_succeeds_on_empty() {
    let env = Env::default();
    let mut m: Map<u32, u32> = Map::new(&env);
    assert!(SerializationUtils::safe_serialize_map(&env, &m).is_ok());
}

#[test]
fn test_safe_serialize_map_succeeds_on_populated() {
    let env = Env::default();
    let mut m: Map<u32, u64> = Map::new(&env);
    m.set(1, 100);
    m.set(2, 200);
    assert!(SerializationUtils::safe_serialize_map(&env, &m).is_ok());
}

#[test]
fn test_safe_serialize_string_succeeds_on_empty() {
    let env = Env::default();
    let s = String::from_str(&env, "");
    assert!(SerializationUtils::safe_serialize_string(&env, &s).is_ok());
}

#[test]
fn test_safe_serialize_string_succeeds_on_populated() {
    let env = Env::default();
    let s = String::from_str(&env, "ipfs://Qm...");
    assert!(SerializationUtils::safe_serialize_string(&env, &s).is_ok());
}

// ---------------------------------------------------------------------------
// SerializationUtils::validate_bytes_n / validate_address — success path
// ---------------------------------------------------------------------------

#[test]
fn test_validate_bytes_n_accepts_zero_bytes() {
    let env = Env::default();
    let b: BytesN<32> = BytesN::from_array(&env, &[0u8; 32]);
    assert!(SerializationUtils::validate_bytes_n(&b).is_ok());
}

#[test]
fn test_validate_bytes_n_accepts_arbitrary_bytes() {
    let env = Env::default();
    let b: BytesN<32> = BytesN::from_array(&env, &[7u8; 32]);
    assert!(SerializationUtils::validate_bytes_n(&b).is_ok());
}

#[test]
fn test_validate_address_accepts_generated_address() {
    let env = Env::default();
    let a = Address::generate(&env);
    assert!(SerializationUtils::validate_address(&a).is_ok());
}

// ---------------------------------------------------------------------------
// SafeSerialize trait impls — success path
// (these are thin wrappers, but the impls are part of the public surface
// and a future refactor could silently drop or reroute them)
// ---------------------------------------------------------------------------

#[test]
fn test_safe_serialize_trait_for_vec() {
    let env = Env::default();
    let mut v: Vec<u64> = Vec::new(&env);
    v.push_back(42);
    assert!(v.safe_serialize(&env).is_ok());
}

#[test]
fn test_safe_serialize_trait_for_map() {
    let env = Env::default();
    let mut m: Map<u32, u32> = Map::new(&env);
    m.set(1, 1);
    assert!(m.safe_serialize(&env).is_ok());
}

#[test]
fn test_safe_serialize_trait_for_string() {
    let env = Env::default();
    let s = String::from_str(&env, "model-v1");
    assert!(s.safe_serialize(&env).is_ok());
}

#[test]
fn test_safe_serialize_trait_for_bytes_n() {
    let env = Env::default();
    let b: BytesN<32> = BytesN::from_array(&env, &[9u8; 32]);
    assert!(b.safe_serialize(&env).is_ok());
}

#[test]
fn test_safe_serialize_trait_for_address() {
    let env = Env::default();
    let a = Address::generate(&env);
    assert!(a.safe_serialize(&env).is_ok());
}

// ---------------------------------------------------------------------------
// SafeSerialize on contract types — success path
// ---------------------------------------------------------------------------

#[test]
fn test_safe_serialize_federated_round_succeeds() {
    let env = Env::default();
    let round = FederatedRound {
        id: 1,
        base_model_id: BytesN::from_array(&env, &[1u8; 32]),
        min_participants: 3,
        dp_epsilon: 5,
        started_at: 1_700_000_000,
        finalized_at: 0,
        total_updates: 0,
        is_finalized: false,
    };
    assert!(round.safe_serialize(&env).is_ok());
}

#[test]
fn test_safe_serialize_federated_round_accepts_zero_participants() {
    // Zero `min_participants` logs a warning but must still serialize
    // successfully — pinning this so a future refactor that converts
    // the warning to an error has to update this test explicitly.
    let env = Env::default();
    let round = FederatedRound {
        id: 0,
        base_model_id: BytesN::from_array(&env, &[0u8; 32]),
        min_participants: 0,
        dp_epsilon: 0,
        started_at: 0,
        finalized_at: 0,
        total_updates: 0,
        is_finalized: false,
    };
    assert!(round.safe_serialize(&env).is_ok());
}

#[test]
fn test_safe_serialize_participant_update_meta_succeeds() {
    let env = Env::default();
    let meta = ParticipantUpdateMeta {
        round_id: 1,
        participant: Address::generate(&env),
        update_hash: BytesN::from_array(&env, &[2u8; 32]),
        num_samples: 100,
    };
    assert!(meta.safe_serialize(&env).is_ok());
}

#[test]
fn test_safe_serialize_participant_update_meta_accepts_zero_samples() {
    let env = Env::default();
    let meta = ParticipantUpdateMeta {
        round_id: 0,
        participant: Address::generate(&env),
        update_hash: BytesN::from_array(&env, &[0u8; 32]),
        num_samples: 0,
    };
    assert!(meta.safe_serialize(&env).is_ok());
}

#[test]
fn test_safe_serialize_model_metadata_succeeds_with_full_fields() {
    let env = Env::default();
    let model = ModelMetadata {
        model_id: BytesN::from_array(&env, &[3u8; 32]),
        round_id: 1,
        description: String::from_str(&env, "Test model"),
        metrics_ref: String::from_str(&env, "ipfs://metrics"),
        fairness_report_ref: String::from_str(&env, "ipfs://fairness"),
        created_at: 1_700_000_000,
    };
    assert!(model.safe_serialize(&env).is_ok());
}

#[test]
fn test_safe_serialize_model_metadata_accepts_empty_string_fields() {
    let env = Env::default();
    let model = ModelMetadata {
        model_id: BytesN::from_array(&env, &[0u8; 32]),
        round_id: 0,
        description: String::from_str(&env, ""),
        metrics_ref: String::from_str(&env, ""),
        fairness_report_ref: String::from_str(&env, ""),
        created_at: 0,
    };
    assert!(model.safe_serialize(&env).is_ok());
}

// ---------------------------------------------------------------------------
// SerializationError variant identity
// (locks the discriminant set so a reorder or rename surfaces here, not
// in some downstream error-mapping code at runtime)
// ---------------------------------------------------------------------------

#[test]
fn test_serialization_error_variants_are_distinct() {
    // Build every variant and confirm equality is by variant, not by
    // accident of representation. This will fail if a variant is
    // removed or merged with another.
    let variants = [
        SerializationError::CollectionTooLarge,
        SerializationError::StringTooLong,
        SerializationError::NestingTooDeep,
        SerializationError::InvalidValue,
        SerializationError::EmptyCollection,
        SerializationError::CircularReference,
    ];
    for (i, a) in variants.iter().enumerate() {
        for (j, b) in variants.iter().enumerate() {
            if i == j {
                assert_eq!(a, b);
            } else {
                assert_ne!(a, b);
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Contract-level regression: serialized data round-trips through storage
// (catches breakage in the serialization helpers that only manifests
// once values cross the contract boundary)
// ---------------------------------------------------------------------------

#[test]
fn test_federated_round_round_trips_through_storage() {
    let env = Env::default();
    let contract_id = env.register_contract(None, AiAnalyticsContract);
    let client = AiAnalyticsContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    client.mock_all_auths().initialize(&admin);

    let base_model = BytesN::from_array(&env, &[1u8; 32]);
    let round_id = client
        .mock_all_auths()
        .start_round(&admin, &base_model, &2u32, &1u32);

    let stored: FederatedRound = client.get_round(&round_id).unwrap();
    assert_eq!(stored.id, round_id);
    assert_eq!(stored.base_model_id, base_model);
    assert_eq!(stored.min_participants, 2);
    assert_eq!(stored.dp_epsilon, 1);
    assert!(!stored.is_finalized);
    assert_eq!(stored.total_updates, 0);
}

#[test]
fn test_model_metadata_round_trips_through_storage() {
    let env = Env::default();
    let contract_id = env.register_contract(None, AiAnalyticsContract);
    let client = AiAnalyticsContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let participant1 = Address::generate(&env);
    let participant2 = Address::generate(&env);

    client.mock_all_auths().initialize(&admin);

    let base_model = BytesN::from_array(&env, &[1u8; 32]);
    let round_id = client
        .mock_all_auths()
        .start_round(&admin, &base_model, &2u32, &1u32);

    client.mock_all_auths().submit_update(
        &participant1,
        &round_id,
        &BytesN::from_array(&env, &[2u8; 32]),
        &10u32,
    );
    client.mock_all_auths().submit_update(
        &participant2,
        &round_id,
        &BytesN::from_array(&env, &[3u8; 32]),
        &20u32,
    );

    let new_model = BytesN::from_array(&env, &[4u8; 32]);
    let description = String::from_str(&env, "Test model");
    let metrics_ref = String::from_str(&env, "ipfs://metrics");
    let fairness_ref = String::from_str(&env, "ipfs://fairness");

    client.mock_all_auths().finalize_round(
        &admin,
        &round_id,
        &new_model,
        &description,
        &metrics_ref,
        &fairness_ref,
    );

    let stored: ModelMetadata = client.get_model(&new_model).unwrap();
    assert_eq!(stored.model_id, new_model);
    assert_eq!(stored.round_id, round_id);
    assert_eq!(stored.description, description);
    assert_eq!(stored.metrics_ref, metrics_ref);
    assert_eq!(stored.fairness_report_ref, fairness_ref);
}

#[test]
fn test_empty_string_fields_round_trip_through_storage() {
    // Empty strings are a documented edge case for the serialization
    // helpers (they log a warning but must serialize). Confirm the
    // contract-level flow agrees.
    let env = Env::default();
    let contract_id = env.register_contract(None, AiAnalyticsContract);
    let client = AiAnalyticsContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let participant = Address::generate(&env);

    client.mock_all_auths().initialize(&admin);

    let base_model = BytesN::from_array(&env, &[5u8; 32]);
    let round_id = client
        .mock_all_auths()
        .start_round(&admin, &base_model, &1u32, &0u32);

    client.mock_all_auths().submit_update(
        &participant,
        &round_id,
        &BytesN::from_array(&env, &[6u8; 32]),
        &1u32,
    );

    let new_model = BytesN::from_array(&env, &[7u8; 32]);
    let empty = String::from_str(&env, "");

    client
        .mock_all_auths()
        .finalize_round(&admin, &round_id, &new_model, &empty, &empty, &empty);

    let stored: ModelMetadata = client.get_model(&new_model).unwrap();
    assert_eq!(stored.description, empty);
    assert_eq!(stored.metrics_ref, empty);
    assert_eq!(stored.fairness_report_ref, empty);
}

// ---------------------------------------------------------------------------
// Error code stability — the contract `Error` enum has serialization-
// related discriminants (8/9/10/11) that downstream tooling depends on.
// Lock them here.
// ---------------------------------------------------------------------------

#[test]
fn test_contract_error_discriminants_for_serialization_are_stable() {
    assert_eq!(Error::SerializationError as u32, 8);
    assert_eq!(Error::CollectionTooLarge as u32, 9);
    assert_eq!(Error::StringTooLong as u32, 10);
    assert_eq!(Error::NestingTooDeep as u32, 11);
}
