/// Custom assertions for contract testing
use soroban_sdk::Address;

#[allow(clippy::unwrap_used)]
#[allow(clippy::panic)]

/// Assert two addresses are equal
#[macro_export]
macro_rules! assert_address_eq {
    ($left:expr, $right:expr) => {
        assert_eq!($left, $right, "Address mismatch");
    };
    ($left:expr, $right:expr, $($msg:tt)*) => {
        assert_eq!($left, $right, $($msg)*);
    };
}

/// Assert two addresses are not equal
#[macro_export]
macro_rules! assert_address_ne {
    ($left:expr, $right:expr) => {
        assert_ne!($left, $right, "Addresses should not be equal");
    };
    ($left:expr, $right:expr, $($msg:tt)*) => {
        assert_ne!($left, $right, $($msg)*);
    };
}

/// Assert contract operation succeeded
#[macro_export]
macro_rules! assert_success {
    ($result:expr) => {
        assert!($result.is_ok(), "Contract operation failed: {:?}", $result)
    };
    ($result:expr, $($msg:tt)*) => {
        assert!($result.is_ok(), $($msg)*)
    };
}

/// Assert contract operation failed with specific error
#[macro_export]
macro_rules! assert_contract_error {
    ($result:expr, $expected_error:expr) => {
        match $result {
            Err(code) => assert_eq!(code, $expected_error, "Contract error code mismatch"),
            Ok(_) => panic!("Expected contract error but succeeded"),
        }
    };
    ($result:expr, $expected_error:expr, $($msg:tt)*) => {
        match $result {
            Err(code) => assert_eq!(code, $expected_error, $($msg)*),
            Ok(_) => panic!("Expected contract error but succeeded"),
        }
    };
}

/// Assert operation timed out
#[macro_export]
macro_rules! assert_timeout {
    ($duration_ms:expr, $max_ms:expr) => {
        assert!(
            $duration_ms < $max_ms,
            "Operation took {}ms, expected < {}ms",
            $duration_ms,
            $max_ms
        )
    };
}

/// Assert value is within range (inclusive)
#[macro_export]
macro_rules! assert_in_range {
    ($value:expr, $min:expr, $max:expr) => {
        assert!(
            $value >= $min && $value <= $max,
            "Value {} not in range [{}, {}]",
            $value,
            $min,
            $max
        )
    };
}

/// Assert collection contains exactly N elements
#[macro_export]
macro_rules! assert_collection_len {
    ($collection:expr, $expected_len:expr) => {
        assert_eq!(
            $collection.len(),
            $expected_len,
            "Collection length mismatch"
        )
    };
}

/// Assert string matches pattern
#[macro_export]
macro_rules! assert_matches_pattern {
    ($value:expr, $pattern:expr) => {
        assert!(
            $value.contains($pattern),
            "String '{}' does not match pattern '{}'",
            $value,
            $pattern
        )
    };
}

/// Helper functions for assertions
pub fn assert_amount_greater_than(amount: u128, threshold: u128) {
    assert!(
        amount > threshold,
        "Amount {} is not greater than {}",
        amount,
        threshold
    );
}

pub fn assert_amount_less_than(amount: u128, threshold: u128) {
    assert!(
        amount < threshold,
        "Amount {} is not less than {}",
        amount,
        threshold
    );
}

pub fn assert_amount_between(amount: u128, min: u128, max: u128) {
    assert!(
        amount >= min && amount <= max,
        "Amount {} not between {} and {}",
        amount,
        min,
        max
    );
}

pub fn assert_addresses_different(addr1: &Address, addr2: &Address) {
    assert_ne!(addr1, addr2, "Addresses should be different");
}

pub fn assert_non_empty_string(s: &str) {
    assert!(!s.is_empty(), "String should not be empty");
}

pub fn assert_string_length(s: &str, expected_len: usize) {
    assert_eq!(
        s.len(),
        expected_len,
        "String length {} does not match expected {}",
        s.len(),
        expected_len
    );
}

pub fn assert_timestamp_recent(timestamp: u64, max_age_secs: u64) {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    assert!(
        now - timestamp <= max_age_secs,
        "Timestamp is too old: {} seconds ago",
        now - timestamp
    );
}

pub fn assert_state_transition(from: &str, to: &str, allowed: &[(&str, &str)]) {
    let transition_allowed = allowed.iter().any(|(f, t)| f == &from && t == &to);
    assert!(
        transition_allowed,
        "Invalid state transition from '{}' to '{}'",
        from,
        to
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_assert_amount_greater_than() {
        assert_amount_greater_than(100, 50);
    }

    #[test]
    #[should_panic]
    fn test_assert_amount_greater_than_fails() {
        assert_amount_greater_than(50, 100);
    }

    #[test]
    fn test_assert_amount_between() {
        assert_amount_between(100, 50, 150);
    }

    #[test]
    #[should_panic]
    fn test_assert_amount_between_fails() {
        assert_amount_between(200, 50, 150);
    }

    #[test]
    fn test_assert_non_empty_string() {
        assert_non_empty_string("test");
    }

    #[test]
    #[should_panic]
    fn test_assert_non_empty_string_fails() {
        assert_non_empty_string("");
    }

    #[test]
    fn test_assert_string_length() {
        assert_string_length("test", 4);
    }

    #[test]
    #[should_panic]
    fn test_assert_string_length_fails() {
        assert_string_length("test", 5);
    }
}
