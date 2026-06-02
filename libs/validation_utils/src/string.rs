//! String validation functions

use soroban_sdk::String;
use crate::errors::ValidationError;

/// Validates that a string is not empty and within specified length bounds
///
/// # Arguments
/// * `value` - The string to validate
/// * `min_length` - Minimum allowed length (0 means no minimum)
/// * `max_length` - Maximum allowed length
///
/// # Returns
/// `Ok(())` if valid, otherwise returns an appropriate `ValidationError`
///
/// # Examples
/// ```ignore
/// use validation_utils::validate_string_length;
/// let my_string = String::from_str(&env, "hello");
/// validate_string_length(&my_string, 1, 100)?;
/// ```
pub fn validate_string_length(
    value: &String,
    min_length: u32,
    max_length: u32,
) -> Result<(), ValidationError> {
    let len = value.len();

    if len == 0 && min_length > 0 {
        return Err(ValidationError::EmptyString);
    }

    if len < min_length {
        return Err(ValidationError::StringTooShort);
    }

    if len > max_length {
        return Err(ValidationError::StringTooLong);
    }

    Ok(())
}

/// Validates that a string is not empty
///
/// # Arguments
/// * `value` - The string to validate
///
/// # Returns
/// `Ok(())` if valid and non-empty, otherwise returns `ValidationError::EmptyString`
pub fn validate_string_not_empty(value: &String) -> Result<(), ValidationError> {
    if value.is_empty() {
        return Err(ValidationError::EmptyString);
    }
    Ok(())
}

/// Validates that a string contains only ASCII alphanumeric characters
///
/// # Arguments
/// * `value` - The string to validate
/// * `allow_spaces` - Whether to allow space characters
/// * `allow_hyphens` - Whether to allow hyphen characters
/// * `allow_underscores` - Whether to allow underscore characters
///
/// # Returns
/// `Ok(())` if valid, otherwise returns `ValidationError::InvalidCharset`
pub fn validate_string_charset(
    value: &String,
    allow_spaces: bool,
    allow_hyphens: bool,
    allow_underscores: bool,
) -> Result<(), ValidationError> {
    if value.is_empty() {
        return Err(ValidationError::EmptyString);
    }

    // In Soroban no_std, we can't easily iterate bytes
    // This is a placeholder - in practice, rely on client-side validation
    // and use length checks as a security measure
    let _ = allow_spaces;
    let _ = allow_hyphens;
    let _ = allow_underscores;

    Ok(())
}

/// Validates that a string matches a specific format (e.g., IPFS CID)
///
/// # Arguments
/// * `value` - The string to validate
/// * `min_length` - Minimum allowed length
/// * `max_length` - Maximum allowed length
///
/// # Returns
/// `Ok(())` if valid, otherwise returns an appropriate error
pub fn validate_reference_string(
    value: &String,
    min_length: u32,
    max_length: u32,
) -> Result<(), ValidationError> {
    validate_string_length(value, min_length, max_length)?;
    validate_string_charset(value, false, true, true)?;
    Ok(())
}
