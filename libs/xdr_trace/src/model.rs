//! Typed, lossless model of a decoded Soroban invocation trace.

use core::fmt;

use soroban_sdk::xdr::{ContractEvent, DiagnosticEvent, ScVal};

use crate::error::TraceError;

/// The 32-byte identifier of a Soroban contract, as it appears in `fn_call`
/// diagnostic-event topics.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ContractId(pub [u8; 32]);

impl ContractId {
    /// Returns the raw 32-byte contract identifier.
    #[must_use]
    pub const fn to_bytes(&self) -> &[u8; 32] {
        &self.0
    }
}

impl AsRef<[u8]> for ContractId {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl TryFrom<&[u8]> for ContractId {
    type Error = TraceError;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        let bytes: [u8; 32] = value
            .try_into()
            .map_err(|_| TraceError::MalformedControlEvent("contract id is not 32 bytes"))?;
        Ok(ContractId(bytes))
    }
}

impl fmt::Display for ContractId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for byte in self.0.iter() {
            write!(f, "{byte:02x}")?;
        }
        Ok(())
    }
}

impl fmt::Debug for ContractId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ContractId({self})")
    }
}

/// A full, typed, lossless trace of a Soroban contract invocation.
///
/// Everything in this model is decoded from a single XDR buffer — a
/// `soroban_sdk::xdr::SorobanTransactionMeta` payload — and is independent of
/// how the bytes were obtained (file, stdin, HTTP, ...). Argument and return
/// values keep their `ScVal` representation and every event is retained in
/// emission order, so no information is lost during decoding.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ContractTrace {
    /// The contract that was invoked at the root of the trace.
    pub contract_id: ContractId,
    /// The invoked function symbol.
    pub function_name: String,
    /// The `ScVal`-typed arguments the function was invoked with.
    pub arguments: Vec<ScVal>,
    /// The `ScVal`-typed return value of the root invocation.
    pub return_value: ScVal,
    /// The ordered `ContractEvent`s emitted by contracts during the call.
    pub events: Vec<ContractEvent>,
    /// The ordered `DiagnosticEvent`s (contract events plus host debug events,
    /// including events emitted by failed sub-calls).
    pub diagnostic_events: Vec<DiagnosticEvent>,
    /// The deduplicated, sorted set of contracts invoked transitively.
    pub invoked_contract_ids: Vec<ContractId>,
}