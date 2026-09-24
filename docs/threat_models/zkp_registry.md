# Threat Model: zkp_registry

> First-pass audit-grade threat model (#1567). Based on a static read of
> `contracts/zkp_registry/src/lib.rs`'s public entrypoints and visible
> `require_auth`/admin patterns — not a full line-by-line code audit. Needs
> review by a maintainer with deep contract knowledge before the risk
> tier and mitigation statuses below are treated as final.

---

## 1. Contract Overview

| Field | Value |
|-------|-------|
| Contract name | `zkp_registry` |
| Review date | see PR history for #1567 |
| Risk tier | **Critical** (verifies zero-knowledge proofs gating medical-record and credential claims; admin path uses multisig/timelock-style proposals) |

### Purpose
Registers ZK circuits, verifies submitted zero-knowledge proofs (range
proofs, credential proofs, medical-record proofs, recursive proofs), and
manages verification-key (VK) rotation for those circuits.

### Trust boundaries

| Actor | Trust level | Notes |
|-------|-------------|-------|
| Admin | High | `initialize`, `configure_multisig`, VK rotation/rollback |
| Multisig proposal executor | High | `create_admin_proposal` / `approve_admin_proposal` / `execute_admin_proposal` |
| Circuit registrant | Medium | `register_circuit` — needs its own auth review |
| Proof submitter | Medium | `submit_zkp`, `submit_zkp_batch`, proof-specific submit functions |
| Unauthenticated caller | None | Read-only getters (`get_verification_result`, `get_circuit_params`, etc.) should be safe to expose; nothing else should be |

---

## 2. Assets and Sensitive Data

| Asset | Sensitivity | Stored where | Protected by |
|-------|------------|-------------|-------------|
| Verification keys (per circuit) | Critical — a bad VK makes proof verification meaningless | Persistent storage | Admin/multisig-gated rotation (`rotate_vk`, `rollback_vk`) |
| Submitted proofs (medical-record, credential, range) | Confidential | Persistent storage | Caller auth on submit; read-gating not independently verified here |
| Admin multisig configuration | Critical | Persistent storage | `configure_multisig` — auth path not independently verified here |

---

## 3. Threat Enumeration (high-level)

| Category | Threat | Apparent mitigation | Status |
|---|---|---|---|
| Spoofing | Forged proof submission attributed to another user | Caller auth on submit functions (25 `require_auth`/`require_admin` sites found) | Needs review — not verified per-entrypoint here |
| Tampering | VK swapped to a weak/malicious one | `rotate_vk`/`rollback_vk`/`migrate_vk_rotation` gated (apparently admin-only) | Needs review |
| Elevation of Privilege | Regular caller triggers `emergency_override` | Function takes an `executor: Address` and a `proposal_id` — implies it's gated behind the proposal/approval flow, not a bare admin call | **Needs review** — confirm `emergency_override` can't be called without a prior approved proposal |
| Denial of Service | Proof-verification cost (range/recursive proofs) exhausts CPU budget | `get_gas_stats` exists, suggesting cost tracking | Needs review — no visible per-call cap found in this pass |
| Information Disclosure | `create_medical_record_proof`/`create_credential_proof` outputs leak more than the ZK proof should | N/A | **Needs review** — highest-sensitivity function in this contract |

---

## 4. High-Risk Entrypoints

| Entrypoint | Risk | Notes |
|------------|------|-------|
| `configure_multisig` | Critical | Defines who can approve admin proposals — misconfiguration is catastrophic |
| `execute_admin_proposal` / `emergency_override` | Critical | Bypass paths for normal governance; confirm the emergency path still requires an approved proposal |
| `rotate_vk` / `rollback_vk` / `migrate_vk_rotation` | Critical | Wrong VK breaks proof integrity for an entire circuit |
| `submit_zkp_batch` | High | Batch entrypoint — check per-item validation isn't skipped for gas savings |
| `create_medical_record_proof` / `create_credential_proof` | High | Highest data-sensitivity proof types |

---

## 5. Residual Risks

| Risk | Notes |
|------|-------|
| Not independently verified | This document is a first pass built from function signatures and grep counts, not a manual trace of every auth path. Treat "Needs review" items as open until a maintainer confirms. |

---

## 6. Security Review Checklist

- [ ] All entrypoints have `require_auth()` or explicit no-auth justification — **not independently confirmed per-entrypoint here**
- [ ] `emergency_override` cannot bypass the proposal/approval flow
- [ ] VK rotation is multisig/timelock-gated, not single-admin
- [ ] Threat model reviewed by a second maintainer
