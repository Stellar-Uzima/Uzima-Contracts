# Secure Testing Practices

## Overview

This document outlines secure testing practices for the Uzima Contracts project to prevent hardcoded secrets and ensure proper security hygiene in testing environments.

## Key Principles

### 1. Never Hardcode Secrets
- **Private Keys**: Never commit private keys or seed phrases to version control
- **Contract Addresses**: Use environment variables for all contract addresses
- **API Keys**: Store API keys in environment variables, never in code
- **Test Credentials**: Use environment-specific test credentials

### 2. Environment Variable Management
- Use `.env` files for local development
- Reference `.env.example` for required environment variables
- Never commit `.env` files to version control
- Use CI/CD secrets for automated testing

### 3. Test Data Security
- Use mock/generated data for testing when possible
- Sanitize test data before committing
- Avoid using production data in tests

## Implementation Guidelines

### Rust Tests

```rust
#[test]
fn test_with_config() {
    let admin_key = std::env::var("TEST_ADMIN_KEY")
        .expect("TEST_ADMIN_KEY must be set");
    
    let contract_address = std::env::var("TEST_CONTRACT_ADDRESS")
        .unwrap_or_else(|_| "DEFAULT_ADDRESS".to_string());
    
    // Use in test
    let admin = Address::from_string(&String::from_str(&env, &admin_key));
}
```

### Python Tests

```python
import os

def setup_contracts():
    contract_address = os.getenv("CONTRACT_ADDRESS", "DEFAULT_ADDRESS")
    api_key = os.getenv("TEST_API_KEY")
    
    if not api_key:
        raise ValueError("TEST_API_KEY environment variable must be set")
```

### Shell Scripts

```bash
#!/bin/bash

# Use environment variables with defaults
USDC_ADDRESS="${USDC_ADDRESS:-DEFAULT_USDC_ADDRESS}"
CONTRACT_ID="${CONTRACT_ID:-DEFAULT_CONTRACT_ID}"

# Require certain variables
if [ -z "$ADMIN_KEY" ]; then
    echo "Error: ADMIN_KEY environment variable must be set"
    exit 1
fi
```

### TypeScript/JavaScript

```typescript
const contractId = process.env.CONTRACT_ID || "DEFAULT_CONTRACT_ID";
const rpcUrl = process.env.RPC_URL || "https://soroban-testnet.stellar.org";

if (!process.env.API_KEY) {
    throw new Error("API_KEY environment variable is required");
}
```

## Environment Variables

### Required Variables

See `.env.example` for the complete list of required environment variables:

- `TEST_ADMIN_ADDRESS`: Admin address for testing
- `TEST_CONTRACT_ID`: Contract ID for integration tests
- `USDC_ADDRESS`: USDC token address (testnet/mainnet specific)
- `SOROBAN_RPC_URL`: RPC endpoint for Soroban network
- `NETWORK_PASSPHRASE`: Network passphrase (Testnet/Mainnet)

### Optional Variables

- `DEV_CONTRACT_ID`: Development contract ID
- `MONITORING_CONTRACT_ID`: Contract ID for health monitoring
- `CI_TEST_TOKEN`: Token for CI/CD automated testing

## CI/CD Integration

### GitHub Actions

```yaml
env:
  TEST_ADMIN_KEY: ${{ secrets.TEST_ADMIN_KEY }}
  TEST_CONTRACT_ID: ${{ secrets.TEST_CONTRACT_ID }}
  USDC_ADDRESS: ${{ secrets.USDC_ADDRESS }}
```

### Local Development

1. Copy `.env.example` to `.env`:
   ```bash
   cp .env.example .env
   ```

2. Fill in your actual values in `.env`

3. Load environment variables:
   ```bash
   source .env  # Linux/Mac
   # or
   $env:TEST_VAR = "value"  # PowerShell
   ```

## Security Checklist

### Before Committing
- [ ] No hardcoded addresses, keys, or secrets in code
- [ ] All sensitive data uses environment variables
- [ ] `.env` file is in `.gitignore`
- [ ] Test data is sanitized
- [ ] API endpoints use environment-specific URLs

### Before Deployment
- [ ] Production secrets are stored in secure vault
- [ ] CI/CD secrets are properly configured
- [ ] Environment variables are documented
- [ ] Test credentials differ from production

### Regular Audits
- [ ] Scan codebase for hardcoded secrets
- [ ] Review environment variable usage
- [ ] Audit CI/CD secrets access
- [ ] Check for leaked credentials in logs

## Tools for Secret Detection

### Recommended Tools
- **git-secrets**: Scans repositories for secrets
- **truffleHog**: Finds hardcoded secrets in git history
- **detect-secrets**: Enterprise secret scanning

### Custom Scans
```bash
# Find potential hardcoded addresses
grep -r "C[A-Z0-9]{55,}" . --exclude-dir=target

# Find potential private keys
grep -r -i "private.*key\|secret.*key\|seed.*phrase" . --exclude-dir=target

# Find environment variable usage
grep -r "std::env::var\|process\.env\|os\.getenv" .
```

## Incident Response

### If Secrets Are Leaked
1. **Immediate Actions**:
   - Revoke all exposed credentials
   - Rotate all affected keys/tokens
   - Remove secrets from code/history

2. **Investigation**:
   - Determine scope of exposure
   - Audit access logs
   - Review recent commits

3. **Prevention**:
   - Implement pre-commit hooks for secret detection
   - Add automated scanning to CI/CD
   - Train team on secure practices

## Best Practices

### Code Reviews
- Always check for hardcoded secrets in PRs
- Verify environment variable usage
- Ensure proper error handling for missing variables

### Documentation
- Keep `.env.example` updated
- Document all required environment variables
- Include setup instructions for new developers

### Testing
- Test with missing environment variables
- Verify fallback values work correctly
- Test with different network configurations

## References

- [OWASP Secret Management Cheat Sheet](https://cheatsheetseries.owasp.org/cheatsheets/Secret_Management_Cheat_Sheet.html)
- [Stellar Soroban Security Best Practices](https://soroban.stellar.org/docs/getting-started/security)
- [Git Security Best Practices](https://git-scm.com/book/en/v2/Git-Tools-Signed-Commits)
