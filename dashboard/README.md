# Dashboard

`index.html` + `app.js` + `styles.css` is a static dashboard. It loads exactly
two files, both from `data/`:

- `data/complexity_report.json`
- `data/complexity_trends.json`

**Those are generated locally and are not committed.** Run
`./scripts/complexity_score.sh` to produce them, then open
`dashboard/index.html` and use the Complexity section. See
[`data/README.md`](data/README.md).

## Committed data

Two files in this directory *are* committed, because they are cheap to
regenerate and useful to diff in review:

| File | Produced by |
| --- | --- |
| `contract_inventory.json` | `scripts/contract_inventory.sh` |
| `risk_matrix.json` | `scripts/risk_scoring_engine.py` |

Regenerate both together after adding, removing or modifying a contract:

```bash
make generate-dashboard-data     # or ./scripts/generate_dashboard_data.sh
make check-dashboard-data        # fails if the committed copies are stale
```

CI runs the check on every push and pull request, so a contract change that
forgets to regenerate will be caught.

### The risk matrix needs a full clone

`risk_matrix.json` scores each contract on complexity, git history and blast
radius. The history term comes from `git log` over the contract's path, so a
shallow clone produces **different scores**. The CI job therefore checks out
with `fetch-depth: 0`. If you regenerate locally from a shallow clone you will
commit scores that CI then rejects.

Note that the engine also falls back to a fixed history score of `10.0` when
`git log` fails, rather than erroring, so a broken or non-git checkout produces
plausible-looking but wrong numbers instead of failing loudly.

### Caveat: the scoring model saturates

The current weights and thresholds put **90 of 114 contracts in CRITICAL**, so
the matrix does not discriminate well enough to prioritise review. The cause is
in the scoring functions rather than the weights: `complexity` reaches its
`100.0` cap for 86/114 contracts and `blast_radius` for 88/114 (median
`100.0`), because both multiply raw per-occurrence counts — `sloc * 0.1`,
`require_auth * 15`, `Client::new( * 20`, `env.storage() * 2` — with no
normalisation against contract size.

Recalibrating means changing which contracts are flagged CRITICAL, which is a
review-prioritisation decision rather than a mechanical one, so it is left to the
maintainers. Until then, treat the tiers as relative ordering within the
saturated groups rather than as an absolute risk rating.
