//! Golden-fixture and error-path tests for `xdr_trace`.

mod common;

use soroban_sdk::xdr::{Limits, WriteXdr};

use xdr_trace::{decode_diagnostic_events, decode_trace, TraceError};

/// The round-tripped canonical fixture decodes to the expected `ContractTrace`.
#[test]
fn golden_round_trip_decodes_to_expected_trace() {
    let meta = common::build_fixture_meta();
    let bytes = meta.to_xdr(Limits::none()).expect("xdr encode");

    let trace = decode_trace(&bytes).expect("decode trace");
    assert_eq!(trace, common::expected_trace());
}

/// The committed golden XDR fixture decodes to the expected `ContractTrace`.
///
/// `tests/fixtures/simple_invoke_trace.sorobanmeta.xdr.hex` is a fixed capture
/// of the canonical fixture's `SorobanTransactionMeta` XDR, so the decoder has
/// a non-regressing baseline independent of any in-memory generation path.
#[test]
fn committed_golden_fixture_decodes() {
    let hex = include_str!("fixtures/simple_invoke_trace.sorobanmeta.xdr.hex");
    let bytes = common::hex_to_bytes(hex);

    let trace = decode_trace(&bytes).expect("decode committed fixture");
    assert_eq!(trace, common::expected_trace());

    // Sanity: the committed fixture is byte-identical to a fresh serialization
    // of the canonical meta, i.e. the file and generator cannot drift apart.
    let bytes2 = common::build_fixture_meta().to_xdr(Limits::none()).expect("xdr encode");
    assert_eq!(bytes, bytes2);
}

/// Empty, truncated and trailing-garbage input surfaces as a typed error,
/// never a panic and never a silently truncated trace.
#[test]
fn malformed_input_errors_typed() {
    assert_eq!(decode_trace(&[]), Err(TraceError::EmptyInput));
    assert_eq!(decode_diagnostic_events(&[]), Err(TraceError::EmptyInput));

    let bytes = common::build_fixture_meta().to_xdr(Limits::none()).expect("xdr encode");

    // Truncations at several boundaries must fail with a typed XDR error.
    for n in [1, 3, 4, 7, 8, 16, 63, 64, 128, bytes.len() - 1] {
        assert!(
            matches!(decode_trace(&bytes[..n]), Err(TraceError::InvalidXdr(_))),
            "expected InvalidXdr for truncation at {n} of {}",
            bytes.len()
        );
    }

    // Trailing garbage makes the buffer a valid prefix but not a complete type.
    let mut trailing = bytes.clone();
    trailing.push(0);
    assert!(matches!(decode_trace(&trailing), Err(TraceError::InvalidXdr(_))));

    // The untouched buffer still decodes.
    assert!(matches!(decode_trace(&bytes), Ok(_)));
}

/// An error-valued return decodes as a typed `FailedInvocation`, not a trace.
#[test]
fn failed_invocation_errors_typed() {
    let meta = common::build_fixture_meta();
    let diagnostic_events = meta.diagnostic_events.as_vec().clone();
    let failed = soroban_sdk::xdr::SorobanTransactionMeta {
        ext: soroban_sdk::xdr::SorobanTransactionMetaExt::V0,
        events: soroban_sdk::xdr::VecM::default(),
        return_value: soroban_sdk::xdr::ScVal::Error(soroban_sdk::xdr::ScError::Context(
            soroban_sdk::xdr::ScErrorCode::InternalError,
        )),
        diagnostic_events: diagnostic_events.try_into().unwrap(),
    };

    let bytes = failed.to_xdr(Limits::none()).expect("xdr encode");
    assert!(matches!(decode_trace(&bytes), Err(TraceError::FailedInvocation)));
}

/// A structurally valid meta without a host-level `fn_call` diagnostic yields a
/// typed `MissingInvocationRecord`, not a trace and not a panic.
#[test]
fn missing_invocation_record_errors_typed() {
    let meta = common::build_fixture_meta();
    let meta = soroban_sdk::xdr::SorobanTransactionMeta {
        ext: meta.ext,
        events: meta.events,
        return_value: meta.return_value,
        diagnostic_events: soroban_sdk::xdr::VecM::default(),
    };

    let bytes = meta.to_xdr(Limits::none()).expect("xdr encode");
    assert!(matches!(decode_trace(&bytes), Err(TraceError::MissingInvocationRecord)));
}

/// `decode_diagnostic_events` decodes a raw `DiagnosticEvents` blob.
#[test]
fn diagnostic_events_decode_standalone() {
    let meta = common::build_fixture_meta();
    let diagnostics = meta.diagnostic_events.as_vec().clone();
    let blob = soroban_sdk::xdr::DiagnosticEvents(diagnostics.clone().try_into().unwrap())
        .to_xdr(Limits::none())
        .expect("xdr encode");

    let decoded = decode_diagnostic_events(&blob).expect("decode diagnostics");
    assert_eq!(decoded, diagnostics);
}