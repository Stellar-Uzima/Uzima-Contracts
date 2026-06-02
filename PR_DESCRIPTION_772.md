# Resolve Issue #772 — Public Getter Functions Return Empty/Default Values

## Summary

This PR resolves issue #772: "Public Getter Functions Return Empty/Default Values Instead of Errors for Missing Data".

---

## Issue #772 — Getter Return Type Fixes

### Problem
Several contracts returned default values (e.g., panicking with `.unwrap()`, or returning `Role::None`) when queried for non-existent data, making it impossible for callers to distinguish between valid data and missing data.

### Changes

#### 1. treasury_controller/src/lib.rs

**Fixed `get_config()`** — Now returns `Result<TreasuryConfig, Error>` instead of panicking with `.unwrap()`.
- Added `Error::ConfigNotFound` variant for missing configuration
- Callers can now handle the "not initialized" case gracefully

**Fixed `get_proposal()`** — Now returns `Result<TreasuryProposal, Error>` instead of panicking with `.unwrap()`.
- Returns `Error::ProposalNotFound` for missing proposals or empty maps

**Fixed `gnosis_get_threshold()`** — Now returns `Result<u32, Error>` instead of `u32`

**Fixed `gnosis_get_owners()`** — Now returns `Result<Vec<Address>, Error>` instead of `Vec<Address>`

#### 2. medical_records/src/lib.rs

**Fixed `get_user_role()`** — Now returns `Result<Role, Error>` instead of silently returning `Role::None` for non-existent or inactive users.
- Returns `Error::Unauthorized` when user is not found or is inactive
- Callers can now distinguish between a valid user with a role and a missing user

#### 3. Updated all callers

- `tests/utils/contract_fixtures.rs`: Updated to use `.unwrap()` on new Result types
- `tests/integration/healthcare_workflows.rs`: Updated to use `.unwrap()` on new Result types
- `contracts/treasury_controller/src/test.rs`: Updated tests to handle new Result return types

---

## 📋 Files Changed

| File | Change |
|------|--------|
| `contracts/treasury_controller/src/lib.rs` | Modified (4 functions) |
| `contracts/treasury_controller/src/test.rs` | Modified (test updates) |
| `contracts/medical_records/src/lib.rs` | Modified (1 function) |
| `tests/utils/contract_fixtures.rs` | Modified (caller update) |
| `tests/integration/healthcare_workflows.rs` | Modified (caller update) |

---

**Closes:** #772
**Assignee:** Icahbod
