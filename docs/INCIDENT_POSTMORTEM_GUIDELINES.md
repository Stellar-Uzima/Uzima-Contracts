# Incident Postmortem Guidelines and Training

## Overview

This document provides comprehensive guidelines for conducting effective incident postmortems for the Uzima-Contracts project. It includes best practices, team training resources, and step-by-step guidance on using the postmortem template.

---

## Table of Contents

1. [What is an Incident Postmortem?](#what-is-an-incident-postmortem)
2. [Why Postmortems Matter](#why-postmortems-matter)
3. [Postmortem Best Practices](#postmortem-best-practices)
4. [When to Conduct a Postmortem](#when-to-conduct-a-postmortem)
5. [Postmortem Process](#postmortem-process)
6. [Role Definitions](#role-definitions)
7. [Timeline - Detailed Guidance](#timeline---detailed-guidance)
8. [Root Cause Analysis Techniques](#root-cause-analysis-techniques)
9. [Impact Assessment Guide](#impact-assessment-guide)
10. [Prevention and Action Items](#prevention-and-action-items)
11. [Lessons Learned Framework](#lessons-learned-framework)
12. [Team Training Checklist](#team-training-checklist)
13. [Common Mistakes to Avoid](#common-mistakes-to-avoid)
14. [Examples and Case Studies](#examples-and-case-studies)

---

## What is an Incident Postmortem?

An **incident postmortem** (also called post-incident review, PIR, or retrospective) is a structured analysis and documentation of an incident that occurred in production or staging environments. 

### Key Characteristics

- **Non-blaming**: Focuses on systems and processes, not individuals
- **Comprehensive**: Documents all aspects of the incident and response
- **Actionable**: Produces specific, trackable action items to prevent recurrence
- **Learning-focused**: Extracts knowledge to improve future operations
- **Time-bound**: Completed within 2 weeks of incident resolution

### Core Goals

1. **Understand** what happened (complete picture)
2. **Identify** root causes (not just symptoms)
3. **Learn** from the incident (knowledge capture)
4. **Improve** systems and processes (prevention)
5. **Share** knowledge across the organization

---

## Why Postmortems Matter

### For Healthcare Organizations

In healthcare, especially with blockchain-based medical records, incidents could impact patient safety and privacy:

- **Patient Privacy**: Medical data breaches must be handled with utmost care and transparency
- **Regulatory Compliance**: HIPAA, GDPR, and other frameworks require incident documentation
- **Trust**: Transparency about incidents and prevention builds patient and provider confidence
- **Continuous Improvement**: Each incident is an opportunity to strengthen safeguards

### For Engineering Teams

- **Knowledge Capture**: Team learns from failures without repetition
- **Process Improvement**: Identifies gaps in testing, monitoring, and deployment processes
- **Psychological Safety**: Normalizes learning from failures (blameless culture)
- **System Resilience**: Prevents similar incidents across different contracts

### For the Organization

- **Institutional Memory**: Documents why decisions were made and lessons learned
- **Regulatory Readiness**: Demonstrates commitment to continuous improvement for audits
- **Risk Reduction**: Systematic approach to reducing categories of incidents
- **Cost Avoidance**: Prevents expensive recurring issues

---

## Postmortem Best Practices

### 1. **Be Blameless**

```
❌ DON'T:  "Engineer failed to check the test results before deploying"
✅ DO:     "The CI/CD pipeline did not automatically block deployment when tests failed"
```

**Why**: Blamelessness focuses on systems not people. The goal is improving systems, not punishing individuals.

### 2. **Be Specific**

```
❌ DON'T:  "There was a bug in the code"
✅ DO:     "Function revoke_patient_access did not emit access_revoked event, causing off-chain 
           indexer to become stale. Affected 47 test users for 2 hours 15 minutes."
```

**Why**: Specificity makes root causes clear and action items actionable.

### 3. **Follow the 5 Whys**

```
Q: Why wasn't access revoked?
A: Because the event wasn't emitted

Q: Why wasn't the event emitted?
A: Because the code didn't call env.events().publish()

Q: Why wasn't this caught in code review?
A: Because the PR reviewer didn't compare the revoke function with the grant function

Q: Why is there no systematic process for this comparison?
A: Because the code review process doesn't have a checklist for asymmetric operations

Action: Add checklist item to enforce comparison of symmetric operations
```

**Why**: Each "why" gets you closer to systematic causes rather than symptoms.

### 4. **Include Everyone Affected**

Invite participants from:
- Engineering team involved in incident
- Operations / DevOps
- Security
- QA
- Product
- Customer support (to share customer impact)
- Leadership (to understand resource needs)

**Why**: Different perspectives reveal blind spots. QA might see a testing gap; Support might reveal customer impact.

### 5. **Timeline First, Root Cause Last**

Build the timeline first (what actually happened), then analyze why. Don't jump to conclusions.

```
Timeline-first approach:
1. Build detailed timeline of events
2. Review logs and data
3. Then identify root causes

Root-cause-first (wrong):
1. Someone suggests a cause
2. Everyone assumes that's correct
3. Miss actual root causes
```

**Why**: Accurate timelines prevent confirmation bias.

### 6. **Document Evidence**

Every conclusion should be backed by evidence:

```
❌ DON'T:  "We think the contract had a state consistency issue"
✅ DO:     "Contract storage query showed doctor removed from permissions (correct state), 
           but event logs had no access_revoked event (missing), and indexer cache still 
           showed doctor with access. Evidence: [link to transaction logs]"
```

**Why**: Evidence-based analysis is credible and reproducible.

### 7. **Distinguish Impact from Cause**

```
Impact: "47 patients potentially had unauthorized access for 2 hours"
Root Cause: "Event emission logic was incomplete in revoke function"
Contributing Factors: "Code review process didn't compare symmetric operations"
```

**Why**: You need to understand both the severity (impact) and the mechanism (cause) to prevent effectively.

### 8. **Make Action Items Specific and Measurable**

```
❌ DON'T:  "Improve testing" (too vague)
✅ DO:     "Add integration test that verifies every state-changing function emits 
           corresponding event. Target: 95%+ code coverage for critical paths. Due: 2025-03-29"
```

**Why**: Vague action items are forgotten or never completed.

---

## When to Conduct a Postmortem

### Mandatory Postmortems

- **Any Critical severity incident** (contract failure, data breach, HIPAA violation)
- **Production downtime > 1 hour**
- **Security incidents** (including potential breaches)
- **Data loss or corruption**
- **Compliance violations**

### Recommended Postmortems

- **High severity incidents** (significant degradation)
- **Multiple customer reports**
- **Unusual or novel problems**
- **Near-misses** (incidents that almost happened)

### Optional Postmortems

- **Medium severity incidents** (may use lighter-weight review)
- **Incidents caught in testing** (may use internal review process)

### Postmortem Timeline

| When | Action |
|---|---|
| During Incident | Rapid incident response (not postmortem) |
| Immediately After Resolution | Declare incident resolved, take preliminary notes |
| 1-3 days After | Initial postmortem session (while details are fresh) |
| 1-2 weeks After | Final postmortem with all stakeholders |
| 1-4 weeks After | Implement prevention measures |
| 30-60 days After | Review and close action items |

---

## Postmortem Process

### Phase 1: Preparation (Incident Commander, 1-2 hours after resolution)

1. **Declare Incident Resolved**
   - Update status page
   - Notify stakeholders
   - Thank response team

2. **Collect Initial Data**
   - Save logs (might be rotated/deleted)
   - Document deployment status
   - Capture monitoring dashboards
   - Preserve evidence

3. **Identify Participants**
   - Who responded to the incident?
   - Who would have different perspective?
   - Who needs to provide context?
   - Incident commander, lead engineer, ops, security, QA, product

4. **Schedule Postmortem**
   - Schedule for 2-5 days after incident (while fresh, but not during crisis)
   - 60-90 minutes meeting
   - Async document for those who can't attend

5. **Share Preparation Materials**
   - List of participants
   - Meeting time
   - Suggested attendees provide brief input before meeting
   - Emphasize blamelessness

### Phase 2: Initial Session (Team, 60-90 minutes)

**Goal**: Build complete, accurate timeline and identify root causes

**Facilitator**: Incident Commander or Senior Leader (someone removed from incident)

**Process**:

1. **Introduction** (5 min)
   - Explain postmortem is blameless
   - Goal is learning, not blame
   - Everyone's perspective is valued

2. **Timeline Building** (20-30 min)
   - Incident Commander or notetaker writes timeline
   - Each participant adds their perspective
   - Ask: "What did you see?" "What did you do?" "What did you assume?"
   - Fill time gaps: "What happened between 14:35 and 14:40?"
   - Add data from logs and systems

3. **Impact Assessment** (10 min)
   - Who was affected? How many?
   - Business impact: revenue, customers, compliance
   - Reputation impact?
   - Recovery time and resources

4. **Root Cause Analysis** (20-30 min)
   - Tech lead explains what failed
   - Walk 5 Whys: Why did this fail? Why wasn't it caught? Why isn't there a process?
   - Identify contributing factors
   - Don't jump to solutions ("fix it by doing X")

5. **Preliminary Action Items** (5-10 min)
   - What needs immediate attention?
   - What should we measure to prevent recurrence?
   - Who will own the postmortem document?
   - When will final version be ready?

### Phase 3: Documentation (Postmortem Lead, 2-7 days)

**Goal**: Document complete analysis with evidence and action items

**Owner**: Usually incident commander or tech lead

**Process**:

1. **Fill in Template**
   - Use provided INCIDENT_POSTMORTEM_TEMPLATE.md
   - Reference the example (docs/examples/INCIDENT_POSTMORTEM_EXAMPLE.md) for format
   - All sections should be detailed but concise

2. **Add Supporting Evidence**
   - Links to GitHub issues
   - Log excerpts
   - Code snippets
   - Transaction history
   - Architecture diagrams

3. **Define Action Items**
   - Short-term (this week)
   - Medium-term (this month)
   - Long-term (next quarter)
   - Each with owner, deadline, acceptance criteria

4. **Share for Review**
   - Incident commander
   - Tech lead
   - Security team (if applicable)
   - Customer success (for customer-facing incidents)
   - Ensure accuracy and completeness

### Phase 4: Review and Approval (Leadership, 1-2 days)

1. **Review Meeting**
   - Brief review to ensure document is complete
   - Approval of action items and timelines
   - Clarification of responsibilities

2. **Approval Sign-off**
   - Incident commander approves completion
   - Leadership approves action items and resources
   - Document marked as "Finalized"

### Phase 5: Distribution and Tracking

1. **Share Postmortem**
   - Team: Full details
   - Leadership: Summary + action items
   - Customers: Public summary (if appropriate)
   - Compliance: Full document (if required)

2. **Track Action Items**
   - Create GitHub issues for each action item
   - Add to project board
   - Track in weekly standups
   - Monthly review of progress

3. **Close-out** (after 30-60 days)
   - Verify all action items completed
   - Note any changes to timeline
   - Document lessons learned
   - Archive postmortem

---

## Role Definitions

### Incident Commander

**Responsible for**: Overall coordination and completion of postmortem

**During incident**:
- Coordinates response
- Makes decisions
- Communicates status

**During postmortem**:
- Schedules postmortem
- Ensures complete timeline
- Tracks action items to completion
- Signs off on final document

**Key Skills**: Leadership, communication, attention to detail

---

### Technical Lead / Infrastructure Owner

**Responsible for**: Technical accuracy of root cause analysis

**Provides**:
- Detailed explanation of technical failure
- Code review
- Architecture context
- Log analysis and interpretation

**Key Skills**: Technical depth, problem-solving, system thinking

---

### Facilitator (if different from Incident Commander)

**Responsible for**: Unbiased facilitation of postmortem meeting

**Ensures**:
- Blameless tone
- All voices heard
- Timeline accuracy
- 5 Why analysis depth
- No premature solutions

**Key Skills**: Facilitation, objectivity, questioning technique

---

### Incident Response Team Participants

**Responsible for**: Providing accurate perspective of incident

**Provides**:
- Timeline of their actions
- What they observed
- What they assumed
- What they missed
- Feedback on processes

**Key Skills**: Attention to detail, honesty, learning mindset

---

### Post-Incident Documentation Owner

**Responsible for**: Comprehensive documentation

**Ensures**:
- Template properly used
- All sections complete
- Evidence linked
- Writing is clear
- Action items specific

**Key Skills**: Writing, organization, attention to detail

---

## Timeline - Detailed Guidance

The timeline is the foundation of a good postmortem. Here's how to build an accurate one:

### What to Include in Timeline

**Events to capture**:
- When first symptoms appeared (from logs, not human memory)
- When different teams detected the issue
- When customer reports started
- When on-call was paged
- When investigation began
- What was tried and when
- Key decisions made
- When resolution was confirmed
- When postmortem process started

### How to Build Accurate Timeline

1. **Start with Logs**
   - Pull system logs with timestamps
   - Pull application logs
   - Pull deployment logs
   - These are more accurate than human memory

2. **Ask Participants: "What did you see?"**
   - "What was the first thing that caught your attention?"
   - "What time did you notice?"
   - "What actions did you take?"
   - Write down answers with times

3. **Cross-Reference**
   - Match participant recollection with logs
   - Note discrepancies
   - Ask clarifying questions
   - Fill gaps

4. **Use Military/UTC Time**
   - Always use UTC to avoid timezone confusion
   - Use 24-hour format (14:32, not 2:32 PM)
   - Include seconds if critical

5. **Add Context to Each Event**
   - Not just "Engineer deployed contract" but "Engineer deployed contract CABC123... to testnet"
   - Not just "Alert fired" but "Alert 'unusual_revocation_patterns' fired with threshold 100/hour"
   - Context helps for future reference

### Example Timeline Format

```
| Time (UTC) | Event | Owner | Details |
|14:32 | Incident Triggered | System | Medical records contract deployed with new access control logic (commit abc123) |
|14:35 | User Report | Patient P12345 | Patient unable to remove doctor access via patient portal UI |
|14:40 | Support Ticket | Support #1847 | Multiple similar complaints received within 5 minutes |
|16:47 | Automated Alert | Monitoring | Alert "Unusual revocation transaction patterns" fired (100+ revocations/hour detected) |
```

**Key principle**: Include enough detail that someone reading this months later understands exactly what happened.

---

## Root Cause Analysis Techniques

### The 5 Whys

**Process**: For each answer, ask "Why?" again until you reach a systemic root cause.

**Example**:

```
Q1: Why did patients lose access to revoke doctor permissions?
A1: Because the revoke_patient_access function didn't emit an event

Q2: Why didn't the function emit an event?
A2: Because the developer didn't include the event emission code when implementing revocation

Q3: Why wasn't the missing event emission caught?
A3: Because the code review process didn't systematically compare the grant and revoke implementations

Q4: Why is there no systematic comparison process?
A4: Because the code review checklist didn't include "verify symmetric operations"

Q5: Why wasn't the checklist comprehensive?
A5: Because the contract was developed before this pattern/risk was known

Root Cause: Lack of systematic code review process for identifying asymmetric implementations of related operations

Suggested Prevention: Add "verify symmetric operations" to code review checklist and create automated tools to detect asymmetry
```

### Fishbone Diagram (Cause and Effect)

Visual method to organize root causes:

```
                     People          Process         Technology
                        |                |                |
                        |                |                |
                   No checklist     No comparison    No automated
                    for asymmetry   of functions      validation
                        |                |                |
                        +----------------+----------------+
                                         |
                    Code Review Failed to Catch Bug
                                         |
                        +----------------+----------------+
                        |                |                |
                   Testing     Monitoring          Environment
                        |                |                |
                 No integration      Alert          Testnet not
                 tests for events   threshold too   fully validated
                                        high
```

### Fault Tree Analysis

Start with the failure and work backward through logical relationships:

```
FAILURE: Patient loses ability to revoke doctor access

├─ On-chain state not updated
│  └─ [FALSE - state WAS updated, checked logs]
│
├─ Off-chain system out of sync
│  ├─ Event not emitted
│  │  └─ [TRUE - ROOT CAUSE]
│  ├─ Indexer not processing events
│  │  └─ [FALSE - indexer works correctly when events present]
│  └─ UI cache not invalidated
│     └─ [FALSE - UI depends on event-driven cache invalidation]
│
└─ Contract deployment failed
   └─ [FALSE - contract deployed and operates correctly]
```

### Change Analysis

Compare what changed between working and non-working versions:

```
Working Code (grant_patient_access):
  1. Check permissions
  2. Update storage
  3. Emit event ← KEY DIFFERENCE
  4. Return success

Broken Code (revoke_patient_access):
  1. Check permissions
  2. Update storage
  3. [NO EVENT EMISSION] ← MISSING
  4. Return success

Difference: Asymmetric event emission pattern

Conclusion: Missing event emission is the root cause
```

### The "Bad Assumption" Method

What did responders assume that proved wrong?

```
Assumption: "The revoke function works the same as grant"
  Reality: Missing event emission in revoke path
  
Assumption: "What you see in UI reflects on-chain state"
  Reality: UI depends on event stream, not on-chain query
  
Assumption: "If a transaction succeeds, it's fully processed"
  Reality: Success = state updated, but off-chain sync needs event emission
```

**Action**: Document assumptions and how to validate them

---

## Impact Assessment Guide

### Data Impact Assessment

**Questions to answer**:

1. **Scope of Data**
   - How many records affected?
   - What type of data (PII, medical/PHI, financial)?
   - What time period?
   - Was data read, written, or deleted?

2. **Data Integrity**
   - Is data corrupted?
   - Is data lost?
   - Is data inconsistent across systems?
   - Can it be recovered?

3. **Data Confidentiality**
   - Was data exposed?
   - Who had unauthorized access?
   - For how long?
   - What was the scope of exposure?

4. **Regulatory Impact**
   - HIPAA: Protected Health Information (PHI) involved?
   - GDPR: Personal data of EU residents involved?
   - Other regulations (CCPA, industry-specific)?
   - Does this trigger breach notification?

**For Uzima healthcare context:**
- Always assume HIPAA applies
- Medical records = PHI = automatic regulatory consideration
- Breach notification rules: Notify HHS and individuals within 60 days
- Document all findings for compliance

### User/Business Impact

**Questions to answer**:

1. **User Affected**
   - How many patients?
   - How many healthcare providers?
   - Specific regions?

2. **Service Impact**
   - What functionality was unavailable?
   - For how long?
   - Was there a complete outage or degradation?
   - Could users work around it?

3. **Business Impact**
   - Revenue impact (lost transactions)?
   - Customer impact (complaints, churn risk)?
   - Reputational impact (media coverage)?
   - Competitive impact (customers switch)?

4. **Compliance Impact**
   - Regulatory reporting required?
   - Audit findings?
   - Certification impact (SOC2, ISO)?

### Financial Impact Assessment

| Category | Calculation | Amount |
|---|---|---|
| **Response Costs** | 8 engineers × 2 hours × $550/hr average | $8,800 |
| **Notification** | HIPAA breach letters for 47 patients × $70/letter | $3,290 |
| **Lost Revenue** | Users unable to use service for 2.25 hours | ~$5,000 (estimate) |
| **Legal Review** | General counsel review hours | $2,000 |
| **Audit Costs** | If required audit of remediation | $10,000+ |
| **Total Estimated** | | ~$29,000 |

---

## Prevention and Action Items

### Categorizing Prevention Measures

**Immediate Actions** (within 48 hours)
- Emergency fixes deployed
- Monitoring alerts added
- Customer notifications sent
- Urgent security patches

**Short-term Measures** (1-4 weeks)
- Testing framework improvements
- Enhanced monitoring
- Documentation updates
- Code review process updates

**Medium-term Initiatives** (1-3 months)
- Architectural changes
- New tools or infrastructure
- Training programs
- Process redesign

**Long-term Improvements** (3+ months)
- Formal verification
- Major system redesigns
- Culture/process changes
- Industry partnerships (auditors)

### Writing Effective Action Items

**Structure**:
- **Title**: Clear, specific action
- **Owner**: Who is responsible
- **Due Date**: When should this be done
- **Priority**: Critical / High / Medium / Low
- **Acceptance Criteria**: How do we know it's done?
- **Resources**: Who/budget needed?
- **Impact**: Why does this matter?

**Examples**:

❌ **Poor**:
```
- Improve testing
- Update monitoring (no owner, no timeline, vague success criteria)
```

✅ **Good**:
```
- Add integration tests for event emission in medical_records contract
  Owner: Lisa Thompson (QA)
  Due: 2025-03-29
  Priority: High
  Acceptance Criteria:
    ☐ Every state-changing function has test verifying event emission
    ☐ Code coverage for revoke path ≥ 95%
    ☐ Test runs in CI/CD pipeline and blocks deployment if failed
  Resources: 40 hours QA engineer time
  Impact: Prevents future state-event synchronization bugs
```

### Tracking Action Items

**GitHub Issues**:
```
Title: [PostMortem] Add event emission tests to medical_records

Description:
As part of incident postmortem PM-2025-03-15-001, we need to add 
comprehensive integration tests to verify that all state-changing 
operations emit corresponding events.

Acceptance Criteria:
- [ ] Test added for every state-changing function (grant, revoke, update_record, etc)
- [ ] Test verifies event is emitted with correct parameters
- [ ] Test verifies off-chain indexer receives event
- [ ] Code coverage for critical paths ≥ 95%
- [ ] Tests run in CI/CD pipeline
- [ ] Deployment blocked if event emission test fails

Timeline: 2025-03-29
Owner: @lisa-thompson
```

**Project Board**:
- Track all action items in one place
- Use columns: Backlog → In Progress → Review → Done
- Update in weekly standups
- Review monthly for completion

---

## Lessons Learned Framework

### What Analysis Questions to Ask

**About things that went well:**
1. What helped us detect the incident so quickly?
2. What processes or practices prevented worse damage?
3. What team communications worked well?
4. What documentation was particularly helpful?
5. What tools were invaluable during response?

**About things that could improve:**
1. What was the first symptom we missed?
2. What would have prevented this?
3. Where was there ambiguity or uncertainty?
4. What did we have to figure out during response?
5. What process gaps exist?

**About assumptions that were wrong:**
1. What did we assume that proved incorrect?
2. What did we not realize we didn't know?
3. Where was our mental model incomplete?
4. What surprised us during response?

### Knowledge Transfer Methods

**Documentation Updates**:
- Update runbooks with lessons learned
- Update architecture documentation if design flaws found
- Create guides for common problems

**Team Training**:
- Session on incident and response
- Discussion of lessons learned
- Practice scenarios for similar situations
- Record session for team reference

**Process Changes**:
- Update checklists based on discovered gaps
- Modify code review process
- Change deployment procedures
- Update monitoring

**Tools and Automation**:
- Implement automated checks discovered during postmortem
- Update CI/CD pipeline with new validations
- Create or update monitoring alerts
- Build tools to prevent similar issues

---

## Team Training Checklist

### Required Training for All Team Members

- [ ] **Incident Response Basics** (2 hours)
  - When to escalate
  - Who to notify
  - Communication protocols
  - Decision-making hierarchy

- [ ] **Postmortem Practices** (1 hour)
  - Blamelessness principles
  - How to participate in postmortem
  - Timeline accuracy importance
  - Root cause analysis introduction

- [ ] **Event-Driven Design** (2 hours) - *Specific to this incident*
  - Soroban event emission patterns
  - Why events are important for off-chain sync
  - Testing event-driven systems
  - Debugging event-driven issues

### Recommended Training for Engineers

- [ ] **Root Cause Analysis Deep Dive** (2 hours)
  - 5 Whys technique
  - Fishbone diagram
  - Fault tree analysis
  - Common pitfalls

- [ ] **Contract Testing Strategies** (3 hours)
  - Unit vs. integration tests
  - Testing state-event consistency
  - Property-based testing
  - Fuzz testing for contracts

- [ ] **Monitoring and Alerting** (2 hours)
  - Effective alert design
  - Alert thresholds
  - Anomaly detection
  - Alert fatigue prevention

### Training Delivery

**Formats**:
1. **Live session** (interactive, questions welcome)
2. **Recorded video** (watch on your schedule)
3. **Written guide** (reference material)
4. **Hands-on lab** (practice exercises)
5. **Scenario-based exercise** (simulate incidents)

**Schedule**:
- Session 1: Week after incident (while fresh)
- Session 2: 2-3 weeks after (deeper dive)
- Session 3: 1 month after (process improvements)

**Tracking**:
- Completion recorded in training management system
- Required for all engineers within 30 days
- Annual refresher

---

## Common Mistakes to Avoid

### Mistakes in Analysis

**1. Jumping to Conclusions**
- ❌ Assuming the likeliest cause is correct
- ✅ Building complete timeline first, then analyzing

**2. Focusing on Symptoms vs. Root Causes**
- ❌ "The error was a null pointer exception"
- ✅ "The function didn't emit event when state changed, causing off-chain cache to become stale"

**3. Incomplete 5 Whys**
- ❌ Stopping at "We forgot to emit the event"
- ✅ Continuing to "Why was there no code review process to catch this?"

**4. Single Root Cause Bias**
- ❌ "It was a coding mistake"
- ✅ "Combination of: incomplete implementation, insufficient testing, code review gaps, high alert threshold"

### Mistakes in Process

**5. Blaming Individuals**
- ❌ "Engineer X failed to test properly"
- ✅ "Testing process doesn't verify event emission consistency"

**6. Action Items Too Vague**
- ❌ "Improve testing" (do what? how do we know it's done?)
- ✅ "Add integration test verifying event emission for each state-changing function. Coverage ≥95%. Due 3/29"

**7. No Follow-up on Action Items**
- ❌ Create action items in postmortem, never mention again
- ✅ Track in project board, review weekly, verify completion

**8. Postmortems Only for Big Incidents**
- ❌ Skip postmortem for "learning" incidents or near-misses
- ✅ Conduct postmortems for all significant incidents to build resilience

### Mistakes in Documentation

**9. Incomplete Timeline**
- ❌ Major time gaps, human memory-based ("about 10 minutes")
- ✅ Logs-based timeline with exact timestamps and context clues

**10. Evidence-Free Claims**
- ❌ "We believe there might have been unauthorized access"
- ✅ "Query of transaction logs from 14:32-16:47 showed X doctors accessed Y patient records during revocation period"

**11. Archived and Forgotten**
- ❌ Postmortem written, filed away, never referenced
- ✅ Postmortem linked to action items, regularly referenced, lessons incorporated into processes

---

## Examples and Case Studies

### Example 1: Simple Deployment Error

**Incident**: Contracts deployed with debug logging left enabled, causing gas limit exceeded errors

**Severity**: Medium

**Quick Analysis**:
- **What happened**: Build process compiled contracts with debug build (accidentally)
- **Why**: Build script defaulted to debug instead of release
- **Why not caught**: CI/CD didn't verify build profile
- **Prevention**: 
  - Enforce release builds in CI pipeline
  - Add build artifact verification
  - Update deployment checklist

**Action Items**:
1. Update build pipeline to enforce release builds
2. Add smoke test after deployment verifying gas usage
3. Document build procedures

### Example 2: Data Inconsistency Issue

**Incident**: Medical records contract state diverged from indexer state over time

**Root Causes**:
1. Event emission logic incomplete (case A: missing entire event type)
2. Event filtering in indexer misconfigured (case B: filtering out valid events)
3. Time sync issues between contract and indexer (case C: clock skew on different servers)

**Prevention**:
- Comprehensive event emission testing
- Indexer filter validation before deployment
- Clock synchronization monitoring

### Example 3: Capacity/Scale Incident

**Incident**: Contract performance degraded during high load (peak patient data access)

**Root Cause**: Algorithm complexity was O(n²) instead of O(1), unnoticeable in testing but critical at scale

**Prevention**:
- Load testing with realistic data volumes
- Automated performance regression checking
- Algorithm complexity review in code review

---

## Template Usage Instructions

### How to Use INCIDENT_POSTMORTEM_TEMPLATE.md

1. **Copy the template**
   ```bash
   cp docs/INCIDENT_POSTMORTEM_TEMPLATE.md \
      docs/incidents/INCIDENT_POSTMORTEM_PM-YYYY-MM-DD-###.md
   ```

2. **Replace all placeholders**
   - Search for `[...]` and replace with actual details
   - Search for `YYYY-MM-DD` and replace with dates
   - Replace section examples with your incident data

3. **Complete each section**
   - **Summary**: 2-3 sentences about what happened
   - **Timeline**: Build from logs, interview participants
   - **Root Cause Analysis**: Run 5 Whys, capture evidence
   - **Impact**: Quantify affected users, data, time
   - **Prevention**: Real action items, specific owners/dates
   - **Lessons Learned**: What went well, what to improve

4. **Review and iterate**
   - Incident commander reviews
   - Tech lead validates technical accuracy
   - Security reviews if applicable
   - Loop until complete

5. **Finalize and distribute**
   - Mark as Finalized in header
   - Create GitHub issues for action items
   - Share with team and stakeholders
   - Update project board

### When Customizing the Template

You can customize the template for different incident types:

**For Security Incidents**: Add section for "Security Implications" and "Compliance Reporting"

**For Performance Incidents**: Add section for "Performance Metrics" and "Capacity Planning"

**For Infrastructure Incidents**: Add section for "Infrastructure Changes"

**For Deployments**: Add section for "Deployment Process Review"

Keep the core structure (Summary, Timeline, RCA, Impact, Prevention, Actions) consistent across all postmortems.

---

## Additional Resources

### Internal Resources
- [Incident Response Plan](./INCIDENT_RESPONSE.md) - Overall incident handling procedures
- [Change Management Process](./docs/DEPLOYMENT_PROCESS.md) - How deployments are managed
- [Monitoring Guide](./docs/MONITORING.md) - How to set up and respond to alerts

### External Resources
- [Google: Postmortem Culture](https://sre.google/docs/postmortems/): Leading resource on blameless postmortems
- [PagerDuty: Incident Response](https://response.pagerduty.com/): Incident management best practices
- [AWS: Operational Excellence](https://docs.aws.amazon.com/wellarchitected/latest/operational-excellence-pillar/):Cloud infrastructure response practices

### Tools and Templates
- **GitHub Issues**: Track action items and status
- **Project Boards**: Visualize progress on prevention measures
- **Slack**: Incident communication and updates
- **Monitoring Dashboards**: Incident detection

---

## Questions?

If you have questions about postmortems or how to conduct one:

1. **Review the example postmortem** (docs/examples/INCIDENT_POSTMORTEM_EXAMPLE.md)
2. **Check this guide** for specific techniques or sections
3. **Ask the team** in Slack (#incidents channel)
4. **Consult your Incident Commander** for incident-specific guidance

---

**Document Version**: 1.0  
**Last Updated**: 2025-03-17  
**Next Review**: 2025-06-17 (quarterly)  
**Questions/Feedback**: Platform Team Slack (#incidents)
