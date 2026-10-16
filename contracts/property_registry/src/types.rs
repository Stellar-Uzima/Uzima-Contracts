use soroban_sdk::{contracttype, Address, String};

/// Version of the storage layout this contract was compiled against.
///
/// Upgrade tooling compares the on-chain value against the compiled-in value
/// to decide whether a migration step is required before the new WASM is
/// installed. Bump this whenever a `DataKey` variant or a `Property` field is
/// added, removed, or reordered.
pub const STORAGE_SCHEMA_VERSION: u32 = 1;

/// Number of `DataKey::StorageGap` slot indices reserved for future upgrades.
///
/// The slots are *reserved in the key layout only* — no ledger entry is written
/// for them, so the reserve costs no storage rent. A future upgrade may claim
/// `StorageGap(0)..StorageGap(STORAGE_GAP_SLOTS)` as ordinary persistent keys
/// without risking a collision with the keys live today, which is what keeps
/// reordering today's `DataKey` variants safe.
pub const STORAGE_GAP_SLOTS: u32 = 16;

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
    /// Instance: administrator address.
    Admin,
    /// Instance: registrar allowed to create and revalue properties.
    Registrar,
    /// Instance: global pause switch.
    Paused,
    /// Instance: number of registered properties.
    PropertyCount,
    /// Instance: layout version marker, see [`STORAGE_SCHEMA_VERSION`].
    StorageVersion,
    /// Persistent: property record keyed by `property_id`.
    Property(String),
    /// Persistent: ownership share keyed by `(property_id, holder)`.
    Ownership(String, Address),
    /// Reserved for future upgrades; see [`STORAGE_GAP_SLOTS`].
    StorageGap(u32),
}