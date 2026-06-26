# Cross-Chain Medical Records Interoperability Architecture

## Overview

The Uzima Cross-Chain Medical Records Interoperability system enables medical records stored on the Stellar blockchain to be securely accessed and managed across multiple blockchain networks while maintaining privacy, security, and regulatory compliance.

## Architecture Components

### 1. Cross-Chain Bridge Contract (`cross_chain_bridge`)

The bridge contract serves as the main orchestrator for cross-chain communication.

**Key Features:**
- Multi-validator message verification
- Atomic cross-chain transactions (2-Phase Commit)
- Nonce-based replay attack protection (via shared `replay_protection` library)
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

## Replay Protection Library

All cross-chain contracts use a shared `replay_protection` library (`libs/replay_protection/`) that
provides three layers of defense in a single call:

### Triple-Check Pattern

```rust
pub fn verify_replay_protection(
    env: &Env,
    message_hash: &BytesN<32>,
    sender_key: &BytesN<32>,
    nonce: u64,
    timestamp: u64,
    ttl_secs: u64,
    source_chain: &ChainId,
    expected_source_chain: &ChainId,
) -> Result<(), ReplayError>
```

| Check | Purpose | Rejects When |
|-------|---------|-------------|
| **Nonce uniqueness** (nonce ≤ last nonce for sender) | Prevents resubmission | `NonceReused` |
| **Expiration** (now > timestamp + ttl) | Prevents stale execution | `MessageExpired` |
| **Chain binding** (source ≠ expected) | Prevents cross-chain replay | `ChainMismatch` |

### Helpers

- `is_message_seen(env, hash)` — query whether a message hash was already processed
- `check_message_expired(env, timestamp, ttl)` — re-check expiration at confirm/execute time

### Contract-Specific Usage

| Contract | Replay Protection |
|----------|-------------------|
| `cross_chain_bridge` | Full `verify_replay_protection` on `submit_message`; `check_message_expired` on `confirm_message` / `execute_message` |
| `cross_chain_access` | `check_message_expired` in `process_request` |
| `cross_chain_identity` | `check_message_expired` in `attest_verification` |
| `cross_chain_enhancements` | `check_replay_protection` uses shared expiration + chain binding, stores idempotency marker |

Each contract defines a `to_replay_chain_id()` conversion that maps its local `ChainId` enum to the
library's `replay_protection::ChainId`.

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

---

## Re-org Protection and Finality Assumptions

### Overview

Blockchain re-organizations (re-orgs) occur when a competing chain fork overtakes the canonical chain, causing previously-confirmed transactions to be reversed. The `cross_chain_bridge` contract protects against re-org-induced fund loss by requiring a minimum number of validator confirmations before a message is considered final.

### Minimum Confirmation Depths

Each supported chain has a defined minimum confirmation depth based on its consensus mechanism and historical re-org data:

| Chain | Minimum Confirmations | Rationale |
|---|---|---|
| **Stellar** | 1 | Federated Byzantine Agreement (FBA) provides instant finality |
| **Ethereum** | 6 | Probabilistic PoS finality; 6 blocks ≈ ~72s safety margin |
| **Polygon** | 3 | PoS with Heimdall checkpoints; 3 blocks provides adequate safety |
| **Avalanche** | 2 | Avalanche consensus achieves finality in ~2 seconds / ~2 rounds |
| **BNB Chain** | 3 | PoSA consensus; 3 confirmations mitigates short re-orgs |
| **Arbitrum** | 3 | Optimistic rollup with fraud proof window; 3 L2 confirmations |
| **Optimism** | 3 | Optimistic rollup; 3 L2 confirmations before Ethereum anchoring |

These values are configurable via `set_min_confirmations()` by the contract admin and can be updated if chain security parameters change.

### Re-org Protection Mechanism

1. **Message Submission**: A validator submits a cross-chain message. The message status is set to `Pending`.
2. **Confirmation Collection**: Additional validators confirm the message. Each confirmation represents one "block depth" of finality evidence.
3. **Finality Gate**: Only after `confirmations.len() >= min_confirmations` does the message transition to `Verified`.
4. **Execution**: Only `Verified` messages can be executed. `Pending` messages cannot trigger any state changes.

### Re-org Scenarios Handled

| Scenario | Re-org Depth | Protection |
|---|---|---|
| Shallow re-org (1 block) | 1 | Message stays `Pending` until 2nd confirmation received |
| Medium re-org (3 blocks) | 3 | Ethereum messages require 6 confirmations; 3-block re-org leaves message `Pending` |
| Deep re-org (6 blocks) | 6 | Message only reaches `Verified` after full finality threshold is met |

### Double-Spend Prevention

- **Per-message confirmation tracking**: Confirmations are stored under `DataKey::Confirmations(message_id)` — not a shared key. Each message has its own confirmation set.
- **Duplicate confirmation rejection**: `Error::DuplicateConfirmation` is returned if a validator attempts to confirm the same message twice.
- **Already-processed rejection**: `Error::MessageAlreadyProcessed` is returned if a message is already `Verified` or `Executed`.
- **Nonce-based replay protection**: Each validator confirmation carries a monotonically-increasing nonce. Re-using a nonce returns `Error::NonceAlreadyUsed`.
- **Message expiry**: Messages not confirmed within `MESSAGE_EXPIRY_SECS` (86,400 seconds / 24 hours) are rejected.

### Test Coverage

Re-org protection is tested in `contracts/cross_chain_bridge/src/reorg_protection_tests.rs`:

| Test | Re-org Depth |
|---|---|
| `reorg_depth_1_message_stays_pending_until_confirmed` | 1 |
| `reorg_depth_3_ethereum_not_verified_before_finality` | 3 |
| `reorg_depth_6_ethereum_verified_after_full_finality` | 6 |
| `double_spend_prevented_same_validator_cannot_confirm_twice` | N/A |
| `double_spend_verified_message_cannot_be_reconfirmed` | N/A |
| `reorg_polygon_depth_3_requires_all_confirmations` | 3 |
| `reorg_message_can_be_retried_after_reorg` | 1 |

