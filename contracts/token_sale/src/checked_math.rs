pub fn checked_sub(a: i128, b: i128) -> Result<i128, &'static str> {
    a.checked_sub(b).ok_or("underflow")
}

pub fn checked_add(a: i128, b: i128) -> Result<i128, &'static str> {
    a.checked_add(b).ok_or("overflow")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_checked_sub_ok() {
        assert_eq!(checked_sub(100, 30), Ok(70));
    }

    #[test]
    fn test_checked_sub_underflow() {
        assert!(checked_sub(10, 100).is_err());
    }

    #[test]
    fn test_checked_sub_zero_balance() {
        assert_eq!(checked_sub(0, 0), Ok(0));
    }

    #[test]
    fn test_checked_add_overflow() {
        assert!(checked_add(i128::MAX, 1).is_err());
    }
}