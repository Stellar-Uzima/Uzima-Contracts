use soroban_sdk::{contracttype, Address, String};

/// Ownership shares are denominated in basis points; 10_000 == 100%.
pub const BASIS_POINTS_SCALE: u128 = 10_000;

/// A registered property record.
///
/// `valuation` is expressed in the smallest unit of `valuation_currency` so
/// that fractional amounts are representable without floating point.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Property {
    /// Stable registry identifier chosen by the registrar.
    pub property_id: String,
    /// Current legal owner of record.
    pub owner: Address,
    /// Ledger timestamp of registration.
    pub created_at: u64,
    /// Ledger timestamp of the most recent mutation.
    pub updated_at: u64,
    /// Latest recorded valuation in minor units.
    pub valuation: u128,
    /// ISO code of the currency `valuation` is denominated in.
    pub valuation_currency: String,
    /// Off-chain metadata document (title deed, survey, licence).
    pub metadata_uri: String,
    /// Number of holders with a recorded share.
    pub holder_count: u32,
}

/// Storage keys for the contract.
#[contracttype]
pub enum DataKey {
    /// Instance: administrator address, mirrored from the `DEFAULT_ADMIN_ROLE`
    /// membership so `admin()` stays a cheap single read.
    Admin,
    /// Instance: registrar allowed to create and revalue properties.
    Registrar,
    /// Instance: global pause switch.
    Paused,
    /// Instance: number of registered properties.
    PropertyCount,
    /// Instance: addresses holding a role, per role id.
    RoleMembers(u32),
    /// Persistent: property record keyed by `property_id`.
    Property(String),
    /// Persistent: ownership share keyed by `(property_id, holder)`.
    Ownership(String, Address),
    /// Persistent: whether `(account, role)` is held.
    Role(Address, u32),
}