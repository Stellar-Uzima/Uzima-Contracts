# Branch Protection & PR Policy for Uzima Contracts

This document defines the branch protection rules, pull request policies, and review requirements for the Stellar-Uzima/Uzima-Contracts repository.

## Branch Protection Rules

### `main` Branch

The `main` branch is the production-ready branch. All changes must go through pull requests.

| Rule | Requirement |
|------|------------|
| **Require pull request** | All changes must be submitted via PR |
| **Required approvals** | Minimum 1 approval from a maintainer |
| **Dismiss stale reviews** | Approvals are dismissed when new commits are pushed |
| **Require review from code owners** | PRs touching CODEOWNERS paths require owner review |
| **Require status checks** | CI must pass before merge |
| **Required status checks** | `build`, `type-check`, `contracts`, `lint-imports` |
| **Require branches to be up to date** | PR branch must be rebased on latest `main` |
| **Require signed commits** | GPG-signed commits required for merge |
| **Require linear history** | Squash or rebase merge only (no merge commits) |
| **Do not allow bypassing** | Even admins must follow the PR process |
| **Restrict force pushes** | Force pushes to `main` are prohibited |
| **Restrict deletions** | Branch deletion of `main` is prohibited |

### `staging` Branch (Optional)

For pre-production testing before merging to `main`.

| Rule | Requirement |
|------|------------|
| **Require pull request** | Yes |
| **Required approvals** | 1 approval |
| **Required status checks** | `build`, `type-check`, `contracts` |
| **Restrict force pushes** | Yes |

## Pull Request Requirements

### PR Template

All PRs must include the following sections:

```markdown
## Summary
Brief description of the change.

## Changes
- [ ] Contract logic changes
- [ ] Documentation updates
- [ ] Test additions/modifications
- [ ] Migration/upgrade considerations
- [ ] Breaking changes

## Issues Closed
Closes #<issue-number>

## Testing
- [ ] Unit tests pass
- [ ] Integration tests pass
- [ ] Manual testing performed (if applicable)

## Checklist
- [ ] Code follows repository conventions
- [ ] Self-reviewed the code
- [ ] Comments added for complex logic
- [ ] Documentation updated
- [ ] No breaking changes (or documented above)
```

### PR Labels

| Label | When to Apply |
|-------|--------------|
| `core-infra` | Changes to common_error, common_auth, rbac, audit, identity_registry, upgradeability |
| `security-sensitive` | Changes affecting auth, consent, encryption, or audit contracts |
| `multi-domain` | PR touches contracts from 2+ domains |
| `needs-security-review` | PR requires security review before merge |
| `breaking-change` | PR introduces breaking changes to contract interfaces |
| `documentation-only` | PR only changes docs (can skip some CI checks) |

## Review Process

### For Regular Changes

1. Create PR with descriptive title and linked issues
2. Ensure CI passes (build, type-check, contracts, lint-imports)
3. Get at least 1 maintainer approval
4. Address review feedback
5. Merge via squash merge

### For Core Infrastructure Changes

Changes to contracts listed under "Core shared infrastructure" in CODEOWNERS:

1. Create PR with descriptive title and linked issues
2. Ensure CI passes
3. Get **2 maintainer approvals** (including code owner)
4. Security review completed
5. Address review feedback
6. Merge via squash merge

### For Security-Sensitive Changes

Changes to auth, consent, encryption, or audit contracts:

1. Create PR with descriptive title and linked issues
2. Ensure CI passes
3. Get **2 maintainer approvals**
4. Security review by designated reviewer
5. Penetration testing results attached (if applicable)
6. Address review feedback
7. Merge via squash merge

### For Multi-Domain Changes

PRs touching contracts from 2+ domains:

1. Create PR with descriptive title and linked issues
2. Ensure CI passes
3. Get **2 maintainer approvals**
4. Domain-specific reviewers for each affected domain
5. Address review feedback
6. Merge via squash merge

## Contract Change Categories

### Low Risk (1 approval, standard CI)

- Documentation changes
- Test additions
- CI/CD configuration
- Non-contract utility scripts

### Medium Risk (1-2 approvals, full CI)

- New contract additions
- Bug fixes in existing contracts
- Non-breaking API changes
- Configuration file updates

### High Risk (2 approvals, full CI, security review)

- Core infrastructure changes
- Authentication/authorization changes
- Financial transaction changes
- Consent management changes
- Storage layout changes

### Critical Risk (2 approvals, security review, manual testing)

- Emergency access mechanism changes
- Cross-chain bridge changes
- Governance parameter changes
- Protocol upgrade changes

## Migration & Upgrade Policy

### Storage Layout Changes

1. Document the change in `CHANGELOG.md`
2. Add migration script if needed
3. Test upgrade path on testnet
4. Include rollback procedure
5. Get approval from contract owner

### Breaking Contract Changes

1. Document the breaking change
2. Provide migration guide in PR description
3. Add deprecation warning if possible
4. Update all affected contracts
5. Coordinate release timing

### Version Bumps

| Change Type | Version Bump | Example |
|-------------|-------------|---------|
| Bug fix | PATCH | 1.0.0 → 1.0.1 |
| New feature (non-breaking) | MINOR | 1.0.0 → 1.1.0 |
| Breaking change | MAJOR | 1.0.0 → 2.0.0 |

## CI/CD Requirements

All PRs must pass the following checks before merge:

| Check | Description | Required |
|-------|------------|---------|
| `build` | `cargo build --release --target wasm32-unknown-unknown` | Yes |
| `type-check` | `cargo check --all-targets` | Yes |
| `contracts` | `cargo test` for all workspace contracts | Yes |
| `lint-imports` | Import path validation across contracts | Yes |
| `wasm-size` | WASM binary size check against performance budgets | Yes |
| `security-scan` | Basic security vulnerability scan | Recommended |

## Code Review Standards

### What to Look For

- **Correctness**: Does the code do what it claims?
- **Security**: Are there injection, overflow, or access control vulnerabilities?
- **Performance**: Does the change stay within resource budgets?
- **Compatibility**: Does it maintain backward compatibility?
- **Tests**: Are there adequate tests for the change?
- **Documentation**: Is the change documented where needed?

### Review Etiquette

- Be constructive and specific
- Reference lines of code in comments
- Suggest alternatives when blocking
- Acknowledge good patterns
- Focus on the code, not the author

## Emergency Procedures

### Hotfix Process

For critical security issues requiring immediate fix:

1. Create a hotfix branch from `main`
2. Implement minimal fix
3. Get expedited review from 1 maintainer
4. Merge to `main`
5. Create follow-up PR for additional hardening
6. Document in incident postmortem

### Rollback Process

If a deployed contract has a critical issue:

1. Identify the affected contract
2. Use `scripts/rollback_deployment.sh` to revert
3. Document the rollback in CHANGELOG.md
4. Create fix PR
5. Redeploy after fix is verified
