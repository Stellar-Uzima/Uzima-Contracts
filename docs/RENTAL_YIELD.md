# RentalYield

`contracts/rental_yield` holds rental income deposited by the admin and lets
each beneficiary claim the yield credited to them.

## Custom errors

All failures return a typed `Error` (codes 3101-3107) instead of panicking:
`NotInitialized`, `AlreadyInitialized`, `Unauthorized`, `InvalidAmount`,
`Overflow`, `NothingToClaim`, `Reentrancy`.

## Reentrancy review

- External calls happen only in `deposit_yield` and `claim` (payment token).
- Both follow checks-effects-interactions: the claimable balance is updated
  before the token transfer.
- Both hold a storage lock (`DataKey::Locked`) for the whole call and fail
  with `Error::Reentrancy` if it is already held.
- Soroban forbids contract re-entry natively; the lock is defense in depth.
