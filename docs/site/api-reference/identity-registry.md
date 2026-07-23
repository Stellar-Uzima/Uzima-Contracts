# Identity Registry

Contract: `identity_registry`

W3C DID-based identity management and credential verification for healthcare providers and patients.

<!-- API_START -->

## Key Functions

| Function | Parameters | Returns | Description |
|---|---|---|---|
| `initialize` | `env: Env, owner: Address, network_id: String, rbac_contract: Address` | `Result<(), Error>` | Initialize the contract with an owner and network identifier |
| `health_check` | `env: Env` | `(Symbol, u32, u64)` | Perform a health check on the contract |
| `is_paused` | `env: Env` | `bool` | Returns true if the contract is currently paused. |
| `pause` | `env: Env, caller: Address` | `Result<bool, Error>` | — |
| `unpause` | `env: Env, caller: Address` | `Result<bool, Error>` | — |
| `initialize_legacy` | `env: Env, owner: Address, rbac_contract: Address` | `()` | Legacy initialize for backward compatibility |
| `create_did` | `env: Env, subject: Address, primary_public_key: BytesN<32>, services: Vec<ServiceEndpoint>` | `Result<String, Error>` | Create a new DID Document for a subject Only the subject can create their own DID |
| `resolve_did` | `env: Env, subject: Address` | `Result<DIDDocument, Error>` | Resolve a DID Document by subject address |
| `resolve_did_by_string` | `env: Env, did_string: String` | `Result<DIDDocument, Error>` | Resolve a DID Document by DID string |
| `update_did` | `env: Env, subject: Address, new_services: Vec<ServiceEndpoint>, new_also_known_as: Vec<String>` | `Result<(), Error>` | Update DID Document (add/modify services, also_known_as) |
| `deactivate_did` | `env: Env, subject: Address` | `Result<(), Error>` | Deactivate a DID (soft delete) |
| `add_verification_method` | `env: Env, subject: Address, method_id: String, method_type: VerificationMethodType, public_key: BytesN<32>, relationships: Vec<VerificationRelationship>` | `Result<(), Error>` | Add a new verification method to a DID |
| `rotate_key` | `env: Env, subject: Address, method_id: String, new_public_key: BytesN<32>` | `Result<(), Error>` | Rotate a verification method key |
| `revoke_verification_method` | `env: Env, subject: Address, method_id: String` | `Result<(), Error>` | Revoke/deactivate a verification method |
| `issue_credential` | `env: Env, issuer: Address, subject: Address, credential_type: CredentialType, credential_hash: BytesN<32>, credential_uri: String, expiration_date: u64` | `Result<BytesN<32>, Error>` | Issue a verifiable credential (only verifiers/issuers can do this) |
| `verify_credential` | `env: Env, credential_id: BytesN<32>` | `Result<CredentialStatus, Error>` | Verify a credential's status |
| `get_credential` | `env: Env, credential_id: BytesN<32>` | `Result<VerifiableCredential, Error>` | Get a credential by ID |
| `revoke_credential` | `env: Env, issuer: Address, credential_id: BytesN<32>, reason: String` | `Result<(), Error>` | Revoke a credential (only issuer can revoke) |
| `get_subject_credentials` | `env: Env, subject: Address` | `Vec<VerifiableCredential>` | Get all credentials for a subject |
| `has_valid_credential` | `env: Env, subject: Address, credential_type: CredentialType` | `bool` | Verify if subject has a valid credential of a specific type |
| `add_recovery_guardian` | `env: Env, subject: Address, guardian: Address, weight: u32` | `Result<(), Error>` | Add a recovery guardian |
| `remove_recovery_guardian` | `env: Env, subject: Address, guardian: Address` | `Result<(), Error>` | Remove a recovery guardian |
| `set_recovery_threshold` | `env: Env, subject: Address, threshold: u32` | `Result<(), Error>` | Set recovery threshold |
| `initiate_recovery` | `env: Env, guardian: Address, subject: Address, new_controller: Address, new_primary_key: BytesN<32>` | `Result<u64, Error>` | Initiate identity recovery |
| `approve_recovery` | `env: Env, guardian: Address, request_id: u64` | `Result<(), Error>` | Approve a recovery request |
| `execute_recovery` | `env: Env, request_id: u64` | `Result<(), Error>` | Execute recovery after timelock and threshold met |
| `cancel_recovery` | `env: Env, subject: Address` | `Result<(), Error>` | Cancel a recovery request (only subject with existing key) |
| `add_service` | `env: Env, subject: Address, service_id: String, service_type: String, endpoint: String` | `Result<(), Error>` | Add a service endpoint to DID |
| `remove_service` | `env: Env, subject: Address, service_id: String` | `Result<(), Error>` | Remove/deactivate a service endpoint |
| `add_verifier` | `env: Env, verifier: Address` | `Result<(), Error>` | Add a verifier (only owner can do this) |
| `remove_verifier` | `env: Env, verifier: Address` | `Result<(), Error>` | Remove a verifier (only owner can do this) |
| `is_verifier` | `env: Env, account: Address` | `bool` | Check if an address is a verifier |
| `get_owner` | `env: Env` | `Result<Address, Error>` | Get the contract owner |
| `register_identity_hash` | `env: Env, hash: BytesN<32>, subject: Address, meta: String` | `()` | Register an identity hash with metadata (legacy support) |
| `attest` | `env: Env, verifier: Address, subject: Address, claim_hash: BytesN<32>` | `()` | Create an attestation (legacy - only verifiers can do this) |
| `revoke_attestation` | `env: Env, verifier: Address, subject: Address, claim_hash: BytesN<32>` | `()` | Revoke an attestation (legacy) |
| `get_identity_hash` | `env: Env, subject: Address` | `Option<BytesN<32>>` | Get identity hash for a subject (legacy) |
| `get_identity_meta` | `env: Env, subject: Address` | `Option<String>` | Get identity metadata for a subject (legacy) |
| `is_attested` | `env: Env, subject: Address, claim_hash: BytesN<32>` | `bool` | Check if a specific attestation is active (legacy) |
| `get_attestations` | `env: Env, subject: Address` | `Vec<BytesN<32>>` | Get all active attestations for a subject (legacy) |
| `verify_did_authorization` | `env: Env, subject: Address, required_relationship: VerificationRelationship` | `bool` | DID-based authorization check |
| `add_fido2_device` | `env: Env, subject: Address, device_name: String, algorithm_tag: u32, public_key_hash: BytesN<32>` | `Result<(), Error>` | Registers a FIDO2 / WebAuthn authenticator device as a verification method in the subject's DID document.  Called by the `fido2_authenticator` contract after a successful device registration ceremony.  The public key is stored as a SHA-256 hash (`public_key_hash`) because DID verification methods use 32-byte keys and FIDO2 P-256 keys are 65 bytes; the hash acts as a stable, compact identifier.  # Arguments * `subject`          — DID owner; must have an active DID document. * `device_name`      — friendly name used as the verification method fragment ID. * `algorithm_tag`    — 1 = EdDSA (Ed25519), 2 = ES256 (P-256). * `public_key_hash`  — SHA-256 of the raw authenticator public key bytes.  If the subject has no DID document the call is silently ignored so that the `fido2_authenticator` registration is never blocked by DID state. |
| `deposit_stake` | `env: Env, provider: Address, amount: i128, token_address: Address` | `Result<(), Error>` | Deposit stake for a healthcare provider. |
| `withdraw_stake` | `env: Env, provider: Address` | `Result<i128, Error>` | Withdraw stake after lock period if not slashed and in good standing. |
| `slash_stake` | `env: Env, governance: Address, provider: Address, amount: i128, reason: String` | `Result<(), Error>` | Slash stake for verified misconduct (governance only). |
| `init_mock` | `_env: Env` | `()` | — |
| `has_role` | `env: Env, address: Address, role: RbacRole` | `Result<bool, RbacError>` | — |
| `assign_role` | `env: Env, address: Address, role: RbacRole` | `Result<bool, RbacError>` | — |
| `remove_role` | `env: Env, address: Address, role: RbacRole` | `Result<bool, RbacError>` | — |

## Types

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

### `enum VerificationMethodType`

| Variant | Value | Description |
|---|---|---|
| `Ed25519VerificationKey2020` | — | — |
| `EcdsaSecp256k1VerifKey2019` | — | — |
| `X25519KeyAgreementKey2020` | — | — |
| `JsonWebKey2020` | — | — |
| `Fido2EdDsa2024` | — | — |
| `Fido2Es2562024` | — | — |

### `enum VerificationRelationship`

| Variant | Value | Description |
|---|---|---|
| `Authentication` | — | — |
| `AssertionMethod` | — | — |
| `KeyAgreement` | — | — |
| `CapabilityInvocation` | — | — |
| `CapabilityDelegation` | — | — |

### `struct VerificationMethod`

| Field | Type | Description |
|---|---|---|
| `id` | `String` | — |
| `method_type` | `VerificationMethodType` | — |
| `controller` | `Address` | — |
| `public_key` | `BytesN<32>` | — |
| `is_active` | `bool` | — |
| `created` | `u64` | — |
| `last_rotated` | `u64` | — |

### `struct ServiceEndpoint`

| Field | Type | Description |
|---|---|---|
| `id` | `String` | — |
| `service_type` | `String` | — |
| `endpoint` | `String` | — |
| `is_active` | `bool` | — |

### `enum DIDStatus`

| Variant | Value | Description |
|---|---|---|
| `Active` | — | — |
| `Deactivated` | — | — |
| `RecoveryPending` | — | — |

### `struct DIDDocument`

| Field | Type | Description |
|---|---|---|
| `id` | `String` | — |
| `controller` | `Address` | — |
| `also_known_as` | `Vec<String>` | — |
| `verification_methods` | `Vec<VerificationMethod>` | — |
| `authentication` | `Vec<String>` | — |
| `assertion_method` | `Vec<String>` | — |
| `key_agreement` | `Vec<String>` | — |
| `capability_invocation` | `Vec<String>` | — |
| `capability_delegation` | `Vec<String>` | — |
| `services` | `Vec<ServiceEndpoint>` | — |
| `status` | `DIDStatus` | — |
| `created` | `u64` | — |
| `updated` | `u64` | — |
| `version` | `u32` | — |
| `previous_hash` | `BytesN<32>` | — |

### `enum CredentialType`

| Variant | Value | Description |
|---|---|---|
| `MedicalLicense` | — | — |
| `SpecialistCertification` | — | — |
| `HospitalAffiliation` | — | — |
| `ResearchAuthorization` | — | — |
| `PatientConsent` | — | — |
| `EmergencyAccess` | — | — |
| `DataAccessPermission` | — | — |

### `struct VerifiableCredential`

| Field | Type | Description |
|---|---|---|
| `id` | `BytesN<32>` | — |
| `credential_type` | `CredentialType` | — |
| `issuer` | `Address` | — |
| `subject` | `Address` | — |
| `issuance_date` | `u64` | — |
| `expiration_date` | `u64` | — |
| `credential_hash` | `BytesN<32>` | — |
| `credential_uri` | `String` | — |
| `is_revoked` | `bool` | — |
| `revoked_at` | `u64` | — |
| `revocation_reason` | `String` | — |

### `enum CredentialStatus`

| Variant | Value | Description |
|---|---|---|
| `Valid` | — | — |
| `Revoked` | — | — |
| `Expired` | — | — |
| `NotFound` | — | — |

### `struct RecoveryGuardian`

| Field | Type | Description |
|---|---|---|
| `address` | `Address` | — |
| `weight` | `u32` | — |
| `added_at` | `u64` | — |

### `struct RecoveryRequest`

| Field | Type | Description |
|---|---|---|
| `request_id` | `u64` | — |
| `subject` | `Address` | — |
| `new_controller` | `Address` | — |
| `new_primary_key` | `BytesN<32>` | — |
| `created_at` | `u64` | — |
| `approvals` | `Vec<Address>` | — |
| `total_weight` | `u32` | — |
| `executed` | `bool` | — |

### `struct IdentityRecord`

| Field | Type | Description |
|---|---|---|
| `hash` | `BytesN<32>` | — |
| `meta` | `String` | — |
| `registered_by` | `Address` | — |

### `struct Attestation`

| Field | Type | Description |
|---|---|---|
| `claim_hash` | `BytesN<32>` | — |
| `verifier` | `Address` | — |
| `is_active` | `bool` | — |

### `struct ProviderStake`

| Field | Type | Description |
|---|---|---|
| `provider` | `Address` | — |
| `token_address` | `Address` | — |
| `amount` | `i128` | — |
| `locked_until` | `u64` | — |
| `slashed` | `bool` | — |
| `deposited_at` | `u64` | — |

### `enum DataKey`

| Variant | Value | Description |
|---|---|---|
| `Owner` | — | — |
| `Initialized` | — | — |
| `NetworkId` | — | — |
| `RbacContract` | — | — |
| `Paused` | — | — |
| `Verifier(Address)` | — | — |
| `IdentityHash(Address)` | — | — |
| `Attestation(Address, BytesN<32>)` | — | — |
| `SubjectAttestations(Address)` | — | — |
| `DIDDocument(Address)` | — | — |
| `DIDByString(String)` | — | — |
| `VerificationMethod(Address, String)` | — | — |
| `Credential(BytesN<32>)` | — | — |
| `SubjectCredentials(Address)` | — | — |
| `IssuerCredentials(Address)` | — | — |
| `CredentialsByType(Address, CredentialType)` | — | — |
| `RecoveryGuardians(Address)` | — | — |
| `RecoveryThreshold(Address)` | — | — |
| `RecoveryRequest(u64)` | — | — |
| `ActiveRecovery(Address)` | — | — |
| `RecoveryCounter` | — | — |
| `LastKeyRotation(Address)` | — | — |
| `KeyRotationCooldown` | — | — |
| `StakeInfo(Address)` | — | — |


## Error Codes

| Variant | Code | Description |
|---|---|---|
| `Unauthorized` | 100 | — |
| `NotVerifier` | 110 | — |
| `CannotRemoveOwner` | 111 | — |
| `InvalidRecoveryGuardian` | 120 | — |
| `InsufficientGuardianApprovals` | 121 | — |
| `InvalidInput` | 200 | — |
| `InputTooLong` | 201 | — |
| `InvalidVerificationMethod` | 250 | — |
| `InvalidCredentialType` | 251 | — |
| `InvalidServiceEndpoint` | 252 | — |
| `NotInitialized` | 300 | — |
| `AlreadyInitialized` | 301 | — |
| `ContractPaused` | 302 | — |
| `RecoveryNotInitiated` | 360 | — |
| `RecoveryAlreadyPending` | 361 | — |
| `RecoveryTimelockNotElapsed` | 362 | — |
| `VerificationMethodNotFound` | 450 | — |
| `CredentialNotFound` | 460 | — |
| `AttestationNotFound` | 461 | — |
| `ServiceNotFound` | 462 | — |
| `DIDNotFound` | 470 | — |
| `DIDAlreadyExists` | 471 | — |
| `DIDDeactivated` | 472 | — |
| `CredentialExpired` | 605 | — |
| `CredentialRevoked` | 606 | — |
| `KeyRotationCooldown` | 603 | — |

<!-- API_END -->

## DID Method

The contract implements a Stellar-native DID method: `did:stellar:<address>`.
