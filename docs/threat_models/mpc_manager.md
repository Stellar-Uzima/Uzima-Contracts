# Threat Model: mpc_manager

> First-pass audit-grade threat model (#1567). Based on a static read of
> `contracts/mpc_manager/src/lib.rs`'s public entrypoints and visible
> `require_auth`/admin patterns — not a full line-by-line code audit.
> Needs review by a maintainer with deep contract knowledge before the
> risk tier and mitigation statuses below are treated as final.

---

## 1. Contract Overview

| Field | Value |
|-------|-------|
| Contract name | `mpc_manager` |
| Review date | see PR history for #1567 |
| Risk tier | **Critical** (coordinates secure multi-party computation sessions — secret sharing, reveals, and statistical/ML computation over data that's presumably sensitive given the healthcare context) |

### Purpose
Manages MPC sessions: starting a session, committing/revealing secret
shares, finalizing sessions, and running statistical analysis / ML
training over the resulting shares.

### Trust boundaries

| Actor | Trust level | Notes |
|-------|-------------|-------|
| Admin | High | `initialize` |
| Session participant | Medium | `commit_share`, `reveal_share` — presumably scoped to the session's registered participants |
| Session initiator | Medium | `start_session` |
| Unauthenticated caller | None | Read-only getters (`get_session`, `get_commitment`, `get_audit_trail`, etc.) |

---

## 2. Assets and Sensitive Data

| Asset | Sensitivity | Stored where | Protected by |
|-------|------------|-------------|-------------|
| Committed shares (pre-reveal) | Critical — the whole point of commit/reveal is that shares aren't usable before reveal | Persistent storage | Commit/reveal protocol; needs review that `reveal_share` checks a matching prior commitment |
| Revealed shares | Confidential | Persistent storage | Session-scoped access, not independently verified here |
| Computation results (statistics/ML model) | Confidential — derived from participant data | Persistent storage | Needs review of read-access gating |

---

## 3. Threat Enumeration (high-level)

| Category | Threat | Apparent mitigation | Status |
|---|---|---|---|
| Spoofing | Non-participant submits a share for a session | Caller auth expected on `commit_share`/`reveal_share` (9 `require_auth`/`require_admin` sites found across the contract) | Needs review — not verified per-entrypoint here |
| Tampering | Revealed share doesn't match its earlier commitment | Standard commit-reveal requires this check | **Needs review** — commitment-binding not independently confirmed in this pass |
| Repudiation | Participant denies submitting a share | `get_audit_trail` exists, suggesting an event log | Needs review of what it actually records |
| Denial of Service | `train_secure_ml_model` / `perform_statistical_analysis` cost exhausts CPU budget | `get_gas_stats` exists | Needs review — no visible per-call cap found in this pass |
| Elevation of Privilege | Non-initiator calls `finalize_session` early, before all shares are revealed | N/A | **Needs review** — finalize-timing/completeness check not independently confirmed |

---

## 4. High-Risk Entrypoints

| Entrypoint | Risk | Notes |
|------------|------|-------|
| `commit_share` / `reveal_share` | Critical | Core of the MPC security guarantee — wrong ordering or binding breaks confidentiality |
| `finalize_session` | Critical | Confirm it requires all expected shares before producing a result |
| `train_secure_ml_model` / `perform_statistical_analysis` | High | Highest computational cost; also highest data-sensitivity output |
| `submit_computation_proof` | High | Presumably attests to correct MPC execution — confirm what it actually verifies |

---

## 5. Residual Risks

| Risk | Notes |
|------|-------|
| Not independently verified | This document is a first pass built from function signatures and grep counts, not a manual trace of the commit-reveal state machine. Treat "Needs review" items as open until a maintainer confirms. |

---

## 6. Security Review Checklist

- [ ] All entrypoints have `require_auth()` or explicit no-auth justification — **not independently confirmed per-entrypoint here**
- [ ] `reveal_share` validates against the matching `commit_share` (commitment binding)
- [ ] `finalize_session` requires session completeness before producing results
- [ ] Threat model reviewed by a second maintainer
