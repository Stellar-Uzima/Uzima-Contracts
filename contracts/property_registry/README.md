# Property Registry

Soroban contract: on-chain registry of property records — legal owner of record,
recorded valuation, and fractional ownership shares in basis points.

| | |
| --- | --- |
| Crate | `contracts/property_registry` |
| Docs | Every entry point carries `# Auth`, `# Errors` and `# Events` sections on the definition |
| Reentrancy | [`REENTRANCY_REVIEW.md`](REENTRANCY_REVIEW.md), guard in [`src/reentrancy_guard.rs`](src/reentrancy_guard.rs) |
| Tests | `src/test.rs` (behaviour), `src/reentrancy_tests.rs` (guard), `tests/fuzz/property_registry` (proptest) |

## Entry points

| Group | Functions |
| --- | --- |
| Lifecycle | `initialize`, `transfer_admin`, `set_registrar` |
| Operations | `pause`, `unpause` |
| Property lifecycle | `register_property`, `update_valuation`, `transfer_property`, `record_ownership` |
| Queries | `get_property`, `ownership_of`, `total_properties`, `admin`, `registrar`, `paused`, `reentrancy_locked` |

## Authorisation

Three independent authorities, all checked before any state is touched:

| Authority | Held by | Gates |
| --- | --- | --- |
| admin | `initialize` caller | `transfer_admin`, `set_registrar`, `pause`, `unpause` |
| registrar | `initialize` second argument | `register_property`, `update_valuation`, `record_ownership` |
| owner of record | `Property.owner` | `transfer_property`, and the debit leg of `record_ownership` |

**`record_ownership` requires two of them.** The registrar says who may amend the
register; the owner says whose position may be debited. Either alone would be
unsafe — without the owner, a compromised registrar could strip every property in
the registry.

## Execution ordering

Every state-changing entry point runs **check → guard → read → compute → write**.
Steps 4 and 5 are kept separate so a rejected call cannot leave a half-applied
mutation, and each ledger key is read at most once per call.

## Reentrancy (#1772)

There is no cross-contract call in any value path. The one dispatch out of the
contract is `require_auth()`, which for an address backed by a custom
`SorobanAuthorization` contract — the normal way to express multi-sig, spending
policies or session keys — is a real contract invocation whose callee the
address holder chooses. That makes it a genuine reentrancy vector.

The two multi-key entry points therefore take the guard **before** any
authorization dispatch. `transfer_property` also validates the successor's credit
against `BASIS_POINTS_SCALE` before debiting the incumbent, so an over-commitment
is rejected while the incumbent's position is still intact.

Single-key entry points are deliberately unguarded: a guard costs a ledger read
and a write on every call, and there is no interleaving to protect against.
`unguarded_entry_points_are_unaffected_by_the_guard` pins that.

Full reasoning, including the two assumptions that were checked and found sound
(rollback releases the lock; sequential calls do not deadlock), is in
[`REENTRANCY_REVIEW.md`](REENTRANCY_REVIEW.md).

## Tests

### Unit (`src/test.rs`)

Grouped by the acceptance-criteria triad: initialization, happy path, error path.
Covers each error variant, the self-transfer and owner-as-holder rejections,
share dilution to exactly 100%, partial-position handover, independence of
property records, and the pause behaviour including that **queries keep working
while paused** — a registry that cannot be inspected cannot be investigated.

### Reentrancy (`src/reentrancy_tests.rs`)

Driven from `ReentryProbe`, a real contract that calls back into the registry
while its own frame is still open — not a test function calling twice
sequentially, which would prove nothing. The re-entrant call is issued with
`try_` so a refusal is observable as a value rather than aborting the outer
frame, which lets each test assert which call was refused and what the ledger
looks like afterwards.

Asserts: re-entrant `transfer_property` and `record_ownership` are refused with
exactly one operation landing; the guard is released after success; the guard is
released after rejection *on both sides* of `enter`, including the over-commit
path that would otherwise wedge the registry; sequential guarded calls both
succeed; unguarded entry points are unaffected.

### Fuzz (`tests/fuzz/property_registry`)

```bash
cd tests/fuzz/property_registry && cargo test
PROPTEST_CASES=5000 cargo test   # long-duration
```

Five properties, chosen as the ones random input can most plausibly break and
example-based tests cannot reach:

1. **No panic, ever** — whatever bytes arrive, the contract returns a typed
   error or a value, never a host trap.
2. **The split is conserved** — for any accepted `record_ownership`, the sum of
   the two affected shares is unchanged.
3. **No share exceeds 100%** — no accepted call pushes a holder past
   `BASIS_POINTS_SCALE`.
4. **Rejected calls are inert** — after any rejection, both shares are exactly
   what they were.
5. **Reentrancy never wedges** — after any failure on a guarded entry point the
   guard is clear and a subsequent valid call still succeeds.

Amounts are drawn from both ends of the `u128` range as well as the whole range,
because wrap behaviour lives at the boundaries. Identifiers are drawn as raw
byte vectors, so empty, over-long and non-ASCII payloads all reach the contract.

## Running

```bash
cargo test -p property_registry
cargo clippy -p property_registry --all-targets -- -D warnings
```
