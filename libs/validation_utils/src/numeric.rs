//! Numeric validation functions

use crate::errors::ValidationError;

/// Validates that a numeric value is not negative
///
/// # Arguments
/// * `value` - The value to validate
/// * `allow_zero` - Whether to allow zero value
///
/// # Returns
/// `Ok(())` if valid (positive or zero if allowed), otherwise returns `ValidationError::NegativeValueNotAllowed`
pub fn validate_non_negative(value: i128, allow_zero: bool) -> Result<(), ValidationError> {
    if value < 0 {
        return Err(ValidationError::NegativeValueNotAllowed);
    }

    if !allow_zero && value == 0 {
        return Err(ValidationError::ZeroValueNotAllowed);
    }

    Ok(())
}

/// Validates that an unsigned numeric value is within a specific range
///
/// # Arguments
/// * `value` - The value to validate
/// * `min` - Minimum allowed value (inclusive)
/// * `max` - Maximum allowed value (inclusive)
///
/// # Returns
/// `Ok(())` if valid, otherwise returns an appropriate error
pub fn validate_u32_range(value: u32, min: u32, max: u32) -> Result<(), ValidationError> {
    if value < min {
        return Err(ValidationError::ValueTooSmall);
    }

    if value > max {
        return Err(ValidationError::ValueTooLarge);
    }

    Ok(())
}

/// Validates that an unsigned numeric value is within a specific range
///
/// # Arguments
/// * `value` - The value to validate
/// * `min` - Minimum allowed value (inclusive)
/// * `max` - Maximum allowed value (inclusive)
///
/// # Returns
/// `Ok(())` if valid, otherwise returns an appropriate error
pub fn validate_u64_range(value: u64, min: u64, max: u64) -> Result<(), ValidationError> {
    if value < min {
        return Err(ValidationError::ValueTooSmall);
    }

    if value > max {
        return Err(ValidationError::ValueTooLarge);
    }

    Ok(())
}

/// Validates that a signed numeric value is within a specific range
///
/// # Arguments
/// * `value` - The value to validate
/// * `min` - Minimum allowed value (inclusive)
/// * `max` - Maximum allowed value (inclusive)
///
/// # Returns
/// `Ok(())` if valid, otherwise returns an appropriate error
pub fn validate_i128_range(value: i128, min: i128, max: i128) -> Result<(), ValidationError> {
    if value < min {
        return Err(ValidationError::ValueTooSmall);
    }

    if value > max {
        return Err(ValidationError::ValueTooLarge);
    }

    Ok(())
}

/// Validates that a percentage is between 0 and 100 (or 0 and 10_000 for basis points)
///
/// # Arguments
/// * `value` - The percentage value to validate
/// * `basis_points` - If true, validates against 10_000 (0-10,000 bps), else against 100 (0-100%)
///
/// # Returns
/// `Ok(())` if valid, otherwise returns an appropriate error
pub fn validate_percentage(value: u32, basis_points: bool) -> Result<(), ValidationError> {
    let max = if basis_points { 10_000 } else { 100 };
    validate_u32_range(value, 0, max)
}

/// Validates that a numeric value represents a valid count
///
/// # Arguments
/// * `count` - The count value to validate
/// * `max_allowed` - Maximum allowed count
///
/// # Returns
/// `Ok(())` if valid, otherwise returns an appropriate error
pub fn validate_count(count: u32, max_allowed: u32) -> Result<(), ValidationError> {
    if count > max_allowed {
        return Err(ValidationError::ValueTooLarge);
    }
    Ok(())
}

/// Validates that a numeric value is not zero
///
/// # Arguments
/// * `value` - The value to validate
///
/// # Returns
/// `Ok(())` if non-zero, otherwise returns `ValidationError::ZeroValueNotAllowed`
pub fn validate_non_zero_u32(value: u32) -> Result<(), ValidationError> {
    if value == 0 {
        return Err(ValidationError::ZeroValueNotAllowed);
    }
    Ok(())
}

/// Validates that a numeric value is not zero
///
/// # Arguments
/// * `value` - The value to validate
///
/// # Returns
/// `Ok(())` if non-zero, otherwise returns `ValidationError::ZeroValueNotAllowed`
pub fn validate_non_zero_u64(value: u64) -> Result<(), ValidationError> {
    if value == 0 {
        return Err(ValidationError::ZeroValueNotAllowed);
    }
    Ok(())
}
