# Uzima Contracts — Error Code Reference

> **Last updated:** 2026-05-30  
> **Soroban SDK:** v23.x  
> **Contracts covered:** `medical_records`, `access_control`, `governance`, `identity_registry`

Soroban contract errors are `u32` enum variants exposed as `Error(Contract, #N)` in transaction
results and diagnostic events. Every error code in this file is **unique across all contracts**
and is grouped first by contract, then by category.

When a transaction fails, inspect the `result_xdr` or run:

```bash
stellar contract invoke ... 2>&1 | grep "Error(Contract"
```

The `#N` value maps directly to the **Code** column below.

---

## Table of Contents

- [medical\_records](#medical_records-contract)
  - [Authentication \& Authorization](#medical_records--authentication--authorization)
  - [Validation](#medical_records--validation)
  - [State](#medical_records--state)
  - [Storage](#medical_records--storage)
- [access\_control](#access_control-contract)
  - [Authentication \& Authorization](#access_control--authentication--authorization)
  - [Validation](#access_control--validation)
  - [State](#access_control--state)
- [identity\_registry](#identity_registry-contract)
  - [Authentication \& Authorization](#identity_registry--authentication--authorization)
  - [Validation](#identity_registry--validation)
  - [State](#identity_registry--state)
- [governance](#governance-contract)
  - [Authentication \& Authorization](#governance--authentication--authorization)
  - [Validation](#governance--validation)
  - [State](#governance--state)
  - [Timelock](#governance--timelock)
- [Appendix: Error Code Index](#appendix-error-code-index)

---

## `medical_records` Contract

**Source:** `contracts/medical_records/src/lib.rs`  
**Code range:** `1xx`

### `medical_records` — Authentication & Authorization

| Code | Symbolic Name | Description | Common Causes | Remediation |
|------|--------------|-------------|---------------|-------------|
| `101` | `Unauthorized` | Caller does not have permission to perform this action. | Calling a restricted function without the required role or being neither the record owner nor an authorized provider. | Ensure the calling address holds the correct role. Check `has_role(address, role)` before invoking the function. |
| `102` | `NotAdmin` | Caller is not the contract administrator. | Attempting an admin-only operation (e.g. `initialize`, `grant_admin`) from a non-admin address. | Only the address stored in `DataKey::Admin` may call admin functions. Retrieve the current admin with `get_admin()`. |
| `103` | `NotPatient` | Caller is not the patient referenced in the record. | Trying to read or update a record where `caller != record.patient`. | Invoke the function from the patient's own keypair, or obtain an explicit delegation via `grant_access`. |
| `104` | `NotProvider` | Caller is not a registered healthcare provider. | Writing a medical record from an address that was never registered as a provider. | Register the address using `register_provider` (admin only) before attempting record writes. |
| `105` | `AccessRevoked` | A previously granted access permission has been revoked. | Patient called `revoke_access` after earlier granting access to this provider. | Request re-authorization from the patient via `grant_access`. |
| `106` | `AccessExpired` | The time-limited access grant has passed its expiry timestamp. | `grant_access` was called with a finite TTL that has now elapsed. | Request a new access grant from the patient with a fresh expiry. |

### `medical_records` — Validation

| Code | Symbolic Name | Description | Common Causes | Remediation |
|------|--------------|-------------|---------------|-------------|
| `111` | `InvalidPatientId` | The provided patient ID string is empty or exceeds the maximum length. | Passing `""` or a string longer than 64 bytes as `patient_id`. | Provide a non-empty patient ID of at most 64 UTF-8 bytes. |
| `112` | `InvalidRecordData` | The encrypted record payload is empty or malformed. | Submitting an empty `BytesN` or a payload below the minimum size. | Encrypt the record data off-chain before submission; ensure the byte slice is non-empty. |
| `113` | `InvalidTimestamp` | The provided timestamp is zero or in the future beyond the allowed skew. | Passing `0` or a timestamp more than 60 seconds ahead of ledger time. | Use `env.ledger().timestamp()` as the base and do not submit records with future-dated timestamps. |
| `114` | `InvalidMetadata` | Record metadata key or value exceeds permitted length or contains disallowed characters. | Metadata strings longer than 128 bytes, or keys containing `/` or `\`. | Limit metadata key/value pairs to 128 bytes; use only alphanumeric characters, `-`, and `_`. |
| `115` | `InvalidEncryptionKey` | The public encryption key submitted with the record is invalid. | Zero-length key or a key that fails basic length validation. | Supply a valid 32-byte (Ed25519) or 65-byte (secp256k1 uncompressed) public key. |

### `medical_records` — State

| Code | Symbolic Name | Description | Common Causes | Remediation |
|------|--------------|-------------|---------------|-------------|
| `121` | `RecordNotFound` | No medical record exists for the given patient + record ID combination. | Querying a record ID that was never created, or after the record's TTL expired and was archived. | Verify the record ID via `list_records(patient_id)` before reading. Extend TTL proactively. |
| `122` | `RecordAlreadyExists` | A record with this ID already exists for this patient. | Calling `write_record` with a duplicate record ID. | Use a unique record ID (e.g. include a timestamp or UUID suffix). |
| `123` | `PatientNotRegistered` | The patient address is not registered in the contract. | Writing a record for an address that was never registered as a patient. | Call `register_patient` before performing record operations. |
| `124` | `ProviderNotRegistered` | The provider address is not registered in the contract. | Attempting a provider action from an unregistered address. | Call `register_provider` (admin only) to onboard the provider first. |
| `125` | `ContractNotInitialized` | The contract has not been initialized yet. | Calling any function before `initialize` has been executed. | Deploy the contract and immediately call `initialize(admin, ...)` before any other interaction. |
| `126` | `ContractAlreadyInitialized` | `initialize` was called on a contract that is already set up. | Replay attack or accidental double-call of `initialize`. | No remediation needed — the initial call succeeded. If you are upgrading, use `upgrade` instead. |

### `medical_records` — Storage

| Code | Symbolic Name | Description | Common Causes | Remediation |
|------|--------------|-------------|---------------|-------------|
| `131` | `StorageLimitExceeded` | The number of records for this patient has reached the maximum allowed. | A patient accumulating more records than the `MAX_RECORDS_PER_PATIENT` constant. | Archive or delete old records before adding new ones. Consider batching records into a single entry. |
| `132` | `TtlExtensionFailed` | Extending the TTL of a storage entry failed. | Ledger constraints or the entry no longer existing at the time of extension. | Re-check that the entry exists before extending. Ensure the contract account has sufficient XLM for storage fees. |

---

## `access_control` Contract

**Source:** `contracts/access_control/src/lib.rs`  
**Code range:** `2xx`

### `access_control` — Authentication & Authorization

| Code | Symbolic Name | Description | Common Causes | Remediation |
|------|--------------|-------------|---------------|-------------|
| `201` | `Unauthorized` | Caller lacks the required role for this operation. | Calling a role-gated function without the role being granted to the caller's address. | Use `has_role(address, role)` to check before invoking. Request role assignment from an admin. |
| `202` | `AdminOnly` | This function is restricted to the contract admin. | Non-admin address attempting to grant/revoke roles or update admin. | Only the address in `DataKey::Admin` may perform this action. |
| `203` | `CannotRevokeLastAdmin` | Revoking this admin would leave the contract with no admin. | Calling `revoke_role(admin_address, ADMIN_ROLE)` when only one admin exists. | Grant `ADMIN_ROLE` to a second address before revoking the first. |

### `access_control` — Validation

| Code | Symbolic Name | Description | Common Causes | Remediation |
|------|--------------|-------------|---------------|-------------|
| `211` | `InvalidRole` | The role identifier provided does not match any known role. | Passing an arbitrary `Symbol` that was never registered as a valid role. | Use only the role constants exported by the contract: `ADMIN_ROLE`, `PROVIDER_ROLE`, `AUDITOR_ROLE`. |
| `212` | `InvalidAddress` | A zero or otherwise invalid address was supplied. | Passing `Address::default()` or a syntactically malformed address string. | Validate that the address is non-zero before calling. |

### `access_control` — State

| Code | Symbolic Name | Description | Common Causes | Remediation |
|------|--------------|-------------|---------------|-------------|
| `221` | `RoleAlreadyGranted` | The role has already been granted to this address. | Calling `grant_role` on an address that already holds the role. | Check `has_role` first; idempotent grants can be safely skipped. |
| `222` | `RoleNotFound` | Attempting to revoke a role that the address does not hold. | Calling `revoke_role` on an address that never had this role. | Check `has_role` before revoking to avoid this error. |

---

## `identity_registry` Contract

**Source:** `contracts/identity_registry/src/lib.rs`  
**Code range:** `3xx`

### `identity_registry` — Authentication & Authorization

| Code | Symbolic Name | Description | Common Causes | Remediation |
|------|--------------|-------------|---------------|-------------|
| `301` | `Unauthorized` | Caller is not authorized to modify this identity record. | Attempting to update an identity that belongs to a different address. | Identity records may only be updated by the owner address or the contract admin. |
| `302` | `RegistrarOnly` | This function is reserved for registered registrars. | Non-registrar address calling `attest_identity` or `revoke_identity`. | Contact the admin to be granted `REGISTRAR_ROLE` before attesting identities. |

### `identity_registry` — Validation

| Code | Symbolic Name | Description | Common Causes | Remediation |
|------|--------------|-------------|---------------|-------------|
| `311` | `InvalidPublicKey` | The submitted public key fails format or length validation. | Empty key, wrong byte length, or a key that fails the curve point check. | Provide a valid 32-byte Ed25519 public key or 65-byte uncompressed secp256k1 key. |
| `312` | `InvalidCredentialHash` | The credential hash is not exactly 32 bytes. | Passing a truncated or extended SHA-256 hash. | Ensure you compute `SHA-256(credential_data)` and pass the raw 32-byte output. |
| `313` | `InvalidAttestationData` | The attestation payload is empty or exceeds the maximum size. | Empty byte array or a payload larger than 1 KB. | Keep attestation data under 1,024 bytes; store larger documents off-chain and hash them. |

### `identity_registry` — State

| Code | Symbolic Name | Description | Common Causes | Remediation |
|------|--------------|-------------|---------------|-------------|
| `321` | `IdentityNotFound` | No identity record exists for this address. | Querying an address that was never registered, or after the record expired. | Register the identity via `register_identity` before performing lookups. |
| `322` | `IdentityAlreadyRegistered` | An identity record already exists for this address. | Calling `register_identity` twice for the same address. | Use `update_identity` to modify an existing record instead. |
| `323` | `IdentityRevoked` | The identity has been explicitly revoked by a registrar or admin. | Using an identity that was revoked due to fraud, expiry, or policy violation. | Contact the registrar to re-attest and reinstate the identity if appropriate. |
| `324` | `CredentialAlreadyRevoked` | The credential was already marked as revoked. | Attempting to revoke a credential that is already in the `Revoked` state. | No action needed; the credential is already invalid. |

---

## `governance` Contract

**Source:** `contracts/governance/src/lib.rs`  
**Code range:** `4xx`

### `governance` — Authentication & Authorization

| Code | Symbolic Name | Description | Common Causes | Remediation |
|------|--------------|-------------|---------------|-------------|
| `401` | `Unauthorized` | Caller is not permitted to perform this governance action. | Non-governor address calling `queue`, `execute`, or `cancel`. | Only the Governor contract address or an admin may call these functions. |
| `402` | `NotProposer` | Caller is not the original proposer of this proposal. | Attempting to cancel a proposal from an address that did not create it. | Only the proposer or a governance admin can cancel a proposal before it is queued. |

### `governance` — Validation

| Code | Symbolic Name | Description | Common Causes | Remediation |
|------|--------------|-------------|---------------|-------------|
| `411` | `InvalidProposalId` | The proposal ID is zero or does not reference any known proposal. | Passing `0` or an ID that was never returned by `propose`. | Retrieve valid proposal IDs from emitted `ProposalCreated` events or via `get_proposal`. |
| `412` | `InvalidVotingPeriod` | The voting period is below the minimum or above the maximum allowed duration. | Specifying a period shorter than `MIN_VOTING_PERIOD` or longer than `MAX_VOTING_PERIOD`. | Use a voting period between `MIN_VOTING_PERIOD` (1 day) and `MAX_VOTING_PERIOD` (30 days). |
| `413` | `InvalidCalldata` | The proposal calldata is empty or exceeds the permitted size. | Submitting a proposal with no actions or with an action payload larger than 4 KB. | Ensure each calldata entry is non-empty and under 4,096 bytes. |
| `414` | `QuorumNotMet` | The proposal did not reach the required quorum of votes. | Too few eligible voters participated before the voting period ended. | Increase voter engagement or lower the quorum threshold via governance itself. |

### `governance` — State

| Code | Symbolic Name | Description | Common Causes | Remediation |
|------|--------------|-------------|---------------|-------------|
| `421` | `ProposalNotFound` | No proposal exists with the given ID. | Querying a proposal ID that was never created or has been fully cleared from storage. | Only query proposal IDs that appear in `ProposalCreated` events. |
| `422` | `ProposalNotActive` | The proposal is not in an `Active` state and cannot accept votes. | Voting on a proposal that is `Pending`, `Defeated`, `Queued`, or `Executed`. | Check `get_proposal_state(id)` before calling `cast_vote`. |
| `423` | `ProposalAlreadyExecuted` | The proposal has already been executed and cannot be executed again. | Calling `execute` on a proposal that is already in the `Executed` state. | No action required; the proposal effects are already applied. |
| `424` | `ProposalDefeated` | The proposal was defeated (failed quorum or majority) and cannot proceed. | Attempting to queue or execute a defeated proposal. | A new proposal must be submitted. |
| `425` | `AlreadyVoted` | This address has already cast a vote on this proposal. | Calling `cast_vote` more than once from the same address for the same proposal. | Each address may vote once. Verify vote status with `get_vote(proposal_id, address)`. |
| `426` | `VotingClosed` | The voting period for this proposal has ended. | Calling `cast_vote` after `proposal.vote_end` ledger timestamp. | Votes cannot be submitted after the voting period closes. Monitor `vote_end` via `get_proposal`. |

### `governance` — Timelock

| Code | Symbolic Name | Description | Common Causes | Remediation |
|------|--------------|-------------|---------------|-------------|
| `431` | `TimelockNotExpired` | The timelock delay has not yet elapsed; the proposal cannot be executed. | Calling `execute` before `execute_after` timestamp is reached. | Wait for the timelock period to pass. Check `get_execute_after(proposal_id)` for the exact timestamp. |
| `432` | `TimelockNotQueued` | The proposal has not been queued in the timelock yet. | Calling `execute` on a proposal that passed voting but was never queued. | Call `queue(proposal_id)` first, then wait for the timelock, then call `execute`. |
| `433` | `TimelockExpired` | The execution window after the timelock has closed. | Calling `execute` after the `GRACE_PERIOD` following `execute_after` has elapsed. | The proposal must be re-queued (if permitted by governance) or a new proposal submitted. |
| `434` | `NoPendingOperation` | There is no pending timelock operation to cancel or execute. | Calling `cancel` or `execute` when no operation is queued for this proposal. | Verify the proposal is in `Queued` state via `get_proposal_state` before operating on the timelock. |

---

## Appendix: Error Code Index

Quick-reference table sorted by code number.

| Code | Contract | Symbolic Name |
|------|----------|--------------|
| 101 | medical_records | Unauthorized |
| 102 | medical_records | NotAdmin |
| 103 | medical_records | NotPatient |
| 104 | medical_records | NotProvider |
| 105 | medical_records | AccessRevoked |
| 106 | medical_records | AccessExpired |
| 111 | medical_records | InvalidPatientId |
| 112 | medical_records | InvalidRecordData |
| 113 | medical_records | InvalidTimestamp |
| 114 | medical_records | InvalidMetadata |
| 115 | medical_records | InvalidEncryptionKey |
| 121 | medical_records | RecordNotFound |
| 122 | medical_records | RecordAlreadyExists |
| 123 | medical_records | PatientNotRegistered |
| 124 | medical_records | ProviderNotRegistered |
| 125 | medical_records | ContractNotInitialized |
| 126 | medical_records | ContractAlreadyInitialized |
| 131 | medical_records | StorageLimitExceeded |
| 132 | medical_records | TtlExtensionFailed |
| 201 | access_control | Unauthorized |
| 202 | access_control | AdminOnly |
| 203 | access_control | CannotRevokeLastAdmin |
| 211 | access_control | InvalidRole |
| 212 | access_control | InvalidAddress |
| 221 | access_control | RoleAlreadyGranted |
| 222 | access_control | RoleNotFound |
| 301 | identity_registry | Unauthorized |
| 302 | identity_registry | RegistrarOnly |
| 311 | identity_registry | InvalidPublicKey |
| 312 | identity_registry | InvalidCredentialHash |
| 313 | identity_registry | InvalidAttestationData |
| 321 | identity_registry | IdentityNotFound |
| 322 | identity_registry | IdentityAlreadyRegistered |
| 323 | identity_registry | IdentityRevoked |
| 324 | identity_registry | CredentialAlreadyRevoked |
| 401 | governance | Unauthorized |
| 402 | governance | NotProposer |
| 411 | governance | InvalidProposalId |
| 412 | governance | InvalidVotingPeriod |
| 413 | governance | InvalidCalldata |
| 414 | governance | QuorumNotMet |
| 421 | governance | ProposalNotFound |
| 422 | governance | ProposalNotActive |
| 423 | governance | ProposalAlreadyExecuted |
| 424 | governance | ProposalDefeated |
| 425 | governance | AlreadyVoted |
| 426 | governance | VotingClosed |
| 431 | governance | TimelockNotExpired |
| 432 | governance | TimelockNotQueued |
| 433 | governance | TimelockExpired |
| 434 | governance | NoPendingOperation |

---

> **Contributing:** When adding a new error variant, assign the next available code in the
> contract's range, add an entry to this file, and run `scripts/validate_error_codes.sh` locally
> before opening a PR. The CI pipeline will reject PRs where the script reports undocumented errors.