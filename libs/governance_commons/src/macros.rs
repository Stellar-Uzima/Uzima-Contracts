/// Enforces admin-only access on functions that return `Result`.
///
/// Expands to:
/// ```rust,ignore
/// caller.require_auth();
/// Self::require_admin(&env, &caller)?;
/// ```
///
/// The contract's `impl` block must provide a `fn require_admin(env: &Env, caller: &Address) -> Result<(), E>`
/// method and the enclosing function must return a `Result`.
///
/// # Example
/// ```rust,ignore
/// use governance_commons::require_admin;
///
/// pub fn pause(env: Env, caller: Address) -> Result<bool, Error> {
///     require_admin!(env, caller);
///     env.storage().instance().set(&DataKey::Paused, &true);
///     Ok(true)
/// }
/// ```
#[macro_export]
macro_rules! require_admin {
    ($env:expr, $caller:expr) => {{
        $caller.require_auth();
        Self::require_admin(&$env, &$caller)?;
    }};
}

/// Enforces role-based access on functions that return `Result`.
///
/// Expands to:
/// ```rust,ignore
/// caller.require_auth();
/// Self::require_role(&env, &caller, role)?;
/// ```
///
/// The contract's `impl` block must provide a `fn require_role(env: &Env, caller: &Address, role: R) -> Result<(), E>`
/// method and the enclosing function must return a `Result`.
///
/// # Example
/// ```rust,ignore
/// use governance_commons::require_role;
///
/// pub fn submit_report(env: Env, caller: Address) -> Result<u64, Error> {
///     require_role!(env, caller, Role::Auditor);
///     // ...
///     Ok(id)
/// }
/// ```
#[macro_export]
macro_rules! require_role {
    ($env:expr, $caller:expr, $role:expr) => {{
        $caller.require_auth();
        Self::require_role(&$env, &$caller, $role)?;
    }};
}
