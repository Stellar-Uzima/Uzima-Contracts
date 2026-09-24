# Gas/Optimization Pass for On-Ramped Contracts

Tracks: #1577 "Gas/optimization pass for on-ramped contracts"

## Current state

`contract_optimizer` (with its `complexity` subcommand, `#481`) scores
every contract on cyclomatic complexity, data structures, external calls,
state transitions, and permission checks — see
`docs/CONTRACT_COMPLEXITY_SCORING.md`. `scripts/check_complexity.sh`
wraps it into a pass/warn/fail gate, but **wasn't invoked by any CI
workflow**, and no `dashboard/data/complexity_report.json` was committed
— so there was no catalog to prioritize a gas-optimization pass from.

## What landed here

- `.github/workflows/contract-complexity-gate.yml` — runs
  `scripts/check_complexity.sh` on every push/PR touching `contracts/**`
  or `contract_optimizer/**`, failing the job when any contract exceeds
  its fail threshold (the tool's own existing exit-code behavior — nothing
  here swallows it), and uploads `dashboard/data/complexity_report.json` +
  the PR-comment file as build artifacts.

This makes the complexity report (and therefore the gas-optimization
priority list the issue asks for) something generated fresh on every CI
run going forward, rather than a document that would go stale the moment
someone adds a contract — the CI artifact is the catalog.

## Why the actual optimization work isn't in this change

Generating the real priority list requires running
`cargo run -p contract_optimizer --features cli -- check-complexity`,
which needs a full compile — not run here. More importantly, the
optimization work itself (`contract_optimizer`'s recommendations engine
covers gas, storage, algorithm, batching, and parallelization changes)
means editing contract logic per-contract based on that report, which is
exactly the kind of change that needs to compile and pass its existing
tests to be trustworthy — guessing at it without being able to verify
would risk introducing real bugs into live contract logic, not just a
broken CI job.

## Follow-ups

1. Run the new CI job (or `./scripts/check_complexity.sh` locally) once,
   commit the resulting `dashboard/data/complexity_report.json` as the
   initial baseline, and read off the actual priority-by-score list.
2. File a follow-up issue per contract that fails/warns, with an owner,
   rather than one large "optimize everything" PR.
