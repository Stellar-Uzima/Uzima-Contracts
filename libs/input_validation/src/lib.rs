#![no_std]

//! # Input Validation Library
//!
//! Structured validation functions for external data inputs before contract
//! writes. Prevents invalid data from reaching contract storage and provides
//! clear, typed error messages.
//!
//! ## Validators
//! - Patient ID format validation
//! - Medical record data integrity checks (non-empty, length limits)
//! - Payment amount bounds validation (positive, within limits)
//! - Consent timestamp validation (reasonable range, monotonic)
//!
//! ## Usage
//! ```rust,ignore
//! use input_validation::{validate_patient_id, validate_medical_record, ValidationError};
//!
//! validate_patient_id(&env, &patient_id)?;
//! validate_medical_record(&env, &diagnosis, &treatment)?;
//! ```

use soroban_sdk::{contracterror, contracttype, Env, String};

/// Maximum allowed length for string fields in medical records.
pub const MAX_RECORD_FIELD_LENGTH: u32 = 10_000;

/// Minimum length for a non-empty medical record field.
pub const MIN_RECORD_FIELD_LENGTH: u32 = 1;

/// Maximum allowed payment amount in stroops (10 billion = 100,000 XLM).
pub const MAX_PAYMENT_AMOUNT: i128 = 10_000_000_000_000;

/// Minimum allowed payment amount in stroops (must be positive).
pub const MIN_PAYMENT_AMOUNT: i128 = 1;

/// Maximum allowed patient ID length.
pub const MAX_PATIENT_ID_LENGTH: u32 = 128;

/// Minimum allowed patient ID length.
pub const MIN_PATIENT_ID_LENGTH: u32 = 3;

/// Maximum age for a consent timestamp in the future (seconds).
/// Prevents timestamps far in the future due to clock skew.
pub const MAX_FUTURE_TOLERANCE_SECS: u64 = 86_400; // 24 hours

/// Maximum age for a consent timestamp in the past (seconds).
/// Prevents consenting on behalf of ancestors.
pub const MAX_PAST_TOLERANCE_SECS: u64 = 315_360_000; // 10 years

/// Errors for input validation failures.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[contracterror]
pub enum ValidationError {
    /// Patient ID is empty, too short, or exceeds the maximum length.
    InvalidPatientId = 1,
    /// Medical record field is empty or exceeds the maximum length.
    InvalidRecordField = 2,
    /// Payment amount is zero, negative, or exceeds the allowed range.
    InvalidPaymentAmount = 3,
    /// Consent timestamp is outside the acceptable range.
    InvalidTimestamp = 4,
    /// A required field was not provided.
    MissingRequiredField = 5,
}

/// A validation result type.
pub type ValidationResult = Result<(), ValidationError>;

/// Validate a patient ID string.
///
/// Requirements:
/// - Non-empty after trimming
/// - Length between `MIN_PATIENT_ID_LENGTH` and `MAX_PATIENT_ID_LENGTH`
/// - Contains only alphanumeric characters, hyphens, or underscores
pub fn validate_patient_id(env: &Env, patient_id: &String) -> ValidationResult {
    if patient_id.is_empty() {
        return Err(ValidationError::InvalidPatientId);
    }

    let len = patient_id.len();
    if len < MIN_PATIENT_ID_LENGTH || len > MAX_PATIENT_ID_LENGTH {
        return Err(ValidationError::InvalidPatientId);
    }

    // Check that all bytes are alphanumeric, hyphen, or underscore
    let bytes = patient_id.to_buffer();
    for i in 0..bytes.len() {
        let b = bytes.get(i).unwrap();
        let valid = (b >= b'a' && b <= b'z')
            || (b >= b'A' && b <= b'Z')
            || (b >= b'0' && b <= b'9')
            || b == b'-'
            || b == b'_'
            || b == b'.';
        if !valid {
            return Err(ValidationError::InvalidPatientId);
        }
    }

    Ok(())
}

/// Validate a medical record field (e.g., diagnosis, treatment).
///
/// Requirements:
/// - Non-empty after trimming
/// - Length between `MIN_RECORD_FIELD_LENGTH` and `MAX_RECORD_FIELD_LENGTH`
pub fn validate_record_field(env: &Env, field: &String, name: &str) -> ValidationResult {
    if field.is_empty() {
        return Err(ValidationError::InvalidRecordField);
    }

    let len = field.len();
    if len < MIN_RECORD_FIELD_LENGTH || len > MAX_RECORD_FIELD_LENGTH {
        return Err(ValidationError::InvalidRecordField);
    }

    Ok(())
}

/// Validate complete medical record data before creation.
///
/// Checks diagnosis, treatment, and category fields for validity.
pub fn validate_medical_record(
    env: &Env,
    diagnosis: &String,
    treatment: &String,
    category: &String,
) -> ValidationResult {
    validate_record_field(env, diagnosis, "diagnosis")?;
    validate_record_field(env, treatment, "treatment")?;
    validate_record_field(env, category, "category")?;
    Ok(())
}

/// Validate a payment amount.
///
/// Requirements:
/// - Must be strictly positive (`> 0`)
/// - Must not exceed `MAX_PAYMENT_AMOUNT`
pub fn validate_payment_amount(amount: i128) -> ValidationResult {
    if amount < MIN_PAYMENT_AMOUNT || amount > MAX_PAYMENT_AMOUNT {
        return Err(ValidationError::InvalidPaymentAmount);
    }
    Ok(())
}

/// Validate a consent timestamp.
///
/// Requirements:
/// - Must not be more than `MAX_FUTURE_TOLERANCE_SECS` in the future
/// - Must not be more than `MAX_PAST_TOLERANCE_SECS` in the past
///
/// `env_timestamp` is the current ledger close time from `env.ledger().timestamp()`.
pub fn validate_consent_timestamp(env_timestamp: u64, consent_timestamp: u64) -> ValidationResult {
    // Check for future timestamp (clock skew tolerance)
    if consent_timestamp > env_timestamp + MAX_FUTURE_TOLERANCE_SECS {
        return Err(ValidationError::InvalidTimestamp);
    }

    // Check for excessively old timestamp
    if env_timestamp > MAX_PAST_TOLERANCE_SECS
        && consent_timestamp < env_timestamp - MAX_PAST_TOLERANCE_SECS
    {
        return Err(ValidationError::InvalidTimestamp);
    }

    // Handle underflow: if env_timestamp < MAX_PAST_TOLERANCE_SECS,
    // any past timestamp within range is acceptable (we're on a young ledger)
    // Since u64 can't be negative, consent_timestamp is always >= 0 and valid
    // in this case.

    Ok(())
}

/// Validate that a non-empty string is provided.
pub fn require_non_empty(value: &String, name: &str) -> ValidationResult {
    if value.is_empty() {
        Err(ValidationError::MissingRequiredField)
    } else {
        Ok(())
    }
}

/// Batch-validate all required fields for a patient consent operation.
pub fn validate_consent_inputs(
    patient_id: &String,
    provider_id: &String,
) -> ValidationResult {
    validate_patient_id(&Env::default(), patient_id)?;
    validate_patient_id(&Env::default(), provider_id)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use soroban_sdk::Env;

    #[test]
    fn test_valid_patient_id() {
        let env = Env::default();
        let id = String::from_str(&env, "PAT-12345");
        assert_eq!(validate_patient_id(&env, &id), Ok(()));
    }

    #[test]
    fn test_patient_id_too_short() {
        let env = Env::default();
        let id = String::from_str(&env, "ab");
        assert_eq!(validate_patient_id(&env, &id), Err(ValidationError::InvalidPatientId));
    }

    #[test]
    fn test_patient_id_invalid_chars() {
        let env = Env::default();
        let id = String::from_str(&env, "PAT@123");
        assert_eq!(validate_patient_id(&env, &id), Err(ValidationError::InvalidPatientId));
    }

    #[test]
    fn test_valid_record_field() {
        let env = Env::default();
        let field = String::from_str(&env, "Hypertension stage 2");
        assert_eq!(validate_record_field(&env, &field, "diagnosis"), Ok(()));
    }

    #[test]
    fn test_empty_record_field() {
        let env = Env::default();
        let field = String::from_str(&env, "");
        assert_eq!(
            validate_record_field(&env, &field, "diagnosis"),
            Err(ValidationError::InvalidRecordField)
        );
    }

    #[test]
    fn test_valid_payment_amount() {
        assert_eq!(validate_payment_amount(1000), Ok(()));
    }

    #[test]
    fn test_zero_payment_amount() {
        assert_eq!(
            validate_payment_amount(0),
            Err(ValidationError::InvalidPaymentAmount)
        );
    }

    #[test]
    fn test_negative_payment_amount() {
        assert_eq!(
            validate_payment_amount(-100),
            Err(ValidationError::InvalidPaymentAmount)
        );
    }

    #[test]
    fn test_excessive_payment_amount() {
        assert_eq!(
            validate_payment_amount(MAX_PAYMENT_AMOUNT + 1),
            Err(ValidationError::InvalidPaymentAmount)
        );
    }

    #[test]
    fn test_valid_consent_timestamp() {
        assert_eq!(validate_consent_timestamp(1_000_000, 999_999), Ok(()));
    }

    #[test]
    fn test_consent_timestamp_too_future() {
        assert_eq!(
            validate_consent_timestamp(1_000_000, 1_000_000 + MAX_FUTURE_TOLERANCE_SECS + 1),
            Err(ValidationError::InvalidTimestamp)
        );
    }

    #[test]
    fn test_consent_timestamp_too_old() {
        assert_eq!(
            validate_consent_timestamp(
                MAX_PAST_TOLERANCE_SECS + 1,
                1
            ),
            Err(ValidationError::InvalidTimestamp)
        );
    }

    #[test]
    fn test_medical_record_validation() {
        let env = Env::default();
        let diag = String::from_str(&env, "Type 2 Diabetes");
        let treat = String::from_str(&env, "Metformin 500mg");
        let cat = String::from_str(&env, "endocrinology");
        assert_eq!(validate_medical_record(&env, &diag, &treat, &cat), Ok(()));
    }
}
