//! Property/fuzz-style tests: the decoder never panics on hostile input and
//! never scrambles event order.
//!
//! These are deterministic in-process fuzz loops (xorshift64*), so they are
//! reproducible and need no external fuzzing infrastructure.

mod common;

use soroban_sdk::xdr::{
    ContractEvent, ContractEventBody, ContractEventType, ContractEventV0, ExtensionPoint, Hash,
    Limits, SorobanTransactionMeta, WriteXdr,
};

use xdr_trace::{decode_contract_events, decode_trace, ContractId};

/// A tiny deterministic xorshift64* PRNG.
struct XorShift(u64);

impl XorShift {
    fn next(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.0 = x;
        x
    }

    fn fill(&mut self, bytes: &mut [u8]) {
        let mut written = 0;
        while written < bytes.len() {
            let word = self.next().to_le_bytes();
            let n = word.len().min(bytes.len() - written);
            bytes[written..written + n].copy_from_slice(&word[..n]);
            written += n;
        }
    }
}

/// Random and arbitrarily-truncated buffers (including empty ones) complete
/// without panicking and produce only typed results.
#[test]
fn random_and_truncated_buffers_never_panic() {
    let mut rng = XorShift(0x9E37_79B9_7F4A_7C15);
    let mut buffer = vec![0u8; 4096];

    for _ in 0..100_000 {
        rng.fill(&mut buffer);
        let len = (rng.next() as usize) % (buffer.len() + 1);
        let _ = decode_trace(&buffer[..len]);
        let _ = decode_contract_events(&buffer[..len]);
    }
}

/// Every prefix of the valid encoded fixture and every single-bit corruption
/// of it completes without panicking.
#[test]
fn valid_fixture_truncated_and_bitflipped_never_panics() {
    let bytes = common::build_fixture_meta()
        .to_xdr(Limits::none())
        .expect("xdr encode");

    for n in 0..=bytes.len() {
        let _ = decode_trace(&bytes[..n]);
    }

    for (i, _byte) in bytes.iter().enumerate() {
        for bit in 0..u8::BITS {
            let mut corrupted = bytes.clone();
            corrupted[i] ^= 1 << bit;
            let _ = decode_trace(&corrupted);
        }
    }
}

/// Decoding preserves the serialization order of both contract events and
/// diagnostic events: no order scrambling, ever.
#[test]
fn decoded_event_order_preserves_serialization_order() {
    let mut rng = XorShift(0x42_73_4D_E4);
    let meta_template = common::build_fixture_meta();

    for _ in 0..500 {
        // Build a meta with several contract events, then scramble their order
        // deterministically (Fisher-Yates over the PRNG).
        let mut events: Vec<ContractEvent> = (0..8)
            .map(|k| ContractEvent {
                ext: ExtensionPoint::V0,
                contract_id: Some(Hash(common::CONTRACT_A)),
                type_: ContractEventType::Contract,
                body: ContractEventBody::V0(ContractEventV0 {
                    topics: common::vecm(vec![common::sym("mix"), common::sym(&format!("e{k}"))]),
                    data: common::u32_val((rng.next() as u32) % 1000),
                }),
            })
            .collect();
        for i in (1..events.len()).rev() {
            let j = (rng.next() as usize) % (i + 1);
            events.swap(i, j);
        }
        let encoded_events = events.clone();

        let meta = SorobanTransactionMeta {
            ext: meta_template.ext.clone(),
            events: encoded_events.clone().try_into().unwrap(),
            return_value: meta_template.return_value.clone(),
            diagnostic_events: meta_template.diagnostic_events.clone(),
        };
        let bytes = meta.to_xdr(Limits::none()).expect("xdr encode");
        let trace = decode_trace(&bytes).expect("decode");

        assert_eq!(
            trace.events, encoded_events,
            "contract event order was scrambled"
        );
        assert_eq!(
            trace.diagnostic_events,
            meta_template.diagnostic_events.as_vec().clone(),
            "diagnostic event order was scrambled"
        );
    }
}

/// A trace never contains phantom invoked contracts: the reported set is
/// exactly the distinct `fn_call` targets encoded in the XDR.
#[test]
fn invoked_contract_set_matches_encoded_fn_calls() {
    let bytes = common::build_fixture_meta()
        .to_xdr(Limits::none())
        .expect("xdr encode");
    let trace = decode_trace(&bytes).expect("decode");

    let mut expected = vec![
        ContractId(common::CONTRACT_A),
        ContractId(common::CONTRACT_B),
    ];
    expected.sort_unstable();
    assert_eq!(trace.invoked_contract_ids, expected);
}
