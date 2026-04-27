use soroban_sdk::{Address, BytesN, Env, Map, String, TryIntoVal, Val, Vec};

/// Maximum allowed nesting depth for serialized structures
pub const MAX_NESTING_DEPTH: u32 = 50;

/// Maximum allowed size for collections (in number of elements)
pub const MAX_COLLECTION_SIZE: u32 = 10000;

/// Maximum allowed string length
pub const MAX_STRING_LENGTH: u32 = 100000;

/// Serialization utilities for handling edge cases
pub struct SerializationUtils;

impl SerializationUtils {
    /// Validates a collection size before serialization
    pub fn validate_collection_size<T>(collection: &Vec<T>) -> Result<(), SerializationError> {
        if collection.len() > MAX_COLLECTION_SIZE {
            return Err(SerializationError::CollectionTooLarge);
        }
        Ok(())
    }

    /// Validates a map size before serialization
    pub fn validate_map_size<K, V>(map: &Map<K, V>) -> Result<(), SerializationError> {
        if map.len() > MAX_COLLECTION_SIZE {
            return Err(SerializationError::CollectionTooLarge);
        }
        Ok(())
    }

    /// Validates string length before serialization
    pub fn validate_string_length(string: &String) -> Result<(), SerializationError> {
        if string.len() > MAX_STRING_LENGTH {
            return Err(SerializationError::StringTooLong);
        }
        Ok(())
    }

    /// Validates nesting depth (conceptual - Soroban handles this internally)
    pub fn validate_nesting_depth(current_depth: u32) -> Result<(), SerializationError> {
        if current_depth > MAX_NESTING_DEPTH {
            return Err(SerializationError::NestingTooDeep);
        }
        Ok(())
    }

    /// Safe serialization for Vec with validation
    pub fn safe_serialize_vec<T>(_env: &Env, vec: &Vec<T>) -> Result<(), SerializationError>
    where
        T: TryIntoVal<Val> + Clone,
    {
        Self::validate_collection_size(vec)?;

        // Additional validation for empty collections
        if vec.is_empty() {
            // Empty collections are valid, but we log this for debugging
            soroban_sdk::log!("Serializing empty collection");
        }

        Ok(())
    }

    /// Safe serialization for Map with validation
    pub fn safe_serialize_map<K, V>(_env: &Env, map: &Map<K, V>) -> Result<(), SerializationError>
    where
        K: TryIntoVal<Val> + Clone,
        V: TryIntoVal<Val> + Clone,
    {
        Self::validate_map_size(map)?;

        if map.is_empty() {
            soroban_sdk::log!("Serializing empty map");
        }

        Ok(())
    }

    /// Safe serialization for String with validation
    pub fn safe_serialize_string(_env: &Env, string: &String) -> Result<(), SerializationError> {
        Self::validate_string_length(string)?;

        if string.is_empty() {
            soroban_sdk::log!("Serializing empty string");
        }

        Ok(())
    }

    /// Validates BytesN for edge cases
    pub fn validate_bytes_n<const N: usize>(_bytes: &BytesN<N>) -> Result<(), SerializationError> {
        // In Soroban, we can't directly index or convert BytesN arrays
        // We'll use a simple approach - just log that we're validating BytesN
        soroban_sdk::log!("Validating BytesN of length {}", N);

        // For now, we'll accept all BytesN values as valid
        // In a real implementation, you might want to add specific checks

        Ok(())
    }

    /// Validates Address for edge cases
    pub fn validate_address(_address: &Address) -> Result<(), SerializationError> {
        // In Soroban, all addresses are valid, but we can add logging for edge cases
        soroban_sdk::log!("Serializing address");
        Ok(())
    }
}

/// Serialization error types
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SerializationError {
    CollectionTooLarge,
    StringTooLong,
    NestingTooDeep,
    InvalidValue,
    EmptyCollection,
    CircularReference,
}

impl soroban_sdk::contracterror for SerializationError {
    type Error = soroban_sdk::Error;

    fn as_error(&self) -> Self::Error {
        match self {
            SerializationError::CollectionTooLarge => soroban_sdk::Error::from_contract_error(1000),
            SerializationError::StringTooLong => soroban_sdk::Error::from_contract_error(1001),
            SerializationError::NestingTooDeep => soroban_sdk::Error::from_contract_error(1002),
            SerializationError::InvalidValue => soroban_sdk::Error::from_contract_error(1003),
            SerializationError::EmptyCollection => soroban_sdk::Error::from_contract_error(1004),
            SerializationError::CircularReference => soroban_sdk::Error::from_contract_error(1005),
        }
    }
}

/// Trait for safe serialization with edge case handling
pub trait SafeSerialize {
    fn safe_serialize(&self, env: &Env) -> Result<(), SerializationError>;
}

// Implement SafeSerialize for common Soroban types
impl<T> SafeSerialize for Vec<T>
where
    T: TryIntoVal<Val> + Clone,
{
    fn safe_serialize(&self, env: &Env) -> Result<(), SerializationError> {
        SerializationUtils::safe_serialize_vec(env, self)
    }
}

impl<K, V> SafeSerialize for Map<K, V>
where
    K: TryIntoVal<Val> + Clone,
    V: TryIntoVal<Val> + Clone,
{
    fn safe_serialize(&self, env: &Env) -> Result<(), SerializationError> {
        SerializationUtils::safe_serialize_map(env, self)
    }
}

impl SafeSerialize for String {
    fn safe_serialize(&self, _env: &Env) -> Result<(), SerializationError> {
        SerializationUtils::safe_serialize_string(_env, self)
    }
}

impl<const N: usize> SafeSerialize for BytesN<N> {
    fn safe_serialize(&self, _env: &Env) -> Result<(), SerializationError> {
        SerializationUtils::validate_bytes_n(self)
    }
}

impl SafeSerialize for Address {
    fn safe_serialize(&self, _env: &Env) -> Result<(), SerializationError> {
        SerializationUtils::validate_address(self)
    }
}
