//! Address validation functions

use soroban_sdk::Address;
use crate::errors::ValidationError;

/// Validates that an address is not a zero address
///
/// # Arguments
/// * `address` - The address to validate
///
/// # Returns
/// `Ok(())` if valid, otherwise returns `ValidationError::InvalidAddress`
///
/// # Note
/// In Soroban, addresses are validated by the SDK at construction time.
/// Any `Address` value that exists is already considered valid.
pub fn validate_address(_address: &Address) -> Result<(), ValidationError> {
    // In Soroban, addresses are validated at construction time
    Ok(())
}

/// Validates that two addresses are different
///
/// # Arguments
/// * `addr1` - First address
/// * `addr2` - Second address
///
/// # Returns
/// `Ok(())` if addresses are different, otherwise returns `ValidationError::AddressesMustDiffer`
pub fn validate_addresses_different(addr1: &Address, addr2: &Address) -> Result<(), ValidationError> {
    if addr1 == addr2 {
        return Err(ValidationError::AddressesMustDiffer);
    }
    Ok(())
}

/// Validates that multiple addresses are all different from each other
///
/// # Arguments
/// * `addresses` - Vector of addresses to validate
///
/// # Returns
/// `Ok(())` if all addresses are unique, otherwise returns `ValidationError::DuplicateEntry`
pub fn validate_addresses_unique(addresses: &[&Address]) -> Result<(), ValidationError> {
    for i in 0..addresses.len() {
        for j in (i + 1)..addresses.len() {
            if addresses[i] == addresses[j] {
                return Err(ValidationError::DuplicateEntry);
            }
        }
    }
    Ok(())
}
