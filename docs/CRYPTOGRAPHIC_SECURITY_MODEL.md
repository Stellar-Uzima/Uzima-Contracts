# Uzima Contracts — Cryptographic Security Model

This document describes the cryptographic security model implemented in this repository (Uzima medical records on Soroban), the on-chain/off-chain trust boundaries, and how the new cryptographic enhancement contracts are intended to be used together.

## Scope and goals

**Primary security goals**
- Confidentiality of medical record content (diagnosis/treatment payloads) against blockchain observers and unauthorized parties.
- Cryptographically verifiable integrity anchors for off-chain encrypted payloads.
- Strong, auditable governance over cryptographic configuration changes (threshold + timelock).
- Key lifecycle management with rotation and revocation support.
- Upgrade path for post-quantum (PQ) cryptography and hybrid encryption.
- Safe coordination primitives for privacy-preserving analytics (HE/MPC) without exposing plaintext on-chain.

**Non-goals / constraints**
- Soroban contracts cannot practically perform full payload encryption/decryption, homomorphic evaluation, or MPC; these operations are performed off-chain (or in specialized compute environments). On-chain contracts store **references**, **hash anchors**, **key envelope metadata**, and **audit trails**.

## System components

### `contracts/medical_records`
The core system contract:
- Role-based access control (admins/doctors/patients)
- Medical record metadata + history + access logs
- Emergency access grants
- Crypto configuration, encrypted record anchoring, and crypto audit logging
- Threshold + timelock governance for sensitive operations

New crypto-related capabilities:
- `add_encrypted_record` stores **ciphertext_ref**, **ciphertext_hash**, and per-recipient **KeyEnvelope** metadata (E2E-ready).
- `upsert_encrypted_record_envelope` supports **key rotation** by allowing a patient/doctor to replace their own envelope for a record.
- `set_encryption_required` gates plaintext `add_record`/`add_record_with_did` (enforces encrypted-only mode).
- `set_require_pq_envelopes` enforces hybrid/PQ key envelope inclusion.
- `get_crypto_audit_logs` exposes append-only crypto audit entries for admins.
- `propose_crypto_config_update` / `approve_crypto_config_update` / `execute_crypto_config_update` implement threshold + timelock governance for crypto configuration changes.

### `contracts/crypto_registry`
Key registry contract:
- Stores per-address **KeyBundle** entries with **monotonic versioning**
- Supports key rotation (re-register) and revocation
- Stores algorithm-tagged keys (classical + PQ placeholders)

This contract stores public keys + metadata only. Actual cryptographic operations use these keys off-chain.

### `contracts/homomorphic_registry`
Homomorphic encryption (HE) metadata contract:
- Registers HE contexts/parameters by `context_id` with `params_ref` and `params_hash`
- Accepts encrypted computation submissions referencing ciphertext/proof artifacts

### `contracts/mpc_manager`
Secure multi-party computation (MPC) coordination contract:
- Manages MPC sessions with a commit/reveal/finalize lifecycle
- Stores only references + hashes of shares/results/proofs

## End-to-end encryption (E2E) model

### On-chain data model
Encrypted records are represented by:
- `EncryptedRecordHeader`: metadata + ciphertext reference/hash (no key envelopes)
- `EncryptedRecord`: includes the header fields + `Vec<KeyEnvelope>` (per-recipient)

Key envelope model:
- `KeyEnvelope.recipient`: who can decrypt
- `KeyEnvelope.key_version`: the recipient’s `crypto_registry` version used for wrapping
- `KeyEnvelope.algorithm`: `X25519`, `Kyber768`, `HybridX25519Kyber768`, etc.
- `KeyEnvelope.wrapped_key`: classical wrapped DEK blob (format defined off-chain)
- `KeyEnvelope.pq_wrapped_key`: optional PQ wrapped DEK blob (hybrid/PQ)

### Recommended payload encryption
The repository ships scripts under `scripts/crypto/` that implement a working baseline:
- Payload encryption: **AES-256-GCM**
- Envelope wrapping: **ephemeral X25519 + HKDF-SHA256 + AES-256-GCM**
- Hybrid/PQ: scripts accept `pq_wrapped_key_b64` provided by external PQ tooling

These scripts are reference tooling for developers and CI smoke checks; production deployments should harden key storage and use audited cryptographic libraries where required.

### Ciphertext references and integrity anchors
The contract stores:
- `ciphertext_ref`: pointer to encrypted payload (e.g., `ipfs://...`, `ar://...`, or an application storage URI)
- `ciphertext_hash`: SHA-256 hash anchor of the ciphertext blob

Clients should verify:
1) they fetched the correct ciphertext for `ciphertext_ref`
2) `sha256(ciphertext) == ciphertext_hash`
3) they can unwrap the DEK using their `KeyEnvelope`
4) they can decrypt payload with DEK

## Key management, rotation, and revocation

### Key bundle lifecycle
In `crypto_registry`:
- Each address maintains `version: u32` that increments with each rotation.
- Previous versions remain queryable (unless pruned by policy in future), and can be revoked.
- The medical_records contract stores `key_version` on each envelope to bind decryption to a specific public key bundle.

### Rotation flow for existing records
When a recipient rotates their key bundle:
1) Recipient keeps old private key available until migration completes.
2) Recipient fetches existing record envelope for themself: `get_encrypted_record_envelope(record_id)`.
3) Recipient unwraps DEK using old private key.
4) Recipient wraps DEK to the new public key (new key version).
5) Recipient submits new envelope to `upsert_encrypted_record_envelope(record_id, envelope)` (only for their own `recipient`).

The repository includes `scripts/crypto/rewrap_envelope.js` to support steps (3)-(4).

### Revocation considerations
Revoking a key bundle in the registry does not retroactively make already-wrapped DEKs undecryptable (the recipient may still possess private key material). Revocation is primarily:
- A signal to stop using that version for new envelopes
- An input to off-chain policy enforcement and monitoring

## Threshold cryptography for admin operations

Sensitive cryptographic configuration changes use threshold + timelock governance:
- `propose_crypto_config_update`: creates a proposal and auto-approves by proposer
- `approve_crypto_config_update`: collects approvals from distinct admins
- `execute_crypto_config_update`: after timelock, applies changes if approval threshold met

This mechanism reduces single-key admin risk and provides time for operational response.

The same threshold/timelock primitives are also used elsewhere in the contract for recovery operations.

## Privacy-preserving computations (HE + MPC)

### Homomorphic encryption
HE computations are coordinated via `homomorphic_registry`:
- On-chain anchors: context IDs, parameter references/hashes, computation references/hashes, optional proofs
- Off-chain: actual ciphertext computation evaluation

### Secure multi-party computation
MPC sessions are coordinated via `mpc_manager`:
- On-chain: session lifecycle, commitments, reveal references/hashes, result references/hashes
- Off-chain: secure computation and share handling by participants

The medical records contract stores the manager addresses (`set_homomorphic_registry`, `set_mpc_manager`) to let applications discover the official compute endpoints for the deployment.

## Quantum-resistant cryptography preparations

This repository provides:
- PQ key slots in `crypto_registry` (`Kyber768`, `Dilithium3`, `Falcon512` placeholders for hybrid adoption)
- Hybrid envelope support in `medical_records` (`HybridX25519Kyber768` and `pq_wrapped_key`)
- Policy switch: `set_require_pq_envelopes(true)` enforces PQ/hybrid envelopes for encrypted record creation and envelope updates

This enables a staged migration:
1) Start with classical envelopes (`X25519`)
2) Register PQ public keys in `crypto_registry`
3) Enable hybrid in clients and begin storing `pq_wrapped_key`
4) Set `require_pq_envelopes=true` once ecosystem support is ready

## Auditing and monitoring

### On-chain audit logs
The medical records contract maintains:
- Access logs (`get_patient_access_logs`, `get_access_logs`)
- Crypto audit logs (`get_crypto_audit_logs`) for:
  - registry/manager address changes
  - encryption policy changes
  - encrypted record creation and envelope updates
  - threshold proposal lifecycle events

### Off-chain audit tooling
Run `scripts/crypto_audit.sh` to execute:
- formatting checks
- clippy linting
- tests for crypto-related crates
- optional `cargo audit`

