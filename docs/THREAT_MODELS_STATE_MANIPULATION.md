# State Manipulation Threat Models

## 1. Medical Record Tampering

**Threat**: Attackers modify existing medical records to alter diagnoses, treatments, or other critical health information.

**Attack Vectors**:
- Direct modification of stored `MedicalRecord` structs in contract storage
- Exploiting flaws in `add_record()` or `add_record_with_did()` to modify existing records
- Manipulating record metadata (`RecordMetadata`) to change categorization or timestamps
- Exploiting versioning mechanisms to roll back to malicious record states
- Tampering with encrypted record references (`ciphertext_ref`) to point to malicious content
- Modifying record hashes (`record_hash`, `ciphertext_hash`) to hide tampering

**Mitigations**:
- Immutable record storage: Once created, records cannot be modified directly
- Append-only design: New records create new entries rather than modifying existing ones
- Cryptographic hashing: `record_hash` and `ciphertext_hash` provide integrity verification
- Version tracking: `RecordMetadata` includes version numbers and history
- Access controls: Record modification functions require proper authorization
- Event logging: All record creation events are emitted for audit trails
- Cross-referencing: Multiple storage keys point to same record data (inconsistency detection)
- Patient record lists: `PatientRecordCount` and `PatientRecord` mappings provide redundancy

**Residual Risk**:
- Social engineering to trick authorized users into creating malicious records
- Key compromise allowing creation of fraudulent records by legitimate actors
- Metadata-only attacks that don't modify core clinical information but affect usability

## 2. Access Control State Corruption

**Threat**: Attackers manipulate role assignments, permissions, or access control state to gain unauthorized access or lock out legitimate users.

**Attack Vectors**:
- Direct modification of user profile storage (`Users` map)
- Tampering with permission grants (`UserPermissions` map)
- Corruption of role-to-member mappings (`get_role_members()` equivalent state)
- Manipulation of DID references in user profiles
- Corruption of access attribute state (`AdvancedAccessState`)
- Tampering with emergency access grants (`PatientEmergencyGrants`)

**Mitigations**:
- All state modifications require authorization via contract functions
- No direct external storage access - all modifications go through validated functions
- Strong typing and contract types prevent arbitrary data injection
- Events emitted for all state-changing operations (role changes, permission grants, etc.)
- Storage redundancy: Multiple ways to verify state (e.g., user roles checkable via multiple paths)
- Consistent hashing: Critical state includes integrity checks where appropriate
- Pause function: Ability to freeze contract state during active attacks
- Upgrade governance: Prevents malicious state-modifying upgrades

**Residual Risk**:
- Logic flaws in authorization checks could allow unauthorized state modifications
- Admin key compromise could enable malicious state changes (mitigated by threshold governance)
- Complex state interactions could create unintended modification pathways

## 3. Cryptographic Configuration Tampering

**Threat**: Attackers modify cryptographic settings to weaken security or disable protections.

**Attack Vectors**:
- Changing `EncryptionRequired` flag to allow plaintext records
- Modifying `RequirePqEnvelopes` to weaken post-quantum protections
- Altering `CryptoRegistry`, `HomomorphicRegistry`, or `MpcManager` addresses
- Manipulating crypto configuration proposals (`CryptoConfigProposal`)
- Tampering with quantum threat level settings (`QuantumThreatLevel`)

**Mitigations**:
- All cryptographic configuration changes require threshold + timelock governance
- `propose_crypto_config_update`, `approve_crypto_config_update`, `execute_crypto_config_update` flow
- Time delay (`TIMELOCK_SECS = 86,400` seconds / 1 day) for community review
- Approval threshold (`APPROVAL_THRESHOLD = 2`) requiring multiple admin agreements
- Event logging for all crypto config proposal lifecycle events
- Crypto audit trail (`CryptoAuditCount`, `CryptoAudit`) tracks all changes
- Ability to detect and revert malicious configurations through same governance process
- Separation of duties: Different admin sets for different crypto functions

**Residual Risk**:
- Governance attack could still enable malicious changes (requires compromising multiple admins)
- Time window creates vulnerability during which attacks could be launched
- Requires active monitoring of crypto governance proposals

## 4. Emergency Access State Manipulation

**Threat**: Attackers create, modify, or extend emergency access grants beyond legitimate needs.

**Attack Vectors**:
- Creating fraudulent emergency access grants for unauthorized users
- Extending expiration times of existing emergency grants
- Expanding record scope to access more patients/records than authorized
- Reactivating expired or revoked emergency access grants
- Manipulating emergency access activation flags (`is_active`)

**Mitigations**:
- Emergency access creation requires admin authorization
- All emergency access grants have explicit expiration timestamps
- Record scope limitation via `record_scope` vector (can be empty for no access)
- Emergency access grants store patient and grantee addresses for accountability
- Activation flag (`is_active`) requires explicit setting
- Events emitted for emergency access grant creation and modification
- Regular audit recommended for emergency access grants
- Time-bound nature limits exposure even if grant is maliciously created

**Residual Risk**:
- Legitimate emergency access grants could be misused by authorized administrators
- Time window between grant creation and discovery could allow access
- Requires procedural controls and monitoring for emergency access usage

## 5. Rate Limit State Corruption

**Threat**: Attackers manipulate rate limiting state to either bypass restrictions or cause denial of service.

**Attack Vectors**:
- Resetting or manipulating `RateLimitEntry` counters to avoid limits
- Corrupting `RateLimitConfig` to set unrealistic limits
- Manipulating `RateLimitBypass` to grant unauthorized exemptions
- Tampering with window timing to create artificial reset opportunities
- Exhausting rate limit storage through excessive entries

**Mitigations**:
- Rate limit modifications require admin authorization via contract functions
- Rate limit entries store window start time to prevent easy reset
- Bypass function requires explicit admin granting per address
- Configuration validation prevents nonsensical values (zero window, etc.)
- Storage efficient: Only active rate limit entries stored
- Events emitted for rate limit configuration changes
- Ability to monitor rate limit effectiveness through metrics
- Separate limits by role type (doctor, patient, admin) prevents cross-role exhaustion

**Residual Risk**:
- Sophisticated timing attacks could potentially exploit window boundaries
- Legitimate users might be affected by rate limits during attack scenarios
- Requires tuning of limits to balance security and usability

## 6. ZK Proof System State Tampering

**Threat**: Attackers manipulate ZK proof system state to bypass validation or disable protections.

**Attack Vectors**:
- Corrupting `ZkEnforced` flag to disable ZK requirements
- Manipulating `ZkGrantTtl` to extend or shorten proof validity windows
- Tampering with `ZkUsedNullifier` set to allow proof replay
- Corrupting credential root storage in `CredentialRegistry`
- Manipulating ZK verifier contract pointers (`ZkVerifierContract`)

**Mitigations**:
- All ZK system modifications require admin authorization
- ZK enforcement flag requires explicit setting
- Grant TTL has maximum limit (`MAX_ZK_GRANT_TTL_SECS = 3,600` / 1 hour)
- Nullifier system prevents proof reuse (one-time use)
- Credential registry has its own authorization controls for root updates
- ZK verifier contract changes require admin authorization
- Events emitted for all ZK system configuration changes
- Multiple validation checks in proof submission reduce reliance on single state variables
- Ability to detect tampering through proof validation failures

**Residual Risk**:
- Zero-day vulnerabilities in ZK proof implementation could bypass validation
- Requires ongoing monitoring of ZK research and potential cryptographic advances
- Side-channel attacks on proof verification remain possible

## 7. Cross-Chain State Manipulation

**Threat**: Attackers manipulate cross-chain bridge or synchronization state to enable theft or fraud.

**Attack Vectors**:
- Corrupting cross-chain contract addresses (`BridgeContract`, `CrossChainIdentityContract`, etc.)
- Manipulating cross-chain enable flag (`CrossChainEnabled`)
- Tampering with cross-chain record references (`CrossChainRef`)
- Exploiting sync timestamp manipulation to create replay opportunities
- Manipulating external chain identifiers to route assets incorrectly

**Mitigations**:
- Cross-chain contract address changes require admin authorization
- Cross-chain enable flag requires explicit setting
- Cross-chain references include external chain ID and external record hash for verification
- Sync timestamps help prevent replay attacks
- Chain ID validation prevents routing to unintended chains
- Events emitted for cross-chain configuration changes
- Ability to verify cross-chain state through external chain observers
- Time delays and confirmation requirements in cross-chain transfers

**Residual Risk**:
- Security depends on weakest link in connected chains
- Requires monitoring of all connected chains for security issues
- Cross-chain complexity increases attack surface

## 8. Governance State Tampering

**Threat**: Attackers manipulate governor or timelock contract state to enable malicious upgrades or parameter changes.

**Attack Vectors**:
- Corrupting governor configuration (`GovernorConfig`)
- Manipulating proposal state to fake approval or execution
- Tampering with voting power calculations (`get_power()` equivalent)
- Corrupting timelock address or delay settings
- Manipulating token or reputation contract pointers

**Mitigations**:
- Governor state modifications require explicit contract function calls
- Proposal lifecycle has multiple states with validation at each transition
- Voting power derived from external tokens with their own security
- Timelock parameters require governance to change (self-reinforcing)
- Events emitted for all governor actions (propose, vote, queue, execute)
- Transparent proposal details allow community verification
- Ability to challenge malicious proposals through dispute mechanisms
- Multi-signature requirements for sensitive governor functions

**Residual Risk**:
- Governance attack could still succeed if attacker controls sufficient voting power
- Requires decentralized and diverse token distribution to prevent concentration
- Complex governance interactions could create unintended pathways

## 9. Storage Key Corruption

**Threat**: Attackers manipulate storage key enumeration or metadata to cause confusion or enable selective state access.

**Attack Vectors**:
- Corrupting `DataKey` enum values through storage manipulation
- Creating alias storage keys that map to same data
- Manipulating storage layout to create hidden or shadow state
- Exploiting storage iteration order dependencies
- Corrupting storage version or initialization flags

**Mitigations**:
- Storage keys are strongly typed enum values preventing arbitrary keys
- Contract types ensure data integrity when reading/writing
- No direct external storage enumeration - all access through known keys
- Storage layout is implicit in contract code, not stored externally
- Initialization and version flags have specific expected values
- Events emitted rather than relying on storage enumeration for auditing
- Ability to verify storage consistency through multiple access paths
- Upgrade process includes storage migration validation

**Residual Risk**:
- Lower-level storage exploits could potentially bypass contract-level protections
- Requires secure execution environment (Soroban VM integrity)
- Storage layout attacks would require sophisticated blockchain exploitation

## Security Controls Mapping

| Threat | Primary Security Controls | Secondary Controls | Monitoring/Detection |
|--------|--------------------------|-------------------|----------------------|
| Medical Record Tampering | Immutability, cryptographic hashing, versioning | Access controls, event logging | Hash verification, version anomaly detection |
| Access Control State Corruption | Authorization requirements, storage integrity | Events, redundancy | Role/permission audit trails, consistency checks |
| Cryptographic Configuration Tampering | Threshold + timelock governance | Event logging, audit trail | Proposal monitoring, config change alerts |
| Emergency Access State Manipulation | Admin authorization, time-bounding, scoping | Events, accountability | Emergency grant reviews, usage analysis |
| Rate Limit State Corruption | Admin authorization, validation, storage efficiency | Events, metrics | Limit effectiveness monitoring, bypass audits |
| ZK Proof System State Tampering | Admin authorization, validation, anti-replay | Events, multi-factor validation | Proof failure rates, nullifier monitoring |
| Cross-Chain State Manipulation | Admin authorization, verification, validation | Events, external verification | Cross-chain anomaly detection, chain monitoring |
| Governance State Tampering | Authorization, multi-state validation, transparency | Events, dispute mechanisms | Proposal scrutiny, voting pattern analysis |
| Storage Key Corruption | Strong typing, contract types, implicit layout | Events, consistency checks | Storage audits, migration validation |

## Recommended Operational Controls

1. **State Consistency Checks**: Regular verification of redundant state mappings (e.g., user roles checkable multiple ways)
2. **Cryptographic Governance Monitoring**: Dedicated watchdog for crypto config proposals and execution
3. **Emergency Access Review Board**: Regular review of all emergency grants with justification requirements
4. **Rate Limit Tuning**: Ongoing adjustment of limits based on observed usage patterns and attack attempts
5. **ZK System Health Monitoring**: Tracking of proof validation performance and failure patterns
6. **Cross-Chain Transaction Monitoring**: Verification of cross-chain transfers against source chain events
7. **Governance Proposal Analysis**: Detailed review of all proposals before voting, including simulation of effects
8. **Storage Audits**: Periodic verification of storage integrity through multiple access paths
9. **Immutability Verification**: Regular checks that historical records cannot be altered
10. **Admin Action Logging**: Comprehensive logging of all administrative actions with alerting on anomalies
11. **Red Team Exercises**: Regular simulated attacks targeting state manipulation vectors
12. **Formal Verification**: Where possible, formally verify critical state transition properties
13. **Upgrade Readiness**: Maintain ability to quickly respond to detected state corruption through governance
14. **Cross-Chain Oracles**: Use trusted oracles to verify cross-chain state when possible
15. **Zero-Knowledge Audits**: Regular third-party review of ZK proof implementations and parameters
