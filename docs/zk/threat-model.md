# ZK Threat Model

## Assets
- Medical record confidentiality.
- Credential validity and issuer trust chain.
- Access-control integrity (ACL + ZK gate).
- Audit integrity.

## Adversaries
1. Unauthorized reader:
   Tries to access records without valid role/permission or proof.
2. Replay attacker:
   Reuses previously accepted proof artifacts.
3. Malicious claimant:
   Submits mismatched public inputs (wrong root, wrong commitment, wrong timestamp bounds).
4. Malicious attestor compromise:
   Trusted attestor key or account is abused.
5. Storage observer:
   Reads all public state and events to infer sensitive data.

## Security Goals
1. No privileged read unless ACL passes and ZK gate passes (when enforcement is enabled).
2. Replay-resistant proof use (nullifier one-time semantics).
3. Credential root consistency with issuer-managed registry.
4. Auditable outcomes without claim leakage.

## Non-Goals
- Hiding all metadata (record ID and time are still visible).
- Defending against a fully malicious trusted attestor without additional governance controls.

## Trust Assumptions
1. Contract admin keys are uncompromised.
2. Credential issuers/issuer-admins manage roots correctly.
3. Trusted attestor does honest off-chain ZK verification.
4. Soroban host cryptographic primitives and storage semantics are correct.

## Mitigations
1. Admin-gated configuration updates.
2. Versioned VK metadata.
3. Nullifier uniqueness checks.
4. Revocation checks via credential registry.
5. Minimal event surface for privacy.

