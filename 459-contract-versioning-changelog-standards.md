# Contract Versioning and Changelog Standards

## Overview

This document establishes the versioning scheme and changelog requirements for all smart contracts in the Uzima-Contracts repository. Consistent versioning and changelog practices ensure clear communication of changes and facilitate smooth upgrades.

## Versioning Policy

### Semantic Versioning (SemVer)

We adopt Semantic Versioning 2.0.0 for all contracts:

```
MAJOR.MINOR.PATCH
```

- **MAJOR**: Breaking changes that require migration
- **MINOR**: New features added in a backward-compatible manner
- **PATCH**: Backward-compatible bug fixes

### Version Examples

- `1.0.0` - Initial stable release
- `1.1.0` - Added new feature, backward compatible
- `1.1.1` - Bug fix for existing feature
- `2.0.0` - Breaking change requiring migration

### Pre-release Versions

For development and testing:

- `1.0.0-alpha.1` - Alpha release
- `1.0.0-beta.2` - Beta release
- `1.0.0-rc.1` - Release candidate

## Changelog Format

### Standard Changelog Structure

```markdown
# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- New feature descriptions

### Changed
- Changes in existing functionality

### Deprecated
- Features that will be removed in future versions

### Removed
- Features removed in this version

### Fixed
- Bug fixes

### Security
- Security-related changes

## [1.2.0] - 2024-01-15

### Added
- New governance voting mechanism
- Enhanced token transfer validation

### Changed
- Updated gas optimization for transfer functions

### Fixed
- Fixed overflow in calculation functions

## [1.1.0] - 2024-01-01

### Added
- Initial contract deployment
- Basic token functionality
```

### Changelog Entry Guidelines

1. **Use clear, descriptive language**
2. **Group changes by type** (Added, Changed, Fixed, etc.)
3. **Include migration notes for breaking changes**
4. **Reference related issues** where applicable
5. **Use past tense** for completed changes

## Breaking Change Documentation

### Breaking Change Template

```markdown
### ⚠️ BREAKING CHANGES

#### Contract: ContractName
- **Description**: Brief description of the breaking change
- **Impact**: Who is affected and how
- **Migration Required**: Yes/No
- **Migration Steps**:
  1. Step one
  2. Step two
  3. Step three
- **Deadline**: Date when old version becomes unsupported
- **Alternative**: Temporary workarounds if available
```

### Breaking Change Categories

1. **Critical Breaking Changes**
   - Storage layout changes
   - Function signature changes
   - Event definition changes
   - Access control modifications

2. **Minor Breaking Changes**
   - Return value changes
   - Error message updates
   - Gas cost increases > 20%

## Upgrade Path Guidelines

### Upgrade Strategies

#### 1. Proxy Pattern Upgrades
```solidity
// Recommended for production contracts
contract UpgradeableContract {
    address public implementation;
    address public admin;
    
    function upgrade(address newImplementation) external onlyAdmin {
        // Validation and upgrade logic
    }
}
```

#### 2. Migration Contracts
```solidity
// For contracts requiring state migration
contract MigrationContract {
    function migrateFromOldContract(
        address oldContract,
        uint256 amount
    ) external {
        // Migration logic
    }
}
```

### Upgrade Process

1. **Pre-Upgrade**
   - Announce upgrade timeline
   - Provide migration guide
   - Test on testnet extensively
   - Get security audit if major change

2. **During Upgrade**
   - Monitor for issues
   - Provide support channels
   - Document any unexpected behavior

3. **Post-Upgrade**
   - Update documentation
   - Decommission old version after grace period
   - Share lessons learned

## Version History Tracking

### Version Registry

Maintain a version registry in `docs/versions/`:

```
docs/versions/
├── v1.0.0/
│   ├── audit-reports/
│   ├── deployment-addresses/
│   └── migration-guides/
├── v1.1.0/
│   ├── audit-reports/
│   ├── deployment-addresses/
│   └── migration-guides/
└── current.json
```

### Version Metadata

Each version should include:

```json
{
  "version": "1.2.0",
  "releaseDate": "2024-01-15",
  "contracts": {
    "Token": "0x1234...",
    "Governance": "0x5678..."
  },
  "auditReports": [
    "audit-report-v1.2.0.pdf"
  ],
  "migrationRequired": false,
  "supportedUntil": "2024-12-31",
  "dependencies": {
    "openzeppelin": "^4.8.0"
  }
}
```

## CI/CD Validation

### Version Validation Rules

```yaml
# .github/workflows/version-check.yml
name: Version Validation

on:
  pull_request:
    paths:
      - 'contracts/**'
      - 'package.json'

jobs:
  version-check:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - name: Check version format
        run: |
          # Validate SemVer format
          # Check changelog updated
          # Verify breaking change documentation
```

### Automated Checks

1. **Version Format Validation**
   - Ensure versions follow SemVer
   - Check for version conflicts
   - Validate pre-release formats

2. **Changelog Validation**
   - Ensure changelog is updated
   - Check for required sections
   - Validate breaking change documentation

3. **Upgrade Path Validation**
   - Verify migration guides exist
   - Check upgrade compatibility
   - Validate deprecation notices

## Contract Tier Versioning

### Tier Classification

1. **Core Contracts** (Critical)
   - Strict versioning requirements
   - Mandatory migration guides
   - Extended support periods

2. **Utility Contracts** (Important)
   - Standard versioning
   - Basic migration support
   - Regular support periods

3. **Experimental Contracts** (Development)
   - Flexible versioning
   - No migration guarantees
   - Limited support

### Version Support Policy

| Contract Tier | Support Duration | Migration Support |
|---------------|------------------|-------------------|
| Core | 24 months | Full |
| Utility | 12 months | Basic |
| Experimental | 6 months | Best effort |

## Best Practices

### Version Management

1. **Increment versions appropriately**
   - MAOR for breaking changes
   - MINOR for new features
   - PATCH for bug fixes

2. **Use version tags in Git**
   ```bash
   git tag -a v1.2.0 -m "Release version 1.2.0"
   git push origin v1.2.0
   ```

3. **Maintain version consistency**
   - All contracts in same release use same version
   - Dependencies clearly documented
   - Version conflicts resolved

### Changelog Management

1. **Update changelog with every change**
2. **Use consistent formatting**
3. **Include relevant technical details**
4. **Reference related issues and PRs**
5. **Review changelog in pull requests**

### Communication

1. **Announce breaking changes well in advance**
2. **Provide clear migration instructions**
3. **Offer upgrade assistance**
4. **Document lessons learned**

## Tools and Resources

### Recommended Tools

- **semantic-release**: Automated versioning
- **conventional-changelog**: Automated changelog generation
- **commitizen**: Standardized commit messages
- **version-checker**: Version validation

### External Resources

- [Semantic Versioning Specification](https://semver.org/)
- [Keep a Changelog](https://keepachangelog.com/)
- [Conventional Commits](https://www.conventionalcommits.org/)

## Review Process

### Version Review Checklist

- [ ] Version follows SemVer format
- [ ] Changelog is updated and complete
- [ ] Breaking changes are properly documented
- [ ] Migration guide is provided if needed
- [ ] Version registry is updated
- [ ] CI/CD validation passes
- [ ] Security review completed for major changes

### Approval Requirements

- **PATCH versions**: Team lead approval
- **MINOR versions**: Team lead + tech lead approval
- **MAJOR versions**: Team lead + tech lead + security team approval

---

This document should be reviewed quarterly and updated as needed to reflect evolving best practices and project requirements.
