# Development Guide

This document describes the pinned Soroban CLI setup and how to upgrade it safely.

---

## Soroban CLI Version Pinning

We pin `soroban-cli` to a known-good version to ensure consistency between local development and CI.

- **Pinned version:** compatible with `soroban-sdk = "22.0.0"`

The pin is applied in two places:

1. [`scripts/setup.sh`](../scripts/setup.sh)  
   Installs the CLI with a locked version.
2. [`.github/workflows/release.yml`](../.github/workflows/release.yml)  
   Ensures CI installs the same version.

---

## Checking Installed Version

Run:

```bash
soroban --version
