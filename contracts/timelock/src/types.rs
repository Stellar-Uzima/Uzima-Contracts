use soroban_sdk::contracttype;

/// Typed storage keys for the Timelock contract (logical prefix: `TIMELOCK_`).
#[derive(Clone)]
#[contracttype]
pub enum DataKey {
    /// Instance: `TimelockConfig` (admin, delay, sequence rules).
    TimelockConfig,
    /// Persistent: `Map<u64, QueuedTx>` of scheduled operations.
    TimelockQueue,
}
