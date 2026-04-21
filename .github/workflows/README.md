# CI/CD Workflows

This directory contains the streamlined CI/CD pipelines for Uzima Contracts.

## Active Workflows (3)

### 1. CI (`ci.yml`)
**Triggers:** Push/PR to `main` or `develop` branches

**What it does:**
- ✅ Code formatting check (rustfmt)
- ✅ Clippy linting
- ✅ Unit tests (all contracts)
- ✅ Integration tests
- ✅ Node.js API tests
- ✅ WASM contract builds
- ✅ Security scan (cargo audit + secret scanning) - only on PRs and main branch

**When it runs:** Every push and pull request to main/develop

---

### 2. Deploy (`deploy.yml`)
**Triggers:** 
- Auto: Push to `develop` branch or version tags → deploys to **testnet**
- Manual: Workflow dispatch → deploys to **testnet** or **mainnet**

**What it does:**
- 🚀 Runs pre-deployment tests (optional)
- 🚀 Builds optimized WASM contracts
- 🚀 Deploys to selected network
- 🚀 Records deployment metadata
- 🚀 Creates deployment summary

**Network Selection:**
- **Testnet**: Auto-deploys on develop branch pushes
- **Mainnet**: Manual trigger only, requires confirmation ("DEPLOY")

**Required Secrets:**
- `TESTNET_DEPLOYER_SECRET_KEY` - For testnet deployments
- `MAINNET_DEPLOYER_SECRET_KEY` - For mainnet deployments

**Required GitHub Environments:**
- `testnet` - With approval rules (optional)
- `mainnet` - With required reviewers

---

### 3. Release (`release.yml`)
**Triggers:** Push of version tags (e.g., `v1.0.0`, `v2.1.3`)

**What it does:**
- 📦 Builds all contracts optimized for production
- 📦 Generates SHA256 checksums
- 📦 Creates GitHub Release with WASM artifacts
- 📦 Auto-generates release notes

**Output:** GitHub Release with downloadable WASM files

---

## Archived Workflows

The following workflows have been consolidated and archived in the `archived/` directory:

- `ci.yml` → Merged into new unified `ci.yml`
- `test.yml` → Tests merged into new `ci.yml`
- `security.yml` → Security scans merged into new `ci.yml`
- `deploy-mainnet.yml` → Merged into unified `deploy.yml`
- `deploy-testnet.yml` → Merged into unified `deploy.yml`

**Why archived?**
- Reduced CI/CD complexity from 6 workflows to 3
- Eliminated redundant test runs (was running 2x on every push)
- Consolidated deployment logic into single workflow with environment support
- Faster feedback loops with parallel job execution

---

## Workflow Triggers Summary

| Event | CI | Deploy | Release |
|-------|----|--------|---------|
| Push to `main` | ✅ | ❌ | ❌ |
| Push to `develop` | ✅ | ✅ (testnet) | ❌ |
| PR to `main`/`develop` | ✅ | ❌ | ❌ |
| Push tag `v*.*.*` | ❌ | ✅ (testnet) | ✅ |
| Manual dispatch | ✅ | ✅ (choose network) | ❌ |

---

## Best Practices

### For Developers
1. **Push frequently** to your feature branch - CI will validate your code
2. **Create PRs** to main/develop - full CI suite runs on PRs
3. **Monitor CI status** before requesting reviews

### For Deployments
1. **Testnet**: Automatically deploys when code merges to `develop`
2. **Mainnet**: Use manual workflow dispatch with confirmation
3. **Always verify** deployments using: `./scripts/monitor_deployments.sh <network>`

### For Releases
1. Tag releases with semantic versioning: `git tag v1.2.3`
2. Push tags: `git push origin v1.2.3`
3. Release workflow automatically creates GitHub Release with artifacts

---

## Environment Configuration

### GitHub Required Settings

**Environments (Settings → Environments):**

#### `testnet`
- Optional: Require reviewers for deployment approval
- URL: `https://soroban-testnet.stellar.org`

#### `mainnet`  
- **Required**: At least 2 reviewers
- **Required**: Wait timer (recommended: 5 minutes)
- URL: `https://soroban-rpc.stellar.org`

**Secrets (Settings → Secrets → Actions):**

```
TESTNET_DEPLOYER_SECRET_KEY=<stellar-secret-key-for-testnet>
MAINNET_DEPLOYER_SECRET_KEY=<stellar-secret-key-for-mainnet>
```

---

## Troubleshooting

### CI Fails on Formatting
```bash
cargo fmt --all
git add .
git commit -m "fix: format code"
```

### CI Fails on Clippy
```bash
cargo clippy --all-targets --all-features --fix
# Review changes carefully
git add .
git commit -m "fix: resolve clippy warnings"
```

### Deployment Fails
1. Check that secrets are configured correctly
2. Verify network connectivity to Stellar RPC
3. Check deployment logs in GitHub Actions
4. Ensure deployer account has sufficient funds (testnet)

### Release Not Created
1. Verify tag format: `v*.*.*` (e.g., `v1.0.0`)
2. Check that tag was pushed: `git push origin <tag>`
3. Verify GITHUB_TOKEN has `contents: write` permission

---

## Migration from Old Workflows

If you need to reference the old workflows, they're preserved in `archived/`:

```bash
# View archived workflows
ls .github/workflows/archived/

# Restore a workflow if needed
cp .github/workflows/archived/ci.yml .github/workflows/
```

---

## Performance Improvements

**Before (6 workflows):**
- ⏱️ 12+ duplicate jobs per push
- 🔄 Average CI time: 8-12 minutes
- 💸 High GitHub Actions minutes usage

**After (3 workflows):**
- ⏱️ Parallel execution, no duplication
- 🔄 Average CI time: 4-6 minutes
- 💸 ~50% reduction in Actions minutes
- 📊 Clear separation of concerns

---

For questions or issues with CI/CD, please open a GitHub Issue.
