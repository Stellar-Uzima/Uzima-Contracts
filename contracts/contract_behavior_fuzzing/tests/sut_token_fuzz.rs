use std::collections::{BTreeMap, BTreeSet};
use std::vec::Vec;

use contract_behavior_fuzzing::{
    execute_sequence, run_regressions, BehaviorHarness, OperationOutcome, RegressionCase,
};
use proptest::collection::vec;
use proptest::prelude::*;
use soroban_sdk::{
    testutils::{Address as _, Events as _},
    Address, Env, String,
};
use sut_token::{Error, SutToken, SutTokenClient};

mod support;

#[derive(Clone, Debug)]
enum SutTokenOp {
    Initialize {
        supply_cap: u16,
        decimals: u8,
    },
    Mint {
        minter: u8,
        to: u8,
        amount: u16,
    },
    Burn {
        minter: u8,
        from: u8,
        amount: u16,
    },
    Transfer {
        from: u8,
        to: u8,
        amount: u16,
    },
    Approve {
        owner: u8,
        spender: u8,
        amount: u16,
    },
    TransferFrom {
        spender: u8,
        owner: u8,
        to: u8,
        amount: u16,
    },
    AddMinter {
        minter: u8,
    },
    RemoveMinter {
        minter: u8,
    },
    Snapshot,
}

#[derive(Default)]
struct SutTokenModel {
    initialized: bool,
    supply_cap: i128,
    total_supply: i128,
    decimals: u32,
    balances: Vec<i128>,
    allowances: BTreeMap<(usize, usize), i128>,
    minters: BTreeSet<usize>,
}

struct SutTokenHarness {
    env: Env,
    client: SutTokenClient<'static>,
    accounts: Vec<Address>,
    model: SutTokenModel,
}

impl SutTokenHarness {
    fn new() -> Self {
        let env = Env::default();
        env.mock_all_auths();

        let contract_id = env.register_contract(None, SutToken);
        let client = SutTokenClient::new(&env, &contract_id);
        let accounts = (0..4).map(|_| Address::generate(&env)).collect::<Vec<_>>();

        Self {
            env,
            client,
            accounts,
            model: SutTokenModel {
                balances: vec![0; 4],
                ..SutTokenModel::default()
            },
        }
    }

    fn account(&self, index: u8) -> &Address {
        &self.accounts[index as usize % self.accounts.len()]
    }

    fn amount(amount: u16) -> i128 {
        i128::from(amount % 250)
    }

    fn initialize_success(&self, supply_cap: i128) -> bool {
        !self.model.initialized && supply_cap > 0
    }

    fn expect_contract_error<T>(
        &self,
        result: Result<T, Result<Error, soroban_sdk::InvokeError>>,
    ) -> bool {
        result.is_ok()
    }
}

impl BehaviorHarness for SutTokenHarness {
    type Operation = SutTokenOp;

    fn apply(&mut self, operation: &Self::Operation) -> OperationOutcome {
        match operation {
            SutTokenOp::Initialize {
                supply_cap,
                decimals,
            } => {
                let cap = i128::from(*supply_cap);
                let result = self.client.try_initialize(
                    self.account(0),
                    &String::from_str(&self.env, "Fuzz Token"),
                    &String::from_str(&self.env, "FUZZ"),
                    &u32::from(*decimals % 18),
                    &cap,
                );

                if self.initialize_success(cap) {
                    assert!(self.expect_contract_error(result));
                    self.model.initialized = true;
                    self.model.supply_cap = cap;
                    self.model.decimals = u32::from(*decimals % 18);
                    self.model.minters.insert(0);
                } else {
                    assert!(result.is_err());
                }

                OperationOutcome::new(0)
            },
            SutTokenOp::Mint { minter, to, amount } => {
                let minter_index = *minter as usize % self.accounts.len();
                let to_index = *to as usize % self.accounts.len();
                let amount = Self::amount(*amount);
                let success = self.model.initialized
                    && amount > 0
                    && self.model.minters.contains(&minter_index)
                    && self.model.total_supply + amount <= self.model.supply_cap;

                let result =
                    self.client
                        .try_mint(self.account(*minter), self.account(*to), &amount);

                if success {
                    assert!(self.expect_contract_error(result));
                    self.model.total_supply += amount;
                    self.model.balances[to_index] += amount;
                } else {
                    assert!(result.is_err());
                }

                OperationOutcome::new(usize::from(success))
            },
            SutTokenOp::Burn {
                minter,
                from,
                amount,
            } => {
                let minter_index = *minter as usize % self.accounts.len();
                let from_index = *from as usize % self.accounts.len();
                let amount = Self::amount(*amount);
                let success = self.model.initialized
                    && amount > 0
                    && self.model.minters.contains(&minter_index)
                    && self.model.balances[from_index] >= amount;

                let result =
                    self.client
                        .try_burn(self.account(*minter), self.account(*from), &amount);

                if success {
                    assert!(self.expect_contract_error(result));
                    self.model.total_supply -= amount;
                    self.model.balances[from_index] -= amount;
                } else {
                    assert!(result.is_err());
                }

                OperationOutcome::new(usize::from(success))
            },
            SutTokenOp::Transfer { from, to, amount } => {
                let from_index = *from as usize % self.accounts.len();
                let to_index = *to as usize % self.accounts.len();
                let amount = Self::amount(*amount);
                let success = amount == 0 || self.model.balances[from_index] >= amount;

                let result =
                    self.client
                        .try_transfer(self.account(*from), self.account(*to), &amount);

                if success {
                    assert!(self.expect_contract_error(result));
                    if amount > 0 {
                        self.model.balances[from_index] -= amount;
                        self.model.balances[to_index] += amount;
                    }
                } else {
                    assert!(result.is_err());
                }

                OperationOutcome::new(usize::from(success && amount > 0))
            },
            SutTokenOp::Approve {
                owner,
                spender,
                amount,
            } => {
                let owner_index = *owner as usize % self.accounts.len();
                let spender_index = *spender as usize % self.accounts.len();
                let amount = Self::amount(*amount);
                let success = true;

                let result =
                    self.client
                        .try_approve(self.account(*owner), self.account(*spender), &amount);

                if success {
                    assert!(self.expect_contract_error(result));
                    if amount == 0 {
                        self.model.allowances.remove(&(owner_index, spender_index));
                    } else {
                        self.model
                            .allowances
                            .insert((owner_index, spender_index), amount);
                    }
                } else {
                    assert!(result.is_err());
                }

                OperationOutcome::new(usize::from(success))
            },
            SutTokenOp::TransferFrom {
                spender,
                owner,
                to,
                amount,
            } => {
                let spender_index = *spender as usize % self.accounts.len();
                let owner_index = *owner as usize % self.accounts.len();
                let to_index = *to as usize % self.accounts.len();
                let amount = Self::amount(*amount);
                let allowance = *self
                    .model
                    .allowances
                    .get(&(owner_index, spender_index))
                    .unwrap_or(&0);
                let success = amount == 0
                    || (self.model.balances[owner_index] >= amount && allowance >= amount);

                let result = self.client.try_transfer_from(
                    self.account(*spender),
                    self.account(*owner),
                    self.account(*to),
                    &amount,
                );

                if success {
                    assert!(self.expect_contract_error(result));
                    if amount > 0 {
                        self.model.balances[owner_index] -= amount;
                        self.model.balances[to_index] += amount;
                        let remaining = allowance - amount;
                        if remaining == 0 {
                            self.model.allowances.remove(&(owner_index, spender_index));
                        } else {
                            self.model
                                .allowances
                                .insert((owner_index, spender_index), remaining);
                        }
                    }
                } else {
                    assert!(result.is_err());
                }

                OperationOutcome::new(usize::from(success && amount > 0))
            },
            SutTokenOp::AddMinter { minter } => {
                let minter_index = *minter as usize % self.accounts.len();
                let success = self.model.initialized;
                let result = self.client.try_add_minter(self.account(*minter));

                if success {
                    assert!(self.expect_contract_error(result));
                    self.model.minters.insert(minter_index);
                } else {
                    assert!(result.is_err());
                }

                OperationOutcome::new(0)
            },
            SutTokenOp::RemoveMinter { minter } => {
                let minter_index = *minter as usize % self.accounts.len();
                let success = self.model.initialized;
                let result = self.client.try_remove_minter(self.account(*minter));

                if success {
                    assert!(self.expect_contract_error(result));
                    self.model.minters.remove(&minter_index);
                } else {
                    assert!(result.is_err());
                }

                OperationOutcome::new(0)
            },
            SutTokenOp::Snapshot => {
                let success = self.model.initialized;
                let result = self.client.try_snapshot();

                if success {
                    assert!(self.expect_contract_error(result));
                } else {
                    assert!(result.is_err());
                }

                OperationOutcome::new(usize::from(success))
            },
        }
    }

    fn assert_invariants(&self) {
        if !self.model.initialized {
            for (index, account) in self.accounts.iter().enumerate() {
                assert_eq!(self.client.balance_of(account), self.model.balances[index]);
                assert_eq!(self.client.is_minter(account), false);
            }

            for owner_index in 0..self.accounts.len() {
                for spender_index in 0..self.accounts.len() {
                    let expected = *self
                        .model
                        .allowances
                        .get(&(owner_index, spender_index))
                        .unwrap_or(&0);
                    assert_eq!(
                        self.client
                            .allowance(&self.accounts[owner_index], &self.accounts[spender_index]),
                        expected
                    );
                }
            }
            return;
        }

        let onchain_total_supply = self.client.total_supply();
        assert_eq!(onchain_total_supply, self.model.total_supply);
        assert!(onchain_total_supply <= self.model.supply_cap);
        assert_eq!(self.client.supply_cap(), self.model.supply_cap);
        assert_eq!(self.client.decimals(), self.model.decimals);

        let summed_balances = self
            .accounts
            .iter()
            .map(|account| self.client.balance_of(account))
            .sum::<i128>();
        assert_eq!(summed_balances, onchain_total_supply);

        for (index, account) in self.accounts.iter().enumerate() {
            assert_eq!(self.client.balance_of(account), self.model.balances[index]);
            assert_eq!(
                self.client.is_minter(account),
                self.model.minters.contains(&index)
            );
        }

        for owner_index in 0..self.accounts.len() {
            for spender_index in 0..self.accounts.len() {
                let expected = *self
                    .model
                    .allowances
                    .get(&(owner_index, spender_index))
                    .unwrap_or(&0);
                assert_eq!(
                    self.client
                        .allowance(&self.accounts[owner_index], &self.accounts[spender_index]),
                    expected
                );
            }
        }
    }

    fn event_count(&self) -> usize {
        self.env.events().all().len() as usize
    }
}

fn sut_token_operation() -> impl Strategy<Value = SutTokenOp> {
    prop_oneof![
        (1u16..600u16, any::<u8>()).prop_map(|(supply_cap, decimals)| SutTokenOp::Initialize {
            supply_cap,
            decimals
        }),
        (any::<u8>(), any::<u8>(), 0u16..300u16)
            .prop_map(|(minter, to, amount)| SutTokenOp::Mint { minter, to, amount }),
        (any::<u8>(), any::<u8>(), 0u16..300u16).prop_map(|(minter, from, amount)| {
            SutTokenOp::Burn {
                minter,
                from,
                amount,
            }
        }),
        (any::<u8>(), any::<u8>(), 0u16..300u16)
            .prop_map(|(from, to, amount)| SutTokenOp::Transfer { from, to, amount }),
        (any::<u8>(), any::<u8>(), 0u16..300u16).prop_map(|(owner, spender, amount)| {
            SutTokenOp::Approve {
                owner,
                spender,
                amount,
            }
        }),
        (any::<u8>(), any::<u8>(), any::<u8>(), 0u16..300u16).prop_map(
            |(spender, owner, to, amount)| SutTokenOp::TransferFrom {
                spender,
                owner,
                to,
                amount,
            }
        ),
        any::<u8>().prop_map(|minter| SutTokenOp::AddMinter { minter }),
        any::<u8>().prop_map(|minter| SutTokenOp::RemoveMinter { minter }),
        Just(SutTokenOp::Snapshot),
    ]
}

#[test]
fn sut_token_regressions() {
    let cases = vec![
        RegressionCase {
            name: "mint-transfer-burn",
            operations: vec![
                SutTokenOp::Initialize {
                    supply_cap: 500,
                    decimals: 6,
                },
                SutTokenOp::Mint {
                    minter: 0,
                    to: 1,
                    amount: 100,
                },
                SutTokenOp::Transfer {
                    from: 1,
                    to: 2,
                    amount: 40,
                },
                SutTokenOp::Burn {
                    minter: 0,
                    from: 2,
                    amount: 10,
                },
            ],
        },
        RegressionCase {
            name: "allowance-roundtrip",
            operations: vec![
                SutTokenOp::Initialize {
                    supply_cap: 500,
                    decimals: 6,
                },
                SutTokenOp::Mint {
                    minter: 0,
                    to: 1,
                    amount: 120,
                },
                SutTokenOp::Approve {
                    owner: 1,
                    spender: 2,
                    amount: 60,
                },
                SutTokenOp::TransferFrom {
                    spender: 2,
                    owner: 1,
                    to: 3,
                    amount: 35,
                },
                SutTokenOp::Snapshot,
            ],
        },
    ];

    run_regressions::<SutTokenHarness, _>(&cases, SutTokenHarness::new);
}

proptest! {
    #![proptest_config(ProptestConfig {
        cases: 24,
        max_shrink_iters: 0,
        .. ProptestConfig::default()
    })]

    #[test]
    fn fuzz_sut_token_behaviors(operations in vec(sut_token_operation(), 1..20)) {
        let mut harness = SutTokenHarness::new();
        let report = execute_sequence(&mut harness, &operations);
        prop_assert!(report.is_ok(), "{report:?}");
    }
}
