# Release Process

## Overview

This document outlines the comprehensive release process for the Uzima-Contracts project, ensuring consistent, reliable, and automated releases with proper versioning, changelog management, and deployment verification.

## Release Types

### Automated Releases

- **Patch Releases**: Bug fixes and security updates
- **Minor Releases**: New features and enhancements
- **Major Releases**: Breaking changes and significant updates
- **Pre-releases**: Alpha, beta, and release candidate versions

### Manual Releases

- **Emergency Hotfixes**: Critical security issues
- **Custom Releases**: Special deployment requirements

## Release Workflow

### 1. Preparation Phase

#### Prerequisites Checklist

- [ ] All tests passing (`make test`)
- [ ] Code coverage requirements met
- [ ] Security scans completed
- [ ] Documentation updated
- [ ] Changelog prepared
- [ ] Migration guides ready (for major releases)
- [ ] Release notes drafted

#### Branch Management

```bash
# For minor/major releases
git checkout develop
git pull origin develop
git checkout -b release/vX.Y.Z

# For hotfixes
git checkout main
git pull origin main
git checkout -b hotfix/vX.Y.Z
```

### 2. Version Bump

#### Automated Version Bumping

```bash
# Patch release
make release VERSION=1.1.1

# Minor release
make release VERSION=1.2.0

# Major release
make release VERSION=2.0.0

# Pre-release
make release VERSION=1.2.0-alpha.1
```

#### Manual Version Updates

If manual version bumping is required:

1. **Update Cargo.toml**
```toml
[workspace.package]
version = "1.2.0"
```

2. **Update Contract Versions**
```bash
# Update individual contract versions
find contracts/ -name Cargo.toml -exec sed -i 's/version = ".*"/version = "1.2.0"/' {} \;
```

3. **Update Documentation**
```bash
# Update version references in README
sed -i 's/v[0-9]\+\.[0-9]\+\.[0-9]\+/v1.2.0/g' README.md
```

### 3. Changelog Management

#### Changelog Update Process

1. **Generate Changelog Entry**
```bash
# Auto-generate from git history
make generate-changelog VERSION=1.2.0

# Or manually edit CHANGELOG.md
```

2. **Changelog Format**
```markdown
## [1.2.0] - 2026-04-20

### Added
- New patient consent management system
- Enhanced audit logging with timestamps
- Support for medical imaging metadata

### Fixed
- Memory leak in record encryption
- Validation error in patient registration
- Network timeout issues in contract deployment

### Changed
- Improved error messages for better debugging
- Optimized WASM contract sizes by 15%
- Updated dependencies to latest stable versions

### Breaking Changes
- `register_patient` function signature updated (see migration guide)
- Storage layout optimized for better performance (requires migration)

### Security
- Fixed potential data exposure in audit logs
- Enhanced encryption key management
```

### 4. Build Verification

#### Comprehensive Build Process

```bash
# Clean build environment
make clean

# Build optimized contracts
make build-opt

# Run all tests
make test

# Run integration tests
make test-integration

# Verify WASM sizes
make check-wasm-size

# Security audit
make audit

# Code quality checks
make check
```

#### Contract Verification

```bash
# Verify contract functionality
./scripts/verify_contracts.sh

# Check contract versions
make check-versions

# Validate contract interfaces
./scripts/validate_interfaces.sh
```

### 5. Tag Creation

#### Git Tag Management

```bash
# Create annotated tag
git tag -a v1.2.0 -m "Release v1.2.0: Patient consent management and performance improvements"

# Push tag to remote
git push origin v1.2.0

# Push release branch
git push origin release/v1.2.0
```

#### Tag Validation

```bash
# Verify tag format
echo "v1.2.0" | grep -E '^v[0-9]+\.[0-9]+\.[0-9]+(-[a-zA-Z0-9]+)?$'

# Check tag exists
git tag -l | grep v1.2.0

# Verify tag points to correct commit
git show v1.2.0 --stat
```

### 6. Deployment Process

#### Environment Deployment

```bash
# Deploy to testnet (automated for minor/major releases)
./scripts/deploy_environment.sh testnet

# Deploy to mainnet (manual approval required)
./scripts/deploy_environment.sh mainnet --confirm
```

#### Deployment Verification

```bash
# Verify deployment health
./scripts/monitor_deployments.sh testnet

# Check contract functionality
./scripts/test_deployment.sh testnet

# Validate contract versions on-chain
./scripts/verify_versions.sh testnet
```

### 7. Release Publication

#### GitHub Release Creation

```bash
# Create GitHub release with artifacts
gh release create v1.2.0 \
  --title "Release v1.2.0" \
  --notes "Release notes for v1.2.0" \
  --target main \
  dist/*.wasm
```

#### Artifact Publication

```bash
# Upload build artifacts
./scripts/publish_artifacts.sh v1.2.0

# Publish to package registries (if applicable)
make publish
```

### 8. Notification System

#### Automated Notifications

- **Slack**: Release announcement in #releases channel
- **Email**: Notification to subscribers
- **Twitter**: Release announcement tweet
- **GitHub**: Release published with changelog

#### Notification Templates

```bash
# Slack notification
./scripts/notify_slack.sh "Uzima-Contracts v1.2.0 released! 🚀 Features: patient consent management, performance improvements. Details: https://github.com/Stellar-Uzima/Uzima-Contracts/releases/tag/v1.2.0"

# Email notification
./scripts/notify_email.sh --template release --version 1.2.0
```

## Automation Scripts

### Main Release Script

```bash
#!/bin/bash
# scripts/release.sh

set -e

VERSION=${1:-$(cat VERSION)}
RELEASE_TYPE=${2:-minor}

echo "🚀 Starting release process for v$VERSION"

# 1. Validation
echo "📋 Validating release prerequisites..."
make check
make test

# 2. Version bump
echo "🔢 Bumping version to $VERSION..."
make bump-version VERSION=$VERSION

# 3. Changelog update
echo "📝 Updating changelog..."
make update-changelog VERSION=$VERSION

# 4. Build verification
echo "🔨 Building and verifying..."
make build-opt
make test-integration

# 5. Tag creation
echo "🏷️  Creating release tag..."
git tag -a v$VERSION -m "Release v$VERSION"
git push origin v$VERSION

# 6. Deployment
echo "🚀 Deploying contracts..."
./scripts/deploy_environment.sh testnet

# 7. Release publication
echo "📢 Publishing release..."
gh release create v$VERSION --title "Release v$VERSION" --notes-file CHANGELOG.md

# 8. Notifications
echo "📧 Sending notifications..."
./scripts/notify_release.sh $VERSION

echo "✅ Release v$VERSION completed successfully!"
```

### Makefile Integration

```makefile
# Release automation
release: check-deps
	@if [ -z "$(VERSION)" ]; then \
		echo "Usage: make release VERSION=X.Y.Z"; \
		exit 1; \
	fi
	@echo "🚀 Starting release process for v$(VERSION)..."
	./scripts/release.sh $(VERSION)

# Version bump
bump-version: check-deps
	@if [ -z "$(VERSION)" ]; then \
		echo "Usage: make bump-version VERSION=X.Y.Z"; \
		exit 1; \
	fi
	./scripts/bump_version.sh $(VERSION)

# Changelog generation
generate-changelog: check-deps
	@if [ -z "$(VERSION)" ]; then \
		echo "Usage: make generate-changelog VERSION=X.Y.Z"; \
		exit 1; \
	fi
	./scripts/generate_changelog.sh $(VERSION)

# Version consistency check
check-versions: check-deps
	./scripts/check_versions.sh
```

## Release Validation

### Pre-release Validation

```bash
# Complete validation suite
make validate-release VERSION=1.2.0

# Individual validation steps
make validate-version VERSION=1.2.0
make validate-changelog VERSION=1.2.0
make validate-build
make validate-tests
make validate-security
```

### Post-release Validation

```bash
# Verify release deployment
make verify-release VERSION=1.2.0

# Check contract functionality
make test-deployment NETWORK=testnet

# Validate release artifacts
make verify-artifacts VERSION=1.2.0
```

## Rollback Procedures

### Automatic Rollback

```bash
# Rollback to previous version
./scripts/rollback_release.sh v1.2.0

# Rollback with specific backup
./scripts/rollback_release.sh v1.2.0 --backup backup_v1.1.0.json
```

### Manual Rollback

```bash
# Revert to previous tag
git checkout v1.1.0
git checkout -b rollback/v1.2.0

# Deploy previous version
./scripts/deploy_environment.sh testnet --version v1.1.0

# Create rollback tag
git tag -a v1.2.1 -m "Rollback v1.2.0 -> v1.1.0"
git push origin v1.2.1
```

## Release Schedule

### Regular Releases

- **Patch Releases**: As needed (critical fixes)
- **Minor Releases**: Every 2-4 weeks (feature updates)
- **Major Releases**: Every 3-6 months (significant changes)

### Release Calendar

```markdown
## Q2 2026 Release Schedule
- **April 20**: v1.2.0 - Patient consent management
- **May 18**: v1.3.0 - Enhanced audit system  
- **June 15**: v2.0.0 - Breaking changes and new architecture
```

## Quality Gates

### Release Requirements

- [ ] All tests passing (100% success rate)
- [ ] Code coverage ≥ 80%
- [ ] Security audit passed
- [ ] Documentation updated
- [ ] Chelog prepared
- [ ] Migration guides ready (major releases)
- [ ] Performance benchmarks met
- [ ] Contract size limits respected

### Release Approval

- **Patch Releases**: Automated approval
- **Minor Releases**: Core maintainer approval
- **Major Releases**: Full team approval
- **Hotfixes**: Security team approval

## Monitoring and Alerts

### Release Monitoring

```bash
# Monitor release health
./scripts/monitor_release.sh v1.2.0

# Set up alerts
./scripts/setup_alerts.sh --release v1.2.0
```

### Alert Conditions

- Deployment failures
- Contract verification failures
- Performance regressions
- Security vulnerabilities
- User-reported issues

## Documentation Updates

### Release Documentation

- Update README with new version
- Update API documentation
- Update migration guides
- Update deployment guides
- Update troubleshooting guides

### Version-specific Documentation

```markdown
docs/releases/v1.2.0/
├── release-notes.md
├── migration-guide.md
├── api-changes.md
├── known-issues.md
└── troubleshooting.md
```

## Support and Maintenance

### Post-release Support

- Monitor user feedback
- Address reported issues promptly
- Provide migration assistance
- Update documentation as needed

### Maintenance Windows

- **Critical Updates**: Immediate deployment
- **Feature Updates**: Scheduled deployment
- **Major Updates**: Planned maintenance window

## Tools and Integration

### Release Tools

- **GitHub Actions**: Automated CI/CD
- **Slack Bot**: Release notifications
- **Email Service**: Release announcements
- **Monitoring**: Release health tracking

### Integration Points

- Package registries
- Documentation sites
- Monitoring systems
- Notification services

---

This release process ensures consistent, reliable, and automated releases that maintain high quality standards and provide clear communication to all stakeholders.
