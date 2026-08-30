//! Trace-based invariant checks for the shared `BehaviorHarness` contract
//! (issue #1512).
//!
//! Exercises, with a fully synthetic (std-only) harness plus a real XDR
//! round-trip:
//!
//! 1. per-operation ordered topic/payload event expectations enforced on the
//!    shared trait (a wrong-topic or wrong-length sequence is detected even
//!    when the event *count* is right);
//! 2. the built-in `FN_INVOKE`/`FN_DONE` pairing invariant (unclosed
//!    invocations and unpaired completions are violations);
//! 3. the recorded `SequenceReport` trace replayed and diffed against the
//!    recorded expectation;
//! 4. XDR encode/decode/re-encode byte-stability for payload round-tripping.

use std::vec::Vec;

use contract_behavior_fuzzing::{
    check_telemetry_pairing, execute_sequence, BehaviorHarness, ExpectedTrace,
    ExpectedTraceEvent, OperationOutcome, SequenceReport, TraceEvent, TraceViolation,
    FN_DONE_TOPIC, FN_INVOKE_TOPIC,
};
use soroban_sdk::{xdr::{FromXdr, ToXdr}, Env};

#[derive(Clone, Debug)]
enum DemoOp {
    Emit { topic: &'static str },
    EmitPair { topic: &'static str },
    EmitInvoke,
    EmitDone,
}

struct DemoHarness {
    trace: Vec<TraceEvent>,
    expected_topic: Option<&'static str>,
}

impl DemoHarness {
    fn track(topic: &'static str) -> Self {
        Self {
            trace: Vec::new(),
            expected_topic: Some(topic),
        }
    }

    fn untracked() -> Self {
        Self {
            trace: Vec::new(),
            expected_topic: None,
        }
    }

    fn push(&mut self, topic: &'static str) -> OperationOutcome {
        self.trace.push(TraceEvent::new(
            vec![topic.as_bytes().to_vec()],
            topic.as_bytes().to_vec(),
        ));
        let mut outcome = OperationOutcome::new(1);
        if let Some(expected) = self.expected_topic {
            outcome.expected_trace =
                ExpectedTrace::new(vec![ExpectedTraceEvent::new_topic(expected)]);
        }
        outcome
    }

    fn event(&self, topic: &str) -> TraceEvent {
        TraceEvent::new(vec![topic.as_bytes().to_vec()], Vec::new())
    }
}

impl BehaviorHarness for DemoHarness {
    type Operation = DemoOp;

    fn apply(&mut self, operation: &Self::Operation) -> OperationOutcome {
        match operation {
            DemoOp::Emit { topic } => self.push(topic),
            DemoOp::EmitPair { topic } => {
                // Simulates a contract emitting two events while the harness
                // declares a one-event expectation (right count, wrong length).
                self.trace.push(self.event(topic));
                self.trace.push(self.event(topic));
                let mut outcome = OperationOutcome::new(2);
                if let Some(expected) = self.expected_topic {
                    outcome.expected_trace =
                        ExpectedTrace::new(vec![ExpectedTraceEvent::new_topic(expected)]);
                }
                outcome
            }
            DemoOp::EmitInvoke => self.push(FN_INVOKE_TOPIC),
            DemoOp::EmitDone => self.push(FN_DONE_TOPIC),
        }
    }

    fn assert_invariants(&self) {}

    fn event_count(&self) -> usize {
        self.trace.len()
    }

    fn captured_trace(&self) -> Vec<TraceEvent> {
        self.trace.clone()
    }
}

#[test]
fn matching_trace_passes_and_report_replays_cleanly() {
    let mut harness = DemoHarness::track("emit_a");
    let report = execute_sequence(&mut harness, &[DemoOp::Emit { topic: "emit_a" }])
        .expect("a trace matching its expectation must pass");
    assert_eq!(report.trace.len(), 1);
    assert_eq!(report.expected_trace.len(), 1);
    assert_eq!(report.final_event_count, 1);
    assert!(report.diff().first_divergence.is_none());
    assert!(report.diff().telemetry.is_ok());
    report.assert_replays_cleanly();
}

#[test]
fn wrong_topic_with_right_event_count_is_detected() {
    let mut harness = DemoHarness::track("emit_a");
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        execute_sequence(&mut harness, &[DemoOp::Emit { topic: "emit_b" }])
    }));
    assert!(
        result.is_err(),
        "a sequence that emits the right number of events but the wrong topic \
         must be detected as a failure"
    );
}

#[test]
fn wrong_event_count_with_matching_topics_is_detected() {
    let mut harness = DemoHarness::track("emit_a");
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        execute_sequence(&mut harness, &[DemoOp::EmitPair { topic: "emit_a" }])
    }));
    assert!(
        result.is_err(),
        "a sequence emitting more events than declared must fail the harness"
    );
}

#[test]
fn unclosed_fn_invoke_at_sequence_end_is_a_violation() {
    let trace = vec![TraceEvent::new(vec![FN_INVOKE_TOPIC.as_bytes().to_vec()], Vec::new())];
    assert_eq!(
        check_telemetry_pairing(&trace),
        Err(TraceViolation::UnclosedInvocation {
            depth: 1,
            at_index: 0
        })
    );
}

#[test]
fn unpaired_completion_is_a_violation() {
    let trace = vec![TraceEvent::new(vec![FN_DONE_TOPIC.as_bytes().to_vec()], Vec::new())];
    assert_eq!(
        check_telemetry_pairing(&trace),
        Err(TraceViolation::UnpairedCompletion { at_index: 0 })
    );
}

#[test]
fn matched_pair_closes_and_sequence_ends_clean() {
    let trace = vec![
        TraceEvent::new(vec![FN_INVOKE_TOPIC.as_bytes().to_vec()], Vec::new()),
        TraceEvent::new(vec![FN_DONE_TOPIC.as_bytes().to_vec()], Vec::new()),
    ];
    assert!(check_telemetry_pairing(&trace).is_ok());
}

#[test]
fn telemetry_pairing_also_recognizes_schema_topic_layout() {
    // docs/TELEMETRY_SCHEMA.md emits (TEL, <type_symbol>) topics, so the
    // invocation/completion markers live at topics[1].
    let trace = vec![
        TraceEvent::new(
            vec![b"TEL".to_vec(), FN_INVOKE_TOPIC.as_bytes().to_vec()],
            Vec::new(),
        ),
        TraceEvent::new(
            vec![b"TEL".to_vec(), FN_DONE_TOPIC.as_bytes().to_vec()],
            Vec::new(),
        ),
    ];
    assert!(check_telemetry_pairing(&trace).is_ok());
}

#[test]
fn execute_sequence_rejects_a_sequence_ending_open() {
    let mut harness = DemoHarness::untracked();
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        execute_sequence(&mut harness, &[DemoOp::EmitInvoke])
    }));
    assert!(
        result.is_err(),
        "any sequence ending with an unclosed FN_INVOKE must be reported as a violation"
    );
}

#[test]
fn execute_sequence_accepts_an_operation_paired_within_itself() {
    let mut harness = DemoHarness::untracked();
    let report = execute_sequence(
        &mut harness,
        &[DemoOp::EmitInvoke, DemoOp::EmitDone],
    )
    .expect("a paired invocation must pass the pairing invariant");
    assert_eq!(report.final_event_count, 2);
    report.assert_replays_cleanly();
}

#[test]
fn reported_trace_with_unclosed_invocation_is_flagged_on_replay() {
    let report = SequenceReport {
        operations: vec![DemoOp::EmitInvoke],
        final_event_count: 1,
        trace: vec![TraceEvent::new(
            vec![FN_INVOKE_TOPIC.as_bytes().to_vec()],
            Vec::new(),
        )],
        expected_trace: vec![ExpectedTraceEvent::new_topic(FN_INVOKE_TOPIC)],
    };
    let diff = report.diff();
    assert!(diff.first_divergence.is_none());
    assert_eq!(
        diff.telemetry,
        Err(TraceViolation::UnclosedInvocation {
            depth: 1,
            at_index: 0
        })
    );
}

#[test]
fn xdr_round_trip_is_byte_stable_for_option_payloads() {
    let env = Env::default();
    for value in [Some(7i128), None::<i128>] {
        let first = value.to_xdr(&env).to_alloc_vec();
        for _ in 0..16 {
            let decoded: Option<i128> = Option::from_xdr(&env, &first).expect("decode");
            assert_eq!(decoded, value);
            assert_eq!(
                decoded.to_xdr(&env).to_alloc_vec(),
                first,
                "a decoded/re-encoded payload must be byte-stable across the sequence"
            );
        }
    }
}