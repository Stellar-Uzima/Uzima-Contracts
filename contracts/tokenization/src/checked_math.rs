use crate::errors::Error;

/// Add `a + b`, mapping overflow onto [`Error::ArithmeticOverflow`].
///
/// Release builds compile with `overflow-checks = true` (root `Cargo.toml`
/// `[profile.release]`), so an unguarded `+` on `u128` panics and burns the
/// whole transaction. Every arithmetic site in this contract goes through
/// these helpers so the failure mode is a typed contract error instead.
pub fn checked_add(a: u128, b: u128) -> Result<u128, Error> {
    a.checked_add(b).ok_or(Error::ArithmeticOverflow)
}

/// Subtract `a - b`, mapping underflow onto [`Error::ArithmeticUnderflow`].
///
/// Callers that need to distinguish "the balance was too small" from "the
/// arithmetic itself was pathological" should pre-check the holder balance and
/// return [`Error::InsufficientBalance`]; this helper is the last line of
/// defence.
pub fn checked_sub(a: u128, b: u128) -> Result<u128, Error> {
    a.checked_sub(b).ok_or(Error::ArithmeticUnderflow)
}

/// Multiply `a * b`, mapping overflow onto [`Error::ArithmeticOverflow`].
pub fn checked_mul(a: u128, b: u128) -> Result<u128, Error> {
    a.checked_mul(b).ok_or(Error::ArithmeticOverflow)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn checked_add_passes_through_in_range_values() {
        assert_eq!(checked_add(100, 23), Ok(123));
        assert_eq!(checked_add(u128::MAX, 0), Ok(u128::MAX));
    }

    #[test]
    fn checked_add_rejects_overflow() {
        assert_eq!(checked_add(u128::MAX, 1), Err(Error::ArithmeticOverflow));
    }

    #[test]
    fn checked_sub_passes_through_in_range_values() {
        assert_eq!(checked_sub(100, 23), Ok(77));
        assert_eq!(checked_sub(0, 0), Ok(0));
    }

    #[test]
    fn checked_sub_rejects_underflow() {
        assert_eq!(checked_sub(0, 1), Err(Error::ArithmeticUnderflow));
    }

    #[test]
    fn checked_mul_passes_through_in_range_values() {
        assert_eq!(checked_mul(12, 12), Ok(144));
    }

    #[test]
    fn checked_mul_rejects_overflow() {
        assert_eq!(checked_mul(u128::MAX, 2), Err(Error::ArithmeticOverflow));
    }
}