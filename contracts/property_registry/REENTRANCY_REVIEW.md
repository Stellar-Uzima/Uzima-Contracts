# Reentrancy Review — `property_registry`

Scope: issue **#1772** (Review PropertyRegistry for reentrancy vulnerabilities).

Reviewed surface: every entry point on `PropertyRegistryContract`, with
particular attention to `transfer_property` and `record_ownership`, which are
the only two that read and write more than one ledger key.

## Threat model

An attacker is assumed to control:

- one or more holder or registrar addresses, with keys but no privileged role;
- **the `SorobanAuthorization` contract attached to any address they control.**

That last one is the interesting capability and the reason this review exists.
On Soroban an address does not sign by itself: `Address::require_auth()` hands
off to whichever `SorobanAuthorization` contract the address is configured
with. Multi-sig, spending policies and session keys are all implemented this
way. That makes `require_auth()` a **real contract invocation whose callee the
address holder chooses**, and therefore a genuine reentrancy vector rather than a
theoretical one.

## The contract has no value-path cross-contract calls

Every entry point reads and writes only this contract's own ledger entries. There
is no `token::Client`, no external registry lookup, no `Address::transfer`
to another contract, and no hook. So the textbook read-write-read pattern is not
reachable — there is no external call to re-enter *through*.

The one dispatch out of the contract is `require_auth()`, described above.

## Findings

### R1 — Reentrancy during `require_auth` on a multi-key write *(mitigated)*

`transfer_property` and `record_ownership` both compute and then settle two
ownership entries plus the property record. An attacker whose authorization
contract re-enters during `require_auth` could interleave with that sequence.

**Mitigation.** Both entry points take the guard in
[`reentrancy_guard`](../src/reentrancy_guard.rs) **before** any authorization
dispatch. Taking it before rather than after is the whole point: `require_auth`
is the attacker's own code, so a guard taken after it would be taken after the
attack has already run.

A re-entrant call is refused with [`Error::ReentrantCall`]. Covered by
`reentrant_transfer_property_is_refused` and
`reentrant_record_ownership_is_refused`, both driven from
[`ReentryProbe`](../src/reentrancy_tests.rs) — a real contract that calls back in
while its own frame is still open, rather than a test function calling twice
sequentially, which would prove nothing.

### R2 — A failed guarded call leaving the contract permanently locked *(checked, sound)*

The guard is released on success paths only. A caller could worry that an early
`return Err(..)` after `enter` would strand the flag and brick the registry.

**Not a defect.** Soroban execution is transactional: returning `Err` rolls back
every storage write made in that frame, including the guard flag. So the lock is
released by the same rollback that released everything else.

This is the single assumption the design rests on, so it is pinned from both
directions: `guard_is_released_when_a_guarded_call_fails` rejects on both sides of
the guard, and `guard_is_released_when_a_guarded_call_is_rejected_mid_flight`
rejects *after* the guard is taken (the over-commit path), which is the case that
would wedge the registry if the assumption were wrong.

### R3 — The guard deadlocking sequential calls *(checked, no)*

**Not a defect.** The guard is per-frame-chain state, not a lock held across
invocations. `two_sequential_guarded_calls_both_succeed` confirms back-to-back
guarded calls both go through.

### R4 — Check-effects-interactions ordering *(held, and made explicit)*

Even with the guard, the ordering is defensive:

1. Check — authorization, pause state, ownership bounds, share bounds.
2. Compute — pure arithmetic into locals.
3. Write — every ledger entry settled with no reads in between.

`transfer_property` computes the successor's credit and validates it against
`BASIS_POINTS_SCALE` **before** debiting the incumbent, so an over-commitment is
rejected while the incumbent's position is still intact. This is asserted
end-to-end in `transfer_property_never_half_applies` in the fuzz harness, which
checks that a refused transfer leaves both shares byte-identical.

### R5 — Unguarded entry points *(deliberate, recorded)*

`register_property`, `update_valuation` and the administrative entry points are
**not** guarded. Each touches a bounded number of keys with no
read-modify-write across a caller-visible boundary, so there is no interleaving
to protect against, and a guard costs a ledger read and a write on every call.
`unguarded_entry_points_are_unaffected_by_the_guard` pins that they keep working.

This is a deliberate cost decision, not an oversight. If a future change makes one
of them multi-key, it must take the guard in the same change.

### R6 — Read-only entry points *(no action)*

The query functions touch no writable state and take no lock. There is nothing to
re-enter.

## Residual risk

- **A compromised admin or registrar key is still full compromise** within its
  own scope. The guard bounds what a *hostile authorization contract* can do; it
  does nothing about a leaked key, which is what a timelock or multi-sig
  governance layer upstream of this contract is for.
- **The guard is per-frame-chain.** If a future entry point is added that invokes
  another contract *and* writes multiple keys, it needs the guard as well. There
  is no compile-time check enforcing that, only this document and the review
  checklist.
- **`SorobanAuthorization` re-entrancy is not exercised by the unit tests in the
  way production would trigger it.** The tests approximate it with a direct
  re-entrant call from a probe contract, which is the same call graph without the
  auth framework in the path. An end-to-end test with a real custom
  authorization contract would be stronger and is the natural follow-up.