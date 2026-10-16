use soroban_sdk::{contracttype, Address, String};

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
/// Instance entries live in instance storage; everything else is persistent and
/// survives upgrades.
#[contracttype]
pub enum DataKey {
    /// Instance: administrator address, mirrored from the `DEFAULT_ADMIN_ROLE`
    /// membership so `admin()` stays a cheap single read.
    Admin,
    /// Instance: treasury receiving protocol fees.
    Treasury,
    /// Instance: global pause switch.
    Paused,
    /// Instance: number of registered assets.
    AssetCount,
    /// Instance: addresses holding a role, per role id.
    RoleMembers(u32),
    /// Persistent: asset record keyed by `asset_id`.
    Asset(String),
    /// Persistent: holder balance keyed by `(asset_id, holder)`.
    Balance(String, Address),
    /// Persistent: whether `(account, role)` is held.
    Role(Address, u32),
}