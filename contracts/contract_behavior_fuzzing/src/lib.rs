use core::fmt::Debug;
use std::panic::{self, AssertUnwindSafe};
use std::vec::Vec;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct OperationOutcome {
    pub expected_event_delta: usize,
}

impl OperationOutcome {
    pub const fn new(expected_event_delta: usize) -> Self {
        Self {
            expected_event_delta,
        }
    }
}

pub trait BehaviorHarness {
    type Operation: Clone + Debug;

    fn apply(&mut self, operation: &Self::Operation) -> OperationOutcome;
    fn assert_invariants(&self);
    fn event_count(&self) -> usize;
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SequenceReport<Op> {
    pub operations: Vec<Op>,
    pub final_event_count: usize,
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

    for (index, operation) in operations.iter().enumerate() {
        let outcome = panic::catch_unwind(AssertUnwindSafe(|| {
            let outcome = harness.apply(operation);
            harness.assert_invariants();
            outcome
        }))
        .map_err(|panic_payload| CrashReport {
            operation_index: index,
            operation: operation.clone(),
            operations: operations[..=index].to_vec(),
            panic_message: panic_message(panic_payload),
        })?;

        let updated_event_count = harness.event_count();
        assert_eq!(
            updated_event_count,
            event_count + outcome.expected_event_delta,
            "unexpected event delta at step {index} for operation {:?}",
            operation
        );
        event_count = updated_event_count;
    }

    Ok(SequenceReport {
        operations: operations.to_vec(),
        final_event_count: event_count,
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
