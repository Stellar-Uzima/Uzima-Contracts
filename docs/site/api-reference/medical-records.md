# Medical Records

Contract: `medical_records`

Encrypted on-chain medical record storage with role-based access control, patient consent management, and full audit trail.

## Key Functions

### `initialize(admin) → Result<(), Error>`

Initialize the contract.

### `register_patient(patient_id, public_key) → Result<(), Error>`

Register a new patient with their public key.

### `write_record(patient_id, doctor_id, encrypted_data, metadata) → Result<u64, Error>`

Store an encrypted medical record. Data must be encrypted client-side before submission.

**Auth**: `doctor_id`

### `read_record(record_id, requester) → Result<EncryptedRecord, Error>`

Read a medical record. Requester must have patient consent.

**Auth**: `requester`

### `grant_access(patient, grantee, record_id) → Result<(), Error>`

Grant a provider access to a specific record.

**Auth**: `patient`

### `revoke_access(patient, grantee, record_id) → Result<(), Error>`

Revoke a provider's access to a record.

**Auth**: `patient`

### `get_patient_records(patient_id) → Vec<u64>`

List all record IDs for a patient.

## Access Control

Records are protected by patient consent. Only the patient can grant/revoke access. All access is logged to the audit contract.

## Traditional Medicine Support

Records support a `metadata` field for traditional healing practice annotations.
