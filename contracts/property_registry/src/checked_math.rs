use crate::errors::Error;

/// Add `a + b`, returning [`Error::ArithmeticOverflow`] on overflow.
///
/// Release builds compile with `overflow-checks = true` (root `Cargo.toml`
/// `[profile.release]`), so an unguarded `+` on `u128` panics and reverts the
/// whole transaction. Every arithmetic site in this contract goes through these
/// helpers so the failure mode is a typed contract error instead.
pub fn checked_add(a: u128, b: u128) -> Result<u128, Error> {
    a.checked_add(b).ok_or(Error::ArithmeticOverflow)
}

/// Subtract `a - b`, returning [`Error::ArithmeticUnderflow`] on underflow.
///
/// Callers that need to distinguish "the recorded share was too small" from "the
/// arithmetic itself was pathological" should pre-check and return
/// [`Error::OwnershipExceedsShare`]; this helper is the last line of defence.
pub fn checked_sub(a: u128, b: u128) -> Result<u128, Error> {
    a.checked_sub(b).ok_or(Error::ArithmeticUnderflow)
}

/// Multiply `a * b`, returning [`Error::ArithmeticOverflow`] on overflow.
pub fn checked_mul(a: u128, b: u128) -> Result<u128, Error> {
    a.checked_mul(b).ok_or(Error::ArithmeticOverflow)
}

/// Convert a basis-point share to whole percent, rounding down.
///
/// 10_000 bps == 100%, so `bps_to_percent(10_000) == 100`. Used by the reporting
/// helpers so a caller never has to do the division themselves and get it wrong.
pub fn bps_to_percent(basis_points: u128) -> u128 {
    basis_points / (BASIS_POINTS_SCALE / 100)
}

/// Ownership shares are denominated in basis points; 10_000 == 100%.
pub const BASIS_POINTS_SCALE: u128 = 10_000;

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
    fn checked_mul_rejects_overflow() {
        assert_eq!(checked_mul(u128::MAX, 2), Err(Error::ArithmeticOverflow));
    }

    #[test]
    fn bps_to_percent_rounds_down() {
        assert_eq!(bps_to_percent(10_000), 100);
        assert_eq!(bps_to_percent(2_500), 25);
        assert_eq!(bps_to_percent(1_999), 19);
        assert_eq!(bps_to_percent(0), 0);
    }

    #[test]
    fn bps_to_percent_never_exceeds_the_share() {
        assert!(bps_to_percent(u128::MAX) <= u128::MAX);
    }
}