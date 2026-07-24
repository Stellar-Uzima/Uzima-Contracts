#![no_std]
#![forbid(alloc)]

use soroban_sdk::Address;

/// Core authorization check: verifies a caller matches the expected admin address.
/// Returns `Ok(())` if authorized, `Err(())` otherwise.
pub fn check_admin(caller: &Address, admin: &Address) -> Result<(), ()> {
    if caller == admin {
        Ok(())
    } else {
        Err(())
    }
}

/// Convenience wrapper: returns `true` when the caller is the admin.
pub fn is_admin(caller: &Address, admin: &Address) -> bool {
    caller == admin
}

/// Macro to generate a `require_admin` function that reads the admin address
/// from instance storage using `DataKey::Admin` and returns an `Error`.
///
/// The calling module must have `DataKey::Admin` and `Error::NotInitialized`
/// / `Error::NotAuthorized` in scope.
///
/// # Example
/// ```ignore
/// require_admin!()
/// ```
/// expands to:
/// ```ignore
/// fn require_admin(env: &Env, caller: &Address) -> Result<(), Error> {
///     let admin: Address = env.storage().instance()
///         .get(&DataKey::Admin)
///         .ok_or(Error::NotInitialized)?;
///     if caller != &admin {
///         return Err(Error::NotAuthorized);
///     }
///     Ok(())
/// }
/// ```
#[macro_export]
macro_rules! require_admin {
    () => {
        fn require_admin(
            env: &soroban_sdk::Env,
            caller: &soroban_sdk::Address,
        ) -> Result<(), Error> {
            let admin: soroban_sdk::Address = env
                .storage()
                .instance()
                .get(&DataKey::Admin)
                .ok_or(Error::NotInitialized)?;
            if caller != &admin {
                return Err(Error::NotAuthorized);
            }
            Ok(())
        }
    };
}

/// Macro to generate a `require_admin` function with custom storage key,
/// storage type (`instance` or `persistent`), and error variants.
#[macro_export]
macro_rules! require_admin_custom {
    ($store:ident, $key:expr, $not_init:path, $not_auth:path) => {
        fn require_admin(
            env: &soroban_sdk::Env,
            caller: &soroban_sdk::Address,
        ) -> Result<(), Error> {
            let admin: soroban_sdk::Address = env
                .storage()
                .$store()
                .get(&$key)
                .ok_or($not_init)?;
            if caller != &admin {
                return Err($not_auth);
            }
            Ok(())
        }
    };
}
