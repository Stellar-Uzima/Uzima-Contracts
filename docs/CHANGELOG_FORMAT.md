# Changelog Format

## Overview

This document defines the standardized changelog format for the Uzima-Contracts project, following [Keep a Changelog](https://keepachangelog.com/) guidelines with project-specific adaptations.

## Changelog Structure

### File Format

The changelog is maintained in `CHANGELOG.md` at the project root with the following structure:

```markdown
# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Changes that add new functionality
- New features and capabilities

### Changed
- Changes to existing functionality
- Improvements and optimizations

### Deprecated
- Soon-to-be removed features
- Alternative functionality recommended

### Removed
- Removed features (in breaking changes)
- Deprecated functionality removed

### Fixed
- Bug fixes and error corrections
- Issue resolutions

### Security
- Security-related changes
- Vulnerability fixes

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
- Function signature changed
- Storage layout updated

### Security
- Fixed vulnerability

## [1.1.0] - 2026-03-15

### Added
- Feature A
- Feature B

### Fixed
- Bug C fixed

## [1.0.0] - 2026-02-01

### Added
- Initial release
- Core functionality
```

## Section Guidelines

### Unreleased Section

The `Unreleased` section contains changes that will be included in the next release but haven't been officially released yet.

**Purpose:**
- Track upcoming changes
- Allow review before release
- Facilitate release planning

**Management:**
- Add changes as they're merged
- Move to appropriate version section during release
- Clear after release publication

### Added Section

**What belongs here:**
- New contract functions
- New features and capabilities
- New configuration options
- New integration points
- New documentation

**Examples:**
```markdown
### Added
- Patient consent management system with granular permissions
- Enhanced audit logging with cryptographic timestamps
- Support for medical imaging metadata (DICOM compatibility)
- Multi-region deployment capabilities
- Real-time contract monitoring dashboard
```

### Changed Section

**What belongs here:**
- Improvements to existing functionality
- Performance optimizations
- Configuration changes
- API enhancements (non-breaking)
- Updated dependencies

**Examples:**
```markdown
### Changed
- Optimized WASM contract sizes by 15% through code refactoring
- Improved error messages for better developer experience
- Updated Soroban SDK to v21.7.7 for better performance
- Enhanced network configuration management
- Streamlined deployment process with better error handling
```

### Deprecated Section

**What belongs here:**
- Features that will be removed in future releases
- Alternative functionality recommendations
- Migration path information

**Examples:**
```markdown
### Deprecated
- Legacy patient registration function (use register_patient_v2 instead)
- Old encryption algorithm (will be removed in v2.0.0)
- Direct contract storage access (use storage interface)
```

### Removed Section

**What belongs here:**
- Features removed in breaking changes
- Previously deprecated functionality
- API endpoints removed

**Examples:**
```markdown
### Removed
- Legacy audit logging system (replaced with enhanced version)
- Old contract storage format (migration required)
- Deprecated patient consent methods
```

### Fixed Section

**What belongs here:**
- Bug fixes and error corrections
- Performance issue resolutions
- Edge case handling improvements
- Memory leak fixes

**Examples:**
```markdown
### Fixed
- Memory leak in record encryption function
- Validation error in patient registration with special characters
- Network timeout issues during contract deployment
- Race condition in concurrent record access
- Incorrect gas estimation for large transactions
```

### Security Section

**What belongs here:**
- Security vulnerability fixes
- Enhanced security measures
- Access control improvements
- Encryption enhancements

**Examples:**
```markdown
### Security
- Fixed potential data exposure in audit logs (CVE-2026-1234)
- Enhanced encryption key management with secure storage
- Improved access control validation for patient records
- Added rate limiting to prevent DoS attacks
- Updated cryptographic libraries for latest security patches
```

### Breaking Changes Section

**What belongs here:**
- Changes that require user intervention
- API modifications that break compatibility
- Storage layout changes requiring migration
- Configuration format changes

**Examples:**
```markdown
### Breaking Changes
- `register_patient` function signature updated to include consent metadata (see migration guide)
- Contract storage layout optimized for better performance (requires data migration)
- Network configuration format changed (update config files before deployment)
- Removed support for legacy encryption keys (migration required)
```

## Entry Format Guidelines

### Entry Structure

Each changelog entry should follow this format:

```markdown
- [Component]: [Description] ([Optional Context])
```

**Components:**
- **Contracts**: Smart contract changes
- **CLI**: Command-line interface changes
- **Deployment**: Deployment process changes
- **Documentation**: Documentation updates
- **Testing**: Test framework changes
- **CI/CD**: Pipeline changes
- **Security**: Security-related changes

**Examples:**
```markdown
- Contracts: Added patient consent management with role-based permissions
- CLI: Enhanced deployment script with rollback support
- Documentation: Updated API reference with new function signatures
- Testing: Added integration tests for cross-chain functionality
- Security: Fixed potential data exposure in audit logs
```

### Description Guidelines

**Keep descriptions:**
- Clear and concise
- User-focused
- Action-oriented
- Specific and detailed

**Good examples:**
```markdown
- Added patient consent management with granular permissions
- Fixed memory leak in record encryption function
- Improved error messages for better debugging
```

**Avoid:**
```markdown
- Fixed some bugs
- Updated stuff
- Made improvements
```

### Context and References

**Add context when helpful:**
- Issue numbers: `(fixes #123)`
- Pull requests: `(via #456)`
- Breaking change notices: `(breaking change)`
- Security advisories: `(CVE-2026-1234)`

**Examples:**
```markdown
- Fixed patient registration validation (fixes #123)
- Added multi-region deployment (via #456)
- Updated contract storage format (breaking change)
- Fixed data exposure vulnerability (CVE-2026-1234)
```

## Version Entry Format

### Standard Version Entry

```markdown
## [1.2.0] - 2026-04-20

### Added
- Patient consent management system with granular permissions
- Enhanced audit logging with cryptographic timestamps
- Support for medical imaging metadata (DICOM compatibility)

### Fixed
- Memory leak in record encryption function (fixes #123)
- Validation error in patient registration with special characters
- Network timeout issues during contract deployment

### Changed
- Optimized WASM contract sizes by 15% through code refactoring
- Improved error messages for better developer experience
- Updated Soroban SDK to v21.7.7 for better performance

### Breaking Changes
- `register_patient` function signature updated (see migration guide)
- Contract storage layout optimized (requires data migration)

### Security
- Fixed potential data exposure in audit logs (CVE-2026-1234)
- Enhanced encryption key management with secure storage
```

### Pre-release Version Entry

```markdown
## [1.2.0-alpha.1] - 2026-04-15

### Added
- Patient consent management system (alpha testing)
- Enhanced audit logging (experimental)

### Changed
- Updated dependencies for testing

### Known Issues
- Patient consent may have edge cases with complex permissions
- Audit logging performance needs optimization
```

### Patch Release Entry

```markdown
## [1.1.1] - 2026-04-10

### Fixed
- Critical memory leak in record encryption (fixes #456)
- Security vulnerability in audit logs (CVE-2026-1234)
- Network timeout during deployment (fixes #789)

### Security
- Fixed potential data exposure in audit logs
- Enhanced input validation for patient data
```

## Changelog Management

### Adding Entries

**During Development:**
1. Add entries to `Unreleased` section as changes are merged
2. Use appropriate section headers
3. Include relevant context and references
4. Keep descriptions clear and user-focused

**Example addition:**
```markdown
## [Unreleased]

### Added
- Patient consent management system with granular permissions

### Fixed
- Memory leak in record encryption function (fixes #123)
```

### Release Process

**During Release:**
1. Review `Unreleased` section
2. Add release date and version number
3. Move entries to version section
4. Add `Breaking Changes` section if needed
5. Clear `Unreleased` section
6. Validate changelog format

**Automation:**
```bash
# Generate changelog from git history
make generate-changelog VERSION=1.2.0

# Validate changelog format
make validate-changelog

# Update changelog for release
make update-changelog VERSION=1.2.0
```

### Validation Rules

**Format validation:**
- Version format: `[X.Y.Z] - YYYY-MM-DD`
- Section headers: `### SectionName`
- Entry format: `- Component: Description`
- Proper markdown syntax

**Content validation:**
- No empty sections
- Proper issue references
- Breaking changes clearly marked
- Security issues properly documented

## Automation Tools

### Changelog Generation

```bash
# Generate from git commits
./scripts/generate_changelog.sh --from v1.1.0 --to v1.2.0

# Generate with custom format
./scripts/generate_changelog.sh --version 1.2.0 --format markdown

# Interactive generation
./scripts/generate_changelog.sh --interactive
```

### Changelog Validation

```bash
# Validate format
make validate-changelog

# Check for missing entries
make check-changelog-completeness

# Verify issue references
make verify-changelog-references
```

### Changelog Updates

```bash
# Update for release
make update-changelog VERSION=1.2.0

# Add entry manually
./scripts/add_changelog_entry.sh --section Added --message "New feature added"

# Batch update from commits
./scripts/batch_update_changelog.sh --since v1.1.0
```

## Integration Points

### Release Integration

The changelog integrates with:

- **Release Process**: Automatic generation and validation
- **GitHub Releases**: Changelog used as release notes
- **Documentation**: Referenced in user guides
- **Migration Guides**: Linked from breaking changes

### Tool Integration

- **Git Hooks**: Pre-commit changelog validation
- **CI/CD**: Automated changelog checks
- **Release Scripts**: Changelog generation and updates
- **Documentation Tools**: Changelog inclusion in docs

## Best Practices

### Writing Guidelines

1. **Be User-Focused**: Write for users, not developers
2. **Be Specific**: Include relevant details and context
3. **Be Consistent**: Use standard format and terminology
4. **Be Complete**: Don't omit significant changes
5. **Be Timely**: Update changelog as changes happen

### Review Process

1. **Peer Review**: Have team members review entries
2. **User Review**: Ensure entries are understandable to users
3. **Technical Review**: Verify technical accuracy
4. **Format Review**: Check compliance with format guidelines

### Maintenance

1. **Regular Updates**: Keep changelog current
2. **Version Cleanup**: Archive old versions if needed
3. **Format Updates**: Update format as project evolves
4. **Documentation**: Keep this format guide updated

## Examples

### Complete Chelog Example

```markdown
# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Cross-chain data synchronization capabilities
- Enhanced patient consent management with audit trails

### Fixed
- Contract deployment timeout issues (fixes #789)

## [1.2.0] - 2026-04-20

### Added
- Patient consent management system with granular permissions
- Enhanced audit logging with cryptographic timestamps
- Support for medical imaging metadata (DICOM compatibility)
- Multi-region deployment capabilities
- Real-time contract monitoring dashboard

### Fixed
- Memory leak in record encryption function (fixes #123)
- Validation error in patient registration with special characters
- Network timeout issues during contract deployment
- Race condition in concurrent record access

### Changed
- Optimized WASM contract sizes by 15% through code refactoring
- Improved error messages for better developer experience
- Updated Soroban SDK to v21.7.7 for better performance
- Enhanced network configuration management
- Streamlined deployment process with better error handling

### Breaking Changes
- `register_patient` function signature updated to include consent metadata (see migration guide)
- Contract storage layout optimized for better performance (requires data migration)
- Network configuration format changed (update config files before deployment)

### Security
- Fixed potential data exposure in audit logs (CVE-2026-1234)
- Enhanced encryption key management with secure storage
- Improved access control validation for patient records
- Added rate limiting to prevent DoS attacks

## [1.1.0] - 2026-03-15

### Added
- Basic patient consent management
- Enhanced audit logging
- Medical imaging support

### Fixed
- Patient registration validation issues
- Contract deployment problems

## [1.0.0] - 2026-02-01

### Added
- Initial release
- Core medical records functionality
- Patient registration system
- Basic audit logging
- Contract deployment tools
```

---

This changelog format ensures consistent, comprehensive, and user-friendly documentation of all project changes.
