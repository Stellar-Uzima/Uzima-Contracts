# Uzima Contracts — API Reference (from Interface Registry)

> Auto-generated from the contract interface registry. Do not edit manually.
>
> Source: `schemas/interface-registry/registry.json`

- **Registry version**: `1.0.0`
- **Generated**: `2026-07-24T06:26:41.435Z`
- **Contracts documented**: 4
- **Total functions**: 19

## Table of Contents

- [contract_monitoring](#contract-monitoring)
- [identity_registry](#identity-registry)
- [medical_records](#medical-records)
- [patient_consent_management](#patient-consent-management)

---

## contract_monitoring

Contract monitoring and metrics dashboard

**Version**: `1.0.0` | **Functions**: 4

### Functions

| Function | Parameters | Returns | Mutates State | Description |
|---|---|---|---|---|
| `get_dashboard` | `—` | `DashboardSnapshot` | No | Get dashboard snapshot |
| `initialize` | `admin: Address, alert_config: AlertConfig` | `void` | Yes | Initialize the monitoring contract |
| `record_call` | `caller: Address, function_name: String, gas_used: u64` | `void` | Yes | Record a successful function call |
| `record_error` | `function_name: String` | `void` | Yes | Record a failed function call |

### Function Details

#### `get_dashboard`

Get dashboard snapshot

- **Returns**: `DashboardSnapshot`
- **State mutation**: No

_No parameters._

#### `initialize`

Initialize the monitoring contract

- **Returns**: `void`
- **State mutation**: Yes

| Parameter | Type | Required | Description |
|---|---|---|---|
| `admin` | `Address` | Yes | — |
| `alert_config` | `AlertConfig` | Yes | — |

#### `record_call`

Record a successful function call

- **Returns**: `void`
- **State mutation**: Yes

| Parameter | Type | Required | Description |
|---|---|---|---|
| `caller` | `Address` | Yes | — |
| `function_name` | `String` | Yes | — |
| `gas_used` | `u64` | Yes | — |

#### `record_error`

Record a failed function call

- **Returns**: `void`
- **State mutation**: Yes

| Parameter | Type | Required | Description |
|---|---|---|---|
| `function_name` | `String` | Yes | — |

---

## identity_registry

Identity registry for healthcare providers and patients

**Version**: `1.0.0` | **Functions**: 4

### Functions

| Function | Parameters | Returns | Mutates State | Description |
|---|---|---|---|---|
| `initialize` | `admin: Address` | `void` | Yes | Initialize the identity registry |
| `register_identity` | `identity: Address, role: Symbol, metadata: String` | `void` | Yes | Register a new identity |
| `revoke_identity` | `identity: Address` | `void` | Yes | Revoke an identity |
| `verify_identity` | `identity: Address, role: Symbol` | `bool` | No | Verify an identity exists and has role |

### Function Details

#### `initialize`

Initialize the identity registry

- **Returns**: `void`
- **State mutation**: Yes

| Parameter | Type | Required | Description |
|---|---|---|---|
| `admin` | `Address` | Yes | — |

#### `register_identity`

Register a new identity

- **Returns**: `void`
- **State mutation**: Yes

| Parameter | Type | Required | Description |
|---|---|---|---|
| `identity` | `Address` | Yes | — |
| `role` | `Symbol` | Yes | — |
| `metadata` | `String` | Yes | — |

#### `revoke_identity`

Revoke an identity

- **Returns**: `void`
- **State mutation**: Yes

| Parameter | Type | Required | Description |
|---|---|---|---|
| `identity` | `Address` | Yes | — |

#### `verify_identity`

Verify an identity exists and has role

- **Returns**: `bool`
- **State mutation**: No

| Parameter | Type | Required | Description |
|---|---|---|---|
| `identity` | `Address` | Yes | — |
| `role` | `Symbol` | Yes | — |

---

## medical_records

Core medical records management contract

**Version**: `1.0.0` | **Functions**: 7

### Functions

| Function | Parameters | Returns | Mutates State | Description |
|---|---|---|---|---|
| `create_record` | `patient: Address, data: String, record_type: String` | `u64` | Yes | Create a new medical record |
| `delete_record` | `patient: Address, record_id: u64` | `void` | Yes | Soft-delete a medical record |
| `grant_access` | `patient: Address, provider: Address` | `void` | Yes | Grant a provider access to patient records |
| `initialize` | `admin: Address, rbac_id: Address` | `void` | Yes | Initialize the contract with admin and RBAC |
| `read_record` | `patient: Address, record_id: u64` | `MedicalRecord` | No | Read a medical record by ID |
| `revoke_access` | `patient: Address, provider: Address` | `void` | Yes | Revoke provider access to patient records |
| `update_record` | `patient: Address, record_id: u64, data: String` | `void` | Yes | Update an existing medical record |

### Function Details

#### `create_record`

Create a new medical record

- **Returns**: `u64`
- **State mutation**: Yes

| Parameter | Type | Required | Description |
|---|---|---|---|
| `patient` | `Address` | Yes | — |
| `data` | `String` | Yes | — |
| `record_type` | `String` | Yes | — |

#### `delete_record`

Soft-delete a medical record

- **Returns**: `void`
- **State mutation**: Yes

| Parameter | Type | Required | Description |
|---|---|---|---|
| `patient` | `Address` | Yes | — |
| `record_id` | `u64` | Yes | — |

#### `grant_access`

Grant a provider access to patient records

- **Returns**: `void`
- **State mutation**: Yes

| Parameter | Type | Required | Description |
|---|---|---|---|
| `patient` | `Address` | Yes | — |
| `provider` | `Address` | Yes | — |

#### `initialize`

Initialize the contract with admin and RBAC

- **Returns**: `void`
- **State mutation**: Yes

| Parameter | Type | Required | Description |
|---|---|---|---|
| `admin` | `Address` | Yes | — |
| `rbac_id` | `Address` | Yes | — |

#### `read_record`

Read a medical record by ID

- **Returns**: `MedicalRecord`
- **State mutation**: No

| Parameter | Type | Required | Description |
|---|---|---|---|
| `patient` | `Address` | Yes | — |
| `record_id` | `u64` | Yes | — |

#### `revoke_access`

Revoke provider access to patient records

- **Returns**: `void`
- **State mutation**: Yes

| Parameter | Type | Required | Description |
|---|---|---|---|
| `patient` | `Address` | Yes | — |
| `provider` | `Address` | Yes | — |

#### `update_record`

Update an existing medical record

- **Returns**: `void`
- **State mutation**: Yes

| Parameter | Type | Required | Description |
|---|---|---|---|
| `patient` | `Address` | Yes | — |
| `record_id` | `u64` | Yes | — |
| `data` | `String` | Yes | — |

---

## patient_consent_management

Patient consent management contract

**Version**: `1.0.0` | **Functions**: 4

### Functions

| Function | Parameters | Returns | Mutates State | Description |
|---|---|---|---|---|
| `check_consent` | `patient: Address, provider: Address, data_type: String` | `bool` | No | Check if consent is active |
| `grant_consent` | `patient: Address, provider: Address, data_type: String, expiry_ledger: u32` | `void` | Yes | Grant consent for data access |
| `initialize` | `admin: Address` | `void` | Yes | Initialize the consent contract |
| `revoke_consent` | `patient: Address, provider: Address, data_type: String` | `void` | Yes | Revoke previously granted consent |

### Function Details

#### `check_consent`

Check if consent is active

- **Returns**: `bool`
- **State mutation**: No

| Parameter | Type | Required | Description |
|---|---|---|---|
| `patient` | `Address` | Yes | — |
| `provider` | `Address` | Yes | — |
| `data_type` | `String` | Yes | — |

#### `grant_consent`

Grant consent for data access

- **Returns**: `void`
- **State mutation**: Yes

| Parameter | Type | Required | Description |
|---|---|---|---|
| `patient` | `Address` | Yes | — |
| `provider` | `Address` | Yes | — |
| `data_type` | `String` | Yes | — |
| `expiry_ledger` | `u32` | Yes | — |

#### `initialize`

Initialize the consent contract

- **Returns**: `void`
- **State mutation**: Yes

| Parameter | Type | Required | Description |
|---|---|---|---|
| `admin` | `Address` | Yes | — |

#### `revoke_consent`

Revoke previously granted consent

- **Returns**: `void`
- **State mutation**: Yes

| Parameter | Type | Required | Description |
|---|---|---|---|
| `patient` | `Address` | Yes | — |
| `provider` | `Address` | Yes | — |
| `data_type` | `String` | Yes | — |

---

