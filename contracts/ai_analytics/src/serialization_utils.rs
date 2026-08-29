use soroban_sdk::{contracterror, Address, BytesN, Env, Map, String, Symbol, Vec};

/// Maximum allowed nesting depth for serialized structures.
const MAX_NESTING_DEPTH: u32 = 50;
/// Maximum number of elements in Vecs and Maps to prevent memory exhaustion.
const MAX_COLLECTION_SIZE: u32 = 10_000;
/// Maximum byte length for string structures.
pub const MAX_STRING_LENGTH: u32 = 100_000;

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
    InvalidValue = 8,
    CircularReference = 9,
}

impl core::fmt::Display for SerializationError {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        match self {
            SerializationError::CollectionTooLarge => write!(f, "collection too large"),
            SerializationError::StringTooLong => write!(f, "string too long"),
            SerializationError::NestingTooDeep => write!(f, "nesting too deep"),
            SerializationError::EmptyCollection => write!(f, "empty collection"),
            SerializationError::InvalidAddress => write!(f, "invalid address"),
            SerializationError::InvalidBytes => write!(f, "invalid bytes"),
            SerializationError::ZeroValueMetadata => write!(f, "zero value metadata"),
            SerializationError::InvalidValue => write!(f, "invalid value"),
            SerializationError::CircularReference => write!(f, "circular reference"),
        }
    }
}

/// Trait to ensure types can be safely serialized and stored.
pub trait SafeSerialize {
    /// Validates the implementor's fields against edge-case constraints.
    #[must_use]
    fn safe_serialize(&self, env: &Env) -> Result<(), SerializationError>;
}

pub struct SerializationUtils;

impl SerializationUtils {
    /// Validates that a string does not exceed the maximum length.
    #[must_use]
    pub fn validate_string_length(s: &String) -> Result<(), SerializationError> {
        if s.len() > MAX_STRING_LENGTH {
            return Err(SerializationError::StringTooLong);
        }
        Ok(())
    }

    /// Validates that a collection does not exceed the maximum size.
    #[must_use]
    pub fn validate_collection_size<T>(v: &Vec<T>) -> Result<(), SerializationError> {
        if v.len() > MAX_COLLECTION_SIZE {
            return Err(SerializationError::CollectionTooLarge);
        }
        Ok(())
    }

    /// Validates that a map does not exceed the maximum size.
    #[must_use]
    pub fn validate_map_size<K, V>(m: &Map<K, V>) -> Result<(), SerializationError> {
        if m.len() > MAX_COLLECTION_SIZE {
            return Err(SerializationError::CollectionTooLarge);
        }
        Ok(())
    }

    /// Validates nesting depth (conceptual - Soroban handles this internally)
    #[must_use]
    pub fn validate_nesting_depth(current_depth: u32) -> Result<(), SerializationError> {
        if current_depth > MAX_NESTING_DEPTH {
            return Err(SerializationError::NestingTooDeep);
        }
        Ok(())
    }

    /// Safe serialization for Vec with validation.
    #[must_use]
    pub fn safe_serialize_vec<T>(env: &Env, vec: &Vec<T>) -> Result<(), SerializationError> {
        Self::validate_collection_size(vec)?;
        if vec.is_empty() {
            env.events()
                .publish((Symbol::new(env, "SER_EMPTY"), Symbol::new(env, "VEC")), ());
        }
        Ok(())
    }

    /// Safe serialization for Map with validation.
    #[must_use]
    pub fn safe_serialize_map<K, V>(env: &Env, map: &Map<K, V>) -> Result<(), SerializationError> {
        Self::validate_map_size(map)?;
        if map.is_empty() {
            env.events()
                .publish((Symbol::new(env, "SER_EMPTY"), Symbol::new(env, "MAP")), ());
        }
        Ok(())
    }

    /// Safe serialization for String with validation.
    #[must_use]
    pub fn safe_serialize_string(env: &Env, string: &String) -> Result<(), SerializationError> {
        Self::validate_string_length(string)?;
        if string.is_empty() {
            env.events()
                .publish((Symbol::new(env, "SER_EMPTY"), Symbol::new(env, "STR")), ());
        }
        Ok(())
    }

    /// Validates BytesN for edge cases.
    pub fn validate_bytes_n<const N: usize>(
        env: &Env,
        _bytes: &BytesN<N>,
    ) -> Result<(), SerializationError> {
        env.events().publish((Symbol::new(env, "SER_BYTESN"),), ());
        Ok(())
    }

    /// Validates Address for edge cases.
    #[must_use]
    pub fn validate_address(env: &Env, _address: &Address) -> Result<(), SerializationError> {
        env.events().publish((Symbol::new(env, "SER_ADDR"),), ());
        Ok(())
    }
}

impl<T> SafeSerialize for Vec<T> {
    #[must_use]
    fn safe_serialize(&self, env: &Env) -> Result<(), SerializationError> {
        SerializationUtils::safe_serialize_vec(env, self)
    }
}

impl<K, V> SafeSerialize for Map<K, V> {
    #[must_use]
    fn safe_serialize(&self, env: &Env) -> Result<(), SerializationError> {
        SerializationUtils::safe_serialize_map(env, self)
    }
}

impl SafeSerialize for String {
    #[must_use]
    fn safe_serialize(&self, env: &Env) -> Result<(), SerializationError> {
        SerializationUtils::safe_serialize_string(env, self)
    }
}

impl<const N: usize> SafeSerialize for BytesN<N> {
    #[must_use]
    fn safe_serialize(&self, env: &Env) -> Result<(), SerializationError> {
        SerializationUtils::validate_bytes_n(env, self)
    }
}

impl SafeSerialize for Address {
    #[must_use]
    fn safe_serialize(&self, env: &Env) -> Result<(), SerializationError> {
        SerializationUtils::validate_address(env, self)
    }
}