# FractionalOwnership – Access Control Audit (#1795)

Scope: `contracts/fractional_ownership/src/lib.rs`.

## Roles

| Role | How it is established |
|---|---|
| Admin | Set once in `initialize`; stored in `DataKey::Admin` |
| Holder | Any address with a positive share balance |
| Anyone | Any address (payout depositors, view callers) |

## Entry point review

| Function | Required auth | Extra check | Result |
|---|---|---|---|
| `initialize` | `admin.require_auth()` | Fails with `AlreadyInitialized` on a second call | OK |
| `create_asset` | `admin.require_auth()` | `admin` must equal stored admin, else `Unauthorized` | OK |
| `transfer` | `from.require_auth()` | Balance check (`InsufficientShares`) | OK |
| `list_shares` | `seller.require_auth()` | Balance check | OK |
| `cancel_listing` | `seller.require_auth()` | Only the seller's own listing key is touched | OK |
| `buy` | `buyer.require_auth()` | Seller's listing and balance re-checked | OK |
| `deposit_payout` | `depositor.require_auth()` | Open to anyone by design (funds come from the caller) | OK |
| `claim_payout` | `owner.require_auth()` | Pays only `owner`'s own accrual | OK |
| `get_*` | none | Read-only | OK |

## Findings

- Every state-changing function authorizes the address whose shares or funds
  it moves. No function lets one address act on another's balance.
- The only privileged function after setup is `create_asset`, gated by both
  `require_auth` and an equality check against the stored admin.
- There is no admin transfer or upgrade function, so the admin key cannot be
  changed or hijacked after `initialize`. Losing the admin key only prevents
  creating new assets; existing holders are unaffected.
- `initialize` is not front-run protected beyond `require_auth`; deploy and
  initialize in the same transaction.

## Tests

`src/test_integration.rs` covers `only_admin_can_create_asset` and
`transfer_requires_sender_auth`. Existing unit tests in `src/test.rs` cover the
other `require_auth` paths.
