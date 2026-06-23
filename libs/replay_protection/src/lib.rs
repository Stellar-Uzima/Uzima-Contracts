#![no_std]

#[cfg(test)]
mod test;

use soroban_sdk::{contracterror, contracttype, BytesN, Env};

/// Chain identifier used for replay-protection chain-binding checks.
/// Mirrors the common subset of `ChainId` across cross-chain contracts.
#[derive(Clone, PartialEq, Eq, Debug)]
#[contracttype]
pub enum ChainId {
    Stellar,
    Ethereum,
    Polygon,
    Avalanche,
    BinanceSmartChain,
    Arbitrum,
    Optimism,
    Custom(u32),
}

/// Errors returned by `verify_replay_protection`.
#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum ReplayError {
    NonceReused = 1,
    MessageExpired = 2,
    ChainMismatch = 3,
    ExpiryOverflow = 4,
}

/// Storage keys used by the replay-protection library.
/// These are stored directly in the calling contract's namespace using
/// unique variant discriminants so they never collide with contract-local keys.
#[contracttype]
pub enum DataKey {
    Nonce(BytesN<32>),
    SeenMessage(BytesN<32>),
}

/// Verify all three replay protections in a single call:
///
/// 1. **Nonce uniqueness** — each `(sender_key, nonce)` pair must be strictly
///    increasing, preventing message re-submission.
/// 2. **Message expiration** — `timestamp + ttl_secs` must be >= the current
///    ledger time, preventing stale messages from being executed.
/// 3. **Chain binding** — `source_chain` must equal `expected_source_chain`,
///    preventing a message emitted on chain A from being replayed on chain B.
///
/// On success the nonce is recorded and the message hash is marked as seen.
///
/// # Arguments
/// * `env` — Soroban environment.
/// * `message_hash` — Unique message identifier (e.g. SHA-256 of the payload).
/// * `sender_key` — Opaque 32-byte sender identity (e.g. `Address::to_bytes()`
///   or `sha256(external_address_string)`).
/// * `nonce` — Monotonically increasing sequence number for this sender.
/// * `timestamp` — Ledger timestamp when the message was created.
/// * `ttl_secs` — Time-to-live in seconds from `timestamp`.
/// * `source_chain` — The chain the message claims to originate from.
/// * `expected_source_chain` — The chain we expect; returns `ChainMismatch`
///   if the two differ.
#[allow(clippy::too_many_arguments)]
pub fn verify_replay_protection(
    env: &Env,
    message_hash: &BytesN<32>,
    sender_key: &BytesN<32>,
    nonce: u64,
    timestamp: u64,
    ttl_secs: u64,
    source_chain: &ChainId,
    expected_source_chain: &ChainId,
) -> Result<(), ReplayError> {
    let now = env.ledger().timestamp();

    // 1. Nonce uniqueness — reject if nonce not strictly increasing
    let nonce_key = DataKey::Nonce(sender_key.clone());
    let last_nonce: u64 = env.storage().persistent().get(&nonce_key).unwrap_or(0);
    if nonce <= last_nonce {
        return Err(ReplayError::NonceReused);
    }
    env.storage().persistent().set(&nonce_key, &nonce);

    // 2. Expiration — reject if current time past deadline
    let expires_at = timestamp.checked_add(ttl_secs).ok_or(ReplayError::ExpiryOverflow)?;
    if now > expires_at {
        return Err(ReplayError::MessageExpired);
    }

    // 3. Chain binding — reject if source chain differs from expected
    if source_chain != expected_source_chain {
        return Err(ReplayError::ChainMismatch);
    }

    // Mark as seen for idempotency
    env.storage()
        .persistent()
        .set(&DataKey::SeenMessage(message_hash.clone()), &true);

    Ok(())
}

/// Query whether a message hash has already been processed.
pub fn is_message_seen(env: &Env, message_hash: &BytesN<32>) -> bool {
    env.storage()
        .persistent()
        .get::<DataKey, bool>(&DataKey::SeenMessage(message_hash.clone()))
        .unwrap_or(false)
}

/// Check only the expiration half of replay protection.
/// Useful at confirm/execute time when nonce and chain binding
/// were already verified at submission.
pub fn check_message_expired(env: &Env, timestamp: u64, ttl_secs: u64) -> Result<(), ReplayError> {
    let now = env.ledger().timestamp();
    let expires_at = timestamp.checked_add(ttl_secs).ok_or(ReplayError::ExpiryOverflow)?;
    if now > expires_at {
        return Err(ReplayError::MessageExpired);
    }
    Ok(())
}
