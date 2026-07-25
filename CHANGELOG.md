# Changelog

All notable changes to the Stellar Uzima Contracts repository will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Canonical address normalization layer for identity and access flows (`identity_registry`)
- Release note generation script (`scripts/generate_release_notes.sh`)
- Semantic versioning discipline for contract interfaces and SDK compatibility
- Memory and CPU budget dashboards per contract

### Changed
- (none yet)

### Fixed
- (none yet)

---

To generate release notes for a tagged version run:

```bash
./scripts/generate_release_notes.sh v0.1.0
```
