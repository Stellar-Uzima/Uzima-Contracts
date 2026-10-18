#![no_std]
//! # RentalYield
//!
//! @title RentalYield
//! @notice Holds rental income deposited by the admin and lets each
//!         beneficiary claim the yield credited to them.
//!
//! ## Errors
//! Every failure returns a typed [`Error`] instead of panicking.
//!
//! ## Reentrancy
//! The only external calls are to the payment token, in `deposit_yield`
//! and `claim`. Both update state first (checks-effects-interactions) and
//! hold a storage lock for the duration of the call. Soroban also forbids
//! a contract from being re-entered; the lock is defense in depth.

mod errors;

pub use errors::Error;

use soroban_sdk::{contract, contractimpl, contracttype, symbol_short, token, Address, Env};

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DataKey {
    Admin,
    PaymentToken,
    Locked,
    Claimable(Address),
}

#[contract]
pub struct RentalYield;

#[contractimpl]
impl RentalYield {
    /// @notice One-time setup of the contract.
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
        Ok(())
    }

    /// @notice Pull `amount` from the admin and credit it to `beneficiary`.
    /// @custom:error Unauthorized `admin` is not the stored admin.
    /// @custom:error InvalidAmount `amount` is zero or negative.
    /// @custom:error Overflow The claimable balance overflows `i128`.
    /// @custom:error Reentrancy A guarded call is in progress.
    pub fn deposit_yield(
        env: Env,
        admin: Address,
        beneficiary: Address,
        amount: i128,
    ) -> Result<(), Error> {
        admin.require_auth();
        enter(&env)?;
        if admin != read_admin(&env)? {
            return Err(Error::Unauthorized);
        }
        if amount <= 0 {
            return Err(Error::InvalidAmount);
        }
        let key = DataKey::Claimable(beneficiary.clone());
        let claimable = claimable_of(&env, &beneficiary)
            .checked_add(amount)
            .ok_or(Error::Overflow)?;
        env.storage().persistent().set(&key, &claimable);
        env.events()
            .publish((symbol_short!("deposit"),), (beneficiary, amount));

        payment_client(&env)?.transfer(&admin, &env.current_contract_address(), &amount);
        exit(&env);
        Ok(())
    }

    /// @notice Withdraw everything credited to `beneficiary`.
    /// @dev Zeroes the balance before transferring (checks-effects-interactions).
    /// @custom:error NothingToClaim Nothing is owed to `beneficiary`.
    /// @custom:error Reentrancy A guarded call is in progress.
    pub fn claim(env: Env, beneficiary: Address) -> Result<i128, Error> {
        beneficiary.require_auth();
        enter(&env)?;
        let owed = claimable_of(&env, &beneficiary);
        if owed <= 0 {
            return Err(Error::NothingToClaim);
        }
        env.storage()
            .persistent()
            .set(&DataKey::Claimable(beneficiary.clone()), &0i128);
        env.events()
            .publish((symbol_short!("claimed"),), (beneficiary.clone(), owed));

        payment_client(&env)?.transfer(&env.current_contract_address(), &beneficiary, &owed);
        exit(&env);
        Ok(owed)
    }

    /// @notice Yield currently claimable by `beneficiary`.
    pub fn get_claimable(env: Env, beneficiary: Address) -> i128 {
        claimable_of(&env, &beneficiary)
    }
}

/// Take the reentrancy lock. A failing call reverts all writes, including the lock.
fn enter(env: &Env) -> Result<(), Error> {
    if env
        .storage()
        .instance()
        .get::<_, bool>(&DataKey::Locked)
        .unwrap_or(false)
    {
        return Err(Error::Reentrancy);
    }
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

fn claimable_of(env: &Env, beneficiary: &Address) -> i128 {
    env.storage()
        .persistent()
        .get(&DataKey::Claimable(beneficiary.clone()))
        .unwrap_or(0)
}
