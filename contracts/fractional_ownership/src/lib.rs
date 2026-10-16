#![no_std]

#[cfg(test)]
extern crate std;

mod errors;
mod types;

#[cfg(test)]
mod test;

pub use errors::Error;
pub use types::{DataKey, Listing};

use soroban_sdk::{contract, contractimpl, symbol_short, token, Address, Env, IntoVal, Val};

pub const PAYOUT_PRECISION: i128 = 1_000_000_000_000;

const LEDGERS_PER_DAY: u32 = 17_280;
const TTL_THRESHOLD: u32 = 30 * LEDGERS_PER_DAY;
const TTL_EXTEND_TO: u32 = 60 * LEDGERS_PER_DAY;

#[contract]
pub struct FractionalOwnership;

#[contractimpl]
impl FractionalOwnership {
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

    pub fn create_asset(
        env: Env,
        admin: Address,
        issuer: Address,
        total_shares: i128,
    ) -> Result<u64, Error> {
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

    pub fn transfer(
        env: Env,
        from: Address,
        to: Address,
        asset_id: u64,
        amount: i128,
    ) -> Result<(), Error> {
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

    pub fn list_shares(
        env: Env,
        seller: Address,
        asset_id: u64,
        amount: i128,
        price_per_share: i128,
    ) -> Result<(), Error> {
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

    pub fn cancel_listing(env: Env, seller: Address, asset_id: u64) -> Result<(), Error> {
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

    pub fn buy(
        env: Env,
        buyer: Address,
        seller: Address,
        asset_id: u64,
        amount: i128,
    ) -> Result<i128, Error> {
        buyer.require_auth();
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
        Ok(cost)
    }

    pub fn deposit_payout(
        env: Env,
        depositor: Address,
        asset_id: u64,
        amount: i128,
    ) -> Result<(), Error> {
        depositor.require_auth();
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
        Ok(())
    }

    pub fn claim_payout(env: Env, owner: Address, asset_id: u64) -> Result<i128, Error> {
        owner.require_auth();
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
        Ok(owed)
    }

    pub fn get_admin(env: Env) -> Result<Address, Error> {
        read_admin(&env)
    }

    pub fn get_payment_token(env: Env) -> Result<Address, Error> {
        env.storage()
            .instance()
            .get(&DataKey::PaymentToken)
            .ok_or(Error::NotInitialized)
    }

    pub fn get_asset_count(env: Env) -> u64 {
        env.storage()
            .instance()
            .get(&DataKey::AssetCount)
            .unwrap_or(0)
    }

    pub fn get_total_shares(env: Env, asset_id: u64) -> Result<i128, Error> {
        total_shares_of(&env, asset_id)
    }

    pub fn get_balance(env: Env, asset_id: u64, owner: Address) -> i128 {
        balance(&env, asset_id, &owner)
    }

    pub fn get_listing(env: Env, asset_id: u64, seller: Address) -> Option<Listing> {
        env.storage()
            .persistent()
            .get(&DataKey::Listing(asset_id, seller))
    }

    pub fn get_payout_index(env: Env, asset_id: u64) -> i128 {
        payout_index(&env, asset_id)
    }

    pub fn get_pending_payout(env: Env, asset_id: u64, owner: Address) -> Result<i128, Error> {
        let settled = read_i128(&env, &DataKey::Pending(asset_id, owner.clone()));
        let unsettled = accrued(&env, asset_id, &owner)?;
        settled.checked_add(unsettled).ok_or(Error::Overflow)
    }
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

fn accrued(env: &Env, asset_id: u64, owner: &Address) -> Result<i128, Error> {
    let index = payout_index(env, asset_id);
    let checkpoint = read_i128(env, &DataKey::Checkpoint(asset_id, owner.clone()));
    let bal = balance(env, asset_id, owner);
    if index <= checkpoint || bal == 0 {
        return Ok(0);
    }
    Ok(bal.checked_mul(index - checkpoint).ok_or(Error::Overflow)? / PAYOUT_PRECISION)
}

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
