# Error Codes

All contracts use stable integer error codes. See [docs/ERROR_CODES.md](../../ERROR_CODES.md) for the full registry. It now registers every contract that defines an `Error` enum, including contracts with inline enums in `lib.rs`; `scripts/check_error_codes.sh` fails on any unregistered contract (issue #1949).

## Common Error Ranges

| Range | Category |
|-------|---------|
| 100–199 | Authorization errors |
| 200–299 | Validation errors |
| 400–499 | Not found errors |
| 480–489 | Escrow-specific errors |
| 500–599 | State transition errors |

## Key Error Codes

| Code | Name | Description | Suggestion |
|------|------|-------------|-----------|
| 100 | `Unauthorized` | Caller lacks permission | Check auth |
| 102 | `NotAdmin` | Caller is not admin | Use admin key |
| 205 | `InvalidAmount` | Amount ≤ 0 | Check amount |
| 481 | `EscrowNotFound` | Escrow ID not found | Check ID |
| 482 | `AlreadySettled` | Escrow already settled/refunded | Check status |

## Error Suggestions

Each error has a `get_suggestion()` helper returning a short symbol hint for tooling:

```rust
get_suggestion(Error::Unauthorized) // → CHK_AUTH
get_suggestion(Error::InvalidAmount) // → CHK_LEN
get_suggestion(Error::EscrowNotFound) // → CHK_ID
```
