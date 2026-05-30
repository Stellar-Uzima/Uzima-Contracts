

All Uzima contracts use consistent numeric error codes derived from `CommonError`
in the `shared_types` crate.

| Code | Variant             | Meaning                                              |
|------|---------------------|------------------------------------------------------|
| 1    | `Unauthorized`      | Caller lacks permission for this action              |
| 2    | `NotFound`          | Requested resource does not exist                    |
| 3    | `InvalidInput`      | One or more arguments are invalid                    |
| 4    | `AlreadyExists`     | Duplicate creation rejected                          |
| 5    | `RateLimitExceeded` | Caller has exceeded their allowed request rate       |
| 6    | `ContractPaused`    | Contract is paused; no state-changing calls allowed  |

## Usage in Contracts

```rust
use shared_types::CommonError;

#[contracterror]
pub enum ContractError {
    Common(CommonError),          // wraps all shared variants
    ConsentAlreadyRevoked = 100,  // contract-specific (start at 100+)
}
