# Error Codes Reference

> Comprehensive reference of all contract error codes across the Uzima Contracts ecosystem.
> Auto-generated from contract source. Keep this file in sync with contract changes.

## Overview

All Uzima contracts use numeric error codes organized by category:

| Range | Category | Description |
|-------|----------|-------------|
| 1–99 | Contract-specific | Per-contract errors (unique to each contract) |
| 100–199 | Access Control | Authorization, authentication, permissions |
| 200–299 | Input Validation | Invalid arguments, format errors |
| 300–399 | Lifecycle & State | Initialization, pause, status transitions |
| 400–499 | Entity Existence | Not found, already exists |
| 500–599 | Financial & Resource | Funds, storage, limits |
| 600–699 | Cryptography | Key management, proofs |
| 700–799 | Cross-Chain | Bridge, oracle, chain operations |
| 800–899 | Reentrancy & Safety | Locking, circuit breaker |

---

## Access Control (100–199)

| Code | Symbol | Contract(s) | Description | Common Causes | Remediation |
|------|--------|-------------|-------------|---------------|-------------|
| 100 | `Unauthorized` | All | Caller lacks permission for this action | Invalid role, expired authorization, not authenticated | Verify caller identity, check role assignments, ensure authentication |
| 101 | `UnauthorizedCaller` | healthcare_payment | Specific caller not authorized | Wrong address calling restricted function | Check caller address matches expected admin/operator |
| 102 | `NotAuthorizedPauser` | healthcare_payment | Not authorized to pause | Caller is not admin or authorized pauser | Add address to authorized pausers list |
| 110 | `NotVerifier` | identity_registry | Caller is not a registered verifier | Address not in verifier registry | Register address as verifier first |
| 111 | `CannotRemoveOwner` | identity_registry | Cannot remove the owner as verifier | Attempt to remove the contract owner | Owner must remain a verifier |
| 120 | `InsufficientConfirmations` | cross_chain_bridge | Not enough validator confirmations | Message/operation has fewer confirmations than required threshold | Wait for more validators to confirm |
| 121 | `InsufficientOracleReports` | cross_chain_bridge | Insufficient oracle reports for consensus | Fewer reports received than MIN_ORACLE_REPORTS | Submit more oracle reports |
| 122 | `DuplicateOracleReport` | cross_chain_bridge | Oracle has already submitted a report | Duplicate submission from same oracle | Wait for other oracles to submit |

## Input Validation (200–299)

| Code | Symbol | Contract(s) | Description | Common Causes | Remediation |
|------|--------|-------------|-------------|---------------|-------------|
| 205 | `InvalidAmount` | escrow, healthcare_payment | Invalid amount (zero or negative) | Amount <= 0 | Provide a positive amount |
| 207 | `InvalidSignature` | cross_chain_bridge, timelock | Cryptographic signature validation failed | Corrupted signature, wrong key | Verify signing key and regenerate signature |
| 280 | `InvalidCoverage` | healthcare_payment | Coverage policy validation failed | Missing policy data, invalid coverage BPS | Check policy parameters |
| 281 | `InvalidNonce` | cross_chain_bridge | Nonce replay protection triggered | Duplicate nonce or nonce too low | Use a higher nonce value |
| 282 | `InvalidPayload` | cross_chain_bridge | Message payload is invalid | Malformed data | Check payload format |
| 290 | `InvalidAddress` | cross_chain_bridge | Chain address format is invalid | Wrong length or prefix | Use correct chain-specific address format |
| 281 | `PolicyMismatch` | healthcare_payment | Policy ID doesn't match claim | Claim policy mismatch | Verify policy ID |

## Lifecycle & State (300–399)

| Code | Symbol | Contract(s) | Description | Common Causes | Remediation |
|------|--------|-------------|-------------|---------------|-------------|
| 300 | `NotInitialized` | All | Contract has not been initialized | Missing `initialize()` call | Call `initialize()` first |
| 301 | `AlreadyInitialized` | All | Contract already initialized | Duplicate initialization attempt | Contract can only be initialized once |
| 302 | `ContractPaused` | All | Contract is paused; no state-changing calls allowed | Emergency pause or maintenance | Wait for unpause or contact admin |
| 303 | `CircuitOpen` | healthcare_payment | Circuit breaker is open | Too many recent failures | Wait for recovery or admin intervention |
| 304 | `InvalidStatus` | healthcare_payment | Claim/entity in wrong status for this operation | Wrong workflow step | Check current status and required transition |
| 305 | `AlreadyInState` | healthcare_payment | Already in the requested state | Redundant state transition | No action needed |
| 306 | `DeadlineExceeded` | healthcare_payment, timelock | Operation deadline/timelock has passed | Operation timed out | Retry with fresh operation |
| 372 | `NotQueued` | timelock | Transaction not in timelock queue | Wrong ID or already executed | Check queue ID |
| 375 | `AlreadyQueued` | timelock | Transaction already queued | Duplicate queue attempt | Use a different ID |
| 376 | `NotReady` | timelock | Timelock delay not yet elapsed | Too early to execute | Wait until ETA has passed |

## Entity Existence (400–499)

| Code | Symbol | Contract(s) | Description | Common Causes | Remediation |
|------|--------|-------------|-------------|---------------|-------------|
| 470 | `DIDNotFound` | identity_registry | DID document not found | Address has no DID | Create DID first |
| 471 | `DIDAlreadyExists` | identity_registry | DID already registered | Duplicate DID creation | Use existing DID |
| 472 | `DIDDeactivated` | identity_registry | DID has been deactivated | DID was intentionally deactivated | Cannot use deactivated DID |
| 480 | `ClaimNotFound` | healthcare_payment | Claim not found | Wrong claim ID | Verify claim ID |
| 481 | `MessageNotFound` | cross_chain_bridge | Cross-chain message not found | Wrong message ID | Verify message ID |
| 482 | `PreAuthNotFound` | healthcare_payment | Pre-authorization not found | Wrong pre-auth ID | Verify pre-auth ID |
| 483 | `PaymentPlanNotFound` | healthcare_payment | Payment plan not found | Wrong plan ID | Verify plan ID |
| 484 | `InsuranceProviderNotFound` | healthcare_payment | Insurance provider not found | Wrong provider ID | Verify provider ID |
| 485 | `CoveragePolicyNotFound` | healthcare_payment | Coverage policy not found | Wrong policy ID | Verify policy ID |
| 486 | `EligibilityCheckNotFound` | healthcare_payment | Eligibility check not found | No eligibility check performed | Run eligibility check first |
| 487 | `EobNotFound` | healthcare_payment | Explanation of benefits not found | EOB not processed yet | Process claim to generate EOB |
| 488 | `AtomicTxNotFound` | cross_chain_bridge | Atomic transaction not found | Wrong transaction ID | Verify tx ID |
| 489 | `RecordRefNotFound` | cross_chain_bridge | Cross-chain record reference not found | Wrong record/chain pair | Register record reference first |
| 490 | `RollbackNotFound` | cross_chain_bridge | Rollback record not found | Wrong operation ID | Verify operation ID |
| 491 | `RollbackAlreadyProcessed` | cross_chain_bridge | Rollback already completed | Duplicate rollback | No action needed |
| 492 | `EventNotFound` | cross_chain_bridge | Cross-chain event not found | Wrong event ID | Verify event ID |
| 481 | `ClaimSubmissionNotFound` | healthcare_payment | Claim submission not found | No EDI submission for this claim | Submit insurance claim first |
| 481 | `EscrowNotFound` | escrow | Escrow not found | Wrong order ID | Verify escrow order ID |
| 482 | `AlreadySettled` | escrow | Escrow already settled | Duplicate settlement attempt | No action needed |
| 471 | `EscrowExists` | escrow | Escrow order already exists | Duplicate order ID | Use unique order ID |
| 404 | `DIDNotFound` | identity_registry | DID not found | Address has no DID | Create DID first |

## Financial & Resource (500–599)

| Code | Symbol | Contract(s) | Description | Common Causes | Remediation |
|------|--------|-------------|-------------|---------------|-------------|
| 500 | `InsufficientFunds` | healthcare_payment, timelock | Insufficient funds for operation | Not enough tokens | Add funds |
| 502 | `StorageFull` | healthcare_payment, timelock | Storage capacity limit reached | Too much data stored | Clean up old data |
| 580 | `FraudDetected` | healthcare_payment | Fraud report exists for this claim | Claim flagged as fraudulent | Resolve fraud report |
| 581 | `EscrowFailed` | healthcare_payment | Escrow creation failed | Escrow contract rejected | Check escrow parameters |
| 582 | `UnsupportedTransaction` | healthcare_payment | Unsupported transaction code | Wrong EDI format | Use supported transaction code |

## Cryptography (600–699)

| Code | Symbol | Contract(s) | Description | Common Causes | Remediation |
|------|--------|-------------|-------------|---------------|-------------|
| 600 | `InvalidKey` | crypto_registry | Invalid key format | Key is empty or malformed | Provide valid key data |
| 601 | `KeyNotFound` | crypto_registry | Key not found in registry | Wrong owner/version | Verify key owner and version |
| 602 | `KeyAlreadyRevoked` | crypto_registry | Key already revoked | Duplicate revocation | No action needed |
| 603 | `InvalidKeyLength` | crypto_registry | Key length doesn't match algorithm | Wrong key size for algorithm | Use correct key length |
| 604 | `CredentialNotFound` | identity_registry | Credential not found | Wrong credential ID | Verify credential ID |
| 605 | `CredentialExpired` | identity_registry | Credential has expired | Past expiration date | Renew credential |
| 610 | `ProofNotFound` | cross_chain_bridge | Cryptographic proof not found | Wrong proof ID | Verify proof ID |
| 611 | `ProofAlreadyVerified` | cross_chain_bridge | Proof already verified | Duplicate verification | No action needed |

## Cross-Chain (700–799)

| Code | Symbol | Contract(s) | Description | Common Causes | Remediation |
|------|--------|-------------|-------------|---------------|-------------|
| 702 | `CrossChainTimeout` | healthcare_payment, timelock | Cross-chain operation timed out | Message not delivered in time | Retry or escalate |
| 703 | `InvalidChain` | cross_chain_bridge | Invalid chain identifier | Chain not recognized | Check chain ID |
| 720 | `ChainNotSupported` | cross_chain_bridge | Chain not in supported list | Chain not configured | Add chain to supported list |
| 721 | `OracleNotFound` | cross_chain_bridge | Oracle node not found | Wrong oracle address | Verify oracle address |
| 722 | `OracleNotActive` | cross_chain_bridge | Oracle not active | Oracle deactivated | Contact admin to reactivate |

## Reentrancy & Safety (800–899)

| Code | Symbol | Contract(s) | Description | Common Causes | Remediation |
|------|--------|-------------|-------------|---------------|-------------|
| 800 | `Reentrancy` | healthcare_payment | Reentrancy guard triggered | Concurrent call detected | Retry after current operation completes |
| 801 | `OperationNotFound` | cross_chain_bridge | Cross-chain operation not found | Wrong operation ID | Verify operation ID |
| 802 | `OperationExpired` | cross_chain_bridge | Operation has expired | Deadline passed | Create new operation |
| 803 | `OperationAlreadyCompleted` | cross_chain_bridge | Operation already completed | Duplicate completion | No action needed |
| 804 | `MaxExtensionsReached` | cross_chain_bridge | Max timeout extensions reached | Too many extensions | Create new operation |
| 804 | `RefundFailed` | cross_chain_bridge | Refund processing failed | Cannot transfer refund | Check balance and retry |

## Per-Contract Error Codes (1–99)

### clinical_trial

| Code | Symbol | Description | Remediation |
|------|--------|-------------|-------------|
| 1 | `ProtocolNotFound` | Protocol ID not found | Check protocol ID |
| 2 | `TrialFull` | Trial enrollment cap reached | Cannot exceed max_participants |

### pharma_supply_chain

| Code | Symbol | Description | Remediation |
|------|--------|-------------|-------------|
| 1 | `AlreadyInitialized` | Already initialized | Cannot reinitialize |
| 2 | `NotInitialized` | Not initialized | Call initialize first |
| 3 | `Unauthorized` | Unauthorized caller | Check permissions |
| 4 | `ManufacturerNotFound` | Manufacturer not found | Verify manufacturer ID |
| 5 | `MedicationNotFound` | Medication not found | Verify medication ID |
| 6 | `BatchNotFound` | Batch not found | Verify batch ID |
| 7 | `ShipmentNotFound` | Shipment not found | Verify shipment ID |
| 8 | `InvalidInput` | Invalid input parameters | Check parameter values |
| 9 | `BatchAlreadyExists` | Batch ID already exists | Use unique batch ID |

### ai_analytics

| Code | Symbol | Description | Remediation |
|------|--------|-------------|-------------|
| 1 | `NotAuthorized` | Caller not authorized | Check admin/permissions |
| 2 | `RoundNotFound` | Round ID not found | Verify round ID |
| 3 | `RoundFinalized` | Round already finalized | Cannot modify finalized round |
| 4 | `NotEnoughParticipants` | Insufficient participants to finalize | Wait for more participants |
| 5 | `DuplicateUpdate` | Duplicate participant update | Each participant submits once per round |
| 6 | `AlreadyInitialized` | Already initialized | Cannot reinitialize |
| 7 | `AdminNotSet` | Admin not yet configured | Call initialize first |
| 8 | `SerializationError` | General serialization error | Check data format |
| 9 | `CollectionTooLarge` | Collection exceeds max size | Reduce collection size |
| 10 | `StringTooLong` | String exceeds max length | Shorten string |
| 11 | `NestingTooDeep` | Structure nesting too deep | Flatten data structure |

### identity_registry

| Code | Symbol | Description | Remediation |
|------|--------|-------------|-------------|
| 100 | `Unauthorized` | Caller not authorized | Check roles |
| 101 | `InputTooLong` | Input exceeds max length | Shorten input |
| 102 | `InvalidInput` | Invalid input parameters | Check parameter values |
| 103 | `InvalidServiceEndpoint` | Invalid service endpoint URL | Use valid URL format |
| 104 | `VerificationMethodNotFound` | Verification method not found | Check method ID |
| 105 | `InvalidVerificationMethod` | Invalid or last active method | Add more methods first |
| 106 | `GuardianNotFound` | Guardian not found | Check guardian address |
| 107 | `ServiceNotFound` | Service endpoint not found | Check service ID |
| 108 | `KeyRotationCooldown` | Key rotation too soon | Wait for cooldown period |
| 110 | `NotVerifier` | Not a verifier | Must be registered as verifier |
| 111 | `CannotRemoveOwner` | Cannot remove owner as verifier | Owner retains verifier status |
| 300 | `NotInitialized` | Not initialized | Call initialize |
| 301 | `AlreadyInitialized` | Already initialized | Cannot reinitialize |
| 470 | `DIDNotFound` | DID not found | Create DID first |
| 471 | `DIDAlreadyExists` | DID already exists | Use existing DID |
| 472 | `DIDDeactivated` | DID deactivated | Reactivate or use different |
| 473 | `CredentialNotFound` | Credential not found | Verify credential ID |
| 474 | `CredentialRevoked` | Credential has been revoked | Cannot use revoked credential |
| 475 | `CredentialExpired` | Credential expired | Renew credential |
| 476 | `RecoveryAlreadyPending` | Recovery already in progress | Wait for completion |
| 477 | `RecoveryNotInitiated` | No recovery initiated | Start recovery first |
| 478 | `InvalidRecoveryGuardian` | Not a guardian for this DID | Only guardians can initiate recovery |
| 479 | `RecoveryTimelockNotElapsed` | Recovery timelock not elapsed | Wait for timelock period |
| 480 | `InsufficientGuardianApprovals` | Insufficient guardian approvals | Get more guardian approvals |
| 481 | `ServiceNotFound` | Service endpoint not found | Check service ID |

### escrow

| Code | Symbol | Description | Remediation |
|------|--------|-------------|-------------|
| 100 | `Unauthorized` | Caller not authorized | Check caller permissions |
| 102 | `NotAdmin` | Only admin can perform this action | Use admin account |
| 205 | `InvalidAmount` | Amount must be positive | Provide positive amount |
| 481 | `EscrowNotFound` | Escrow not found | Verify escrow ID |
| 482 | `AlreadySettled` | Already settled/refunded | Check escrow status |
| 471 | `EscrowExists` | Escrow already exists | Use unique ID |
| 483 | `FeeNotSet` | Fee config not set | Configure fee first |
| 484 | `InvalidFeeBps` | Invalid fee (max 10000 bps) | Use fee <= 10000 |
| 485 | `InsufficientApprovals` | Not enough approvals | Get more approvals |
| 486 | `InvalidStateTransition` | Invalid status transition | Follow correct workflow |
| 487 | `NoBasisToRefund` | No basis to refund | Escrow must have approvals |
| 490 | `NoCredit` | No credit balance | No funds to withdraw |
| 800 | `ReentrancyGuard` | Reentrancy guard triggered | Wait for completion |
| 580 | `Overflow` | Arithmetic overflow | Use smaller values |

### cross_chain_bridge

| Code | Symbol | Description | Remediation |
|------|--------|-------------|-------------|
| 100 | `Unauthorized` | Caller not authorized | Check permissions |
| 101 | `UnauthorizedRelayer` | Not an authorized relayer | Register as relayer first |
| 120 | `InsufficientConfirmations` | Not enough validator confirmations | Wait for more validators |
| 121 | `InsufficientOracleReports` | Insufficient oracle reports | Submit more reports |
| 122 | `DuplicateOracleReport` | Oracle already reported | Unique report per oracle |
| 207 | `InvalidSignature` | Invalid signature | Check signing key |
| 280 | `InvalidMessage` | Invalid message format | Check message data |
| 281 | `InvalidNonce` | Invalid nonce (replay protection) | Use higher nonce |
| 282 | `InvalidPayload` | Invalid payload | Check payload format |
| 290 | `InvalidAddress` | Invalid chain address | Use correct address format |
| 301 | `AlreadyInitialized` | Already initialized | Cannot reinitialize |
| 302 | `ContractPaused` | Contract is paused | Wait for unpause |
| 580 | `Overflow` | Arithmetic overflow | Use reasonable values |
| 480 | `MessageNotFound` | Message not found | Verify message ID |
| 481 | `MessageExpired` | Message expired | Submit new message |
| 482 | `MessageAlreadyProcessed` | Message already processed | Check message status |
| 483 | `ValidatorNotFound` | Validator not found | Verify validator address |
| 484 | `ValidatorNotActive` | Validator not active | Contact admin |
| 485 | `DuplicateConfirmation` | Already confirmed | No action needed |
| 486 | `AtomicTxNotFound` | Atomic transaction not found | Verify tx ID |
| 487 | `AtomicTxExpired` | Atomic transaction expired | Create new tx |
| 488 | `AtomicTxAlreadyProcessed` | Atomic tx already processed | Check status |
| 489 | `RecordRefNotFound` | Record reference not found | Verify record/chain |
| 490 | `RollbackNotFound` | Rollback not found | Verify operation ID |
| 491 | `RollbackAlreadyProcessed` | Rollback already processed | Check status |
| 492 | `EventNotFound` | Sync event not found | Verify event ID |
| 610 | `ProofNotFound` | Proof not found | Verify proof ID |
| 611 | `ProofAlreadyVerified` | Proof already verified | No action needed |
| 703 | `InvalidChain` | Invalid chain | Check chain ID |
| 720 | `ChainNotSupported` | Chain not supported | Add chain first |
| 721 | `OracleNotFound` | Oracle not found | Verify oracle address |
| 722 | `OracleNotActive` | Oracle not active | Contact admin |
| 800 | `OperationNotFound` | Operation not found | Verify operation ID |
| 801 | `OperationExpired` | Operation expired | Create new operation |
| 802 | `OperationAlreadyCompleted` | Already completed | No action needed |
| 803 | `MaxExtensionsReached` | Max extensions reached | Create new operation |
| 804 | `RefundFailed` | Refund failed | Check balance |

### medical_records

| Code | Symbol | Description | Remediation |
|------|--------|-------------|-------------|
| 100 | `Unauthorized` | Not authorized | Check RBAC role |
| 300 | `NotInitialized` | Not initialized | Call initialize |
| 301 | `AlreadyInitialized` | Already initialized | Cannot reinitialize |
| 302 | `ContractPaused` | Contract paused | Wait for unpause |
| 400 | `RecordNotFound` | Record not found | Verify record ID |
| 401 | `RecordAlreadyExists` | Record already exists | Use unique ID |
| 500 | `StorageLimit` | Storage limit reached | Clean up old records |
| 600 | `EncryptionRequired` | Encryption required | Provide encryption proof |
| 700 | `AccessDenied` | Access denied | Check permissions |

### timelock

| Code | Symbol | Description | Remediation |
|------|--------|-------------|-------------|
| 100 | `Unauthorized` | Not authorized | Check caller |
| 207 | `InvalidSignature` | Invalid signature | Check signing key |
| 300 | `NotInitialized` | Not initialized | Call initialize |
| 301 | `AlreadyInitialized` | Already initialized | Cannot reinitialize |
| 302 | `ContractPaused` | Contract paused | Wait for unpause |
| 306 | `DeadlineExceeded` | Deadline exceeded | Create new operation |
| 372 | `NotQueued` | Not in queue | Check queue ID |
| 375 | `AlreadyQueued` | Already queued | Use different ID |
| 376 | `NotReady` | Timelock not elapsed | Wait for ETA |
| 500 | `InsufficientFunds` | Insufficient funds | Add funds |
| 502 | `StorageFull` | Storage full | Clean up |
| 702 | `CrossChainTimeout` | Cross-chain timeout | Retry |

## Script Validation

Run the validation script to check that all error codes are documented:

```bash
./scripts/check_error_codes.sh
```

This script scans all `contracts/**/src/**/*.rs` files for `#[contracterror]` enums and validates that every error variant is documented in this file.
