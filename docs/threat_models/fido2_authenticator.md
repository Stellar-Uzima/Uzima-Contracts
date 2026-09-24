# Threat Model: fido2_authenticator

> First-pass audit-grade threat model (#1567). Based on a static read of
> `contracts/fido2_authenticator/src/lib.rs`'s public entrypoints and its
> own module documentation — not a full line-by-line code audit. Needs
> review by a maintainer with deep contract knowledge before the risk
> tier and mitigation statuses below are treated as final.

---

## 1. Contract Overview

| Field | Value |
|-------|-------|
| Contract name | `fido2_authenticator` |
| Review date | see PR history for #1567 |
| Risk tier | **Critical** (this contract *is* the authentication mechanism for other identity flows — a bypass here compromises everything gated behind it) |

### Purpose
FIDO2/WebAuthn device registry on Soroban: registers platform/hardware
authenticators, issues and verifies registration/auth challenges, verifies
Ed25519 assertions (and optionally ZK-based assertions), and supports
device revocation.

### Trust boundaries

| Actor | Trust level | Notes |
|-------|-------------|-------|
| Admin | High | `initialize`, `set_identity_registry`, `set_zk_verifier` |
| Device owner (user) | Medium | `register_device`, `issue_auth_challenge`, `verify_ed25519_assertion`, `revoke_device`, `update_device_name` |
| Unauthenticated caller | None | `issue_registration_challenge`/`issue_auth_challenge` must be safely callable pre-auth by design (that's the point of a challenge), but should only ever return a challenge, never grant access by themselves |

---

## 2. Assets and Sensitive Data

| Asset | Sensitivity | Stored where | Protected by |
|-------|------------|-------------|-------------|
| Registered credential IDs / public keys | Critical | Persistent storage | `require_auth` on registration; needs review of replay protection on `verify_ed25519_assertion` |
| Auth/registration challenges | Critical (short-lived) | Persistent storage | Needs review of expiry/single-use enforcement — a replayable challenge would be a full auth bypass |
| `rp_id_hash` (relying-party ID) | Critical | Set at `initialize` | Confirm it's immutable after init (a mutable RP ID would let an admin redirect what "site" a signature is scoped to) |

---

## 3. Threat Enumeration (high-level)

| Category | Threat | Apparent mitigation | Status |
|---|---|---|---|
| Spoofing | Assertion signature replayed to re-authenticate | `env.crypto().ed25519_verify()` per the module doc; needs review that the signed payload binds to a single-use challenge (nonce), not just the credential ID | **Needs review — this is the single highest-value check in the whole contract** |
| Spoofing | Registration/auth challenge reused across sessions | Challenge issuance functions exist as separate calls from verification | **Needs review** — confirm challenges are marked consumed after one use |
| Tampering | `revoke_device` called by someone other than the device owner or admin | Function present with presumed caller auth | Needs review — not independently confirmed |
| Elevation of Privilege | `set_zk_verifier`/`set_identity_registry` pointed at an attacker-controlled contract | Admin-gated per function naming | Needs review of whether this is single-admin (like `secure_enclave`) or multisig-gated (like `zkp_registry`) |
| Denial of Service | `list_devices`/`get_revocation_history` unbounded growth exhausts read budget | N/A | Needs review — check pagination |

---

## 4. High-Risk Entrypoints

| Entrypoint | Risk | Notes |
|------------|------|-------|
| `verify_ed25519_assertion` | Critical | The actual authentication check — confirm challenge-binding and single-use enforcement |
| `verify_zk_assertion` | Critical | Same as above, plus depends on `zkp_registry`/`set_zk_verifier` being correctly configured |
| `set_identity_registry` / `set_zk_verifier` | Critical | Repointing either changes what "valid" means for every future verification |
| `revoke_device` | High | Confirm only the device owner or admin can revoke, and that a revoked device is actually rejected in verification (not just marked) |
| `register_device` | High | Entry point for new trust anchors |

---

## 5. Residual Risks

| Risk | Notes |
|------|-------|
| Not independently verified | This document is a first pass built from function signatures and the module's own doc comments, not a manual trace of the challenge-response state machine. Treat "Needs review" items as open until a maintainer confirms, especially challenge single-use enforcement. |

---

## 6. Security Review Checklist

- [ ] `verify_ed25519_assertion` binds the signed payload to a single-use challenge (nonce), preventing replay
- [ ] Registration/auth challenges are consumed after one use and expire
- [ ] `revoke_device` is restricted to the device owner or admin, and revoked devices are rejected at verification time
- [ ] `set_identity_registry` / `set_zk_verifier` — confirm single-admin vs. multisig-gated, and whether that matches the contract's Critical risk tier
- [ ] Threat model reviewed by a second maintainer
