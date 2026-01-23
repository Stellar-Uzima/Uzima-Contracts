# Decentralized Identity (DID) Integration

## Overview

The Uzima Healthcare Platform implements W3C Decentralized Identifiers (DIDs) and Verifiable Credentials to provide secure, interoperable, and user-controlled identity management for healthcare stakeholders.

This document describes the DID integration architecture, compliance with W3C standards, and usage guidelines for the healthcare ecosystem.

## Table of Contents

1. [DID Method Specification](#did-method-specification)
2. [DID Document Structure](#did-document-structure)
3. [Verifiable Credentials](#verifiable-credentials)
4. [Identity Recovery](#identity-recovery)
5. [Key Management](#key-management)
6. [Medical Records Integration](#medical-records-integration)
7. [Compliance Standards](#compliance-standards)
8. [API Reference](#api-reference)
9. [Security Considerations](#security-considerations)
10. [Best Practices](#best-practices)

---

## DID Method Specification

### Method Name

```
did:stellar:uzima
```

### DID Syntax

```
did:stellar:uzima:<network>:<address-hash>
```

**Components:**
- `did:stellar:uzima` - Method prefix
- `<network>` - Network identifier (`testnet`, `mainnet`, `futurenet`)
- `<address-hash>` - SHA-256 hash of the Stellar address (first 16 hex chars)

**Example:**
```
did:stellar:uzima:testnet:a1b2c3d4e5f67890
```

### CRUD Operations

| Operation | Method | Authorization |
|-----------|--------|---------------|
| Create | `create_did` | Subject only |
| Read | `resolve_did` | Public |
| Update | `update_did`, `add_verification_method` | Subject only |
| Deactivate | `deactivate_did` | Subject only |

---

## DID Document Structure

The DID Document follows the [W3C DID Core Specification](https://www.w3.org/TR/did-core/).

### Schema

```rust
pub struct DIDDocument {
    // The DID identifier
    pub id: String,

    // Controller of this DID
    pub controller: Address,

    // Alternative identifiers (for interoperability)
    pub also_known_as: Vec<String>,

    // Verification methods (public keys)
    pub verification_methods: Vec<VerificationMethod>,

    // Methods for authentication
    pub authentication: Vec<String>,

    // Methods for issuing credentials
    pub assertion_method: Vec<String>,

    // Methods for key agreement
    pub key_agreement: Vec<String>,

    // Methods for capability invocation
    pub capability_invocation: Vec<String>,

    // Methods for capability delegation
    pub capability_delegation: Vec<String>,

    // Service endpoints
    pub services: Vec<ServiceEndpoint>,

    // Document status
    pub status: DIDStatus,

    // Timestamps
    pub created: u64,
    pub updated: u64,

    // Version control
    pub version: u32,
    pub previous_hash: BytesN<32>,
}
```

### Verification Methods

Supported verification method types:

| Type | Use Case |
|------|----------|
| `Ed25519VerificationKey2020` | Authentication, assertion |
| `EcdsaSecp256k1VerificationKey2019` | Bitcoin/Ethereum compatibility |
| `X25519KeyAgreementKey2020` | Key agreement, encryption |
| `JsonWebKey2020` | Web standards compatibility |

### Service Endpoints

Service endpoints allow DIDs to advertise services:

```rust
pub struct ServiceEndpoint {
    pub id: String,           // e.g., "#medical-records"
    pub service_type: String, // e.g., "MedicalRecords"
    pub endpoint: String,     // e.g., "ipfs://Qm..."
    pub is_active: bool,
}
```

**Common Service Types:**
- `MedicalRecords` - Link to medical records contract
- `CredentialRegistry` - Link to credential storage
- `LinkedDomains` - Domain verification
- `MessagingService` - Secure messaging endpoint

---

## Verifiable Credentials

### Credential Types

| Type | Description | Typical Issuer |
|------|-------------|----------------|
| `MedicalLicense` | Medical practice license | Medical Board |
| `SpecialistCertification` | Specialty certification | Medical Association |
| `HospitalAffiliation` | Hospital staff affiliation | Healthcare Institution |
| `ResearchAuthorization` | Research access permission | Ethics Board |
| `PatientConsent` | Patient consent for data access | Patient |
| `EmergencyAccess` | Emergency access authorization | Patient/System |
| `DataAccessPermission` | General data access permission | Data Controller |

### Credential Structure

```rust
pub struct VerifiableCredential {
    pub id: BytesN<32>,
    pub credential_type: CredentialType,
    pub issuer: Address,
    pub subject: Address,
    pub issuance_date: u64,
    pub expiration_date: u64,      // 0 = no expiration
    pub credential_hash: BytesN<32>, // Off-chain data hash
    pub credential_uri: String,     // IPFS CID or URL
    pub is_revoked: bool,
    pub revoked_at: u64,
    pub revocation_reason: String,
}
```

### Credential Lifecycle

```
┌─────────┐    ┌─────────┐    ┌─────────┐    ┌─────────┐
│  Issue  │ -> │  Active │ -> │ Verify  │ -> │ Revoke  │
└─────────┘    └─────────┘    └─────────┘    └─────────┘
     │              │              │              │
     │              │              │              │
     v              v              v              v
 Only Verifier  On-chain    Public Query   Only Issuer
 can issue      reference   with status    can revoke
```

### Credential Status

```rust
pub enum CredentialStatus {
    Valid,      // Active and not expired
    Revoked,    // Revoked by issuer
    Expired,    // Past expiration date
    NotFound,   // Credential doesn't exist
}
```

---

## Identity Recovery

### Overview

The identity recovery mechanism allows users to regain control of their DID if they lose access to their private keys. It uses a guardian-based social recovery system with timelock protection.

### Recovery Components

1. **Recovery Guardians**: Trusted entities designated by the DID subject
2. **Recovery Threshold**: Minimum approval weight required
3. **Recovery Timelock**: 24-hour waiting period before execution

### Guardian Structure

```rust
pub struct RecoveryGuardian {
    pub address: Address,
    pub weight: u32,
    pub added_at: u64,
}
```

### Recovery Process

```
1. Subject adds guardians    ->  Guardians stored with weights
2. Subject loses key access  ->  Cannot authenticate
3. Guardian initiates recovery -> Recovery request created
4. Other guardians approve   ->  Approvals accumulated
5. Wait 24 hours (timelock)  ->  Protection against attacks
6. Execute recovery          ->  New controller & key set
```

### Recovery Request

```rust
pub struct RecoveryRequest {
    pub request_id: u64,
    pub subject: Address,
    pub new_controller: Address,
    pub new_primary_key: BytesN<32>,
    pub created_at: u64,
    pub approvals: Vec<Address>,
    pub total_weight: u32,
    pub executed: bool,
}
```

### Security Measures

- **Timelock**: 24-hour delay allows subject to cancel malicious recovery
- **Threshold**: Requires multiple guardian approvals
- **Cancellation**: Subject can cancel if they regain access
- **Status Flag**: DID marked as "RecoveryPending" during process

---

## Key Management

### Key Rotation

Keys can be rotated to enhance security:

```rust
pub fn rotate_key(
    subject: Address,
    method_id: String,
    new_public_key: BytesN<32>,
) -> Result<(), Error>
```

**Cooldown Period**: 1 hour between rotations (configurable)

### Key Revocation

Compromised keys can be revoked:

```rust
pub fn revoke_verification_method(
    subject: Address,
    method_id: String,
) -> Result<(), Error>
```

**Note**: At least one active verification method must remain.

### Verification Relationships

| Relationship | Purpose |
|--------------|---------|
| `Authentication` | Prove DID control |
| `AssertionMethod` | Issue credentials |
| `KeyAgreement` | Establish shared secrets |
| `CapabilityInvocation` | Invoke capabilities |
| `CapabilityDelegation` | Delegate capabilities |

---

## Medical Records Integration

### DID-Linked Records

Medical records can include DID references:

```rust
pub struct MedicalRecord {
    // ... existing fields ...
    pub doctor_did: Option<String>,
    pub authorization_credential: Option<BytesN<32>>,
}
```

### Emergency Access

Patients can grant time-limited emergency access:

```rust
pub struct EmergencyAccess {
    pub grantee: Address,
    pub patient: Address,
    pub expires_at: u64,
    pub record_scope: Vec<u64>,
    pub is_active: bool,
}
```

### Access Logging

All access attempts are logged for audit:

```rust
pub struct AccessRequest {
    pub requester: Address,
    pub patient: Address,
    pub record_id: u64,
    pub purpose: String,
    pub timestamp: u64,
    pub granted: bool,
    pub credential_used: Option<BytesN<32>>,
}
```

### DID Authentication Levels

```rust
pub enum DIDAuthLevel {
    None,               // Legacy mode
    Basic,              // DID must be active
    CredentialRequired, // Must have valid credential
    Full,               // DID + credential + specific permission
}
```

---

## Compliance Standards

### W3C Standards

| Standard | Implementation |
|----------|----------------|
| [DID Core 1.0](https://www.w3.org/TR/did-core/) | ✅ Full compliance |
| [DID Resolution](https://w3c.github.io/did-resolution/) | ✅ Implemented |
| [Verifiable Credentials 2.0](https://www.w3.org/TR/vc-data-model-2.0/) | ✅ Data model compliant |

### Healthcare Compliance Considerations

#### HIPAA (US)

- **Audit Logging**: All access attempts logged
- **Minimum Necessary**: Role-based access control
- **Emergency Access**: Patient-controlled emergency grants
- **Encryption**: Off-chain data referenced by hash

#### GDPR (EU)

- **Data Minimization**: Only references stored on-chain
- **Right to Access**: Patients can view access logs
- **Right to Erasure**: DID deactivation supported
- **Data Portability**: Standard DID format for interoperability

### Security Standards

| Requirement | Implementation |
|-------------|----------------|
| Key Security | Ed25519 signatures |
| Replay Protection | Nonce-based (via Stellar) |
| Timelock Protection | 24-hour recovery delay |
| Multi-signature | Guardian threshold system |

---

## API Reference

### DID Document Management

```rust
// Create a new DID
fn create_did(
    subject: Address,
    primary_public_key: BytesN<32>,
    services: Vec<ServiceEndpoint>,
) -> Result<String, Error>

// Resolve a DID
fn resolve_did(subject: Address) -> Result<DIDDocument, Error>

// Update a DID
fn update_did(
    subject: Address,
    new_services: Vec<ServiceEndpoint>,
    new_also_known_as: Vec<String>,
) -> Result<(), Error>

// Deactivate a DID
fn deactivate_did(subject: Address) -> Result<(), Error>
```

### Verification Methods

```rust
// Add verification method
fn add_verification_method(
    subject: Address,
    method_id: String,
    method_type: VerificationMethodType,
    public_key: BytesN<32>,
    relationships: Vec<VerificationRelationship>,
) -> Result<(), Error>

// Rotate key
fn rotate_key(
    subject: Address,
    method_id: String,
    new_public_key: BytesN<32>,
) -> Result<(), Error>

// Revoke method
fn revoke_verification_method(
    subject: Address,
    method_id: String,
) -> Result<(), Error>
```

### Verifiable Credentials

```rust
// Issue credential
fn issue_credential(
    issuer: Address,
    subject: Address,
    credential_type: CredentialType,
    credential_hash: BytesN<32>,
    credential_uri: String,
    expiration_date: u64,
) -> Result<BytesN<32>, Error>

// Verify credential
fn verify_credential(credential_id: BytesN<32>) -> Result<CredentialStatus, Error>

// Revoke credential
fn revoke_credential(
    issuer: Address,
    credential_id: BytesN<32>,
    reason: String,
) -> Result<(), Error>
```

### Identity Recovery

```rust
// Add guardian
fn add_recovery_guardian(
    subject: Address,
    guardian: Address,
    weight: u32,
) -> Result<(), Error>

// Initiate recovery
fn initiate_recovery(
    guardian: Address,
    subject: Address,
    new_controller: Address,
    new_primary_key: BytesN<32>,
) -> Result<u64, Error>

// Execute recovery
fn execute_recovery(request_id: u64) -> Result<(), Error>
```

---

## Security Considerations

### Attack Vectors & Mitigations

| Attack | Mitigation |
|--------|------------|
| Key Compromise | Key rotation, recovery guardians |
| Guardian Collusion | Threshold requirements, timelock |
| Replay Attacks | Stellar nonce mechanism |
| Credential Forgery | Cryptographic signatures |
| Unauthorized Access | Role-based access control |

### Best Practices

1. **Key Storage**: Use hardware security modules for key storage
2. **Guardian Selection**: Choose guardians from different trust domains
3. **Credential Expiration**: Set appropriate expiration for credentials
4. **Audit Review**: Regularly review access logs
5. **DID Backup**: Maintain secure backup of DID-related keys

---

## Best Practices

### For Healthcare Providers

1. Create a DID before providing services
2. Obtain `MedicalLicense` credential from appropriate authority
3. Link DID to user profile in medical records system
4. Use credential-based authorization for sensitive operations

### For Patients

1. Create a DID to control your health data
2. Set up recovery guardians (family members, trusted parties)
3. Grant emergency access to trusted healthcare providers
4. Review access logs periodically

### For Healthcare Institutions

1. Register as a verifier to issue credentials
2. Issue `HospitalAffiliation` credentials to staff
3. Integrate DID verification into access control
4. Maintain audit logs for compliance

---

## References

- [W3C DID Core 1.0](https://www.w3.org/TR/did-core/)
- [W3C Verifiable Credentials 2.0](https://www.w3.org/TR/vc-data-model-2.0/)
- [DID Resolution Specification](https://w3c.github.io/did-resolution/)
- [Decentralized Identity Foundation](https://identity.foundation/)
- [Stellar Soroban Documentation](https://soroban.stellar.org/docs)

---

## Changelog

### v1.0.0 (2025-01-21)

- Initial DID integration
- W3C DID Core compliance
- Verifiable credentials for healthcare
- Guardian-based identity recovery
- Key rotation mechanisms
- Medical records DID integration
- Emergency access management
- Access audit logging
