# Error-Handling Policy for Soroban Contract Utilities

## 1. Error Enum Definition

Every contract **must** define a single `#[contracterror]` enum that covers all
recoverable failure modes for that contract. Error discriminants **must** be
explicit (`#[repr(u32)]`) and grouped by category:

| Range       | Category            |
|-------------|---------------------|
| 100–199     | Access / Auth       |
| 200–299     | Input validation    |
| 300–399     | Lifecycle / State   |
| 400–499     | Entity existence    |
| 500–599     | Financial / Storage |
| 600–699     | Cryptography / ZK   |
| 700–799     | Cross-chain         |
| 800–899     | Domain-specific     |

## 2. Result Propagation

- All public functions that can fail **must** return `Result<T, Error>`.
- Use `Ok(())` for side-effect-only functions instead of `bool`.
- Prefer the `?` operator over `.unwrap()` / `.expect()` in production code.

## 3. Checked Arithmetic

- All arithmetic **must** use `checked_add`, `checked_sub`, `checked_mul`,
  or `saturating_*` variants. Never use raw `+`, `-`, `*` on numeric values
  that could overflow.
- Fee calculations must use the `fp_math` crate's `mul_bps` helper.

## 4. Error-to-Suggestion Mapping

Each error enum **should** provide a `get_suggestion(error) -> Symbol` helper
that maps error variants to short remediation hints:

- `CHK_AUTH` — check caller authorization
- `CHK_ID` — verify ID exists
- `CHK_DATA` — validate input format
- `RE_TRY_L` — retry later (rate-limit, paused)
- `ALREADY` — operation already completed
- `FILL_FLD` — required field is empty
- `ADD_FUND` — insufficient balance
- `CLN_OLD` — storage limit reached
- `CONTACT` — contact support

## 5. Event Emission on Errors

Critical errors (auth failures, overflow, storage full) **should** emit a
structured log or diagnostic event before returning, using the contract's
`emit_structured_log` or `env.events().publish()` pattern.

## 6. Reentrancy Protection

Contracts that perform external token transfers **must** use either:
- A reentrancy guard (CEI pattern: checks → effects → interactions), or
- An explicit `acquire_lock` / `release_lock` pair around mutating calls.

## 7. Testing Requirements

- Every error variant **must** have at least one test that triggers it.
- Error discriminant values **must** be locked with a stability test.
- The `get_suggestion` helper **must** be tested for all documented variants.

## References

- [ERROR_CODES.md](./ERROR_CODES.md) — complete error code registry
- Each contract's `errors.rs` / `Error` enum
