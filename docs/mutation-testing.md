# Mutation Testing with `cargo-mutants`

## Setup

Install the tool:
```sh
cargo install cargo-mutants
```

## Running mutation tests

Run against the three core contracts:
```sh
cargo mutants -p patient_consent_management -p identity_registry -p medical_records
```

Results are written to `mutants.out/`. A passing mutation test means the mutant
was **caught** by the test suite (good). An **uncaught** mutant indicates a gap.

## Interpreting results

| Outcome | Meaning |
|---|---|
| `caught` | Test suite detected the mutation ✅ |
| `missed` | No test caught this mutation — add a test |
| `timeout` | Mutation caused an infinite loop — likely caught |
| `unviable` | Mutation did not compile — skip |

## CI integration

Add to `.github/workflows/mutation.yml` (run on schedule, not every PR):
```yaml
- name: Install cargo-mutants
  run: cargo install cargo-mutants
- name: Run mutation tests
  run: cargo mutants -p patient_consent_management --timeout 60
```

## Baseline targets

- `patient_consent_management` — focus on `grant_consent`, `revoke_consent`
- `identity_registry` — focus on `register_did`, `verify_did`
- `medical_records` — focus on access-control paths