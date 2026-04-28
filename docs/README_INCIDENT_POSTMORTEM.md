# Incident Postmortem Resources

This directory contains comprehensive resources for conducting incident postmortems in the Uzima-Contracts project. These resources ensure we learn from incidents, prevent recurrence, and maintain a blameless, learning-focused culture.

---

## 📋 Quick Reference

| Resource | Purpose | When to Use |
|----------|---------|------------|
| [INCIDENT_POSTMORTEM_TEMPLATE.md](./INCIDENT_POSTMORTEM_TEMPLATE.md) | Complete postmortem template with all sections | Create a new postmortem after any critical incident |
| [INCIDENT_POSTMORTEM_GUIDELINES.md](./INCIDENT_POSTMORTEM_GUIDELINES.md) | Training, best practices, and guidance | Learn how to conduct an effective postmortem |
| [examples/INCIDENT_POSTMORTEM_EXAMPLE.md](./examples/INCIDENT_POSTMORTEM_EXAMPLE.md) | Real-world example postmortem | Reference for format and level of detail |

---

## 🚀 Getting Started

### I Need to Create a Postmortem (Incident Just Happened)

1. **Declare the incident resolved** and notify stakeholders
2. **Collect evidence** immediately (logs, screenshots, deployment info)
3. **Schedule postmortem** for 2-5 days out (while fresh, but not during crisis)
4. **Copy the template**: 
   ```bash
   cp docs/INCIDENT_POSTMORTEM_TEMPLATE.md \
      docs/incidents/INCIDENT_POSTMORTEM_PM-2025-03-20-001.md
   ```
5. **Conduct postmortem session** (60-90 minutes) with all responders
6. **Complete documentation** following the template sections
7. **Create action items** as GitHub issues
8. **Track progress** in project board

### I Need to Understand Postmoortems (New Team Member)

1. **Read this README** (you're reading it!)
2. **Skim INCIDENT_POSTMORTEM_GUIDELINES.md** sections:
   - [What is an Incident Postmortem?](./INCIDENT_POSTMORTEM_GUIDELINES.md#what-is-an-incident-postmortem)
   - [Postmortem Best Practices](./INCIDENT_POSTMORTEM_GUIDELINES.md#postmortem-best-practices)
   - [When to Conduct a Postmortem](./INCIDENT_POSTMORTEM_GUIDELINES.md#when-to-conduct-a-postmortem)

3. **Review the example** postmortem to see real-world format
4. **Take team training** when offered

### I'm Facilitating a Postmortem (I'm the Incident Commander)

1. **Before meeting**: Read [Postmortem Process](./INCIDENT_POSTMORTEM_GUIDELINES.md#postmortem-process) phase 1-2
2. **Run meeting**: Follow [Role Definitions](./INCIDENT_POSTMORTEM_GUIDELINES.md#role-definitions) and [Timeline Guidance](./INCIDENT_POSTMORTEM_GUIDELINES.md#timeline---detailed-guidance)
3. **After meeting**: Assign documentation owner to fill in [INCIDENT_POSTMORTEM_TEMPLATE.md](./INCIDENT_POSTMORTEM_TEMPLATE.md)
4. **Manager action items**: Use [Prevention and Action Items](./INCIDENT_POSTMORTEM_GUIDELINES.md#prevention-and-action-items) guidance
5. **Track to completion**: Use [project board](https://github.com/Stellar-Uzima/Uzima-Contracts/projects/3)

---

## 📖 Resource Details

### 1. INCIDENT_POSTMORTEM_TEMPLATE.md

**What it is**: A complete template for documenting incidents (8 sections)

**Sections**:
1. **Incident Summary** - Executive overview (severity, contracts, metrics)
2. **Timeline of Events** - Chronological log with timestamps
3. **Root Cause Analysis** - Five Whys, contributing factors, evidence
4. **Impact Assessment** - Data, user, financial, and organizational impact
5. **Prevention Measures** - Immediate, short-term, medium-term, long-term improvements
6. **Action Items** - Specific, measurable, owned, tracked tasks
7. **Lessons Learned** - What went well, what to improve, knowledge transfer
8. **Appendices** - Supporting docs, logs, contacts, sign-off

**How to use**:
- Copy to new file with naming pattern: `INCIDENT_POSTMORTEM_PM-YYYY-MM-DD-###.md`
- Replace all `[...]` placeholders with actual details
- Keep formality and detail level consistent throughout
- Reference the example for style guidance

**Typical length**: 5-15 pages, depending on incident complexity

---

### 2. INCIDENT_POSTMORTEM_GUIDELINES.md

**What it is**: Comprehensive training and reference guide (3,000+ words)

**Key sections**:

| Section | Purpose | Read When |
|---------|---------|-----------|
| Postmortem Best Practices | 8 core principles | Before first postmortem |
| Postmortem Process | 5 phases of postmortem | Implementing process |
| Role Definitions | Who does what | Assigning roles |
| Timeline Guidance | How to build accurate timelines | Before postmortem session |
| Root Cause Analysis | 5 Whys, fishbone, fault trees | Analyzing problems |
| Impact Assessment | How to quantify impact | Documenting impact |
| Prevention Measures | Short vs. long-term improvements | Planning improvements |
| Lessons Learned | Knowledge transfer framework | Team training |
| Common Mistakes | Anti-patterns to avoid | Before postmortem |

**How to use**:
- **First time**: Read sections relevant to your role
- **Facilitating**: Read process, role definitions, timeline guidance
- **Writing RCA**: Read root cause analysis techniques
- **Planning actions**: Read prevention measures section
- **Team training**: Use examples and exercises from lessons learned
- **Reference**: Search for specific technique when needed

---

### 3. INCIDENT_POSTMORTEM_EXAMPLE.md

**What it is**: Complete, realistic example postmortem from a fictional incident

**Incident details**:
- **What**: Medical records contract had access revocation bug
- **When**: 2025-03-15 (fictional date)
- **Impact**: 47 test patients, 2 hours downtime, HIPAA notification required
- **Root Cause**: Missing event emission in revoke function
- **Lessons**: Code review gaps, testing gaps, monitoring gaps

**Resource ID**: `PM-2025-03-15-001`

**How to use**:
- **Format reference**: See how to structure each section
- **Level of detail**: Match the detail level shown
- **Evidence format**: Note how logs and transaction data are included
- **Action items**: See realistic examples of short/medium/long-term items
- **Timeline**: Study how detailed timeline is built from logs
- **RCA**: See 5 Whys applied to real scenario

**Note**: This is a fictional example for training purposes. Real incident details will vary.

---

## 🔄 Postmortem Workflow

```
INCIDENT OCCURS
        ↓
INCIDENT RESPONSE (separate from postmortem)
        ↓
INCIDENT RESOLVED
        ↓
1. DECLARE RESOLUTION & COLLECT EVIDENCE (Incident Commander, 2 hours)
   - Save logs, deployments, monitoring data
   - Identify participants & schedule postmortem
   - Share preparation materials
        ↓
2. POSTMORTEM SESSION (Team, 60-90 minutes)
   - Build timeline from logs
   - Assess impact
   - Initial RCA
   - Preliminary action items
        ↓
3. DOCUMENTATION (Postmortem Lead, 2-7 days)
   - Fill INCIDENT_POSTMORTEM_TEMPLATE.md
   - Add evidence (logs, code, links)
   - Define specific action items
   - Route for review
        ↓
4. REVIEW & APPROVAL (Leadership, 1-2 days)
   - Verify accuracy & completeness
   - Approve action items & resources
   - Mark Finalized
        ↓
5. DISTRIBUTION & TRACKING (Ongoing)
   - Share postmortem with team
   - Create GitHub issues for actions
   - Track progress in project board
   - Weekly status updates
        ↓
6. CLOSEOUT (30-60 days after)
   - Verify all actions completed
   - Document any timeline changes
   - Archive postmortem
```

---

## ✅ Acceptance Criteria

To declare a postmortem complete, all of the following must be true:

### Document Quality
- [ ] All 8 sections of template completed
- [ ] Evidence linked (GitHub issues, code, logs, monitoring)
- [ ] Timeline: timestamps accurate and sourced from logs
- [ ] RCA: 5 Whys applied, root cause clearly identified
- [ ] Impact: specific numbers (not vague "many users")
- [ ] Prevention: concrete action items, not suggestions
- [ ] Action items: specific owner, date, acceptance criteria

### Process
- [ ] Postmortem session held with all key responders
- [ ] Blameless tone maintained throughout
- [ ] Different perspectives represented (eng, ops, security, QA)
- [ ] Evidence reviewed and cross-referenced
- [ ] Document reviewed by incident commander
- [ ] Leadership approved action items

### Stakeholder Notification
- [ ] Team notified of incident and response
- [ ] Stakeholders notified of postmortem findings
- [ ] Customers notified (if public/testnet incident)
- [ ] Compliance notified (if regulatory implications)

### Action Items
- [ ] All action items created as GitHub issues
- [ ] Issues linked to postmortem document
- [ ] Action items added to project board
- [ ] Owners assigned and accepted
- [ ] Timelines are realistic

### Knowledge Transfer
- [ ] Relevant documentation updated
- [ ] Team training scheduled
- [ ] Lessons captured for future reference
- [ ] Related processes updated

---

## 📊 Metrics and Success

### We know postmortems are effective when:

1. **Same type of incident doesn't happen twice**
2. **Team learns from other teams' incidents**
3. **Action items are completed in committed timeframes**
4. **Prevention measures reduce incident rate**
5. **Culture shifts toward blamelessness and learning**

### Track These Metrics

| Metric | Target | How to Track |
|--------|--------|--------------|
| Postmortem completion time | < 2 weeks after incident | GitHub project board |
| Action item completion rate | > 90% | GitHub project board |
| Recurrence rate | < 5% of same incident type | Stored postmortems |
| Time to detection | < 1 hour | Postmortem timeline |
| Time to resolution | < 2 hours (target) | Postmortem timeline |
| Team participation in postmortem | 100% of involved staff | Attendance tracking |

---

## 🎓 Team Training

### Required Training

All engineering team members must complete:

1. **Incident Response Basics** (2 hours)
   - Provided: [Postmortem Best Practices](./INCIDENT_POSTMORTEM_GUIDELINES.md#postmortem-best-practices)
   - Frequency: Annual (onboarding for new members)

2. **Postmortem Participation** (1 hour)
   - Provided: [Postmortem Process](./INCIDENT_POSTMORTEM_GUIDELINES.md#postmortem-process)
   - Frequency: Annual (after first incident response)

3. **Incident-Specific Training**
   - Based on: [Lessons Learned Framework](./INCIDENT_POSTMORTEM_GUIDELINES.md#lessons-learned-framework)
   - Schedule: 1-2 weeks after postmortem for critical incidents
   - Delivery: Live session, recorded, written guide

### Recommended Training

Specialized roles should take additional training:

- **Engineering Lead**: [Root Cause Analysis Techniques](./INCIDENT_POSTMORTEM_GUIDELINES.md#root-cause-analysis-techniques)
- **Incident Commander**: Full [Postmortem Process](./INCIDENT_POSTMORTEM_GUIDELINES.md#postmortem-process)
- **DevOps/Infra**: [Timeline Guidance](./INCIDENT_POSTMORTEM_GUIDELINES.md#timeline---detailed-guidance)
- **QA**: [Prevention Measures](./INCIDENT_POSTMORTEM_GUIDELINES.md#prevention-and-action-items)

---

## 🚨 When Things Go Wrong

### If Postmortem Isn't Blameless

**Signs**: Language like "Engineer failed", "Human error", "Careless mistake"

**Fix**: 
1. Pause and reframe
2. Focus on systems, not people
3. Ask "Why was there no process to catch this?"
4. Trace to systematic root cause

**Reference**: [Postmortem Best Practices #1](./INCIDENT_POSTMORTEM_GUIDELINES.md#1-be-blameless)

### If RCA Stops Too Early

**Signs**: "Root cause was a bug" or "Lack of testing"

**Fix**:
1. Ask "Why did this bug exist?"
2. Ask "Why wasn't this caught in testing?"
3. Continue 5 Whys until you reach systematic cause
4. Reference: [5 Whys Technique](./INCIDENT_POSTMORTEM_GUIDELINES.md#the-5-whys)

### If Action Items Aren't Specific

**Signs**: "Improve testing", "Better monitoring", no owner/date

**Fix**: 
1. Make specific (what exactly?)
2. Add owner and date
3. Add acceptance criteria
4. Reference: [Writing Effective Action Items](./INCIDENT_POSTMORTEM_GUIDELINES.md#writing-effective-action-items)

### If Postmortem Gets Archived and Forgotten

**Signs**: No mention of action items 2 weeks later

**Fix**:
1. Create GitHub issues for all action items
2. Add to project board
3. Track in weekly standups
4. Review for completion monthly
5. Reference: [Tracking Action Items](./INCIDENT_POSTMORTEM_GUIDELINES.md#tracking-action-items)

---

## 📚 Additional Resources

### Internal Documentation
- [Incident Response Plan](./INCIDENT_RESPONSE.md) - Immediate response procedures
- [Deployment Process](./docs/DEPLOYMENT_PROCESS.md) - How changes are deployed
- [Monitoring Guide](./docs/MONITORING.md) - Setting up alerts
- [Developer Guide](./docs/DEVELOPER_GUIDE.md) - Coding standards

### External References
- [Google SRE Book - Postmortems](https://sre.google/documents/postmortems/) - Industry standard
- [PagerDuty Incident Response](https://response.pagerduty.com/) - Incident management
- [O'Reilly - Blameless Postmortems](https://www.oreilly.com/library/) - Learning from failures

### Tools
- **GitHub Issues**: Track action items
- **Project Boards**: Visualize progress
- **Slack**: Incident communication
- **Google Docs**: Collaborative postmortem writing

---

## 👥 Key Contacts

| Role | Responsibility | Contact |
|------|---|---|
| **Incident Commander** | Coordinate postmortem | Dr. Sarah Chen |
| **Platform Lead** | Approve action items | Dr. Sarah Chen |
| **DevOps Lead** | Infrastructure aspects | Maria Patel |
| **Security Lead** | Security findings | Hassan Ahmed |
| **QA Lead** | Testing improvements | Lisa Thompson |

---

## 📝 Quick Checklist: Creating Your First Postmortem

- [ ] I have incident details (what, when, duration)
- [ ] I have names of responders
- [ ] I have access to logs and monitoring data
- [ ] I've read the template: [INCIDENT_POSTMORTEM_TEMPLATE.md](./INCIDENT_POSTMORTEM_TEMPLATE.md)
- [ ] I understand the 8 sections
- [ ] I've reviewed the example: [INCIDENT_POSTMORTEM_EXAMPLE.md](./examples/INCIDENT_POSTMORTEM_EXAMPLE.md)
- [ ] I've scheduled postmortem session (2-5 days out)
- [ ] I've prepared participants (shared materials, emphasized blamelessness)
- [ ] I'm ready to facilitate [Postmortem Process](./INCIDENT_POSTMORTEM_GUIDELINES.md#postmortem-process) phases 2-4

---

## ❓ FAQ

### How long should a postmortem take?
- **Session**: 60-90 minutes
- **Documentation**: 2-7 days
- **Action planning**: 1-2 weeks
- **Total process**: 2-3 weeks

### What if the incident was small/low-impact?
Use a lighter-weight postmortem, but don't skip entirely. Smaller incidents can still reveal systemic issues.

### Do we need external people for postmortem?
Include: engineers involved + your team lead at minimum. Invite: operations, security, QA, product for broader perspective.

### What happens to postmortems after they're done?
They're stored in docs/incidents/, linked to action items, and referenced during team training on similar topics.

### Who should read postmortems?
- **Required**: Team that caused/responded to incident
- **Recommended**: Engineering team, leadership
- **Optional (if public)**: Customers, community

### How do we track action items?
GitHub issues with labels `postmortem` and `incident-<PM-ID>`. Add to project board. Review weekly.

---

## 📞 Questions or Feedback?

- **Slack**: #incidents channel
- **GitHub**: Create issue on this repo
- **Email**: platform-team@stellar-uzima.io
- **Review**: Next scheduled review 2025-06-17

---

**Document Version**: 1.0  
**Last Updated**: 2025-03-17  
**Maintained By**: Platform Engineering Team  
**Next Review**: 2025-06-17 (quarterly)
