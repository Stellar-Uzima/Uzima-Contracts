# Decision Procedures Documentation

## Overview

This document details the formal decision procedures for the Uzima Contract Governance System. It provides step-by-step processes for making decisions on contract upgrades, treasury management, emergency responses, and protocol changes.

## Table of Contents

1. [Proposal Submission Process](#proposal-submission-process)
2. [Voting Procedures](#voting-procedures)
3. [Execution Procedures](#execution-procedures)
4. [Emergency Decision Procedures](#emergency-decision-procedures)
5. [Dispute Resolution](#dispute-resolution)
6. [Treasury Decision Procedures](#treasury-decision-procedures)

---

## Proposal Submission Process

### 1.1 Standard Proposal Submission

#### Eligibility Requirements

- **Minimum Voting Power**: 100,000 units (combined SUT + Reputation)
- **Active Reputation**: Non-negative reputation score
- **No Active Proposals**: Cannot have more than 2 active proposals
- **Proposal Deposit**: 1,000 SUT (refundable upon success)

#### Required Information

1. **Title**: Clear, descriptive title (max 100 characters)
2. **Description**: Detailed explanation (IPFS hash)
3. **Rationale**: Why this proposal is needed
4. **Specification**: Technical or operational details
5. **Execution Data**: Calldata for contract calls
6. **Impact Assessment**: Potential risks and benefits
7. **Implementation Timeline**: Estimated execution schedule

#### Submission Steps

**Step 1: Preparation**
```bash
# Prepare proposal documentation
mkdir proposals/my-proposal
cd proposals/my-proposal

# Create proposal document
echo "# Proposal Title" > README.md
echo "## Description" >> README.md
echo "## Rationale" >> README.md
# ... etc

# Upload to IPFS
ipfs add README.md
# Returns: QmXoypizjW3WknFiJnKLwHCnL72vedxjQkDDP1mXWo6uco
```

**Step 2: Encode Execution Data**
```rust
// Example: Upgrade contract
let execution_data = borsh::to_vec(&Instruction::Upgrade {
    contract_id: contract_address,
    new_wasm_hash: wasm_hash,
}).unwrap();
```

**Step 3: Submit Proposal**
```rust
// Call Governor contract
let proposal_id = governor.propose(
    proposer_address,
    ipfs_hash.into(),
    execution_data.into(),
)?;
```

**Step 4: Pay Deposit**
```bash
# Transfer deposit to Governor contract
soroban invoke \
  --id $GOVERNOR_ID \
  --fn deposit_proposal \
  --arg $PROPOSAL_ID \
  --amount 1000
```

### 1.2 Expedited Proposal Submission

**Requirements**:
- Minimum voting power: 250,000 units
- Emergency justification required
- 75% supermajority threshold
- 3/5 ERT pre-approval

**Reduced Timeline**:
- Voting delay: 12 hours (vs 2 days)
- Voting period: 2 days (vs 5 days)
- Timelock: 6 hours (vs 24 hours)

### 1.3 Emergency Proposal Submission

**Trigger Conditions**:
- Critical security vulnerability
- Active exploit in progress
- Catastrophic failure
- Governance attack

**Process**:
1. ERT declares emergency (3/5 signatures)
2. Emergency proposal submitted
3. 12-hour community review
4. 2-hour voting period
5. Immediate execution upon approval

---

## Voting Procedures

### 2.1 Voting Timeline

```
Day 0: Proposal Submitted
       ↓
Day 2: Voting Begins (after 2-day delay)
       ↓
Day 7: Voting Ends (5-day period)
       ↓
Day 8: Timelock Begins (24 hours)
       ↓
Day 9: Execution Window Opens
```

### 2.2 Vote Types

#### For Vote
- **Effect**: Supports proposal
- **Weight**: Full voting power
- **Quorum**: Counts towards minimum

#### Against Vote
- **Effect**: Opposes proposal
- **Weight**: Full voting power
- **Quorum**: Counts towards minimum

#### Abstain Vote
- **Effect**: Neutral stance
- **Weight**: Counts towards quorum only
- **Use**: Voter present but undecided

### 2.3 Voting Power Calculation

```rust
pub fn calculate_voting_power(
    env: &Env,
    voter: &Address,
    snapshot_block: u64,
) -> i128 {
    // Token balance at snapshot
    let token_balance = get_token_balance_at(
        voter, 
        snapshot_block
    );
    
    // Reputation score (non-transferable)
    let reputation = get_reputation_score(voter);
    
    // Total voting power
    token_balance + reputation
}
```

### 2.4 Delegation Process

```rust
// Delegate voting power
pub fn delegate_votes(
    env: Env,
    delegatee: Address,
    amount: i128,
) -> Result<(), Error> {
    // Verify delegator has sufficient balance
    // Update delegation records
    // Emit delegation event
}

// Undelegate voting power
pub fn undelegate_votes(
    env: Env,
    delegatee: Address,
    amount: i128,
) -> Result<(), Error> {
    // Verify delegation exists
    // Update delegation records
    // Emit undelegation event
}
```

**Delegation Rules**:
- Can delegate to any address
- Delegation can be partial or full
- Delegation can be revoked at any time
- Delegated votes count towards delegatee's power

### 2.5 Vote Execution

```rust
pub fn cast_vote(
    env: Env,
    voter: Address,
    proposal_id: u64,
    support: u8, // 0=Against, 1=For, 2=Abstain
) -> Result<(), Error> {
    // Verify voting period is active
    // Verify voter hasn't already voted
    // Calculate voting power
    // Update proposal votes
    // Record vote
}
```

### 2.6 Quorum Requirements

| Proposal Type | Minimum Quorum | Description |
|--------------|----------------|-------------|
| Standard | 4% | Regular proposals |
| Expedited | 2% | Fast-track proposals |
| Emergency | 50% | Emergency proposals |
| Constitutional | 20% | Constitution changes |

**Quorum Calculation**:
```
Quorum = Total Supply × Quorum Percentage
```

### 2.7 Vote Tallying

```rust
pub function tally_votes(
    env: Env,
    proposal_id: u64,
) -> Result<bool, Error> {
    let proposal = get_proposal(proposal_id);
    
    // Check if voting period ended
    if now() < proposal.end_time {
        return Err(Error::VotingNotEnded);
    }
    
    // Calculate success
    let success = proposal.for_votes > proposal.against_votes
        && (proposal.for_votes + proposal.against_votes + proposal.abstain_votes) >= quorum;
    
    // Update proposal status
    if success {
        queue_proposal(proposal_id);
    }
    
    success
}
```

---

## Execution Procedures

### 3.1 Timelock Process

**Purpose**: Provide delay between proposal success and execution

**Duration**: 24 hours (configurable)

**Process**:
1. Proposal reaches success criteria
2. Proposal enters Timelock queue
3. 24-hour delay begins
4. Dispute window opens (48 hours)
5. After delay, execution available

```rust
pub fn queue_proposal(
    env: Env,
    proposal_id: u64,
) -> Result<(), Error> {
    let proposal = get_proposal(proposal_id);
    
    // Calculate execution time
    let execution_time = now() + TIMELOCK_DELAY;
    
    // Queue for execution
    timelock_queue.push(QueuedProposal {
        id: proposal_id,
        ready_time: execution_time,
        disputed: false,
    });
}
```

### 3.2 Execution Authorization

**Who Can Execute**:
- Anyone (after timelock expires)
- Must pay execution gas fees
- Requires successful timelock period

**Execution Process**:
```rust
pub fn execute_proposal(
    env: Env,
    proposal_id: u64,
) -> Result<(), Error> {
    let proposal = get_proposal(proposal_id);
    
    // Verify timelock expired
    if now() < proposal.ready_time {
        return Err(Error::TimelockNotExpired);
    }
    
    // Verify not disputed
    if proposal.disputed {
        return Err(Error::ProposalDisputed);
    }
    
    // Verify not executed
    if proposal.executed {
        return Err(Error::AlreadyExecuted);
    }
    
    // Execute proposal
    invoke_proposal(&proposal.exec_data)?;
    
    // Mark as executed
    proposal.executed = true;
}
```

### 3.3 Execution Verification

**Post-Execution Checks**:
1. Verify transaction succeeded
2. Check contract state updated
3. Confirm events emitted
4. Validate storage changes
5. Monitor for anomalies

```rust
pub fn verify_execution(
    env: Env,
    proposal_id: u64,
) -> Result<bool, Error> {
    let proposal = get_proposal(proposal_id);
    
    // Check execution status
    if !proposal.executed {
        return Ok(false);
    }
    
    // Verify state changes
    let state_valid = verify_state_changes(proposal_id);
    
    // Check events
    let events_valid = verify_events(proposal_id);
    
    Ok(state_valid && events_valid)
}
```

---

## Emergency Decision Procedures

### 4.1 Emergency Declaration

**Requirements**:
- 3/5 ERT signatures
- Clear emergency justification
- Evidence documentation

**Process**:
```rust
pub fn declare_emergency(
    env: Env,
    emergency_type: u8,
    justification: String,
    evidence_hash: Bytes,
    signatures: Vec<Signature>,
) -> Result<u64, Error> {
    // Verify 3/5 signatures
    if verify_signatures(signatures) < 3 {
        return Err(Error::InsufficientSignatures);
    }
    
    // Create emergency declaration
    let emergency = EmergencyDeclaration {
        id: next_emergency_id(),
        emergency_type,
        justification,
        evidence_hash,
        declared_at: now(),
        status: EmergencyStatus::Active,
    };
    
    // Activate emergency pause
    activate_emergency_pause();
    
    // Notify stakeholders
    notify_emergency(emergency);
    
    emergency.id
}
```

### 4.2 Emergency Voting

**Timeline**:
- 12-hour community review
- 2-hour voting period
- Immediate execution upon approval

**Thresholds**:
- Quorum: 50%
- Approval: 75% supermajority

```rust
pub fn emergency_vote(
    env: Env,
    voter: Address,
    emergency_id: u64,
    support: bool,
) -> Result<(), Error> {
    // Verify emergency is active
    let emergency = get_emergency(emergency_id);
    if emergency.status != EmergencyStatus::Active {
        return Err(Error::EmergencyNotActive);
    }
    
    // Verify within voting period
    if now() > emergency.voting_end_time {
        return Err(Error::VotingEnded);
    }
    
    // Record vote
    record_emergency_vote(voter, emergency_id, support);
}
```

### 4.3 Emergency Execution

**Immediate Execution**:
- Can bypass timelock
- Requires 75% approval
- 50% quorum met

```rust
pub fn execute_emergency(
    env: Env,
    emergency_id: u64,
) -> Result<(), Error> {
    let emergency = get_emergency(emergency_id);
    
    // Verify approval threshold
    let approval = calculate_approval(emergency_id);
    if approval < 0.75 {
        return Err(Error::InsufficientApproval);
    }
    
    // Verify quorum
    let quorum = calculate_quorum(emergency_id);
    if quorum < 0.50 {
        return Err(Error::InsufficientQuorum);
    }
    
    // Execute immediately
    execute_emergency_action(emergency);
    
    // Update status
    emergency.status = EmergencyStatus::Resolved;
}
```

---

## Dispute Resolution

### 5.1 Dispute Filing

**Requirements**:
- 10,000 SUT deposit (refundable if valid)
- Clear grounds for dispute
- Evidence documentation
- Filed within dispute window

**Grounds for Dispute**:
1. Proposal violates DAO Constitution
2. Execution would cause harm
3. Fraud or misrepresentation
4. Technical error in proposal
5. Legal compliance issue

**Process**:
```rust
pub fn file_dispute(
    env: Env,
    proposal_id: u64,
    grounds: String,
    evidence_hash: Bytes,
    deposit: i128,
) -> Result<u64, Error> {
    // Verify within dispute window
    let proposal = get_proposal(proposal_id);
    if now() > proposal.dispute_deadline {
        return Err(Error::DisputeWindowClosed);
    }
    
    // Verify deposit
    if deposit < MIN_DISPUTE_DEPOSIT {
        return Err(Error::InsufficientDeposit);
    }
    
    // Create dispute
    let dispute = Dispute {
        id: next_dispute_id(),
        proposal_id,
        grounds,
        evidence_hash,
        deposit,
        filed_at: now(),
        status: DisputeStatus::Active,
    };
    
    // Pause proposal execution
    pause_proposal(proposal_id);
    
    dispute.id
}
```

### 5.2 Dispute Review

**Review Panel**: 5 arbiters (rotating)

**Timeline**: 7 days

**Process**:
1. Evidence collection (2 days)
2. Expert consultation (2 days)
3. Panel deliberation (2 days)
4. Decision (1 day)

```rust
pub fn review_dispute(
    env: Env,
    dispute_id: u64,
    arbiter_decisions: Vec<bool>,
) -> Result<bool, Error> {
    // Verify arbiter authorization
    if !is_authorized_arbiter(env.caller) {
        return Err(Error::Unauthorized);
    }
    
    // Count votes (4/5 majority required)
    let approve_count = arbiter_decisions.iter().filter(|&d| *d).count();
    let decision = approve_count >= 4;
    
    // Update dispute status
    let dispute = get_dispute(dispute_id);
    dispute.decision = decision;
    dispute.resolved_at = now();
    
    // Execute decision
    if decision {
        // Cancel proposal
        cancel_proposal(dispute.proposal_id);
        // Return deposit to filer
        refund_deposit(dispute.deposit);
    } else {
        // Resume proposal
        resume_proposal(dispute.proposal_id);
        // Forfeit deposit
        forfeit_deposit(dispute.deposit);
    }
    
    decision
}
```

### 5.3 Dispute Outcomes

**Dispute Upheld**:
- Proposal cancelled
- Deposit returned to filer
- Proposal cannot be resubmitted (30-day ban)

**Dispute Rejected**:
- Proposal continues
- Deposit forfeited
- Execution proceeds as planned

---

## Treasury Decision Procedures

### 6.1 Expenditure Categories

| Category | Threshold | Approval Process |
|----------|-----------|------------------|
| Operational | <$1,000 | Multi-sig (2/5) |
| Small | $1,000 - $10,000 | Multi-sig (3/5) |
| Medium | $10,000 - $100,000 | Expedited Proposal |
| Large | $100,000 - $1,000,000 | Standard Proposal |
| Major | >$1,000,000 | Standard Proposal + Audit |

### 6.2 Operational Expenditures

**Process**:
1. Submit expenditure request
2. Multi-sig approval (2/5)
3. Execute payment
4. Monthly reporting

**Requirements**:
- Valid invoice
- Budget allocation
- Purpose documentation

### 6.3 Small Expenditures

**Process**:
1. Submit expenditure request
2. Multi-sig approval (3/5)
3. Execute payment
4. Quarterly reporting

**Requirements**:
- Valid invoice
- Budget allocation
- Purpose documentation
- Vendor verification

### 6.4 Medium Expenditures

**Process**:
1. Prepare expedited proposal
2. 3-day community review
3. 3-day voting period
4. Multi-sig execution (3/5)

**Requirements**:
- Detailed budget
- Vendor quotes (3 minimum)
- Impact assessment
- Timeline

### 6.5 Large Expenditures

**Process**:
1. Prepare standard proposal
2. 2-day voting delay
3. 5-day voting period
4. 24-hour timelock
5. Governance execution

**Requirements**:
- Detailed budget
- Vendor quotes (5 minimum)
- Impact assessment
- Risk analysis
- Legal review

### 6.6 Major Expenditures

**Process**:
1. Prepare standard proposal
2. Independent audit required
3. 2-day voting delay
4. 5-day voting period
5. 24-hour timelock
6. Governance execution

**Requirements**:
- Detailed budget
- Vendor quotes (5 minimum)
- Impact assessment
- Risk analysis
- Legal review
- Independent audit
- Community consultation

---

## Appendices

### Appendix A: Decision Matrix

| Decision Type | Timeline | Quorum | Threshold | Execution |
|--------------|----------|--------|-----------|-----------|
| Standard Proposal | 9 days | 4% | 100k VP | Timelock |
| Expedited Proposal | 4 days | 2% | 250k VP | Timelock |
| Emergency Proposal | 1 day | 50% | ERT | Immediate |
| Treasury (<$10k) | 1 day | N/A | 3/5 MS | Immediate |
| Treasury (>$100k) | 9 days | 4% | Standard | Timelock |

### Appendix B: Contact Information

- **Emergency Response Team**: [To be populated]
- **Treasury Committee**: [To be populated]
- **Dispute Resolution**: [To be populated]

### Appendix C: Templates

- [Proposal Template](templates/proposal.md)
- [Emergency Declaration Template](templates/emergency.md)
- [Dispute Filing Template](templates/dispute.md)

---

*Document Status: Active*
*Last Updated: 2026-04-25*
*Next Review: 2026-07-25*