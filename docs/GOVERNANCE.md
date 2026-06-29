# DAO Governance Framework

## 1. Governance Architecture

The DAO operates on a hybrid governance model designed to balance financial stake with active contribution, safeguarding the protocol through a judicial layer.

### Core Components
* **Governor Contract:** The central logic handler. It calculates voting power, manages proposal lifecycles, and executes passed proposals.
* **SUT Token (Plutocratic Layer):** Represents financial stake. 1 Token = 1 Vote.
* **Reputation System (Meritocratic Layer):** A non-transferable score earned by contributors. 1 Reputation Point = 1 Vote.
* **Dispute Resolution (Judicial Layer):** A council of arbiters who can veto malicious proposals that pass the vote but violate the DAO Constitution.
* **Emergency Response Team (ERT):** 5-member team authorized to declare emergencies and execute emergency protocols.

---

## 2. Proposal Lifecycle

### Phase 1: Proposal
* **Threshold:** A proposer must hold a combined voting power (Token + Reputation) greater than `proposal_threshold` (e.g., 100,000 units).
* **Action:** The user calls `propose()` with a description hash (IPFS CID) and the executable data (WASM calls).
* **Deposit:** 1,000 SUT deposit required (refundable upon success).

### Phase 2: Voting Delay
* **Duration:** 2 Days (configurable).
* **Purpose:** A cooling-off period for the community to discuss the proposal before voting begins. Snapshotting of balances occurs here.

### Phase 3: Active Voting
* **Duration:** 5 Days (configurable).
* **Mechanism:** Users cast votes (`For`, `Against`, `Abstain`).
* **Power Calculation:** `Voting Power = SUT Balance + Reputation Score`.
* **Delegation:** Token holders can delegate voting power to other addresses.

### Phase 4: Resolution & Timelock
* **Success Criteria:**
    1. `For` votes > `Against` votes.
    2. Total votes meet the `Quorum` requirement (4% minimum).
* **Queuing:** Successful proposals enter a Timelock queue (24-hour delay).

### Phase 5: Dispute Window
* **Safety Check:** During the Timelock, any Arbiter can call `dispute()` on the proposal ID.
* **Effect:** If disputed, the proposal state changes to `Disputed` and cannot be executed until resolved by the council.
* **Deposit:** 10,000 SUT deposit required to file a dispute.

### Phase 6: Execution
* **Finalization:** If the Timelock expires and no disputes exist, anyone can call `execute()`. The Governor triggers the Treasury or Contract upgrades.

---

## 3. Treasury Management

The Treasury Controller has been upgraded to support multiple execution paths:

### 3.1 Expenditure Categories

| Category | Threshold | Approval Process |
|----------|-----------|------------------|
| Operational | <$1,000 | Multi-sig (2/5) |
| Small | $1,000 - $10,000 | Multi-sig (3/5) |
| Medium | $10,000 - $100,000 | Expedited Proposal |
| Large | $100,000 - $1,000,000 | Standard Proposal |
| Major | >$1,000,000 | Standard Proposal + Audit |

### 3.2 Execution Paths

1.  **Multisig Ops (Fast Track):** Routine operational expenses (e.g., server costs) can be signed by the elected multisig committee without a full DAO vote.
2.  **Governance Execution (DAO Track):** Large capital allocations require a full passed proposal. The Governor contract has special permission to bypass the multisig threshold and execute transfers directly via `governance_execute`.

---

## 4. Reputation Rules

* **Earning:** Reputation is minted by the DAO (via passed proposals) to reward code contributions, community management, or auditing.
* **Slashing:** If a contributor acts maliciously, a governance proposal can slash their reputation score.
* **Non-Transferable:** Reputation is bound to the address and cannot be sold or transferred.
* **Voting Power:** Reputation contributes directly to voting power (1:1 ratio with tokens).

---

## 5. Emergency Protocols

### 5.1 Emergency Levels

#### Level 1: Low Severity
- **Impact:** Minor functionality issues, non-critical bugs
- **Response Time:** 24-48 hours
- **Authority:** Contract maintainers

#### Level 2: Medium Severity
- **Impact:** Partial service degradation, moderate security concerns
- **Response Time:** 4-12 hours
- **Authority:** Emergency Response Team (ERT)

#### Level 3: High Severity
- **Impact:** Critical security vulnerability, significant fund risk
- **Response Time:** 1-4 hours
- **Authority:** ERT + Multi-sig

#### Level 4: Critical Severity
- **Impact:** Active exploit, catastrophic failure
- **Response Time:** Immediate (< 1 hour)
- **Authority:** ERT + Multi-sig + Core Devs

### 5.2 Emergency Pause

```rust
pub fn emergency_pause(
    env: Env,
    reason: String,
    evidence_hash: Bytes,
    signatures: Vec<Signature>,
) -> Result<(), Error>
```

**Requirements:**
- Authorization from 3/5 ERT members
- Valid reason and evidence hash (IPFS)
- Automatic notification to all stakeholders

### 5.3 Emergency Upgrade

**Trigger Conditions:**
1. Critical vulnerability discovered
2. Active exploit in progress
3. Data corruption detected
4. Governance failure

**Procedure:**
1. ERT declares emergency (3/5 signatures)
2. Emergency proposal submitted
3. 2-hour voting period (reduced from 5 days)
4. 50% quorum requirement
5. 75% supermajority threshold
6. Immediate execution upon approval

---

## 6. Risk Management

### 6.1 Risk Categories

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

### 6.2 Mitigation Strategies

#### Technical Risk Mitigation
1. **Code Audits:** Pre-deployment audits by 3+ firms
2. **Testing:** Comprehensive test suite (>90% coverage)
3. **Upgrade Safety:** Timelock delays, multi-sig requirements, rollback capability

#### Financial Risk Mitigation
1. **Treasury Management:** Diversified holdings, risk-adjusted allocations
2. **Insurance:** Smart contract coverage, custody insurance
3. **Circuit Breakers:** Automatic pause on extreme volatility

#### Governance Risk Mitigation
1. **Voting Incentives:** Staking rewards, reputation bonuses
2. **Anti-Capture Measures:** Reputation-weighted voting, delegation limits
3. **Spam Prevention:** Proposal deposit, minimum thresholds

#### Operational Risk Mitigation
1. **Key Management:** Multi-sig wallets (3/5), HSM, regular rotation
2. **Disaster Recovery:** Geographic redundancy, regular backups
3. **Compliance:** Legal review, regulatory monitoring

---

## 7. Community Participation

### 7.1 Participation Mechanisms

#### Token-Based Voting
- **Eligibility:** Holders of SUT tokens
- **Power:** 1 Token = 1 Vote
- **Delegation:** Supported

#### Reputation-Based Voting
- **Eligibility:** Contributors with reputation score
- **Power:** 1 Reputation Point = 1 Vote
- **Transferability:** Non-transferable

### 7.2 Proposal Submission

**Requirements:**
- Minimum voting power: 100,000 units
- Valid description hash (IPFS)
- Valid execution data
- Proposal deposit: 1,000 SUT

**Process:**
1. Prepare proposal off-chain
2. Submit via Governor contract
3. Wait for voting delay
4. Participate in voting
5. Monitor results

### 7.3 Discussion Forums
- **Governance Forum:** Discourse-based discussion
- **Discord:** Real-time community chat
- **Town Halls:** Monthly video calls
- **GitHub Issues:** Technical discussions

### 7.4 Transparency Measures
- All proposals publicly viewable
- Voting records transparent
- Treasury transactions auditable
- Regular reporting (monthly)
- Open-source codebase

---

## 8. Decision Procedures

### 8.1 Standard Decision Procedure

#### For Contract Upgrades
1. **Pre-Proposal:** Technical review, security audit, testnet deployment
2. **Proposal:** Submit with technical spec, audit report, impact assessment
3. **Voting:** 5-day voting period with delegation support
4. **Resolution:** Quorum check (4%), timelock activation (24h)
5. **Execution:** Execute upgrade, verify deployment, monitor

#### For Treasury Decisions
- **Small (<$10k):** Multi-sig (3/5), no governance vote
- **Medium ($10k-$100k):** Expedited proposal (3-day voting)
- **Large (>$100k):** Standard proposal process

### 8.2 Emergency Decision Procedure

**Fast-Track Process:**
1. Emergency declaration (3/5 ERT signatures)
2. Emergency proposal submission
3. 12-hour community review
4. 2-hour voting period
5. Immediate execution upon approval

### 8.3 Dispute Resolution

1. **Dispute Filing:** 10,000 SUT deposit, clear grounds, evidence
2. **Review Process:** 7-day review, evidence collection, expert consultation
3. **Resolution:** Council vote (4/5 majority), binding decision

---

## 9. Upgrade Procedures

### 9.1 Standard Upgrade Procedure

1. **Development:** Build new contract WASM
2. **Installation:** Deploy new WASM hash to network
3. **Proposal:** Call `propose_upgrade` on UpgradeManager
4. **Observation:** Wait for timelock (24 hours)
5. **Approval:** Obtain validator signatures
6. **Execution:** Trigger `execute` on UpgradeManager
7. **Migration:** Call migrate function if data layout changed

### 9.2 Emergency Rollback

In case of critical bug:
1. **Detection:** Monitoring system or user reports
2. **Proposal:** Submit emergency rollback proposal
3. **Fast-Track:** Bypass standard voting delay (75% supermajority)
4. **Execution:** Execute rollback within 2 hours

### 9.3 Security Features

- **Rollback:** Facilitate rollback to previous known-good WASM hash
- **Freezing:** Permanently freeze contract (immutability lock)
- **Required Auth:** Strict cryptographic authentication
- **Immutable Storage:** Critical config in Instance storage
- **Version Tracking:** Every contract stores current version

---

*For detailed procedures, see:*
- [Contract Governance Guide](CONTRACT_GOVERNANCE_GUIDE.md)
- [Decision Procedures](DECISION_PROCEDURES.md)
- [Emergency Playbooks](EMERGENCY_PLAYBOOKS.md)