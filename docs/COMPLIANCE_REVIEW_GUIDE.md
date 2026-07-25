# Uzima Compliance Review Guide

This guide describes how to perform compliance reviews using the Uzima audit trail export tools.

## Quick Start

```bash
# Export audit trail for the last 30 days (JSON + CSV)
./scripts/export_audit_trail.sh --network mainnet --start-date 2026-06-25

# Export only specific contract events
./scripts/export_audit_trail.sh --contract <contract_id> --format json

# Preview export without writing files
./scripts/export_audit_trail.sh --dry-run
```

## Compliance Frameworks

| Framework | Relevant Sections | Retention Requirement |
|-----------|------------------|----------------------|
| HIPAA | §164.312(b), §164.308(a)(1)(ii)(D) | 6 years minimum |
| SOC 2 | CC6.1, CC7.2 | Duration of service + 1 year |
| GDPR | Art. 5(1)(e), Art. 7(1) | Duration of processing + 6 years |
| Internal | Uzima Policy | 7 years (2555 days) |

## Audit Event Types

| Event Type | Description | Risk Level |
|-----------|-------------|-----------|
| `record_accessed` | Medical record was read | Medium |
| `record_created` | New medical record created | Low |
| `record_updated` | Medical record modified | High |
| `access_granted` | Permission to access record granted | High |
| `access_requested` | Request to access record submitted | Medium |
| `emergency_access_granted` | Emergency override activated | Critical |
| `user_role_updated` | User permissions changed | High |
| `contract_paused` | Contract operation paused | Critical |

## Compliance Review Checklist

### Pre-Review

- [ ] Verify export covers the required date range
- [ ] Confirm all event types are included
- [ ] Check retention policy compliance

### During Review

- [ ] Review emergency access events first (highest risk)
- [ ] Check for unauthorized access patterns
- [ ] Verify consent events match data processing activities
- [ ] Review role changes for least-privilege compliance
- [ ] Check contract pause/unpause events for operational integrity

### Post-Review

- [ ] Document findings in compliance report
- [ ] Archive export artifacts per retention policy
- [ ] Create action items for any compliance gaps
- [ ] Schedule follow-up review if needed

## Troubleshooting

| Issue | Solution |
|-------|----------|
| Export fails with network error | Check `--network` flag matches your deployment |
| Missing event types | Verify contract emits expected events (check `docs/EVENTS.md`) |
| Date range too large | Break into smaller chunks (max 1 year recommended) |
