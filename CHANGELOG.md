# Changelog

All notable changes to the Stellar Uzima Contracts repository will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Removed
- CI workflow definitions (`.github/workflows/ci-gates.yml`) removed from the repository.

### Changed
- `contracts/medical_record_backup` removed from the workspace `exclude` list: the crate already compiled and passed its standalone test suite, so it now builds and tests as part of `cargo check --workspace` / `cargo test --workspace` (#1452)
- `contracts/medical_record_backup/src/test.rs` gained an initialization-guard regression test (`initialize_can_only_run_once`) and an authorization-failure regression test (`register_target_rejects_unauthorized_caller`), covering the init/happy-path/error-path triad required for reintegration (#1451)

### Added
- Canonical address normalization layer for identity and access flows (`identity_registry`)
- Release note generation script (`scripts/generate_release_notes.sh`)
- Semantic versioning discipline for contract interfaces and SDK compatibility
- Memory and CPU budget dashboards per contract
- **Workspace-wide soroban-sdk pin:** All active member crates now inherit `soroban-sdk` from the workspace root via `workspace = true`. The `contracts/upgradeability` crate was the only non-excluded member with a hardcoded version — corrected in this PR.
- `scripts/check_sdk_version.sh` — CI guard that fails the build if any member crate overrides the workspace soroban-sdk pin. Scans all Cargo.toml files including excluded/deferred contracts.
- `docs/VERSIONING_STRATEGY.md` — Documents SDK bump cadence (patch/minor/major), compatibility matrix, deprecation policy, and the CI enforcement mechanism.
- Added note in changelog about `healthcare_compliance` contract's former `22.0.0` drift (issue #828 deferred).
- Standardized re-initialization guard `init_guard` in `libs/governance_commons`
  (`init_guard`/`try_init_guard`/`is_initialized`/`require_initialized`),
  re-exported from `libs/validation_utils`. Documents one-shot init semantics
  (re-init is rejected; admin transfer is a separate, independent operation) and
  is referenced by `docs/SECURITY_CHECKLIST.md` Item 4. Unit tests cover
  init-succeeds-once, init-fails-second-time (Result and panicking variants),
  and admin-transfer-independence.
- Contract versioning and release process implementation
- Automated release scripts and GitHub Actions workflow
- Comprehensive versioning strategy documentation
- Release process documentation with detailed steps
- Changelog format documentation and guidelines
- Version bump automation scripts
- Changelog generation from git history
- Release validation and verification tools
- Automated deployment to testnet
- GitHub release creation with artifacts
- Notification system for release announcements
- Release rollback procedures and tools
- Security audit integration in release process
- WASM contract size validation
- Contract deployment monitoring and health checks
- `docs/MIGRATION_GUIDE.md` — Documented upgrade procedures for storage layout changes and schema migrations (#1203).
- `scripts/check_changelog.sh` — CI validation that changelog entries exist for any modified contracts (#1203).
- `libs/partial_update/` — Reusable `PartialUpdate<T>` generic type with builder pattern for selective field updates, reducing storage costs and race conditions on medical records (#1212).
- `scripts/legacy_migrator.sh` — Offline migration tool for legacy medical record formats with format detection, field mapping, validation, and dry-run mode (#1215).
- `config/feature_flags.json` — Per-contract feature flag configuration with rollout percentages and stage gates (#1222).
- `libs/feature_flags/` — Soroban-compatible feature flag evaluation library with percentage-based rollout and environment overrides (#1222).

### Changed
- Audited all contracts and migrated their `initialize`/`init` entry points to
  the shared `init_guard`, replacing inconsistent per-contract re-init checks.
  Behavior note: a few contracts that previously treated a second `initialize`
  call as a silent no-op now reject it (panic or `AlreadyInitialized`), and
  contracts lacking an `AlreadyInitialized` error variant (escrow,
  fhir_integration, healthcare_data_conversion) gained one.
- `validation_utils` now builds against `soroban-sdk =21.7.7` (was 20.5.0) so the
  re-exported guard shares a single `Env` type with `governance_commons`.
- Enhanced makefile with release automation targets
- Improved CI/CD pipeline with release validation
- Updated project structure for better release management
- Standardized version management across contracts
- Enhanced error handling in deployment scripts
- Improved logging and monitoring capabilities

### Fixed
- Version consistency checks across workspace
- Contract deployment validation issues
- Chelog generation edge cases
- Release script error handling

### Security
- **Centralized Admin & Role Authorization Checks:** Introduced shared `require_admin!(env, caller)` and `require_role!(env, caller, role)` macros in `libs/governance_commons` to eliminate duplicate auth logic across multiple contracts (`anomaly_detector`, `cross_chain_bridge`, `aml`, `audit`, and `rbac`).
- Enhanced security audit integration
- Improved access control validation
- Added security-focused clippy checks
- Enhanced encryption key management validation

### Storage & Schema Changes

| Version | Date | Contract | Change Type | Migration Required |
|---------|------|----------|-------------|--------------------|
| 1.1.0 | 2026-07-25 | medical_records | Storage layout expansion (partial update support) | Yes — see [MIGRATION_GUIDE.md](docs/MIGRATION_GUIDE.md) |
| 1.1.0 | 2026-07-25 | audit | Schema addition (partial update audit events) | No — additive only |
| 1.0.0 | 2026-02-01 | All | Initial storage layout | N/A — first deploy |

#### Legend

- **Storage layout expansion** — New persistent storage keys or changed key encoding.
- **Schema addition** — New fields added to existing contract type definitions.
- **Breaking change** — Storage key or encoding incompatible with previous version.

> When adding storage or schema changes, add a row above **and** update
> `docs/MIGRATION_GUIDE.md` with the specific upgrade procedure.

---

## [1.0.0] - 2026-02-01

### Added
- Initial release of Uzima-Contracts
- Core medical records smart contracts
- Patient registration and management system
- Medical record storage and retrieval
- Role-based access control (patients, doctors, admins)
- Audit logging system with timestamps
- Basic encryption for sensitive data
- Integration with traditional healing metadata
- Contract deployment scripts
- Local development environment setup
- Basic testing framework
- Documentation and API reference

### Fixed
- Initial bug fixes and stability improvements
- Memory management optimizations
- Error handling improvements

### Security
- Initial security implementation
- Basic access control mechanisms
- Data encryption for medical records

---

## Version History

### Development Versions
- **v0.1.0** - Initial development prototype
- **v0.2.0** - Core functionality implementation
- **v0.3.0** - Testing and validation framework
- **v0.4.0** - Security enhancements
- **v0.5.0** - Performance optimizations
- **v0.6.0** - Documentation and API completion
- **v0.7.0** - Integration testing
- **v0.8.0** - Final testing and validation
- **v0.9.0** - Release candidate preparation

### Release Process
This changelog is automatically maintained as part of the release process. 
For more information about the release process, see:
- [Release Process Documentation](docs/RELEASE_PROCESS.md)
- [Versioning Strategy](docs/VERSIONING_STRATEGY.md)
- [Changelog Format Guide](docs/CHANGELOG_FORMAT.md)

### Contributing to Changelog
Changes are automatically categorized based on commit messages using
[Conventional Commits](https://www.conventionalcommits.org/) format:

- `feat:` for new features (Added section)
- `fix:` for bug fixes (Fixed section)
- `docs:` for documentation changes (Changed section)
- `style:` for code style changes (Changed section)
- `refactor:` for code refactoring (Changed section)
- `perf:` for performance improvements (Changed section)
- `test:` for test changes (Changed section)
- `chore:` for maintenance tasks (Changed section)

Breaking changes should be marked with `BREAKING CHANGE:` in the commit message.

### Security Issues
Security-related commits are automatically detected and categorized in the Security section.
Commits containing keywords like 'security', 'cve', 'vulnerability', or 'fix.*security' are marked as security fixes.

### Release Types
- **Major releases** (X.0.0): Breaking changes and significant updates
- **Minor releases** (X.Y.0): New features and enhancements
- **Patch releases** (X.Y.Z): Bug fixes and security updates
- **Pre-releases** (X.Y.Z-alpha.1, X.Y.Z-beta.1, X.Y.Z-rc.1): Development and testing versions

### Migration Information
For major releases, migration guides are provided in the release notes and documentation.
See the [Migration Guide](docs/MIGRATION_GUIDE.md) for detailed instructions.

### Support and Maintenance
- Supported versions: Latest major and minor releases
- Security updates: Provided for supported versions
- Bug fixes: Backported to supported minor releases when applicable
- End-of-life: Announced 6 months before discontinuation

---

*This changelog follows the [Keep a Changelog](https://keepachangelog.com/) guidelines and is automatically maintained as part of the release process.*
