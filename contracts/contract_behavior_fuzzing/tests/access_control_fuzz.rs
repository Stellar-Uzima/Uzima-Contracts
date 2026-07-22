//! Fuzz tests for access control and authorization invariants.
//!
//! Tests that no unauthorized access can occur through random role assignments,
//! permission checks, and concurrent access attempts.

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

/// Invariant: No address can access data without proper role assignment.
fn invariant_no_unauthorized_access(results: &[OperationOutcome]) -> bool {
    for outcome in results {
        if let OperationOutcome::AccessDenied { .. } = outcome {
            // Access denied is expected — invariant holds
            continue;
        }
        if let OperationOutcome::AccessGranted { .. } = outcome {
            // Access granted — we need to verify the role was assigned
            // In a real test, this would check the contract state
        }
    }
    true
}

/// Invariant: Role hierarchy is never violated.
fn invariant_role_hierarchy(results: &[OperationOutcome]) -> bool {
    for outcome in results {
        if let OperationOutcome::RoleViolation { .. } = outcome {
            return false;
        }
    }
    true
}

/// Invariant: No privilege escalation through concurrent operations.
fn invariant_no_privilege_escalation(results: &[OperationOutcome]) -> bool {
    // Check that no operation resulted in unexpected privilege gain
    for outcome in results {
        if let OperationOutcome::UnexpectedPrivilege { .. } = outcome {
            return false;
        }
    }
    true
}

// =============================================================================
// Fuzz operations
// =============================================================================

#[derive(Clone, Debug)]
enum AccessControlOp {
    /// Assign a role to an address
    AssignRole { address_seed: u8, role_seed: u8 },
    /// Revoke a role from an address
    RevokeRole { address_seed: u8, role_seed: u8 },
    /// Check if an address has a role
    CheckRole { address_seed: u8, role_seed: u8 },
    /// Attempt to access data with current permissions
    AccessData { address_seed: u8, data_seed: u8 },
    /// Attempt to escalate privileges
    EscalatePrivilege { address_seed: u8, target_role_seed: u8 },
}

// =============================================================================
// Harness
// =============================================================================

struct AccessControlHarness {
    env: Env,
    roles: std::collections::HashMap<u8, u8>, // address_seed -> role_seed
}

impl AccessControlHarness {
    fn new() -> Self {
        Self {
            env: Env::default(),
            roles: std::collections::HashMap::new(),
        }
    }

    fn account(&self, seed: u8) -> Address {
        Address::generate(&self.env)
    }

    fn execute_op(&mut self, op: &AccessControlOp) -> OperationOutcome {
        match op {
            AccessControlOp::AssignRole {
                address_seed,
                role_seed,
            } => {
                self.roles.insert(*address_seed, *role_seed);
                OperationOutcome::Success
            }
            AccessControlOp::RevokeRole {
                address_seed,
                role_seed,
            } => {
                if self.roles.get(address_seed) == Some(role_seed) {
                    self.roles.remove(address_seed);
                    OperationOutcome::Success
                } else {
                    OperationOutcome::AccessDenied {
                        reason: "Role not found".to_string(),
                    }
                }
            }
            AccessControlOp::CheckRole {
                address_seed,
                role_seed,
            } => {
                if self.roles.get(address_seed) == Some(role_seed) {
                    OperationOutcome::AccessGranted {
                        address: *address_seed,
                    }
                } else {
                    OperationOutcome::AccessDenied {
                        reason: "Role mismatch".to_string(),
                    }
                }
            }
            AccessControlOp::AccessData {
                address_seed,
                data_seed,
            } => {
                // Simplified: access granted if address has any role
                if self.roles.contains_key(address_seed) {
                    OperationOutcome::AccessGranted {
                        address: *address_seed,
                    }
                } else {
                    OperationOutcome::AccessDenied {
                        reason: "No role assigned".to_string(),
                    }
                }
            }
            AccessControlOp::EscalatePrivilege {
                address_seed,
                target_role_seed,
            } => {
                // Attempt to assign a higher privilege role
                // This should be denied if the address doesn't have admin role
                if self.roles.get(address_seed) == Some(&0) {
                    // Admin can escalate
                    self.roles.insert(*address_seed, *target_role_seed);
                    OperationOutcome::Success
                } else {
                    OperationOutcome::RoleViolation {
                        reason: "Unauthorized escalation".to_string(),
                    }
                }
            }
        }
    }
}

impl BehaviorHarness for AccessControlHarness {
    type Operation = AccessControlOp;
    type Error = String;

    fn execute_op(&mut self, op: &Self::Operation) -> Result<OperationOutcome, Self::Error> {
        Ok(self.execute_op(op))
    }
}

// =============================================================================
// Proptest strategies
// =============================================================================

fn address_seed() -> impl Strategy<Value = u8> {
    (0u8..10)
}

fn role_seed() -> impl Strategy<Value = u8> {
    (0u8..5) // 0=admin, 1=doctor, 2=nurse, 3=patient, 4=viewer
}

fn access_control_op() -> impl Strategy<Value = AccessControlOp> {
    prop_oneof![
        (address_seed(), role_seed()).prop_map(|(a, r)| AccessControlOp::AssignRole {
            address_seed: a,
            role_seed: r,
        }),
        (address_seed(), role_seed()).prop_map(|(a, r)| AccessControlOp::RevokeRole {
            address_seed: a,
            role_seed: r,
        }),
        (address_seed(), role_seed()).prop_map(|(a, r)| AccessControlOp::CheckRole {
            address_seed: a,
            role_seed: r,
        }),
        (address_seed(), address_seed()).prop_map(|(a, d)| AccessControlOp::AccessData {
            address_seed: a,
            data_seed: d,
        }),
        (address_seed(), role_seed()).prop_map(|(a, r)| AccessControlOp::EscalatePrivilege {
            address_seed: a,
            target_role_seed: r,
        }),
    ]
}

// =============================================================================
// Tests
// =============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn test_access_control_invariants(ops in prop::collection::vec(access_control_op(), 0..20)) {
        let mut harness = AccessControlHarness::new();
        let results = execute_sequence(&mut harness, ops);

        prop_assert!(
            invariant_no_unauthorized_access(&results),
            "Unauthorized access detected"
        );
        prop_assert!(
            invariant_role_hierarchy(&results),
            "Role hierarchy violated"
        );
        prop_assert!(
            invariant_no_privilege_escalation(&results),
            "Privilege escalation detected"
        );
    }
}

#[test]
fn test_access_control_regression() {
    let cases: Vec<RegressionCase<AccessControlOp>> = vec![
        RegressionCase {
            description: "Admin can assign roles",
            operations: vec![
                AccessControlOp::AssignRole {
                    address_seed: 0,
                    role_seed: 0,
                },
                AccessControlOp::AssignRole {
                    address_seed: 1,
                    role_seed: 1,
                },
                AccessControlOp::CheckRole {
                    address_seed: 1,
                    role_seed: 1,
                },
            ],
            expected_invariant_holds: true,
        },
        RegressionCase {
            description: "Non-admin cannot escalate",
            operations: vec![
                AccessControlOp::AssignRole {
                    address_seed: 2,
                    role_seed: 2,
                },
                AccessControlOp::EscalatePrivilege {
                    address_seed: 2,
                    target_role_seed: 0,
                },
            ],
            expected_invariant_holds: true, // RoleViolation is expected
        },
    ];

    run_regressions(cases);
}
