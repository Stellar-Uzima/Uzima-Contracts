# Security Incident Response Procedures

This document defines the procedures for detecting, responding to, and recovering from security incidents in the Uzima Contracts system.

---

## Severity Classification

| Severity | Description | Response Time |
|----------|-------------|---------------|
| **P0 – Critical** | Active exploit, data breach, contract funds at risk | Immediate (< 1 hour) |
| **P1 – High** | Vulnerability confirmed, no active exploit yet | < 4 hours |
| **P2 – Medium** | Potential vulnerability, limited impact | < 24 hours |
| **P3 – Low** | Minor issue, no immediate risk | < 72 hours |

---

## 1. Vulnerability Disclosure Process

### Reporting a Vulnerability

- **Private disclosure**: Email `security@uzima.health` with subject `[SECURITY] <brief description>`
- **GitHub**: Use [GitHub Security Advisories](https://github.com/Stellar-Uzima/Uzima-Contracts/security/advisories/new) (do **not** open a public issue)
- Include: affected contract(s), reproduction steps, potential impact, and suggested fix if known

### Acknowledgement & Triage

1. Acknowledge receipt within **24 hours**
2. Assign severity (P0–P3) within **4 hours** of acknowledgement
3. Notify reporter of severity and expected timeline
4. Open a private tracking issue with label `security`

---

## 2. Emergency Response Plan

### P0 / P1 Incident Steps

```
1. DETECT   → Alert fires or report received
2. TRIAGE   → Confirm severity, identify affected contracts/networks
3. CONTAIN  → Pause affected contract functions if possible (emergency_access_override)
4. ASSESS   → Determine scope: funds at risk? data exposed? which users?
5. PATCH    → Develop and test fix on local/testnet
6. DEPLOY   → Deploy patch via deploy workflow (requires 2-person approval for mainnet)
7. VERIFY   → Confirm fix resolves issue; monitor for 24 hours
8. DISCLOSE → Publish advisory after patch is live
9. POSTMORTEM → Complete postmortem within 5 business days (see INCIDENT_POSTMORTEM_TEMPLATE.md)
```

### Incident Commander Responsibilities

- Declare incident severity and own the response timeline
- Coordinate communication between engineering, security, and stakeholders
- Authorize emergency contract pauses or rollbacks
- Sign off on patch deployment to mainnet

### On-Call Rotation

Maintain a documented on-call schedule. The on-call engineer is the first responder for P0/P1 alerts and must be reachable within 30 minutes.

---

## 3. Communication Templates

### Internal Alert (Slack / Email)

```
🚨 SECURITY INCIDENT – [P0/P1/P2/P3]
Contract(s): <name>
Network: <testnet/mainnet>
Summary: <one sentence>
Impact: <what is at risk>
Incident Commander: <name>
Tracking: <private issue link>
Next update: <time>
```

### External Disclosure (after patch)

```
Subject: Security Advisory – Uzima Contracts [CVE/GHSA ID]

We have identified and patched a [severity] vulnerability in [contract name].

Affected versions: <range>
Fixed in: <version/commit>
Impact: <description>
Recommended action: <upgrade instructions>

Full details: <advisory link>
```

### User Notification (if data or funds affected)

```
Subject: Important Security Notice – Action Required

We are contacting you because a security issue may have affected your account.

What happened: <brief description>
What data/funds were affected: <specifics>
What we have done: <remediation steps>
What you should do: <user action items>

Contact: security@uzima.health
```

---

## 4. Post-Incident Analysis Process

After every P0/P1 incident, and optionally for P2/P3:

1. **Schedule postmortem** within 48 hours of resolution
2. **Complete postmortem document** using `docs/INCIDENT_POSTMORTEM_TEMPLATE.md`
3. **Identify root cause** (not blame) and contributing factors
4. **Define action items** with owners and due dates
5. **Publish postmortem** internally within 5 business days
6. **Track action items** to completion in GitHub issues

Postmortem examples: `docs/examples/INCIDENT_POSTMORTEM_EXAMPLE.md`

---

## 5. Patch and Deployment Procedures

### Emergency Patch Process

1. Create a private branch: `security/fix-<short-description>`
2. Develop fix with tests covering the vulnerability
3. Review: minimum 2 engineers must approve (one must be a maintainer)
4. Deploy to testnet and verify fix
5. For mainnet: use `deploy.yml` workflow with `confirm_mainnet: DEPLOY`
6. Tag release with patch version (e.g., `v1.2.1`)
7. Publish GitHub Security Advisory

### Contract Pause (Emergency)

If a contract must be paused immediately:

```bash
# Via emergency_access_override contract
./scripts/interact.sh <CONTRACT_ID> mainnet emergency_pause \
  --reason "Security incident <ID>" \
  --duration 3600
```

### Rollback

If a patch introduces regressions:

```bash
./scripts/rollback_deployment.sh <contract_name> mainnet <backup_file>
```

See `docs/EMERGENCY_PLAYBOOKS.md` for detailed runbooks.

---

## 6. Tools and References

| Resource | Location |
|----------|----------|
| Postmortem template | `docs/INCIDENT_POSTMORTEM_TEMPLATE.md` |
| Emergency playbooks | `docs/EMERGENCY_PLAYBOOKS.md` |
| Security controls mapping | `docs/SECURITY_CONTROLS_MAPPING.md` |
| Threat models | `docs/MASTER_THREAT_MODEL.md` |
| Weekly security scan | `.github/workflows/weekly-security-report.yml` |
| Security scan script | `scripts/security-scan.sh` |

---

*Maintained by the Uzima security team. Review and update this document after every P0/P1 incident.*
