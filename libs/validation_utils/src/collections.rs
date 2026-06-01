//! Collection validation functions

use soroban_sdk::Vec;
use crate::errors::ValidationError;

/// Validates that a collection is not empty
///
/// # Arguments
/// * `collection_len` - The length of the collection
///
/// # Returns
/// `Ok(())` if valid (non-empty), otherwise returns `ValidationError::CollectionEmpty`
pub fn validate_collection_not_empty(collection_len: u32) -> Result<(), ValidationError> {
    if collection_len == 0 {
        return Err(ValidationError::CollectionEmpty);
    }
    Ok(())
}

/// Validates that a collection size is within allowed bounds
///
/// # Arguments
/// * `collection_len` - The length of the collection
/// * `min_size` - Minimum allowed size (0 means no minimum)
/// * `max_size` - Maximum allowed size
///
/// # Returns
/// `Ok(())` if valid, otherwise returns an appropriate error
pub fn validate_collection_size(
    collection_len: u32,
    min_size: u32,
    max_size: u32,
) -> Result<(), ValidationError> {
    if collection_len < min_size {
        if min_size > 0 {
            return Err(ValidationError::CollectionEmpty);
        }
    }

    if collection_len > max_size {
        return Err(ValidationError::CollectionTooLarge);
    }

    Ok(())
}

/// Validates that a vector is not empty
///
/// # Arguments
/// * `vec` - The vector to validate
///
/// # Returns
/// `Ok(())` if valid (non-empty), otherwise returns `ValidationError::CollectionEmpty`
pub fn validate_vector_not_empty<T>(vec: &Vec<T>) -> Result<(), ValidationError> {
    if vec.is_empty() {
        return Err(ValidationError::CollectionEmpty);
    }
    Ok(())
}

/// Validates that a vector size is within allowed bounds
///
/// # Arguments
/// * `vec` - The vector to validate
/// * `min_size` - Minimum allowed size (0 means no minimum)
/// * `max_size` - Maximum allowed size
///
/// # Returns
/// `Ok(())` if valid, otherwise returns an appropriate error
pub fn validate_vector_size<T>(
    vec: &Vec<T>,
    min_size: u32,
    max_size: u32,
) -> Result<(), ValidationError> {
    validate_collection_size(vec.len(), min_size, max_size)
}
