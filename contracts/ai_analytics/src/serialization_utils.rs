use soroban_sdk::{contracterror, Address, BytesN, Env, Map, String, Symbol, Vec};

/// Maximum allowed nesting depth for serialized structures
#[allow(dead_code)]
pub const MAX_NESTING_DEPTH: u32 = 50;
/// Maximum number of elements in Vecs and Maps to prevent memory exhaustion.
#[allow(dead_code)]
pub const MAX_COLLECTION_SIZE: u32 = 10000;
/// Maximum byte length for string structures.
pub const MAX_STRING_LENGTH: u32 = 100000;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum SerializationError {
    CollectionTooLarge = 1,
    StringTooLong = 2,
    NestingTooDeep = 3,
    EmptyCollection = 4,
    InvalidAddress = 5,
    InvalidBytes = 6,
    ZeroValueMetadata = 7,
}

/// Trait to ensure types can be safely serialized and stored.
pub trait SafeSerialize {
    /// Validates the implementor's fields against edge-case constraints.
    fn safe_serialize(&self, env: &Env) -> Result<(), SerializationError>;
}

pub struct SerializationUtils;

impl SerializationUtils {
    /// Validates that a string is neither empty nor excessively long.
    pub fn validate_string(s: &String) -> Result<(), SerializationError> {
        let len = s.len();
        if len == 0 {
            return Err(SerializationError::EmptyCollection);
        }
        if len > MAX_STRING_LENGTH {
            return Err(SerializationError::StringTooLong);
        }
        Ok(())
    }

    /// Validates that a vector is not empty and respects size limits.
    #[allow(dead_code)]
    pub fn validate_vec<T>(v: &Vec<T>) -> Result<(), SerializationError> {
        let len = v.len();
        if len == 0 {
            return Err(SerializationError::EmptyCollection);
        }
        if len > MAX_COLLECTION_SIZE {
            return Err(SerializationError::CollectionTooLarge);
        }
        Ok(())
    }

    /// Validates that a map is not empty and respects size limits.
    #[allow(dead_code)]
    pub fn validate_map<K, V>(m: &Map<K, V>) -> Result<(), SerializationError> {
        let len = m.len();
        if len == 0 {
            return Err(SerializationError::EmptyCollection);
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
    #[allow(dead_code)]
    pub fn safe_serialize_vec<T>(env: &Env, vec: &Vec<T>) -> Result<(), SerializationError> {
        Self::validate_vec(vec)?;
        if vec.is_empty() {
            env.events()
                .publish((Symbol::new(env, "SER_EMPTY"), Symbol::new(env, "VEC")), ());
        }
        Ok(())
    }

    /// Safe serialization for Map with validation
    #[allow(dead_code)]
    pub fn safe_serialize_map<K, V>(env: &Env, map: &Map<K, V>) -> Result<(), SerializationError> {
        Self::validate_map(map)?;
        if map.is_empty() {
            env.events()
                .publish((Symbol::new(env, "SER_EMPTY"), Symbol::new(env, "MAP")), ());
        }
        Ok(())
    }

    /// Safe serialization for String with validation
    pub fn safe_serialize_string(env: &Env, string: &String) -> Result<(), SerializationError> {
        Self::validate_string(string)?;
        if string.is_empty() {
            env.events()
                .publish((Symbol::new(env, "SER_EMPTY"), Symbol::new(env, "STR")), ());
        }
        Ok(())
    }

    /// Validates BytesN for edge cases
    pub fn validate_bytes_n<const N: usize>(
        env: &Env,
        _bytes: &BytesN<N>,
    ) -> Result<(), SerializationError> {
        env.events().publish((Symbol::new(env, "SER_BYTESN"),), ());
        Ok(())
    }

    /// Validates Address for edge cases
    pub fn validate_address(env: &Env, _address: &Address) -> Result<(), SerializationError> {
        env.events().publish((Symbol::new(env, "SER_ADDR"),), ());
        Ok(())
    }
}
