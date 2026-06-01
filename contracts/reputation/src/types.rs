use soroban_sdk::contracttype;

/// Typed storage keys for the Reputation contract (logical prefix: `REPUT_`).
#[derive(Clone)]
#[contracttype]
pub enum DataKey {
    /// Instance: governor admin `Address`.
    ReputAdmin,
    /// Persistent: `Map<Address, i128>` reputation scores.
    ReputScores,
}
