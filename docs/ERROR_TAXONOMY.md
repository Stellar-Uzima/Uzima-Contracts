# Contract Error Taxonomy and SDK Mapping

This document is the authoritative reference for all error codes emitted by
Uzima Contracts. Every on-chain `#[contracterror]` variant must appear in
this table and in `contracts/common_error/src/sdk_mapping.rs`.

## Discriminant Range Allocation

| Range | Owner | Notes |
|-------|-------|-------|
| 0 – 99 | `common_error` (this module) | Core Soroban-level errors |
| 100 – 199 | `upgradeability` | Upgrade/migration errors |
| 200 – 299 | `lifecycle` | Lifecycle state machine errors |
| 300 – 399 | `rbac` | Role-based access control errors |
| 400 – 499 | `identity_registry` | DID / identity errors |
| 500 – 599 | `cross_chain_bridge` | Cross-chain errors |
| 600 – 699 | `healthcare_payment` / `escrow` | Payment / escrow errors |
| 700 – 799 | `patient_consent_management` | Consent lifecycle errors |
| 800 – 899 | `medical_records` | Medical record errors |
| 900 – 999 | `audit` | Audit trail errors |
| 1000+ | Domain-specific contracts | Reserve a contiguous range |

**New contract authors**: choose an unused range and add an entry to this table.

---

## Common Errors (0 – 99)

| Code | Slug | Category | Message |
|------|------|----------|---------|
| 0 | `unknown` | Internal | An unknown error occurred |
| 1 | `unauthorized` | Forbidden | Caller not authorized |
| 2 | `not_initialized` | Client | Contract not initialized |
| 3 | `already_initialized` | Conflict | Already initialized |
| 4 | `contract_paused` | Transient | Contract temporarily paused |
| 5 | `deadline_exceeded` | Client | Operation deadline passed |
| 6 | `rate_limit_exceeded` | Transient | Too many requests |
| 7 | `insufficient_funds` | Client | Not enough XLM |
| 8 | `invalid_input` | Client | Invalid input parameters |
| 9 | `invalid_state` | Conflict | Invalid contract state |
| 10 | `not_found` | NotFound | Resource not found |
| 11 | `access_denied` | Forbidden | Access denied |
| 12 | `timeout` | Transient | Operation timed out |
| 13 | `invalid_argument` | Client | Invalid argument |
| 14 | `external_contract_not_set` | Internal | Missing external contract |
| 15 | `invalid_data` | Client | Invalid data |
| 16 | `invalid_payload` | Client | Malformed payload |
| 17 | `duplicate_submission` | Conflict | Already submitted |
| 18 | `unauthorized_caller` | Forbidden | Unauthorized cross-contract call |

## Upgrade Errors (100 – 199)

| Code | Slug | Category | Message |
|------|------|----------|---------|
| 100 | `upgrade_not_authorized` | Forbidden | Not authorized to upgrade |
| 101 | `invalid_wasm_hash` | Client | Invalid WASM hash |
| 102 | `version_already_exists` | Conflict | Version already exists |
| 103 | `migration_failed` | Internal | Migration failed |
| 104 | `incompatible_version` | Client | Incompatible version |
| 105 | `contract_paused_upgrade` | Transient | Contract paused during upgrade |
| 106 | `history_not_found` | NotFound | Upgrade history not found |
| 107 | `integrity_check_failed` | Internal | Integrity check failed |
| 108 | `deprecated_function` | Client | Deprecated function called |

## Lifecycle Errors (200 – 299)

| Code | Slug | Category | Message |
|------|------|----------|---------|
| 200 | `lifecycle_not_initialized` | Client | Not initialized |
| 201 | `lifecycle_paused` | Transient | Paused |
| 202 | `lifecycle_upgrade_in_progress` | Transient | Upgrade in progress |
| 203 | `lifecycle_deprecated` | Internal | Permanently deprecated |
| 204 | `lifecycle_invalid_transition` | Conflict | Invalid transition |

---

## SDK Integration

The `contracts/common_error/src/sdk_mapping.rs` module provides:

```rust
// Get full SDK error metadata for a discriminant
let err = error_from_code(6).unwrap();
// err.slug == "rate_limit_exceeded"
// err.category == ErrorCategory::TransientError
// err.hint == Some("Wait before retrying. Use exponential backoff.")

// Check if an error is retryable
if is_retryable(code) {
    retry_with_backoff(request);
}
```

### TypeScript SDK usage

```typescript
import { UzimaError, errorFromCode } from '@uzima/sdk';

try {
  await contract.createRecord(patient, data);
} catch (e) {
  const err = errorFromCode(e.code);
  console.error(`[${err.slug}] ${err.message}`);
  if (err.isRetryable) {
    await retryWithBackoff(() => contract.createRecord(patient, data));
  }
}
```

---

## Adding a New Error Code

1. Choose an unregistered discriminant in your module's range.
2. Add it to your `#[contracterror]` enum with `#[repr(u32)]`.
3. Add an entry to `contracts/common_error/src/sdk_mapping.rs` in `FULL_TAXONOMY`.
4. Update this document.
5. Run `scripts/check_error_codes.sh` to validate consistency.
