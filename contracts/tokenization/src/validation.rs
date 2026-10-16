//! Boundary checks applied to every externally supplied argument.
//!
//! Validation runs *before* any `require_auth`, so a malformed call cannot
//! even be used to probe which addresses hold authority: the caller spends
//! nothing and learns nothing from the rejection.

use crate::errors::Error;
use soroban_sdk::{Address, String};

/// Minimum length of an `asset_id`.
pub const MIN_ASSET_ID_LEN: u32 = 3;
/// Maximum length of an `asset_id`.
pub const MAX_ASSET_ID_LEN: u32 = 64;
/// Maximum display decimals, mirroring the Stellar Asset Convention cap.
pub const MAX_DECIMALS: u32 = 18;
/// Minimum display decimals.
pub const MIN_DECIMALS: u32 = 0;
/// Maximum length of a `metadata_uri`.
pub const MAX_METADATA_URI_LEN: u32 = 512;

/// Reject an empty, over-long, or non-identifier `asset_id`.
///
/// The accepted alphabet is ASCII alphanumerics plus `-` and `_`, which covers
/// the `HOSP-2026-001` style identifiers used across the platform without
/// allowing control characters or bidi overrides into a value that ends up
/// rendered in dashboards and explorers.
pub fn validate_asset_id(asset_id: &String) -> Result<(), Error> {
    let len = asset_id.len();
    if len < MIN_ASSET_ID_LEN || len > MAX_ASSET_ID_LEN {
        return Err(Error::InvalidAssetId);
    }

    let bytes = asset_id.to_buffer();
    for i in 0..bytes.len() {
        let b = bytes.get(i).unwrap_or(0);
        let ok = (b >= b'a' && b <= b'z')
            || (b >= b'A' && b <= b'Z')
            || (b >= b'0' && b <= b'9')
            || b == b'-'
            || b == b'_';
        if !ok {
            return Err(Error::InvalidAssetId);
        }
    }

    Ok(())
}

/// Reject a `decimals` value outside the Stellar Asset Convention range.
pub fn validate_decimals(decimals: u32) -> Result<(), Error> {
    if decimals < MIN_DECIMALS || decimals > MAX_DECIMALS {
        return Err(Error::DecimalsOutOfRange);
    }
    Ok(())
}

/// Reject an empty or over-long `metadata_uri`.
///
/// Length is bounded because the URI is stored verbatim on the ledger and paid
/// for by every holder who has to extend the entry's TTL.
pub fn validate_metadata_uri(uri: &String) -> Result<(), Error> {
    let len = uri.len();
    if len == 0 || len > MAX_METADATA_URI_LEN {
        return Err(Error::InvalidMetadataUri);
    }
    Ok(())
}

/// Reject a `mint` / `burn` / `transfer` amount of zero.
///
/// A zero-amount write costs a ledger write and burns rent to move nothing.
pub fn validate_amount(amount: u128) -> Result<(), Error> {
    if amount == 0 {
        return Err(Error::ZeroAmount);
    }
    Ok(())
}

/// Reject a self-transfer, which would debit and credit the same entry.
pub fn validate_transfer_parties(from: &Address, to: &Address) -> Result<(), Error> {
    if from == to {
        return Err(Error::TransferToSelf);
    }
    Ok(())
}

/// Assert `holder` can cover a debit of `amount` without wrapping.
pub fn validate_debit(balance: u128, amount: u128) -> Result<(), Error> {
    if balance < amount {
        return Err(Error::InsufficientBalance);
    }
    Ok(())
}

/// Assert a mint of `amount` stays inside the registered supply cap.
pub fn validate_supply_cap(current_supply: u128, amount: u128, cap: u128) -> Result<(), Error> {
    match current_supply.checked_add(amount) {
        Some(next) if next <= cap => Ok(()),
        _ => Err(Error::SupplyCapExceeded),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use soroban_sdk::Env;

    #[test]
    fn asset_id_accepts_the_documented_shape() {
        let env = Env::default();
        assert!(validate_asset_id(&String::from_str(&env, "HOSP-2026-001")).is_ok());
        assert!(validate_asset_id(&String::from_str(&env, "abc")).is_ok());
    }

    #[test]
    fn asset_id_rejects_empty_short_and_long_values() {
        let env = Env::default();
        assert_eq!(
            validate_asset_id(&String::from_str(&env, "")),
            Err(Error::InvalidAssetId)
        );
        assert_eq!(
            validate_asset_id(&String::from_str(&env, "ab")),
            Err(Error::InvalidAssetId)
        );
        assert_eq!(
            validate_asset_id(&String::from_str(&env, "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa")),
            Err(Error::InvalidAssetId)
        );
    }

    #[test]
    fn asset_id_rejects_characters_outside_the_alphabet() {
        let env = Env::default();
        assert_eq!(
            validate_asset_id(&String::from_str(&env, "HOSP 001")),
            Err(Error::InvalidAssetId)
        );
        assert_eq!(
            validate_asset_id(&String::from_str(&env, "HOSP/001")),
            Err(Error::InvalidAssetId)
        );
    }

    #[test]
    fn decimals_respect_the_sac_range() {
        assert!(validate_decimals(0).is_ok());
        assert!(validate_decimals(7).is_ok());
        assert!(validate_decimals(18).is_ok());
        assert_eq!(validate_decimals(19), Err(Error::DecimalsOutOfRange));
    }

    #[test]
    fn metadata_uri_must_be_present_and_bounded() {
        let env = Env::default();
        assert!(validate_metadata_uri(&String::from_str(&env, "ipfs://abc")).is_ok());
        assert_eq!(
            validate_metadata_uri(&String::from_str(&env, "")),
            Err(Error::InvalidMetadataUri)
        );
    }

    #[test]
    fn amount_must_be_non_zero() {
        assert!(validate_amount(1).is_ok());
        assert_eq!(validate_amount(0), Err(Error::ZeroAmount));
    }

    #[test]
    fn debit_precheck_distinguishes_short_balances() {
        assert!(validate_debit(100, 100).is_ok());
        assert_eq!(validate_debit(99, 100), Err(Error::InsufficientBalance));
    }

    #[test]
    fn supply_cap_check_is_overflow_safe() {
        assert!(validate_supply_cap(10, 90, 100).is_ok());
        assert_eq!(validate_supply_cap(10, 91, 100), Err(Error::SupplyCapExceeded));
        assert_eq!(
            validate_supply_cap(u128::MAX, 1, u128::MAX),
            Err(Error::SupplyCapExceeded)
        );
    }
}