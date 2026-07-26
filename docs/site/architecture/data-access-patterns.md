# Data Access Patterns

See [docs/DATA_ACCESS_PATTERNS.md](../../DATA_ACCESS_PATTERNS.md).

## Patient-Controlled Access

1. Patient owns their records
2. Patient grants access to providers via `grant_access()`
3. Provider reads record — access is logged
4. Patient can revoke at any time via `revoke_access()`

## Role-Based Access

The `rbac` contract defines roles: `Admin`, `Doctor`, `Nurse`, `Patient`, `Auditor`.

## Audit Logging

Every record access emits an event and is logged to the `audit` contract for HIPAA compliance.
