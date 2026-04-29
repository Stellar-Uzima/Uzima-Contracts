use std::vec::Vec;

use contract_behavior_fuzzing::{
    execute_sequence, run_regressions, BehaviorHarness, OperationOutcome, RegressionCase,
};
use proptest::collection::vec;
use proptest::prelude::*;
use soroban_sdk::{
    testutils::{Address as _, Events as _, Ledger},
    token, Address, Env,
};
use token_sale::{TokenSaleContract, TokenSaleContractClient};

mod support;

#[derive(Clone, Debug)]
enum TokenSaleOp {
    SetTimestamp(u32),
    Pause,
    Unpause,
    Contribute { contributor: u8, amount: u16 },
    Finalize,
    ClaimTokens { contributor: u8 },
    ClaimRefund { contributor: u8 },
}

#[derive(Clone, Debug, Default)]
struct ContributionModel {
    amount: u128,
    tokens_allocated: u128,
    claimed: bool,
}

struct TokenSaleHarness {
    env: Env,
    client: TokenSaleContractClient<'static>,
    payment_token: token::Client<'static>,
    contributors: Vec<Address>,
    total_raised: u128,
    phase_sold_tokens: u128,
    paused: bool,
    finalized: bool,
    refunds_enabled: bool,
    contributions: Vec<ContributionModel>,
}

impl TokenSaleHarness {
    const PHASE_START: u64 = 1_000;
    const PHASE_END: u64 = 2_000;
    const PRICE_PER_TOKEN: u128 = 100;
    const MAX_TOKENS: u128 = 50_000_000;
    const PER_ADDRESS_CAP: u128 = 2_000;
    const SOFT_CAP: u128 = 500;
    const HARD_CAP: u128 = 5_000;

    fn new() -> Self {
        let env = Env::default();
        env.mock_all_auths();

        let owner = Address::generate(&env);
        let treasury = Address::generate(&env);
        let contributors = (0..3).map(|_| Address::generate(&env)).collect::<Vec<_>>();

        let sut_contract = env
            .register_stellar_asset_contract_v2(owner.clone())
            .address();
        let payment_contract = env
            .register_stellar_asset_contract_v2(owner.clone())
            .address();
        let _sut_token = token::Client::new(&env, &sut_contract);
        let sut_admin = token::StellarAssetClient::new(&env, &sut_contract);
        let payment_token = token::Client::new(&env, &payment_contract);
        let payment_admin = token::StellarAssetClient::new(&env, &payment_contract);

        let sale_contract = env.register_contract(None, TokenSaleContract);
        let client = TokenSaleContractClient::new(&env, &sale_contract);
        client.initialize(
            &owner,
            &sut_contract,
            &treasury,
            &Self::SOFT_CAP,
            &Self::HARD_CAP,
        );
        client.add_supported_token(&payment_contract);
        client.add_sale_phase(
            &Self::PHASE_START,
            &Self::PHASE_END,
            &Self::PRICE_PER_TOKEN,
            &Self::MAX_TOKENS,
            &Self::PER_ADDRESS_CAP,
        );

        sut_admin.mint(&sale_contract, &(Self::MAX_TOKENS as i128));
        payment_admin.mint(&sale_contract, &10_000);
        for contributor in &contributors {
            payment_admin.mint(contributor, &5_000);
        }

        env.ledger().with_mut(|ledger| {
            ledger.timestamp = Self::PHASE_START;
        });

        Self {
            env,
            client,
            payment_token,
            contributors,
            total_raised: 0,
            phase_sold_tokens: 0,
            paused: false,
            finalized: false,
            refunds_enabled: false,
            contributions: vec![ContributionModel::default(); 3],
        }
    }

    fn contributor(&self, index: u8) -> &Address {
        &self.contributors[index as usize % self.contributors.len()]
    }

    fn amount(amount: u16) -> u128 {
        u128::from(amount % 1_000)
    }
}

impl BehaviorHarness for TokenSaleHarness {
    type Operation = TokenSaleOp;

    fn apply(&mut self, operation: &Self::Operation) -> OperationOutcome {
        match operation {
            TokenSaleOp::SetTimestamp(timestamp) => {
                let timestamp = 800 + u64::from(*timestamp % 1_600);
                self.env.ledger().with_mut(|ledger| {
                    ledger.timestamp = timestamp;
                });
                OperationOutcome::new(0)
            },
            TokenSaleOp::Pause => {
                self.client.pause_sale();
                self.paused = true;
                OperationOutcome::new(1)
            },
            TokenSaleOp::Unpause => {
                self.client.unpause_sale();
                self.paused = false;
                OperationOutcome::new(1)
            },
            TokenSaleOp::Contribute {
                contributor,
                amount,
            } => {
                let contributor_index = *contributor as usize % self.contributors.len();
                let contributor = self.contributor(*contributor).clone();
                let amount = Self::amount(*amount);
                let timestamp = self.env.ledger().timestamp();
                let tokens = (amount * 1_000_000) / Self::PRICE_PER_TOKEN;
                let contribution = &self.contributions[contributor_index];
                let current_balance = self.payment_token.balance(&contributor);
                let success = !self.paused
                    && !self.finalized
                    && timestamp >= Self::PHASE_START
                    && timestamp <= Self::PHASE_END
                    && amount + contribution.amount <= Self::PER_ADDRESS_CAP
                    && self.phase_sold_tokens + tokens <= Self::MAX_TOKENS
                    && i128::try_from(amount).unwrap() <= current_balance;

                let result = self.client.try_contribute(
                    &contributor,
                    &0,
                    &self.payment_token.address,
                    &amount,
                );

                if success {
                    assert!(result.is_ok());
                    let contribution = &mut self.contributions[contributor_index];
                    contribution.amount += amount;
                    contribution.tokens_allocated += tokens;
                    self.total_raised += amount;
                    self.phase_sold_tokens += tokens;
                } else {
                    assert!(result.is_err());
                }

                OperationOutcome::new(if success { 2 } else { 0 })
            },
            TokenSaleOp::Finalize => {
                let success = !self.finalized;
                let result = self.client.try_finalize_sale();

                if success {
                    assert!(result.is_ok());
                    self.finalized = true;
                    self.refunds_enabled = self.total_raised < Self::SOFT_CAP;
                } else {
                    assert!(result.is_err());
                }

                OperationOutcome::new(usize::from(success))
            },
            TokenSaleOp::ClaimTokens { contributor } => {
                let contributor_index = *contributor as usize % self.contributors.len();
                let contributor = self.contributor(*contributor).clone();
                let contribution = &self.contributions[contributor_index];
                let call_succeeds = self.finalized
                    && !self.refunds_enabled
                    && (contribution.amount == 0
                        || (!contribution.claimed && contribution.tokens_allocated > 0));
                let emits_event = self.finalized
                    && !self.refunds_enabled
                    && contribution.amount > 0
                    && !contribution.claimed
                    && contribution.tokens_allocated > 0;

                let result = self.client.try_claim_tokens(&contributor);

                if call_succeeds {
                    assert!(result.is_ok());
                    if emits_event {
                        self.contributions[contributor_index].claimed = true;
                    }
                } else {
                    assert!(result.is_err());
                }

                OperationOutcome::new(if emits_event { 2 } else { 0 })
            },
            TokenSaleOp::ClaimRefund { contributor } => {
                let contributor_index = *contributor as usize % self.contributors.len();
                let contributor = self.contributor(*contributor).clone();
                let contribution = &self.contributions[contributor_index];
                let call_succeeds = self.finalized
                    && self.refunds_enabled
                    && (contribution.amount == 0
                        || (!contribution.claimed && contribution.amount > 0));
                let emits_event = self.finalized
                    && self.refunds_enabled
                    && contribution.amount > 0
                    && !contribution.claimed;

                let result = self
                    .client
                    .try_claim_refund(&contributor, &self.payment_token.address);

                if call_succeeds {
                    assert!(result.is_ok());
                    if emits_event {
                        self.contributions[contributor_index].claimed = true;
                    }
                } else {
                    assert!(result.is_err());
                }

                OperationOutcome::new(if emits_event { 2 } else { 0 })
            },
        }
    }

    fn assert_invariants(&self) {
        let config = self.client.get_config();
        assert_eq!(config.is_finalized, self.finalized);
        assert_eq!(config.refunds_enabled, self.refunds_enabled);
        assert_eq!(self.client.get_total_raised(), self.total_raised);

        let phase = self.client.get_sale_phase(&0).unwrap();
        assert_eq!(phase.sold_tokens, self.phase_sold_tokens);
        assert!(phase.sold_tokens <= phase.max_tokens);

        let total_contribution_amount = self
            .contributions
            .iter()
            .map(|contribution| contribution.amount)
            .sum::<u128>();
        assert_eq!(total_contribution_amount, self.total_raised);

        for (index, contributor) in self.contributors.iter().enumerate() {
            let expected = &self.contributions[index];
            let actual = self.client.get_contribution(contributor);
            if expected.amount == 0 {
                assert!(actual.is_none());
            } else {
                let actual = actual.expect("contribution must exist");
                assert_eq!(actual.amount, expected.amount);
                assert_eq!(actual.tokens_allocated, expected.tokens_allocated);
                assert_eq!(actual.claimed, expected.claimed);
            }
        }
    }

    fn event_count(&self) -> usize {
        self.env.events().all().len() as usize
    }
}

fn token_sale_operation() -> impl Strategy<Value = TokenSaleOp> {
    prop_oneof![
        any::<u32>().prop_map(TokenSaleOp::SetTimestamp),
        Just(TokenSaleOp::Pause),
        Just(TokenSaleOp::Unpause),
        (any::<u8>(), 0u16..1_200u16).prop_map(|(contributor, amount)| TokenSaleOp::Contribute {
            contributor,
            amount
        }),
        Just(TokenSaleOp::Finalize),
        any::<u8>().prop_map(|contributor| TokenSaleOp::ClaimTokens { contributor }),
        any::<u8>().prop_map(|contributor| TokenSaleOp::ClaimRefund { contributor }),
    ]
}

#[test]
fn token_sale_regressions() {
    let cases = vec![
        RegressionCase {
            name: "successful_sale_and_claim",
            operations: vec![
                TokenSaleOp::SetTimestamp(1_200),
                TokenSaleOp::Contribute {
                    contributor: 0,
                    amount: 300,
                },
                TokenSaleOp::Contribute {
                    contributor: 1,
                    amount: 250,
                },
                TokenSaleOp::Finalize,
                TokenSaleOp::ClaimTokens { contributor: 0 },
                TokenSaleOp::ClaimTokens { contributor: 1 },
            ],
        },
        RegressionCase {
            name: "failed_sale_refund",
            operations: vec![
                TokenSaleOp::SetTimestamp(1_100),
                TokenSaleOp::Contribute {
                    contributor: 0,
                    amount: 100,
                },
                TokenSaleOp::Finalize,
                TokenSaleOp::ClaimRefund { contributor: 0 },
            ],
        },
    ];

    run_regressions::<TokenSaleHarness, _>(&cases, TokenSaleHarness::new);
}

proptest! {
    #![proptest_config(ProptestConfig {
        cases: 40,
        max_shrink_iters: 0,
        .. ProptestConfig::default()
    })]

    #[test]
    fn fuzz_token_sale_sequences(operations in vec(token_sale_operation(), 1..36)) {
        let mut harness = TokenSaleHarness::new();
        let report = execute_sequence(&mut harness, &operations);
        prop_assert!(report.is_ok(), "{report:?}");
    }
}
