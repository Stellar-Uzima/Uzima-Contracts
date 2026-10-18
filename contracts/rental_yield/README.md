# rental_yield

Minimal Soroban contract tracking a per-property rental-yield pool: the admin
deposits collected rent and distributes it to recipients.

This is intentionally a small foundation, not a complete implementation —
see issues #1811 (access control), #1809 (shared-primitive refactor), #1807
(input validation), and #1806 (events), all of which it already addresses
at this minimal scope:

- **Access control**: admin-only `deposit`/`distribute`, enforced via
  `common_auth::require_admin_custom!`.
- **Shared primitives**: auth and errors reuse `common_auth` and
  `common_error` rather than hand-rolled equivalents, following the same
  pattern as `rbac`'s `errors.rs`.
- **Input validation**: non-positive amounts and over-withdrawal are
  rejected with `Error::InvalidInput` / `Error::InsufficientFunds`.
- **Events**: `deposit` and `distribute` each publish an event with the
  amount and resulting pool balance.
