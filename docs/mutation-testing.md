# Mutation Testing with `cargo-mutants`

## Overview

Mutation testing measures the **effectiveness** of a test suite, not its
size. A mutation tool (`cargo-mutants`) applies small changes ("mutants") to
the source code — flipping `&&` to `||`, removing boundary checks, swapping
constants — and re-runs the test suite. Each mutant is either:

| Outcome | Meaning |
|---|---|
| `caught`   | Test suite detected the mutation ✅ |
| `missed`   | No test caught this mutation — add a test |
| `timeout`  | Mutation caused an infinite loop or exceeded the per-mutant timeout — counted as caught (the test did not pass within budget) |
| `unviable` | Mutation produced a compile error; excluded from the score denominator |

The **mutation score** for a contract is

```
score = (caught + timeout) / (caught + missed + timeout) * 100
```

`unviable` mutants are excluded from the denominator.

The score target for this project is **> 85 %** on at least two of the
three core contracts (`patient_consent_management`, `identity_registry`,
`medical_records`).

## Setup

Install `cargo-mutants` at the **same version as CI** (see
[Updating the pinned tool version](#updating-the-pinned-tool-version)
below for the procedure). The current pin is `25.3.1`:

```sh
cargo install cargo-mutants --version 25.3.1 --locked
```

## Running mutation tests locally

Use `--dir` so cargo-mutants always uses a package-local `Cargo.toml` as
the root. This works the same way for workspace members and for excluded
packages such as `medical_records`:

```sh
# Patient consent management
cargo mutants --dir contracts/patient_consent_management --timeout 120 --jobs 4

# Identity registry
cargo mutants --dir contracts/identity_registry       --timeout 120 --jobs 4

# Medical records (workspace-excluded; --dir still works)
cargo mutants --dir contracts/medical_records         --timeout 120 --jobs 4
```

Results are written to `<dir>/mutants.out/`. The full HTML report is at
`mutants.out/index.html` and the live mutant list lives in
`mutants.out/mutants.json`.

### Useful cargo-mutants flags

| Flag                       | Effect                                                                     |
|---|---|
| `--timeout <SECS>`         | Per-mutant test timeout (default 30). Use ≥ 120 for contracts with slow tests. |
| `--jobs <N>`               | Parallel mutant evaluation. 4 ≈ GH-hosted runner core count.               |
| `--output <DIR>`           | Override output directory (default `./mutants.out`).                       |
| `--baseline <FILE>`        | Treat mutants listed in `<FILE>` as caught (use instead of the `comm`-based diff in CI). |
| `--check`                  | Job finishes when one uncaught mutant is found (faster local iteration).  |
| `--re <REGEX>`             | Restrict mutations to paths matching the regex.                            |
| `--file <PATH>`            | Restrict mutations to one file.                                            |

### Recommended focus areas for this repo

Mutation score is most lifted by adding tests for the existing public
functions below (these are also the highest-risk paths in a healthcare
context):

- `patient_consent_management`:
  - `grant_consent` / `grant_consent_with_expiry`
  - `revoke_consent`
  - `check_consent` (especially the expiry boundary)
  - `cleanup_expired_consents`
- `identity_registry`:
  - `create_did` (note: the API uses `create_did`, not `register_did`)
  - `verify_did_authorization`
  - `update_did` / `deactivate_did`
  - verifier membership management
- `medical_records`:
  - `add_record` / `get_record` / authorization paths
  - `set_rate_limit_config` / rate-limit enforcement
  - metadata update / history export paths

## CI integration

The weekly mutation-testing workflow lives at
**[`.github/workflows/mutation.yml`](../.github/workflows/mutation.yml)**.

### Triggers

- **Scheduled**: weekly, Sundays at 00:00 UTC (`cron: '0 0 * * 0'`).
- **Manual dispatch**: `workflow_dispatch` runs the full three-contract
  matrix on demand.

### What each matrix job does

For each contract (`patient_consent_management`, `identity_registry`,
`medical_records`) in parallel:

1. Install Rust stable + cache `~/.cargo` via `Swatinem/rust-cache@v2`.
2. `cargo install cargo-mutants --locked`.
3. `cd contracts/<contract>; cargo mutants --dir . --timeout 120 --jobs 4`.
4. Compute caught / missed / timeout / unviable counts and the score.
5. Diff the current run's `missed.txt` against the committed baseline at
   [`docs/baselines/<contract>-missed.txt`](baselines/).
6. Fail the job if any **new** uncaught mutant appears (`comm -13`).
7. Always upload `mutants.out/` as an artifact named
   `mutants-out-<contract>` (30-day retention).

`fail-fast: false` on the matrix means a regression in one contract does
not cancel the score reports for the others.

### Per-job timeout

Each matrix job has a 360-minute wall-clock timeout (the maximum on a
GitHub-hosted runner). With `--jobs 4` the two smaller contracts finish
well under this budget. `medical_records` is the largest; see
[Known gaps and scaling](#known-gaps-and-scaling) below for mitigation.

### Failure condition — "new uncaught mutants"

A committed baseline file in `docs/baselines/` lists every currently-known
missed mutant (one per line, in the format `cargo-mutants` emits). The
comparison step runs:

```sh
comm -13 \
  <(sort docs/baselines/<contract>-missed.txt) \
  <(sort contracts/<contract>/mutants.out/missed.txt)
```

If the result is non-empty, the job exits 1 and emits each new miss as a
`::error::` annotation in the workflow log. This is the strict failure
condition called out in issue #833.

### Bootstrap: capturing the initial baseline

The committed baseline files in `docs/baselines/` are intentionally
**empty placeholders** on first merge. That means the first scheduled
run after this PR lands will:

1. Run cargo-mutants against each contract.
2. Compute scores (likely some contracts will score > 85 % on first try,
   others will surface as < 85 % — see goal below).
3. **Fail the comparison step** because every miss is "new" against an
   empty baseline.

That is the correct outcome of the bootstrap. A maintainer must then:

1. Download the `mutants-out-<contract>` artifact from that workflow run.
2. Review `mutants.out/missed.txt`. For each miss:
   - **Real gap**: add a test, re-run, the mutant is now `caught`.
   - **Acceptable gap** (e.g. defensive code that cannot be exercised
     without contract corruption): list it in the baseline file with a
     trailing `# rationale` comment.
3. Copy `mutants.out/missed.txt` over `docs/baselines/<contract>-missed.txt`
   (or curate and commit a subset), then push.

After step 3, future runs only fail when a genuinely new mutant is
introduced.

### Updating an existing baseline

When you intentionally add code without a corresponding test (and that's
the right call), update the relevant baseline file in the same PR so the
scheduled run stays green. This is also the right place to record
defensive checks that are exercised only under fault injection.

## Goal: ≥ 2/3 contracts > 85 % score

The acceptance criterion for issue #833 is "at least two contracts pass
with a mutation score > 85 %". The first scheduled run will produce a
real measurement for each contract:

- `patient_consent_management` has ~30 tests in `src/test.rs` covering
  grant/revoke with and without expiry, audit/error-code stability, and
  time-boundary edge cases. It is most likely to land above 85 % on the
  first run.
- `identity_registry` has inline `#[cfg(test)] mod tests` plus
  `comprehensive_tests.rs`. It is also expected to land close to or
  above 85 %.
- `medical_records` is the largest surface area and is expected to need
  either an expanded test suite or scope-restricted mutation runs (see
  next section). The first run will not necessarily hit 85 % here.

If `medical_records` falls below 85 %, that is acceptable as long as at
least two of the three targets meet the threshold. The gap is documented
in the workflow summary step and should be tracked as a follow-up.

## Known gaps and scaling

- **medical_records workspace exclusion.** `medical_records` is in the
  workspace `exclude` list in the root `Cargo.toml`. The workflow handles
  this by passing `--dir contracts/medical_records` so cargo-mutants uses
  that package's own `Cargo.toml` as the root.
- **medical_records mutation surface.** With ~6,300 lines of source in
  `src/lib.rs` plus helpers, fully mutating the contract may approach
  the 360-minute single-job ceiling. Mitigation if needed: pass
  `--re 'src/(access_control|lib\.rs)'` on a future iteration to focus
  on the highest-risk paths. This decision is intentionally **not** taken
  in the initial workflow to avoid hiding regressions elsewhere.
- **Bootstrap-baseline burn-in.** The first scheduled run will fail by
  design (see [Bootstrap](#bootstrap-capturing-the-initial-baseline)). Do
  not treat that as a regression.
- **Local cache.** cargo-mutants compiles each mutated crate from
  scratch, so local runs benefit from `Swatinem/rust-cache@v2` (used in
  CI). Locally you can also pre-warm with
  `cargo test -p <contract>` before invoking `cargo mutants`.

## Verifying mutations locally before pushing

Before raising a PR that touches one of the three contracts, run:

```sh
cargo mutants --dir contracts/<contract> --timeout 120 --jobs 4
cat contracts/<contract>/mutants.out/missed.txt
```

Any new entry versus the committed baseline is a regression you owe a
test for. The CI workflow will not catch you locally — it will only fail
the scheduled run, so doing this check before pushing keeps the project
tree clean.

## Updating the pinned tool version

`cargo-mutants` is pinned in `.github/workflows/mutation.yml` for
reproducibility. cargo-mutants' mutant set evolves with every release,
so an un-pinned upgrade would silently churn `missed.txt` and produce
spurious "new mutant" failures relative to the committed baseline.

When you intend to bump the pin, run the tool locally across all three
contracts once with the new version, regenerate
`docs/baselines/<contract>-missed.txt` from the freshly captured
`mutants.out/missed.txt`, commit both the workflow and the baseline
files together, and submit them as a single PR so the diff is reviewable
in one place. Do not update one without the other.

## Updating this document

When changing the workflow, please keep the [CI integration](#ci-integration)
section in sync with `.github/workflows/mutation.yml`, and update the
[focus areas](#recommended-focus-areas-for-this-repo) section whenever
the public surface area of the three core contracts changes.
