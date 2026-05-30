# Regulatory Compliance Framework

This document outlines how the Uzima Contracts implement features to comply with major healthcare regulations like HIPAA and GDPR.

## Framework Overview

The **RegulatoryCompliance** smart contract integrates directly into the core `medical_records` architecture. By configuring real-time audit trails and granular right-to-be-forgotten controls, the platform enables healthcare networks to enforce their data policies on-chain.

### 1. HIPAA Compliance
The Health Insurance Portability and Accountability Act (HIPAA) requires strict auditing and confidentiality features.
- **Audit Trails**: All data access is intrinsically logged to the blockchain. When strict auditing is enabled, an advanced, standardized log is written directly to the `RegulatoryCompliance` contract using the `log_audit` function via intra-contract calls from `medical_records`.
- **Identity Enforcement**: `medical_records` ties strongly into the `identity_registry` validating the credentials and roles of each user seamlessly.

### 2. GDPR Compliance
The General Data Protection Regulation (GDPR) empowers individuals to have granular control over their information.
- **Right to Be Forgotten**: Regulators mandate that a user can demand immediate deletion of access to their data. Calling `invoke_right_to_be_forgotten` writes a persistent flag that instantly breaks the authorization flow in `can_view_record`, `add_record`, and `add_record_with_did` within `medical_records`. All further attempts to read or write data associated with that specific patient identity will explicitly revert with `NotAuthorized`.
- **Consent Management**: Users can dynamically grant or revoke access using native consent structures (supported via granular fine-grained control flags within the compliance logic).

### 3. Usage & Setup

Deploy the `regulatory_compliance` contract, then bind it to the central `medical_records` logic:

```shell
# Deploy the contract
soroban contract deploy --wasm target/wasm32-unknown-unknown/release/regulatory_compliance.wasm

# Link to Medical Records
soroban contract invoke --id $MEDICAL_RECORDS_ID \
  -- source $ADMIN_SECRET \
  -- set_regulatory_compliance \
  -- caller $ADMIN_ADDRESS \
  -- compliance $COMPLIANCE_ID
```


## GDPR Article 17 — Right to Erasure (Cryptographic Key Deletion)

Since blockchain data is immutable, full deletion is not possible. The system uses
**cryptographic erasure**: deleting the patient's encryption key renders all encrypted
records permanently inaccessible.

### How It Works

1. Patient calls `request_erasure(patient_id)` on the `medical_records` contract.
2. Identity is verified against `identity_registry`.
3. The patient's encryption key is deleted from `crypto_registry`.
4. All active consents for the patient are atomically revoked.
5. An `ErasureCompleted` event is emitted (timestamp + patient ID hash — no PII).
6. Subsequent read attempts return `Err(ContractError::DataErased)`.

### Key Properties

| Property | Value |
|---|---|
| Irreversible | Yes — deleted key cannot be recovered |
| Who can invoke | Patient only (verified via `identity_registry`) |
| Event emitted | `ErasureCompleted { timestamp, patient_hash }` |
| Read after erasure | Returns `DataErased` error |

> **Note:** This satisfies GDPR Article 17 because encrypted data becomes permanently
> unreadable without the encryption key.
