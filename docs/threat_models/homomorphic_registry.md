# Threat Model: homomorphic_registry

> First-pass audit-grade threat model (#1567). Based on a static read of
> `contracts/homomorphic_registry/src/lib.rs`'s public entrypoints — not a
> full line-by-line code audit. Needs review by a maintainer with deep
> contract knowledge before the risk tier and mitigation statuses below
> are treated as final.

---

## 1. Contract Overview

| Field | Value |
|-------|-------|
| Contract name | `homomorphic_registry` |
| Review date | see PR history for #1567 |
| Risk tier | **Critical** (manages homomorphic-encryption key bundles and performs FHE operations — key mismanagement or a broken op directly compromises encrypted-data confidentiality/correctness) |

### Purpose
Registers HE key bundles (CKKS/BGV schemes implied by function names),
encrypts vectors under those schemes, performs FHE add/multiply and
bootstrap operations on ciphertexts, and runs encrypted statistics /
linear inference over them.

### Trust boundaries

| Actor | Trust level | Notes |
|-------|-------------|-------|
| Admin | High | `initialize`, `register_context`, `deactivate_context` |
| Key bundle owner | Medium | `register_key_bundle`, `set_performance_profile` |
| Computation submitter | Medium | `encrypt_ckks_vector`/`encrypt_bgv_vector`, `fhe_add`/`fhe_multiply`, `submit_encrypted_computation` |
| Unauthenticated caller | None | Read-only getters (`get_ciphertext`, `get_context`, `get_computation`, `estimate_operation_cost`) |

**Note on `get_ciphertext`:** a ciphertext being readable by anyone is
expected under HE (the point is confidentiality survives disclosure of
the ciphertext) — this is *not* automatically a finding, but it's worth a
maintainer explicitly confirming that assumption holds for however this
contract's scheme parameters are chosen.

---

## 2. Assets and Sensitive Data

| Asset | Sensitivity | Stored where | Protected by |
|-------|------------|-------------|-------------|
| HE key bundles | Critical | Persistent storage | `register_key_bundle`; confirm only the owning address can register/rotate their own bundle |
| Ciphertexts | Confidential-by-design (HE) | Persistent storage | Semantic security of the underlying scheme, not access control — correctness of `fhe_add`/`fhe_multiply`/`bootstrap_ciphertext` matters more than read-gating |
| HE context/scheme parameters | Critical | Persistent storage | `register_context`/`deactivate_context` — wrong parameters silently weaken every ciphertext under that context |

---

## 3. Threat Enumeration (high-level)

| Category | Threat | Apparent mitigation | Status |
|---|---|---|---|
| Tampering | `fhe_add`/`fhe_multiply` mixes ciphertexts from incompatible contexts/key bundles, producing garbage or leaking structure | N/A | **Needs review** — this is the class of bug most specific to HE contracts and least likely to be caught by generic auth checks |
| Elevation of Privilege | Non-owner calls `register_key_bundle`/`set_performance_profile` for another user's bundle | Caller auth expected (19 `require_auth`/`require_admin` sites found across the contract) | Needs review — not verified per-entrypoint here |
| Denial of Service | `bootstrap_ciphertext`/`encrypted_linear_inference` cost exhausts CPU budget | `estimate_operation_cost` exists, suggesting cost is estimable up front | Needs review — confirm callers are expected/required to check this before submitting, and whether there's a hard cap |
| Information Disclosure | `encrypted_statistics`/`encrypted_linear_inference` output structure leaks more than intended even though individual values stay encrypted | N/A | **Needs review** — flagged as the highest-sensitivity output surface in this contract |

---

## 4. High-Risk Entrypoints

| Entrypoint | Risk | Notes |
|------------|------|-------|
| `fhe_add` / `fhe_multiply` / `bootstrap_ciphertext` | Critical | Core homomorphic operations — confirm context/key-bundle compatibility is checked, not assumed |
| `register_context` / `deactivate_context` | Critical | Wrong scheme parameters compromise every ciphertext under that context |
| `register_key_bundle` | Critical | Entry point for new trust anchors into the HE system |
| `encrypted_statistics` / `encrypted_linear_inference` | High | Highest data-sensitivity aggregate outputs |
| `submit_encrypted_computation` | High | Confirm this can't be used to bypass the individually-gated FHE ops above |

---

## 5. Residual Risks

| Risk | Notes |
|------|-------|
| HE-specific correctness bugs | This is a domain where the biggest risks (context/key mismatches, parameter weaknesses) aren't the kind of thing a grep-based audit can catch — needs someone with FHE scheme expertise, not just Soroban-contract-auth expertise. |
| Not independently verified | This document is a first pass built from function signatures and grep counts, not a manual trace of every auth path. Treat "Needs review" items as open until a maintainer confirms. |

---

## 6. Security Review Checklist

- [ ] `fhe_add`/`fhe_multiply` reject ciphertexts from mismatched contexts/key bundles
- [ ] `register_key_bundle` restricted to the bundle's own owner
- [ ] Cost estimation (`estimate_operation_cost`) is enforced, not just advisory, for expensive ops
- [ ] A maintainer with FHE scheme expertise reviews `register_context`/`deactivate_context` parameter handling
- [ ] Threat model reviewed by a second maintainer
