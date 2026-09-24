# Threat Model: secure_enclave

> First-pass audit-grade threat model (#1567). Based on a static read of
> `contracts/secure_enclave/src/lib.rs`'s public entrypoints — not a full
> line-by-line code audit. Needs review by a maintainer with deep contract
> knowledge before the risk tier and mitigation statuses below are treated
> as final.

---

## 1. Contract Overview

| Field | Value |
|-------|-------|
| Contract name | `secure_enclave` |
| Review date | see PR history for #1567 |
| Risk tier | **Critical** (registers off-chain trusted-execution-environment nodes and verifies their remote attestations — the on-chain root of trust for anything computed inside an enclave) |

### Purpose
Registers enclave nodes, verifies their attestation, assigns and tracks
compute tasks to them, and can fall back a task to `mpc_manager` if the
enclave path isn't viable.

### Trust boundaries

| Actor | Trust level | Notes |
|-------|-------------|-------|
| Admin | High | `initialize`, `verify_attestation`, `assign_task`, `fallback_to_mpc` — every non-registration, non-submission entrypoint here takes an explicit `admin: Address` parameter |
| Enclave node operator | Medium | `register_enclave` — presumably self-registers, then depends on admin attestation verification before being trusted |
| Task submitter | Medium | `submit_task`, `complete_task` |
| Unauthenticated caller | None | No read-only getters were found in this pass — confirm whether task/enclave state is queryable elsewhere |

---

## 2. Assets and Sensitive Data

| Asset | Sensitivity | Stored where | Protected by |
|-------|------------|-------------|-------------|
| Enclave attestation status | Critical — this is the trust anchor for every task routed to that node | Persistent storage | `verify_attestation(admin, node_id, is_valid)` — a single admin's call flips trust; confirm this isn't a single point of failure (no multisig/timelock visible here, unlike `zkp_registry`'s admin-proposal flow) |
| Task inputs/outputs | Confidential (healthcare context) | Persistent storage | Assignment restricted to attested nodes, not independently verified here |

---

## 3. Threat Enumeration (high-level)

| Category | Threat | Apparent mitigation | Status |
|---|---|---|---|
| Spoofing | A node claims another node's identity to receive tasks | `register_enclave` presumably binds a `node_id` to a specific caller/address | Needs review — binding not independently confirmed |
| Tampering | `verify_attestation` marks a compromised node as valid | Admin-gated, but appears to be a **single admin call** with no visible multisig/timelock, unlike other Critical-tier contracts in this repo | **Flagged — likely gap.** Compare against `zkp_registry`'s proposal/approval pattern for admin-critical actions |
| Elevation of Privilege | `assign_task` routes a task to an unattested node | Function signature takes `admin`, `task_id`, `node_id` — confirm it checks the node's attestation status before assignment, not just admin caller auth | **Needs review** |
| Denial of Service | Enclave never calls `complete_task`, task stuck indefinitely | No visible timeout/reassignment path found in this pass | **Needs review** — consider whether `fallback_to_mpc` is the intended recovery path, and whether it's reachable without admin intervention |

---

## 4. High-Risk Entrypoints

| Entrypoint | Risk | Notes |
|------------|------|-------|
| `verify_attestation` | Critical | Single-admin trust decision for an entire enclave node — no multisig/timelock visible, unlike comparable admin-critical actions elsewhere in the repo |
| `assign_task` | High | Confirm attestation status is checked at assignment time, not just at registration |
| `fallback_to_mpc` | High | Cross-contract call into `mpc_manager` — confirm the target `mpc_manager_id` is validated, not attacker-supplied |
| `register_enclave` | Medium | Entry point for new trust anchors into the system |

---

## 5. Residual Risks

| Risk | Notes |
|------|-------|
| Single-admin attestation | Unlike `zkp_registry`'s multisig-gated admin-critical actions, `verify_attestation` here appears to be a single admin call. Whether that's an accepted risk (smaller blast radius, faster incident response) or a real gap needs a maintainer decision, not an assumption in this doc. |
| Not independently verified | This document is a first pass built from function signatures, not a manual trace of every code path. Treat "Needs review"/"Flagged" items as open until a maintainer confirms. |

---

## 6. Security Review Checklist

- [ ] Confirm `verify_attestation` should remain single-admin or needs the multisig/timelock pattern used elsewhere (e.g. `zkp_registry`)
- [ ] `assign_task` checks the target node's current attestation status, not just admin caller auth
- [ ] Timeout/reassignment path exists for tasks whose assigned enclave never calls `complete_task`
- [ ] `fallback_to_mpc`'s target contract ID is validated against an expected registry, not caller-supplied
- [ ] Threat model reviewed by a second maintainer
