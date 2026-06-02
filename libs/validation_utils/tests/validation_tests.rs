#![cfg(test)]

use validation_utils::{
    validate_string_length, validate_non_negative, validate_u32_range, validate_u64_range,
    validate_i128_range, validate_string_not_empty, validate_non_zero_u32, validate_non_zero_u64,
    validate_percentage, validate_count, ValidationError,
};

// ==================== STRING VALIDATION TESTS ====================

#[test]
fn test_validate_string_length_valid() {
    // Note: In no_std, we can't easily create String without an environment
    // These tests demonstrate the pattern
    
    // Valid string of appropriate length should return Ok
    // Example: let s = String::from_str(&env, "hello");
    // validate_string_length(&s, 1, 100).unwrap();
}

#[test]
fn test_validate_string_empty_when_not_allowed() {
    // Empty string with min > 0 should return EmptyString error
    // Example: let s = String::from_str(&env, "");
    // assert_eq!(validate_string_length(&s, 1, 100), Err(ValidationError::EmptyString));
}

#[test]
fn test_validate_string_too_short() {
    // String shorter than min should return StringTooShort error
}

#[test]
fn test_validate_string_too_long() {
    // String longer than max should return StringTooLong error
}

// ==================== NUMERIC VALIDATION TESTS ====================

#[test]
fn test_validate_non_negative_positive() {
    assert_eq!(validate_non_negative(100, true), Ok(()));
    assert_eq!(validate_non_negative(100, false), Ok(()));
}

#[test]
fn test_validate_non_negative_zero() {
    assert_eq!(validate_non_negative(0, true), Ok(()));
    assert_eq!(
        validate_non_negative(0, false),
        Err(ValidationError::ZeroValueNotAllowed)
    );
}

#[test]
fn test_validate_non_negative_negative() {
    assert_eq!(
        validate_non_negative(-1, true),
        Err(ValidationError::NegativeValueNotAllowed)
    );
    assert_eq!(
        validate_non_negative(-100, false),
        Err(ValidationError::NegativeValueNotAllowed)
    );
}

#[test]
fn test_validate_u32_range_valid() {
    assert_eq!(validate_u32_range(50, 0, 100), Ok(()));
    assert_eq!(validate_u32_range(0, 0, 100), Ok(()));
    assert_eq!(validate_u32_range(100, 0, 100), Ok(()));
}

#[test]
fn test_validate_u32_range_too_small() {
    assert_eq!(
        validate_u32_range(5, 10, 100),
        Err(ValidationError::ValueTooSmall)
    );
}

#[test]
fn test_validate_u32_range_too_large() {
    assert_eq!(
        validate_u32_range(150, 0, 100),
        Err(ValidationError::ValueTooLarge)
    );
}

#[test]
fn test_validate_u64_range_valid() {
    assert_eq!(validate_u64_range(50, 0, 100), Ok(()));
    assert_eq!(validate_u64_range(1000000, 0, 10000000), Ok(()));
}

#[test]
fn test_validate_u64_range_boundaries() {
    assert_eq!(validate_u64_range(0, 0, 100), Ok(()));
    assert_eq!(validate_u64_range(100, 0, 100), Ok(()));
    assert_eq!(
        validate_u64_range(101, 0, 100),
        Err(ValidationError::ValueTooLarge)
    );
}

#[test]
fn test_validate_i128_range_valid() {
    assert_eq!(validate_i128_range(0, -100, 100), Ok(()));
    assert_eq!(validate_i128_range(-50, -100, 100), Ok(()));
    assert_eq!(validate_i128_range(50, -100, 100), Ok(()));
}

#[test]
fn test_validate_i128_range_boundaries() {
    assert_eq!(validate_i128_range(-100, -100, 100), Ok(()));
    assert_eq!(validate_i128_range(100, -100, 100), Ok(()));
    assert_eq!(
        validate_i128_range(-101, -100, 100),
        Err(ValidationError::ValueTooSmall)
    );
    assert_eq!(
        validate_i128_range(101, -100, 100),
        Err(ValidationError::ValueTooLarge)
    );
}

#[test]
fn test_validate_percentage_valid() {
    assert_eq!(validate_percentage(0, false), Ok(()));
    assert_eq!(validate_percentage(50, false), Ok(()));
    assert_eq!(validate_percentage(100, false), Ok(()));
}

#[test]
fn test_validate_percentage_basis_points() {
    assert_eq!(validate_percentage(0, true), Ok(()));
    assert_eq!(validate_percentage(5000, true), Ok(()));
    assert_eq!(validate_percentage(10000, true), Ok(()));
}

#[test]
fn test_validate_percentage_invalid() {
    assert_eq!(
        validate_percentage(101, false),
        Err(ValidationError::ValueTooLarge)
    );
    assert_eq!(
        validate_percentage(10001, true),
        Err(ValidationError::ValueTooLarge)
    );
}

#[test]
fn test_validate_count_valid() {
    assert_eq!(validate_count(0, 100), Ok(()));
    assert_eq!(validate_count(50, 100), Ok(()));
    assert_eq!(validate_count(100, 100), Ok(()));
}

#[test]
fn test_validate_count_exceeds_max() {
    assert_eq!(
        validate_count(101, 100),
        Err(ValidationError::ValueTooLarge)
    );
}

#[test]
fn test_validate_non_zero_u32() {
    assert_eq!(validate_non_zero_u32(1), Ok(()));
    assert_eq!(validate_non_zero_u32(1000000), Ok(()));
    assert_eq!(
        validate_non_zero_u32(0),
        Err(ValidationError::ZeroValueNotAllowed)
    );
}

#[test]
fn test_validate_non_zero_u64() {
    assert_eq!(validate_non_zero_u64(1), Ok(()));
    assert_eq!(validate_non_zero_u64(9999999999), Ok(()));
    assert_eq!(
        validate_non_zero_u64(0),
        Err(ValidationError::ZeroValueNotAllowed)
    );
}

// ==================== PROPERTY-BASED TEST PATTERNS ====================

/// Demonstrates monotonic property: if a value is in range (min, max), 
/// it should always pass validation
#[test]
fn property_u32_range_monotonic() {
    for value in 0u32..=100 {
        assert_eq!(validate_u32_range(value, 0, 100), Ok(()));
    }
}

/// Property: out of range values should always fail
#[test]
fn property_u32_range_out_of_bounds() {
    assert_eq!(
        validate_u32_range(101, 0, 100),
        Err(ValidationError::ValueTooLarge)
    );
    assert_eq!(
        validate_u32_range(u32::MAX, 0, 100),
        Err(ValidationError::ValueTooLarge)
    );
}

/// Property: boundaries should be inclusive
#[test]
fn property_u64_range_boundaries_inclusive() {
    let min = 100u64;
    let max = 200u64;
    
    assert_eq!(validate_u64_range(min, min, max), Ok(()));
    assert_eq!(validate_u64_range(max, min, max), Ok(()));
    assert_eq!(validate_u64_range(min - 1, min, max), Err(ValidationError::ValueTooSmall));
    assert_eq!(validate_u64_range(max + 1, min, max), Err(ValidationError::ValueTooLarge));
}

/// Property: non-negative validation should always accept positive numbers
#[test]
fn property_non_negative_all_positive() {
    for value in 1i128..=1000 {
        assert_eq!(validate_non_negative(value, false), Ok(()));
    }
}

/// Property: non-negative validation should reject all negative numbers
#[test]
fn property_non_negative_all_negative() {
    for value in (-1000i128)..=-1 {
        assert_eq!(
            validate_non_negative(value, false),
            Err(ValidationError::NegativeValueNotAllowed)
        );
    }
}

/// Property: percentage values should be within [0, 100]
#[test]
fn property_percentage_range() {
    for pct in 0u32..=100 {
        assert_eq!(validate_percentage(pct, false), Ok(()));
    }
    
    assert_eq!(
        validate_percentage(101, false),
        Err(ValidationError::ValueTooLarge)
    );
}

/// Property: basis points should be within [0, 10000]
#[test]
fn property_basis_points_range() {
    assert_eq!(validate_percentage(0, true), Ok(()));
    assert_eq!(validate_percentage(10000, true), Ok(()));
    assert_eq!(
        validate_percentage(10001, true),
        Err(ValidationError::ValueTooLarge)
    );
}

/// Property: if validation passes, the value is always within bounds
#[test]
fn property_validation_consistency() {
    for min in 0u32..10 {
        for max in (min + 1)..20 {
            for value in min..=max {
                assert_eq!(validate_u32_range(value, min, max), Ok(()));
            }
        }
    }
}
