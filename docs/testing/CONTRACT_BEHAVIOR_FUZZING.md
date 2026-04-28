# Contract Behavior Fuzzing

This repo now includes a reusable behavior fuzzing harness in `contracts/contract_behavior_fuzzing`.

## Coverage

- `sut_token`
- `token_sale`
- `identity_registry`

## What the framework exercises

- Generated inputs for state-changing contract calls
- Stateful operation sequences
- Event progression checks after each step
- Crash detection with sequence replay context
- Regression suites for previously interesting sequences

## Running locally

```bash
scripts/run_contract_fuzz.sh
```

You can increase or reduce depth with `PROPTEST_CASES`:

```bash
PROPTEST_CASES=80 scripts/run_contract_fuzz.sh
```

## Extending the framework

1. Add a new harness that implements `BehaviorHarness`.
2. Model the contract state you want to keep invariant.
3. Return the expected event delta for each operation.
4. Add at least one regression case alongside the proptest sequence.
