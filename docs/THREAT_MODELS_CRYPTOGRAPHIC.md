# Cryptographic Threat Models

## 1. Key Compromise and Theft

**Threat**: Private keys (encryption, signing, or post-quantum keys) are stolen or compromised, allowing attackers to decrypt sensitive data or forge signatures.

**Attack Vectors**:
- Compromise of off-chain private key storage (HSM, secure enclave, software wallet)
- Side-channel attacks on key usage (timing, power analysis, electromagnetic)
- Memory scraping attacks on client devices
- Malware or keylogger on user devices
- Social engineering to obtain key material or passphrases
- Insider threats with legitimate access to key material
- Quantum computing attacks on classical public-key algorithms (X25519, Ed25519)
- Weak random number generation during key creation

**Mitigations**:
- Hardware security modules (HSMs) or secure enclaves for key storage
- Multi-factor authentication for key access
- Key rotation via `crypto_registry` key bundle versioning
- Per-record envelope rewrapping (`upsert_encrypted_record_envelope`) to limit exposure
- Hybrid encryption with post-quantum algorithms (Kyber, Dilithium) as placeholders
- `require_pq_envelopes` policy to enforce PQ/hybrid key usage
- Quantum threat detection and migration procedures (`QuantumThreatDetected`, `QuantumMigrationStarted`)
- Secure key generation with proper entropy sources
- Key usage policies (time-bounded, purpose-limited)
- Regular key rotation schedules
- Separation of duties: different keys for different operations
- Threshold cryptography for sensitive operations (multiple key holders required)

**Residual Risk**:
- Zero-day vulnerabilities in cryptographic implementations
- Advanced persistent threats with long-term access to key material
- Quantum computers breaking current algorithms (transition period risk)
- Human factors (social engineering, insider threats) remain difficult to fully mitigate
- Backup and recovery procedures may create additional attack surface

## 2. Cryptographic Algorithm Vulnerabilities

**Threat**: Cryptographic algorithms or implementations have weaknesses that allow bypassing security protections.

**Attack Vectors**:
- Mathematical breakthroughs in factoring, discrete logarithm, or lattice problems
- Implementation bugs (buffer overflows, memory corruption in native code)
- Weak random number generation affecting key creation
- Side-channel vulnerabilities in cryptographic libraries
- Protocol-level flaws in encryption schemes (e.g., padding oracle, nonce reuse)
- Hash function collisions or preimage attacks
- Post-quantum algorithm vulnerabilities (some are relatively new and less battle-tested)
- Hybrid encryption implementation flaws (combining classical and PQ incorrectly)

**Mitigations**:
- Use well-vetted, standardized algorithms (X25519, Ed25519, AES-256-GCM, SHA-256)
- Post-quantum algorithms as placeholders with hybrid approach (Kyber768, Dilithium3)
- Regular security audits of cryptographic implementations
- Formal verification where possible
- Constant-time implementations to prevent timing attacks
- Proper nonce/IV generation and management
- Algorithm agility: ability to migrate to new algorithms (`CryptoConfigProposal`)
- Hybrid envelopes provide defense-in-depth during transition
- Quantum threat level monitoring and migration triggers
- Separation of concerns: different algorithms for different purposes (encryption vs signing)
- Key length and parameter validation in `crypto_registry`

**Residual Risk**:
- Unknown vulnerabilities in cryptographic primitives (especially newer PQ algorithms)
- Implementation bugs despite best practices
- Quantum computing advances may break current assumptions faster than expected
- Supply chain attacks on cryptographic libraries
- Nation-state level cryptanalytic capabilities

## 3. Ciphertext Manipulation and Replay

**Threat**: Attackers modify, substitute, or replay encrypted data to cause harm or gain unauthorized access.

**Attack Vectors**:
- Replace `ciphertext_ref` to point to malicious encrypted content
- Modify `ciphertext_hash` to hide tampering (if hash not properly verified)
- Replay previously valid encrypted records or envelopes
- Manipulate key envelope metadata to redirect decryption
- Exploit malleability in encryption schemes
- Replay ZK proofs or access grants
- Modify off-chain storage content while keeping on-chain hash unchanged

**Mitigations**:
- `ciphertext_hash` stored on-chain for integrity verification
- Clients must verify `sha256(ciphertext) == ciphertext_hash` before decryption
- Key envelopes bound to specific record IDs and recipient addresses
- Per-record envelope updates only allowed for the recipient (`upsert_encrypted_record_envelope`)
- ZK proof nonces and nullifiers prevent replay
- Timestamp validation in ZK public inputs
- Commitment schemes bind data to specific values
- Event logging for all encrypted record operations
- Access logs track who accessed what and when
- Quantum audit logs track all crypto configuration changes

**Residual Risk**:
- Hash collisions (theoretically possible but practically infeasible for SHA-256)
- Implementation bugs in client-side verification
- Social engineering to trick users into accepting malicious ciphertext
- Compromised client devices could bypass verification
- Race conditions in envelope updates

## 4. Cryptographic Configuration Subversion

**Threat**: Attackers modify cryptographic configuration to weaken security or disable protections.

**Attack Vectors**:
- Change `encryption_required` to allow plaintext records
- Disable `require_pq_envelopes` to weaken post-quantum protections
- Modify `crypto_registry`, `homomorphic_registry`, or `mpc_manager` addresses
- Manipulate quantum threat level to delay migration
- Corrupt key bundle versions or algorithm tags
- Exploit governance to approve malicious configuration changes

**Mitigations**:
- Threshold + timelock governance for crypto config changes
- `propose_crypto_config_update` / `approve_crypto_config_update` / `execute_crypto_config_update`
- Timelock delay (86,400 seconds / 1 day) for community review
- Approval threshold (2) requiring multiple admin agreements
- Crypto audit logs track all configuration changes
- Events emitted for all crypto lifecycle events
- Separation of admin powers across different functions
- Ability to detect and revert malicious configurations
- Regular monitoring of crypto configuration state

**Residual Risk**:
- Governance attack compromising threshold of admins
- Time window vulnerability during timelock period
- Insider threat from admin group
- Requires active community governance participation

## 5. Post-Quantum Migration Failures

**Threat**: Migration to post-quantum cryptography fails or creates vulnerabilities during transition.

**Attack Vectors**:
- Premature disabling of classical crypto before PQ is ready
- Incorrect hybrid envelope implementations
- Key generation failures during migration
- Verification failures causing denial of service
- Rollback attacks to weaker classical crypto
- Incomplete migration leaving some data unprotected
- Compatibility issues with external systems

**Mitigations**:
- Staged migration plan with hybrid envelopes
- `require_pq_envelopes` policy switch for gradual enforcement
- PQ key slots in `crypto_registry` for parallel operation
- Quantum threat level monitoring and gradual response
- Ability to revert configuration changes via governance
- Comprehensive testing of PQ implementations
- Fallback mechanisms for PQ failures
- Clear migration timeline and communication
- Regular drills and testing of migration procedures

**Residual Risk**:
- Unforeseen issues in PQ algorithms under real-world conditions
- Migration complexity creates operational errors
- External dependencies may not support PQ crypto
- Time pressure from quantum advances may force rushed migration
- Some legacy data may remain encrypted with classical algorithms

## 6. Homomorphic Encryption and MPC Vulnerabilities

**Threat**: Weaknesses in HE/MPC implementations leak sensitive data or allow computation manipulation.

**Attack Vectors**:
- Exploiting HE scheme vulnerabilities to recover plaintext
- Manipulating HE computation inputs or parameters
- MPC protocol deviations or malicious participant behavior
- Side-channel attacks on HE/MPC implementations
- Verifiable computation proof forgery
- Data poisoning through malicious inputs
- Model inversion attacks on HE-encrypted data

**Mitigations**:
- HE/MPC coordination via dedicated contracts (`homomorphic_registry`, `mpc_manager`)
- On-chain anchors for parameters, ciphertexts, and proofs
- Reference implementations with security audits
- Parameter validation and bounds checking
- Proof verification where applicable
- Participant authentication and authorization
- Rate limiting and access controls
- Off-chain computation with on-chain verification
- Secure multi-party computation protocols with guaranteed output delivery
- Differential privacy budgets for analytics

**Residual Risk**:
- HE/MPC are active research areas with potential undiscovered vulnerabilities
- Performance vs. security tradeoffs may lead to weaker configurations
- Malicious insiders in MPC protocols
- Complexity increases attack surface
- Requires specialized expertise to implement correctly

## 7. Quantum Computing Threats

**Threat**: Quantum computers break classical public-key cryptography, enabling decryption of historical data and forgery of signatures.

**Attack Vectors**:
- Shor's algorithm breaking X25519, Ed25519, RSA, ECC
- Grover's algorithm reducing symmetric key security
- Harvest-now-decrypt-later attacks on encrypted data
- Forgery of historical signatures
- Breaking hash-based signatures (if quantum-secure alternatives not used)
- Compromise of long-term key material

**Mitigations**:
- Post-quantum algorithm placeholders in `crypto_registry`
- Hybrid envelopes combining classical and PQ algorithms
- `require_pq_envelopes` enforcement option
- Quantum threat level monitoring (`QuantumThreatLevel`)
- Migration procedures (`QuantumMigrationStarted`, `QuantumMigrationCompleted`)
- Key rotation to PQ algorithms
- Re-encryption of sensitive historical data
- Quantum-resistant signature schemes (Dilithium, Falcon)
- Lattice-based KEMs (Kyber)
- Code-based cryptography (McEliece) as backup
- Hash-based signatures (XMSS, SPHINCS+) for specific use cases

**Residual Risk**:
- Quantum computers may arrive before migration completes
- Some historical data may be irretrievably compromised
- PQ algorithms may have undiscovered vulnerabilities
- Migration complexity and coordination challenges
- Performance overhead of PQ algorithms
- Key sizes and computational requirements may be prohibitive

## 8. Cryptographic Implementation Bugs

**Threat**: Bugs in cryptographic code lead to security bypasses or data loss.

**Attack Vectors**:
- Memory safety issues in Rust code (despite language protections)
- Integer overflow/underflow in cryptographic operations
- Incorrect protocol implementations
- Poor entropy for random number generation
- Key material exposure through logs or error messages
- Timing side-channels in cryptographic operations
- Cache-based side-channel attacks
- Power analysis attacks on embedded devices
- Fault injection attacks

**Mitigations**:
- Memory-safe language (Rust) with additional safety checks
- Comprehensive testing including fuzzing
- Formal methods for critical components
- Code audits by cryptographic experts
- Secure coding practices and guidelines
- Constant-time implementations
- Protection against fault injection
- Secure key handling (zeroization, secure storage)
- Minimal attack surface
- Defense in depth with multiple security layers
- Regular security updates and patches

**Residual Risk**:
- Zero-day vulnerabilities despite best practices
- Sophisticated attackers with significant resources
- Supply chain attacks on dependencies
- Human error in implementation or configuration
- Hardware-level vulnerabilities (TEMPEST, Rowhammer, etc.)

## 9. Cryptographic Key Management Failures

**Threat**: Poor key management practices lead to key compromise or loss.

**Attack Vectors**:
- Inadequate key storage protection
- Insufficient key rotation
- Poor key backup and recovery procedures
- Key material exposure during generation or usage
- Insecure key distribution
- Lack of key usage policies
- No key revocation procedures
- Single points of failure in key management
- Inadequate separation of duties

**Mitigations**:
- HSM or secure enclave usage for key storage
- Automated key rotation via `crypto_registry`
- Multi-party key management (threshold schemes)
- Secure key generation with proper entropy
- Key usage policies and audit trails
- Key revocation procedures
- Backup and recovery with appropriate security
- Separation of duties in key management
- Regular key management audits
- Key lifecycle management procedures

**Residual Risk**:
- Insider threats with legitimate key access
- Physical attacks on HSMs or secure enclaves
- Procedural failures in key management
- Complexity of multi-party key management
- Recovery procedures may create additional risks

## Security Controls Mapping

| Threat | Primary Security Controls | Secondary Controls | Monitoring/Detection |
|--------|--------------------------|-------------------|----------------------|
| Key Compromise | HSM/secure enclave, MFA, key rotation | Hybrid PQ, threshold crypto | Key usage anomalies, unauthorized access |
| Algorithm Vulnerabilities | Standard algorithms, audits, formal verification | Algorithm agility, hybrid approach | Security advisories, implementation testing |
| Ciphertext Manipulation | Hash verification, envelope binding, nonces | Event logging, access controls | Hash mismatch alerts, replay detection |
| Config Subversion | Threshold+timelock governance | Event logging, audit trails | Config change monitoring, proposal scrutiny |
| PQ Migration Failures | Staged migration, hybrid envelopes, rollback | Testing, monitoring, communication | Migration progress tracking, failure detection |
| HE/MPC Vulnerabilities | Parameter validation, proof verification | Rate limiting, access controls | Computation verification, anomaly detection |
| Quantum Computing | PQ algorithms, hybrid approach, migration plan | Key rotation, re-encryption | Quantum threat level monitoring |
| Implementation Bugs | Memory safety, testing, audits | Defense in depth, minimal attack surface | Security testing, fuzzing, monitoring |
| Key Management Failures | HSM, automated rotation, multi-party | Policies, audits, separation of duties | Key lifecycle monitoring, usage analytics |

## Recommended Operational Controls

1. **Cryptographic Governance Committee**: Dedicated team for crypto decisions and migration
2. **Regular Cryptographic Audits**: Annual audits by external experts
3. **Quantum Readiness Drills**: Regular testing of migration procedures
4. **Key Management Procedures**: Documented and tested key lifecycle management
5. **Algorithm Monitoring**: Track cryptographic research and potential vulnerabilities
6. **Implementation Security**: Secure coding training, code reviews, fuzzing
7. **Crypto-Agility Testing**: Regular testing of algorithm migration capabilities
8. **Hybrid Deployment Monitoring**: Track hybrid envelope usage and effectiveness
9. **Quantum Threat Intelligence**: Monitor quantum computing advances
10. **Crypto Incident Response**: Specific procedures for cryptographic incidents
11. **Third-Party Dependency Security**: Monitor security of cryptographic libraries
12. **Performance vs. Security Tradeoffs**: Regular review of crypto parameter choices
13. **Backup and Recovery Testing**: Regular testing of key recovery procedures
14. **Compliance with Standards**: Follow NIST, IETF, and other relevant standards
15. **Crypto Training**: Regular training for developers and operators on crypto best practices

## Specific Implementation Strengths

1. **Hybrid Envelope Support**: Allows gradual PQ migration without breaking compatibility
2. **Algorithm Agility**: `CryptoConfigProposal` enables algorithm changes via governance
3. **Quantum Threat Monitoring**: `QuantumThreatLevel` and migration triggers
4. **Key Versioning**: Monotonic versioning in `crypto_registry` enables rotation
5. **Multi-Algorithm Support**: Wide range of classical and PQ algorithms supported
6. **Governance Integration**: Crypto changes require multi-admin approval and timelock
7. **Audit Trail**: Comprehensive crypto audit logging
8. **Parameter Validation**: Key length and algorithm validation in registry
9. **Separation of Concerns**: Different contracts for different crypto functions
10. **Upgrade Path**: Clear migration strategy for post-quantum transition

## Areas for Improvement

1. **Explicit Key Rotation Policies**: Automated rotation schedules and triggers
2. **Enhanced Side-Channel Protection**: Constant-time implementations throughout
3. **Formal Verification**: For critical cryptographic operations
4. **Quantum Migration Automation**: More automated migration procedures
5. **External Key Management Integration**: HSM and cloud KMS integration
6. **Crypto-Agility Testing Framework**: Automated testing of algorithm changes
7. **Post-Quantum Algorithm Maturation**: As PQ algorithms become more standardized
8. **Multi-Party Computation Enhancements**: More robust MPC protocols
9. **Zero-Knowledge Proof Integration**: More extensive ZK proof usage
10. **Hardware Security Integration**: TPM/HSM integration for key operations

The cryptographic threat model requires continuous monitoring and adaptation as new threats emerge and cryptographic research advances. The hybrid approach and governance integration provide flexibility, but require active management and expertise.