# Healthcare Payment

Contract: `healthcare_payment`

Handles insurance claim submission, pre-authorization, payment processing, and EDI integration.

## Security

Uses the **CEI pattern**: claim status is set to `Paid` and persisted to storage before token transfers are executed, preventing reentrancy.

## Key Functions

### `initialize(admin, payment_router, escrow_contract, treasury, token) → Result<(), Error>`

Initialize the contract configuration.

### `submit_claim(patient, provider, service_id, amount, policy_id, preauth_id) → Result<u64, Error>`

Submit an insurance claim for processing.

**Auth**: `provider`

**Returns**: `claim_id`

### `approve_claim(caller, claim_id) → Result<(), Error>`

Approve a submitted claim for payment.

**Auth**: admin

**State transition**: `Submitted/Verified → Approved`

### `process_payment(claim_id) → Result<(), Error>`

Process payment for an approved claim. Splits payment between provider and treasury via `payment_router`.

**State transition**: `Approved → Paid`

**Emits**: `CLAIM_PD`, `DIAG/ENTER`, `DIAG/EXIT`, `DIAG/STATE`

### `batch_process_payments(claim_ids) → Result<Vec<u64>, Error>`

Process multiple approved claims in a single transaction.

**Returns**: List of successfully paid claim IDs.

### `request_preauth(patient, provider, service_id, estimated_cost, expiry) → Result<u64, Error>`

Request pre-authorization for a service.

### `create_payment_plan(patient, provider, total_amount, installment_amount, frequency) → Result<u64, Error>`

Create an installment payment plan.

### `trip_circuit_breaker(caller) → Result<(), Error>`

Manually open the circuit breaker to halt payments.

**Auth**: admin or authorized pauser

### `reset_circuit_breaker(caller) → Result<(), Error>`

Reset the circuit breaker to resume operations.

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
