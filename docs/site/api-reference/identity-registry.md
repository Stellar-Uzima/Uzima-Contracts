# Identity Registry

Contract: `identity_registry`

W3C DID-based identity management and credential verification for healthcare providers and patients.

## Key Functions

### `register_did(subject, did_document) → Result<(), Error>`

Register a new DID for a subject.

### `update_did(subject, did_document) → Result<(), Error>`

Update an existing DID document.

### `resolve_did(did) → Option<DIDDocument>`

Resolve a DID to its document.

### `issue_credential(issuer, subject, credential_type, data) → Result<u64, Error>`

Issue a verifiable credential.

### `verify_credential(credential_id) → Result<bool, Error>`

Verify a credential is valid and not revoked.

### `revoke_credential(issuer, credential_id) → Result<(), Error>`

Revoke a previously issued credential.

## DID Method

The contract implements a Stellar-native DID method: `did:stellar:<address>`.
