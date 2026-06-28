# Role-Based Access Control (RBAC) — Uzima Contracts

## Role Hierarchy

## Rules

- **Delegation**: A role holder can only grant roles at or below their own level.
  An `admin` can grant `sub_admin` or `viewer`. A `sub_admin` can grant `viewer` only.
- **Revocation**: Revoking a role is immediate and cascades to all roles that were
  delegated downstream from the revoked role.
- **No self-escalation**: No address can grant itself or others a role above its
  current permission level.

## Contract Functions

| Function | Required Role | Description |
|---|---|---|
| `initialize(admin)` | — | Sets the initial admin at deployment |
| `grant_role(granter, grantee, role)` | admin or sub_admin (within hierarchy) | Delegates a role |
| `revoke_role(revoker, target, role)` | Parent role holder | Revokes a role and cascades |
| `has_role(address, role)` | — | Read-only role check |

## Adding a New Role

1. Add the role symbol to the contract constants
2. Insert it into the hierarchy map
3. Add tests to `delegation_tests.rs`
4. Update this document