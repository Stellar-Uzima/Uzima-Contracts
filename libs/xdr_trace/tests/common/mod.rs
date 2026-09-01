//! Shared test support: builders for the canonical trace fixture.
#![allow(dead_code)]

use soroban_sdk::xdr::{
    ContractEvent, ContractEventBody, ContractEventType, ContractEventV0, DiagnosticEvent,
    ExtensionPoint, Hash, ScBytes, ScSymbol, ScVal, ScVec, SorobanTransactionMeta,
    SorobanTransactionMetaExt, VecM,
};

use xdr_trace::{ContractId, ContractTrace};

/// Contract A: the contract invoked at the trace root, `0xaa`.
pub const CONTRACT_A: [u8; 32] = id(0xaa);
/// Contract B: a contract invoked transitively by `CONTRACT_A`, `0xbb`.
pub const CONTRACT_B: [u8; 32] = id(0xbb);

/// Builds a 32-byte contract id whose first byte is `c` (rest zeroed).
pub const fn id(c: u8) -> [u8; 32] {
    let mut bytes = [0u8; 32];
    bytes[0] = c;
    bytes
}

/// A symbol `ScVal` (the canonical encoding for function/event names).
pub fn sym(s: &str) -> ScVal {
    ScVal::Symbol(ScSymbol(s.try_into().unwrap()))
}

/// A `u32` `ScVal`.
pub fn u32_val(v: u32) -> ScVal {
    ScVal::U32(v)
}

/// A 32-byte-buffer `ScVal` (the canonical encoding for a contract id).
pub fn bytes32(bytes: [u8; 32]) -> ScVal {
    ScVal::Bytes(ScBytes(bytes.as_slice().try_into().unwrap()))
}

/// Builds a `VecM` from a `Vec`.
pub fn vecm<T>(items: Vec<T>) -> VecM<T> {
    items.try_into().unwrap()
}

/// Builds the host-level `fn_call` diagnostic event the host emits around a
/// contract call. `calling` is the calling contract — `None` for the
/// transaction-root invocation.
pub fn fn_call_diagnostic(
    calling: Option<[u8; 32]>,
    called: [u8; 32],
    function: &str,
    args: Vec<ScVal>,
) -> DiagnosticEvent {
    DiagnosticEvent {
        in_successful_contract_call: true,
        event: ContractEvent {
            ext: ExtensionPoint::V0,
            contract_id: calling.map(Hash),
            type_: ContractEventType::Diagnostic,
            body: ContractEventBody::V0(ContractEventV0 {
                topics: vecm(vec![sym("fn_call"), bytes32(called), sym(function)]),
                data: ScVal::Vec(Some(ScVec(vecm(args)))),
            }),
        },
    }
}

/// Builds the host-level `fn_return` diagnostic event the host emits when a
/// contract function returns.
pub fn fn_return_diagnostic(contract: [u8; 32], function: &str, ret: ScVal) -> DiagnosticEvent {
    DiagnosticEvent {
        in_successful_contract_call: true,
        event: ContractEvent {
            ext: ExtensionPoint::V0,
            contract_id: Some(Hash(contract)),
            type_: ContractEventType::Diagnostic,
            body: ContractEventBody::V0(ContractEventV0 {
                topics: vecm(vec![sym("fn_return"), sym(function)]),
                data: ScVal::Vec(Some(ScVec(vecm(vec![ret])))),
            }),
        },
    }
}

/// The canonical golden fixture: a `SorobanTransactionMeta` encoding a
/// `burn(7u32) -> 42u32` invocation on `CONTRACT_A`, which itself calls
/// `recv(9u32) -> 99u32` on `CONTRACT_B`, and which emits one `transfer`
/// contract event. Its XDR serialization is committed under
/// `tests/fixtures/simple_invoke_trace.sorobanmeta.xdr.hex`.
pub fn build_fixture_meta() -> SorobanTransactionMeta {
    let contract_a = CONTRACT_A;
    let contract_b = CONTRACT_B;

    let transfer_event = ContractEvent {
        ext: ExtensionPoint::V0,
        contract_id: Some(Hash(contract_a)),
        type_: ContractEventType::Contract,
        body: ContractEventBody::V0(ContractEventV0 {
            topics: vecm(vec![sym("transfer"), sym("alice"), sym("bob")]),
            data: u32_val(7),
        }),
    };

    let diagnostics = vec![
        fn_call_diagnostic(None, contract_a, "burn", vec![u32_val(7)]),
        fn_call_diagnostic(Some(contract_a), contract_b, "recv", vec![u32_val(9)]),
        fn_return_diagnostic(contract_b, "recv", u32_val(99)),
        fn_return_diagnostic(contract_a, "burn", u32_val(42)),
    ];

    SorobanTransactionMeta {
        ext: SorobanTransactionMetaExt::V0,
        events: vecm(vec![transfer_event]),
        return_value: u32_val(42),
        diagnostic_events: vecm(diagnostics),
    }
}

/// The `ContractTrace` the golden fixture must decode to.
pub fn expected_trace() -> ContractTrace {
    ContractTrace {
        contract_id: ContractId(CONTRACT_A),
        function_name: "burn".to_string(),
        arguments: vec![u32_val(7)],
        return_value: u32_val(42),
        events: vec![ContractEvent {
            ext: ExtensionPoint::V0,
            contract_id: Some(Hash(CONTRACT_A)),
            type_: ContractEventType::Contract,
            body: ContractEventBody::V0(ContractEventV0 {
                topics: vecm(vec![sym("transfer"), sym("alice"), sym("bob")]),
                data: u32_val(7),
            }),
        }],
        diagnostic_events: vec![
            fn_call_diagnostic(None, CONTRACT_A, "burn", vec![u32_val(7)]),
            fn_call_diagnostic(Some(CONTRACT_A), CONTRACT_B, "recv", vec![u32_val(9)]),
            fn_return_diagnostic(CONTRACT_B, "recv", u32_val(99)),
            fn_return_diagnostic(CONTRACT_A, "burn", u32_val(42)),
        ],
        invoked_contract_ids: vec![ContractId(CONTRACT_A), ContractId(CONTRACT_B)],
    }
}

/// Decodes a lowercase hex string into bytes.
pub fn hex_to_bytes(hex: &str) -> Vec<u8> {
    let hex = hex.trim();
    assert_eq!(hex.len() % 2, 0, "hex must have an even length");
    (0..hex.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&hex[i..i + 2], 16).expect("hex digit"))
        .collect()
}
