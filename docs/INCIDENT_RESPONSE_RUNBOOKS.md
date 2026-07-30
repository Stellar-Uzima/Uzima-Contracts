# Incident Response Runbooks

Formal incident response procedures for suspected contract compromise in the Uzima-Contracts portfolio.

## Severity Classification

| Level | Description | Response Time | Example |
|-------|-------------|---------------|---------|
| P0 - Critical | Active exploit, funds at risk | Immediate | Reentrancy attack, admin key compromise |
| P1 - High | Confirmed vulnerability, no active exploit | 4 hours | Logic bug allowing unauthorized access |
| P2 - Medium | Suspicious activity detected | 24 hours | Unusual transaction patterns |
| P3 - Low | Minor issue, no immediate risk | 72 hours | Informational finding |

## Runbook 1: Active Exploit Response (P0)

### Detection Indicators
- Unexpected large-value transfers
- Admin functions called from unknown addresses
- Storage manipulation detected
- Cross-chain bridge anomalies

### Immediate Actions

```
1. ASSESS
   - Identify which contract is affected
   - Determine if funds are currently at risk
   - Check if exploit is ongoing

2. CONTAIN
   - If emergency_access_override is available, pause affected contracts
   - Contact team leads via secure channel
   - Preserve all transaction hashes and timestamps

3. DOCUMENT
   - Record all affected addresses and transactions
   - Screenshot any relevant block explorer data
   - Note the exact time of first suspicious activity

4. COMMUNICATE
   - Notify stakeholders via pre-established secure channel
   - Prepare initial incident summary
```

### Escalation Path
1. On-call engineer
2. Security lead
3. Protocol maintainer
4. External auditors (if needed)

## Runbook 2: Unauthorized Admin Access (P1)

### Detection Indicators
- Admin functions called from unexpected addresses
- Configuration changes not initiated by team
- Permission escalation attempts
- Role assignment anomalies

### Response Procedure

```
1. VERIFY
   - Confirm the caller address is not an authorized admin
   - Check governance contract for recent role changes
   - Review timelock queue for pending operations

2. ISOLATE
   - Revoke compromised admin keys if possible
   - Use timelock to delay any pending changes
   - Document all unauthorized operations

3. REMEDIATE
   - Initiate emergency access override if needed
   - Rotate admin keys through governance process
   - Update access control lists

4. RECOVER
   - Verify all contract state is consistent
   - Run integrity checks on affected storage
   - Resume normal operations
```

### Key Contacts
- Admin key holders (list in governance contract)
- Timelock operator
- Security response team

## Runbook 3: Suspicious Transaction Patterns (P2)

### Detection Indicators
- Unusual gas consumption patterns
- High frequency of failed transactions
- Access attempts from new addresses
- Rate limit violations

### Investigation Procedure

```
1. COLLECT DATA
   - Pull transaction history from block explorer
   - Review contract_monitoring telemetry events
   - Check security_telemetry alerts

2. ANALYZE
   - Map transaction patterns over time
   - Identify common source addresses
   - Correlate with known threat patterns

3. ASSESS
   - Determine if activity is malicious or benign
   - Evaluate potential impact if attack succeeds
   - Rate risk level (P2/P3)

4. RESPOND
   - If malicious: escalate to P1 procedure
   - If benign: document and monitor
   - Update detection rules if needed
```

### Monitoring Commands
```bash
# Check recent transactions for a contract
stellar contract invoke --id <CONTRACT_ID> -- get_dashboard

# Review security telemetry
stellar contract invoke --id <MONITORING_CONTRACT> -- get_security_snapshot

# Check specific address failure count
stellar contract invoke --id <MONITORING_CONTRACT> -- \
  get_address_failure_count --address <ADDRESS> --event_type <TYPE>
```

## Runbook 4: Key Compromise Recovery (P0)

### Detection Indicators
- Unexpected signature verification failures
- Admin operations from unknown keys
- Key material exposed in logs or repos

### Recovery Procedure

```
1. IMMEDIATE CONTAINMENT
   - Assume all associated keys are compromised
   - Revoke all active sessions/tokens
   - Pause affected contracts if possible

2. KEY ROTATION
   - Generate new key pairs on secure hardware
   - Update governance contract with new admin addresses
   - Multi-sig approval required for key changes

3. STATE VERIFICATION
   - Compare current state with last known good snapshot
   - Check all storage modifications since compromise
   - Verify no backdoors were introduced

4. POST-INCIDENT
   - Conduct full security audit
   - Update key management procedures
   - Document lessons learned
```

## Runbook 5: Data Integrity Incident (P1)

### Detection Indicators
- Hash mismatches in deployment verification
- Unexpected storage layout changes
- ABI incompatibilities detected

### Response Procedure

```
1. IDENTIFY
   - Run deployment hash verification
   - Compare current WASM with audited version
   - Check for unauthorized contract upgrades

2. ASSESS IMPACT
   - Determine which records/data may be affected
   - Check if patient data integrity is compromised
   - Evaluate regulatory implications

3. REMEDIATE
   - Roll back to last known good deployment if needed
   - Restore data from verified backups
   - Update verification checksums

4. VERIFY
   - Re-run all verification scripts
   - Confirm data consistency
   - Document findings for compliance
```

## Runbook 6: Cross-Chain Bridge Incident (P0)

### Detection Indicators
- Replay attack detected
- Nonce sequence anomalies
- Message verification failures
- Unexpected bridge transaction volumes

### Response Procedure

```
1. PAUSE BRIDGE
   - Disable cross-chain message processing
   - Preserve all pending messages for analysis
   - Notify counterpart chains

2. INVESTIGATE
   - Review replay_protection contract state
   - Analyze message signatures and nonces
   - Check for message tampering

3. RECOVER
   - Resolve any stuck transactions
   - Verify nonce sequences are consistent
   - Resume bridge operations with enhanced monitoring

4. HARDEN
   - Update replay protection parameters
   - Add additional verification checks
   - Document attack vectors discovered
```

## Communication Templates

### Initial Incident Notification
```
[INCIDENT - P{SEVERITY}] {CONTRACT_NAME} Suspected Compromise

Time: {TIMESTAMP_UTC}
Status: Investigating
Affected: {CONTRACT_ADDRESSES}
Impact: {DESCRIPTION}

Next update in {TIMEFRAME}.
```

### Resolution Notification
```
[RESOLVED - P{SEVERITY}] {CONTRACT_NAME} Incident

Time Resolved: {TIMESTAMP_UTC}
Duration: {DURATION}
Root Cause: {DESCRIPTION}
Actions Taken: {LIST}

Post-mortem scheduled for {DATE}.
```

## Post-Incident Procedure

1. **Timeline Reconstruction**: Document every event from detection to resolution
2. **Root Cause Analysis**: Identify the fundamental cause, not just symptoms
3. **Action Items**: Create tracked issues for all remediation steps
4. **Process Updates**: Update runbooks based on lessons learned
5. **Team Review**: Conduct blameless post-mortem within 48 hours

## Appendix: Emergency Contacts

| Role | Contact Method | Backup |
|------|---------------|--------|
| On-call Engineer | Secure messaging | Rotation schedule |
| Security Lead | Direct line | Deputy |
| Protocol Maintainer | GitHub + Secure channel | Co-maintainer |
| External Auditors | Email | Engagement contract |

## Appendix: Verification Commands

```bash
# Verify deployment integrity
./scripts/verify_release_artifacts.sh v<VERSION> <NETWORK>

# Check contract health
make health-check

# Run security scan
./scripts/security-scan.sh

# Review audit logs
./scripts/verify_audit_root.py
```
