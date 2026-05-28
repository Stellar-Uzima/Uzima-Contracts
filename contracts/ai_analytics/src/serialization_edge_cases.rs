#![cfg(test)]
extern crate alloc;

use crate::serialization_utils::{
    SafeSerialize, SerializationError, SerializationUtils, MAX_COLLECTION_SIZE, MAX_STRING_LENGTH,
};
use soroban_sdk::{testutils::Address as _, Address, Bytes, BytesN, Env, Map, String, Vec};

#[test]
fn test_validate_string_empty() {
    let env = Env::default();
    let empty_str = String::from_str(&env, "");
    assert_eq!(
        SerializationUtils::validate_string(&empty_str),
        Err(SerializationError::EmptyCollection)
    );
}

#[test]
fn test_validate_string_valid() {
    let env = Env::default();
    let valid_str = String::from_str(&env, "valid metadata");
    assert_eq!(SerializationUtils::validate_string(&valid_str), Ok(()));
}

#[test]
fn test_validate_vec_empty() {
    let env = Env::default();
    let empty_vec: Vec<u32> = Vec::new(&env);
    assert_eq!(
        SerializationUtils::validate_vec(&empty_vec),
        Err(SerializationError::EmptyCollection)
    );
}

#[test]
fn test_validate_vec_valid() {
    let env = Env::default();
    let mut vec = Vec::new(&env);
    vec.push_back(1);
    assert_eq!(SerializationUtils::validate_vec(&vec), Ok(()));
}

#[test]
fn test_validate_map_empty() {
    let env = Env::default();
    let empty_map: Map<u32, u32> = Map::new(&env);
    assert_eq!(
        SerializationUtils::validate_map(&empty_map),
        Err(SerializationError::EmptyCollection)
    );
}

#[test]
fn test_validate_map_valid() {
    let env = Env::default();
    let mut map: Map<u32, u32> = Map::new(&env);
    map.set(1, 100);
    assert_eq!(SerializationUtils::validate_map(&map), Ok(()));
}

#[test]
fn test_validate_bytes_empty() {
    let env = Env::default();
    let empty_bytes = Bytes::new(&env);
    assert_eq!(
        SerializationUtils::validate_bytes(&empty_bytes),
        Err(SerializationError::InvalidBytes)
    );
}

#[test]
fn test_validate_bytesn_empty() {
    let env = Env::default();
    let empty_bytesn = BytesN::from_array(&env, &[0u8; 32]);
    assert_eq!(
        SerializationUtils::validate_bytesn(&empty_bytesn),
        Err(SerializationError::InvalidBytes)
    );
}

#[test]
fn test_validate_bytesn_valid() {
    let env = Env::default();
    let mut arr = [0u8; 32];
    arr[0] = 1;
    let valid_bytesn = BytesN::from_array(&env, &arr);
    assert_eq!(SerializationUtils::validate_bytesn(&valid_bytesn), Ok(()));
}

#[test]
fn test_validate_address_valid() {
    let env = Env::default();
    let addr = Address::generate(&env);
    assert_eq!(SerializationUtils::validate_address(&env, &addr), Ok(()));
}

#[test]
fn test_validate_metadata_zero() {
    assert_eq!(
        SerializationUtils::validate_metadata_value(0),
        Err(SerializationError::ZeroValueMetadata)
    );
}

#[test]
fn test_validate_metadata_valid() {
    assert_eq!(SerializationUtils::validate_metadata_value(42), Ok(()));
}

struct MockModelMetadata {
    pub name: String,
    pub description: String,
    pub participant: Address,
    pub score: u64,
}

impl SafeSerialize for MockModelMetadata {
    fn safe_serialize(&self, env: &Env) -> Result<(), SerializationError> {
        SerializationUtils::validate_string(&self.name)?;
        SerializationUtils::validate_string(&self.description)?;
        SerializationUtils::validate_address(env, &self.participant)?;
        SerializationUtils::validate_metadata_value(self.score)?;
        Ok(())
    }
}

#[test]
fn test_mock_model_safe_serialize() {
    let env = Env::default();
    let mut model = MockModelMetadata {
        name: String::from_str(&env, "Model-A"),
        description: String::from_str(&env, "Valid Description"),
        participant: Address::generate(&env),
        score: 100,
    };
    assert_eq!(model.safe_serialize(&env), Ok(()));
    
    model.name = String::from_str(&env, "");
    assert_eq!(model.safe_serialize(&env), Err(SerializationError::EmptyCollection));
}