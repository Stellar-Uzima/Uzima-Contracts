#![cfg(test)]

use soroban_sdk::{Bytes, Env, String, Vec};

use crate::{FieldUpdate, PartialUpdate, PartialUpdateError};

fn field(env: &Env, path: &str, val: u32) -> FieldUpdate {
    let mut b = Bytes::new(env);
    // Encode u32 as 4 little-endian bytes
    b.push_back((val & 0xFF) as u8);
    b.push_back(((val >> 8) & 0xFF) as u8);
    b.push_back(((val >> 16) & 0xFF) as u8);
    b.push_back(((val >> 24) & 0xFF) as u8);
    FieldUpdate {
        path: String::from_str(env, path),
        value: b,
    }
}

#[test]
fn test_builder_adds_fields() {
    let env = Env::default();
    let v1 = Bytes::from_array(&env, &[1, 0, 0, 0]);
    let v2 = Bytes::from_array(&env, &[2, 0, 0, 0]);

    let update = PartialUpdate::new(&env)
        .set(&env, "name", &v1)
        .set(&env, "email", &v2)
        .build()
        .unwrap();

    assert_eq!(update.len(), 2);
    assert!(!update.is_empty());
}

#[test]
fn test_empty_build_errors() {
    let env = Env::default();
    let result = PartialUpdate::new(&env).build(&env);
    assert_eq!(result, Err(PartialUpdateError::EmptyUpdate));
}

#[test]
fn test_has_field() {
    let env = Env::default();
    let v = Bytes::from_array(&env, &[1]);

    let update = PartialUpdate::new(&env)
        .set(&env, "name", &v)
        .build()
        .unwrap();

    assert!(update.has_field(&env, "name"));
    assert!(!update.has_field(&env, "email"));
}

#[test]
fn test_merge_replaces_existing() {
    let env = Env::default();

    let mut existing = Vec::new(&env);
    existing.push_back(field(&env, "name", 10));
    existing.push_back(field(&env, "email", 20));

    let update = PartialUpdate::new(&env)
        .set(&env, "name", &Bytes::from_array(&env, &[99, 0, 0, 0]))
        .build()
        .unwrap();

    let merged = update.merge_with_existing(&env, &existing).unwrap();
    assert_eq!(merged.len(), 2);

    let name_entry = merged.get(0).unwrap();
    assert_eq!(name_entry.path, String::from_str(&env, "name"));
    assert_eq!(name_entry.value, Bytes::from_array(&env, &[99, 0, 0, 0]));
}

#[test]
fn test_merge_adds_new_field() {
    let env = Env::default();

    let mut existing = Vec::new(&env);
    existing.push_back(field(&env, "name", 10));

    let update = PartialUpdate::new(&env)
        .set(&env, "phone", &Bytes::from_array(&env, &[42, 0, 0, 0]))
        .build()
        .unwrap();

    let merged = update.merge_with_existing(&env, &existing).unwrap();
    assert_eq!(merged.len(), 2);

    let phone_entry = merged.get(1).unwrap();
    assert_eq!(phone_entry.path, String::from_str(&env, "phone"));
}

#[test]
fn test_merge_nested_path() {
    let env = Env::default();

    let mut existing = Vec::new(&env);
    existing.push_back(field(&env, "address.city", 5));

    let update = PartialUpdate::new(&env)
        .set(&env, "address.city", &Bytes::from_array(&env, &[7, 0, 0, 0]))
        .build()
        .unwrap();

    let merged = update.merge_with_existing(&env, &existing).unwrap();
    assert_eq!(merged.len(), 1);

    let entry = merged.get(0).unwrap();
    assert_eq!(entry.value, Bytes::from_array(&env, &[7, 0, 0, 0]));
}

#[test]
fn test_merge_empty_existing() {
    let env = Env::default();

    let existing = Vec::new(&env);

    let update = PartialUpdate::new(&env)
        .set(&env, "field_a", &Bytes::from_array(&env, &[1]))
        .build()
        .unwrap();

    let merged = update.merge_with_existing(&env, &existing).unwrap();
    assert_eq!(merged.len(), 1);
}
