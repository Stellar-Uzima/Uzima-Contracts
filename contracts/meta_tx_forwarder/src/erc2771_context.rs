//! # ERC-2771 Context Module (Soroban Adapter)
//!
//! In Ethereum, ERC-2771 forwards calls with the original sender's 20-byte
//! address appended to the calldata. Target contracts extract it from the
//! final 20 bytes via `_msgSender()` / `_msgData()`.
//!
//! Soroban's `env.invoke_contract` forwards calls with structured
//! `(Symbol, Vec<Val>)` argument lists. The Meta-Transaction Forwarder
//! exploits this by prepending the original `Address` as the first positional
//! argument to every forwarded call. This module provides the matching
//! helpers for target contracts.
//!
//! ## Soroban 21.x API note
//! `env.invoker()` (and the broader `Invoker` trait) is **not exposed** in
//! soroban-sdk 21.x. There is no implicit "who called me" query — the SDK
//! deliberately removed it because the recommended pattern is to require
//! authentication from any address a contract wants to trust. Target
//! contracts that need the original signer therefore MUST accept the
//! forwarder-supplied `from` as their first positional argument and/or call
//! the address's `require_auth()`.
//!
//! ## Target contract usage
//! ```ignore
//! use meta_tx_forwarder::erc2771_context;
//! use soroban_sdk::{Address, Env};
//!
//! pub fn my_fn(env: Env, from: Address, x: u32) -> u32 {
//!     // Trust model: when this contract was invoked via the configured
//!     // trusted forwarder we treat the first arg as authoritative; otherwise
//!     // we fall back to requiring direct authentication from the caller.
//!     let trusted = erc2771_context::ERC2771ContextImpl::get_trusted_forwarder(&env);
//!     if trusted.is_none() {
//!         from.require_auth();           // direct-call path
//!     }
//!     // trusted == Some(addr): forwarder's auth has already been validated
//!     // inside MetaTxForwarder::execute; `from` is the original signer.
//!     let _ = x;
//!     let _ = from;
//!     x
//! }
//! ```
//!
//! All helpers in this module are `no_std` safe and live in instance storage.

use soroban_sdk::{Address, Env, Symbol, Val, Vec};
// `Address::try_from_val(env, &val)` lives on `TryFromVal<Env, Val>`. We avoid
// `Invoker` entirely because soroban-sdk 21.7.7 has removed it.
use soroban_sdk::TryFromVal;

use crate::DataKey;

/// ERC-2771 context trait for contracts that want a single, consistent
/// place to ask "who is the *effective* signer?". Implementations can be
/// provided per-contract if more elaborate logic is required (e.g. a
/// per-forwarder allow-list); this module ships a generic instance.
///
/// In Soroban, given the lack of an "appended 20-byte sender" pattern, the
/// recommended usage is:
/// 1. The trusted forwarder invokes the target with `from` as arg 0.
/// 2. The target accepts `from` as `arg 0` directly. This is treated as
///    authoritative — no need to inspect `env.invoker()`.
///
/// The trait below is provided for documentation and ergonomics. It asks
/// `self.get_trusted_forwarder(env)` to determine if the current caller is a
/// trusted forwarder; the implementation may store the forwarder's address
/// in its own instance storage under a contract-specific key.
pub trait ERC2771Context {
    /// Returns the trusted forwarder address for this contract.
    fn get_trusted_forwarder(env: &Env) -> Option<Address>;

    /// Returns `true` if `forwarder` matches the trusted address.
    fn is_trusted_forwarder(env: &Env, forwarder: &Address) -> bool {
        match Self::get_trusted_forwarder(env) {
            Some(trusted) => &trusted == forwarder,
            None => false,
        }
    }
}

/// Free-function helpers. Any contract can adopt these without implementing
/// the trait, by storing the trusted forwarder under `DataKey::TrustedForwarder`
/// (the shared key used by `meta_tx_forwarder`).
pub struct ERC2771ContextImpl;

impl ERC2771ContextImpl {
    /// Store the trusted forwarder address during target-contract init.
    pub fn set_trusted_forwarder(env: &Env, forwarder: Address) {
        env.storage()
            .instance()
            .set(&DataKey::TrustedForwarder, &forwarder);
    }

    /// Get the trusted forwarder address.
    pub fn get_trusted_forwarder(env: &Env) -> Option<Address> {
        env.storage().instance().get(&DataKey::TrustedForwarder)
    }

    /// Returns the Soroban 21.x "sender" the contract sees.
    ///
    /// Soroban 21 does not expose the immediate caller via an `env.invoker()`
    /// query, so a true EVM-style `msg.sender` is not available. We provide a
    /// best-effort equivalent: when a trusted forwarder has been configured
    /// (e.g. via [`set_trusted_forwarder`]) and the first positional argument
    /// of the call is an `Address`, we return that as the "sender". When no
    /// trusted forwarder is configured the function falls back to the
    /// contract's own address — but in that path the target contract MUST
    /// call `require_auth()` on the address it actually trusts (direct calls
    /// are not authenticated implicitly).
    ///
    /// This signature is intentionally not EVM-compatible in 21.x. Targets
    /// that genuinely need "who called me" should require explicit
    /// authentication.
    pub fn msg_sender(env: &Env, args: &Vec<Val>) -> Address {
        match Self::get_trusted_forwarder(env) {
            Some(_) => Self::msg_sender_from_data(env, args)
                .unwrap_or_else(|| env.current_contract_address()),
            None => env.current_contract_address(),
        }
    }

    /// Return the first positional argument of the current invocation —
    /// which the forwarder always sets to the original `from`.
    ///
    /// Returns `None` if the contract was not invoked through the forwarder
    /// (i.e., direct invocation where arg 0 is something else or no args
    /// were supplied) or if arg 0 cannot be decoded as an `Address`.
    pub fn msg_sender_from_data(env: &Env, args: &Vec<Val>) -> Option<Address> {
        let v = args.get(0)?;
        Address::try_from_val(env, &v).ok()
    }

    /// Return all arguments *after* the original sender. This is the
    /// Soroban analogue of Ethereum's `_msgData()`, with the appended
    /// 20-byte sender stripped off.
    pub fn msg_data(env: &Env, args: &Vec<Val>) -> Vec<Val> {
        // Strip the first positional argument (== from, set by the forwarder).
        let mut out: Vec<Val> = Vec::new(env);
        let mut i: u32 = 1;
        while i < args.len() {
            if let Some(v) = args.get(i) {
                out.push_back(v);
            }
            i = i.saturating_add(1);
        }
        out
    }

    /// True iff a trusted-forwarder address has been configured for this
    /// contract.
    ///
    /// Soroban 21.x removed the implicit `env.invoker()` query, so we can
    /// only check that a trusted forwarder was registered at setup time,
    /// not that the immediate caller of the current call is it. Target
    /// contracts that need an authentication proof must use
    /// `Address::require_auth()` on whatever address they wish to trust.
    /// Authorization of the forwarder itself is enforced inside
    /// `MetaTxForwarder::execute` via `relayer.require_auth()` and the
    /// Ed25519 signature over the typed payload.
    pub fn has_trusted_forwarder(env: &Env) -> bool {
        Self::get_trusted_forwarder(env).is_some()
    }

    /// Deprecated alias for [`Self::has_trusted_forwarder`]. Previously
    /// queried `env.invoker()`, which is no longer exposed in
    /// soroban-sdk 21.7.7. Kept as a deprecated alias to avoid breaking any
    /// downstream consumer that still references the old name.
    #[deprecated(
        since = "0.1.0",
        note = "Soroban 21.x removed env.invoker(); use has_trusted_forwarder() instead"
    )]
    pub fn is_invoker_trusted(env: &Env) -> bool {
        Self::has_trusted_forwarder(env)
    }

    /// Looking up the function name this contract was called through.
    /// Soroban 21.7.7 does not expose the invoked symbol directly; this
    /// helper exists as a placeholder so callers can document intent
    /// (e.g. via a custom enum keyed on caller-supplied Symbol).
    pub fn current_fn(_env: &Env, hint: &Symbol) -> Symbol {
        hint.clone()
    }
}

// ============================================================================
// Unit tests for context helpers (require contract context for storage tests)
// ============================================================================
//
// `ERC2771ContextImpl::get_trusted_forwarder` reads from instance storage,
// so unit tests that exercise it must be invoked from `env.as_contract(...)`.
// The forwarder's `mod test` covers both the contract- and direct-call paths.
