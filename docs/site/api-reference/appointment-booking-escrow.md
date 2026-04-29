# Appointment Booking Escrow

Contract: `appointment_booking_escrow`

Manages appointment scheduling with escrow-backed payments. Funds are locked when an appointment is booked and released to the provider on confirmation, or refunded to the patient on cancellation.

## Security

This contract follows the **Checks-Effects-Interactions (CEI)** pattern: state is updated before any token transfer to prevent reentrancy attacks.

## Functions

### `initialize(admin, token) → Result<(), Error>`

Initialize the contract. Can only be called once.

| Param | Type | Description |
|-------|------|-------------|
| `admin` | `Address` | Contract administrator |
| `token` | `Address` | Default token address |

### `book_appointment(patient, provider, amount, token) → Result<u64, Error>`

Book an appointment and lock funds in escrow. Transfers `amount` from `patient` to the contract.

**Auth**: `patient`

| Param | Type | Description |
|-------|------|-------------|
| `patient` | `Address` | Patient booking the appointment |
| `provider` | `Address` | Healthcare provider |
| `amount` | `i128` | Payment amount (must be > 0) |
| `token` | `Address` | Token contract address |

**Returns**: `appointment_id`

**Emits**: `APPT/BOOK`, `DIAG/ENTER`, `DIAG/EXIT`

### `confirm_appointment(provider, appointment_id) → Result<(), Error>`

Confirm appointment completion and release funds to provider.

**Auth**: `provider`

**State transition**: `Booked → Completed`

**Emits**: `APPT/CONF`, `APPT/RELEASE`, `DIAG/STATE`

### `refund_appointment(patient, appointment_id) → Result<(), Error>`

Cancel appointment and refund funds to patient. Only valid for `Booked` appointments.

**Auth**: `patient`

**State transition**: `Booked → Refunded`

**Emits**: `APPT/REFUND`, `DIAG/STATE`

### `get_appointment(appointment_id) → Option<AppointmentEscrow>`

Get appointment details.

### `get_patient_appointments(patient) → Vec<u64>`

Get all appointment IDs for a patient.

### `get_provider_appointments(provider) → Vec<u64>`

Get all appointment IDs for a provider.

### `get_escrow_balance() → i128`

Get total funds currently locked in escrow.

### `health_check() → ContractHealth`

Get contract health metrics including success rate and active escrow balance.

## Errors

| Code | Name | Description |
|------|------|-------------|
| — | `AppointmentNotFound` | No appointment with given ID |
| — | `AppointmentAlreadyConfirmed` | Already confirmed/completed |
| — | `AppointmentAlreadyRefunded` | Already refunded |
| — | `OnlyProviderCanConfirm` | Caller is not the provider |
| — | `OnlyPatientCanRefund` | Caller is not the patient |
| — | `DoubleWithdrawal` | Funds already released |
| — | `InvalidAmount` | Amount must be > 0 |
