# Automated Refactoring Suggestions

This document describes the automated refactoring suggestion system for Uzima Contracts. It identifies common improvement opportunities across the codebase and provides guidance for addressing them.

## Categories

### 1. Function Extraction Opportunities

Long functions (>80 lines) that can be split into smaller, focused helpers:

| Contract | Function | Lines | Suggestion |
|---|---|---|---|
| `medical_records` | `write_record` | ~120 | Extract `validate_record_fields`, `encrypt_payload` |
| `cross_chain_bridge` | `submit_message` | ~90 | Extract `build_message`, `validate_chain_pair` |
| `anomaly_detector` | `run_inference` | ~100 | Extract `compute_weighted_score`, `classify_alert_level` |

### 2. Duplicate Code Consolidation

Patterns repeated across multiple contracts that should be extracted into shared utilities:

- **Admin check pattern**: `caller.require_auth(); Self::require_admin(&env, &caller)?;`
  - Appears in: `anomaly_detector`, `cross_chain_bridge`, `aml`, `audit`, `rbac`
  - Suggestion: Macro `require_admin!(env, caller)` or shared trait

- **Initialization guard**: Check-then-set `DataKey::Admin` / `DataKey::Initialized`
  - Appears in: all contracts
  - Suggestion: Shared `init_guard` helper in a common module

- **Overflow-safe counter increment**: `count.checked_add(1).ok_or(Error::Overflow)?`
  - Appears in: `cross_chain_bridge`, `anomaly_detector`, `medical_records`
  - Suggestion: `increment_counter(env, key)` utility

### 3. Dead Code Identification

Run the following to identify unused functions and types:

```bash
cargo clippy --all -- -W dead_code 2>&1 | grep "warning: .* is never used"
```

Known candidates (from last audit):
- `contracts/ai_analytics/src/admin.rs` — `get_admin` function appears unused externally
- `contracts/credential_notifications/src/lib.rs` — `initialize` is a no-op stub

### 4. Performance Optimization Suggestions

- **Avoid repeated storage reads**: Several functions read the same key multiple times in a single call. Cache in a local variable.
  ```rust
  // Before
  let admin: Address = env.storage().instance().get(&DataKey::Admin).unwrap();
  // ... later in same function ...
  let admin2: Address = env.storage().instance().get(&DataKey::Admin).unwrap();

  // After
  let admin: Address = env.storage().instance().get(&DataKey::Admin).unwrap();
  // reuse `admin`
  ```

- **Use `unwrap_or_default` over `unwrap_or(Default::default())`** where applicable.

- **Prefer `saturating_add` over `checked_add` for counters** that cannot realistically overflow `u64`.

### 5. API Improvement Recommendations

- **Consistent error return types**: Some contracts return `bool` on success, others return `()`. Standardize to `Result<(), Error>` for all state-mutating functions.

- **Event emission on all state changes**: Several functions mutate state without emitting events, making off-chain indexing harder. Audit with:
  ```bash
  grep -rn "pub fn " contracts/*/src/lib.rs | grep -v "get_\|is_\|has_\|query_"
  # Then verify each has a corresponding env.events().publish(...)
  ```

- **Batch operations**: High-frequency callers (e.g., bulk record ingestion) would benefit from batch variants of `write_record`, `create_alert`, and `submit_message`.

## CI/CD Integration

Add the following step to `.github/workflows/ci.yml` to generate a refactoring report on every PR:

```yaml
- name: Refactoring suggestions
  run: |
    cargo clippy --all -- \
      -W clippy::too_many_lines \
      -W clippy::cognitive_complexity \
      -W dead_code \
      2>&1 | tee reports/refactoring.txt
```

Upload `reports/refactoring.txt` as a PR artifact so reviewers can see suggestions inline.

## Developer Notification

When the CI report contains new suggestions (diff against `main`), post a summary comment on the PR using the GitHub Actions `actions/github-script` step:

```yaml
- name: Comment refactoring diff
  if: github.event_name == 'pull_request'
  uses: actions/github-script@v7
  with:
    script: |
      const fs = require('fs');
      const report = fs.readFileSync('reports/refactoring.txt', 'utf8');
      const lines = report.split('\n').filter(l => l.startsWith('warning')).slice(0, 20);
      if (lines.length > 0) {
        github.rest.issues.createComment({
          issue_number: context.issue.number,
          owner: context.repo.owner,
          repo: context.repo.repo,
          body: `### 🔧 Refactoring Suggestions\n\`\`\`\n${lines.join('\n')}\n\`\`\``
        });
      }
```
