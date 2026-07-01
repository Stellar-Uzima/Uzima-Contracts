# ADR-003: Timelock Duration for Governance Execution

**Status:** Accepted  
**Date:** 2025-06-29

## Context

The governance system uses a `timelock` contract that delays execution of approved proposals by a fixed period. This delay gives stakeholders (patients, healthcare providers, administrators) a window to detect malicious or erroneous proposals and take protective action before they are enacted on-chain.

Candidates evaluated:
- **0 delay** — immediate execution after approval (no safety window)
- **24 hours** — minimal delay, reduces operational friction
- **48 hours** — moderate delay, balances safety and speed
- **7 days** — maximum safety, standard in DeFi governance

Healthcare-specific considerations differ from DeFi: the primary threat model is insider attack or key compromise, not large token-holder manipulation. Patients and providers need enough time to notice and report, but excessive delays block urgent contract fixes during active security incidents.

## Decision

Use a **48-hour timelock** (172,800 seconds) as the default delay for all governance proposal execution, with a configurable `emergency_delay` of **4 hours** reserved exclusively for proposals tagged as `EMERGENCY` by a supermajority (>66%) of the governance council.

## Rationale

- **48 hours** is sufficient for all stakeholders (patients, providers, admins) to receive notifications and review pending changes through the governance portal.
- The **4-hour emergency path** allows rapid response to critical security vulnerabilities without eliminating all oversight — supermajority requirement prevents abuse.
- 48 hours aligns with HIPAA breach response expectations, making it a familiar timeframe for healthcare compliance teams.
- A 7-day delay was rejected because it would block timely security patches in a live healthcare system where data breaches have regulatory deadlines.

## Consequences

- All standard governance proposals will take at minimum 48 hours to execute after approval.
- An emergency governance mechanism requires a separate, higher approval threshold.
- The timelock duration is stored as a configurable contract parameter; changes to the duration itself require a standard governance proposal (48-hour delay applies).
- Monitoring must alert on any queued proposals within the timelock window.
