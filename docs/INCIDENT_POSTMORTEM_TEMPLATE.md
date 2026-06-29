# Contract Incident Postmortem Template

## Document Information
- **Postmortem ID**: `PM-YYYY-MM-DD-###` (e.g., PM-2024-01-15-001)
- **Date of Incident**: YYYY-MM-DD HH:MM UTC
- **Date of Postmortem**: YYYY-MM-DD
- **Postmortem Lead**: [Name, Role]
- **Participants**: [Team members involved]
- **Status**: Draft / In Review / Finalized

---

## 1. Incident Summary

### Overview
**[Concise 2-3 sentence description of what happened]**

### Severity Level
- [ ] Critical (Contract failure, data loss, security breach)
- [ ] High (Significant service degradation, user impact)
- [ ] Medium (Limited impact, recoverable without intervention)
- [ ] Low (Minor issue, no operational impact)

### Contract(s) Affected
- Contract Name: [medical_records / identity_registry / etc]
- Contract ID: [C...]
- Network: [local / testnet / futurenet / mainnet]

### User/System Impact
- **Users Affected**: [Number or list of affected users]
- **Data Affected**: [Types of records/data impacted]
- **Service Downtime**: [Duration]
- **Business Impact**: [Financial, reputation, compliance implications]

### Key Metrics
- **Detection Time**: [When noticed]
- **Response Time**: [Time to acknowledge incident]
- **Resolution Time**: [Time to restore functionality]
- **Time to Recovery**: [Total incident duration]

---

## 2. Timeline of Events

### Detailed Chronological Log

| Time (UTC) | Event | Owner | Details |
|---|---|---|---|
| HH:MM | [Event Type] | [Person] | [Description of what happened] |
| HH:MM | Incident Detected | [Person] | [How was it discovered? Monitoring alert? User report?] |
| HH:MM | Incident Acknowledged | [Person] | [Initial response action taken] |
| HH:MM | [Action] | [Person] | [Investigation steps, findings, commands run] |
| HH:MM | [Action] | [Person] | [This section should be detailed and comprehensive] |
| HH:MM | Issue Resolved | [Person] | [Resolution confirmed] |
| HH:MM | Incident Closed | [Person] | [Post-incident cleanup completed] |

### Key Decision Points

#### Decision 1: [Decision Description]
- **Time**: HH:MM UTC
- **Decision Maker**: [Name]
- **Rationale**: [Why this decision was made]
- **Impact**: [What was the outcome]

#### Decision 2: [Decision Description]
- **Time**: HH:MM UTC
- **Decision Maker**: [Name]
- **Rationale**: [Why this decision was made]
- **Impact**: [What was the outcome]

---

## 3. Root Cause Analysis

### Primary Root Cause

**[Clearly identify the main underlying cause, not just the symptom]**

### Contributing Factors

#### Factor 1: [Description]
- **Type**: [Code bug / Configuration / Deployment / Dependency / Infrastructure / Process / Documentation]
- **Severity**: [Critical / High / Medium / Low]
- **Evidence**: [Specific logs, code snippets, or data that shows this factor]

#### Factor 2: [Description]
- **Type**: [Code bug / Configuration / Deployment / Dependency / Infrastructure / Process / Documentation]
- **Severity**: [Critical / High / Medium / Low]
- **Evidence**: [Specific logs, code snippets, or data that shows this factor]

### Why Wasn't This Caught Earlier?

1. **Insufficient Testing**: [What tests were missing? Unit/integration/load tests?]
2. **Monitoring Gap**: [What metrics/alerts should have caught this?]
3. **Process Gap**: [What review or verification step failed?]
4. **Documentation**: [Was relevant information unavailable?]

### Technical Deep Dive

#### Affected Code/Component
```rust
// Example: Include relevant code snippet
// Path: contracts/medical_records/src/lib.rs
pub fn write_record(
    env: &Env,
    patient_id: String,
    data: Bytes
) -> Result<(), Error> {
    // This function had an issue with...
}
```

#### Issue Description
[Detailed technical explanation of what went wrong]

#### Reproduction Steps
1. [Step 1]
2. [Step 2]
3. [Step 3]

---

## 4. Impact Assessment

### Direct Impact

#### Data Impact
- **Records Affected**: [Number and type of records]
- **Data Integrity**: [Was data corrupted? Lost?]
- **Data Confidentiality**: [Was data exposed?]
- **HIPAA/Compliance Implications**: [Yes/No - if yes, regulatory reporting required]

#### User Impact
- **Affected User Groups**: [Patients / Doctors / Admins / etc]
- **Service Availability**: [% downtime, specific features unavailable]
- **User Experience**: [Error messages, functionality loss]

#### Financial Impact
- **Remediation Cost**: [Development, infrastructure, etc]
- **Notification/Legal**: [Regulatory notification costs if applicable]
- **Reputation**: [Customer trust impact]

### Indirect Impact

#### System Stability
- **Other Contracts Affected**: [List any cascading effects]
- **Network Effect**: [Did this impact other networks or nodes?]
- **Performance**: [System load, recovery resources needed]

#### Organizational Impact
- **Team Resources**: [Hours spent on mitigation]
- **Stakeholder Communication**: [Internal/external notices required]
- **Compliance Reporting**: [Regulatory obligations triggered]

---

## 5. Prevention Measures

### Immediate Actions (Already Taken)

#### Action 1: [Description]
- **Status**: Completed / In Progress
- **Owner**: [Name]
- **Completion Date**: YYYY-MM-DD
- **Details**: [How this prevents recurrence]

#### Action 2: [Description]
- **Status**: Completed / In Progress
- **Owner**: [Name]
- **Completion Date**: YYYY-MM-DD
- **Details**: [How this prevents recurrence]

### Short-term Preventive Measures (1-4 weeks)

#### Measure 1: Enhanced Test Coverage
- **Description**: Add unit/integration tests for [specific scenario]
- **Owner**: [Name]
- **Target Date**: YYYY-MM-DD
- **Acceptance Criteria**:
  - [ ] Test added to CI/CD pipeline
  - [ ] Test catches the original issue
  - [ ] Code coverage increased by X%

#### Measure 2: Monitoring Enhancement
- **Description**: Add alert for [specific metric/condition]
- **Owner**: [Name]
- **Target Date**: YYYY-MM-DD
- **Acceptance Criteria**:
  - [ ] Alert configured on all networks
  - [ ] Alert threshold validated
  - [ ] Runbook updated

#### Measure 3: Documentation Update
- **Description**: Update [specific documentation/runbook]
- **Owner**: [Name]
- **Target Date**: YYYY-MM-DD
- **Acceptance Criteria**:
  - [ ] Missing information added
  - [ ] Team reviewed and approved
  - [ ] Published to knowledge base

### Medium-term Preventive Measures (1-3 months)

#### Initiative 1: [Description]
- **Owner**: [Name]
- **Timeline**: YYYY-MM-DD to YYYY-MM-DD
- **Resources Required**: [Team, infrastructure, budget]
- **Expected Outcome**: [Specific improvement]

#### Initiative 2: [Description]
- **Owner**: [Name]
- **Timeline**: YYYY-MM-DD to YYYY-MM-DD
- **Resources Required**: [Team, infrastructure, budget]
- **Expected Outcome**: [Specific improvement]

### Long-term Improvements (3+ months)

#### Improvement 1: [Description]
- **Owner**: [Name]
- **Timeline**: YYYY-MM-DD to YYYY-MM-DD
- **Resources Required**: [Team, infrastructure, budget]
- **Expected ROI**: [Benefits]
- **Priority**: [Critical / High / Medium / Low]

#### Improvement 2: [Description]
- **Owner**: [Name]
- **Timeline**: YYYY-MM-DD to YYYY-MM-DD
- **Resources Required**: [Team, infrastructure, budget]
- **Expected ROI**: [Benefits]
- **Priority**: [Critical / High / Medium / Low]

---

## 6. Action Items

### Tracking Information
- All action items must be tracked in [GitHub Issues / Project Board / etc]
- Target completion tracked with assigned owners
- Status updates required weekly

### Immediate Actions (Due within 48 hours)

| ID | Action Item | Owner | Due Date | Priority | Status |
|---|---|---|---|---|---|
| A1 | [Action] | [Name] | YYYY-MM-DD | Critical | Not Started |
| A2 | [Action] | [Name] | YYYY-MM-DD | Critical | In Progress |

### Follow-up Actions (Due within 2 weeks)

| ID | Action Item | Owner | Due Date | Priority | Status |
|---|---|---|---|---|---|
| A3 | [Action] | [Name] | YYYY-MM-DD | High | Not Started |
| A4 | [Action] | [Name] | YYYY-MM-DD | High | Not Started |

### Longer-term Actions (Due within 1 month)

| ID | Action Item | Owner | Due Date | Priority | Status |
|---|---|---|---|---|---|
| A5 | [Action] | [Name] | YYYY-MM-DD | Medium | Not Started |
| A6 | [Action] | [Name] | YYYY-MM-DD | Medium | Not Started |

### Tracking
- **Review Schedule**: Weekly stand-up / [Cadence]
- **Escalation Process**: [Who to notify if delayed]
- **Acceptance Criteria**: Action item linked to GitHub issue with specific acceptance criteria

---

## 7. Lessons Learned

### What Went Well

1. [Positive aspect of response]
2. [Positive aspect of response]
3. [Positive aspect of response]

### What Could Be Improved

1. [Area for improvement]
   - **Current State**: [How it's done now]
   - **Desired State**: [How it should be done]
   - **Gap**: [What's missing]

2. [Area for improvement]
   - **Current State**: [How it's done now]
   - **Desired State**: [How it should be done]
   - **Gap**: [What's missing]

### Knowledge Transfer

- **Documentation Updates**: [List of docs that need updating]
- **Training Topics**: [What should the team learn?]
- **Team Discussions**: [What should be discussed in team syncs?]

---

## 8. Appendices

### A. Supporting Documentation

- **Incident Ticket**: [Link to GitHub issue]
- **Related Code**: [Links to relevant files/commits]
- **Monitoring Dashboards**: [Links to metrics/dashboards]
- **Configuration**: [Relevant config files]
- **Architecture Diagrams**: [Links or embedded diagrams]

### B. Log Excerpts

#### Error Log
```
[Date Time] ERROR: [Error message]
[Stack trace or relevant context]
```

#### System Log
```
[Relevant system events]
```

### C. Related Processes

- **Incident Response Process**: See [INCIDENT_RESPONSE.md](./docs)
- **Change Management**: See [CHANGE_MANAGEMENT.md](./docs)
- **Incident Classification**: [Link to classification scheme]
- **Severity Definitions**: [Link to severity levels]

### D. Contact Information

| Role | Name | Email | Phone |
|---|---|---|---|
| Incident Commander | [Name] | [email] | [phone] |
| Contract Lead | [Name] | [email] | [phone] |
| DevOps Lead | [Name] | [email] | [phone] |
| Security Lead | [Name] | [email] | [phone] |

---

## Sign-off

### Postmortem Review

- **Reviewed By**: [Name, Title]
- **Date**: YYYY-MM-DD
- **Approved**: Yes / No
- **Comments**: [Any additional feedback]

### Stakeholder Notification

- [ ] Team notified
- [ ] Stakeholders notified
- [ ] Customers notified (if applicable)
- [ ] Compliance/Legal notified (if applicable)

---

**Document Version**: 1.0  
**Last Updated**: YYYY-MM-DD  
**Next Review Date**: YYYY-MM-DD
