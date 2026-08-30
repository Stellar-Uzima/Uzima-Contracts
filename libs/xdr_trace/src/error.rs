//! Decoder error type.

use core::fmt;

use soroban_sdk::xdr::Error as XdrError;

/// Errors produced while decoding XDR trace payloads.
///
/// Every failure mode the decoder knows about is a typed variant, so a caller
/// can distinguish a malformed buffer from a structurally-correct buffer that
/// does not contain a Soroban invocation trace. Consistent with the crate's
/// contract, decode failures are *always* reported through this type — hostile
/// input never causes a panic.
#[derive(Debug, PartialEq)]
pub enum TraceError {
    /// The input buffer was empty.
    EmptyInput,
    /// The buffer is not valid XDR for the expected type (truncated, trailing
    /// garbage, or otherwise malformed).
    InvalidXdr(XdrError),
    /// The buffer decoded, but contains no host-level `fn_call` diagnostic
    /// event, so no invocation trace can be reconstructed.
    MissingInvocationRecord,
    /// A structurally-valid control event (`fn_call`/`fn_return`) violated a
    /// well-formedness rule (bad topic shape, non-UTF-8 symbol, ...).
    MalformedControlEvent(&'static str),
    /// The invocation itself failed: the decoded return value is an error
    /// value.
    FailedInvocation,
}

impl fmt::Display for TraceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TraceError::EmptyInput => write!(f, "cannot decode an empty XDR buffer"),
            TraceError::InvalidXdr(err) => write!(f, "invalid or truncated XDR: {err}"),
            TraceError::MissingInvocationRecord => write!(
                f,
                "no host-level `fn_call` diagnostic event found; \
                 cannot reconstruct the invocation trace"
            ),
            TraceError::MalformedControlEvent(reason) => {
                write!(f, "malformed control event: {reason}")
            }
            TraceError::FailedInvocation => {
                write!(f, "the invocation failed: the return value is an error value")
            }
        }
    }
}

impl std::error::Error for TraceError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            TraceError::InvalidXdr(err) => Some(err),
            _ => None,
        }
    }
}