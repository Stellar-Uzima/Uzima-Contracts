# Technical Debt Tracking System

## Overview
This document serves as the central catalog and tracking system for technical debt across the Uzima-Contracts repository. It defines the process for identifying, assessing, prioritizing, and repaying technical debt.

## Debt Catalog

| ID | Contract/Module | Description | Type | Impact | Priority | Status | Target Date |
|---|---|---|---|---|---|---|---|
| TD-001 | `zkp_registry` | Replace simulated ZKP verification in `verify_zkp_internal` with actual cryptographic verification. | Security | High | High | Open | Q3 2024 |
| TD-002 | `zkp_registry` | Implement proper expiration decryption and validation in `create_credential_proof`. | Security | High | High | Open | Q3 2024 |
| TD-003 | `zkp_registry` | Replace simulated range proof verification in `verify_range_proof_internal` with actual verification. | Security | High | High | Open | Q3 2024 |
| TD-004 | `zkp_registry` | Replace simulated recursive proof verification in `verify_recursive_proof_internal` with actual verification. | Security | High | High | Open | Q4 2024 |

## Assessment Matrix

### Impact Assessment
* **High**: Affects security, core functionality, or causes significant performance degradation. Needs immediate attention.
* **Medium**: Affects maintainability, causes minor performance issues, or lacks test coverage in non-critical paths.
* **Low**: Minor code quality issues, outdated documentation, or non-blocking architectural improvements.

### Priority Ranking
* **High**: Must be resolved in the next milestone or sprint.
* **Medium**: Should be resolved within the next 1-2 quarters.
* **Low**: Can be resolved when working on related components.

## Repayment Tracking Process

1. **Identification**: Any contributor can identify technical debt and propose adding it to the catalog via a Pull Request to this document.
2. **Assessment**: Maintainers review the proposed debt, assess its impact, and assign a priority.
3. **Scheduling**: High-priority items are converted into GitHub Issues and added to the current or next milestone.
4. **Repayment**: A contributor resolves the debt in a PR, referencing the TD-ID.
5. **Closure**: Once the PR is merged, the status in the catalog is updated to "Resolved", with a link to the fixing PR.

## Reporting and Reviews

* **Quarterly Reviews**: The maintainers will conduct a review of the technical debt catalog at the beginning of each quarter (Jan, Apr, Jul, Oct).
* **Review Goals**:
    * Re-assess open items and adjust priorities if necessary.
    * Identify new systemic technical debt.
    * Track the rate of debt repayment vs. accumulation.
    * Schedule repayment for the upcoming quarter.

## Types of Technical Debt

* **Architecture**: Suboptimal design choices that make the system harder to scale or modify.
* **Code Quality**: Code that is difficult to read, understand, or maintain (e.g., duplicated code, complex logic).
* **Security**: Potential vulnerabilities or missing security best practices (e.g., simulated verification instead of actual cryptography).
* **Testing**: Missing or inadequate test coverage for critical components.
* **Documentation**: Outdated, missing, or inaccurate documentation.
* **Observability**: Insufficient logging, monitoring, or tracing capabilities.