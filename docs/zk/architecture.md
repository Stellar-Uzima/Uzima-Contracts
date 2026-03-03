# Uzima ZK Architecture

## Scope
This design adds privacy-preserving access verification for `medical_records` using:
- `contracts/zk_verifier`: lightweight on-chain verifier for off-chain ZK attestations.
- `contracts/credential_registry`: issuer-controlled credential root registry.
- `contracts/medical_records`: ACL + ZK gate integration for record access paths.

## Why This Design
Full Groth16/PLONK verifier execution inside Soroban is costly in code size and host budget for this repository's constraints.  
The implementation uses a practical hybrid:
- Off-chain ZK proving and verification.
- On-chain attestation checking with versioned verifying-key commitments and proof/public-input hashes.

This keeps proof generation heavy off-chain and keeps on-chain checks deterministic and cheap.

## Components
### 1) Credential Registry Contract
- Stores per-issuer credential roots (versioned).
- Supports admin/issuer-admin updates.
- Supports root revocation and revocation-root storage.
- Provides:
  - `get_active_root(issuer) -> Option<BytesN<32>>`
  - `is_root_revoked(issuer, root) -> bool`

### 2) ZK Verifier Contract
- Stores versioned verifying-key metadata (`vk_hash`, `circuit_id`, `attestor`).
- Accepts authenticated attestations from trusted attestor address:
  - `(vk_version, public_inputs_hash, proof_hash, verified, ttl)`
- Verifies by matching:
  - attestation exists,
  - not expired,
  - verified=true,
  - active key version.

### 3) Medical Records Contract Integration
- Stores record commitments.
- Adds admin config:
  - verifier contract address,
  - credential registry address,
  - ZK enforcement toggle,
  - ZK grant TTL.
- Adds `submit_zk_access_proof(...)`:
  - validates public inputs against on-chain state,
  - checks credential root and revocation status,
  - verifies proof through `zk_verifier`,
  - enforces nullifier one-time usage,
  - issues short-lived ZK access grant,
  - emits privacy-preserving audit event.
- Access methods require:
  - existing ACL pass,
  - valid ZK grant (if enforcement enabled),
  - checked before returning privileged data or access events.

## Public Input Model
`ZkPublicInputs` includes:
- `record_id`
- `record_commitment`
- `credential_root`
- `issuer`
- `requester_commitment`
- `provider_commitment`
- `claim_commitment`
- `min_timestamp`, `max_timestamp`
- `nullifier`
- `pseudonym`
- `vk_version`

## Text Flow Diagram
1. Issuer publishes credential root in `credential_registry`.
2. Admin registers verifier key version in `zk_verifier`.
3. Holder generates ZK proof off-chain for record access predicate.
4. Trusted attestor validates the ZK proof off-chain and submits attestation on-chain.
5. Holder calls `medical_records.submit_zk_access_proof(...)`.
6. `medical_records` validates:
   - commitment and predicate bounds,
   - credential root consistency,
   - nullifier uniqueness,
   - verifier attestation match.
7. On success, short-lived grant is stored.
8. `get_record`/`get_record_with_did`/encrypted header-envelope access require ACL AND valid grant.
9. Audit event logs record id, pseudonym, timestamp, result, optional nullifier only.

## Data Leakage Boundary
Never logged on-chain:
- diagnosis/treatment plaintext from proof witness,
- credential claim bodies,
- credential subject identity in clear.

Revealed:
- record identifier,
- proof verification result,
- pseudonymous requester hash,
- nullifier hash.

