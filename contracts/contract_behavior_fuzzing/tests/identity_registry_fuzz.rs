use std::collections::BTreeMap;
use std::vec::Vec;

use contract_behavior_fuzzing::{
    execute_sequence, run_regressions, BehaviorHarness, OperationOutcome, RegressionCase,
};
use identity_registry::{
    DIDStatus, Error, IdentityRegistryContract, IdentityRegistryContractClient, ServiceEndpoint,
    VerificationMethodType, VerificationRelationship,
};
use proptest::collection::vec;
use proptest::prelude::*;
use soroban_sdk::{
    testutils::{Address as _, Events as _, Ledger},
    Address, Env, String, Vec as SorobanVec,
};

mod support;

#[derive(Clone, Debug)]
enum IdentityOp {
    CreateDid {
        subject: u8,
        key_seed: u8,
    },
    AddService {
        subject: u8,
        service_slot: u8,
    },
    RemoveService {
        subject: u8,
        service_slot: u8,
    },
    AddVerificationMethod {
        subject: u8,
        method_slot: u8,
    },
    RotateKey {
        subject: u8,
        method_slot: u8,
        key_seed: u8,
    },
    RevokeVerificationMethod {
        subject: u8,
        method_slot: u8,
    },
    DeactivateDid {
        subject: u8,
    },
}

#[derive(Clone, Debug, Default)]
struct MethodModel {
    total: usize,
    active: usize,
}

#[derive(Clone, Debug, Default)]
struct DidModel {
    exists: bool,
    deactivated: bool,
    version: u32,
    services: BTreeMap<u8, usize>,
    methods: BTreeMap<u8, MethodModel>,
}

struct IdentityHarness {
    env: Env,
    client: IdentityRegistryContractClient<'static>,
    subjects: Vec<Address>,
    models: Vec<DidModel>,
}

impl IdentityHarness {
    fn new() -> Self {
        let env = Env::default();
        env.mock_all_auths();
        env.ledger().with_mut(|ledger| {
            ledger.timestamp = 1;
        });

        let owner = Address::generate(&env);
        let contract_id = env.register_contract(None, IdentityRegistryContract);
        let client = IdentityRegistryContractClient::new(&env, &contract_id);
        let init_result = client
            .try_initialize(&owner, &support::s(&env, "testnet"))
            .expect("identity registry should initialize");
        assert_eq!(init_result, Ok(()));

        let subjects = (0..2).map(|_| Address::generate(&env)).collect::<Vec<_>>();

        Self {
            env,
            client,
            subjects,
            models: vec![DidModel::default(); 2],
        }
    }

    fn bump_time(&self) {
        self.env.ledger().with_mut(|ledger| {
            ledger.timestamp += 100_000;
        });
    }

    fn subject(&self, index: u8) -> &Address {
        &self.subjects[index as usize % self.subjects.len()]
    }

    fn service_id(env: &Env, slot: u8) -> String {
        String::from_str(env, &format!("#service-{}", slot % 4))
    }

    fn method_id(env: &Env, slot: u8) -> String {
        String::from_str(env, &format!("#key-{}", 2 + (slot % 4)))
    }
}

impl BehaviorHarness for IdentityHarness {
    type Operation = IdentityOp;

    fn apply(&mut self, operation: &Self::Operation) -> OperationOutcome {
        self.bump_time();

        match operation {
            IdentityOp::CreateDid { subject, key_seed } => {
                let subject_index = *subject as usize % self.subjects.len();
                let success = !self.models[subject_index].exists;
                let services = SorobanVec::<ServiceEndpoint>::new(&self.env);
                let result = self.client.try_create_did(
                    self.subject(*subject),
                    &support::bytes32(&self.env, *key_seed),
                    &services,
                );

                if success {
                    assert!(result.is_ok());
                    self.models[subject_index] = DidModel {
                        exists: true,
                        deactivated: false,
                        version: 1,
                        services: BTreeMap::new(),
                        methods: BTreeMap::new(),
                    };
                } else {
                    assert!(matches!(result, Err(Ok(Error::DIDAlreadyExists))));
                }

                OperationOutcome::new(usize::from(success))
            }
            IdentityOp::AddService {
                subject,
                service_slot,
            } => {
                let subject_index = *subject as usize % self.subjects.len();
                let model = &self.models[subject_index];
                let success = model.exists && !model.deactivated;
                let result = self.client.try_add_service(
                    self.subject(*subject),
                    &Self::service_id(&self.env, *service_slot),
                    &support::s(&self.env, "LinkedDomains"),
                    &support::s(&self.env, "https://uzima.test/service"),
                );

                if success {
                    assert!(result.is_ok());
                    self.models[subject_index]
                        .services
                        .entry(*service_slot % 4)
                        .and_modify(|count| *count += 1)
                        .or_insert(1);
                    self.models[subject_index].version += 1;
                } else {
                    assert!(result.is_err());
                }

                OperationOutcome::new(usize::from(success))
            }
            IdentityOp::RemoveService {
                subject,
                service_slot,
            } => {
                let subject_index = *subject as usize % self.subjects.len();
                let has_service = self.models[subject_index]
                    .services
                    .get(&(*service_slot % 4))
                    .copied()
                    .unwrap_or(0)
                    > 0;
                let success = self.models[subject_index].exists && has_service;

                let result = self.client.try_remove_service(
                    self.subject(*subject),
                    &Self::service_id(&self.env, *service_slot),
                );

                if success {
                    assert!(result.is_ok());
                    self.models[subject_index]
                        .services
                        .remove(&(*service_slot % 4));
                    self.models[subject_index].version += 1;
                } else {
                    assert!(result.is_err());
                }

                OperationOutcome::new(usize::from(success))
            }
            IdentityOp::AddVerificationMethod {
                subject,
                method_slot,
            } => {
                let subject_index = *subject as usize % self.subjects.len();
                let success =
                    self.models[subject_index].exists && !self.models[subject_index].deactivated;
                let relationships =
                    SorobanVec::from_array(&self.env, [VerificationRelationship::Authentication]);
                let result = self.client.try_add_verification_method(
                    self.subject(*subject),
                    &Self::method_id(&self.env, *method_slot),
                    &VerificationMethodType::Ed25519VerificationKey2020,
                    &support::bytes32(&self.env, 20 + (*method_slot % 4)),
                    &relationships,
                );

                if success {
                    assert!(result.is_ok());
                    self.models[subject_index]
                        .methods
                        .entry(2 + (*method_slot % 4))
                        .and_modify(|method| {
                            method.total += 1;
                            method.active += 1;
                        })
                        .or_insert(MethodModel {
                            total: 1,
                            active: 1,
                        });
                    self.models[subject_index].version += 1;
                } else {
                    assert!(result.is_err());
                }

                OperationOutcome::new(usize::from(success))
            }
            IdentityOp::RotateKey {
                subject,
                method_slot,
                key_seed,
            } => {
                let subject_index = *subject as usize % self.subjects.len();
                let method_key = 2 + (*method_slot % 4);
                let success = self.models[subject_index].exists
                    && !self.models[subject_index].deactivated
                    && self.models[subject_index]
                        .methods
                        .get(&method_key)
                        .map(|method| method.total > 0)
                        .unwrap_or(false);
                let result = self.client.try_rotate_key(
                    self.subject(*subject),
                    &Self::method_id(&self.env, *method_slot),
                    &support::bytes32(&self.env, *key_seed),
                );

                if success {
                    assert!(result.is_ok());
                    self.models[subject_index].version += 1;
                } else {
                    assert!(result.is_err());
                }

                OperationOutcome::new(usize::from(success))
            }
            IdentityOp::RevokeVerificationMethod {
                subject,
                method_slot,
            } => {
                let subject_index = *subject as usize % self.subjects.len();
                let method_key = 2 + (*method_slot % 4);
                let active_methods = self.models[subject_index]
                    .methods
                    .values()
                    .map(|method| method.active)
                    .sum::<usize>()
                    + usize::from(self.models[subject_index].exists);
                let success = self.models[subject_index].exists
                    && !self.models[subject_index].deactivated
                    && self.models[subject_index]
                        .methods
                        .get(&method_key)
                        .map(|method| method.active > 0)
                        .unwrap_or(false)
                    && active_methods > 1;
                let result = self.client.try_revoke_verification_method(
                    self.subject(*subject),
                    &Self::method_id(&self.env, *method_slot),
                );

                if success {
                    assert!(result.is_ok());
                    if let Some(method) = self.models[subject_index].methods.get_mut(&method_key) {
                        method.active = 0;
                    }
                    self.models[subject_index].version += 1;
                } else {
                    assert!(result.is_err());
                }

                OperationOutcome::new(usize::from(success))
            }
            IdentityOp::DeactivateDid { subject } => {
                let subject_index = *subject as usize % self.subjects.len();
                let success = self.models[subject_index].exists;
                let result = self.client.try_deactivate_did(self.subject(*subject));

                if success {
                    assert!(result.is_ok());
                    self.models[subject_index].deactivated = true;
                } else {
                    assert!(result.is_err());
                }

                OperationOutcome::new(usize::from(success))
            }
        }
    }

    fn assert_invariants(&self) {
        for (index, subject) in self.subjects.iter().enumerate() {
            let model = &self.models[index];

            if !model.exists {
                assert!(matches!(
                    self.client.try_resolve_did(subject),
                    Err(Ok(Error::DIDNotFound))
                ));
                continue;
            }

            if model.deactivated {
                assert!(matches!(
                    self.client.try_resolve_did(subject),
                    Err(Ok(Error::DIDDeactivated))
                ));
                continue;
            }

            let did = self.client.resolve_did(subject);
            assert_eq!(did.status, DIDStatus::Active);
            assert_eq!(did.version, model.version);

            let active_services = model.services.values().sum::<usize>();
            assert_eq!(did.services.len() as usize, active_services);

            let total_methods = 1 + model
                .methods
                .values()
                .map(|method| method.total)
                .sum::<usize>();
            let active_methods = did
                .verification_methods
                .iter()
                .filter(|method| method.is_active)
                .count() as usize;
            assert_eq!(did.verification_methods.len() as usize, total_methods);
            assert_eq!(
                active_methods,
                1 + model
                    .methods
                    .values()
                    .map(|method| method.active)
                    .sum::<usize>()
            );
        }
    }

    fn event_count(&self) -> usize {
        self.env.events().all().len() as usize
    }
}

fn identity_operation() -> impl Strategy<Value = IdentityOp> {
    prop_oneof![
        (any::<u8>(), any::<u8>())
            .prop_map(|(subject, key_seed)| IdentityOp::CreateDid { subject, key_seed }),
        (any::<u8>(), any::<u8>()).prop_map(|(subject, service_slot)| IdentityOp::AddService {
            subject,
            service_slot
        }),
        (any::<u8>(), any::<u8>()).prop_map(|(subject, service_slot)| IdentityOp::RemoveService {
            subject,
            service_slot
        }),
        (any::<u8>(), any::<u8>()).prop_map(|(subject, method_slot)| {
            IdentityOp::AddVerificationMethod {
                subject,
                method_slot,
            }
        }),
        (any::<u8>(), any::<u8>(), any::<u8>()).prop_map(|(subject, method_slot, key_seed)| {
            IdentityOp::RotateKey {
                subject,
                method_slot,
                key_seed,
            }
        }),
        (any::<u8>(), any::<u8>()).prop_map(|(subject, method_slot)| {
            IdentityOp::RevokeVerificationMethod {
                subject,
                method_slot,
            }
        }),
        any::<u8>().prop_map(|subject| IdentityOp::DeactivateDid { subject }),
    ]
}

#[test]
fn identity_registry_regressions() {
    let cases = vec![
        RegressionCase {
            name: "did_service_lifecycle",
            operations: vec![
                IdentityOp::CreateDid {
                    subject: 0,
                    key_seed: 7,
                },
                IdentityOp::AddService {
                    subject: 0,
                    service_slot: 1,
                },
                IdentityOp::RemoveService {
                    subject: 0,
                    service_slot: 1,
                },
            ],
        },
        RegressionCase {
            name: "key_management_then_deactivate",
            operations: vec![
                IdentityOp::CreateDid {
                    subject: 1,
                    key_seed: 9,
                },
                IdentityOp::AddVerificationMethod {
                    subject: 1,
                    method_slot: 0,
                },
                IdentityOp::RotateKey {
                    subject: 1,
                    method_slot: 0,
                    key_seed: 11,
                },
                IdentityOp::RevokeVerificationMethod {
                    subject: 1,
                    method_slot: 0,
                },
                IdentityOp::DeactivateDid { subject: 1 },
            ],
        },
    ];

    run_regressions::<IdentityHarness, _>(&cases, IdentityHarness::new);
}

proptest! {
    #![proptest_config(ProptestConfig {
        cases: 24,
        max_shrink_iters: 0,
        .. ProptestConfig::default()
    })]

    #[test]
    fn fuzz_identity_registry_sequences(operations in vec(identity_operation(), 1..18)) {
        let mut harness = IdentityHarness::new();
        let report = execute_sequence(&mut harness, &operations);
        prop_assert!(report.is_ok(), "{report:?}");
    }
}
