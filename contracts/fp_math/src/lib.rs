#![no_std]

/// Multiply `amount` by basis points (1 bps = 0.01%) using floor division.
///
/// Floor rounding ensures fees are never rounded up — the fee taker always
/// receives ≤ the exact fractional amount. Callers can reconstruct the
/// complementary side as `amount - fee` to guarantee `fee + remainder == amount`.
///
/// Returns `None` if the intermediate `amount * bps` overflows `i128`.
pub fn mul_bps(amount: i128, bps: u32) -> Option<i128> {
    amount.checked_mul(bps as i128).map(|n| n / 10_000)
}

/// Multiply `amount` by basis points with round-half-up rounding.
///
/// Returns `None` on overflow.
pub fn mul_bps_round_half_up(amount: i128, bps: u32) -> Option<i128> {
    amount
        .checked_mul(bps as i128)
        .and_then(|n| n.checked_add(5_000))
        .map(|n| n / 10_000)
}

/// Calculate tokens to allocate for a payment:
/// `tokens = payment * 10^token_decimals / price_per_token`
///
/// Returns `None` on overflow or if `price_per_token` is zero.
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
}
