# Security Model

See [docs/MASTER_THREAT_MODEL.md](../../MASTER_THREAT_MODEL.md) and [docs/CRYPTOGRAPHIC_SECURITY_MODEL.md](../../CRYPTOGRAPHIC_SECURITY_MODEL.md).

## Threat Categories

- **Reentrancy** — Mitigated by CEI pattern and reentrancy guards
- **Access Control** — Mitigated by `require_auth()` and RBAC
- **State Manipulation** — Mitigated by status state machines
- **Resource Exhaustion** — Mitigated by storage TTL and batch limits
- **Cross-Contract Attacks** — Mitigated by authorization context checks

## Cryptographic Security

- Records encrypted with patient's public key (client-side)
- ZK proofs via `zk_verifier` for privacy-preserving verification
- ABE (Attribute-Based Encryption) via `abe_sdk` for fine-grained access
