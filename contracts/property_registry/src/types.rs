use soroban_sdk::{contracttype, Address, String};

/// Ownership shares are denominated in basis points; `10_000 == 100%`.
///
/// Shares are integers, so a property can be split at 1/10 000 of a percent and
/// no finer. A request above this scale is refused with
/// [`crate::Error::ShareTotalExceeded`] rather than silently clamped.
pub const BASIS_POINTS_SCALE: u128 = 10_000;

/// A registered property record.
///
/// # Invariants
///
/// - `owner` is always the account holding the controlling position; it is
///   changed only by [`crate::PropertyRegistryContract::transfer_property`].
/// - `created_at` never changes after registration; `updated_at` is refreshed on
///   every mutation.
/// - `holder_count` counts distinct holders with a non-zero share, and is
///   derived from the share transitions rather than from the caller's argument.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Property {
    /// Stable registry identifier chosen by the registrar, e.g. `"PROP-001"`.
    pub property_id: String,
    /// Current legal owner of record.
    pub owner: Address,
    /// Ledger timestamp of registration. Immutable.
    pub created_at: u64,
    /// Ledger timestamp of the most recent mutation.
    pub updated_at: u64,
    /// Latest recorded valuation in the minor units of `valuation_currency`.
    pub valuation: u128,
    /// ISO code of the currency `valuation` is denominated in.
    pub valuation_currency: String,
    /// Off-chain metadata document (title deed, survey, licence).
    pub metadata_uri: String,
    /// Number of holders with a non-zero recorded share.
    pub holder_count: u32,
}

/// Storage keys for the contract.
///
/// # Layout
///
/// Instance entries are cleared when the instance storage is replaced during a
/// WASM upgrade unless the upgrade explicitly preserves them, so anything that
/// must outlive an upgrade belongs in persistent storage. Persistent entries
/// survive and are keyed by [`DataKey`] variants, whose XDR encoding is derived
/// from declaration order — so **appending** a variant is safe and **inserting**
/// or **reordering** one is not. `DATA_KEY_VARIANT_COUNT` exists to make an
/// accidental insertion a compile error rather than a storage collision at
/// upgrade time.
///
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
    /// Instance: reentrancy guard flag, see [`crate::reentrancy_guard`].
    ReentrancyStatus,
    /// Persistent: property record keyed by `property_id`.
    Property(String),
    /// Persistent: ownership share keyed by `(property_id, holder)`.
    Ownership(String, Address),
}

/// Number of variants currently declared in [`DataKey`].
///
/// Deliberately duplicated by hand and checked against the enum by a unit test.
/// If someone inserts a variant in the middle, the test fails and points at the
/// storage-collision hazard instead of letting it reach a deployment.
pub const DATA_KEY_VARIANT_COUNT: u32 = 7;
