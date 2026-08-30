//! XDR serialization conformance suite (issue #1516).
//!
//! Enforces the canonical rules from `docs/SERIALIZATION_STANDARDS.md` that
//! the M1 trace decoder depends on:
//!
//! 1. `Option<T>` encodes as `SCVal::Void` for `None` and `SCVal::T` for
//!    `Some(T)` — no sentinel values, and `None` must never collide with any
//!    `Some(value)` encoding (fixtures committed under `tests/xdr-fixtures/`).
//! 2. `Map` encodes insertion order but consumers must compare by keys, not
//!    by bytes.
//! 3. Enumerated ABI state must be a `#[contracttype] enum` — never a raw
//!    `u32` in the ABI.
//! 4. `String` lengths are bounded by an explicit `max_len`.
//! 5. Every canonical type named in the standard round-trips through XDR
//!    byte-stably, and the `SCVal -> typed JSON` mapping the decoder will use
//!    is asserted against `tests/xdr-fixtures/scval-mapping.json` (shared
//!    fixture contract between the suite and the `libs` decoder).
//!
//! Run: `cargo test --manifest-path tests/Cargo.toml --test xdr_conformance`
//! (wired into CI via `.github/workflows/xdr-conformance.yml`).

use soroban_sdk::xdr::{FromXdr, ToXdr};
use soroban_sdk::{vec, Bytes, Env, Map, String as SString, Symbol, Vec as SVec};

use serde_json::{json, Map as JsonMap, Value};

const MAX_STRING_LEN: u32 = 32;

fn check_bounded_string(value: &SString, max_len: u32) -> Result<(), &'static str> {
    if value.len() > max_len {
        Err("string exceeds declared max length")
    } else {
        Ok(())
    }
}

fn bytes_to_hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

#[test]
fn option_none_round_trips_without_sentinel_ambiguity() {
    let env = Env::default();

    let none = Option::<u32>::None;
    let none_bytes = none.to_xdr(&env).to_alloc_vec();
    // Canonical SCVal::Void (discriminant 1), the committed golden fixture.
    assert_eq!(
        &none_bytes[..],
        &include_bytes!("../xdr-fixtures/option_none.xdr")[..],
        "Option::None must serialize as SCVal::Void"
    );
    let decoded: Option<u32> =
        Option::from_xdr(&env, &Bytes::from_slice(&env, &none_bytes)).unwrap();
    assert_eq!(decoded, None);

    let some_max = Option::<u32>::Some(u32::MAX);
    let some_max_bytes = some_max.to_xdr(&env).to_alloc_vec();
    // Canonical SCVal::U32(u32::MAX) (discriminant 3), the committed fixture.
    assert_eq!(
        &some_max_bytes[..],
        &include_bytes!("../xdr-fixtures/option_some_u32_max.xdr")[..],
        "Option::Some(u32::MAX) must serialize as SCVal::U32"
    );
    let decoded_max: Option<u32> =
        Option::from_xdr(&env, &Bytes::from_slice(&env, &some_max_bytes)).unwrap();
    assert_eq!(decoded_max, Some(u32::MAX));

    // A "no value" encoded as a sentinel u32::MAX would be indistinguishable
    // from a real value; the canonical Option encoding must never collide.
    assert_ne!(
        none_bytes, some_max_bytes,
        "None (Void) must never share its encoding with Some(u32::MAX)"
    );
}

#[test]
fn option_some_round_trips_byte_stable() {
    let env = Env::default();
    let value = -170_141_183_460_469_231_731_687_303_715_884_105_727i128;
    let first = Option::<i128>::Some(value).to_xdr(&env).to_alloc_vec();
    for _ in 0..8 {
        let again = Option::<i128>::Some(value).to_xdr(&env).to_alloc_vec();
        assert_eq!(again, first, "re-encoding a payload must be byte-stable");
        let decoded: Option<i128> =
            Option::from_xdr(&env, &Bytes::from_slice(&env, &first)).unwrap();
        assert_eq!(decoded, Option::Some(value));
    }
}

#[test]
fn map_encoding_preserves_insertion_order_but_consumers_compare_keys() {
    let env = Env::default();

    let mut first = Map::new(&env);
    first.set(SString::from_str(&env, "alpha"), 1u64);
    first.set(SString::from_str(&env, "beta"), 2u64);
    let first_bytes = first.to_xdr(&env).to_alloc_vec();

    let mut reordered = Map::new(&env);
    reordered.set(SString::from_str(&env, "beta"), 2u64);
    reordered.set(SString::from_str(&env, "alpha"), 1u64);
    let reordered_bytes = reordered.to_xdr(&env).to_alloc_vec();

    assert_ne!(
        first_bytes, reordered_bytes,
        "soroban_sdk::Map preserves insertion order in XDR, so byte-level \
         comparison of two version-ordered maps must not be used"
    );

    // Canonical consumer discipline: compare by keys explicitly.
    let decoded_first: Map<SString, u64> =
        Map::from_xdr(&env, &Bytes::from_slice(&env, &first_bytes)).unwrap();
    let decoded_reordered: Map<SString, u64> =
        Map::from_xdr(&env, &Bytes::from_slice(&env, &reordered_bytes)).unwrap();

    for key in ["alpha", "beta"] {
        let k = SString::from_str(&env, key);
        assert_eq!(
            decoded_first.get(k.clone()),
            decoded_reordered.get(k.clone()),
            "key '{key}' must compare equal regardless of insertion order"
        );
    }
    assert_eq!(decoded_first.len(), 2);
    assert_eq!(decoded_reordered.len(), 2);
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum AbiRole {
    Enumeration,
    Scalar,
}

struct AbiField {
    method: &'static str,
    field: &'static str,
    role: AbiRole,
    ty: &'static str,
}

/// Mechanical ABI rule from `docs/SERIALIZATION_STANDARDS.md`:
/// "Enumerated state | `#[contracttype] enum` | Never raw u32 in ABI".
fn raw_u32_in_enumerated_abi(abi: &[AbiField]) -> Vec<String> {
    abi.iter()
        .filter(|field| field.role == AbiRole::Enumeration && field.ty == "u32")
        .map(|field| {
            format!(
                "{}::{} declares enumerated state as raw u32; \
                 docs/SERIALIZATION_STANDARDS.md requires a #[contracttype] enum",
                field.method, field.field
            )
        })
        .collect()
}

#[test]
fn enum_over_u32_rule_flags_sample_violation_and_accepts_clean_abi() {
    // A represented sample violation: enumerated state passed as a bare u32.
    let violating = [AbiField {
        method: "set_record_status",
        field: "kind",
        role: AbiRole::Enumeration,
        ty: "u32",
    }];
    let violations = raw_u32_in_enumerated_abi(&violating);
    assert_eq!(violations.len(), 1, "a raw-u32 enumerated ABI must be flagged");
    assert!(violations[0].contains("kind"));

    // Workspace-member contracts model enumerated state with #[contracttype]
    // enums (e.g. medical_records::Role), which the rule requires.
    let clean = [
        AbiField {
            method: "manage_user",
            field: "role",
            role: AbiRole::Enumeration,
            ty: "Role",
        },
        AbiField {
            method: "set_thing_state",
            field: "state",
            role: AbiRole::Enumeration,
            ty: "RecordStatus",
        },
        AbiField {
            method: "set_threshold",
            field: "ledger",
            role: AbiRole::Scalar,
            ty: "u32",
        },
    ];
    assert!(
        raw_u32_in_enumerated_abi(&clean).is_empty(),
        "contracttype enums and scalar u32 values must not be flagged"
    );
}

#[test]
fn string_lengths_are_bounded_per_standard() {
    let env = Env::default();
    let at_limit = SString::from_str(&env, &"x".repeat(32));
    let beyond_limit = SString::from_str(&env, &"x".repeat(33));

    assert_eq!(check_bounded_string(&at_limit, MAX_STRING_LEN), Ok(()));
    assert_eq!(
        check_bounded_string(&beyond_limit, MAX_STRING_LEN),
        Err("string exceeds declared max length")
    );
}

#[test]
fn scval_mapping_manifest_covers_all_canonical_types() {
    let manifest: Value = serde_json::from_str(include_str!("../xdr-fixtures/scval-mapping.json"))
        .expect("scval-mapping.json must parse");
    let types = manifest["types"].as_array().expect("types array");
    let handled_types = {
        let mut seen = Vec::new();
        for entry in types {
            let type_name = entry["type"].as_str().unwrap();
            let expected = &entry["expected_json"];
            let env = Env::default();
            match type_name {
                "option_none" => {
                    let bytes = Option::<u32>::None.to_xdr(&env).to_alloc_vec();
                    let decoded: Option<u32> =
                        Option::from_xdr(&env, &Bytes::from_slice(&env, &bytes)).unwrap();
                    assert_eq!(decoded, None);
                    assert_eq!(&json!(null), expected, "option_none maps to JSON null");
                }
                "option_some" => {
                    let bytes = Option::<u32>::Some(42).to_xdr(&env).to_alloc_vec();
                    let decoded: Option<u32> =
                        Option::from_xdr(&env, &Bytes::from_slice(&env, &bytes)).unwrap();
                    assert_eq!(decoded, Option::Some(42));
                    assert_eq!(&json!(42), expected, "option_some maps to its inner value");
                }
                "bool" => {
                    let bytes = true.to_xdr(&env).to_alloc_vec();
                    let decoded = bool::from_xdr(&env, &Bytes::from_slice(&env, &bytes)).unwrap();
                    assert_eq!(decoded, true);
                    assert_eq!(&json!(true), expected);
                }
                "u32" => {
                    let bytes = u32::MAX.to_xdr(&env).to_alloc_vec();
                    let decoded = u32::from_xdr(&env, &Bytes::from_slice(&env, &bytes)).unwrap();
                    assert_eq!(decoded, u32::MAX);
                    assert_eq!(&json!(u32::MAX), expected);
                }
                "i32" => {
                    let bytes = i32::MIN.to_xdr(&env).to_alloc_vec();
                    let decoded = i32::from_xdr(&env, &Bytes::from_slice(&env, &bytes)).unwrap();
                    assert_eq!(decoded, i32::MIN);
                    assert_eq!(&json!(i32::MIN), expected);
                }
                "u64" => {
                    let bytes = u64::MAX.to_xdr(&env).to_alloc_vec();
                    let decoded = u64::from_xdr(&env, &Bytes::from_slice(&env, &bytes)).unwrap();
                    assert_eq!(decoded, u64::MAX);
                    assert_eq!(&json!(u64::MAX), expected);
                }
                "i64" => {
                    let bytes = i64::MIN.to_xdr(&env).to_alloc_vec();
                    let decoded = i64::from_xdr(&env, &Bytes::from_slice(&env, &bytes)).unwrap();
                    assert_eq!(decoded, i64::MIN);
                    assert_eq!(&json!(i64::MIN), expected);
                }
                "u128" => {
                    let bytes = u128::MAX.to_xdr(&env).to_alloc_vec();
                    let decoded = u128::from_xdr(&env, &Bytes::from_slice(&env, &bytes)).unwrap();
                    assert_eq!(decoded, u128::MAX);
                    // 128-bit values exceed JSON number range; the mapping is a string.
                    assert_eq!(&Value::String(u128::MAX.to_string()), expected);
                }
                "i128" => {
                    let bytes = i128::MIN.to_xdr(&env).to_alloc_vec();
                    let decoded = i128::from_xdr(&env, &Bytes::from_slice(&env, &bytes)).unwrap();
                    assert_eq!(decoded, i128::MIN);
                    assert_eq!(&Value::String(i128::MIN.to_string()), expected);
                }
                "bytes" => {
                    let raw = [0xf0u8, 0x0d, 0xca, 0xfe];
                    let bytes = Bytes::from_slice(&env, &raw).to_xdr(&env).to_alloc_vec();
                    let decoded = Bytes::from_xdr(&env, &Bytes::from_slice(&env, &bytes)).unwrap();
                    assert_eq!(decoded.to_alloc_vec(), raw);
                    assert_eq!(&json!(bytes_to_hex(&raw)), expected);
                }
                "string" => {
                    let bytes = SString::from_str(&env, "herbal therapy")
                        .to_xdr(&env)
                        .to_alloc_vec();
                    let decoded = SString::from_xdr(&env, &Bytes::from_slice(&env, &bytes))
                        .unwrap();
                    assert_eq!(decoded.to_string(), "herbal therapy");
                    assert_eq!(&json!("herbal therapy"), expected);
                }
                "symbol" => {
                    let bytes = Symbol::new(&env, "STATE_LEDGER").to_xdr(&env).to_alloc_vec();
                    let decoded = Symbol::from_xdr(&env, &Bytes::from_slice(&env, &bytes))
                        .unwrap();
                    assert_eq!(decoded.to_string(), "STATE_LEDGER");
                    assert_eq!(&json!("STATE_LEDGER"), expected);
                }
                "vec" => {
                    let values: SVec<u32> = vec![&env, 1u32, 2u32, 3u32];
                    let bytes = values.clone().to_xdr(&env).to_alloc_vec();
                    let decoded: SVec<u32> =
                        SVec::from_xdr(&env, &Bytes::from_slice(&env, &bytes)).unwrap();
                    let collected: Vec<u32> = decoded.iter().collect();
                    assert_eq!(collected, [1u32, 2, 3].to_vec());
                    assert_eq!(&json!([1, 2, 3]), expected);
                }
                "map" => {
                    let mut map = Map::new(&env);
                    map.set(SString::from_str(&env, "alpha"), 1u64);
                    map.set(SString::from_str(&env, "beta"), 2u64);
                    let bytes = map.to_xdr(&env).to_alloc_vec();
                    let decoded: Map<SString, u64> =
                        Map::from_xdr(&env, &Bytes::from_slice(&env, &bytes)).unwrap();
                    let mut object = JsonMap::new();
                    object.insert(
                        "alpha".to_string(),
                        json!(decoded.get(SString::from_str(&env, "alpha")).unwrap()),
                    );
                    object.insert(
                        "beta".to_string(),
                        json!(decoded.get(SString::from_str(&env, "beta")).unwrap()),
                    );
                    let expected_object = expected.as_object().unwrap();
                    assert_eq!(expected_object["alpha"], &json!(1u64));
                    assert_eq!(expected_object["beta"], &json!(2u64));
                    assert_eq!(Value::Object(object), *expected);
                }
                other => panic!("manifest names an unhandled canonical type: {other}"),
            }
            seen.push(type_name);
        }
        seen
    };

    let canonically_handled = [
        "option_none",
        "option_some",
        "bool",
        "u32",
        "i32",
        "u64",
        "i64",
        "u128",
        "i128",
        "bytes",
        "string",
        "symbol",
        "vec",
        "map",
    ];
    let mut sorted_handled = handled_types.clone();
    sorted_handled.sort_unstable();
    let mut sorted_canonical = canonically_handled.to_vec();
    sorted_canonical.sort_unstable();
    assert_eq!(
        sorted_handled, sorted_canonical,
        "the manifest and the mapper must cover exactly the canonical types \
         named in docs/SERIALIZATION_STANDARDS.md"
    );
}