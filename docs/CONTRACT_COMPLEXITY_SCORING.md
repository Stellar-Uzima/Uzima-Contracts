# Contract Complexity Scoring

Issue **#481** — automated complexity scoring for Soroban contracts in this workspace.

## Score components

| Component | What we measure |
|-----------|-----------------|
| **Cyclomatic complexity** | Branches (`if`, `match` arms, loops, `&&` / `\|\|`) per contract |
| **Data structure complexity** | `contracttype` structs/enums, field counts, storage access patterns |
| **External interaction count** | `invoke_contract` and cross-contract calls |
| **State transition count** | Status enums, `match` on status, transition helpers |
| **Permission model complexity** | `require_auth`, admin/role checks |

Each component is normalized to **0–100**, then combined with weights:

- Cyclomatic 25%
- Data structure 20%
- External interactions 20%
- State transitions 20%
- Permission model 15%

**Total score** is 0–100. **Grade**: Low (&lt;40), Medium (40–69), High (≥70).

## CLI

```bash
# From repo root
./scripts/complexity_score.sh

# Or directly
cargo run -p contract_optimizer --features cli -- complexity \
  --contracts-path contracts \
  --output dashboard/data/complexity_report.json \
  --trends dashboard/data/complexity_trends.json
```

## Dashboard

Report files are **generated locally** (gitignored). See `dashboard/data/README.md`.

1. Run `./scripts/complexity_score.sh` to write `dashboard/data/complexity_report.json` and append to `complexity_trends.json`.
2. Open `dashboard/index.html` in a browser (or serve the `dashboard/` folder).
3. Use the **Complexity** section: per-contract table (five component scores), workspace average, and trend chart.

Trend snapshots append on each script run (last 90 retained).

## Tests

```bash
cargo test -p contract_optimizer complexity
```
