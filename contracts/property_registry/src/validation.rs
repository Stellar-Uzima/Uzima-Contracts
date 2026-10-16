//! Boundary checks applied to every externally supplied argument.
//!
//! Validation runs *before* any `require_auth`, so a malformed call cannot even
//! be used to probe which addresses hold authority: the caller spends nothing
//! and learns nothing from the rejection.

use crate::errors::Error;
use soroban_sdk::{Address, String};

/// Minimum length of a `property_id`.
pub const MIN_PROPERTY_ID_LEN: u32 = 3;
/// Maximum length of a `property_id`.
pub const MAX_PROPERTY_ID_LEN: u32 = 64;
/// Expected length of an ISO 4217 alphabetic currency code, e.g. `"USD"`.
pub const CURRENCY_LEN: u32 = 3;
/// Maximum length of a `valuation_currency`, allowing for longer local codes.
pub const MAX_CURRENCY_LEN: u32 = 8;
/// Maximum length of a `metadata_uri`.
pub const MAX_METADATA_URI_LEN: u32 = 512;
/// Ownership shares are denominated in basis points; 10_000 == 100%.
pub const BASIS_POINTS_SCALE: u128 = 10_000;

/// Reject an empty, over-long, or non-identifier `property_id`.
///
/// The accepted alphabet is ASCII alphanumerics plus `-` and `_`, which covers
/// the `PROP-2026-001` style identifiers used across the platform without
/// allowing control characters or bidi overrides into a value that ends up
/// rendered in dashboards and land registries.
pub fn validate_property_id(property_id: &String) -> Result<(), Error> {
    let len = property_id.len();
    if len < MIN_PROPERTY_ID_LEN || len > MAX_PROPERTY_ID_LEN {
        return Err(Error::InvalidPropertyId);
    }

    let bytes = property_id.to_buffer();
    for i in 0..bytes.len() {
        let b = bytes.get(i).unwrap_or(0);
        let ok = (b >= b'a' && b <= b'z')
            || (b >= b'A' && b <= b'Z')
            || (b >= b'0' && b <= b'9')
            || b == b'-'
            || b == b'_';
        if !ok {
            return Err(Error::InvalidPropertyId);
        }
    }

    Ok(())
}


/// Reject a `valuation_currency` that is empty, over-long, or not alphabetic.
///
/// The value is stored verbatim and paid for on every TTL extension, so it is
/// bounded; it is also required to be alphabetic so a downstream indexer can
/// treat it as an ISO 4217 code without a second parse.
pub fn validate_currency(currency: &String) -> Result<(), Error> {
    let len = currency.len();
    if len == 0 || len > MAX_CURRENCY_LEN {
        return Err(Error::InvalidCurrency);
    }

    let bytes = currency.to_buffer();
    for i in 0..bytes.len() {
        let b = bytes.get(i).unwrap_or(0);
        if !(b >= b'A' && b <= b'Z') && !(b >= b'a' && b <= b'z') {
            return Err(Error::InvalidCurrency);
        }
    }

    Ok(())
}

/// Whether `currency` is exactly an ISO 4217-shaped three-letter code.
pub fn is_iso_4217(currency: &String) -> bool {
    currency.len() == CURRENCY_LEN && validate_currency(currency).is_ok()
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

/// Reject a `valuation` of zero.
pub fn validate_valuation(valuation: u128) -> Result<(), Error> {
    if valuation == 0 {
        return Err(Error::ZeroValue);
    }
    Ok(())
}

/// Reject a share of zero, or one that cannot be expressed in basis points.
pub fn validate_share(share: u128) -> Result<(), Error> {
    if share == 0 {
        return Err(Error::ZeroValue);
    }
    if share > BASIS_POINTS_SCALE {
        return Err(Error::ShareTooLarge);
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

/// Reject crediting the owner of record, which would be a no-op written twice.
pub fn validate_holder(holder: &Address, owner: &Address) -> Result<(), Error> {
    if holder == owner {
        return Err(Error::HolderIsOwner);
    }
    Ok(())
}

/// Assert `available` can cover a debit of `amount` without wrapping.
pub fn validate_debit(available: u128, amount: u128) -> Result<(), Error> {
    if available < amount {
        return Err(Error::OwnershipExceedsShare);
    }
    Ok(())
}

/// Assert a credit of `amount` would leave the split at or below 100%.
pub fn validate_share_total(credited: u128) -> Result<(), Error> {
    if credited > BASIS_POINTS_SCALE {
        return Err(Error::ShareTotalExceeded);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use soroban_sdk::Env;

    #[test]
    fn property_id_accepts_the_documented_shape() {
        let env = Env::default();
        assert!(validate_property_id(&String::from_str(&env, "PROP-2026-001")).is_ok());
        assert!(validate_property_id(&String::from_str(&env, "abc")).is_ok());
    }

    #[test]
    fn property_id_rejects_empty_short_and_long_values() {
        let env = Env::default();
        assert_eq!(
            validate_property_id(&String::from_str(&env, "")),
            Err(Error::InvalidPropertyId)
        );
        assert_eq!(
            validate_property_id(&String::from_str(&env, "ab")),
            Err(Error::InvalidPropertyId)
        );
        assert_eq!(
            validate_property_id(&String::from_str(
                &env,
                "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
            )),
            Err(Error::InvalidPropertyId)
        );
    }

    #[test]
    fn property_id_rejects_characters_outside_the_alphabet() {
        let env = Env::default();
        assert_eq!(
            validate_property_id(&String::from_str(&env, "PROP 001")),
            Err(Error::InvalidPropertyId)
        );
        assert_eq!(
            validate_property_id(&String::from_str(&env, "PROP/001")),
            Err(Error::InvalidPropertyId)
        );
    }

    #[test]
    fn currency_must_be_alphabetic_and_bounded() {
        let env = Env::default();
        assert!(validate_currency(&String::from_str(&env, "USD")).is_ok());
        assert!(validate_currency(&String::from_str(&env, "NGN")).is_ok());
        assert_eq!(
            validate_currency(&String::from_str(&env, "")),
            Err(Error::InvalidCurrency)
        );
        assert_eq!(
            validate_currency(&String::from_str(&env, "US1")),
            Err(Error::InvalidCurrency)
        );
    }

    #[test]
    fn iso_4217_shape_is_recognisable() {
        let env = Env::default();
        assert!(is_iso_4217(&String::from_str(&env, "USD")));
        assert!(!is_iso_4217(&String::from_str(&env, "DOLLARS")));
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
    fn valuation_must_be_positive() {
        assert!(validate_valuation(1).is_ok());
        assert_eq!(validate_valuation(0), Err(Error::ZeroValue));
    }

    #[test]
    fn share_bounds_are_checked_before_anything_else() {
        assert_eq!(validate_share(0), Err(Error::ZeroValue));
        assert_eq!(
            validate_share(BASIS_POINTS_SCALE + 1),
            Err(Error::ShareTooLarge)
        );
        assert!(validate_share(BASIS_POINTS_SCALE).is_ok());
    }

    #[test]
    fn debit_and_total_prechecks_distinguish_failure_reasons() {
        assert!(validate_debit(100, 100).is_ok());
        assert_eq!(validate_debit(99, 100), Err(Error::OwnershipExceedsShare));

        assert!(validate_share_total(BASIS_POINTS_SCALE).is_ok());
        assert_eq!(
            validate_share_total(BASIS_POINTS_SCALE + 1),
            Err(Error::ShareTotalExceeded)
        );
    }
}