# ZK Security Analysis

## Soundness Assumptions
The on-chain verifier enforces soundness through a trusted-attestor model:
- Off-chain attestor validates ZK proof against circuit + VK.
- On-chain checks attestation integrity by matching:
  - `vk_version`,
  - `public_inputs_hash`,
  - `proof_hash`,
  - unexpired attestation.

If attestor is honest, forged proofs are rejected.

## Trusted Setup Implications
If a SNARK with trusted setup is used off-chain:
- Setup toxic waste compromise can break proof soundness.
- Mitigation:
  - ceremony transparency,
  - per-circuit VK versioning,
  - rapid VK rotation,
  - circuit-id pinning in verifier metadata.

For transparent proof systems (STARK-like), setup risk is reduced but verification cost may increase.

## Key Management Risks
1. Admin key compromise:
   Attacker could swap verifier/registry endpoints or disable enforcement.
2. Attestor compromise:
   Attacker could submit `verified=true` for invalid proofs.

Mitigations:
- multi-sig governance for admin operations,
- short attestation TTL,
- key rotation and emergency deactivation of VK versions,
- off-chain monitoring on verifier and registry events.

## Replay Protection
Nullifier hashes are stored and enforced one-time during proof submission.
- Reuse is rejected (`CredentialRevoked` path).
- This blocks proof replay even when witness/public inputs are otherwise valid.

## Side-Channel Considerations
1. Timing:
   Verification path should avoid data-dependent heavy branches.
2. Event payloads:
   Never include witness values or credential claims.
3. Error signaling:
   Avoid over-specific failures that leak protected values.

## Failure Modes and Safe Defaults
1. ZK config missing while enforcement is enabled:
   Verification fails closed.
2. Expired/missing grant:
   Access denied.
3. Revoked credential root:
   Proof submission denied.
4. Unknown VK version:
   Verification fails.
5. Malformed proof bytes:
   Proof hash mismatch -> fail.

## Trade-off Statement
This implementation favors Soroban practicality and deterministic gas over in-contract full SNARK arithmetic by using attested verification.  
Security therefore depends on attestor trust and governance controls in addition to cryptographic proof validity.

