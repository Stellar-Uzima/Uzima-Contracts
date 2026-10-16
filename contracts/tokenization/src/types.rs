use soroban_sdk::{contracttype, Address, String};

/// Version of the storage layout this contract was compiled against.
///
/// Upgrade tooling compares the on-chain value against the compiled-in value
/// to decide whether a migration step is required before the new WASM is
/// installed. Bump this whenever a `DataKey` variant or an `Asset` field is
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

/// A tokenized asset tracked by the registry.
///
/// `total_supply` is denominated in the smallest unit implied by `decimals`,
/// matching Stellar Asset Convention display precision.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Asset {
    /// Stable identifier chosen by the issuer (e.g. `"HOSP-2026-001"`).
    pub asset_id: String,
    /// Account allowed to mint and burn units of this asset.
    pub issuer: Address,
    /// Sum of all holder balances, in smallest units.
    pub total_supply: u128,
    /// Display decimals, mirroring SAC convention.
    pub decimals: u32,
    /// Off-chain metadata document describing the underlying asset.
    pub metadata_uri: String,
    /// Ledger timestamp of registration.
    pub created_at: u64,
}

/// Storage keys for the contract.
///
/// Instance entries (`Admin`, `Paused`, `AssetCount`, `StorageVersion`) live in
/// instance storage; everything else is persistent and survives upgrades.
#[contracttype]
pub enum DataKey {
    /// Instance: administrator address.
    Admin,
    /// Instance: treasury receiving protocol fees.
    Treasury,
    /// Instance: global pause switch.
    Paused,
    /// Instance: number of registered assets.
    AssetCount,
    /// Instance: layout version marker, see [`STORAGE_SCHEMA_VERSION`].
    StorageVersion,
    /// Persistent: asset record keyed by `asset_id`.
    Asset(String),
    /// Persistent: holder balance keyed by `(asset_id, holder)`.
    Balance(String, Address),
    /// Reserved for future upgrades; see [`STORAGE_GAP_SLOTS`].
    StorageGap(u32),
}