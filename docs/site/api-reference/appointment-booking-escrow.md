# Appointment Booking Escrow

Contract: `appointment_booking_escrow`

Manages appointment scheduling with escrow-backed payments. Funds are locked when an appointment is booked and released to the provider on confirmation, or refunded to the patient on cancellation.

## Security

This contract follows the **Checks-Effects-Interactions (CEI)** pattern: state is updated before any token transfer to prevent reentrancy attacks.

<!-- API_START -->

## Key Functions

| Function | Parameters | Returns | Description |
|---|---|---|---|
| `initialize` | `env: Env, admin: Address, _token: Address` | `Result<(), Error>` | Initialize the contract with an admin and token address |
| `book_appointment` | `env: Env, patient: Address, provider: Address, amount: i128, token: Address` | `Result<u64, Error>` | Book an appointment with payment locked in escrow Transfers `amount` from patient to contract and creates an appointment escrow |
| `confirm_appointment` | `env: Env, provider: Address, appointment_id: u64` | `Result<(), Error>` | Confirm appointment completion and release funds to provider Only the provider can confirm the appointment |
| `refund_appointment` | `env: Env, patient: Address, appointment_id: u64` | `Result<(), Error>` | Refund appointment if canceled Only the patient can request a refund Can only be done if appointment is still in Booked state (not Confirmed/Refunded) |
| `mark_no_show` | `env: Env, provider: Address, appointment_id: u64` | `Result<(), Error>` | Mark an appointment as a no-show (provider only). Only callable by the appointment's provider. No funds are released. |
| `send_reminder` | `env: Env, caller: Address, appointment_id: u64` | `Result<(), Error>` | Send an appointment reminder (provider or admin only). Records the timestamp when the reminder was last sent. |
| `get_appointment` | `env: Env, appointment_id: u64` | `Option<AppointmentEscrow>` | Get appointment details |
| `get_patient_appointments` | `env: Env, patient: Address` | `Vec<u64>` | Get all appointments for a patient |
| `get_provider_appointments` | `env: Env, provider: Address` | `Vec<u64>` | Get all appointments for a provider |
| `get_appointment_status` | `env: Env, appointment_id: u64` | `Result<AppointmentStatus, Error>` | Get appointment status |
| `get_escrow_balance` | `env: Env` | `i128` | Get escrow balance (should be equal to sum of all booked but not confirmed/refunded appointments) |
| `get_admin` | `env: Env` | `Result<Address, Error>` | Get the current admin |
| `health_check` | `env: Env` | `ContractHealth` | Get comprehensive health check |
| `set_paused` | `env: Env, admin: Address, paused: bool` | `Result<(), Error>` | Set pause status (admin only) |
| `is_paused` | `env: Env` | `bool` | Check if contract is paused |

## Types

### `enum AppointmentStatus`

| Variant | Value | Description |
|---|---|---|
| `Booked` | 0 | — |
| `Confirmed` | 1 | — |
| `Refunded` | 2 | — |
| `Completed` | 3 | — |
| `NoShow` | 4 | — |

### `struct AppointmentEscrow`

| Field | Type | Description |
|---|---|---|
| `appointment_id` | `u64` | — |
| `patient` | `Address` | — |
| `provider` | `Address` | — |
| `amount` | `i128` | — |
| `token` | `Address` | — |
| `booked_at` | `u64` | — |
| `scheduled_time` | `u64` | — |
| `confirmed_at` | `u64` | — |
| `refunded_at` | `u64` | — |
| `reminder_sent_at` | `u64` | — |
| `no_show_marked_at` | `u64` | — |
| `status` | `AppointmentStatus` | — |
| `funds_released` | `bool` | — |

### `enum DataKey`

| Variant | Value | Description |
|---|---|---|
| `Initialized` | — | — |
| `Admin` | — | — |
| `AppointmentCounter` | — | — |
| `Appointment(u64)` | — | — |
| `PatientAppointments(Address),  
    ProviderAppointments(Address), 
    Paused,
    LastActivity,
    TotalOperations,
    FailedOperations,
    Version,` | — | — |

### `struct ContractHealth`

| Field | Type | Description |
|---|---|---|
| `version` | `String` | — |
| `is_paused` | `bool` | — |
| `storage_usage` | `u64` | — |
| `last_activity` | `u64` | — |
| `total_operations` | `u64` | — |
| `failed_operations` | `u64` | — |
| `success_rate` | `u32` | — |
| `total_appointments` | `u64` | — |
| `active_escrow_balance` | `i128` | — |


## Error Codes

| Variant | Code | Description |
|---|---|---|
| `Unauthorized` | 100 | — |
| `OnlyPatientCanRefund` | 110 | — |
| `OnlyProviderCanConfirm` | 111 | — |
| `InvalidAmount` | 205 | — |
| `InvalidPatient` | 210 | — |
| `InvalidProvider` | 211 | — |
| `NotInitialized` | 300 | — |
| `AlreadyInitialized` | 301 | — |
| `InvalidState` | 304 | — |
| `AppointmentNotFound` | 410 | — |
| `AppointmentAlreadyConfirmed` | 411 | — |
| `AppointmentAlreadyRefunded` | 412 | — |
| `AppointmentNoShow` | 413 | — |
| `InsufficientFunds` | 500 | — |
| `TokenTransferFailed` | 501 | — |
| `DoubleWithdrawal` | 505 | — |

<!-- API_END -->
