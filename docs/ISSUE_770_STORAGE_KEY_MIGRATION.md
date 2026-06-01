# Issue #770 — Storage Key Migration Draft

Draft `DataKey` enums for **timelock**, **reputation**, and **escrow**, plus a storage compatibility matrix for review before contract changes.

**Status:** Applied in `contracts/timelock`, `contracts/reputation`, and `contracts/escrow` (Issue #770 Phase 1).

---

## Prefix convention

Soroban stores `DataKey` as a typed `Val` (enum discriminant + payload), not as a short `Symbol`. Prefixing avoids ambiguous names in cross-contract tooling, migration scripts, and docs.

| Contract   | Logical prefix | Rust variant prefix (PascalCase) | Example logical ID   |
|------------|----------------|----------------------------------|----------------------|
| Timelock   | `TIMELOCK_`    | `Timelock`                       | `TIMELOCK_CONFIG`    |
| Reputation | `REPUT_`       | `Reput`                          | `REPUT_ADMIN`        |
| Escrow     | `ESCROW_`      | `Escrow`                         | `ESCROW_FEE_CONFIG`  |

**Rules (aligned with `medical_records` / `rbac` / `audit`):**

1. One `#[contracttype] pub enum DataKey` per contract (in `types.rs` or top of `lib.rs`).
2. Variant names: `{Prefix}{Purpose}` — no bare `Admin` / `Config` in migration docs when multiple contracts are listed together.
3. **Phase 1 (this draft):** 1:1 replacement of each `Symbol` key; keep existing **values** (`Map`, structs) unchanged.
4. **Phase 2 (optional later):** split nested maps into per-id variants (e.g. `EscrowOrder(u64)`) — separate migration; not in matrix below.
5. Events and error hints stay on `symbol_short!`; out of scope for `DataKey`.

**Serialization note:** Migrating from `Symbol` → `DataKey` **changes the ledger key bytes**. Existing deployments need a migration entrypoint or redeploy with state import. The matrix column **Breaking** is `Yes` for every row unless you dual-read during a transition window.

---

## 1. Timelock — `DataKey`

**File (proposed):** `contracts/timelock/src/types.rs`

```rust
use soroban_sdk::contracttype;

/// Typed storage keys for the Timelock contract.
/// Logical prefix: TIMELOCK_
#[derive(Clone)]
#[contracttype]
pub enum DataKey {
    /// TimelockConfig — admin, delay, sequence rules (`TimelockConfig` struct).
    /// Was: `symbol_short!("cfg")` on instance storage.
    TimelockConfig,

    /// TimelockQueue — `Map<u64, QueuedTx>` of scheduled operations.
    /// Was: `symbol_short!("queue")` on persistent storage.
    TimelockQueue,
}
```

---

## 2. Reputation — `DataKey`

**File (proposed):** `contracts/reputation/src/types.rs`

```rust
use soroban_sdk::contracttype;

/// Typed storage keys for the Reputation contract.
/// Logical prefix: REPUT_
#[derive(Clone)]
#[contracttype]
pub enum DataKey {
    /// ReputAdmin — governor address.
    /// Was: `symbol_short!("admin")` on instance storage.
    ReputAdmin,

    /// ReputScores — `Map<Address, i128>` of reputation balances.
    /// Was: `symbol_short!("scores")` on persistent storage.
    ReputScores,
}
```

---

## 3. Escrow — `DataKey`

**File (proposed):** `contracts/escrow/src/types.rs`

```rust
use soroban_sdk::contracttype;

/// Typed storage keys for the Escrow contract.
/// Logical prefix: ESCROW_
#[derive(Clone)]
#[contracttype]
pub enum DataKey {
    /// EscrowAdmin — contract admin address.
    /// Was: `symbol_short!("admin")` on instance storage.
    EscrowAdmin,

    /// EscrowFeeConfig — platform fee settings (`FeeConfig`).
    /// Was: `symbol_short!("feeconf")` on instance storage.
    EscrowFeeConfig,

    /// EscrowEscrows — `Map<u64, Escrow>` keyed by order_id.
    /// Was: `symbol_short!("escrow")` on persistent storage.
    EscrowEscrows,

    /// EscrowCredits — pull-payment credit balances `Map<Address, i128>`.
    /// Was: `symbol_short!("credits")` on persistent storage.
    EscrowCredits,

    /// EscrowStats — aggregate `PlatformStats` on instance storage.
    /// Was: `symbol_short!("stats")` on instance storage.
    EscrowStats,

    /// EscrowDailyStats — `Map<u64, DailyStats>` keyed by day_id.
    /// Was: `symbol_short!("dlystats")` on persistent storage.
    EscrowDailyStats,

    /// EscrowReentrancyLock — temporary bool guard (CEI / reentrancy).
    /// Was: `symbol_short!("relock")` on temporary storage.
    EscrowReentrancyLock,
}
```

---

## Storage compatibility matrix

### Timelock

| Old key (`Symbol`) | Storage | Value type | New `DataKey` variant | Logical ID | Breaking |
|--------------------|---------|------------|------------------------|------------|----------|
| `symbol_short!("cfg")` | `instance` | `TimelockConfig` | `DataKey::TimelockConfig` | `TIMELOCK_CONFIG` | Yes |
| `symbol_short!("queue")` | `persistent` | `Map<u64, QueuedTx>` | `DataKey::TimelockQueue` | `TIMELOCK_QUEUE` | Yes |

**Code replacements (Phase 1):**

| Before | After |
|--------|-------|
| `has(&CFG)` / `get(&CFG)` / `set(&CFG, …)` | `has(&DataKey::TimelockConfig)` / `get` / `set` |
| `get(&QUEUE)` / `set(&QUEUE, …)` / `extend_ttl(&QUEUE, …)` | `DataKey::TimelockQueue` |

Remove: `const CFG`, `const QUEUE`.

---

### Reputation

| Old key (`Symbol`) | Storage | Value type | New `DataKey` variant | Logical ID | Breaking |
|--------------------|---------|------------|------------------------|------------|----------|
| `symbol_short!("admin")` | `instance` | `Address` | `DataKey::ReputAdmin` | `REPUT_ADMIN` | Yes |
| `symbol_short!("scores")` | `persistent` | `Map<Address, i128>` | `DataKey::ReputScores` | `REPUT_SCORES` | Yes |

**Code replacements (Phase 1):**

| Before | After |
|--------|-------|
| `has(&ADMIN)` / `get(&ADMIN)` / `set(&ADMIN, …)` | `DataKey::ReputAdmin` |
| `get(&SCORES)` / `set(&SCORES, …)` | `DataKey::ReputScores` |

Remove: `const ADMIN`, `const SCORES`.

---

### Escrow

| Old key (`Symbol`) | Storage | Value type | New `DataKey` variant | Logical ID | Breaking |
|--------------------|---------|------------|------------------------|------------|----------|
| `symbol_short!("admin")` | `instance` | `Address` | `DataKey::EscrowAdmin` | `ESCROW_ADMIN` | Yes |
| `symbol_short!("feeconf")` | `instance` | `FeeConfig` | `DataKey::EscrowFeeConfig` | `ESCROW_FEE_CONFIG` | Yes |
| `symbol_short!("escrow")` | `persistent` | `Map<u64, Escrow>` | `DataKey::EscrowEscrows` | `ESCROW_ESCROWS` | Yes |
| `symbol_short!("credits")` | `persistent` | `Map<Address, i128>` | `DataKey::EscrowCredits` | `ESCROW_CREDITS` | Yes |
| `symbol_short!("stats")` | `instance` | `PlatformStats` | `DataKey::EscrowStats` | `ESCROW_STATS` | Yes |
| `symbol_short!("dlystats")` | `persistent` | `Map<u64, DailyStats>` | `DataKey::EscrowDailyStats` | `ESCROW_DAILY_STATS` | Yes |
| `symbol_short!("relock")` | `temporary` | `bool` | `DataKey::EscrowReentrancyLock` | `ESCROW_REENTRANCY_LOCK` | Yes |

**Code replacements (Phase 1):**

| Before | After |
|--------|-------|
| `&ADMIN` | `&DataKey::EscrowAdmin` |
| `&FEE_CONF` | `&DataKey::EscrowFeeConfig` |
| `&ESCROWS` | `&DataKey::EscrowEscrows` |
| `&CREDITS` | `&DataKey::EscrowCredits` |
| `&STATS` | `&DataKey::EscrowStats` |
| `&DAILY_STATS` | `&DataKey::EscrowDailyStats` |
| `&REENTRANCY_LOCK` | `&DataKey::EscrowReentrancyLock` |

**Tests:** Replace inline `symbol_short!("relock")` in `escrow/src/lib.rs` tests with `DataKey::EscrowReentrancyLock` so keys match production.

Remove: all seven `const …: Symbol` storage key constants.

---

## Master mapping (quick reference)

| Contract | Old `symbol_short!` | New variant | Logical ID |
|----------|---------------------|-------------|--------------|
| timelock | `"cfg"` | `TimelockConfig` | `TIMELOCK_CONFIG` |
| timelock | `"queue"` | `TimelockQueue` | `TIMELOCK_QUEUE` |
| reputation | `"admin"` | `ReputAdmin` | `REPUT_ADMIN` |
| reputation | `"scores"` | `ReputScores` | `REPUT_SCORES` |
| escrow | `"admin"` | `EscrowAdmin` | `ESCROW_ADMIN` |
| escrow | `"feeconf"` | `EscrowFeeConfig` | `ESCROW_FEE_CONFIG` |
| escrow | `"escrow"` | `EscrowEscrows` | `ESCROW_ESCROWS` |
| escrow | `"credits"` | `EscrowCredits` | `ESCROW_CREDITS` |
| escrow | `"stats"` | `EscrowStats` | `ESCROW_STATS` |
| escrow | `"dlystats"` | `EscrowDailyStats` | `ESCROW_DAILY_STATS` |
| escrow | `"relock"` | `EscrowReentrancyLock` | `ESCROW_REENTRANCY_LOCK` |

---

## Phase 2 (future, not in initial PR)

Optional structural improvements after Phase 1 ships and migration is proven:

| Contract | Idea | New variants (examples) |
|----------|------|-------------------------|
| timelock | Per-tx slot instead of one map | `TimelockQueuedTx(u64)` |
| reputation | Per-user score slot | `ReputScore(Address)` |
| escrow | Per-order escrow row | `EscrowOrder(u64)` |
| escrow | Per-address credit | `EscrowCredit(Address)` |
| escrow | Per-day bucket | `EscrowDailyStat(u64)` |

Each Phase 2 change is an additional breaking migration from Phase 1 `DataKey` values.

---

## Suggested implementation order

1. Add `types.rs` + `pub mod types;` + `pub use types::DataKey` per contract.
2. Replace storage `get` / `set` / `has` / `remove` / `extend_ttl` keys only (no value shape changes).
3. Add `migrate_storage_keys()` (admin-only) **or** document redeploy-only path per `docs/CONTRACT_UPGRADE_SAFETY.md`.
4. Update tests; run contract tests for all three crates.
5. Link this doc from the issue #770 PR description.

---

## Naming alternatives (if you prefer shorter variants)

If `EscrowEscrows` feels redundant, acceptable renames **before** implementation:

| Current draft | Alternative |
|---------------|-------------|
| `TimelockConfig` | `TimelockCfg` |
| `EscrowEscrows` | `EscrowOrders` |
| `ReputScores` | `ReputScoreMap` |

Keep the **logical ID** column (`ESCROW_*`, etc.) stable in docs even if Rust variant names change.
