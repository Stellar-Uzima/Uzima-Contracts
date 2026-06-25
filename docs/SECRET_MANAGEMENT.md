# Secret Management Policy

How Uzima-Contracts prevents credentials (Stellar seeds, RPC keys, API tokens,
private keys) from being committed, and what to do when scanning flags
something. Implements issue #915. See also
[SECURITY_BEST_PRACTICES.md](./SECURITY_BEST_PRACTICES.md) and
[SECURITY_CHECKLIST.md](./SECURITY_CHECKLIST.md).

## Rules

1. **Never hard-code secrets.** No private keys, seeds, mnemonics, RPC secrets,
   or API tokens in source, tests, scripts, or docs — not even "temporary" ones.
2. **Load secrets from the environment.** Read them from environment variables
   (or a local `.env`, which is git-ignored). For ad-hoc/test accounts, generate
   ephemeral keys at runtime (e.g. `Keypair.random()`), never a literal seed.
3. **`.env*` and `secrets.toml` stay out of git.** They are already listed in
   `.gitignore`; do not force-add them.
4. **Rotate on exposure.** If a real secret is ever committed, treat it as
   compromised: rotate/revoke it immediately, then scrub it from history. The
   scanner failing is not "fixed" by deleting the line in a new commit — the
   value is still reachable in the old commit.

## Automated scanning

Two layers, both running [`gitleaks`](https://github.com/gitleaks/gitleaks)
against the shared allowlist in [`.gitleaks.toml`](../.gitleaks.toml):

| Layer | Trigger | Config |
| --- | --- | --- |
| Pre-commit hook | every local `git commit` (staged changes) | [`.pre-commit-config.yaml`](../.pre-commit-config.yaml) |
| CI gate | every push / PR to `main` and `develop` | [`.github/workflows/secret-scan.yml`](../.github/workflows/secret-scan.yml) |

The CI job scans the **full commit history** and fails the build on any
non-allowlisted finding, so a secret cannot merge even if the local hook was
skipped.

### Developer setup (one time)

```bash
pip install pre-commit       # or: brew install pre-commit
pre-commit install           # installs the git pre-commit hook

# scan everything on demand:
pre-commit run --all-files
# or run gitleaks directly:
gitleaks git . -c .gitleaks.toml --redact --verbose
```

## When a scan flags something

1. **Is it a real secret?** If yes — remove it, switch to an env var, rotate the
   credential, and scrub history (`git filter-repo` or BFG). Do not allowlist it.
2. **Is it a false positive?** (a public constant, a test fixture, a truncated
   placeholder.) Add a **narrowly scoped** entry to `.gitleaks.toml` with a
   comment explaining why it is not a secret. Prefer matching the exact value or
   path over a broad pattern, so real secrets still trip the scanner.

## History audit (issue #915, task 3)

A full-history scan (all commits, `gitleaks git .`) was run when this policy
landed. **No live credentials were found.** The only matches were non-secret
false positives, now documented in `.gitleaks.toml`:

- W3C DID verification-method type identifiers (`Ed25519VerificationKey2020`,
  `X25519KeyAgreementKey2020`) in the generated SDK bindings — public constants.
- Zero-knowledge test fixtures under `tests/.tmp-zk/` — public commitment hashes
  and proof points, not secrets.
- A truncated placeholder string in `scripts/deploy_forensics.ts`
  (`'SAKLGIB3G6P7E5YV...'`, with a `// placeholder` comment) — not a valid seed.
- Committed Rust build artifacts under `**/target/` — binary scanner noise.
