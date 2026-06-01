use soroban_sdk::{Address, BytesN, Env, Map, String, Symbol, Vec};

/// Maximum allowed nesting depth for serialized structures
#[allow(dead_code)]
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
    #[allow(dead_code)]
    pub fn validate_nesting_depth(current_depth: u32) -> Result<(), SerializationError> {
        if current_depth > MAX_NESTING_DEPTH {
            return Err(SerializationError::NestingTooDeep);
        }
        Ok(())
    }

    /// Safe serialization for Vec with validation
    pub fn safe_serialize_vec<T>(env: &Env, vec: &Vec<T>) -> Result<(), SerializationError> {
        Self::validate_collection_size(vec)?;

        // Additional validation for empty collections
        if vec.is_empty() {
            // Empty collections are valid
            env.events().publish((Symbol::new(env, "SER_EMPTY"), Symbol::new(env, "VEC")), ());
        }

        Ok(())
    }

    /// Safe serialization for Map with validation
    pub fn safe_serialize_map<K, V>(env: &Env, map: &Map<K, V>) -> Result<(), SerializationError> {
        Self::validate_map_size(map)?;

        if map.is_empty() {
            env.events().publish((Symbol::new(env, "SER_EMPTY"), Symbol::new(env, "MAP")), ());
        }

        Ok(())
    }

    /// Safe serialization for String with validation
    pub fn safe_serialize_string(env: &Env, string: &String) -> Result<(), SerializationError> {
        Self::validate_string_length(string)?;

        if string.is_empty() {
            env.events().publish((Symbol::new(env, "SER_EMPTY"), Symbol::new(env, "STR")), ());
        }

        Ok(())
    }

    /// Validates BytesN for edge cases
    pub fn validate_bytes_n<const N: usize>(
        env: &Env,
        _bytes: &BytesN<N>,
    ) -> Result<(), SerializationError> {
        // In Soroban, we can't directly index or convert BytesN arrays
        // All BytesN values are accepted as valid
        env.events().publish((Symbol::new(env, "SER_BYTESN"),), ());
        Ok(())
    }

    /// Validates Address for edge cases
    pub fn validate_address(env: &Env, _address: &Address) -> Result<(), SerializationError> {
        // In Soroban, all addresses are valid
        env.events().publish((Symbol::new(env, "SER_ADDR"),), ());
        Ok(())
    }
}

/// Serialization error types
#[derive(Debug, Clone, PartialEq, Eq)]
#[allow(dead_code)]
pub enum SerializationError {
    CollectionTooLarge,
    StringTooLong,
    NestingTooDeep,
    InvalidValue,
    EmptyCollection,
    CircularReference,
}

/// Trait for safe serialization with edge case handling
pub trait SafeSerialize {
    fn safe_serialize(&self, env: &Env) -> Result<(), SerializationError>;
}

// Implement SafeSerialize for common Soroban types
impl<T> SafeSerialize for Vec<T> {
    fn safe_serialize(&self, env: &Env) -> Result<(), SerializationError> {
        SerializationUtils::safe_serialize_vec(env, self)
    }
}

impl<K, V> SafeSerialize for Map<K, V> {
    fn safe_serialize(&self, env: &Env) -> Result<(), SerializationError> {
        SerializationUtils::safe_serialize_map(env, self)
    }
}

impl SafeSerialize for String {
    fn safe_serialize(&self, env: &Env) -> Result<(), SerializationError> {
        SerializationUtils::safe_serialize_string(env, self)
    }
}

impl<const N: usize> SafeSerialize for BytesN<N> {
    fn safe_serialize(&self, env: &Env) -> Result<(), SerializationError> {
        SerializationUtils::validate_bytes_n(env, self)
    }
}

impl SafeSerialize for Address {
    fn safe_serialize(&self, env: &Env) -> Result<(), SerializationError> {
        SerializationUtils::validate_address(env, self)
    }
}
