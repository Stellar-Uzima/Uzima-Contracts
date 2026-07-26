//! Fuzz tests for cross-contract interaction invariants.
//!
//! Tests that state consistency is maintained across contracts,
//! no orphaned references exist, and rollback is atomic.

use contract_behavior_fuzzing::{
    execute_sequence, run_regressions, BehaviorHarness, OperationOutcome, RegressionCase,
};
use proptest::prelude::*;
use soroban_sdk::{
    testutils::{Address as _, Events as _},
    Address, Env, String, Vec as SorobanVec,
};

mod support;

// =============================================================================
// High-risk invariants
// =============================================================================

/// Invariant: State consistency across contracts.
fn invariant_state_consistency(results: &[OperationOutcome]) -> bool {
    // Check that all cross-contract operations succeeded or failed atomically
    for outcome in results {
        if let OperationOutcome::PartialFailure { .. } = outcome {
            return false;
        }
    }
    true
}

/// Invariant: No orphaned references.
fn invariant_no_orphaned_references(results: &[OperationOutcome]) -> bool {
    // After a cross-contract operation, all references should be valid
    for outcome in results {
        if let OperationOutcome::OrphanedReference { .. } = outcome {
            return false;
        }
    }
    true
}

/// Invariant: Rollback is atomic.
fn invariant_atomic_rollback(results: &[OperationOutcome]) -> bool {
    // If rollback occurs, all state changes should be reverted
    let mut rolled_back = false;
    for outcome in results {
        match outcome {
            OperationOutcome::Rollback { .. } => rolled_back = true,
            OperationOutcome::StateChange { .. } if rolled_back => return false,
            _ => {}
        }
    }
    true
}

// =============================================================================
// Fuzz operations
// =============================================================================

#[derive(Clone, Debug)]
enum CrossContractOp {
    /// Create a reference from contract A to contract B
    CreateReference {
        source_seed: u8,
        target_seed: u8,
        ref_type: u8,
    },
    /// Delete a reference
    DeleteReference {
        source_seed: u8,
        target_seed: u8,
    },
    /// Migrate data from contract A to contract B
    MigrateData {
        source_seed: u8,
        target_seed: u8,
        data_seed: u8,
    },
    /// Rollback the last migration
    Rollback {
        source_seed: u8,
        target_seed: u8,
    },
    /// Check reference validity
    CheckReference {
        source_seed: u8,
        target_seed: u8,
    },
}

// =============================================================================
// Harness
// =============================================================================

struct CrossContractHarness {
    env: Env,
    references: std::collections::HashMap<(u8, u8), u8>, // (source, target) -> ref_type
    migrations: std::collections::Vec<(u8, u8, u8)>,     // (source, target, data) history
}

impl CrossContractHarness {
    fn new() -> Self {
        Self {
            env: Env::default(),
            references: std::collections::HashMap::new(),
            migrations: std::collections::Vec::new(),
        }
    }

    fn execute_op(&mut self, op: &CrossContractOp) -> OperationOutcome {
        match op {
            CrossContractOp::CreateReference {
                source_seed,
                target_seed,
                ref_type,
            } => {
                self.references
                    .insert((*source_seed, *target_seed), *ref_type);
                OperationOutcome::Success
            }
            CrossContractOp::DeleteReference {
                source_seed,
                target_seed,
            } => {
                if self.references.remove(&(*source_seed, *target_seed)).is_some() {
                    OperationOutcome::Success
                } else {
                    OperationOutcome::OrphanedReference {
                        source: *source_seed,
                        target: *target_seed,
                    }
                }
            }
            CrossContractOp::MigrateData {
                source_seed,
                target_seed,
                data_seed,
            } => {
                // Check that reference exists
                if self
                    .references
                    .contains_key(&(*source_seed, *target_seed))
                {
                    self.migrations
                        .push((*source_seed, *target_seed, *data_seed));
                    OperationOutcome::StateChange {
                        source: *source_seed,
                        target: *target_seed,
                    }
                } else {
                    OperationOutcome::OrphanedReference {
                        source: *source_seed,
                        target: *target_seed,
                    }
                }
            }
            CrossContractOp::Rollback {
                source_seed,
                target_seed,
            } => {
                if let Some(pos) = self
                    .migrations
                    .iter()
                    .rposition(|&(s, t, _)| s == *source_seed && t == *target_seed)
                {
                    self.migrations.remove(pos);
                    OperationOutcome::Rollback {
                        source: *source_seed,
                        target: *target_seed,
                    }
                } else {
                    OperationOutcome::Failure {
                        reason: "Nothing to rollback".to_string(),
                    }
                }
            }
            CrossContractOp::CheckReference {
                source_seed,
                target_seed,
            } => {
                if self
                    .references
                    .contains_key(&(*source_seed, *target_seed))
                {
                    OperationOutcome::Success
                } else {
                    OperationOutcome::OrphanedReference {
                        source: *source_seed,
                        target: *target_seed,
                    }
                }
            }
        }
    }
}

impl BehaviorHarness for CrossContractHarness {
    type Operation = CrossContractOp;
    type Error = String;

    fn execute_op(&mut self, op: &Self::Operation) -> Result<OperationOutcome, Self::Error> {
        Ok(self.execute_op(op))
    }
}

// =============================================================================
// Proptest strategies
// =============================================================================

fn contract_seed() -> impl Strategy<Value = u8> {
    (0u8..5)
}

fn ref_type() -> impl Strategy<Value = u8> {
    (0u8..3)
}

fn data_seed() -> impl Strategy<Value = u8> {
    (0u8..10)
}

fn cross_contract_op() -> impl Strategy<Value = CrossContractOp> {
    prop_oneof![
        (contract_seed(), contract_seed(), ref_type()).prop_map(
            |(s, t, r)| CrossContractOp::CreateReference {
                source_seed: s,
                target_seed: t,
                ref_type: r,
            }
        ),
        (contract_seed(), contract_seed()).prop_map(|(s, t)| CrossContractOp::DeleteReference {
            source_seed: s,
            target_seed: t,
        }),
        (contract_seed(), contract_seed(), data_seed()).prop_map(|(s, t, d)| {
            CrossContractOp::MigrateData {
                source_seed: s,
                target_seed: t,
                data_seed: d,
            }
        }),
        (contract_seed(), contract_seed()).prop_map(|(s, t)| CrossContractOp::Rollback {
            source_seed: s,
            target_seed: t,
        }),
        (contract_seed(), contract_seed()).prop_map(|(s, t)| CrossContractOp::CheckReference {
            source_seed: s,
            target_seed: t,
        }),
    ]
}

// =============================================================================
// Tests
// =============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn test_cross_contract_invariants(ops in prop::collection::vec(cross_contract_op(), 0..20)) {
        let mut harness = CrossContractHarness::new();
        let results = execute_sequence(&mut harness, ops);

        prop_assert!(
            invariant_state_consistency(&results),
            "State inconsistency detected"
        );
        prop_assert!(
            invariant_no_orphaned_references(&results),
            "Orphaned references detected"
        );
        prop_assert!(
            invariant_atomic_rollback(&results),
            "Non-atomic rollback detected"
        );
    }
}

#[test]
fn test_cross_contract_regression() {
    let cases: Vec<RegressionCase<CrossContractOp>> = vec![
        RegressionCase {
            description: "Create reference and migrate",
            operations: vec![
                CrossContractOp::CreateReference {
                    source_seed: 0,
                    target_seed: 1,
                    ref_type: 0,
                },
                CrossContractOp::MigrateData {
                    source_seed: 0,
                    target_seed: 1,
                    data_seed: 5,
                },
            ],
            expected_invariant_holds: true,
        },
        RegressionCase {
            description: "Migrate without reference fails",
            operations: vec![CrossContractOp::MigrateData {
                source_seed: 2,
                target_seed: 3,
                data_seed: 7,
            }],
            expected_invariant_holds: false, // OrphanedReference detected
        },
        RegressionCase {
            description: "Rollback reverts last migration",
            operations: vec![
                CrossContractOp::CreateReference {
                    source_seed: 0,
                    target_seed: 1,
                    ref_type: 1,
                },
                CrossContractOp::MigrateData {
                    source_seed: 0,
                    target_seed: 1,
                    data_seed: 5,
                },
                CrossContractOp::Rollback {
                    source_seed: 0,
                    target_seed: 1,
                },
            ],
            expected_invariant_holds: true,
        },
    ];

    run_regressions(cases);
}
