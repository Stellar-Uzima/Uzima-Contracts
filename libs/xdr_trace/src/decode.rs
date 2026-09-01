//! The XDR decoding layer.
//!
//! Wraps the `soroban-sdk` `xdr` types to turn raw result XDR bytes into the
//! typed [`ContractTrace`] model. All decoding goes through
//! `soroban_sdk::xdr::{ReadXdr, WriteXdr}`, honouring `docs/SERIALIZATION_
//! STANDARDS.md` ("XDR is the wire format — all encoding/decoding must go
//! through `soroban_sdk`").

use soroban_sdk::xdr::{
    ContractEvent, ContractEventBody, DiagnosticEvent, DiagnosticEvents, Limits, ReadXdr, ScVal,
    SorobanTransactionMeta, VecM,
};

use crate::error::TraceError;
use crate::model::{ContractId, ContractTrace};

/// Decodes a full invocation trace from a [`SorobanTransactionMeta`] XDR
/// buffer.
///
/// The buffer is the `sorobanMeta` payload carried by the transaction meta a
/// Soroban RPC / `soroban contract invoke` run returns. It holds the ordered
/// contract events, the typed return value, and the diagnostic events; the
/// host-level `fn_call`/`fn_return` diagnostics embedded in it let the decoder
/// reconstruct the invoked contract, the function symbol, the arguments and the
/// transitively invoked contracts.
///
/// # Errors
///
/// Returns [`TraceError::EmptyInput`] for an empty buffer,
/// [`TraceError::InvalidXdr`] when the bytes are not a valid, complete
/// `SorobanTransactionMeta` (truncated, trailing garbage, ...),
/// [`TraceError::FailedInvocation`] when the invocation itself errored,
/// [`TraceError::MissingInvocationRecord`] when no host-level `fn_call`
/// diagnostic exists, and [`TraceError::MalformedControlEvent`] when a control
/// event is not well-formed. This function never panics on hostile input.
pub fn decode_trace(bytes: &[u8]) -> Result<ContractTrace, TraceError> {
    if bytes.is_empty() {
        return Err(TraceError::EmptyInput);
    }

    if let Ok(meta) = SorobanTransactionMeta::from_xdr(bytes, Limits::none()) {
        return trace_from_meta(&meta);
    }

    if let Ok(soroban_sdk::xdr::TransactionMeta::V3(v3)) =
        soroban_sdk::xdr::TransactionMeta::from_xdr(bytes, Limits::none())
    {
        if let Some(soroban_meta) = v3.soroban_meta {
            return trace_from_meta(&soroban_meta);
        }
    }

    if let Ok(tx_res_meta) =
        soroban_sdk::xdr::TransactionResultMeta::from_xdr(bytes, Limits::none())
    {
        if let soroban_sdk::xdr::TransactionMeta::V3(v3) = tx_res_meta.tx_apply_processing {
            if let Some(soroban_meta) = v3.soroban_meta {
                return trace_from_meta(&soroban_meta);
            }
        }
    }

    let meta =
        SorobanTransactionMeta::from_xdr(bytes, Limits::none()).map_err(TraceError::InvalidXdr)?;

    trace_from_meta(&meta)
}

/// Decodes a `DiagnosticEvents` XDR buffer (a `VecM<DiagnosticEvent>`) into an
/// ordered vector of [`DiagnosticEvent`]s.
pub fn decode_diagnostic_events(bytes: &[u8]) -> Result<Vec<DiagnosticEvent>, TraceError> {
    if bytes.is_empty() {
        return Err(TraceError::EmptyInput);
    }

    let events =
        DiagnosticEvents::from_xdr(bytes, Limits::none()).map_err(TraceError::InvalidXdr)?;
    Ok(events.0.as_vec().clone())
}

/// Decodes a `VecM<ContractEvent>` XDR buffer into an ordered vector of
/// [`ContractEvent`]s.
pub fn decode_contract_events(bytes: &[u8]) -> Result<Vec<ContractEvent>, TraceError> {
    if bytes.is_empty() {
        return Err(TraceError::EmptyInput);
    }

    let events =
        VecM::<ContractEvent>::from_xdr(bytes, Limits::none()).map_err(TraceError::InvalidXdr)?;
    Ok(events.as_vec().clone())
}

/// A host-level control event recorded around a contract call.
enum ControlEvent<'a> {
    /// The host is about to call `function` on `called_contract`; `data` is
    /// the argument vector.
    FnCall {
        called_contract: &'a [u8],
        function: &'a [u8],
        data: &'a ScVal,
    },
    /// The host recorded the return of `function`; `data` is the result vector.
    #[allow(dead_code)]
    FnReturn { function: &'a [u8], data: &'a ScVal },
}

/// Extracts a control event from a diagnostic event, if its topic names one.
///
/// Events with any other tag (e.g. `log`) are not control events and yield
/// `Ok(None)`, so they don't break tracing. An event *named* `fn_call` or
/// `fn_return` but with a malformed topic shape yields `Ok(None)` is not an
/// option — the shape is part of the trace contract, so callers get a typed
/// [`TraceError::MalformedControlEvent`] instead.
fn control_event(diagnostic: &DiagnosticEvent) -> Result<Option<ControlEvent<'_>>, TraceError> {
    let ContractEventBody::V0(v0) = &diagnostic.event.body;
    let topics = v0.topics.as_vec();

    let Some(ScVal::Symbol(tag)) = topics.first() else {
        return Ok(None);
    };
    match &tag.0[..] {
        b"fn_call" => {
            let called_contract = match topics.get(1) {
                Some(ScVal::Bytes(called)) => &called.0[..],
                _ => {
                    return Err(TraceError::MalformedControlEvent(
                        "`fn_call` topic[1] must be the called contract id (ScBytes)",
                    ))
                },
            };
            let function = match topics.get(2) {
                Some(ScVal::Symbol(function)) => &function.0[..],
                _ => {
                    return Err(TraceError::MalformedControlEvent(
                        "`fn_call` topic[2] must be the function symbol",
                    ))
                },
            };
            Ok(Some(ControlEvent::FnCall {
                called_contract,
                function,
                data: &v0.data,
            }))
        },
        b"fn_return" => {
            let function = match topics.get(1) {
                Some(ScVal::Symbol(function)) => &function.0[..],
                _ => {
                    return Err(TraceError::MalformedControlEvent(
                        "`fn_return` topic[1] must be the function symbol",
                    ))
                },
            };
            Ok(Some(ControlEvent::FnReturn {
                function,
                data: &v0.data,
            }))
        },
        _ => Ok(None),
    }
}

/// Reconstructs a [`ContractTrace`] from a decoded `SorobanTransactionMeta`.
fn trace_from_meta(meta: &SorobanTransactionMeta) -> Result<ContractTrace, TraceError> {
    // A failed invocation surfaces as a typed error, never as a `ContractTrace`
    // (the model records the fields of a successful invocation only).
    if let ScVal::Error(_) = &meta.return_value {
        return Err(TraceError::FailedInvocation);
    }

    let diagnostic_events = meta.diagnostic_events.as_vec().clone();

    // The outermost `fn_call` is the one with no calling contract: it carries
    // the invocation identity (contract id, function symbol, arguments).
    let mut root_invocation: Option<(ContractId, String, Vec<ScVal>)> = None;
    let mut invoked_contract_ids: Vec<ContractId> = Vec::new();

    for diagnostic in &diagnostic_events {
        let Some(control) = control_event(diagnostic)? else {
            continue;
        };
        match control {
            ControlEvent::FnCall {
                called_contract,
                function,
                data,
            } => {
                let contract_id = ContractId::try_from(called_contract)?;
                if !invoked_contract_ids.contains(&contract_id) {
                    invoked_contract_ids.push(contract_id);
                }

                if diagnostic.event.contract_id.is_none() && root_invocation.is_none() {
                    let function_name = String::from_utf8(function.to_vec()).map_err(|_| {
                        TraceError::MalformedControlEvent("`fn_call` function symbol is not UTF-8")
                    })?;
                    let arguments = match data {
                        ScVal::Vec(Some(arguments)) => arguments.0.as_vec().clone(),
                        ScVal::Vec(None) => Vec::new(),
                        _ => {
                            return Err(TraceError::MalformedControlEvent(
                                "`fn_call` data must be an ScVal vector of arguments",
                            ))
                        },
                    };
                    root_invocation = Some((contract_id, function_name, arguments));
                }
            },
            ControlEvent::FnReturn { .. } => {},
        }
    }

    let (contract_id, function_name, arguments) =
        root_invocation.ok_or(TraceError::MissingInvocationRecord)?;

    invoked_contract_ids.sort_unstable();
    invoked_contract_ids.dedup();

    Ok(ContractTrace {
        contract_id,
        function_name,
        arguments,
        return_value: meta.return_value.clone(),
        events: meta.events.as_vec().clone(),
        diagnostic_events,
        invoked_contract_ids,
    })
}
