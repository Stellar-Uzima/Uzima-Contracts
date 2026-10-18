use soroban_sdk::{contracttype, Address};

/// Storage keys. `Admin`, `PaymentToken`, `AssetCount`, `Locked` and
/// `StorageVersion` live in instance storage; every other key lives in
/// persistent storage.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DataKey {
    Admin,
    PaymentToken,
    AssetCount,
    Locked,
    TotalShares(u64),
    Balance(u64, Address),
    Listing(u64, Address),
    PayoutIndex(u64),
    Checkpoint(u64, Address),
    Pending(u64, Address),
    StorageVersion,
    // Storage gap: reserved for future upgrades. Do not reuse or reorder.
    Reserved1,
    Reserved2,
    Reserved3,
}

/// An open sell offer: `amount` shares at `price_per_share` payment-token
/// base units each.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Listing {
    pub amount: i128,
    pub price_per_share: i128,
}
