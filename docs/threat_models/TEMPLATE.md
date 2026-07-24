# Threat Model: [CONTRACT_NAME]

> **Instructions**: Copy this template to `docs/threat_models/<contract_name>.md`
> and fill in every section before the contract is merged to `main`.

---

## 1. Contract Overview

| Field | Value |
|-------|-------|
| Contract name | `<contract_name>` |
| Version | `0.1.0` |
| Author | @handle |
| Review date | YYYY-MM-DD |
| Risk tier | Low / Medium / High / Critical |

### Purpose
_One paragraph describing what the contract does and why it exists._

### Trust boundaries
_List every external actor and contract that interacts with this contract._

| Actor | Trust level | Notes |
|-------|-------------|-------|
| Admin | High | Configures contract, can pause/upgrade |
| Patient | Medium | Can read/write their own records |
| Doctor | Medium | Authorised by patient consent |
| Unauthenticated caller | None | Should be rejected |

---

## 2. Assets and Sensitive Data

| Asset | Sensitivity | Stored where | Protected by |
|-------|------------|-------------|-------------|
| Patient address | PII | Persistent storage key | Soroban auth |
| Medical record hash | Confidential | Persistent storage | Encryption off-chain |
| Admin key | Critical | Not stored on-chain | Caller auth |

---

## 3. Threat Enumeration (STRIDE)

### Spoofing

| # | Threat | Mitigation | Status |
|---|--------|------------|--------|
| S1 | Attacker forges patient address | `patient.require_auth()` on all write entrypoints | ✅ Mitigated |
| S2 | Unauthorised contract call | Policy engine + RBAC role check | ✅ Mitigated |

### Tampering

| # | Threat | Mitigation | Status |
|---|--------|------------|--------|
| T1 | Modify another patient's record | Per-patient key namespace + auth | ✅ Mitigated |
| T2 | Replay old transaction | Nonce / ledger sequence check | ✅ Mitigated |

### Repudiation

| # | Threat | Mitigation | Status |
|---|--------|------------|--------|
| R1 | Deny write occurred | Immutable audit event on every mutation | ✅ Mitigated |

### Information Disclosure

| # | Threat | Mitigation | Status |
|---|--------|------------|--------|
| I1 | Unauthorized read of patient data | Consent check before read | ✅ Mitigated |
| I2 | Contract storage exposed | All PII hashed or encrypted off-chain | ✅ Mitigated |

### Denial of Service

| # | Threat | Mitigation | Status |
|---|--------|------------|--------|
| D1 | Storage exhaustion via spam | Rate limiting + storage budget checks | ⚠️ Partial |
| D2 | CPU exhaustion via complex queries | Pagination + budget limits | ✅ Mitigated |

### Elevation of Privilege

| # | Threat | Mitigation | Status |
|---|--------|------------|--------|
| E1 | Regular user calls admin entrypoint | `require_admin()` guard | ✅ Mitigated |
| E2 | Upgrade without multi-sig | Timelock + governance vote required | ✅ Mitigated |

---

## 4. High-Risk Entrypoints

List entrypoints that require special scrutiny:

| Entrypoint | Risk | Required auth | Review notes |
|------------|------|---------------|-------------|
| `initialize()` | Critical | Admin | One-time only, uses InitGuard |
| `upgrade()` | Critical | Admin + Timelock | Requires lifecycle state |
| `emergency_pause()` | High | Admin | Should emit event |

---

## 5. Residual Risks

List accepted risks with justification:

| Risk | Likelihood | Impact | Justification |
|------|-----------|--------|---------------|
| Stellar network downtime | Low | High | Out of scope — Stellar infrastructure |

---

## 6. Security Review Checklist

Before merging:

- [ ] All entrypoints have `require_auth()` or explicit no-auth justification
- [ ] No `unwrap()` in production paths (see `scripts/check_no_production_unwraps.sh`)
- [ ] All storage writes have lifecycle guard (`require_active`)
- [ ] All string inputs are length-bounded
- [ ] Events emitted for all state-changing entrypoints
- [ ] Rate limiting in place for public entrypoints
- [ ] Threat model reviewed by a second maintainer
