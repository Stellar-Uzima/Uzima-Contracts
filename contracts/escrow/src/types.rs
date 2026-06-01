use soroban_sdk::contracttype;

/// Typed storage keys for the Escrow contract (logical prefix: `ESCROW_`).
#[derive(Clone)]
#[contracttype]
pub enum DataKey {
    /// Instance: contract admin `Address`.
    EscrowAdmin,
    /// Instance: platform fee settings (`FeeConfig`).
    EscrowFeeConfig,
    /// Persistent: `Map<u64, Escrow>` keyed by order_id.
    EscrowEscrows,
    /// Persistent: pull-payment credit balances `Map<Address, i128>`.
    EscrowCredits,
    /// Instance: aggregate `PlatformStats`.
    EscrowStats,
    /// Persistent: `Map<u64, DailyStats>` keyed by day_id.
    EscrowDailyStats,
    /// Temporary: reentrancy guard bool (CEI).
    EscrowReentrancyLock,
}
