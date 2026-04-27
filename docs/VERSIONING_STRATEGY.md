# Versioning Strategy

## Overview

This document outlines the versioning strategy for the Uzima-Contracts project, following Semantic Versioning (SemVer) to ensure clear, predictable version increments that communicate the nature of changes.

## Semantic Versioning (SemVer)

We adopt the [Semantic Versioning 2.0.0](https://semver.org/) standard with the format `MAJOR.MINOR.PATCH`.

### Version Format

```
MAJOR.MINOR.PATCH[-PRERELEASE[+BUILD]]
```

- **MAJOR**: Incompatible API changes that break existing functionality
- **MINOR**: New functionality added in a backward-compatible manner  
- **PATCH**: Backward-compatible bug fixes
- **PRERELEASE**: Pre-release versions (alpha, beta, rc) (optional)
- **BUILD**: Build metadata (optional)

### Version Examples

- `1.0.0` - First stable release
- `1.1.0` - New features added
- `1.1.1` - Bug fix release
- `2.0.0` - Breaking changes introduced
- `1.2.0-alpha.1` - Pre-release version
- `1.2.0-beta.2+20230401` - Pre-release with build metadata

## Change Classification

### MAJOR Version Changes

**MAJOR** version increments indicate breaking changes that require user intervention:

#### Contract Interface Changes
- Function signature modifications
- Parameter type changes
- Return value modifications
- Function removal or renaming

#### Storage Structure Changes
- Data layout modifications in contract storage
- Key structure changes
- Migration requirements for existing data

#### Protocol Changes
- Network compatibility breaking changes
- Stellar protocol version requirements
- Soroban framework compatibility changes

#### Examples
```rust
// BEFORE (v1.x.x)
pub fn add_patient(&self, patient_id: &str, data: &Bytes) -> Result<(), Error>

// AFTER (v2.0.0) - Breaking: parameter order changed
pub fn add_patient(&self, data: &Bytes, patient_id: &str) -> Result<(), Error>
```

### MINOR Version Changes

**MINOR** version increments add new functionality without breaking changes:

#### New Features
- New contract functions added
- Additional optional parameters to existing functions
- New events emitted
- Enhanced error types with backward compatibility

#### Examples
```rust
// v1.1.0 - New function added (backward compatible)
pub fn get_patient_history(&self, patient_id: &str, from_date: Option<u64>) -> Result<Vec<Record>, Error>

// v1.2.0 - New optional parameter (backward compatible)  
pub fn update_record(&self, record_id: &str, data: &Bytes, metadata: Option<Bytes>) -> Result<(), Error>
```

### PATCH Version Changes

**PATCH** version increments fix bugs without changing functionality:

#### Bug Fixes
- Logic errors corrected
- Edge case handling improved
- Performance optimizations
- Security vulnerability fixes
- Documentation corrections

#### Examples
```rust
// v1.1.1 - Bug fix: proper validation
pub fn add_record(&self, patient_id: &str, data: &Bytes) -> Result<(), Error> {
    // Fixed: Added proper validation
    if patient_id.is_empty() {
        return Err(Error::InvalidPatientId);
    }
    // ... rest of implementation
}
```

## Pre-release Versions

### Pre-release Identifiers

- `alpha` - Early development, unstable, API may change
- `beta` - Feature complete, testing phase, API stable
- `rc` - Release candidate, final testing before stable release

### Pre-release Examples

```
1.2.0-alpha.1    # First alpha of v1.2.0
1.2.0-alpha.2    # Second alpha of v1.2.0
1.2.0-beta.1     # First beta of v1.2.0
1.2.0-rc.1       # First release candidate
1.2.0            # Stable release
```

### Pre-release Rules

1. Pre-release versions have lower precedence than the associated normal version
2. Build metadata MUST be ignored when determining version precedence
3. Pre-release versions are for testing and development only

## Version Bumping Guidelines

### Automated Version Bumping

Use the provided release automation:

```bash
# Patch release (bug fixes)
make release VERSION=1.1.1

# Minor release (new features)  
make release VERSION=1.2.0

# Major release (breaking changes)
make release VERSION=2.0.0

# Pre-release
make release VERSION=1.2.0-alpha.1
```

### Manual Version Updates

If manual version updates are needed:

1. Update `Cargo.toml` workspace version
2. Update individual contract versions if needed
3. Update documentation references
4. Create appropriate git tags
5. Update changelog

### Version Validation

The release process includes validation:

- Semantic version format validation
- Git tag existence checks
- Changelog entry verification
- Contract version consistency checks

## Contract Versioning

### Contract Version Storage

Each contract maintains its version information:

```rust
// Contract version storage
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

pub fn get_version(&self) -> Result<String, Error> {
    Ok(CONTRACT_VERSION.to_string())
}
```

### Version Compatibility

- Contracts expose version via `get_version()` function
- Migration scripts check version compatibility
- Upgrade contracts validate target versions

## Release Branching Strategy

### Branch Structure

```
main                 # Stable releases (v1.x.x, v2.x.x)
├── develop          # Development branch (next minor/major)
├── release/v1.2.0   # Release preparation branch
├── hotfix/v1.1.1    # Critical fixes for current release
└── feature/*        # Feature branches
```

### Release Process Flow

1. **Feature Development**: Branch from `develop`
2. **Integration**: Merge to `develop` via PR
3. **Release Preparation**: Create `release/vX.Y.Z` branch
4. **Testing**: Final testing on release branch
5. **Release**: Merge to `main` and tag
6. **Hotfixes**: Branch from `main` for critical fixes

## Version Communication

### Changelog Requirements

Every version release MUST include:

- Version number and release date
- Breaking changes (if any)
- New features (if any)
- Bug fixes (if any)
- Migration instructions (if needed)
- Security considerations (if applicable)

### Release Notes Template

```markdown
## [1.2.0] - 2026-04-20

### Added
- New feature X
- Enhanced functionality Y

### Fixed  
- Bug Z resolved
- Performance issue fixed

### Changed
- Improved process A
- Updated dependency B

### Breaking Changes
- Function signature changed (migration required)
- Storage layout updated (see migration guide)

### Security
- Fixed vulnerability CVE-XXXX-XXXX
```

## Version Enforcement

### CI/CD Validation

The CI pipeline enforces version consistency:

- Version format validation
- Changelog existence checks
- Tag verification
- Contract version consistency

### Automated Checks

```yaml
# Example CI validation
- name: Validate Version
  run: |
    # Check semantic version format
    echo "${{ github.ref_name }}" | grep -E '^[0-9]+\.[0-9]+\.[0-9]+(-[a-zA-Z0-9]+)?$'
    
    # Verify changelog entry exists
    grep -q "## [${{ github.ref_name }}]" CHANGELOG.md
    
    # Check contract versions
    make check-versions
```

## Migration Support

### Version Compatibility Matrix

| From Version | To Version | Migration Required | Automated |
|--------------|------------|-------------------|-----------|
| 1.0.x        | 1.1.x      | No                | N/A       |
| 1.0.x        | 2.0.0      | Yes               | Yes       |
| 1.1.x        | 1.2.x      | No                | N/A       |
| 1.1.x        | 2.0.0      | Yes               | Yes       |

### Migration Tools

- Automated migration scripts for major versions
- Data validation tools
- Rollback procedures
- Migration verification tests

## Best Practices

### Development Guidelines

1. **Plan Breaking Changes Carefully**: Major versions should be well-planned
2. **Maintain Backward Compatibility**: Prefer minor versions when possible
3. **Document Changes Thoroughly**: Clear changelogs and migration guides
4. **Test Extensively**: Comprehensive testing for all version changes
5. **Communicate Early**: Announce breaking changes well in advance

### Release Frequency

- **Patch releases**: As needed for critical fixes
- **Minor releases**: Every 2-4 weeks for feature updates
- **Major releases**: Every 3-6 months for significant changes

### Quality Gates

- All tests must pass
- Code coverage requirements met
- Security scans completed
- Documentation updated
- Migration guides prepared (for major releases)

## Tools and Automation

### Version Management Tools

- `make release` - Automated release process
- `make check-versions` - Version consistency validation
- `scripts/version_bump.sh` - Manual version bumping
- `scripts/migrate.sh` - Migration assistance

### Integration Points

- GitHub Actions for automated releases
- Changelog generation from git history
- Contract deployment with version tagging
- Notification systems for release announcements

---

This versioning strategy ensures predictable, maintainable releases that clearly communicate the impact of changes to all stakeholders.
