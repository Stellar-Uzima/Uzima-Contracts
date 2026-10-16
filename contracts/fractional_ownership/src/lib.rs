#![no_std]
//! # FractionalOwnership
//!
//! @title FractionalOwnership
//! @notice Splits ownership of an asset into fungible shares that can be
//!         transferred, sold on a simple fixed-price order book, and entitled
//!         to a pro-rata share of payouts (for example rental income).
//! @dev Amounts are `i128`. Shares are tracked per asset id. Payouts use a
//!      cumulative "payout index" (payout per share, scaled by
//!      [`PAYOUT_PRECISION`]) so that deposits cost O(1) regardless of the
//!      number of holders. A holder's entitlement is settled lazily, always
//!      before that holder's balance changes.
//!
//! ## Rounding
//! Every division floors, so the contract can never owe more than it holds.
//! The remainder ("dust", less than one payment-token base unit per deposit
//! and per settlement) stays in the contract.
//!
//! ## Reentrancy
//! The only external calls are to the payment token, in `buy`,
//! `deposit_payout` and `claim_payout`. All three update their own state
//! first (checks-effects-interactions) and also hold a storage lock for
//! the duration of the call. Every other state-changing entry point
//! refuses to run while the lock is held. Soroban itself also forbids a
//! contract from being re-entered; the lock is defense in depth. See
//! `docs/FRACTIONAL_OWNERSHIP_SECURITY_REVIEW.md`.
//!
//! ## Storage
//! Instance: `Admin`, `PaymentToken`, `AssetCount`, `Locked`.
//! Persistent: balances, listings, payout index, checkpoints, pending claims.

#[cfg(test)]
extern crate std;

mod errors;
mod types;

#[cfg(test)]
mod test;

#[cfg(test)]
mod test_reentrancy;

#[cfg(test)]
mod fuzz_test;

pub use errors::Error;
pub use types::{DataKey, Listing};

use soroban_sdk::{contract, contractimpl, symbol_short, token, Address, Env, IntoVal, Val};

/// @notice Fixed-point scale applied to the cumulative payout index.
pub const PAYOUT_PRECISION: i128 = 1_000_000_000_000;

const LEDGERS_PER_DAY: u32 = 17_280;
const TTL_THRESHOLD: u32 = 30 * LEDGERS_PER_DAY;
const TTL_EXTEND_TO: u32 = 60 * LEDGERS_PER_DAY;

#[contract]
pub struct FractionalOwnership;

#[contractimpl]
impl FractionalOwnership {
    /// @notice One-time setup of the contract.
    /// @dev Requires `admin` authorization. Can only succeed once.
    /// @param admin Address allowed to create assets.
    /// @param payment_token Token contract used for purchases and payouts.
    /// @return `Ok(())` on success.
    /// @custom:error AlreadyInitialized The contract was already initialized.
    pub fn initialize(env: Env, admin: Address, payment_token: Address) -> Result<(), Error> {
        if env.storage().instance().has(&DataKey::Admin) {
            return Err(Error::AlreadyInitialized);
        }
        admin.require_auth();
        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage()
            .instance()
            .set(&DataKey::PaymentToken, &payment_token);
        env.storage().instance().set(&DataKey::AssetCount, &0u64);
        extend_instance(&env);
        env.events().publish(
            (symbol_short!("init"),),
            (admin.clone(), payment_token.clone()),
        );
        Ok(())
    }

    /// @notice Create a new fractionalized asset and mint every share to `issuer`.
    /// @dev Only the admin can call this. Asset ids start at 1 and increase
    ///      by one per asset. The total supply is fixed at creation.
    /// @param admin The stored admin; must authorize the call.
    /// @param issuer Initial holder of all minted shares.
    /// @param total_shares Total supply of shares, must be positive.
    /// @return The new asset id.
    /// @custom:error NotInitialized The contract is not initialized.
    /// @custom:error Unauthorized `admin` is not the stored admin.
    /// @custom:error InvalidAmount `total_shares` is zero or negative.
    /// @custom:error Reentrancy A guarded call (an external token call) is in progress.
    pub fn create_asset(
        env: Env,
        admin: Address,
        issuer: Address,
        total_shares: i128,
    ) -> Result<u64, Error> {
        ensure_unlocked(&env)?;
        let stored = read_admin(&env)?;
        admin.require_auth();
        if admin != stored {
            return Err(Error::Unauthorized);
        }
        if total_shares <= 0 {
            return Err(Error::InvalidAmount);
        }
        let count: u64 = env
            .storage()
            .instance()
            .get(&DataKey::AssetCount)
            .unwrap_or(0);
        let asset_id = count.checked_add(1).ok_or(Error::Overflow)?;
        env.storage()
            .instance()
            .set(&DataKey::AssetCount, &asset_id);
        extend_instance(&env);

        put(&env, &DataKey::TotalShares(asset_id), &total_shares);
        put(
            &env,
            &DataKey::Balance(asset_id, issuer.clone()),
            &total_shares,
        );
        env.events().publish(
            (symbol_short!("created"), asset_id),
            (issuer.clone(), total_shares),
        );
        Ok(asset_id)
    }

    /// @notice Move shares from `from` to `to`.
    /// @dev Settles both parties' pending payouts first so the transfer never
    ///      changes what either of them is owed.
    /// @param from Current holder; must authorize the call.
    /// @param to Receiver of the shares.
    /// @param asset_id Asset to transfer.
    /// @param amount Number of shares, must be positive.
    /// @custom:error InvalidAmount `amount` is zero or negative.
    /// @custom:error SelfTransfer `from` equals `to`.
    /// @custom:error AssetNotFound Unknown `asset_id`.
    /// @custom:error InsufficientShares `from` holds fewer than `amount`.
    /// @custom:error Reentrancy A guarded call (an external token call) is in progress.
    pub fn transfer(
        env: Env,
        from: Address,
        to: Address,
        asset_id: u64,
        amount: i128,
    ) -> Result<(), Error> {
        ensure_unlocked(&env)?;
        from.require_auth();
        if amount <= 0 {
            return Err(Error::InvalidAmount);
        }
        if from == to {
            return Err(Error::SelfTransfer);
        }
        total_shares_of(&env, asset_id)?;
        move_shares(&env, asset_id, &from, &to, amount)?;
        env.events().publish(
            (symbol_short!("transfer"), asset_id),
            (from.clone(), to.clone(), amount),
        );
        Ok(())
    }

    /// @notice Offer `amount` shares for sale at a fixed price per share.
    /// @dev Shares are not escrowed. The seller's balance is re-checked when
    ///      someone buys. A new listing replaces any earlier one for the same
    ///      seller and asset.
    /// @param seller Holder making the offer; must authorize the call.
    /// @param asset_id Asset being sold.
    /// @param amount Shares on offer, must be positive and not exceed the balance.
    /// @param price_per_share Price in payment-token base units, must be positive.
    /// @custom:error InvalidAmount `amount` or `price_per_share` is not positive.
    /// @custom:error AssetNotFound Unknown `asset_id`.
    /// @custom:error InsufficientShares `seller` holds fewer than `amount`.
    /// @custom:error Reentrancy A guarded call (an external token call) is in progress.
    pub fn list_shares(
        env: Env,
        seller: Address,
        asset_id: u64,
        amount: i128,
        price_per_share: i128,
    ) -> Result<(), Error> {
        ensure_unlocked(&env)?;
        seller.require_auth();
        if amount <= 0 || price_per_share <= 0 {
            return Err(Error::InvalidAmount);
        }
        total_shares_of(&env, asset_id)?;
        if balance(&env, asset_id, &seller) < amount {
            return Err(Error::InsufficientShares);
        }
        put(
            &env,
            &DataKey::Listing(asset_id, seller.clone()),
            &Listing {
                amount,
                price_per_share,
            },
        );
        env.events().publish(
            (symbol_short!("listed"), asset_id),
            (seller.clone(), amount, price_per_share),
        );
        Ok(())
    }

    /// @notice Remove the caller's open listing for an asset.
    /// @param seller Listing owner; must authorize the call.
    /// @param asset_id Asset whose listing is removed.
    /// @custom:error NoListing `seller` has no open listing for `asset_id`.
    /// @custom:error Reentrancy A guarded call (an external token call) is in progress.
    pub fn cancel_listing(env: Env, seller: Address, asset_id: u64) -> Result<(), Error> {
        ensure_unlocked(&env)?;
        seller.require_auth();
        let key = DataKey::Listing(asset_id, seller.clone());
        if !env.storage().persistent().has(&key) {
            return Err(Error::NoListing);
        }
        env.storage().persistent().remove(&key);
        env.events()
            .publish((symbol_short!("unlisted"), asset_id), seller.clone());
        Ok(())
    }

    /// @notice Buy shares from a seller's open listing.
    /// @dev Follows checks-effects-interactions: share balances, payout
    ///      checkpoints and the listing are updated before the payment token
    ///      is called. The seller does not need to authorize; the listing is
    ///      their standing authorization at the listed price.
    /// @param buyer Purchaser; must authorize the call and pays in the payment token.
    /// @param seller Holder whose listing is being filled.
    /// @param asset_id Asset being bought.
    /// @param amount Shares to buy, must be positive and not exceed the listing.
    /// @return Total cost paid in payment-token base units.
    /// @custom:error InvalidAmount `amount` is zero or negative.
    /// @custom:error SelfTransfer `buyer` equals `seller`.
    /// @custom:error AssetNotFound Unknown `asset_id`.
    /// @custom:error NoListing `seller` has no open listing.
    /// @custom:error ExceedsListing `amount` is larger than the listed amount.
    /// @custom:error Overflow `amount * price_per_share` overflows `i128`.
    /// @custom:error InsufficientShares `seller` no longer holds `amount` shares.
    /// @custom:error Reentrancy A guarded call (an external token call) is in progress.
    pub fn buy(
        env: Env,
        buyer: Address,
        seller: Address,
        asset_id: u64,
        amount: i128,
    ) -> Result<i128, Error> {
        buyer.require_auth();
        enter(&env)?;
        if amount <= 0 {
            return Err(Error::InvalidAmount);
        }
        if buyer == seller {
            return Err(Error::SelfTransfer);
        }
        total_shares_of(&env, asset_id)?;
        let key = DataKey::Listing(asset_id, seller.clone());
        let listing: Listing = env
            .storage()
            .persistent()
            .get(&key)
            .ok_or(Error::NoListing)?;
        if amount > listing.amount {
            return Err(Error::ExceedsListing);
        }
        let cost = amount
            .checked_mul(listing.price_per_share)
            .ok_or(Error::Overflow)?;

        move_shares(&env, asset_id, &seller, &buyer, amount)?;
        let remaining = listing.amount - amount;
        if remaining == 0 {
            env.storage().persistent().remove(&key);
        } else {
            put(
                &env,
                &key,
                &Listing {
                    amount: remaining,
                    price_per_share: listing.price_per_share,
                },
            );
        }
        env.events().publish(
            (symbol_short!("bought"), asset_id),
            (buyer.clone(), seller.clone(), amount, cost),
        );

        payment_client(&env)?.transfer(&buyer, &seller, &cost);
        exit(&env);
        Ok(cost)
    }

    /// @notice Add funds to an asset's payout pool, shared pro rata by holders.
    /// @dev Raises the payout index by `amount * PAYOUT_PRECISION / total_shares`
    ///      (floored) before pulling the tokens from `depositor`.
    /// @param depositor Funds source; must authorize the call.
    /// @param asset_id Asset receiving the payout.
    /// @param amount Payment-token base units to distribute, must be positive.
    /// @custom:error InvalidAmount `amount` is zero or negative.
    /// @custom:error AssetNotFound Unknown `asset_id`.
    /// @custom:error Overflow The scaled amount or the index overflows `i128`.
    /// @custom:error PayoutTooSmall The deposit is too small to raise the index
    ///               (it would be unclaimable), so it is rejected.
    /// @custom:error Reentrancy A guarded call (an external token call) is in progress.
    pub fn deposit_payout(
        env: Env,
        depositor: Address,
        asset_id: u64,
        amount: i128,
    ) -> Result<(), Error> {
        depositor.require_auth();
        enter(&env)?;
        if amount <= 0 {
            return Err(Error::InvalidAmount);
        }
        let total = total_shares_of(&env, asset_id)?;
        let delta = amount
            .checked_mul(PAYOUT_PRECISION)
            .ok_or(Error::Overflow)?
            / total;
        if delta == 0 {
            return Err(Error::PayoutTooSmall);
        }
        let index = payout_index(&env, asset_id)
            .checked_add(delta)
            .ok_or(Error::Overflow)?;
        put(&env, &DataKey::PayoutIndex(asset_id), &index);
        env.events().publish(
            (symbol_short!("deposit"), asset_id),
            (depositor.clone(), amount),
        );

        payment_client(&env)?.transfer(&depositor, &env.current_contract_address(), &amount);
        exit(&env);
        Ok(())
    }

    /// @notice Withdraw everything the caller is owed from an asset's payout pool.
    /// @dev Settles, zeroes the pending balance, then transfers (checks-
    ///      effects-interactions).
    /// @param owner Share holder; must authorize the call and receives the funds.
    /// @param asset_id Asset to claim from.
    /// @return Amount paid out in payment-token base units.
    /// @custom:error AssetNotFound Unknown `asset_id`.
    /// @custom:error Overflow Settlement arithmetic overflowed.
    /// @custom:error NothingToClaim Nothing is owed to `owner`.
    /// @custom:error Reentrancy A guarded call (an external token call) is in progress.
    pub fn claim_payout(env: Env, owner: Address, asset_id: u64) -> Result<i128, Error> {
        owner.require_auth();
        enter(&env)?;
        total_shares_of(&env, asset_id)?;
        settle(&env, asset_id, &owner)?;
        let key = DataKey::Pending(asset_id, owner.clone());
        let owed = read_i128(&env, &key);
        if owed <= 0 {
            return Err(Error::NothingToClaim);
        }
        put(&env, &key, &0i128);
        env.events()
            .publish((symbol_short!("claimed"), asset_id), (owner.clone(), owed));

        payment_client(&env)?.transfer(&env.current_contract_address(), &owner, &owed);
        exit(&env);
        Ok(owed)
    }

    /// @notice The stored admin address.
    /// @custom:error NotInitialized The contract is not initialized.
    pub fn get_admin(env: Env) -> Result<Address, Error> {
        read_admin(&env)
    }

    /// @notice The token used for purchases and payouts.
    /// @custom:error NotInitialized The contract is not initialized.
    pub fn get_payment_token(env: Env) -> Result<Address, Error> {
        env.storage()
            .instance()
            .get(&DataKey::PaymentToken)
            .ok_or(Error::NotInitialized)
    }

    /// @notice Number of assets created so far (also the latest asset id).
    pub fn get_asset_count(env: Env) -> u64 {
        env.storage()
            .instance()
            .get(&DataKey::AssetCount)
            .unwrap_or(0)
    }

    /// @notice Fixed total supply of an asset.
    /// @custom:error AssetNotFound Unknown `asset_id`.
    pub fn get_total_shares(env: Env, asset_id: u64) -> Result<i128, Error> {
        total_shares_of(&env, asset_id)
    }

    /// @notice Shares of `asset_id` held by `owner` (zero if none).
    pub fn get_balance(env: Env, asset_id: u64, owner: Address) -> i128 {
        balance(&env, asset_id, &owner)
    }

    /// @notice The open listing of `seller` for `asset_id`, if any.
    pub fn get_listing(env: Env, asset_id: u64, seller: Address) -> Option<Listing> {
        env.storage()
            .persistent()
            .get(&DataKey::Listing(asset_id, seller))
    }

    /// @notice Cumulative payout per share, scaled by [`PAYOUT_PRECISION`].
    pub fn get_payout_index(env: Env, asset_id: u64) -> i128 {
        payout_index(&env, asset_id)
    }

    /// @notice What `owner` could claim right now, including unsettled accrual.
    /// @custom:error Overflow Accrual arithmetic overflowed.
    pub fn get_pending_payout(env: Env, asset_id: u64, owner: Address) -> Result<i128, Error> {
        let settled = read_i128(&env, &DataKey::Pending(asset_id, owner.clone()));
        let unsettled = accrued(&env, asset_id, &owner)?;
        settled.checked_add(unsettled).ok_or(Error::Overflow)
    }
}

/// Fail with `Reentrancy` if a guarded external call is in progress.
fn ensure_unlocked(env: &Env) -> Result<(), Error> {
    if env
        .storage()
        .instance()
        .get::<_, bool>(&DataKey::Locked)
        .unwrap_or(false)
    {
        return Err(Error::Reentrancy);
    }
    Ok(())
}

/// Take the reentrancy lock. Released by [`exit`] on the success path;
/// a failing call reverts all storage writes, including the lock.
fn enter(env: &Env) -> Result<(), Error> {
    ensure_unlocked(env)?;
    env.storage().instance().set(&DataKey::Locked, &true);
    Ok(())
}

/// Release the reentrancy lock.
fn exit(env: &Env) {
    env.storage().instance().set(&DataKey::Locked, &false);
}

fn read_admin(env: &Env) -> Result<Address, Error> {
    env.storage()
        .instance()
        .get(&DataKey::Admin)
        .ok_or(Error::NotInitialized)
}

fn payment_client(env: &Env) -> Result<token::Client<'_>, Error> {
    let addr: Address = env
        .storage()
        .instance()
        .get(&DataKey::PaymentToken)
        .ok_or(Error::NotInitialized)?;
    Ok(token::Client::new(env, &addr))
}

fn extend_instance(env: &Env) {
    env.storage()
        .instance()
        .extend_ttl(TTL_THRESHOLD, TTL_EXTEND_TO);
}

fn put<V: IntoVal<Env, Val>>(env: &Env, key: &DataKey, value: &V) {
    env.storage().persistent().set(key, value);
    env.storage()
        .persistent()
        .extend_ttl(key, TTL_THRESHOLD, TTL_EXTEND_TO);
}

fn read_i128(env: &Env, key: &DataKey) -> i128 {
    env.storage().persistent().get(key).unwrap_or(0)
}

fn total_shares_of(env: &Env, asset_id: u64) -> Result<i128, Error> {
    env.storage()
        .persistent()
        .get(&DataKey::TotalShares(asset_id))
        .ok_or(Error::AssetNotFound)
}

fn balance(env: &Env, asset_id: u64, owner: &Address) -> i128 {
    read_i128(env, &DataKey::Balance(asset_id, owner.clone()))
}

fn payout_index(env: &Env, asset_id: u64) -> i128 {
    read_i128(env, &DataKey::PayoutIndex(asset_id))
}

/// Payout accrued by `owner` since their last checkpoint, not yet settled.
fn accrued(env: &Env, asset_id: u64, owner: &Address) -> Result<i128, Error> {
    let index = payout_index(env, asset_id);
    let checkpoint = read_i128(env, &DataKey::Checkpoint(asset_id, owner.clone()));
    let bal = balance(env, asset_id, owner);
    if index <= checkpoint || bal == 0 {
        return Ok(0);
    }
    Ok(bal.checked_mul(index - checkpoint).ok_or(Error::Overflow)? / PAYOUT_PRECISION)
}

/// Move accrued payout into the pending balance and advance the checkpoint.
fn settle(env: &Env, asset_id: u64, owner: &Address) -> Result<(), Error> {
    let owed = accrued(env, asset_id, owner)?;
    if owed > 0 {
        let key = DataKey::Pending(asset_id, owner.clone());
        let pending = read_i128(env, &key)
            .checked_add(owed)
            .ok_or(Error::Overflow)?;
        put(env, &key, &pending);
    }
    let index = payout_index(env, asset_id);
    put(env, &DataKey::Checkpoint(asset_id, owner.clone()), &index);
    Ok(())
}

/// Settle both parties, then move `amount` shares from `from` to `to`.
fn move_shares(
    env: &Env,
    asset_id: u64,
    from: &Address,
    to: &Address,
    amount: i128,
) -> Result<(), Error> {
    settle(env, asset_id, from)?;
    settle(env, asset_id, to)?;
    let from_balance = balance(env, asset_id, from);
    if from_balance < amount {
        return Err(Error::InsufficientShares);
    }
    let to_balance = balance(env, asset_id, to)
        .checked_add(amount)
        .ok_or(Error::Overflow)?;
    put(
        env,
        &DataKey::Balance(asset_id, from.clone()),
        &(from_balance - amount),
    );
    put(env, &DataKey::Balance(asset_id, to.clone()), &to_balance);
    Ok(())
}
