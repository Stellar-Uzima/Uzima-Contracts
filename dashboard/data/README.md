# Dashboard data

Complexity scoring output is **generated locally** (not committed):

```bash
./scripts/complexity_score.sh
```

This writes:

- `complexity_report.json` — latest per-contract scores
- `complexity_trends.json` — historical workspace averages (append-only)

Then open `dashboard/index.html` and use the **Complexity** section.
