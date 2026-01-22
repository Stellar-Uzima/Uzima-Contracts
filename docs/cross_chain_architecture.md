# Cross-Chain Medical Records Interoperability Architecture

## Overview

The Uzima Cross-Chain Medical Records Interoperability system enables medical records stored on the Stellar blockchain to be securely accessed and managed across multiple blockchain networks while maintaining privacy, security, and regulatory compliance.

## Architecture Components

### 1. Cross-Chain Bridge Contract (`cross_chain_bridge`)

The bridge contract serves as the main orchestrator for cross-chain communication.

**Key Features:**
- Multi-validator message verification
- Atomic cross-chain transactions (2-Phase Commit)
- Nonce-based replay attack protection
- Message expiration handling
- Support for multiple blockchain networks

**Supported Chains:**
- Stellar (native)
- Ethereum
- Polygon
- Avalanche
- Binance Smart Chain
- Arbitrum
- Optimism
- Custom chains (via ChainId::Custom(u32))

**Message Types:**
- `RecordRequest` - Request to access a medical record
- `RecordResponse` - Response with record data
- `IdentityVerify` - Identity verification request
- `IdentityConfirm` - Identity confirmation
- `AccessGrant` - Grant access to records
- `AccessRevoke` - Revoke access to records
- `RecordSync` - Synchronize record state
- `EmergencyAccess` - Emergency access request

### 2. Cross-Chain Identity Contract (`cross_chain_identity`)

Manages identity verification and synchronization across chains.

**Key Features:**
- Cross-chain identity mapping (Stellar address ↔ External chain address)
- Multi-validator attestation for identity verification
- Identity expiration and renewal
- Trust score management for validators
- Identity synchronization across chains

**Verification Flow:**
1. User requests verification linking their Stellar address to an external chain address
2. Validators attest to the identity proof
3. Once minimum attestations are met, identity is verified
4. Verified identity can be synced to other chains

### 3. Cross-Chain Access Contract (`cross_chain_access`)

Manages access permissions for medical records across chains.

**Key Features:**
- Granular permission levels (None, Read, ReadConfidential, Write, Admin)
- Flexible access scopes (AllRecords, SpecificRecords, CategoryBased, TimeRanged)
- Access conditions (EmergencyOnly, RequireConsent, AuditRequired, SingleUse, TimeRestricted)
- Delegation of access management
- Emergency access configuration
- Complete audit trail

**Permission Levels:**
| Level | Description |
|-------|-------------|
| None | No access |
| Read | Can view non-confidential records |
| ReadConfidential | Can view all records including confidential |
| Write | Can create new records |
| Admin | Full access including management functions |

### 4. Medical Records Contract (Enhanced)

The existing medical records contract has been enhanced with cross-chain capabilities.

**New Cross-Chain Features:**
- Cross-chain contract references
- Record metadata for cross-chain queries
- Cross-chain record reference registration
- Cross-chain record retrieval (via bridge)
- Record hash computation for integrity verification

## Security Model

### Validator-Based Security

The system uses a multi-validator approach for security:

1. **Minimum Confirmations**: Messages require confirmation from multiple validators (default: 2)
2. **Validator Staking**: Validators stake tokens as collateral
3. **Trust Scores**: Validators have trust scores that can be adjusted
4. **Validator Deactivation**: Malicious validators can be deactivated

### Access Control

Multiple layers of access control protect medical records:

1. **Role-Based Access**: Admin, Doctor, Patient roles
2. **Cross-Chain Access Grants**: Time-limited, condition-based access
3. **Delegation**: Patients can delegate access management
4. **Emergency Access**: Pre-configured emergency access with trusted providers

### Audit Trail

All cross-chain access is logged:

- Accessor chain and address
- Patient address
- Record ID
- Action type (View, Download, Share, Export, EmergencyAccess)
- Timestamp
- IP hash (privacy-preserving)
- Success/failure status

## Data Flow

### Cross-Chain Record Access Flow

```
External Chain                    Bridge                    Stellar
      │                             │                          │
      │ 1. Request Access           │                          │
      ├────────────────────────────>│                          │
      │                             │ 2. Verify Identity       │
      │                             ├─────────────────────────>│
      │                             │<─────────────────────────┤
      │                             │ 3. Check Access Rights   │
      │                             ├─────────────────────────>│
      │                             │<─────────────────────────┤
      │                             │ 4. Retrieve Record       │
      │                             ├─────────────────────────>│
      │                             │<─────────────────────────┤
      │ 5. Return Record (encrypted)│                          │
      │<────────────────────────────┤                          │
      │                             │ 6. Log Access            │
      │                             ├─────────────────────────>│
```

### Atomic Transaction Flow (2-Phase Commit)

```
Phase 1: Prepare
┌─────────────────────────────────────────────────────────┐
│ 1. Initiator creates atomic transaction                 │
│ 2. Validators prepare and confirm                       │
│ 3. Transaction moves to "Prepared" state                │
└─────────────────────────────────────────────────────────┘
                           │
                           ▼
Phase 2: Commit/Abort
┌─────────────────────────────────────────────────────────┐
│ If all parties ready: Commit transaction                │
│ If any party fails: Abort transaction                   │
│ If timeout expires: Transaction expires                 │
└─────────────────────────────────────────────────────────┘
```

## Deployment

### Prerequisites

- Stellar CLI (soroban)
- Rust toolchain with wasm32 target
- Network access to target chain

### Deployment Order

1. Deploy `cross_chain_bridge`
2. Deploy `cross_chain_identity`
3. Deploy `cross_chain_access`
4. Initialize all contracts with cross-references
5. Configure `medical_records` with cross-chain contracts

### Deployment Script

```bash
./scripts/deploy_cross_chain.sh <network> [identity]

# Example:
./scripts/deploy_cross_chain.sh testnet admin
```

## Configuration

### Adding Validators

```bash
soroban contract invoke \
    --id <BRIDGE_CONTRACT_ID> \
    --source <ADMIN> \
    --network testnet \
    -- add_validator \
    --caller <ADMIN_ADDRESS> \
    --validator_address <VALIDATOR_ADDRESS> \
    --public_key <PUBLIC_KEY_32_BYTES> \
    --initial_stake 1000
```

### Adding Supported Chains

```bash
soroban contract invoke \
    --id <BRIDGE_CONTRACT_ID> \
    --source <ADMIN> \
    --network testnet \
    -- add_supported_chain \
    --caller <ADMIN_ADDRESS> \
    --chain Avalanche
```

### Granting Cross-Chain Access

```bash
soroban contract invoke \
    --id <ACCESS_CONTRACT_ID> \
    --source <PATIENT> \
    --network testnet \
    -- grant_access \
    --grantor <PATIENT_ADDRESS> \
    --grantee_chain Ethereum \
    --grantee_address "0x1234..." \
    --permission_level Read \
    --record_scope AllRecords \
    --duration 2592000 \
    --conditions "[]"
```

## Emergency Access

### Configuration

Patients can configure emergency access settings:

```bash
soroban contract invoke \
    --id <ACCESS_CONTRACT_ID> \
    --source <PATIENT> \
    --network testnet \
    -- configure_emergency \
    --patient <PATIENT_ADDRESS> \
    --is_enabled true \
    --auto_approve_duration 3600 \
    --required_attestations 2 \
    --trusted_providers '["0xhospital1...", "0xhospital2..."]'
```

### Emergency Request Flow

1. Emergency responder requests access with `is_emergency: true`
2. If responder is in trusted providers list, access is auto-approved
3. Otherwise, request requires validator attestations
4. Access is time-limited based on `auto_approve_duration`

## Error Handling

### Bridge Errors
- `NotAuthorized` - Caller lacks required permissions
- `ContractPaused` - Contract operations are paused
- `InvalidChain` - Unsupported blockchain
- `MessageExpired` - Message exceeded expiry time
- `InsufficientConfirmations` - Not enough validator confirmations

### Identity Errors
- `IdentityNotFound` - No verified identity exists
- `IdentityExpired` - Identity verification has expired
- `DuplicateAttestation` - Validator already attested

### Access Errors
- `GrantNotFound` - Access grant doesn't exist
- `GrantExpired` - Access grant has expired
- `InsufficientPermissions` - Permission level too low

## Testing

Each contract includes comprehensive tests:

```bash
# Run all tests
cargo test --workspace

# Run specific contract tests
cd contracts/cross_chain_bridge && cargo test
cd contracts/cross_chain_identity && cargo test
cd contracts/cross_chain_access && cargo test
```

## Security Considerations

1. **Key Management**: Validators must securely manage their keys
2. **Message Validation**: All cross-chain messages are validated before processing
3. **Rate Limiting**: Consider implementing rate limits for cross-chain requests
4. **Privacy**: Only record metadata is exposed for cross-chain queries; full records require access verification
5. **Regulatory Compliance**: The system supports HIPAA-compliant access controls and audit trails

## Future Enhancements

1. **Zero-Knowledge Proofs**: Implement ZK proofs for privacy-preserving verification
2. **Multi-Party Computation**: Enable secure computation on encrypted records
3. **Cross-Chain Record Updates**: Support updating records from external chains
4. **Interoperability Standards**: Implement HL7 FHIR standards for healthcare data
5. **Decentralized Identifiers (DIDs)**: Integrate with W3C DID standards
