# `healthcare_payment` Contract — Public API Reference

## `initialize`
Initialises the contract. Must be called once before any other function.

**Parameters**
- `admin: Address` — contract administrator
- `payment_router: Address` — computes provider/fee splits
- `escrow_contract: Address` — holds disputed funds
- `treasury: Address` — receives protocol fees
- `token: Address` — SEP-41 token used for payments

**Returns** `Result<(), Error>`
**Errors** `AlreadyInitialized`

---

## `register_insurance_provider`
Registers a new insurance payer. Admin only.

**Parameters**
- `caller: Address` — must be admin
- `name: String` — human-readable payer name
- `payer_code: String` — unique payer identifier
- `supports_edi_837: bool` — accepts claim submissions
- `supports_edi_834: bool` — accepts enrollment syncs

**Returns** `Result<u64, Error>` — new provider ID
**Errors** `Unauthorized`, `InvalidCoverage`

---

## `register_coverage_policy`
Registers a patient coverage policy. Callable by patient or admin.

**Parameters**
- `caller: Address`
- `patient: Address`
- `insurance_provider_id: u64`
- `policy_external_id: String` — payer-assigned policy number
- `member_id: String`
- `group_number: String`
- `deductible_total: i128` — in token base units
- `copay_amount: i128`
- `coinsurance_bps: u32` — basis points (0–10 000)

**Returns** `Result<u64, Error>` — new policy ID
**Errors** `InsuranceProviderNotFound`, `InvalidCoverage`

---

## `submit_claim`
Submits a new healthcare claim. Called by the provider.

**Parameters**
- `patient: Address`
- `provider: Address` — must sign
- `service_id: String`
- `amount: i128` — must be > 0
- `policy_id: String` — external policy reference
- `preauth_id: Option<u64>`

**Returns** `Result<u64, Error>` — new claim ID
**Errors** `InvalidAmount`, `CircuitOpen`

---

## `verify_claim` / `approve_claim` / `reject_claim`
Advance a claim through the `Submitted → Verified → Approved / Rejected` lifecycle.

**Common parameters** `claim_id: u64`, `caller: Address`
`reject_claim` additionally takes `reason: String`.

**Errors** `ClaimNotFound`, `InvalidStatus`, `FraudDetected` (approve only)

---

## `process_payment`
Transfers funds to provider and treasury for an `Approved` claim.
Follows CEI pattern — state updated before token transfers.

**Parameters** `claim_id: u64`
**Errors** `ClaimNotFound`, `InvalidStatus`, `NotInitialized`, `CircuitOpen`

---

## `emergency_pause` / `begin_recovery` / `resume_operations`
Circuit-breaker controls. `emergency_pause` callable by admin or authorised pausers.
`begin_recovery` and `resume_operations` are admin-only.

---

## `get_coverage_policy` / `get_eligibility_check` / `get_claim_submission`
Read-only getters. Return the stored struct or the appropriate `Error::*NotFound`.