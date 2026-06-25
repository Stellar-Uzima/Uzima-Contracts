# Dependency Hygiene Policy

Keeps `Cargo.toml` dependency lists lean. Unused dependencies slow compile
times and inflate WASM binary size — which directly affects on-chain Soroban
contract deploy cost — so they are treated as a CI failure. Implements issue
#883.

## Rule

Every dependency declared in a `Cargo.toml` must actually be used by that crate.
No "just in case" or copy-pasted dependencies.

## Automated check

CI runs [`cargo machete`](https://github.com/bnjbvr/cargo-machete) on every push
and PR (`.github/workflows/unused-deps.yml`). It is static analysis — it parses
manifests and scans source, it does **not** build the workspace — so it is fast
and the job **fails on any unused dependency**.

Run it locally before pushing:

```bash
cargo install cargo-machete --locked   # once
cargo machete                          # from the repo root
```

Optionally wire it into the pre-commit hook alongside the secret scanner:

```yaml
# .pre-commit-config.yaml
- repo: local
  hooks:
    - id: cargo-machete
      name: cargo-machete (unused deps)
      entry: cargo machete
      language: system
      pass_filenames: false
```

## Handling a finding

1. **Genuinely unused?** Remove the dependency from `Cargo.toml`. If it is an
   `optional` dependency, also remove it from any `[features]` entry that
   references it via `dep:<name>`. Rebuild the affected crate to confirm.
2. **False positive?** (used only through a macro, a build script, or a
   `cfg`-gated path that the scanner can't see.) Add it to the per-crate ignore
   list and explain why:

   ```toml
   [package.metadata.cargo-machete]
   ignored = ["prost"]   # used only via prost-derive macros
   ```

   Prefer per-crate ignores with a justification over leaving real dead weight.

## Baseline (issue #883)

The initial sweep removed 12 unused dependencies across 6 crates:

| Crate | Removed |
| --- | --- |
| `contract_optimizer` | `anyhow`, `octocrab`, `proc-macro2`, `regex`, `reqwest` |
| `clinical_nlp` | `medical_records`, `upgradeability` |
| `cross_chain_bridge` | `replay_protection` |
| `appointment_booking_escrow` | `upgradeability` |
| `provider_directory` | `uzima-sanitization` |
| `uzima-tests` (`tests/`) | `reputation`, `healthcare_payment` |

Removing `octocrab`/`reqwest` from `contract_optimizer` (they were referenced
only in a comment) also drops the `rustls 0.21` / `rustls-webpki 0.101` chain
from the dependency graph, eliminating the advisories tracked in the cargo-audit
allowlist (#913).
