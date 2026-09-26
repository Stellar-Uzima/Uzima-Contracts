# Development Guide

This document describes the pinned Soroban CLI setup and how to upgrade it safely.

---

## Soroban CLI Version Pinning

We pin `soroban-cli` to a known-good version to ensure consistency between local development and CI.

- **Pinned version:** compatible with `soroban-sdk = "21.7.7"`

The pin is applied in two places:

1. [`scripts/setup.sh`](../scripts/setup.sh)  
   Installs the CLI with a locked version.
2. [`.github/workflows/release.yml`](../.github/workflows/release.yml)  
   Ensures CI installs the same version.

---

## Checking Installed Version

Run:

```bash
soroban --version
```

---

## Repository Gates

`make check` and `make check-all` cover `fmt`, `lint`, `test` and the build.
They do **not** cover the repository's other correctness gates — event schemas,
generated artifacts, contract templates, dead code, or resource budgets — which
means it is easy to open a pull request that passes `make check` and still
fails CI.

Two aggregate targets compose those gates in a defined order:

| Command | What it runs |
| --- | --- |
| `make check-gates` | Every gate that needs no cargo build (8 gates) |
| `make check-everything` | All of the above, plus `fmt`, `lint`, dead code and budgets (12 gates) |
| `make gates-list` | Print the order without running anything |

The same targets are available through npm, so neither entrypoint can drift
from the other:

```bash
npm run check:gates       # == make check-gates
npm run check:everything  # == make check-everything
npm run gates:list        # == make gates-list
```

### The order, and why

The order lives in [`scripts/gates.json`](../scripts/gates.json) and is executed
by [`scripts/run-gates.mjs`](../scripts/run-gates.mjs). It is ordered
cheapest-and-most-selective first, so a bad change fails in seconds instead of
after a release build:

| Tier | Gates | Needs |
| --- | --- | --- |
| `structure` | `template`, `path-case` | nothing but `bash` / `python3` |
| `schema` | `events`, `interfaces`, `trace` | `npm ci` |
| `artifacts` | `compat`, `api-docs`, `drift` | `npm ci` |
| `toolchain` | `fmt`, `lint`, `deadcode` | cargo |
| `build` | `budgets` | `npm ci` **and** a release build |

`budgets` is last because it is the only gate that measures compiled output. It
reads `target/wasm32-unknown-unknown/release/*.wasm`, so run `make build-opt`
first; the runner refuses to run it otherwise rather than reporting a pass it
did not measure.

### Prerequisites

The gates do not run silently on missing dependencies. A missing prerequisite
stops the run with the command that fixes it:

- `npm ci` for any tier that parses sources with Node
- `make build-opt` before `budgets`
- `cargo` on `PATH` for the `toolchain` tier

### Running a single gate

```bash
node scripts/run-gates.mjs --only drift     # one gate, by id
node scripts/run-gates.mjs --list           # ids and commands, runs nothing
```

Gate ids are listed by `make gates-list`.

### `cargo test` is not part of the aggregate

`check-everything` deliberately stops short of the test suite. It is by far the
slowest gate, and `make check-all` already runs it together with the build
gates. Use both when you want the full picture.

### Adding or changing a gate

Edit [`scripts/gates.json`](../scripts/gates.json) and add a matching script in
[`package.json`](../package.json) if the gate is not already exposed there. Each
gate declares exactly one entrypoint (`npm` or `command`) and the requirements
it needs; the runner validates that manifest at startup, so a malformed entry
fails immediately instead of being skipped.

---

## Note: `make help` is not a complete index

`make help` is a hand-maintained short list and does not enumerate every target,
nor the group headings in this makefile. Use `make gates-list` for the gate
targets, and read the makefile for the rest.

