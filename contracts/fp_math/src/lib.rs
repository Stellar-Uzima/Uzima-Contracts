#![no_std]
#![forbid(alloc)]
//! fp_math - fixed-point helpers for Stellar smart contracts.
//!
//! Every function here is total: it returns [`Option`] rather than panicking or
//! wrapping, and `None` always means "the exact result is not representable",
//! never "the result was clamped". Callers are expected to treat `None` as a
//! hard error, so the precision guarantees below are part of this crate's
//! contract, not implementation detail.
//!
//! All three functions truncate rather than round (see [`mul_bps`]), which
//! keeps value conserved when a caller splits an amount with
//! `part + (amount - part) == amount`. The rounding direction, and the two
//! places where signed input behaves asymmetrically, are documented on each
//! function and pinned by the tests at the bottom of this file.

/// Multiply `amount` by basis points (1 bps = 0.01%) with truncating division.
///
/// # Precision invariants
///
/// * The quotient is truncated **toward zero**, not toward negative infinity.
///   For a non-negative `amount` that is the same as flooring; for a negative
///   `amount` it rounds the magnitude *down*, so `mul_bps(-100, 3333) ==
///   Some(-33)`, not `Some(-34)`. The earlier "floor division" wording in this
///   doc comment was wrong for signed input.
/// * `bps` is not range-checked. `bps > 10_000` is accepted and returns a
///   result larger than `amount` (`mul_bps(100, 20_000) == Some(200)`). Keep
///   `bps <= 10_000`; the callers in this repository do.
/// * For `bps <= 10_000` the magnitude never inflates: `|result| <= |amount|`.
/// * Endpoints are exact: `mul_bps(x, 0) == Some(0)` and, for non-negative
///   `amount`, `mul_bps(x, 10_000) == Some(x)`.
/// * Monotonic in `bps`: non-decreasing for a non-negative `amount`,
///   non-increasing for a negative one, matching proportionality.
/// * A negative `amount` yields a negative result. Nothing here rejects signed
///   input, and the current caller `payment_router::compute_split` takes a bare
///   `i128` with no sign check, so validate the sign before splitting funds.
///
/// # Conservation
///
/// Because the result is a truncation, `mul_bps(amount, bps) + (amount -
/// mul_bps(amount, bps)) == amount` holds for every input, so a fee and its
/// remainder always re-add to the original amount. The fee taker receives at
/// most the exact fractional amount.
///
/// Returns `None` if the intermediate `amount * bps` overflows `i128`.
pub fn mul_bps(amount: i128, bps: u32) -> Option<i128> {
    amount.checked_mul(i128::from(bps)).map(|n| n / 10_000)
}

/// Multiply `amount` by basis points, adding 5_000 before dividing so that
/// exact halves round up.
///
/// # Precision invariants
///
/// * "Half up" holds only for a non-negative `amount`. The `+ 5_000` bias is
///   applied to the numerator without regard to sign, so a negative `amount`
///   is rounded **toward zero** instead: `mul_bps_round_half_up(-100, 3333) ==
///   Some(-32)` where [`mul_bps`] gives `Some(-33)`.
/// * Consequently the 100% identity degrades for negative input:
///   `mul_bps_round_half_up(x, 10_000) == Some(x)` for `x >= 0`, but
///   `Some(x + 1)` for `x < 0` (`-100` becomes `-99`). Signed callers that
///   need the exact 100% value must use [`mul_bps`].
/// * Where both functions return `Some`, the two results differ by at most 1.
/// * The `+ 5_000` nudge is itself overflow-checked, which makes the accepted
///   range fractionally narrower than [`mul_bps`] at the very top of `i128`:
///   `mul_bps_round_half_up(i128::MAX, 1) == None` even though `i128::MAX * 1`
///   does not overflow, because the addition does. The `i128::MIN` end is
///   unaffected, since adding a positive value to `i128::MIN` cannot overflow.
///   The result is a conservative `None`, never a wrapped value.
///
/// Returns `None` if `amount * bps` or the subsequent `+ 5_000` overflows.
pub fn mul_bps_round_half_up(amount: i128, bps: u32) -> Option<i128> {
    amount
        .checked_mul(i128::from(bps))
        .and_then(|n| n.checked_add(5_000))
        .map(|n| n / 10_000)
}

/// Calculate tokens to allocate for a payment:
/// `tokens = payment * 10^token_decimals / price_per_token`
///
/// # Precision invariants
///
/// * Division truncates, so a dust payment can legitimately allocate zero
///   tokens: `tokens_for_payment(1, 3, 0) == Some(0)`. This function will not
///   signal an under-payment, so a caller that must reject one has to compare
///   the returned amount against what it expected.
/// * At par — `price_per_token == 10^token_decimals` — the result is exactly
///   `payment`, with no rounding.
/// * `price_per_token` is the raw on-chain integer price, already scaled by
///   `10^token_decimals`; it is not a human-readable decimal.
/// * `token_decimals` must be `<= 38`, because `10^39` exceeds `u128::MAX`.
///
/// Returns `None` on overflow (including an unrepresentable `10^token_decimals`)
/// or if `price_per_token` is zero.
pub fn tokens_for_payment(
    payment: u128,
    price_per_token: u128,
    token_decimals: u32,
) -> Option<u128> {
    if price_per_token == 0 {
        return None;
    }
    let scale = 10u128.checked_pow(token_decimals)?;
    payment.checked_mul(scale)?.checked_div(price_per_token)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mul_bps_basic() {
        assert_eq!(mul_bps(1000, 1000), Some(100)); // 10% of 1000
        assert_eq!(mul_bps(1000, 250), Some(25)); // 2.5% of 1000
        assert_eq!(mul_bps(1000, 10_000), Some(1000)); // 100%
        assert_eq!(mul_bps(0, 500), Some(0));
        assert_eq!(mul_bps(1000, 0), Some(0));
    }

    #[test]
    fn test_mul_bps_floor_and_conservation() {
        // floor: 33.33% of 100 = 33, not 34
        let fee = mul_bps(100, 3333).unwrap();
        assert_eq!(fee, 33);
        // conservation: fee + remainder == amount
        assert_eq!(fee + (100 - fee), 100);
    }

    #[test]
    fn test_mul_bps_overflow() {
        // i128::MAX * 2 overflows i128 in checked_mul
        assert_eq!(mul_bps(i128::MAX, 2), None);
        // i128::MAX * 1 = i128::MAX — no overflow, result is i128::MAX / 10_000
        assert!(mul_bps(i128::MAX, 1).is_some());
    }

    #[test]
    fn test_mul_bps_round_half_up() {
        // 3 * 3333 = 9999 → floor 0, half-up 1
        assert_eq!(mul_bps(3, 3333), Some(0));
        assert_eq!(mul_bps_round_half_up(3, 3333), Some(1));
        // exact multiple: no rounding difference
        assert_eq!(mul_bps(1000, 250), mul_bps_round_half_up(1000, 250));
    }

    #[test]
    fn test_tokens_for_payment_6_decimals() {
        // 500 payment at price=100, 6 decimals → 500 * 1_000_000 / 100 = 5_000_000
        assert_eq!(tokens_for_payment(500, 100, 6), Some(5_000_000));
    }

    #[test]
    fn test_tokens_for_payment_7_decimals() {
        // Stellar standard: 7 decimals
        assert_eq!(tokens_for_payment(1_000_000, 500_000, 7), Some(20_000_000));
    }

    #[test]
    fn test_tokens_for_payment_division_by_zero() {
        assert_eq!(tokens_for_payment(1000, 0, 6), None);
    }

    #[test]
    fn test_tokens_for_payment_overflow() {
        // 10^39 > u128::MAX so decimals=39 overflows the scale
        assert_eq!(tokens_for_payment(1, 1, 39), None);
        // payment * scale overflow
        assert_eq!(tokens_for_payment(u128::MAX, 1, 1), None);
    }

    #[test]
    fn test_tokens_for_payment_zero_payment() {
        assert_eq!(tokens_for_payment(0, 100, 6), Some(0));
    }

    // ─────────────────────────────────────────────────────────────────────
    // Invariants pinned for #1579. The examples above cover the happy path;
    // the tests below pin the rounding direction, the overflow boundaries and
    // the signed-input behaviour that the doc comments promise.
    // ─────────────────────────────────────────────────────────────────────

    /// `mul_bps` truncates toward zero, so a negative amount has its magnitude
    /// rounded *down*. This is the case the old "floor division" wording got
    /// wrong: a true floor of -33.3 is -34.
    #[test]
    fn test_mul_bps_negative_truncates_toward_zero() {
        assert_eq!(mul_bps(-100, 3333), Some(-33));
        assert_eq!(mul_bps(-100, 2500), Some(-25));
        assert_eq!(mul_bps(-100, 10_000), Some(-100));
        assert_eq!(mul_bps(-1, 1), Some(0));
    }

    /// The `+ 5_000` bias is sign-blind, so for a negative amount it rounds
    /// toward zero rather than away from it — the opposite of "half up".
    #[test]
    fn test_mul_bps_round_half_up_negative_biases_toward_zero() {
        assert_eq!(mul_bps_round_half_up(-100, 3333), Some(-32));
        assert_eq!(mul_bps_round_half_up(-100, 2500), Some(-24));
        assert_eq!(mul_bps_round_half_up(-1, 1), Some(0));
    }

    /// The 100% identity holds for non-negative input and is off by exactly
    /// +1 (toward zero) for negative input. Documented on the function; pinned
    /// here so a future change to the nudge cannot alter it silently.
    #[test]
    fn test_round_half_up_full_rate_identity() {
        assert_eq!(mul_bps_round_half_up(100, 10_000), Some(100));
        assert_eq!(mul_bps_round_half_up(1, 10_000), Some(1));
        assert_eq!(mul_bps_round_half_up(0, 10_000), Some(0));
        assert_eq!(mul_bps_round_half_up(-100, 10_000), Some(-99));
        assert_eq!(mul_bps_round_half_up(-1, 10_000), Some(0));
    }

    /// Conservation is the property callers rely on when splitting a payment,
    /// and it has to hold for negative input too.
    #[test]
    fn test_mul_bps_conservation_holds_for_signed_amounts() {
        for amount in -500..=500 {
            for bps in (0..=10_000).step_by(37) {
                if let Some(fee) = mul_bps(amount, bps) {
                    assert_eq!(fee + (amount - fee), amount);
                }
            }
        }
    }

    /// For `bps <= 10_000` the result never inflates the magnitude, and the
    /// zero/100% endpoints are exact.
    #[test]
    fn test_mul_bps_endpoints_and_magnitude() {
        for amount in -500..=500 {
            assert_eq!(mul_bps(amount, 0), Some(0));
            assert_eq!(mul_bps(amount, 10_000), Some(amount));
            for bps in (0..=10_000).step_by(53) {
                if let Some(fee) = mul_bps(amount, bps) {
                    assert!(fee.abs() <= amount.abs());
                }
            }
        }
    }

    /// Proportionality: growing `bps` never shrinks the result for a
    /// non-negative amount, and never grows it for a negative one.
    #[test]
    fn test_mul_bps_monotonic_in_bps() {
        for amount in [-1_000i128, -7, 0, 7, 1_000] {
            for bps in 0..10_000u32 {
                let current = mul_bps(amount, bps);
                let next = mul_bps(amount, bps + 1);
                if let (Some(current), Some(next)) = (current, next) {
                    if amount >= 0 {
                        assert!(next >= current);
                    } else {
                        assert!(next <= current);
                    }
                }
            }
        }
    }

    /// Where both rounding modes succeed they differ by at most 1, so a caller
    /// can treat the choice of mode as sub-unit noise.
    #[test]
    fn test_rounding_modes_differ_by_at_most_one() {
        for amount in -500..=500 {
            for bps in (0..=10_000).step_by(29) {
                if let (Some(trunc), Some(rounded)) =
                    (mul_bps(amount, bps), mul_bps_round_half_up(amount, bps))
                {
                    assert!(rounded.abs_diff(trunc) <= 1);
                }
            }
        }
    }

    /// Overflow boundaries for `mul_bps`: `i128::MIN * 1` is representable and
    /// `i128::MAX * 1` is too, but doubling either is not.
    #[test]
    fn test_mul_bps_overflow_boundaries() {
        assert_eq!(mul_bps(i128::MAX, 0), Some(0));
        assert_eq!(mul_bps(i128::MAX, 1), Some(17_014_118_346_046_923_173_168_730_371_588_410));
        assert_eq!(mul_bps(i128::MAX, 2), None);
        assert_eq!(mul_bps(i128::MIN, 1), Some(-17_014_118_346_046_923_173_168_730_371_588_410));
        assert_eq!(mul_bps(i128::MIN, 2), None);
        assert_eq!(mul_bps(i128::MAX, 10_000), None);
        // A large `bps` alone is not enough to overflow: 1 * u32::MAX fits in
        // i128 comfortably. It needs a large amount as well.
        assert_eq!(mul_bps(1, u32::MAX), Some(429_496));
        assert_eq!(mul_bps(i128::MAX, u32::MAX), None);
    }

    /// The `+ 5_000` nudge is checked, so the top of the range is rejected
    /// conservatively while the bottom is not. `None` here is a refusal, not a
    /// wrapped value.
    #[test]
    fn test_round_half_up_top_of_range_is_conservative() {
        assert_eq!(mul_bps_round_half_up(i128::MAX, 1), None);
        assert_eq!(mul_bps_round_half_up(i128::MAX, 0), Some(0));
        assert_eq!(
            mul_bps_round_half_up(i128::MIN, 1),
            Some(-17_014_118_346_046_923_173_168_730_371_588_410)
        );
    }

    /// `price_per_token` is never range-checked, so an out-of-contract `bps`
    /// inflates the result. Pinned to keep the documented caveat honest.
    #[test]
    fn test_mul_bps_accepts_rates_above_full() {
        assert_eq!(mul_bps(100, 20_000), Some(200));
        assert_eq!(mul_bps(100, 10_001), Some(100));
    }

    /// Truncating division means dust payments allocate zero tokens rather than
    /// signalling an error.
    #[test]
    fn test_tokens_for_payment_dust_truncates_to_zero() {
        assert_eq!(tokens_for_payment(1, 3, 0), Some(0));
        assert_eq!(tokens_for_payment(2, 3, 0), Some(0));
        assert_eq!(tokens_for_payment(10, 3, 0), Some(3));
    }

    /// `10^token_decimals` must fit in `u128`, which caps decimals at 38.
    #[test]
    fn test_tokens_for_payment_decimals_upper_bound() {
        assert_eq!(
            tokens_for_payment(1, 1, 38),
            Some(100_000_000_000_000_000_000_000_000_000_000_000_000)
        );
        assert_eq!(tokens_for_payment(1, 1, 39), None);
    }

    /// At par the conversion is the identity — no rounding, no drift.
    #[test]
    fn test_tokens_for_payment_at_par_is_identity() {
        for decimals in 0..=7u32 {
            let price = 10u128.pow(decimals);
            for payment in [0u128, 1, 7, 500, 1_000_000] {
                assert_eq!(tokens_for_payment(payment, price, decimals), Some(payment));
            }
        }
    }

    /// Allocating more payment never yields fewer tokens.
    #[test]
    fn test_tokens_for_payment_monotonic_in_payment() {
        let mut previous = 0u128;
        for payment in 0..2_000u128 {
            if let Some(tokens) = tokens_for_payment(payment, 7, 6) {
                assert!(tokens >= previous);
                previous = tokens;
            }
        }
    }
}
