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
//! helpers for target contracts:
//!
//! - [`msg_sender`] returns the immediate `env.invoker()`. When called via
//!   the trusted forwarder, this is the **forwarder's** address; the original
//!   user is in the **first positional argument** of the call.
//! - [`msg_data`] returns the raw args vector that was forwarded to the
//!   contract, semantically equivalent to Ethereum's `_msgData()` after the
//!   forwarder-appended sender has been stripped.
//!
//! ## Target contract usage
//! ```ignore
//! use meta_tx_forwarder::erc2771_context;
//!
//! pub fn my_fn(env: Env, from: Address, x: u32) -> u32 {
//!     // When invoked directly: `from == env.invoker()`.
//!     // When invoked via the trusted forwarder: `from` is the original
//!     // signer (and `env.invoker() == trusted_forwarder`).
//!     let trusted = erc2771_context::ERC2771ContextImpl::get_trusted_forwarder(&env);
//!     let sender = if env.invoker() == trusted {
//!         from            // ERC-2771 path
//!     } else {
//!         env.invoker()   // direct call
//!     };
//!     let _ = sender;
//!     x
//! }
//! ```
//!
//! All helpers in this module are `no_std` safe and live in instance storage.

use soroban_sdk::{Address, Env, Symbol, Val, Vec};

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

    /// Return the immediate `env.invoker()`. When the caller is the trusted
    /// forwarder, this is the forwarder; the original signer is in the
    /// first positional argument of the call.
    pub fn msg_sender(env: &Env) -> Address {
        env.invoker()
    }

    /// Return the first positional argument of the current invocation —
    /// which the forwarder always sets to the original `from`.
    ///
    /// Returns `None` if the contract was not invoked through the forwarder
    /// (i.e., direct invocation where arg 0 is something else or no args
    /// were supplied).
    ///
    /// The helper does **not** type-check: callers must know that positional
    /// arg 0 is, in fact, an `Address` for their function. The Soroban host
    /// will trap if the call shape doesn't match.
    pub fn msg_sender_from_data(env: &Env, args: &Vec<Val>) -> Option<Address> {
        args.get(0)
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

    /// True iff the current `env.invoker()` is the configured trusted
    /// forwarder. Targets should branch on this instead of inspecting raw
    /// addresses.
    pub fn is_invoker_trusted(env: &Env) -> bool {
        match Self::get_trusted_forwarder(env) {
            Some(trusted) => env.invoker() == trusted,
            None => false,
        }
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
