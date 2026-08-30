//! contract_behavior_fuzzing - Healthcare smart contract on Stellar blockchain.
// no_std-exempt: this crate is a native test/fuzz harness, not a WASM contract.
// It uses std::panic and std::vec intentionally and does not target wasm32.
use core::fmt::Debug;
use std::panic::{self, AssertUnwindSafe};
use std::vec::Vec;

// =============================================================================
// Trace-based invariants (issue #1512)
//
// The harness API below is additive: `OperationOutcome::expected_trace`,
// `BehaviorHarness::captured_trace` and `BehaviorHarness::assert_round_trip`
// default to no-ops/empty so existing harnesses keep compiling and passing
// unchanged, while harnesses that opt in get ordered, typed event assertions
// and the built-in telemetry pairing invariant.
// =============================================================================

/// The topic symbols that open and close a telemetry invocation per
/// `docs/TELEMETRY_SCHEMA.md` (`FN_INVOKE` / `FN_DONE`, with failures carried
/// as the FN_DONE error variant).
pub const FN_INVOKE_TOPIC: &str = "FN_INVOKE";
pub const FN_DONE_TOPIC: &str = "FN_DONE";

/// A single event observed on the ledger during a fuzz sequence.
///
/// `topics` holds the UTF-8 bytes of each topic symbol (e.g. `b"FN_INVOKE"`,
/// `b"did_created"`, `b"transfer"`), while `payload` holds the canonical
/// serialized (XDR) bytes of the event data. Keeping both canonical makes the
/// captured stream re-playable and byte-comparable across a whole sequence.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct TraceEvent {
    pub topics: Vec<Vec<u8>>,
    pub payload: Vec<u8>,
}

impl TraceEvent {
    pub fn new(topics: Vec<Vec<u8>>, payload: Vec<u8>) -> Self {
        Self { topics, payload }
    }

    pub fn has_topic(&self, name: &str) -> bool {
        let name = name.as_bytes();
        self.topics.iter().any(|topic| topic == name)
    }
}

/// A declared expectation for a single event of an operation.
///
/// `payload` is checked only when `Some`; `None` matches any payload, which
/// lets harnesses assert on ordered topics (and optionally key payload fields)
/// without hard-coding every data field.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExpectedTraceEvent {
    pub topics: Vec<Vec<u8>>,
    pub payload: Option<Vec<u8>>,
}

impl ExpectedTraceEvent {
    pub fn new(topics: Vec<Vec<u8>>, payload: Option<Vec<u8>>) -> Self {
        Self { topics, payload }
    }

    /// Expectations declared with `new_topic` check only the ordered topic
    /// symbols (e.g. `ExpectedTraceEvent::new_topic("did_created")`).
    pub fn new_topic(topic: &str) -> Self {
        Self {
            topics: vec![topic.as_bytes().to_vec()],
            payload: None,
        }
    }

    pub fn matches(&self, observed: &TraceEvent) -> bool {
        self.topics == observed.topics
            && self
                .payload
                .as_ref()
                .map_or(true, |expected| expected == &observed.payload)
    }
}

/// The ordered expected-event sequence an operation declares it will emit.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExpectedTrace {
    pub events: Vec<ExpectedTraceEvent>,
}

impl ExpectedTrace {
    pub const fn empty() -> Self {
        Self { events: Vec::new() }
    }

    pub fn new(events: Vec<ExpectedTraceEvent>) -> Self {
        Self { events }
    }

    pub fn is_empty(&self) -> bool {
        self.events.is_empty()
    }
}

impl Default for ExpectedTrace {
    fn default() -> Self {
        Self::empty()
    }
}

/// Outcome of applying one operation to a harness.
///
/// `expected_trace` is the ordered event sequence the operation declares it
/// will emit. It is asserted by `execute_sequence` against the events actually
/// emitted during the operation; an empty trace means "events are untracked"
/// (topic/payload assertions are skipped, only the count is still checked).
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OperationOutcome {
    pub expected_event_delta: usize,
    pub expected_trace: ExpectedTrace,
}

impl OperationOutcome {
    pub const fn new(expected_event_delta: usize) -> Self {
        Self {
            expected_event_delta,
            expected_trace: ExpectedTrace::empty(),
        }
    }

    pub fn with_expected_trace(mut self, expected_trace: ExpectedTrace) -> Self {
        self.expected_trace = expected_trace;
        self
    }
}

pub trait BehaviorHarness {
    type Operation: Clone + Debug;

    fn apply(&mut self, operation: &Self::Operation) -> OperationOutcome;
    fn assert_invariants(&self);
    fn event_count(&self) -> usize;

    /// Ordered event stream captured so far (events emitted by the contracts
    /// under fuzz, oldest first). Harnesses that opt into per-operation trace
    /// assertions or the built-in telemetry pairing invariant implement this.
    /// Default: no events are tracked.
    fn captured_trace(&self) -> Vec<TraceEvent> {
        Vec::new()
    }

    /// For XDR round-tripping harnesses: verify that the serialized payload
    /// representation is byte-stable across the sequence (issue #1512 scope 4).
    /// Default: no-op.
    fn assert_round_trip(&self) {}
}

/// A telemetry pairing violation found by `check_telemetry_pairing`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TraceViolation {
    /// An `FN_INVOKE` was never closed by a matching `FN_DONE`.
    UnclosedInvocation { depth: usize, at_index: usize },
    /// An `FN_DONE` appeared with no open invocation to close.
    UnpairedCompletion { at_index: usize },
}

/// Built-in trace integrity invariant (issue #1512 scope 2): every `FN_INVOKE`
/// event must eventually be matched by an `FN_DONE` (success) or its failure
/// variant, and a sequence must never end with an unclosed invocation. This is
/// the same pairing rule `docs/TELEMETRY_SCHEMA.md` makes the basis of trace
/// reconstruction, enforced in-process so a fuzz pass is a real guarantee
/// about trace integrity rather than only a count.
pub fn check_telemetry_pairing(trace: &[TraceEvent]) -> Result<(), TraceViolation> {
    let mut depth: usize = 0;
    for (index, event) in trace.iter().enumerate() {
        if event.has_topic(FN_INVOKE_TOPIC) {
            depth += 1;
        }
        if event.has_topic(FN_DONE_TOPIC) {
            if depth == 0 {
                return Err(TraceViolation::UnpairedCompletion { at_index: index });
            }
            depth -= 1;
        }
    }
    if depth != 0 {
        return Err(TraceViolation::UnclosedInvocation {
            depth,
            at_index: trace.len().saturating_sub(1),
        });
    }
    Ok(())
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SequenceReport<Op> {
    pub operations: Vec<Op>,
    pub final_event_count: usize,
    /// Ordered event stream captured during the sequence (recorded trace).
    pub trace: Vec<TraceEvent>,
    /// The reconstructed ordered expectation, so a failing input can be
    /// replayed and diffed against `trace`.
    pub expected_trace: Vec<ExpectedTraceEvent>,
}

impl<Op> SequenceReport<Op> {
    /// Replays the recorded trace against the recorded expectation and the
    /// built-in pairing invariant — mirroring what the M1 trace decoder would
    /// verify when a minimized failing input is replayed off-chain.
    pub fn diff(&self) -> TraceDiff {
        let span = self.expected_trace.len().max(self.trace.len());
        let mut first_divergence = None;
        for index in 0..span {
            let same = match (self.expected_trace.get(index), self.trace.get(index)) {
                (Some(expected), Some(observed)) => expected.matches(observed),
                (None, None) => true,
                _ => false,
            };
            if !same {
                first_divergence = Some(index);
                break;
            }
        }
        let telemetry = check_telemetry_pairing(&self.trace);
        TraceDiff {
            expected: self.expected_trace.clone(),
            observed: self.trace.clone(),
            first_divergence,
            telemetry,
        }
    }

    /// Asserts that the recorded trace replays cleanly against the recorded
    /// expectation and the pairing invariant; panics with a diff otherwise.
    pub fn assert_replays_cleanly(&self) {
        let diff = self.diff();
        assert!(
            diff.first_divergence.is_none(),
            "recorded trace diverges from recorded expectation: {diff:?}"
        );
        assert!(
            diff.telemetry.is_ok(),
            "recorded trace violates telemetry pairing: {diff:?}"
        );
    }
}

/// The diff produced when replaying a recorded trace against its expectation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TraceDiff {
    pub expected: Vec<ExpectedTraceEvent>,
    pub observed: Vec<TraceEvent>,
    pub first_divergence: Option<usize>,
    pub telemetry: Result<(), TraceViolation>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CrashReport<Op> {
    pub operation_index: usize,
    pub operation: Op,
    pub operations: Vec<Op>,
    pub panic_message: String,
}

pub fn execute_sequence<H>(
    harness: &mut H,
    operations: &[H::Operation],
) -> Result<SequenceReport<H::Operation>, CrashReport<H::Operation>>
where
    H: BehaviorHarness,
{
    let mut event_count = harness.event_count();
    let mut trace = harness.captured_trace();
    let mut expected_trace: Vec<ExpectedTraceEvent> = Vec::new();

    for (index, operation) in operations.iter().enumerate() {
        let trace_before = trace.len();

        let outcome = panic::catch_unwind(AssertUnwindSafe(|| {
            let outcome = harness.apply(operation);
            harness.assert_invariants();
            harness.assert_round_trip();
            outcome
        }))
        .map_err(|panic_payload| CrashReport {
            operation_index: index,
            operation: operation.clone(),
            operations: operations[..=index].to_vec(),
            panic_message: panic_message(panic_payload),
        })?;

        expected_trace.extend(outcome.expected_trace.events.iter().cloned());

        let updated_event_count = harness.event_count();
        assert_eq!(
            updated_event_count,
            event_count + outcome.expected_event_delta,
            "unexpected event delta at step {index} for operation {:?}",
            operation
        );
        event_count = updated_event_count;

        // Ordered, typed event assertions declared by the operation, enforced
        // on the shared trait for every harness that opts in.
        if !outcome.expected_trace.is_empty() {
            let emitted: Vec<TraceEvent> = harness
                .captured_trace()
                .iter()
                .skip(trace_before)
                .cloned()
                .collect();
            if outcome.expected_trace.events.len() != emitted.len() {
                panic!(
                    "trace expectation length mismatch at step {index} for operation {:?}: \
                     expected {:?}, observed {:?}",
                    operation, outcome.expected_trace.events, emitted
                );
            }
            for (expected, observed) in outcome
                .expected_trace
                .events
                .iter()
                .zip(emitted.iter())
            {
                assert!(
                    expected.matches(observed),
                    "trace topic/payload mismatch at step {index} for operation {:?}: \
                     expected {:?}, observed {:?}",
                    operation,
                    expected,
                    observed
                );
            }
        }

        trace = harness.captured_trace();
    }

    // Built-in telemetry pairing invariant (scope 2): no `FN_INVOKE` may be
    // left unclosed, and the sequence must never end mid-invocation.
    if let Err(violation) = check_telemetry_pairing(&trace) {
        panic!("telemetry pairing invariant violated: {violation:?}");
    }

    Ok(SequenceReport {
        operations: operations.to_vec(),
        final_event_count: event_count,
        trace,
        expected_trace,
    })
}

#[derive(Clone, Debug)]
pub struct RegressionCase<Op> {
    pub name: &'static str,
    pub operations: Vec<Op>,
}

pub fn run_regressions<H, F>(cases: &[RegressionCase<H::Operation>], mut make_harness: F)
where
    H: BehaviorHarness,
    F: FnMut() -> H,
{
    for case in cases {
        let mut harness = make_harness();
        if let Err(report) = execute_sequence(&mut harness, &case.operations) {
            panic!(
                "regression case '{}' failed at step {} on {:?}: {}",
                case.name, report.operation_index, report.operation, report.panic_message
            );
        }
    }
}

fn panic_message(payload: Box<dyn core::any::Any + Send>) -> String {
    if let Some(message) = payload.downcast_ref::<&'static str>() {
        (*message).to_owned()
    } else if let Some(message) = payload.downcast_ref::<String>() {
        message.clone()
    } else {
        "non-string panic payload".to_owned()
    }
}