# Security Telemetry Conformance — Gap Analysis

Tracks: #1571 "Standardize security telemetry conformance for newly
on-ramped contracts"

## Policy (what should happen)

`docs/TELEMETRY_SCHEMA.md` defines a versioned `TelemetryEvent` schema
(topic `(TEL, <type_symbol>)`) that contracts are expected to emit for
security-relevant events (`AUTH_FAIL`, `AUTHZ_FAIL`, `THRESHOLD`, `ANOMALY`,
`CFG_CHG`) so an off-chain consumer can do centralized detection across the
whole contract set, grouped by `trace_id`.

## Reality (what's actually deployed)

Only **2 of 114** contracts in `contracts/` reference the telemetry API
(`TelemetryEvent`, `derive_trace_id`, or `emit_telemetry`) anywhere in their
source:

- `contract_monitoring` — the canonical implementation (`src/telemetry.rs`).
- `zkp_registry` — has its **own separate copy** of `telemetry.rs` rather
  than depending on `contract_monitoring`'s. It is not verified here whether
  this copy stays in sync with the canonical schema version; that's a
  follow-up (see below).

No other contract in the workspace depends on `contract_monitoring` as a
library crate (`grep -rl contract_monitoring contracts/*/Cargo.toml` matches
only `contract_monitoring` itself), so the "centralized detection" the
schema doc describes does not actually cover the contract set — it covers
two contracts.

## Why this wasn't fixed wholesale here

Retrofitting telemetry emission into the other ~112 contracts means adding a
new dependency, call sites at every auth/authz/threshold decision point, and
event-shape changes per contract — a large, contract-by-contract body of
work that needs individual review (some contracts may have deliberately
opted out, e.g. pure read-only or already-deprecated ones) and isn't
something that can be done safely without compiling and testing each
change. That's out of scope for a single small PR.

## What landed instead

`scripts/check_security_telemetry_conformance.sh` — a heuristic,
informational check: it flags contracts whose directory name suggests a
security-sensitive responsibility (auth, treasury, admin, bridge, consent,
credential, custody, escrow, recovery, access_control, identity, key
rotation) and that don't reference the telemetry API. Run it locally:

```bash
./scripts/check_security_telemetry_conformance.sh
```

It's wired into CI (`.github/workflows/security-checks.yml`) as a
**non-blocking, informational** step for now — the flagged list is long and
each entry needs an owner decision (instrument it, or document why it's
exempt), not an automatic CI failure.

## Follow-ups (to be filed as separate issues)

1. Triage the flagged contract list from the check script and assign owners
   per contract to either instrument telemetry or document an explicit
   exemption.
2. Reconcile `zkp_registry/src/telemetry.rs` against
   `contract_monitoring/src/telemetry.rs` — confirm schema version parity or
   consolidate to a single shared dependency.
3. Once a critical mass of contracts conform, flip the CI check from
   informational to `--strict` (non-zero exit) for new contracts only.
