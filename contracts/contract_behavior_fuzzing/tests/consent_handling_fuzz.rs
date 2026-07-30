//! Fuzz tests for patient consent handling invariants.
//!
//! Tests that consent is always required before data access,
//! expired consent is blocked, and consent revocation is immediate.

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

/// Invariant: No data access without valid consent.
fn invariant_consent_required(results: &[OperationOutcome]) -> bool {
    for outcome in results {
        if let OperationOutcome::DataAccess { .. } = outcome {
            // In a real test, this would verify consent was granted
        }
        if let OperationOutcome::UnauthorizedDataAccess { .. } = outcome {
            return false;
        }
    }
    true
}

/// Invariant: Expired consent is always blocked.
fn invariant_expired_consent_blocked(results: &[OperationOutcome]) -> bool {
    for outcome in results {
        if let OperationOutcome::ExpiredConsentAccess { .. } = outcome {
            return false;
        }
    }
    true
}

/// Invariant: Consent revocation is immediate.
fn invariant_revocation_immediate(results: &[OperationOutcome]) -> bool {
    // After revocation, no access should be granted
    let mut revoked = false;
    for outcome in results {
        match outcome {
            OperationOutcome::ConsentRevoked { .. } => revoked = true,
            OperationOutcome::DataAccess { .. } if revoked => return false,
            _ => {}
        }
    }
    true
}

// =============================================================================
// Fuzz operations
// =============================================================================

#[derive(Clone, Debug)]
enum ConsentOp {
    /// Grant consent from patient to provider
    GrantConsent {
        patient_seed: u8,
        provider_seed: u8,
        expiry_ledger: u32,
    },
    /// Revoke consent
    RevokeConsent {
        patient_seed: u8,
        provider_seed: u8,
    },
    /// Access data with consent check
    AccessData {
        provider_seed: u8,
        patient_seed: u8,
        current_ledger: u32,
    },
    /// Check consent status
    CheckConsent {
        patient_seed: u8,
        provider_seed: u8,
    },
}

// =============================================================================
// Harness
// =============================================================================

struct ConsentHarness {
    env: Env,
    consents: std::collections::HashMap<(u8, u8), u32>, // (patient, provider) -> expiry_ledger
}

impl ConsentHarness {
    fn new() -> Self {
        Self {
            env: Env::default(),
            consents: std::collections::HashMap::new(),
        }
    }

    fn execute_op(&mut self, op: &ConsentOp) -> OperationOutcome {
        match op {
            ConsentOp::GrantConsent {
                patient_seed,
                provider_seed,
                expiry_ledger,
            } => {
                self.consents
                    .insert((*patient_seed, *provider_seed), *expiry_ledger);
                OperationOutcome::ConsentGranted {
                    patient: *patient_seed,
                    provider: *provider_seed,
                }
            }
            ConsentOp::RevokeConsent {
                patient_seed,
                provider_seed,
            } => {
                self.consents.remove(&(*patient_seed, *provider_seed));
                OperationOutcome::ConsentRevoked {
                    patient: *patient_seed,
                    provider: *provider_seed,
                }
            }
            ConsentOp::AccessData {
                provider_seed,
                patient_seed,
                current_ledger,
            } => {
                if let Some(&expiry) = self.consents.get(&(*patient_seed, *provider_seed)) {
                    if *current_ledger <= expiry {
                        OperationOutcome::DataAccess {
                            provider: *provider_seed,
                            patient: *patient_seed,
                        }
                    } else {
                        OperationOutcome::ExpiredConsentAccess {
                            provider: *provider_seed,
                            patient: *patient_seed,
                        }
                    }
                } else {
                    OperationOutcome::UnauthorizedDataAccess {
                        provider: *provider_seed,
                        patient: *patient_seed,
                    }
                }
            }
            ConsentOp::CheckConsent {
                patient_seed,
                provider_seed,
            } => {
                if self.consents.contains_key(&(*patient_seed, *provider_seed)) {
                    OperationOutcome::ConsentActive {
                        patient: *patient_seed,
                        provider: *provider_seed,
                    }
                } else {
                    OperationOutcome::ConsentInactive {
                        patient: *patient_seed,
                        provider: *provider_seed,
                    }
                }
            }
        }
    }
}

impl BehaviorHarness for ConsentHarness {
    type Operation = ConsentOp;
    type Error = String;

    fn execute_op(&mut self, op: &Self::Operation) -> Result<OperationOutcome, Self::Error> {
        Ok(self.execute_op(op))
    }
}

// =============================================================================
// Proptest strategies
// =============================================================================

fn patient_seed() -> impl Strategy<Value = u8> {
    (0u8..5)
}

fn provider_seed() -> impl Strategy<Value = u8> {
    (5u8..10)
}

fn expiry_ledger() -> impl Strategy<Value = u32> {
    (100u32..1000)
}

fn current_ledger() -> impl Strategy<Value = u32> {
    (0u32..1500)
}

fn consent_op() -> impl Strategy<Value = ConsentOp> {
    prop_oneof![
        (patient_seed(), provider_seed(), expiry_ledger())
            .prop_map(|(p, pr, e)| ConsentOp::GrantConsent {
                patient_seed: p,
                provider_seed: pr,
                expiry_ledger: e,
            }),
        (patient_seed(), provider_seed()).prop_map(|(p, pr)| ConsentOp::RevokeConsent {
            patient_seed: p,
            provider_seed: pr,
        }),
        (provider_seed(), patient_seed(), current_ledger())
            .prop_map(|(pr, p, cl)| ConsentOp::AccessData {
                provider_seed: pr,
                patient_seed: p,
                current_ledger: cl,
            }),
        (patient_seed(), provider_seed()).prop_map(|(p, pr)| ConsentOp::CheckConsent {
            patient_seed: p,
            provider_seed: pr,
        }),
    ]
}

// =============================================================================
// Tests
// =============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn test_consent_invariants(ops in prop::collection::vec(consent_op(), 0..20)) {
        let mut harness = ConsentHarness::new();
        let results = execute_sequence(&mut harness, ops);

        prop_assert!(
            invariant_consent_required(&results),
            "Data access without consent detected"
        );
        prop_assert!(
            invariant_expired_consent_blocked(&results),
            "Expired consent access detected"
        );
        prop_assert!(
            invariant_revocation_immediate(&results),
            "Revocation not immediate"
        );
    }
}

#[test]
fn test_consent_regression() {
    let cases: Vec<RegressionCase<ConsentOp>> = vec![
        RegressionCase {
            description: "Grant and access within expiry",
            operations: vec![
                ConsentOp::GrantConsent {
                    patient_seed: 0,
                    provider_seed: 5,
                    expiry_ledger: 500,
                },
                ConsentOp::AccessData {
                    provider_seed: 5,
                    patient_seed: 0,
                    current_ledger: 250,
                },
            ],
            expected_invariant_holds: true,
        },
        RegressionCase {
            description: "Access after expiry denied",
            operations: vec![
                ConsentOp::GrantConsent {
                    patient_seed: 1,
                    provider_seed: 6,
                    expiry_ledger: 100,
                },
                ConsentOp::AccessData {
                    provider_seed: 6,
                    patient_seed: 1,
                    current_ledger: 200,
                },
            ],
            expected_invariant_holds: false, // ExpiredConsentAccess detected
        },
        RegressionCase {
            description: "Revocation blocks subsequent access",
            operations: vec![
                ConsentOp::GrantConsent {
                    patient_seed: 2,
                    provider_seed: 7,
                    expiry_ledger: 1000,
                },
                ConsentOp::RevokeConsent {
                    patient_seed: 2,
                    provider_seed: 7,
                },
                ConsentOp::AccessData {
                    provider_seed: 7,
                    patient_seed: 2,
                    current_ledger: 100,
                },
            ],
            expected_invariant_holds: false, // UnauthorizedDataAccess after revocation
        },
    ];

    run_regressions(cases);
}
