# Dead-Code and Unused Dependency Scanning

## Overview

The `scripts/deadcode_scan.sh` script performs repository-wide dead-code detection
and unused dependency scanning for both Rust and JavaScript packages.

## What It Checks

### Rust
- **Unused dependencies**: Uses `cargo-udeps` (if installed) to find Cargo.toml dependencies
  that are not actually used in source code
- **Fallback analysis**: If `cargo-udeps` is unavailable, checks workspace dependencies
  against source imports
- **Dead code warnings**: Runs `cargo clippy` and captures dead_code/unused warnings

### JavaScript
- **Unused npm packages**: Uses `npx depcheck` to find package.json dependencies
  that are not imported in any JS/TS source file

## Usage

`ash
# Full scan (Rust + JS)
./scripts/deadcode_scan.sh

# Rust only
./scripts/deadcode_scan.sh --rust-only

# JavaScript only
./scripts/deadcode_scan.sh --js-only

# Via npm
npm run deadcode:scan
npm run deadcode:scan:rust
npm run deadcode:scan:js
`"
# Dead-Code and Unused Dependency Scanning  ## Overview  The `scripts/deadcode_scan.sh` script performs repository-wide dead-code detection and unused dependency scanning for both Rust and JavaScript packages.  ## What It Checks  ### Rust - **Unused dependencies**: Uses `cargo-udeps` (if installed) to find Cargo.toml dependencies   that are not actually used in source code - **Fallback analysis**: If `cargo-udeps` is unavailable, checks workspace dependencies   against source imports - **Dead code warnings**: Runs `cargo clippy` and captures dead_code/unused warnings  ### JavaScript - **Unused npm packages**: Uses `npx depcheck` to find package.json dependencies   that are not imported in any JS/TS source file  ## Usage  `ash # Full scan (Rust + JS) ./scripts/deadcode_scan.sh  # Rust only ./scripts/deadcode_scan.sh --rust-only  # JavaScript only ./scripts/deadcode_scan.sh --js-only  # Via npm npm run deadcode:scan npm run deadcode:scan:rust npm run deadcode:scan:js += "
# Dead-Code and Unused Dependency Scanning  ## Overview  The `scripts/deadcode_scan.sh` script performs repository-wide dead-code detection and unused dependency scanning for both Rust and JavaScript packages.  ## What It Checks  ### Rust - **Unused dependencies**: Uses `cargo-udeps` (if installed) to find Cargo.toml dependencies   that are not actually used in source code - **Fallback analysis**: If `cargo-udeps` is unavailable, checks workspace dependencies   against source imports - **Dead code warnings**: Runs `cargo clippy` and captures dead_code/unused warnings  ### JavaScript - **Unused npm packages**: Uses `npx depcheck` to find package.json dependencies   that are not imported in any JS/TS source file  ## Usage  `ash # Full scan (Rust + JS) ./scripts/deadcode_scan.sh  # Rust only ./scripts/deadcode_scan.sh --rust-only  # JavaScript only ./scripts/deadcode_scan.sh --js-only  # Via npm npm run deadcode:scan npm run deadcode:scan:rust npm run deadcode:scan:js += 

- Rust toolchain with `clippy` component
- Optional: `cargo-udeps` (`cargo install cargo-udeps`) for more accurate Rust unused dep detection
- Optional: `npx` and `depcheck` for JavaScript unused dependency detection

## Output

Reports are saved to `.deadcode-reports/` directory with timestamps.
The script exits with code 1 if any violations are found, making it suitable for CI.
