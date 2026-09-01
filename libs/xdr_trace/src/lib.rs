//! A shared, typed, lossless decoder for Soroban contract-invocation trace XDR.
//!
//! The repository holds substantial on-chain observability scaffolding but,
//! before this crate, no off-chain code decoded the XDR that a Soroban
//! transaction produces. Every consumer instead re-implemented decoding ad hoc
//! in shell, Python, and JS across `scripts/` — a maintenance trap, because one
//! XDR layout change silently drifts half a dozen independent parsers.
//!
//! This crate is the M1 foundation for every subsequent trace-consumer (CLI
//! extractor, fuzz trace invariants, trace schema). It turns a single XDR
//! buffer — a `soroban_sdk::xdr::SorobanTransactionMeta` payload, i.e. a
//! Soroban contract-invocation result with its attached environment
//! diagnostics — into a typed, lossless [`ContractTrace`], without
//! panicking on malformed or truncated input.
//!
//! # What is decoded
//!
//! The trace is reconstructed from both parts of the meta payload:
//!
//! - the typed `return_value` and the ordered [`ContractEvent`]s emitted by the
//!   contract itself, and
//! - the [`DiagnosticEvent`]s, which include the host's `fn_call`/`fn_return`
//!   control events. The outermost `fn_call` (the one with no calling contract)
//!   carries the invoked contract id, the function symbol, and the `ScVal`
//!   arguments; every `fn_call` contributes to the set of transitively invoked
//!   contract addresses.
//!
//! Decoding is **lossless**: argument and return values stay in their `ScVal`
//! representation, and every event is retained in the order it was emitted.
//!
//! # Example
//!
//! ```
//! use xdr_trace::{decode_trace, TraceError};
//!
//! fn handle(bytes: &[u8]) -> Result<(), TraceError> {
//!     let trace = decode_trace(bytes)?;
//!     println!(
//!         "{}: {} -> {:?}",
//!         trace.contract_id,
//!         trace.function_name,
//!         trace.return_value,
//!     );
//!     Ok(())
//! }
//! ```

pub mod decode;
pub mod error;
pub mod json;
pub mod model;
pub mod validate;

pub use decode::{decode_contract_events, decode_diagnostic_events, decode_trace};
pub use error::TraceError;
pub use json::{
    address_to_strkey, contract_id_to_strkey, derive_trace_id, format_events, scval_to_native_json,
    scval_to_typed_value, ContractTraceJson, TraceEventJson, TraceJsonOptions, TypedValue,
};
pub use model::{ContractId, ContractTrace};
pub use validate::validate_trace_record;

pub use soroban_sdk::xdr::{ContractEvent, DiagnosticEvent, ScVal};
