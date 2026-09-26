# scripts/

Deployment, release and maintenance scripts.

## Deployment

`deploy-all.sh` is the single entry point for deploying contracts.

```bash
./scripts/deploy-all.sh                      # local
./scripts/deploy-all.sh testnet              # testnet
./scripts/deploy-all.sh futurenet            # futurenet
./scripts/deploy-all.sh testnet --contracts medical_records,identity_registry
./scripts/deploy-all.sh local --skip-build   # reuse existing build
```

It resolves paths from its own location, so it can be invoked from anywhere, and
it is what `make deploy`, `make deploy-testnet` and `make deploy-futurenet` call.

### `deploy_all.sh` was removed

There used to be a second script, `deploy_all.sh` (underscore), with a different
interface: `<environment> <network> [identity]`, where it merged
`config/default.json` with `config/<environment>.json` and deployed only the
contracts marked `enabled: true`. Nothing referenced it, and it could not deploy
anything: `development`, `staging` and `production` — the three environments its
own usage message advertised — each enable **zero** contracts, so it always hit
its "no contracts are enabled" branch and exited `0`. It reported success without
deploying.

`deploy-all.sh` is the survivor because the makefile calls it and it works.

If environment-based selection is wanted later, it belongs in `deploy-all.sh`
rather than as a second script. The `enabled` flags in `config/*.json` are still
there and are the natural place to drive it; note that all three environments
currently enable nothing, so that config needs filling in first.

## Linting shell scripts

```bash
make shellcheck                                    # all tracked .sh files
make shellcheck SHELLCHECK_SEVERITY=style           # tighten locally
```

`make shellcheck` and [`shellcheck.yml`](../.github/workflows/shellcheck.yml) run
the same pinned ShellCheck over the same file set — every `.sh` file git tracks,
not just the ones directly under `scripts/`. Both default to
`--severity=error`.

That default is deliberate and temporary. The corpus had never been linted in CI,
so failing on `warning`/`info`/`style` would have blocked every pull request on
the day the gate landed. Raise `SEVERITY` in the workflow and
`SHELLCHECK_SEVERITY` in the makefile together as the existing scripts are
cleaned up.

The make target deliberately does not depend on `check-deps`: linting a shell
script should not require rustc, soroban and the wasm32 target.

## Known script duplication

Not resolved here, recorded so it is not rediscovered from scratch:

| Scripts | State |
| --- | --- |
| `check_dead_code.sh` | 5-line stub; prints "Dead-code check completed successfully" and checks nothing. Unreferenced. |
| `dead_code_scan.sh` | The real scanner (134 lines). Wired to `npm run deadcode:scan` and `docs/DEADCODE_SCANNING.md`. |
| `dead_code_scan.sh` | A larger, unreferenced variant (351 lines) with extra flags. Needs a functional comparison against the 134-line one before either is deleted. |

A guard rejecting two tracked paths that differ only by `-` versus `_` would stop
the `deploy-all.sh` / `deploy_all.sh` class from recurring. There are currently
no such pairs.
