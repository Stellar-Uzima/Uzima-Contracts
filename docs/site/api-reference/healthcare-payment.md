# Healthcare Payment

Contract: `healthcare_payment`

Handles insurance claim submission, pre-authorization, payment processing, and EDI integration.

## Security

Uses the **CEI pattern**: claim status is set to `Paid` and persisted to storage before token transfers are executed, preventing reentrancy.

<!-- API_START -->

## Key Functions

| Function | Parameters | Returns | Description |
|---|---|---|---|
| `initialize` | `env: Env, admin: Address, payment_router: Address, escrow_contract: Address, treasury: Address, token: Address, aml_contract: Address, rbac_contract: Address` | `Result<(), Error>` | — |
| `register_insurance_provider` | `env: Env, caller: Address, name: String, payer_code: String, supports_edi_837: bool, supports_edi_834: bool` | `Result<u64, Error>` | — |
| `register_coverage_policy` | `env: Env, caller: Address, patient: Address, insurance_provider_id: u64, policy_external_id: String, member_id: String, group_number: String, deductible_total: i128, copay_amount: i128, coinsurance_bps: u32` | `Result<u64, Error>` | — |
| `verify_insurance_eligibility` | `env: Env, caller: Address, policy_id: u64, service_id: String, coverage_bps: u32, provider_ref: String` | `Result<u64, Error>` | — |
| `submit_claim` | `env: Env, patient: Address, provider: Address, service_id: String, amount: i128, policy_id: String, preauth_id: Option<u64>` | `Result<u64, Error>` | — |
| `submit_insurance_claim` | `env: Env, caller: Address, claim_id: u64, coverage_policy_id: u64, payer_ref: String, transaction_code: String` | `Result<bool, Error>` | — |
| `sync_coverage_enrollment` | `env: Env, caller: Address, coverage_policy_id: u64, enrollment_ref: String, transaction_code: String` | `Result<u64, Error>` | — |
| `verify_claim` | `env: Env, claim_id: u64, verifier: Address` | `Result<(), Error>` | — |
| `approve_claim` | `env: Env, claim_id: u64, approver: Address` | `Result<(), Error>` | — |
| `reject_claim` | `env: Env, claim_id: u64, rejector: Address, reason: String` | `Result<(), Error>` | — |
| `process_eob` | `env: Env, caller: Address, claim_id: u64, coverage_policy_id: u64, allowed_amount: i128, insurer_paid: i128, deductible_applied: i128, adjudication_notes: String, edi_transaction: String` | `Result<bool, Error>` | — |
| `process_payment` | `env: Env, claim_id: u64` | `Result<(), Error>` | — |
| `batch_process_payments` | `env: Env, claim_ids: Vec<u64>` | `Result<Vec<u64>, Error>` | Process multiple approved claims in one call. Reads Config and creates TokenClient once. |
| `escrow_claim` | `env: Env, claim_id: u64` | `Result<(), Error>` | — |
| `request_preauth` | `env: Env, patient: Address, provider: Address, service_id: String, estimated_cost: i128` | `Result<u64, Error>` | — |
| `approve_preauth` | `env: Env, preauth_id: u64, approver: Address` | `Result<(), Error>` | — |
| `report_fraud` | `env: Env, claim_id: u64, reporter: Address, reason: String` | `Result<(), Error>` | — |
| `create_payment_plan` | `env: Env, patient: Address, provider: Address, total_amount: i128, installment_amount: i128, frequency: u64` | `Result<u64, Error>` | — |
| `pay_installment` | `env: Env, plan_id: u64` | `Result<(), Error>` | — |
| `get_coverage_policy` | `env: Env, coverage_policy_id: u64` | `Result<CoveragePolicy, Error>` | — |
| `get_eligibility_check` | `env: Env, eligibility_id: u64` | `Result<EligibilityCheck, Error>` | — |
| `get_claim_submission` | `env: Env, claim_id: u64` | `Result<ClaimSubmission, Error>` | — |
| `get_coverage_enrollment` | `env: Env, enrollment_id: u64` | `Result<CoverageEnrollment, Error>` | — |
| `get_explanation_of_benefits` | `env: Env, claim_id: u64` | `Result<ExplanationOfBenefits, Error>` | — |
| `submit_coverage_proof` | `env: Env, patient: Address, policy_id: u64, proof_hash: BytesN<32>, circuit_version: u32, proven_coverage_bps: u32, expires_at: u64, registry_proof_id: BytesN<32>` | `Result<(), Error>` | Submit a zero-knowledge proof of insurance coverage. The patient proves they have active coverage without revealing sensitive policy details on-chain. |
| `verify_coverage_with_zk` | `env: Env, caller: Address, policy_id: u64, patient: Address` | `Result<u32, Error>` | Verify insurance coverage using a previously submitted ZK proof. Returns the proven coverage basis points if the proof is valid. |
| `get_coverage_proof` | `env: Env, caller: Address, policy_id: u64, patient: Address` | `Result<CoverageProof, Error>` | Get the ZK coverage proof for a patient's policy. Get the ZK coverage proof for a patient's policy. |
| `get_coverage_proof_count` | `env: Env` | `u64` | Get the total number of coverage proofs submitted. |
| `get_patient_responsibility` | `env: Env, patient: Address` | `Option<PatientResponsibility>` | — |
| `emergency_pause` | `env: Env, caller: Address` | `Result<(), Error>` | Immediately open the circuit (emergency stop). Callable by admin or any authorized pauser. |
| `begin_recovery` | `env: Env, caller: Address` | `Result<(), Error>` | Transition circuit from Open -> HalfOpen to begin gradual recovery. Admin only. |
| `resume_operations` | `env: Env, caller: Address` | `Result<(), Error>` | Transition circuit from HalfOpen -> Closed, resetting the failure counter. Admin only. |
| `add_authorized_pauser` | `env: Env, caller: Address, pauser: Address` | `Result<(), Error>` | Grant an address the ability to trigger an emergency pause. Admin only. |
| `remove_authorized_pauser` | `env: Env, caller: Address, pauser: Address` | `Result<(), Error>` | Revoke an address's ability to trigger an emergency pause. Admin only. |
| `report_anomaly` | `env: Env, caller: Address, increment: u32` | `Result<CircuitState, Error>` | Report an anomaly. Increments the internal failure counter and automatically trips the circuit when the threshold is reached. Callable by admin or any authorized pauser (e.g. the anomaly_detection contract). |
| `set_failure_threshold` | `env: Env, caller: Address, threshold: u32` | `Result<(), Error>` | Set the failure threshold for automatic circuit tripping. Admin only. |
| `get_circuit_state` | `env: Env` | `CircuitState` | Returns the current circuit state (defaults to Closed if never set). |
| `get_circuit_breaker` | `env: Env` | `Option<CircuitBreaker>` | Returns the full circuit breaker record. |

## Types

### `enum ClaimStatus`

| Variant | Value | Description |
|---|---|---|
| `Submitted` | 0 | — |
| `Verified` | 1 | — |
| `Approved` | 2 | — |
| `PendingAMLReview` | 3 | — |
| `Rejected` | 4 | — |
| `Paid` | 5 | — |
| `Disputed` | 6 | — |

### `enum PreAuthStatus`

| Variant | Value | Description |
|---|---|---|
| `Pending` | 0 | — |
| `Approved` | 1 | — |
| `Denied` | 2 | — |
| `Expired` | 3 | — |

### `enum PaymentPlanStatus`

| Variant | Value | Description |
|---|---|---|
| `Active` | 0 | — |
| `Completed` | 1 | — |
| `Defaulted` | 2 | — |
| `Cancelled` | 3 | — |

### `enum CircuitState`

| Variant | Value | Description |
|---|---|---|
| `Closed` | — | — |
| `Open` | — | — |
| `HalfOpen` | — | — |

### `struct CircuitBreaker`

| Field | Type | Description |
|---|---|---|
| `state` | `CircuitState` | — |
| `failure_count` | `u32` | — |
| `failure_threshold` | `u32` | — |
| `opened_at` | `u64` | — |
| `last_state_change` | `u64` | — |
| `triggered_by` | `Option<Address>` | — |

### `enum ClaimSubmissionStatus`

| Variant | Value | Description |
|---|---|---|
| `Submitted` | 0 | — |
| `Acknowledged` | 1 | — |
| `Adjudicated` | 2 | — |

### `struct Claim`

| Field | Type | Description |
|---|---|---|
| `id` | `u64` | — |
| `patient` | `Address` | — |
| `provider` | `Address` | — |
| `service_id` | `String` | — |
| `amount` | `i128` | — |
| `status` | `ClaimStatus` | — |
| `policy_id` | `String` | — |
| `preauth_id` | `Option<u64>` | — |
| `created_at` | `u64` | — |
| `updated_at` | `u64` | — |

### `struct PreAuth`

| Field | Type | Description |
|---|---|---|
| `id` | `u64` | — |
| `patient` | `Address` | — |
| `provider` | `Address` | — |
| `service_id` | `String` | — |
| `estimated_cost` | `i128` | — |
| `status` | `PreAuthStatus` | — |
| `expiry` | `u64` | — |

### `struct PaymentPlan`

| Field | Type | Description |
|---|---|---|
| `id` | `u64` | — |
| `patient` | `Address` | — |
| `provider` | `Address` | — |
| `total_amount` | `i128` | — |
| `remaining_amount` | `i128` | — |
| `installment_amount` | `i128` | — |
| `frequency` | `u64` | — |
| `next_due` | `u64` | — |
| `status` | `PaymentPlanStatus` | — |

### `struct FraudReport`

| Field | Type | Description |
|---|---|---|
| `claim_id` | `u64` | — |
| `reporter` | `Address` | — |
| `reason` | `String` | — |
| `timestamp` | `u64` | — |

### `struct InsuranceProvider`

| Field | Type | Description |
|---|---|---|
| `id` | `u64` | — |
| `name` | `String` | — |
| `payer_code` | `String` | — |
| `supports_edi_837` | `bool` | — |
| `supports_edi_834` | `bool` | — |
| `active` | `bool` | — |

### `struct CoveragePolicy`

| Field | Type | Description |
|---|---|---|
| `id` | `u64` | — |
| `patient` | `Address` | — |
| `insurance_provider_id` | `u64` | — |
| `policy_external_id` | `String` | — |
| `member_id` | `String` | — |
| `group_number` | `String` | — |
| `deductible_total` | `i128` | — |
| `deductible_met` | `i128` | — |
| `copay_amount` | `i128` | — |
| `coinsurance_bps` | `u32` | — |
| `coverage_active` | `bool` | — |
| `last_verified_at` | `u64` | — |

### `struct EligibilityCheck`

| Field | Type | Description |
|---|---|---|
| `id` | `u64` | — |
| `policy_id` | `u64` | — |
| `service_id` | `String` | — |
| `eligible` | `bool` | — |
| `coverage_bps` | `u32` | — |
| `copay_amount` | `i128` | — |
| `deductible_remaining` | `i128` | — |
| `checked_at` | `u64` | — |
| `provider_ref` | `String` | — |

### `struct ClaimSubmission`

| Field | Type | Description |
|---|---|---|
| `claim_id` | `u64` | — |
| `policy_id` | `u64` | — |
| `submission_format` | `String` | — |
| `transaction_code` | `String` | — |
| `payer_ref` | `String` | — |
| `submitted_at` | `u64` | — |
| `status` | `ClaimSubmissionStatus` | — |

### `struct CoverageEnrollment`

| Field | Type | Description |
|---|---|---|
| `id` | `u64` | — |
| `policy_id` | `u64` | — |
| `transaction_code` | `String` | — |
| `enrollment_ref` | `String` | — |
| `synced_at` | `u64` | — |

### `struct ExplanationOfBenefits`

| Field | Type | Description |
|---|---|---|
| `claim_id` | `u64` | — |
| `policy_id` | `u64` | — |
| `allowed_amount` | `i128` | — |
| `insurer_paid` | `i128` | — |
| `patient_responsibility` | `i128` | — |
| `deductible_applied` | `i128` | — |
| `copay_amount` | `i128` | — |
| `adjudication_notes` | `String` | — |
| `processed_at` | `u64` | — |
| `edi_transaction` | `String` | — |

### `struct CoverageProof`

| Field | Type | Description |
|---|---|---|
| `policy_id` | `u64` | — |
| `patient` | `Address` | — |
| `proof_hash` | `BytesN<32>` | — |
| `circuit_version` | `u32` | — |
| `is_verified` | `bool` | — |
| `proven_coverage_bps` | `u32` | — |
| `submitted_at` | `u64` | — |
| `expires_at` | `u64` | — |
| `registry_proof_id` | `BytesN<32>` | — |

### `struct PatientResponsibility`

| Field | Type | Description |
|---|---|---|
| `patient` | `Address` | — |
| `total_copay_tracked` | `i128` | — |
| `total_deductible_tracked` | `i128` | — |
| `total_patient_responsibility` | `i128` | — |
| `last_updated` | `u64` | — |

### `enum RbacRole`

| Variant | Value | Description |
|---|---|---|
| `Admin` | 0 | — |
| `Doctor` | 1 | — |
| `Patient` | 2 | — |
| `Staff` | 3 | — |
| `Insurer` | 4 | — |
| `Researcher` | 5 | — |
| `Auditor` | 6 | — |
| `Service` | 7 | — |

### `enum RbacError`

| Variant | Value | Description |
|---|---|---|
| `Unauthorized` | 100 | — |
| `NotInitialized` | 300 | — |
| `AlreadyInitialized` | 301 | — |

### `struct Config`

| Field | Type | Description |
|---|---|---|
| `admin` | `Address` | — |
| `payment_router` | `Address` | — |
| `escrow_contract` | `Address` | — |
| `treasury` | `Address` | — |
| `token` | `Address` | — |
| `aml_contract` | `Address` | — |
| `rbac_contract` | `Address` | — |

### `enum DataKey`

| Variant | Value | Description |
|---|---|---|
| `Config` | — | — |
| `ClaimCount` | — | — |
| `Claim(u64)` | — | — |
| `PreAuthCount` | — | — |
| `PreAuth(u64)` | — | — |
| `PaymentPlanCount` | — | — |
| `PaymentPlan(u64)` | — | — |
| `FraudReport(u64)` | — | — |
| `InsuranceProviderCount` | — | — |
| `InsuranceProvider(u64)` | — | — |
| `CoveragePolicyCount` | — | — |
| `CoveragePolicy(u64)` | — | — |
| `PolicyByExternalId(String)` | — | — |
| `EligibilityCount` | — | — |
| `Eligibility(u64)` | — | — |
| `LatestEligibilityByPolicy(u64)` | — | — |
| `ClaimSubmission(u64)` | — | — |
| `CoverageEnrollmentCount` | — | — |
| `CoverageEnrollment(u64)` | — | — |
| `Eob(u64)` | — | — |
| `PatientResponsibility(Address)` | — | — |
| `CircuitBreakerState` | — | — |
| `AuthorizedPausers` | — | — |
| `Locked` | — | — |
| `CoverageProof(u64, Address)` | — | — |
| `CoverageProofCount` | — | — |


## Error Codes

| Variant | Code | Description |
|---|---|---|
| `Unauthorized` | 100 | — |
| `UnauthorizedCaller` | 101 | — |
| `NotAuthorizedPauser` | 102 | — |
| `InvalidAmount` | 205 | — |
| `InvalidSignature` | 207 | — |
| `InvalidCoverage` | 280 | — |
| `PolicyMismatch` | 281 | — |
| `NotInitialized` | 300 | — |
| `AlreadyInitialized` | 301 | — |
| `ContractPaused` | 302 | — |
| `CircuitOpen` | 303 | — |
| `InvalidStatus` | 304 | — |
| `AlreadyInState` | 305 | — |
| `DeadlineExceeded` | 306 | — |
| `ClaimNotFound` | 480 | — |
| `PreAuthNotFound` | 481 | — |
| `PaymentPlanNotFound` | 482 | — |
| `InsuranceProviderNotFound` | 483 | — |
| `CoveragePolicyNotFound` | 484 | — |
| `EligibilityCheckNotFound` | 485 | — |
| `ClaimSubmissionNotFound` | 486 | — |
| `EobNotFound` | 487 | — |
| `InsufficientFunds` | 500 | — |
| `StorageFull` | 502 | — |
| `FraudDetected` | 580 | — |
| `EscrowFailed` | 581 | — |
| `UnsupportedTransaction` | 582 | — |
| `CrossChainTimeout` | 702 | — |
| `Reentrancy` | 800 | — |

<!-- API_END -->

## Claim Status Flow

```
Submitted → Verified → Approved → Paid
                    ↘ Rejected
         ↘ Disputed
```

## Errors

See [Error Codes](error-codes.md) for the full list. Key errors:

| Name | Description |
|------|-------------|
| `ClaimNotFound` | No claim with given ID |
| `InvalidStatus` | Claim not in expected status |
| `CircuitOpen` | Circuit breaker is open |
| `Unauthorized` | Caller lacks permission |
