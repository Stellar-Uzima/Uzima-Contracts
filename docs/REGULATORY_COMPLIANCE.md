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
- **Genomic Research Consent**: The `genomic_data` contract now supports explicit, category-based research consents. Patients can grant separate consent for:
  - `GeneralResearch`
  - `DiseaseSpecific(String)`
  - `CommercialResearch`
  - `InternationalTransfer`
  Research access requests must declare the category being used, and access is rejected unless the patient has explicitly consented to that category.
- **Audit & Withdrawal Notifications**: Every research access attempt emits a `GenomicConsentAudit` event with the consent category. When a patient withdraws category-specific genomic consent, active research projects that have used the data receive a withdrawal notification.

### GDPR Article 9 — Processing of Special Categories of Personal Data
GDPR Article 9 restricts processing of sensitive personal data, including genetic data, unless a specific legal basis exists. In the `genomic_data` contract, explicit patient consent for research is implemented as that legal basis.

- **Article 9(2)(a)**: Processing is permitted when the data subject has given explicit consent for one or more specified purposes. The `GenomicConsentCategory` enum records the exact research purpose.
- **Article 9(2)(j)**: Processing for scientific research purposes is allowed under appropriate safeguards. Category-specific consent and audit logging provide those safeguards.
- **Revocation**: Patients can withdraw consent at any time for a specific category, and the contract notifies active research projects to enforce GDPR data minimization and consent lifecycle requirements.

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

### GDPR Article 17 in `patient_consent_management`

The `patient_consent_management` contract implements a controlled, two-stage erasure process to comply with GDPR's "Right to Erasure".

1.  **Erasure Request**: A patient initiates an erasure request by calling the `request_erasure` entrypoint. This creates an on-chain record of the request and emits an `ErasureRequested` event. This step is restricted to the patient themselves, ensuring that only the data subject can initiate the process.

2.  **Admin-Triggered Execution**: A contract administrator executes the erasure by calling the `execute_erasure` entrypoint, providing the patient's address. This two-step process prevents accidental or malicious erasures and provides a clear, auditable separation of duties.

3.  **Data Nullification**: Upon execution, the contract nullifies all of the patient's consent records. It does this by:
    *   Deleting the patient's `ConsentLog`, which contains their entire history of consent grants and revocations.
    *   Deleting all `ConsentRecord` entries from the `ProviderIndex`, which tracks individual consents.

4.  **Finalization**: The erasure request's status is updated to `Executed`, and an `ErasureExecuted` event is emitted. After this point, all of the patient's consent data is irrecoverably removed from the contract's storage.

This flow provides a robust and auditable mechanism for complying with GDPR's right to erasure, ensuring that patient data can be completely and verifiably removed from the system upon request.