# Emergency Playbooks

## Overview

This document contains detailed emergency playbooks for the Uzima Contract Governance System. These playbooks provide step-by-step procedures for responding to various emergency situations, including smart contract exploits, key compromises, governance attacks, and market crashes.

## Table of Contents

1. [Smart Contract Exploit Playbook](#smart-contract-exploit-playbook)
2. [Key Compromise Playbook](#key-compromise-playbook)
3. [Governance Attack Playbook](#governance-attack-playbook)
4. [Market Crash Playbook](#market-crash-playbook)
5. [Emergency Response Team Procedures](#emergency-response-team-procedures)
6. [Communication Protocols](#communication-protocols)

---

## Smart Contract Exploit Playbook

### Severity Level: CRITICAL

#### Scenario
A vulnerability in a smart contract has been discovered or is being actively exploited, potentially resulting in loss of funds or data.

### Phase 1: Immediate Response (0-1 hour)

#### Actions Required

- [ ] **ERT Declaration**: 3/5 Emergency Response Team members sign emergency declaration
- [ ] **Emergency Pause**: Activate emergency pause on affected contracts
- [ ] **Stakeholder Notification**: Notify all stakeholders via all channels
- [ ] **Incident Channel**: Create dedicated incident response channel
- [ ] **War Room**: Establish virtual war room for coordination

#### Emergency Declaration Template

```
EMERGENCY DECLARATION
=====================
Type: Smart Contract Exploit
Contract: [Contract Address]
Severity: CRITICAL
Time: [Timestamp]

Description:
[Detailed description of the exploit]

Evidence:
[IPFS hash of evidence]

ERT Signatures:
1. [Name] - [Signature] - [Timestamp]
2. [Name] - [Signature] - [Timestamp]
3. [Name] - [Signature] - [Timestamp]

Next Steps:
1. Emergency pause activated
2. Assessment phase initiated
3. Community notification sent
```

#### Emergency Pause Activation

```rust
// Call emergency pause function
pub fn emergency_pause(
    env: Env,
    reason: String,
    evidence_hash: Bytes,
    signatures: Vec<Signature>,
) -> Result<(), Error> {
    // Verify 3/5 ERT signatures
    verify_ert_signatures(signatures)?;
    
    // Activate pause
    state.emergency_paused = true;
    state.pause_reason = reason;
    state.pause_evidence = evidence_hash;
    state.pause_time = now();
    
    // Emit event
    emit_event(EventType::EmergencyPause, reason);
    
    Ok(())
}
```

### Phase 2: Assessment Phase (1-4 hours)

#### Actions Required

- [ ] **Vulnerability Identification**: Determine exact vulnerability type
- [ ] **Impact Assessment**: Calculate funds at risk and affected users
- [ ] **Scope Determination**: List all affected contracts and systems
- [ ] **Evidence Collection**: Gather all relevant logs and data
- [ ] **Communication**: Update stakeholders with initial findings

#### Vulnerability Types

| Type | Description | Response |
|------|-------------|----------|
| Reentrancy | Recursive calls drain funds | Immediate pause, audit all contracts |
| Integer Overflow | Arithmetic errors | Patch and upgrade |
| Access Control | Unauthorized access | Revoke permissions, patch |
| Logic Error | Flawed business logic | Patch and upgrade |
| Oracle Manipulation | Price feed tampering | Switch to backup oracles |

#### Impact Assessment Template

```
IMPACT ASSESSMENT
=================
Contract: [Contract Address]
Vulnerability: [Type]
Time Discovered: [Timestamp]

Funds at Risk:
- Total Value Locked: $[amount]
- Potentially Affected: $[amount]
- Confirmed Loss: $[amount]

Affected Users:
- Total Users: [count]
- High Value Accounts: [list]
- New Users (last 24h): [count]

Systems Affected:
- Primary Contract: [address]
- Dependent Contracts: [list]
- Oracles: [list]
- Frontend: [status]

Recommended Actions:
1. [Action 1]
2. [Action 2]
3. [Action 3]
```

### Phase 3: Mitigation Phase (4-12 hours)

#### Actions Required

- [ ] **Patch Development**: Develop fix for vulnerability
- [ ] **Staging Test**: Test patch in staging environment
- [ ] **Emergency Proposal**: Submit emergency upgrade proposal
- [ ] **Community Briefing**: Brief community on situation
- [ ] **Validator Coordination**: Coordinate with validators

#### Patch Development Checklist

- [ ] Root cause identified and documented
- [ ] Fix implemented in code
- [ ] Unit tests written and passing
- [ ] Integration tests updated
- [ ] Security review completed
- [ ] Code review by 2+ developers
- [ ] Staging deployment tested
- [ ] Rollback plan prepared

#### Emergency Proposal Template

```
EMERGENCY UPGRADE PROPOSAL
==========================
Proposal ID: [auto-generated]
Type: Emergency Upgrade
Contract: [Contract Address]
Severity: CRITICAL

Description:
Emergency upgrade to patch [vulnerability type] vulnerability
discovered in [contract name].

Technical Details:
- Vulnerability: [detailed description]
- Fix: [description of fix]
- WASM Hash: [new hash]
- Version: [new version]

Impact:
- Fixes critical security vulnerability
- No breaking changes to API
- Minimal gas cost increase

Testing:
- Unit tests: [number] passing
- Integration tests: [number] passing
- Security audit: [status]
- Staging deployment: [status]

Timeline:
- Discovery: [timestamp]
- Patch developed: [timestamp]
- Testing complete: [timestamp]
- Proposed upgrade: [timestamp]

Signatures:
- Lead Developer: [signature]
- Security Lead: [signature]
- ERT Representative: [signature]
```

### Phase 4: Recovery Phase (12-24 hours)

#### Actions Required

- [ ] **Emergency Upgrade**: Execute emergency upgrade
- [ ] **Verification**: Verify vulnerability is fixed
- [ ] **Operations Resume**: Resume normal operations
- [ ] **Post-Mortem**: Initiate post-mortem analysis
- [ ] **Stakeholder Update**: Final update to stakeholders

#### Upgrade Execution

```rust
// Execute emergency upgrade
pub fn execute_emergency_upgrade(
    env: Env,
    upgrade_id: u64,
) -> Result<(), Error> {
    // Verify emergency status
    let emergency = get_emergency(upgrade_id);
    if emergency.status != EmergencyStatus::Approved {
        return Err(Error::NotApproved);
    }
    
    // Execute upgrade
    upgrade_contract(
        emergency.contract_id,
        emergency.new_wasm_hash,
    )?;
    
    // Verify upgrade
    verify_upgrade_success(upgrade_id)?;
    
    // Deactivate emergency pause
    deactivate_emergency_pause();
    
    // Update status
    emergency.status = EmergencyStatus::Resolved;
    emergency.resolved_at = now();
    
    Ok(())
}
```

### Phase 5: Post-Incident (1-7 days)

#### Actions Required

- [ ] **Root Cause Analysis**: Complete detailed RCA
- [ ] **Full Report**: Publish comprehensive incident report
- [ ] **Compensation Plan**: Develop plan for affected users (if needed)
- [ ] **Security Improvements**: Implement preventive measures
- [ ] **Process Updates**: Update procedures and documentation

#### Post-Mortem Template

```
POST-MORTEM REPORT
==================
Incident: [Incident ID]
Date: [Date]
Duration: [Start] to [End]
Severity: [Level]

Summary:
[Executive summary of incident]

Timeline:
- [Time] - [Event]
- [Time] - [Event]
- [Time] - [Event]

Root Cause:
[Detailed root cause analysis]

Impact:
- Users affected: [count]
- Funds lost: $[amount]
- Funds recovered: $[amount]
- Reputation impact: [assessment]

Response:
- Detection time: [time]
- Response time: [time]
- Resolution time: [time]
- Communication: [assessment]

Lessons Learned:
1. [Lesson 1]
2. [Lesson 2]
3. [Lesson 3]

Action Items:
- [ ] [Action 1] - Owner: [name] - Due: [date]
- [ ] [Action 2] - Owner: [name] - Due: [date]
- [ ] [Action 3] - Owner: [name] - Due: [date]
```

---

## Key Compromise Playbook

### Severity Level: HIGH

#### Scenario
Private keys or credentials have been compromised, potentially allowing unauthorized access to contracts or funds.

### Phase 1: Immediate Response (0-30 minutes)

#### Actions Required

- [ ] **Identify Compromise**: Determine which keys are compromised
- [ ] **Revoke Keys**: Immediately revoke compromised keys
- [ ] **Rotate Keys**: Generate and deploy new keys
- [ ] **Increase Security**: Enhance multi-sig requirements
- [ ] **Emergency Session**: Call emergency response session

#### Key Revocation Process

```rust
pub fn revoke_compromised_key(
    env: Env,
    key_id: String,
    reason: String,
    signatures: Vec<Signature>,
) -> Result<(), Error> {
    // Verify authorization (3/5 signatures)
    verify_signatures(signatures)?;
    
    // Revoke key
    let mut key = get_key(key_id);
    key.revoked = true;
    key.revoked_at = now();
    key.revocation_reason = reason;
    
    // Disable key usage
    disable_key(key_id);
    
    // Emit event
    emit_event(EventType::KeyRevoked, key_id);
    
    Ok(())
}
```

### Phase 2: Assessment Phase (30 minutes - 2 hours)

#### Actions Required

- [ ] **Access Log Review**: Review all access logs
- [ ] **Unauthorized Actions**: Identify any unauthorized transactions
- [ ] **Affected Systems**: List all potentially affected systems
- [ ] **Impact Assessment**: Determine full scope of compromise
- [ ] **Stakeholder Notification**: Notify affected parties

#### Access Log Analysis

```
ACCESS LOG REVIEW CHECKLIST
===========================
- [ ] Review authentication logs (last 90 days)
- [ ] Check for unusual login patterns
- [ ] Identify unauthorized transactions
- [ ] Review contract interactions
- [ ] Check for data exfiltration
- [ ] Analyze IP addresses and geolocation
- [ ] Review failed authentication attempts
- [ ] Check for privilege escalation
```

### Phase 3: Recovery Phase (2-8 hours)

#### Actions Required

- [ ] **Generate New Keys**: Create new cryptographic keys (HSM)
- [ ] **Update Access Controls**: Deploy updated key configuration
- [ ] **Verify Security**: Confirm systems are secure
- [ ] **Resume Operations**: Restore normal operations
- [ ] **Enhanced Monitoring**: Enable additional monitoring

#### Key Generation Process

```bash
# Generate new keys in HSM
aws cloudhsm create-key \
  --label "emergency-replacement-key" \
  --type "RSA_2048" \
  --usage "SIGN_VERIFY" \
  --hsm-type "hsm1.medium"

# Deploy to contracts
soroban invoke \
  --id $CONTRACT_ID \
  --fn update_key \
  --arg $NEW_KEY_ID \
  --arg $NEW_KEY_DATA
```

### Phase 4: Post-Incident (1-7 days)

#### Actions Required

- [ ] **Incident Report**: Publish detailed incident report
- [ ] **Security Audit**: Conduct comprehensive security audit
- [ ] **Key Management**: Improve key management procedures
- [ ] **Training Update**: Update security training
- [ ] **Procedure Enhancement**: Enhance security procedures

---

## Governance Attack Playbook

### Severity Level: HIGH

#### Scenario
Attack on governance system, including vote buying, Sybil attacks, proposal spam, or manipulation.

### Detection Phase

#### Indicators

- [ ] Unusual voting patterns detected
- [ ] Multiple proposals from new accounts
- [ ] Vote buying offers observed
- [ ] Sybil attack suspected
- [ ] Proposal spam detected

#### Monitoring Tools

```rust
pub fn detect_governance_attack(
    env: Env,
) -> Result<Vec<AttackIndicator>, Error> {
    let indicators = vec![];
    
    // Check for unusual voting patterns
    if detect_vote_buying() {
        indicators.push(AttackIndicator::VoteBuying);
    }
    
    // Check for Sybil patterns
    if detect_sybil_attack() {
        indicators.push(AttackIndicator::SybilAttack);
    }
    
    // Check for proposal spam
    if detect_proposal_spam() {
        indicators.push(AttackIndicator::ProposalSpam);
    }
    
    Ok(indicators)
}
```

### Response Phase

#### Actions Required

- [ ] **Emergency Proposal**: Submit emergency pause proposal
- [ ] **Voting Freeze**: Freeze voting if authorized
- [ ] **Investigation**: Launch investigation
- [ ] **Community Notification**: Inform community
- [ ] **Evidence Collection**: Gather evidence

#### Emergency Pause for Governance

```rust
pub fn pause_governance(
    env: Env,
    reason: String,
    evidence_hash: Bytes,
) -> Result<(), Error> {
    // Verify authorization
    verify_ert_signatures()?;
    
    // Pause governance functions
    state.governance_paused = true;
    state.governance_pause_reason = reason;
    
    // Emit event
    emit_event(EventType::GovernancePaused, reason);
    
    Ok(())
}
```

### Resolution Phase

#### Actions Required

- [ ] **Attack Vector Identification**: Identify how attack occurred
- [ ] **Countermeasures**: Deploy countermeasures
- [ ] **Governance Adjustment**: Adjust governance parameters
- [ ] **Voting Resumption**: Resume normal voting
- [ ] **Attacker Identification**: Identify attackers (if possible)

#### Countermeasures

```rust
pub fn deploy_countermeasures(
    env: Env,
    countermeasure_type: u8,
) -> Result<(), Error> {
    match countermeasure_type {
        // Rate limiting
        0 => enable_rate_limiting(),
        // Minimum stake increase
        1 => increase_minimum_stake(),
        // Delegation limits
        2 => enable_delegation_limits(),
        // Reputation requirements
        3 => enable_reputation_requirements(),
        _ => return Err(Error::InvalidCountermeasure),
    }
    
    Ok(())
}
```

### Prevention Phase

#### Actions Required

- [ ] **Anti-Sybil Enhancement**: Enhance Sybil resistance
- [ ] **Voting Improvement**: Improve voting mechanisms
- [ ] **Monitoring Strengthening**: Strengthen monitoring
- [ ] **Community Education**: Educate community
- [ ] **Policy Updates**: Update governance policies

---

## Market Crash Playbook

### Severity Level: MEDIUM-HIGH

#### Scenario
Significant market downturn affecting protocol stability, liquidity, or user funds.

### Monitoring Phase

#### Indicators

- [ ] Price feeds show extreme volatility
- [ ] Circuit breakers triggered
- [ ] Liquidity pools drained
- [ ] User withdrawals spike
- [ ] Protocol metrics degraded

#### Monitoring Setup

```rust
pub fn monitor_market_conditions(
    env: Env,
) -> Result<MarketStatus, Error> {
    let price_feed = get_price_feed();
    let volatility = calculate_volatility(price_feed);
    let liquidity = get_liquidity_pools();
    
    let status = if volatility > VOLATILITY_THRESHOLD {
        MarketStatus::HighVolatility
    } else if liquidity < LIQUIDITY_THRESHOLD {
        MarketStatus::LowLiquidity
    } else {
        MarketStatus::Normal
    };
    
    Ok(status)
}
```

### Response Phase (if triggered)

#### Actions Required

- [ ] **Automatic Pause**: Activate automatic pause (if triggered)
- [ ] **Emergency Session**: Call emergency response session
- [ ] **Treasury Assessment**: Assess treasury exposure
- [ ] **Stakeholder Notification**: Notify stakeholders
- [ ] **Strategy Review**: Review risk management strategy

#### Automatic Pause Activation

```rust
pub fn check_circuit_breaker(
    env: Env,
) -> Result<(), Error> {
    let volatility = get_current_volatility();
    
    if volatility > CIRCUIT_BREAKER_THRESHOLD {
        // Activate automatic pause
        activate_automatic_pause();
        
        // Notify stakeholders
        notify_circuit_breaker_activation();
        
        // Start emergency session
        start_emergency_session();
    }
    
    Ok(())
}
```

### Stabilization Phase

#### Actions Required

- [ ] **Liquidity Provision**: Deploy liquidity provisions
- [ ] **Risk Adjustment**: Adjust risk parameters
- [ ] **Gradual Resumption**: Gradually resume operations
- [ ] **Market Making**: Activate market making
- [ ] **Volatility Monitoring**: Enhanced volatility monitoring

#### Liquidity Provision

```rust
pub fn deploy_liquidity(
    env: Env,
    pool_id: Address,
    amount: i128,
) -> Result<(), Error> {
    // Verify authorization
    verify_treasury_authorization()?;
    
    // Deploy liquidity
    transfer_tokens(pool_id, amount)?;
    
    // Update pool parameters
    update_pool_parameters(pool_id);
    
    // Emit event
    emit_event(EventType::LiquidityDeployed, amount);
    
    Ok(())
}
```

### Recovery Phase

#### Actions Required

- [ ] **Market Normalization**: Wait for market conditions to normalize
- [ ] **Parameter Reset**: Reset risk parameters to normal
- [ ] **Lessons Learned**: Document lessons learned
- [ ] **Stress Tests**: Conduct stress tests
- [ ] **Model Updates**: Update risk models

---

## Emergency Response Team Procedures

### ERT Composition

- **5 Core Members**: Rotating membership
- **Backup Members**: 3 additional members
- **Specialized Roles**: Technical, Legal, Communications

### ERT Activation

#### Activation Thresholds

| Severity | Required Signatures | Response Time |
|----------|-------------------|---------------|
| Level 1 | 1/5 | 24 hours |
| Level 2 | 2/5 | 12 hours |
| Level 3 | 3/5 | 4 hours |
| Level 4 | 4/5 | 1 hour |

### ERT Decision Making

```rust
pub fn make_ert_decision(
    env: Env,
    decision_type: u8,
    signatures: Vec<Signature>,
) -> Result<(), Error> {
    // Count valid signatures
    let valid_signatures = verify_signatures(signatures)?;
    
    // Check threshold based on decision type
    let threshold = match decision_type {
        DecisionType::EmergencyPause => 3,
        DecisionType::EmergencyUpgrade => 3,
        DecisionType::KeyRotation => 2,
        DecisionType::TreasuryAccess => 3,
        _ => return Err(Error::InvalidDecisionType),
    };
    
    if valid_signatures < threshold {
        return Err(Error::InsufficientSignatures);
    }
    
    // Execute decision
    execute_ert_decision(decision_type);
    
    Ok(())
}
```

---

## Communication Protocols

### Notification Channels

#### Immediate Notifications (Level 3-4)

- Discord emergency channel
- Email to all stakeholders
- Twitter/X announcement
- Telegram broadcast
- SMS alerts (critical)

#### Standard Notifications (Level 1-2)

- Discord announcement
- Email to subscribers
- Governance forum post

### Communication Templates

#### Emergency Notification

```
🚨 EMERGENCY ALERT 🚨

Type: [Emergency Type]
Severity: [Level]
Time: [Timestamp]

Description:
[Brief description]

Impact:
[Impact assessment]

Actions Taken:
- [Action 1]
- [Action 2]
- [Action 3]

Next Steps:
- [Next step 1]
- [Next step 2]

Updates:
- Discord: [link]
- Status Page: [link]
- Forum: [link]

Emergency Response Team
```

#### Status Update

```
📊 STATUS UPDATE

Incident: [Incident ID]
Time: [Timestamp]
Status: [In Progress/Resolved]

Current Status:
[Detailed status update]

Actions Completed:
- [Action 1] ✅
- [Action 2] ✅
- [Action 3] 🔄

Next Actions:
- [Next action 1]
- [Next action 2]

ETA: [Estimated time]

Emergency Response Team
```

#### Resolution Notification

```
✅ INCIDENT RESOLVED

Incident: [Incident ID]
Resolved: [Timestamp]
Duration: [Duration]

Summary:
[Summary of incident and resolution]

Impact:
- Users affected: [count]
- Funds lost: $[amount]
- Funds recovered: $[amount]

Actions Taken:
- [Action 1]
- [Action 2]
- [Action 3]

Post-Incident:
- Post-mortem: [link]
- Report: [link]
- Compensation: [details if applicable]

Emergency Response Team
```

---

## Appendices

### Appendix A: Emergency Contacts

| Role | Primary | Backup | Contact |
|------|---------|---------|---------|
| ERT Lead | [Name] | [Name] | [Contact] |
| Technical Lead | [Name] | [Name] | [Contact] |
| Legal Counsel | [Name] | [Name] | [Contact] |
| Communications | [Name] | [Name] | [Contact] |
| Treasury | [Name] | [Name] | [Contact] |

### Appendix B: Emergency Tools

- Emergency Pause Contract: [Address]
- Emergency Multisig: [Address]
- Status Page: [URL]
- Incident Channel: [URL]
- War Room: [URL]

### Appendix C: Checklists

- [ ] Emergency Declaration Checklist
- [ ] Assessment Phase Checklist
- [ ] Mitigation Phase Checklist
- [ ] Recovery Phase Checklist
- [ ] Post-Incident Checklist

---

*Document Status: Active*
*Last Updated: 2026-04-25*
*Next Review: 2026-07-25*