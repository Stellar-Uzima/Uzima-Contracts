# fee

Fee tier calculations for the Uzima network.

Delegates tier-matching to the shared [`calculate_tiered_rate`]
utility in `shared-types` (`contracts/shared_types/src/rate_curve.rs`)
so fee tiers stay consistent with `batch-rewards` and `rewards`
instead of duplicating the logic (issue #1341).

## Public functions

- `calculate_tiered_rate(value, tiers)` — applies the rate of the tier
  `value` falls into, in basis points.
