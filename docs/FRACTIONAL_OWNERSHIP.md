# FractionalOwnership

`contracts/fractional_ownership` splits ownership of an asset into fungible
shares. Shares can be transferred, sold at a fixed price, and entitle their
holder to a pro-rata part of payouts (for example rental income) funded into
the contract.

Tracking issues: #1785 (unit tests), #1786 (fuzz tests), #1787 (NatSpec),
#1788 (reentrancy review).

## Model

- An **asset** is created by the admin with a fixed number of shares, all
  minted to an issuer. Asset ids start at 1.
- A **listing** is a standing offer by a holder: `amount` shares at
  `price_per_share` payment-token base units. Shares are not escrowed; the
  seller's balance is re-checked when someone buys.
- A **payout pool** per asset is funded by anyone through `deposit_payout`
  and shared by holders in proportion to their shares.

## Entry points

| Function | Caller | Effect |
|---|---|---|
| `initialize(admin, payment_token)` | admin | One-time setup |
| `create_asset(admin, issuer, total_shares)` | admin | Mint a fixed supply to `issuer`, returns the asset id |
| `transfer(from, to, asset_id, amount)` | holder | Move shares |
| `list_shares(seller, asset_id, amount, price_per_share)` | holder | Create or replace a listing |
| `cancel_listing(seller, asset_id)` | holder | Remove a listing |
| `buy(buyer, seller, asset_id, amount)` | buyer | Fill (part of) a listing, returns the cost |
| `deposit_payout(depositor, asset_id, amount)` | anyone | Fund the payout pool |
| `claim_payout(owner, asset_id)` | holder | Withdraw everything owed, returns the amount |
| `get_admin`, `get_payment_token`, `get_asset_count` | anyone | Configuration views |
| `get_total_shares`, `get_balance`, `get_listing` | anyone | Share and listing views |
| `get_payout_index`, `get_pending_payout` | anyone | Payout views |

Read-only functions use the `get_` prefix so `scripts/check_events.sh`
recognises them as non-mutating.

## Payout accounting

The contract keeps a cumulative **payout index** per asset: payout per share
scaled by `PAYOUT_PRECISION` (10^12).

```
deposit:  index += amount * PAYOUT_PRECISION / total_shares      (floored)
accrued:  balance * (index - checkpoint) / PAYOUT_PRECISION      (floored)
```

A holder's accrual is **settled** (moved to a pending balance and the
checkpoint advanced) before that holder's balance changes, so transfers and
purchases never change what either side is already owed. A deposit costs the
same regardless of the number of holders.

Rounding always floors, so the contract can never owe more than it holds.
The remainder (dust, under one payment-token base unit per deposit and per
settlement) stays in the contract. A deposit so small that it would not move
the index is rejected with `PayoutTooSmall` instead of being locked up.

## Errors

Codes are registered in `docs/ERROR_CODES.md` under the reserved range
`3000-3099`.

| Code | Name | Meaning |
|---|---|---|
| 3001 | `NotInitialized` | `initialize` has not run |
| 3002 | `AlreadyInitialized` | `initialize` already ran |
| 3003 | `Unauthorized` | Caller is not the stored admin |
| 3004 | `InvalidAmount` | Zero or negative amount, supply or price |
| 3005 | `AssetNotFound` | Unknown asset id |
| 3006 | `InsufficientShares` | Holder owns fewer shares than requested |
| 3007 | `SelfTransfer` | `from == to` or `buyer == seller` |
| 3008 | `NoListing` | No open listing to cancel or buy |
| 3009 | `ExceedsListing` | Buying more than the listing offers |
| 3010 | `Overflow` | Checked arithmetic overflowed |
| 3011 | `NothingToClaim` | Nothing is owed to the caller |
| 3012 | `PayoutTooSmall` | Deposit too small to distribute |
| 3013 | `Reentrancy` | A guarded external call is in progress |

## Storage

Instance: `Admin`, `PaymentToken`, `AssetCount`, `Locked`. Persistent: `TotalShares`,
`Balance`, `Listing`, `PayoutIndex`, `Checkpoint`, `Pending`. Persistent
entries have their TTL extended on every write.

## Known limitations

- The supply of an asset is fixed at creation; there is no burn or re-mint.
- There is no pause switch and no fee. Both are intentionally out of scope for
  the base contract.
- The payment token is fixed at `initialize` and is trusted to behave like a
  standard token (see the security review for what that assumes).

## Unit tests (#1785)

`src/test.rs` holds the unit tests. Run them with:

```bash
cargo test -p fractional_ownership
```

Coverage by area:

| Area | What is checked |
|---|---|
| `initialize` | Stores config, emits `init`, rejects a second call, views and `create_asset` fail before init, requires admin auth |
| `create_asset` | Mints the whole supply to the issuer, sequential independent ids, `Unauthorized`, `InvalidAmount`, emits `created` |
| `transfer` | Moves shares, full-balance transfer, `InvalidAmount`, `SelfTransfer`, `AssetNotFound`, `InsufficientShares`, requires sender auth, state untouched on failure |
| `list_shares` / `cancel_listing` | Stores and replaces listings, no escrow, every input error, `NoListing` |
| `buy` | Full and partial fills, funds and shares move, every input error, seller moved shares away, cost overflow, insufficient buyer funds reverts the share movement |
| payouts | Single holder, pro-rata split, transfer after deposit keeps the seller's entitlement, late buyer gets nothing from earlier deposits, accumulation, double claim, rounding dust, `PayoutTooSmall`, overflow, per-asset isolation |

Every `Error` variant except `Reentrancy` (covered in
`src/test_reentrancy.rs`, see the security review) is asserted at least once.

## Fuzz tests (#1786)

`src/fuzz_test.rs` uses `proptest` (the same 1.6 line as the other contract
fuzz targets). Run it with:

```bash
PROPTEST_CASES=256 cargo test -p fractional_ownership fuzz_
```

CI runs it through `scripts/run_contract_fuzz.sh` (`PROPTEST_CASES` defaults to
40 there).

| Property | What it checks |
|---|---|
| `fuzz_random_sequences_match_model` | Random sequences (up to 39 operations) of transfer, list, cancel, buy, deposit and claim over 4 users. Every outcome (success or typed error) must match a plain-Rust model, and after every step: balances match the model, shares sum to the supply, the contract's token balance equals deposits minus claims, and the sum of everything owed never exceeds what the contract holds |
| `fuzz_extreme_amounts_never_trap` | Boundary and random `i128` values (`MIN`, `-1`, `0`, `1`, `MAX/2`, `MAX`, arbitrary) through every entry point. The result must be success or a typed contract error, never a host trap, which would mean unchecked arithmetic |
| `fuzz_payout_split_is_pro_rata_and_solvent` | Random supplies, holder splits and deposits. Each holder's payout is at most the exact pro-rata share and at most 2 base units below it, total paid never exceeds the deposit, and dust is bounded by `2 * holders` |

Panic witnesses are written to `contracts/fractional_ownership/proptest-regressions/`
and replayed first on the next run. The test environment budget is lifted in
the sequence test because one `Env` serves a whole sequence of calls.

## NatSpec documentation (#1787)

`src/lib.rs` documents the crate and every public entry point with
NatSpec-style tags inside Rust doc comments, so they also render in `cargo doc`:

| Tag | Used for |
|---|---|
| `@title`, `@notice` | One-line purpose of the contract or function |
| `@dev` | Implementation notes: authorization, ordering, rounding |
| `@param` | Each argument, with its constraints |
| `@return` | The returned value |
| `@custom:error` | Each error the function can return and when |

Module-level notes describe rounding, the payout index and storage layout.

## Reentrancy review (#1788)

The full review is in
[`FRACTIONAL_OWNERSHIP_SECURITY_REVIEW.md`](FRACTIONAL_OWNERSHIP_SECURITY_REVIEW.md).
In short:

- The only external calls are to the payment token, in `buy`,
  `deposit_payout` and `claim_payout`, and each one is the last step of its
  function after all state is updated.
- Those three functions also hold a storage lock (`DataKey::Locked`), and every
  other mutating entry point refuses to run while it is held (`Reentrancy`,
  code 3013).
- `src/test_reentrancy.rs` uses a hostile payment token that tries to re-enter
  `claim_payout`, `buy`, `deposit_payout`, `transfer` and `list_shares`
  mid-call. All attempts are rejected and the balances, listing and payout
  state show exactly one effect.

## Recent changes

- `initialize` rejects `admin == payment_token` and `transfer` rejects sending shares to the contract itself (`InvalidAddress`, 3014).
- A `settled` event (`asset_id`, `owner`, `amount`) is emitted whenever accrued payout is moved into a holder's pending balance.
- `settle` skips the checkpoint write when it is already current, saving a storage write.
- Admin checks use an OpenZeppelin `Ownable`-style `only_admin` helper.
