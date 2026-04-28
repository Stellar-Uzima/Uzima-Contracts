# Access Control Threat Models

## 1. Unauthorized Record Access

**Threat**: An attacker gains unauthorized access to medical records by bypassing role-based access controls or exploiting permission validation flaws.

**Attack Vectors**:
- Impersonating authorized users (doctors, patients) to access records
- Exploiting flaws in `check_permission()` logic to bypass authorization
- Manipulating caller address in contract calls to appear as authorized user
- Exploiting race conditions in permission granting/revoking
- Bypassing DID verification in `get_record_with_did()`
- Exploiting ZK proof verification flaws in `submit_zk_access_proof()`

**Mitigations**:
- All entry points require `caller.require_auth()` for authentication
- Centralized permission checking via `check_permission()` function
- Role-based access control with explicit permission enumeration
- DID verification required for DID-based access flows
- ZK proof verification with multiple validation checks (commitments, timestamps, nullifiers)
- Event logging for all access attempts (success and failure)
- Forensic logging via `log_to_forensics()` for security monitoring
- Input validation on all parameters before processing

**Residual Risk**:
- Social engineering attacks compromising legitimate user credentials
- Advanced cryptographic attacks on ZK proof systems (if implemented incorrectly)
- Timing attacks on permission checking logic (mitigated by constant-time patterns where possible)

## 2. Privilege Escalation

**Threat**: An attacker with limited privileges gains higher-level access (e.g., patient gaining doctor privileges or unauthorized user gaining admin access).

**Attack Vectors**:
- Exploiting flaws in `manage_user()` or `grant_permission()` functions
- Manipulating role assignment logic to gain unauthorized roles
- Exploiting admin key compromise to assign malicious roles
- Exploiting delegation permission misuse to grant excessive privileges
- Manipulating storage keys to alter role mappings

**Mitigations**:
- `manage_user()` and `grant_permission()` require admin authorization via `require_admin()`
- Delegation permissions require explicit `DelegatePermission` check
- Role changes trigger access attribute epoch updates to invalidate cached permissions
- All role modifications emit events for audit trail
- Storage access restricted to contract functions only (no direct external storage manipulation)
- Multi-signature governance for sensitive admin operations (via governor/timelock contracts)

**Residual Risk**:
- Compromised admin keys could still enable privilege escalation (mitigated by threshold governance)
- Insider threats from malicious administrators (requires organizational controls)

## 3. Emergency Access Abuse

**Threat**: Emergency access mechanisms are exploited to gain unauthorized access to medical records outside legitimate emergency scenarios.

**Attack Vectors**:
- Fraudulent emergency access requests using compromised credentials
- Excessive emergency access scope (accessing more records than necessary)
- Failure to properly expire emergency access grants
- Exploiting emergency access to bypass normal audit trails

**Mitigations**:
- Emergency access requires explicit granting via authorized emergency override mechanisms
- Emergency access grants are time-bound with expiration timestamps
- Emergency access can be scoped to specific record IDs via `record_scope` vector
- All emergency access attempts are logged via standard access logging mechanisms
- Emergency access grants require admin-level authorization to create/modify
- Regular audit of emergency access grants recommended

**Residual Risk**:
- Legitimate emergency access could still be misused by authorized personnel (requires procedural controls and monitoring)
- Time-bound nature limits exposure window but doesn't prevent misuse during valid period

## 4. Permission Granting/Revoking Abuse

**Threat**: Malicious actors exploit permission granting/revoking mechanisms to maintain persistent unauthorized access or lock out legitimate users.

**Attack Vectors**:
- Granting excessive permissions to compromised accounts
- Revoking permissions from legitimate users to cause denial of service
- Exploiting delegation chains to propagate unauthorized permissions widely
- Manipulating permission expiration timestamps for persistent access

**Mitigations**:
- Permission granting requires authorization (either admin or delegate permission)
- Permission revocation requires similar authorization controls
- All permission changes emit events for audit tracking
- Permission grants can be time-bound via expiration parameter
- Delegation flag controls whether permissions can be further delegated
- Regular review of permission grants recommended as operational control

**Residual Risk**:
- Legitimate permission changes could be mistaken for malicious activity (requires contextual monitoring)
- Time-bounded permissions require active management to prevent access disruption

## 5. DID-Based Access Control Bypass

**Threat**: Attackers bypass Decentralized Identifier (DID) based access controls to access records without proper identity verification.

**Attack Vectors**:
- Presenting forged or stolen DID credentials
- Exploiting flaws in DID resolution or verification logic
- Replay attacks on DID authentication flows
- Manipulating DID reference storage to associate wrong DID with user

**Mitigations**:
- DID verification integrated into access control flow (`get_record_with_did()`)
- DID references stored in user profiles and verified during access attempts
- DID authentication levels allow configurable verification strictness
- DID changes trigger access attribute epoch updates to invalidate cached permissions
- Events emitted for DID-related operations (user creation, role updates)

**Residual Risk**:
- DID system security depends on underlying DID method implementation
- DID theft or compromise outside the contract system remains a risk
- Requires secure DID management practices off-chain

## 6. Zero-Knowledge Proof Access Control Bypass

**Threat**: Attackers exploit weaknesses in ZK proof-based access control systems to gain unauthorized access.

**Attack Vectors**:
- Submitting invalid or forged ZK proofs that pass verification
- Replay attacks using previously valid proofs
- Exploiting flaws in commitment verification (record commitment, requester commitment, etc.)
- Manipulating nullifier system to allow proof reuse
- Exploiting timestamp validation flaws in proof inputs

**Mitigations**:
- Multiple validation checks in `submit_zk_access_proof()`:
  - Record ID matching
  - Commitment verification (record, requester, provider)
  - Pseudonym verification
  - Credential root validation and revocation checking
  - Nullifier usage prevention (one-time use)
  - Actual ZK proof verification
- All validation failures trigger ZK audit logging
- Nullifier system prevents proof replay
- Timestamp bounding prevents temporal replay attacks
- Commitment binding ensures proofs are tied to specific records and users

**Residual Risk**:
- Security depends on correctness of ZK proof implementation and underlying cryptography
- Requires trusted setup for ZK proof systems (if applicable)
- Side-channel attacks on proof verification remain possible (requires constant-time implementations)

## 7. Role Confusion and Misassignment

**Threat**: Users are assigned incorrect roles leading to either excessive privileges or insufficient access to perform legitimate functions.

**Attack Vectors**:
- Administrative error in role assignment
- Exploiting interface ambiguities in role assignment functions
- Manipulating role enumeration values (if not properly validated)
- Exploiting storage corruption to alter role mappings

**Mitigations**:
- Strongly typed `Role` enum prevents invalid role values
- Input validation on role parameters in all functions
- Role assignment functions require explicit role specification
- Storage uses contract types ensuring data integrity
- Events emitted for all role changes enabling audit and anomaly detection
- Read-only role query functions prevent accidental modification through query interfaces

**Residual Risk**:
- Administrative errors still possible (requires procedural controls and training)
- Complex role combinations may lead to unintended permission overlaps (requires regular permission reviews)

## 8. Access Control Logic Bypass via Contract Upgrades

**Threat**: Malicious upgrades to access control logic weaken or bypass security mechanisms.

**Attack Vectors**:
- Upgrading medical records contract with weakened access control logic
- Modifying permission checking functions to always return true
- Altering role hierarchy or permission mappings
- Introducing backdoors in access control functions

**Mitigations**:
- Upgradeability pattern uses timelock and threshold governance for contract upgrades
- All contract modifications require governance approval via governor/timelock contracts
- Upgrade delay provides window for community review and objection
- Transparent upgrade process with public proposal and voting mechanisms
- Ability to revert malicious upgrades through same governance process

**Residual Risk**:
- Governance attack could still enable malicious upgrades (requires decentralized and diverse governance)
- Upgrade delay creates vulnerability window during which attacks could be launched
- Requires active community monitoring of upgrade proposals

## 9. Denial of Service via Access Control Exhaustion

**Threat**: Attackers exhaust access control mechanisms to prevent legitimate users from accessing the system.

**Attack Vectors**:
- Excessive permission granting/revoking to fill storage limits
- Creating many DID associations to bloat user profiles
- Generating many emergency access grants to consume resources
- Excessive role assignments to exceed max roles per address limits
- Spamming access requests to trigger rate limits on legitimate users

**Mitigations**:
- Rate limiting on sensitive operations (user management, permission granting)
- Maximum roles per address configurable (default 10)
- Storage efficient data structures (maps rather than lists where appropriate)
- Events emitted rather than storing large data on-chain where possible
- Ability to pause contract during extreme attacks (admin function)
- Regular monitoring of storage usage and growth trends

**Residual Risk**:
- Resource exhaustion attacks may still succeed against rate limits (requires ongoing tuning)
- Legitimate users may be affected by rate limits during attacks (requires careful limit calibration)
- Pause function itself could be targeted (requires secure admin key management)

## Security Controls Mapping

| Threat | Primary Security Controls | Secondary Controls | Monitoring/Detection |
|--------|--------------------------|-------------------|----------------------|
| Unauthorized Record Access | `require_auth()`, `check_permission()`, DID/ZK validation | Event logging, forensic logging | Access logs, failed attempt alerts |
| Privilege Escalation | Admin authorization requirements, delegation controls | Role change events, epoch updates | Role change monitoring, admin action audits |
| Emergency Access Abuse | Time-bounded grants, scoped access, admin authorization | Emergency access logging | Emergency grant reviews, access pattern analysis |
| Permission Granting/Revoking Abuse | Authorization requirements, time-bounded grants, delegation flags | Permission change events | Permission audit trails, anomalous grant detection |
| DID-Based Access Control Bypass | DID verification in access flow, DID references in profiles | DID change events, auth levels | DID verification logs, revocation checking |
| ZK Proof Access Control Bypass | Multi-factor validation, nullifier system, timestamp bounding | ZK audit logging | Proof validation failure rates, nullifier reuse detection |
| Role Confusion and Misassignment | Strongly typed enums, input validation, storage integrity | Role change events | Role assignment audits, permission effectiveness reviews |
| Access Control Logic Bypass via Contract Upgrades | Governance timelock, threshold requirements, transparency | Upgrade delay, revert capability | Proposal monitoring, upgrade announcement watching |
| Denial of Service via Access Control Exhaustion | Rate limits, storage limits, pause function | Storage monitoring, efficient data structures | Resource usage metrics, rate limit alerts |

## Recommended Operational Controls

1. **Regular Access Reviews**: Quarterly review of role assignments and permission grants
2. **Anomaly Detection**: Monitor for unusual access patterns (time of day, frequency, record types)
3. **Emergency Access Audits**: Regular review of emergency grant usage and justification
4. **Permission Hygiene**: Regular cleanup of expired permissions and unused DID associations
5. **Admin Activity Monitoring**: Alert on admin actions outside normal hours or patterns
6. **ZK System Monitoring**: Track proof validation failures and nullifier usage patterns
7. **Upgrade Proposal Review**: Dedicated committee review of all contract upgrade proposals
8. **Incident Response Playbooks**: Specific procedures for access control incidents
9. **Penetration Testing**: Regular testing of access control mechanisms
10. **Security Training**: Regular training for administrators on access control best practices
