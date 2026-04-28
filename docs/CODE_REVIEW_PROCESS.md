# Code Review Process

This document defines the formal code review process for all changes to Uzima-Contracts.

## Review Criteria

All pull requests must satisfy the following before approval:

- **Correctness**: Logic is sound and handles edge cases
- **Security**: No vulnerabilities introduced (auth, input validation, overflow)
- **Tests**: New code is covered by unit and/or integration tests
- **Documentation**: Public functions have doc comments; complex logic is explained
- **Style**: Passes `cargo fmt` and `cargo clippy -- -D warnings`
- **Performance**: No unnecessary storage reads/writes or gas-heavy patterns

## Reviewer Responsibilities

- Review within **2 business days** of assignment
- Leave actionable, specific comments (not vague requests)
- Distinguish blocking issues from suggestions using labels:
  - `[blocking]` — must be resolved before merge
  - `[nit]` — optional improvement
  - `[question]` — clarification needed, not necessarily blocking
- Approve only when all blocking issues are resolved
- Re-review within 1 business day after author addresses feedback

## Approval Process

| Change Type | Required Approvals |
|---|---|
| Bug fix / docs | 1 maintainer |
| New feature | 2 maintainers |
| Security-sensitive change | 2 maintainers + security review |
| Breaking change | 2 maintainers + issue discussion |

A PR may not be merged by its own author unless it has the required approvals.

## Escalation Procedures

1. **Reviewer unavailable**: Reassign to another maintainer after 2 business days with no response.
2. **Disagreement between reviewers**: Author opens a discussion thread; maintainers reach consensus or defer to project lead.
3. **Security concern**: Immediately label `security` and notify the security team. Do not merge until resolved.
4. **Urgent hotfix**: Requires at least 1 approval; second approval may follow within 24 hours post-merge.

## Common Issues Checklist

Before requesting review, verify:

- [ ] `cargo fmt --all` passes
- [ ] `cargo clippy --all-targets --all-features -- -D warnings` passes
- [ ] `cargo test --all` passes locally
- [ ] No hardcoded secrets or private keys
- [ ] Storage keys use the `DataKey` enum pattern
- [ ] Error types use `#[contracterror]` where applicable
- [ ] Events are emitted for state-changing operations
- [ ] Authorization checks are present on all privileged functions
- [ ] Integer arithmetic uses `saturating_*` or `checked_*` where overflow is possible
- [ ] PR description references the relevant issue(s)

## Timeline Expectations

| Stage | Target |
|---|---|
| First review | ≤ 2 business days |
| Follow-up review | ≤ 1 business day |
| Merge after final approval | Same day |
| Stale PR (no activity 7 days) | Labeled `stale`; closed after 14 days |

## GitHub Automation

The following automations are configured:

- **CI required**: All status checks in `.github/workflows/ci.yml` must pass before merge is allowed.
- **Branch protection**: Direct pushes to `main` are blocked; PRs are required.
- **Auto-assign**: PRs are auto-assigned to the on-call reviewer via CODEOWNERS.
- **Stale bot**: PRs with no activity for 7 days are labeled `stale` and closed after 14 days.

### CODEOWNERS

```
# Default reviewers for all files
*                   @Stellar-Uzima/maintainers

# Contract source code requires extra review
/contracts/         @Stellar-Uzima/maintainers @Stellar-Uzima/security
/.github/workflows/ @Stellar-Uzima/maintainers
```

## Reviewer Training Materials

New reviewers should:

1. Read [Soroban Security Best Practices](https://soroban.stellar.org/docs/learn/security)
2. Review the [Threat Model](./MASTER_THREAT_MODEL.md)
3. Shadow an experienced reviewer on 2–3 PRs before solo reviewing
4. Complete the internal security checklist walkthrough with a maintainer
