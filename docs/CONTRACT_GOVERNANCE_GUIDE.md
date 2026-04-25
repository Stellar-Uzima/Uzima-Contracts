# Uzima Contract Governance Guide

## Overview

This document provides comprehensive governance models for contract decisions within the Uzima healthcare blockchain ecosystem. It covers decision-making processes, upgrade procedures, emergency protocols, community participation mechanisms, and risk management frameworks.

## Table of Contents

1. [Decision-Making Processes](#decision-making-processes)
2. [Upgrade Procedures](#upgrade-procedures)
3. [Emergency Protocols](#emergency-protocols)
4. [Community Participation](#community-participation)
5. [Risk Management Framework](#risk-management-framework)
6. [Decision Procedures](#decision-procedures)
7. [Emergency Playbooks](#emergency-playbooks)

---

## Decision-Making Processes

### 1.1 Governance Architecture

The Uzima DAO operates on a **hybrid governance model** designed to balance financial stake with active contribution, safeguarding the protocol through a judicial layer.

#### Core Components

- **Governor Contract**: The central logic handler. Calculates voting power, manages proposal lifecycles, and executes passed proposals.
- **SUT Token (Plutocratic Layer)**: Represents financial stake. 1 Token = 1 Vote.
- **Reputation System (Meritocratic Layer)**: A non-transferable score earned by contributors. 1 Reputation Point = 1 Vote.
- **Dispute Resolution (Judicial Layer)**: A council of arbiters who can veto malicious proposals that pass the vote but violate the DAO Constitution.

### 1.2 Proposal Lifecycle

#### Phase 1: Proposal Submission

**Threshold**: A proposer must hold a combined voting power (Token + Reputation) greater than `proposal_threshold` (e.g., 100,000 units).

**Action**: The user calls `propose()` with:
- Description hash (IPFS CID)
- Execution data (WASM calls)

**Smart Contract Function**:
```rust
pub fn propose(
    env: Env,
    proposer: Address,
    description_hash: Bytes,
    execution_data: Bytes,
) -> Result<u64, Error>
```

#### Phase 2: Voting Delay

- **Duration**: 2 Days (configurable)
- **Purpose**: Cooling-off period for community discussion before voting begins
- **Snapshot**: Balances are snapshotted during this period

#### Phase 3: Active Voting

- **Duration**: 5 Days (configurable)
- **Mechanism**: Users cast votes (`For`, `Against`, `Abstain`)
- **Power Calculation**: `Voting Power = SUT Balance + Reputation Score`

**Vote Types**:
- `For`: Approve the proposal
- `Against`: Reject the proposal
- `Abstain`: Neutral stance (counts towards quorum)

#### Phase 4: Resolution & Timelock

**Success Criteria**:
1. `For` votes > `Against` votes
2. Total votes meet the `Quorum` requirement (e.g., 4% in basis points)

**Queuing**: Successful proposals enter a Timelock queue (default 24 hours).

#### Phase 5: Dispute Window

**Safety Check**: During the Timelock, any Arbiter can call `dispute()` on the proposal ID.

**Effect**: If disputed, the proposal state changes to `Disputed` and cannot be executed until resolved by the council.

#### Phase 6: Execution

**Finalization**: If the Timelock expires and no disputes exist, anyone can call `execute()`. The Governor triggers the Treasury or Contract upgrades.

### 1.3 Voting Power Calculation

```rust
pub fn get_power(env: &Env, cfg: &GovernorConfig, voter: &Address) -> i128 {
    let token_balance = get_token_balance(cfg.token, voter);
    let reputation = get_reputation(cfg.rep_contract, voter);
    token_balance + reputation
}
```

**Rules**:
- Reputation is non-transferable and bound to the address
- Token balance is snapshotted at the start of voting
- Reputation can be slashed via governance proposal for malicious behavior

---

## Upgrade Procedures

### 2.1 Upgrade Architecture

The Uzima-Contracts repository implements a sophisticated upgradeability system designed for long-term maintainability, security, and zero-downtime updates. The system leverages Soroban's native `update_current_contract_wasm` capability.

### 2.2 UpgradeManager Contract

The `UpgradeManager` contract acts as the central authority for all upgrades:

**Features**:
- **Proposal System**: Admins propose a new WASM hash
- **Timelock**: Mandatory delay (default 24h) before execution
- **Multi-Sig Validation**: Requires multiple validator signatures for high-stakes upgrades
- **Auditable History**: Tracks every upgrade, version number, and description

### 2.3 Standard Upgrade Procedure

#### Step 1: Development
```bash
# Build the new contract WASM
cd contracts/healthcare_payment
cargo build --target wasm32-unknown-unknown --release
soroban contract optimize --wasm target/wasm32-unknown-unknown/release/healthcare_payment.wasm
```

#### Step 2: Installation
```bash
# Deploy (install) the new WASM hash to the network
soroban contract install --wasm target/wasm32-unknown-unknown/release/healthcare_payment.wasm
```

#### Step 3: Proposal
```rust
// Call propose_upgrade on the UpgradeManager
pub fn propose_upgrade(
    env: Env,
    contract_id: Address,
    new_wasm_hash: Bytes,
    description: String,
) -> Result<u64, Error>
```

#### Step 4: Observation
- Wait for the timelock to expire (default 24 hours)
- Community can review and audit the new code

#### Step 5: Approval
- Obtain necessary validator signatures
- Multi-sig threshold must be met

#### Step 6: Execution
```rust
pub fn execute_upgrade(
    env: Env,
    upgrade_id: u64,
) -> Result<(), Error>
```

#### Step 7: Migration (Optional)
If the data layout changed, the new version's `initialize` or a dedicated `migrate` function should be called.

```rust
pub fn migrate(
    env: Env,
    old_version: u32,
    migration_data: Bytes,
) -> Result<(), Error>
```

### 2.4 Emergency Rollback Procedure

In case of a critical bug:

1. **Detection**: Monitoring system or user reports critical issue
2. **Proposal**: Submit emergency rollback proposal with:
   - Previous known-good WASM hash
   - Justification and impact assessment
3. **Fast-Track**: Emergency proposals can bypass standard voting delay (requires 75% supermajority)
4. **Execution**: Execute rollback within 2 hours of approval

### 2.5 Security Features

- **Rollback**: `UpgradeManager` can facilitate rollback to previous known-good WASM hash
- **Freezing**: Admin can permanently freeze a contract, disabling future upgrades (immutability lock)
- **Required Auth**: All upgrade functions require strict cryptographic authentication
- **Immutable Storage**: Critical configuration stored in `Instance` storage, user data in `Persistent` storage
- **Version Tracking**: Every contract stores its current version

---

## Emergency Protocols

### 3.1 Emergency Response Framework

#### Level 1: Low Severity
- **Impact**: Minor functionality issues, non-critical bugs
- **Response Time**: 24-48 hours
- **Authority**: Contract maintainers
- **Actions**: Standard proposal process

#### Level 2: Medium Severity
- **Impact**: Partial service degradation, moderate security concerns
- **Response Time**: 4-12 hours
- **Authority**: Emergency Response Team (ERT)
- **Actions**: Expedited proposal with reduced voting period

#### Level 3: High Severity
- **Impact**: Critical security vulnerability, significant fund risk
- **Response Time**: 1-4 hours
- **Authority**: Emergency Response Team + Multi-sig
- **Actions**: Emergency pause, immediate proposal

#### Level 4: Critical Severity
- **Impact**: Active exploit, catastrophic failure
- **Response Time**: Immediate (< 1 hour)
- **Authority**: Emergency Response Team + Multi-sig + Core Devs
- **Actions**: Contract freeze, emergency upgrade

### 3.2 Emergency Pause Protocol

```rust
pub fn emergency_pause(
    env: Env,
    reason: String,
    evidence_hash: Bytes,
) -> Result<(), Error>
```

**Requirements**:
- Authorization from 3/5 Emergency Response Team members
- Valid reason and evidence hash (IPFS)
- Automatic notification to all stakeholders

**Effects**:
- All state-changing functions are disabled
- Read-only operations remain available
- Timelock starts for emergency resolution (4 hours)

### 3.3 Emergency Upgrade Protocol

**Trigger Conditions**:
1. Critical vulnerability discovered
2. Active exploit in progress
3. Data corruption detected
4. Governance failure

**Procedure**:

1. **Declaration**: ERT declares emergency (requires 3/5 signatures)
2. **Notification**: All stakeholders notified via multiple channels
3. **Proposal**: Emergency upgrade proposal submitted
4. **Voting**: 2-hour voting period (reduced from 5 days)
5. **Execution**: Immediate execution upon approval
6. **Post-Mortem**: Detailed analysis within 7 days

### 3.4 Incident Response Playbook

#### Smart Contract Vulnerability

1. **Detection**
   - Automated monitoring alerts
   - Bug bounty report
   - User report

2. **Triage**
   - Assess severity level
   - Determine impact scope
   - Identify affected contracts

3. **Containment**
   - Emergency pause if Level 3-4
   - Isolate affected components
   - Notify stakeholders

4. **Eradication**
   - Develop patch
   - Test in staging environment
   - Propose upgrade

5. **Recovery**
   - Execute upgrade
   - Verify fix
   - Resume operations

6. **Post-Incident**
   - Root cause analysis
   - Update documentation
   - Improve monitoring

#### Key Compromise

1. **Immediate Actions**
   - Revoke compromised key
   - Rotate all related keys
   - Enable multi-sig requirements

2. **Assessment**
   - Determine what was accessed
   - Check for unauthorized transactions
   - Review access logs

3. **Recovery**
   - Restore from secure backups
   - Implement additional security measures
   - Notify affected parties

---

## Community Participation

### 4.1 Participation Mechanisms

#### Token-Based Voting

**Eligibility**: Holders of SUT tokens
- 1 Token = 1 Vote
- Tokens must be held at snapshot time
- Delegation supported

#### Reputation-Based Voting

**Eligibility**: Contributors with reputation score
- Earned through contributions
- Non-transferable
- Can be slashed for malicious behavior

#### Delegation

Token holders can delegate their voting power:
```rust
pub fn delegate(
    env: Env,
    delegatee: Address,
    amount: i128,
) -> Result<(), Error>
```

### 4.2 Proposal Submission

**Requirements**:
- Minimum voting power: 100,000 units
- Valid description hash (IPFS)
- Valid execution data
- Proposal deposit: 1,000 SUT (refundable if successful)

**Process**:
1. Prepare proposal off-chain
2. Submit via Governor contract
3. Wait for voting delay
4. Participate in voting
5. Monitor results

### 4.3 Discussion Forums

- **Governance Forum**: Discourse-based discussion
- **Discord**: Real-time community chat
- **Town Halls**: Monthly video calls
- **GitHub Issues**: Technical discussions

### 4.4 Transparency Measures

- All proposals publicly viewable
- Voting records transparent
- Treasury transactions auditable
- Regular reporting (monthly)
- Open-source codebase

---

## Risk Management Framework

### 5.1 Risk Categories

#### Technical Risks
- Smart contract vulnerabilities
- Upgrade failures
- Network congestion
- Oracle failures

#### Financial Risks
- Market volatility
- Liquidity issues
- Treasury mismanagement
- Flash loan attacks

#### Governance Risks
- Voter apathy
- Plutocratic capture
- Proposal spam
- Governance attacks

#### Operational Risks
- Key compromise
- Infrastructure failure
- Regulatory changes
- Legal challenges

### 5.2 Risk Mitigation Strategies

#### Technical Risk Mitigation

1. **Code Audits**
   - Pre-deployment audits by 3+ firms
   - Continuous monitoring
   - Bug bounty program

2. **Testing**
   - Comprehensive test suite (>90% coverage)
   - Integration tests
   - Stress tests
   - Formal verification for critical components

3. **Upgrade Safety**
   - Timelock delays
   - Multi-sig requirements
   - Rollback capability
   - Staged rollouts

#### Financial Risk Mitigation

1. **Treasury Management**
   - Diversified holdings
   - Risk-adjusted allocations
   - Regular rebalancing
   - Emergency reserves (6 months)

2. **Insurance**
   - Smart contract coverage
   - Custody insurance
   - Liability coverage

3. **Circuit Breakers**
   - Automatic pause on extreme volatility
   - Daily withdrawal limits
   - Rate limiting

#### Governance Risk Mitigation

1. **Voting Incentives**
   - Staking rewards for participation
   - Reputation bonuses
   - Proposal submission rewards

2. **Anti-Capture Measures**
   - Quadratic voting (under consideration)
   - Reputation-weighted voting
   - Delegation limits
   - Minimum participation requirements

3. **Spam Prevention**
   - Proposal deposit
   - Minimum voting power threshold
   - Cool-down periods

#### Operational Risk Mitigation

1. **Key Management**
   - Multi-sig wallets (3/5 minimum)
   - Hardware security modules
   - Regular key rotation
   - Shamir's Secret Sharing

2. **Disaster Recovery**
   - Geographic redundancy
   - Regular backups
   - Recovery procedures
   - Incident response team

3. **Compliance**
   - Legal review of proposals
   - Regulatory monitoring
   - KYC/AML for sensitive operations
   - Jurisdiction analysis

### 5.3 Risk Assessment Matrix

| Risk | Probability | Impact | Score | Mitigation |
|------|------------|--------|-------|------------|
| Smart Contract Bug | Medium | Critical | 12 | Audits, Bug Bounty, Formal Verification |
| Governance Attack | Low | High | 8 | Reputation System, Anti-Capture |
| Key Compromise | Low | High | 8 | Multi-sig, HSM, Rotation |
| Market Crash | Medium | Medium | 9 | Diversification, Circuit Breakers |
| Regulatory Action | Low | High | 8 | Legal Review, Compliance |

### 5.4 Continuous Monitoring

- **Automated Alerts**: Price feeds, contract events, governance metrics
- **Dashboard**: Real-time risk metrics
- **Reporting**: Weekly risk reports
- **Review**: Monthly risk committee meetings

---

## Decision Procedures

### 6.1 Standard Decision Procedure

#### For Contract Upgrades

1. **Pre-Proposal Phase**
   - Technical review by core team
   - Security audit completion
   - Test deployment on testnet
   - Community feedback collection

2. **Proposal Phase**
   - Submit proposal with:
     - Technical specification
     - Security audit report
     - Impact assessment
     - Migration plan (if needed)
   - Pay proposal deposit
   - Wait for voting delay (2 days)

3. **Voting Phase**
   - 5-day voting period
   - Active campaigning allowed
   - Delegation encouraged
   - Real-time vote tracking

4. **Resolution Phase**
   - Vote tallying
   - Quorum check (4% minimum)
   - Timelock activation (24 hours)
   - Dispute window (48 hours)

5. **Execution Phase**
   - Execute upgrade
   - Verify deployment
   - Monitor for issues
   - Report results

#### For Treasury Decisions

1. **Small Expenditures (< $10,000)**
   - Multi-sig approval (3/5)
   - No governance vote required
   - Monthly reporting

2. **Medium Expenditures ($10,000 - $100,000)**
   - Expedited proposal (3-day voting)
   - Lower quorum (2%)
   - Multi-sig execution

3. **Large Expenditures (>$100,000)**
   - Standard proposal process
   - Full governance vote
   - Detailed justification required

### 6.2 Emergency Decision Procedure

#### Fast-Track Process

1. **Emergency Declaration**
   - 3/5 ERT signatures required
   - Clear justification
   - Evidence documentation

2. **Emergency Proposal**
   - 12-hour community review
   - 2-hour voting period
   - 50% quorum requirement
   - 75% supermajority threshold

3. **Immediate Action**
   - Can execute without timelock
   - Post-execution review required
   - Full report within 7 days

### 6.3 Dispute Resolution

1. **Dispute Filing**
   - Any arbiter can dispute
   - Requires 10,000 SUT deposit
   - Clear grounds for dispute

2. **Review Process**
   - 7-day review period
   - Evidence collection
   - Expert consultation

3. **Resolution**:
   - Council vote (5 members)
   - 4/5 majority required
   - Binding decision

---

## Emergency Playbooks

### 7.1 Smart Contract Exploit Playbook

#### Immediate Response (0-1 hour)

- [ ] ERT declares emergency (3/5 signatures)
- [ ] Emergency pause activated
- [ ] All stakeholders notified
- [ ] Incident channel created
- [ ] War room established

#### Assessment Phase (1-4 hours)

- [ ] Vulnerability identified
- [ ] Impact scope determined
- [ ] Affected contracts listed
- [ ] Funds at risk calculated
- [ ] Communication plan activated

#### Mitigation Phase (4-12 hours)

- [ ] Patch developed
- [ ] Tested in staging
- [ ] Emergency proposal submitted
- [ ] Community briefed
- [ ] Validator coordination

#### Recovery Phase (12-24 hours)

- [ ] Emergency upgrade executed
- [ ] Vulnerability verified fixed
- [ ] Operations resumed
- [ ] Post-mortem initiated
- [ ] Stakeholder update

#### Post-Incident (1-7 days)

- [ ] Root cause analysis completed
- [ ] Full report published
- [ ] Compensation plan (if needed)
- [ ] Security improvements implemented
- [ ] Process updates made

### 7.2 Key Compromise Playbook

#### Immediate Response (0-30 minutes)

- [ ] Compromised key identified
- [ ] Key immediately revoked
- [ ] All related keys rotated
- [ ] Multi-sig threshold increased
- [ ] Emergency session called

#### Assessment Phase (30 minutes - 2 hours)

- [ ] Access logs reviewed
- [ ] Unauthorized actions identified
- [ ] Affected systems listed
- [ ] Impact assessment completed
- [ ] Stakeholders notified

#### Recovery Phase (2-8 hours)

- [ ] New keys generated (HSM)
- [ ] Access controls updated
- [ ] Systems verified secure
- [ ] Operations resumed
- [ ] Enhanced monitoring enabled

#### Post-Incident (1-7 days)

- [ ] Incident report published
- [ ] Security audit conducted
- [ ] Key management improved
- [ ] Training updated
- [ ] Procedures enhanced

### 7.3 Governance Attack Playbook

#### Detection Phase

- [ ] Suspicious voting patterns detected
- [ ] Sybil attack identified
- [ ] Proposal spam observed
- [ ] Vote buying suspected

#### Response Phase

- [ ] Emergency proposal to pause
- [ ] Voting frozen (if authorized)
- [ ] Investigation launched
- [ ] Community informed
- [ ] Evidence collected

#### Resolution Phase

- [ ] Attack vector identified
- [ ] Countermeasures deployed
- [ ] Governance parameters adjusted
- [ ] Voting resumed
- [ ] Attackers identified (if possible)

#### Prevention Phase

- [ ] Anti-sybil measures enhanced
- [ ] Voting mechanisms improved
- [ ] Monitoring strengthened
- [ ] Community education
- [ ] Policy updates

### 7.4 Market Crash Playbook

#### Monitoring Phase

- [ ] Price feeds monitored
- [ ] Circuit breakers checked
- [ ] Liquidity assessed
- [ ] Risk metrics reviewed

#### Response Phase (if triggered)

- [ ] Automatic pause activated
- [ ] Emergency session called
- [ ] Treasury exposure assessed
- [ ] Stakeholders notified
- [ ] Strategy review initiated

#### Stabilization Phase

- [ ] Liquidity provisions deployed
- [ ] Risk parameters adjusted
- [ ] Trading gradually resumed
- [ ] Market making activated
- [ ] Volatility monitoring

#### Recovery Phase

- [ ] Market conditions normalized
- [ ] Parameters reset
- [ ] Lessons learned documented
- [ ] Risk models updated
- [ ] Stress tests conducted

---

## Appendices

### Appendix A: Contact Information

- **Emergency Response Team**: [To be populated]
- **Core Developers**: [To be populated]
- **Governance Forum**: [Link]
- **Discord**: [Link]
- **Treasury Multisig**: [Address]

### Appendix B: Key Parameters

| Parameter | Value | Description |
|-----------|-------|-------------|
| Voting Delay | 2 days | Time before voting starts |
| Voting Period | 5 days | Duration of voting |
| Timelock | 24 hours | Delay before execution |
| Quorum | 4% | Minimum participation |
| Proposal Threshold | 100,000 | Minimum voting power |
| Dispute Period | 48 hours | Time for disputes |

### Appendix C: Legal Disclaimers

This governance guide is for informational purposes only and does not constitute legal advice. All participants should seek independent legal counsel. The Uzima DAO reserves the right to modify governance parameters as needed.

### Appendix D: Version History

| Version | Date | Author | Changes |
|---------|------|--------|---------|
| 1.0.0 | 2026-04-25 | Governance Team | Initial release |

---

*Document Status: Active*
*Last Updated: 2026-04-25*
*Next Review: 2026-07-25*