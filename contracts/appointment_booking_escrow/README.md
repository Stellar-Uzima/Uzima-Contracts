# Appointment Booking Escrow Contract

A Soroban smart contract implementing an escrow system for booking medical appointments. Ensures fair payment handling by locking patient funds until appointment completion, providing secure fund release to providers or refunds to patients.

## Overview

The Appointment Booking Escrow contract solves the trust problem in healthcare services by:
- Securely holding patient payment in contract until appointment completion
- Releasing funds to provider only after confirmed appointment delivery
- Allowing patient refunds if appointment is canceled before completion
- Preventing double withdrawal and unauthorized fund access
- Creating immutable audit trails of all escrow transactions

### Key Features

- **Secure Payment Lock**: Patient funds transferred to contract at booking time
- **Conditional Fund Release**: Funds only released when provider confirms appointment
- **Patient Refunds**: Full refund available if appointment not confirmed
- **Anti-Fraud Protection**: Double withdrawal prevention with state tracking
- **Complete Audit Trail**: All transactions timestamped on blockchain
- **Event Tracking**: Full event emission for off-chain monitoring
- **Multi-Appointment Support**: Patients and providers can have multiple concurrent appointments

## Contract Functions

### Core Operations

#### `initialize(env: Env, admin: Address, token: Address) -> Result<(), Error>`
Initializes the contract with an administrator and token address.

**Parameters:**
- `admin`: The address to be set as the contract administrator
- `token`: The token address used for appointment payments

**Returns:** Success or `AlreadyInitialized` error if already initialized

**Requirements:**
- `admin` must have authorization (require_auth)

**Events:** Publishes `INIT` event

---

#### `book_appointment(env: Env, patient: Address, provider: Address, amount: i128, token: Address) -> Result<u64, Error>`
Books a new appointment and locks patient payment in escrow.

**Parameters:**
- `patient`: The patient's address (must match caller for authorization)
- `provider`: The healthcare provider's address
- `amount`: Payment amount in token units (must be > 0)
- `token`: The token contract address for payment

**Returns:** Successfully booked appointment ID or error

**Errors:**
- `NotInitialized`: Contract not yet initialized
- `InvalidAmount`: Amount <= 0
- `InvalidProvider`: Patient attempted to book with themselves
- `TokenTransferFailed`: Token transfer from patient failed

**Flow:**
1. Validates amount and addresses
2. Transfers `amount` from patient to contract (locks funds)
3. Creates appointment escrow record with status=Booked
4. Adds appointment to patient's and provider's lists
5. Returns unique appointment ID

**Acceptance Criteria Met:**
- ✅ Funds are securely held in contract
- ✅ Only patient can book (require_auth)

**Events:** Publishes `BOOK` event with (appointment_id, patient, provider, amount, timestamp)

---

#### `confirm_appointment(env: Env, provider: Address, appointment_id: u64) -> Result<(), Error>`
Confirms appointment completion and releases locked funds to provider.

**Parameters:**
- `provider`: The provider's address (must match appointment provider)
- `appointment_id`: The ID of the appointment to confirm

**Returns:** Success or error

**Errors:**
- `NotInitialized`: Contract not yet initialized
- `AppointmentNotFound`: Appointment ID doesn't exist
- `OnlyProviderCanConfirm`: Caller is not the appointment's provider
- `AppointmentAlreadyConfirmed`: Appointment already confirmed
- `AppointmentAlreadyRefunded`: Appointment was already refunded
- `DoubleWithdrawal`: Funds already released (safety protection)
- `TokenTransferFailed`: Token transfer to provider failed

**Requirements:**
- `provider` must have authorization (require_auth)
- Must be the provider of the appointment
- Appointment must be in Booked state
- Funds must not have been previously released

**Flow:**
1. Validates appointment exists and caller is provider
2. Checks appointment is still in Booked state
3. Prevents double withdrawal via `funds_released` flag
4. Transfers full `amount` from contract to provider
5. Sets status to Completed and marks funds as released
6. Stores updated appointment record

**Acceptance Criteria Met:**
- ✅ Only valid conditions trigger release (confirmed by provider)
- ✅ Prevent double withdrawal (funds_released flag + state checks)

**Events:** 
- `CONF` event: (appointment_id, provider, timestamp)
- `RELEASE` event: (appointment_id, provider, amount, timestamp)

---

#### `refund_appointment(env: Env, patient: Address, appointment_id: u64) -> Result<(), Error>`
Refunds patient payment if appointment is canceled before completion.

**Parameters:**
- `patient`: The patient's address (must match appointment patient)
- `appointment_id`: The ID of the appointment to refund

**Returns:** Success or error

**Errors:**
- `NotInitialized`: Contract not yet initialized
- `AppointmentNotFound`: Appointment ID doesn't exist
- `OnlyPatientCanRefund`: Caller is not the appointment's patient
- `AppointmentAlreadyRefunded`: Appointment already refunded
- `InvalidState`: Appointment already confirmed or completed
- `DoubleWithdrawal`: Funds already released
- `TokenTransferFailed`: Token transfer to patient failed

**Requirements:**
- `patient` must have authorization (require_auth)
- Must be the patient of the appointment
- Appointment must still be in Booked state (not Confirmed/Refunded)
- Funds must not have been previously released

**Flow:**
1. Validates appointment exists and caller is patient
2. Checks appointment is still in Booked state
3. Prevents double withdrawal via `funds_released` flag
4. Transfers full `amount` from contract back to patient
5. Sets status to Refunded and marks funds as released
6. Stores updated appointment record

**Acceptance Criteria Met:**
- ✅ Only valid conditions trigger refund (booked, not confirmed)
- ✅ Prevent double withdrawal (funds_released flag + state checks)

**Events:** Publishes `REFUND` event with (appointment_id, patient, amount, timestamp)

---

### Query Operations

#### `get_appointment(env: Env, appointment_id: u64) -> Option<AppointmentEscrow>`
Retrieves complete details of a specific appointment.

**Parameters:**
- `appointment_id`: The appointment ID to retrieve

**Returns:** Complete appointment record or None if not found

**Data Structure:**
```rust
pub struct AppointmentEscrow {
    pub appointment_id: u64,
    pub patient: Address,
    pub provider: Address,
    pub amount: i128,
    pub token: Address,
    pub booked_at: u64,
    pub confirmed_at: u64,      // 0 if not confirmed
    pub refunded_at: u64,        // 0 if not refunded
    pub status: AppointmentStatus,
    pub funds_released: bool,     // Prevents double withdrawal
}
```

---

#### `get_appointment_status(env: Env, appointment_id: u64) -> Result<AppointmentStatus, Error>`
Gets the current status of an appointment.

**Parameters:**
- `appointment_id`: The appointment ID to check

**Returns:** Current `AppointmentStatus` or `AppointmentNotFound` error

**Status Values:**
```rust
pub enum AppointmentStatus {
    Booked = 0,      // Awaiting provider confirmation
    Confirmed = 1,   // (Deprecated - use Completed)
    Refunded = 2,    // Payment refunded to patient
    Completed = 3,   // Appointment completed, funds released
}
```

---

#### `get_patient_appointments(env: Env, patient: Address) -> Vec<u64>`
Retrieves all appointment IDs for a patient (both active and settled).

**Parameters:**
- `patient`: The patient's address

**Returns:** Vector of all appointment IDs for this patient

---

#### `get_provider_appointments(env: Env, provider: Address) -> Vec<u64>`
Retrieves all appointment IDs for a provider (both pending and settled).

**Parameters:**
- `provider`: The provider's address

**Returns:** Vector of all appointment IDs for this provider

---

#### `get_escrow_balance(env: Env) -> i128`
Calculates total funds currently held in escrow.

**Returns:** Sum of amounts for all Booked appointments that haven't been released

**Note:** This iterates through all appointments, so use carefully with large numbers

---

#### `get_admin(env: Env) -> Result<Address, Error>`
Retrieves the current contract administrator.

**Returns:** Admin address or `NotInitialized` error

---

## Data Structures

### AppointmentEscrow
```rust
pub struct AppointmentEscrow {
    pub appointment_id: u64,          // Unique appointment identifier
    pub patient: Address,              // Patient's wallet address
    pub provider: Address,             // Provider's wallet address
    pub amount: i128,                  // Payment amount (in token units)
    pub token: Address,                // Token contract address
    pub booked_at: u64,                // Block timestamp when booked
    pub confirmed_at: u64,             // Block timestamp when confirmed (0 if not)
    pub refunded_at: u64,              // Block timestamp when refunded (0 if not)
    pub status: AppointmentStatus,     // Current state
    pub funds_released: bool,          // Anti-double-withdrawal flag
}
```

### AppointmentStatus
```rust
pub enum AppointmentStatus {
    Booked = 0,      // Initial state - funds locked
    Confirmed = 1,   // (Deprecated)
    Refunded = 2,    // Funds returned to patient
    Completed = 3,   // Funds released to provider
}
```

## Error Codes

| Error | Code | Description |
|-------|------|-------------|
| `NotInitialized` | 1 | Contract has not been initialized |
| `AlreadyInitialized` | 2 | Contract already initialized |
| `NotAuthorized` | 3 | Caller lacks required authorization |
| `InvalidPatient` | 4 | Invalid patient address |
| `InvalidProvider` | 5 | Invalid provider address (e.g., same as patient) |
| `InvalidAmount` | 6 | Amount is <= 0 |
| `AppointmentNotFound` | 7 | Appointment doesn't exist |
| `AppointmentAlreadyConfirmed` | 8 | Already confirmed |
| `AppointmentAlreadyRefunded` | 9 | Already refunded |
| `InsufficientFunds` | 10 | Patient has insufficient balance |
| `TokenTransferFailed` | 11 | Token transfer failed |
| `InvalidState` | 12 | Operation invalid for current state |
| `DoubleWithdrawal` | 13 | Funds already released (security check) |
| `OnlyPatientCanRefund` | 14 | Caller is not the patient |
| `OnlyProviderCanConfirm` | 15 | Caller is not the provider |

## Events

The contract publishes the following events for compliance and monitoring:

- **`INIT`**: Contract initialization - emits admin address
- **`BOOK`**: Appointment booked - emits (appointment_id, patient, provider, amount, timestamp)
- **`CONF`**: Appointment confirmed - emits (appointment_id, provider, timestamp)
- **`RELEASE`**: Funds released - emits (appointment_id, provider, amount, timestamp)
- **`REFUND`**: Refund issued - emits (appointment_id, patient, amount, timestamp)

## Acceptance Criteria Fulfillment

✅ **Funds are securely held in contract**
- Patient calls `book_appointment()`, which immediately transfers funds to contract
- Funds remain in contract until either confirmed or refunded
- Persistent storage ensures durability

✅ **Only valid conditions trigger release/refund**
- Release: Only provider can confirm, appointment must be in Booked state
- Refund: Only patient can request, appointment must be in Booked state (not confirmed)
- Double withdrawal prevented with `funds_released` boolean flag

✅ **Prevent double withdrawal**
- `funds_released` flag set when funds move in either direction
- Multiple checks prevent state transitions after funds released
- Appointment status enums enforce valid state transitions
- All operations guarded by appointment existence and state checks

## Storage Architecture

- **Instance Storage** (immutable):
  - Admin address and initialization flag
  - Appointment counter for ID generation
  
- **Persistent Storage** (durable):
  - Individual appointment records keyed by ID
  - Patient → Appointment ID mapping for patient queries
  - Provider → Appointment ID mapping for provider queries
  - Enables efficient lookups for all stakeholder queries

## Security Considerations

1. **Patient Authorization**: `patient.require_auth()` on booking and refund operations
2. **Provider Authorization**: `provider.require_auth()` on confirmation operations
3. **Double Withdrawal Prevention**: 
   - `funds_released` flag prevents re-execution
   - Status checks ensure state validity
   - Funds can only be moved once
4. **Anti-Tampering**: Blockchain timestamps prevent timestamp manipulation
5. **Token Safety**: Uses standard Soroban token transfer interface
6. **Escrow Invariants**:
   - Funds locked: patient → contract
   - Funds released: contract → provider OR contract → patient (never both)
   - No funds created or destroyed

## Workflow Examples

### Complete Appointment (Happy Path)
```rust
// Patient books appointment with 500 tokens payment
let appt_id = escrow.book_appointment(&patient, &provider, &500, &token)?;

// Later: Provider confirms completion
escrow.confirm_appointment(&provider, &appt_id)?;
// Funds automatically transferred to provider
```

### Canceled Appointment (Refund Path)
```rust
// Patient books appointment
let appt_id = escrow.book_appointment(&patient, &provider, &500, &token)?;

// Patient decides to cancel
escrow.refund_appointment(&patient, &appt_id)?;
// Funds returned to patient
```

### Multiple Providers
```rust
// Patient has appointments with different providers
let appt1 = escrow.book_appointment(&patient, &provider_a, &500, &token)?;
let appt2 = escrow.book_appointment(&patient, &provider_b, &300, &token)?;

// Independently manage each
escrow.confirm_appointment(&provider_a, &appt1)?;
escrow.refund_appointment(&patient, &appt2)?;

// Get all patient appointments
let patient_appts = escrow.get_patient_appointments(&patient);
```

### Provider Workflow
```rust
// Get all provider's pending appointments
let provider_appts = escrow.get_provider_appointments(&provider);

// For each appointment
for appt_id in provider_appts {
    let appointment = escrow.get_appointment(&appt_id)?;
    if appointment.status == AppointmentStatus::Booked {
        // Service delivered, confirm completion
        escrow.confirm_appointment(&provider, &appt_id)?;
    }
}
```

## Testing

Comprehensive tests in `src/test.rs` (20+ tests) covering:

- Contract initialization and re-initialization prevention
- Appointment booking with valid amounts
- Invalid amount handling (zero, negative)
- Self-booking prevention
- Multiple appointment ID generation
- Provider confirmation workflow
- Wrong provider confirmation prevention
- Double confirmation prevention
- Patient refund workflow
- Wrong patient refund prevention
- Refund after confirmation prevention (state validation)
- Double refund prevention
- Appointment retrieval and status queries
- Patient and provider appointment lists
- Escrow balance calculation
- Complete state transition scenarios

Run tests with:
```bash
cd contracts/appointment_booking_escrow
cargo test --lib test::tests
```

## HIPAA & Healthcare Compliance

1. **Non-Repudiation**: Blockchain timestamps prove when events occurred
2. **Audit Trail**: All transactions recorded immutably
3. **Patient Control**: Patients retain control over refunds
4. **Provider Incentive**: Direct fund transfer incentivizes service delivery
5. **Dispute Prevention**: Clear state transitions prevent ambiguity
6. **Regulatory Ready**: Can integrate with compliance monitoring systems

## Related Contracts

- `patient_consent_management`: Consent requirements before service
- `medical_records`: Patient medical record access and management
- `medical_record_hash_registry`: Record integrity verification
- `reputation`: Provider reputation system with appointment history

## Future Enhancements

- Dispute resolution system for refund disagreements
- Partial refunds for service cancellations
- Appointment time slots with automatic expiration
- Provider deposits and collateral requirements
- Multi-party signature requirements
- Insurance payment integration
- Appointment rescheduling support
- Time-locked escrow releases
- Service quality ratings and reviews
- Integration with zero-knowledge proofs for privacy
