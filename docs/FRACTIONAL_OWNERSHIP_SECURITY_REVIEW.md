# FractionalOwnership: reentrancy security review

Issue #1788. Scope: `contracts/fractional_ownership/src/lib.rs`, payment token
interaction only. Reviewed against soroban-sdk 21.7.7.

## Method

1. Inventory every call that leaves the contract.
2. For each one, check that all state it depends on is final before the call
   (checks-effects-interactions).
3. Check what a hostile callee could do with control mid-call, including
   calling back into this contract.
4. Add tests with a hostile payment token (`src/test_reentrancy.rs`).

## External call inventory

The contract calls no contract other than the payment token, and only through
`token::Client::transfer`.

| Function | External call | State final before the call |
|---|---|---|
| `buy` | `transfer(buyer, seller, cost)` | Seller and buyer settled, both balances updated, listing reduced or removed, event emitted |
| `deposit_payout` | `transfer(depositor, this, amount)` | Payout index raised, event emitted |
| `claim_payout` | `transfer(this, owner, owed)` | Owner settled, pending balance zeroed, event emitted |

`create_asset`, `transfer`, `list_shares`, `cancel_listing` and every view make
no external call.

## Findings

### R1: Re-entry is already blocked by the host (informational)

Soroban refuses to call a contract that is already on the call stack. A
hostile token calling back into this contract is rejected before any contract
code runs. In `src/test_reentrancy.rs` every re-entry attempt ends with outcome
code 2 (rejected by the host). This is a platform guarantee, not something this
contract enforces, so it is treated as the first layer and not the only one.

### R2: No in-contract guard (hardened)

Relying only on a platform rule leaves no protection if that rule changes or if
a future code path reaches this contract through a different instance. A
storage lock was added:

- `buy`, `deposit_payout` and `claim_payout` take the lock before doing any
  work and release it just before returning success.
- `create_asset`, `transfer`, `list_shares` and `cancel_listing` return
  `Reentrancy` (3013) while the lock is held.
- A failing call reverts all of its storage writes, including the lock, so a
  failure can never leave the contract locked. This is asserted by
  `lock_is_not_left_behind_by_a_failing_guarded_call`.

The lock itself cannot be reached through a hostile token while R1 holds, so
`lock_blocks_every_mutator` sets the flag directly and checks that all seven
mutators refuse to run, and that views still work.

### R3: Checks-effects-interactions ordering (no change needed)

All three external calls are the last statement before returning, after every
storage write. Even if control were handed back mid-call, the state it would
observe is already final: a second `claim_payout` would find a zero pending
balance, a second `buy` would find the reduced listing and updated balances,
and a second `deposit_payout` would add its own amount to the index exactly
once.

### R4: Read-only reentrancy (no issue)

Views read state that is already final at the time of any external call (see
R3), so a callee reading them mid-call sees consistent values.

### R5: Listings are not escrowed (no issue)

Because shares are not held by the contract, there is no escrow release path to
re-enter. `buy` re-checks the seller's balance after settlement, so a seller who
moved shares after listing causes `InsufficientShares` instead of a negative
balance.

## Residual risk: trusted payment token

The payment token is chosen once by the admin in `initialize` and is trusted.
This review assumes a standard token (for example a Stellar asset contract).
A non-standard token can still hurt accounting even without reentrancy:

- A fee-on-transfer token would deliver less than `amount` to the contract in
  `deposit_payout`, while the payout index is raised by the full `amount`. The
  pool would then owe more than it holds.
- A token that can freeze balances can block `claim_payout` for affected
  holders.

Mitigation is operational: only configure a standard token. A balance-delta
check in `deposit_payout` would remove the first case and is a reasonable
follow-up, but it is a behaviour change outside this issue.

## Tests

| Test | Shows |
|---|---|
| `claim_cannot_be_reentered` | Hostile token cannot double-claim; pending is 0 and a second claim fails |
| `buy_cannot_be_reentered` | Exactly one share leaves the listing, balances reflect one purchase |
| `deposit_cannot_be_reentered` | The index reflects exactly one deposit |
| `share_movement_cannot_be_reentered_during_a_deposit` | `transfer` and `list_shares` cannot be run mid-call |
| `lock_blocks_every_mutator` | All seven mutators return `Reentrancy` while locked, views still work, unlocking restores service |
| `lock_is_released_after_every_guarded_call` | The lock is not left set after a successful `buy`, `deposit_payout` or `claim_payout` |
| `lock_is_not_left_behind_by_a_failing_guarded_call` | A failed guarded call leaves no lock behind |
