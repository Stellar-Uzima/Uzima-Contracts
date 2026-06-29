/// Requires that the caller is authenticated and matches the admin check.
///
/// Expands to: `caller.require_auth(); Self::require_admin(&env, &caller)?;`
#[macro_export]
macro_rules! require_admin {
    ($env:expr, $caller:expr) => {
        $caller.require_auth();
        Self::require_admin(&$env, &$caller)?;
    };
}

/// Requires that the caller is authenticated and matches the role check.
///
/// Expands to: `caller.require_auth(); Self::require_role(&env, &caller, role)?;`
#[macro_export]
macro_rules! require_role {
    ($env:expr, $caller:expr, $role:expr) => {
        $caller.require_auth();
        Self::require_role(&$env, &$caller, $role)?;
    };
}
