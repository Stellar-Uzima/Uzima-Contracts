# Uzima Contracts — Threat Model (Cryptographic Security)

This threat model focuses on the cryptographic and privacy properties of the Uzima medical records system on Soroban. It complements `docs/CRYPTOGRAPHIC_SECURITY_MODEL.md`.

## Assets
- **Medical record content**: diagnosis/treatment notes, attachments, structured data.
- **Encryption keys**:
  - Recipient private keys (off-chain, most sensitive)
  - Public keys and key bundle metadata (on-chain registry)
  - Data encryption keys (DEKs) per record (off-chain, transient)
- **Ciphertext artifacts**: encrypted payloads stored off-chain (IPFS/Arweave/S3/etc.).
- **Access control state**: roles, emergency grants, DID links.
- **Audit trails**: access logs + crypto audit logs.
- **Admin configuration**: registry addresses, encryption policy switches.

## Actors
- **Patients**: own their medical record confidentiality; can access their encrypted records.
- **Doctors/providers**: create records; may access records under role/policy constraints.
- **Admins**: manage roles and system configuration; highest privilege.
- **AI coordinators / analytics participants**: submit privacy-preserving outputs and metadata.
- **Chain observers**: anyone with read access to the ledger and contract storage.
- **Off-chain storage operators**: IPFS pinning services, storage gateways, etc.
- **Adversaries**:
  - External attackers (network + endpoint compromise)
  - Insider threats (malicious or compromised doctor/admin)
  - Colluding admins

## Trust boundaries
- **On-chain (Soroban contracts)**: public, globally readable state; strong integrity, no confidentiality.
- **Off-chain storage**: confidentiality depends on encryption and access control at endpoints; integrity verified by ciphertext hashes anchored on-chain.
- **Client devices / key custody**: confidentiality depends on secure key storage and correct client implementation.

## Threats and mitigations

### 1) Plaintext leakage via on-chain storage
**Threat**: medical record content stored directly on-chain becomes public.

**Mitigations**
- Encrypted-only mode: `set_encryption_required(true)` blocks plaintext record creation (`add_record`, `add_record_with_did`).
- Encrypted record model stores only ciphertext references + hashes and key envelope metadata.

**Residual risk**
- Metadata leakage (patient/doctor addresses, timestamps, categories/tags) remains public.

### 2) Ciphertext substitution or tampering in off-chain storage
**Threat**: attacker replaces ciphertext at `ciphertext_ref` with malicious/incorrect data.

**Mitigations**
- `ciphertext_hash` stored on-chain; clients verify `sha256(ciphertext) == ciphertext_hash`.
- Applications should treat mismatches as security incidents and refuse decryption.

### 3) Key compromise (patient/doctor private key stolen)
**Threat**: attacker decrypts records for compromised recipient.

**Mitigations**
- Key rotation via `crypto_registry` key bundles and per-record envelope rewrapping.
- Envelope update controls: `upsert_encrypted_record_envelope` only allows updating caller’s own envelope.
- Operational controls: endpoint hardening, HSMs/secure enclaves where applicable.

**Residual risk**
- Records already encrypted to compromised key version remain decryptable if attacker retains old key material.

### 4) Malicious admin changes cryptographic configuration
**Threat**: a single compromised admin key changes registries or disables encryption.

**Mitigations**
- Threshold + timelock governance:
  - `propose_crypto_config_update`
  - `approve_crypto_config_update`
  - `execute_crypto_config_update`
- Crypto audit log entries provide accountability and incident response inputs.

**Residual risk**
- Colluding admins meeting threshold can still execute malicious changes; timelock provides detection window.

### 5) Unauthorized access via role misuse or emergency access abuse
**Threat**: unauthorized party reads record header/envelope or exploits emergency grants.

**Mitigations**
- RBAC checks for record access.
- Emergency access is time-bound and can be scoped to record IDs.
- Access logs provide visibility of access attempts (including denied access in DID access path).

### 6) Quantum adversary (future) breaking classical public-key crypto
**Threat**: future quantum computers break X25519/Ed25519, compromising key exchange/signatures.

**Mitigations / preparations**
- PQ key slots in `crypto_registry`.
- Hybrid envelopes (`pq_wrapped_key`) + `require_pq_envelopes` policy switch.
- Staged migration plan documented in the security model.

**Residual risk**
- PQ algorithms are placeholders until production PQ libraries and standardization are selected and deployed.

### 7) Privacy-preserving analytics misuse (HE/MPC)
**Threats**
- Analytics participants collude to infer sensitive information.
- Submissions link identities to ciphertexts via metadata correlation.

**Mitigations**
- On-chain coordination contracts store references/hashes only; computations stay off-chain.
- Deployments should enforce additional off-chain privacy controls:
  - differential privacy budgets
  - participant authentication and rate-limits
  - secure attestation of compute environments

## Security assumptions
- Client implementations correctly perform encryption/decryption and envelope handling.
- Recipients protect private key material; production deployments use secure storage.
- Off-chain storage is treated as untrusted; integrity checked against on-chain hashes.
- Admin threshold set is not fully compromised; timelock monitoring exists.

## Known limitations
- Blockchain metadata remains public (addresses, timestamps, categories/tags).
- True HE and MPC evaluation cannot be performed inside the Soroban contract environment; only coordination/anchoring is on-chain.
- Revocation does not retroactively invalidate ciphertext already encrypted to a compromised key.

## Recommended operational controls
- Continuous monitoring of `get_crypto_audit_logs` and access logs.
- Periodic key rotations and incident response runbooks.
- Enable encrypted-only mode in production and require PQ envelopes when feasible.
- Run `scripts/crypto_audit.sh` in CI for crypto-related crates and linting.

