#!/usr/bin/env bash
set -euo pipefail

# Soroban CLI pinned version
SOROBAN_VERSION="23.1.4"

echo "Checking for soroban-cli $SOROBAN_VERSION..."

# Install Soroban CLI if missing or wrong version
if ! command -v soroban &>/dev/null || [[ "$(soroban --version)" != *"$SOROBAN_VERSION"* ]]; then
  echo "Installing soroban-cli $SOROBAN_VERSION..."
  cargo install --locked --version "$SOROBAN_VERSION" soroban-cli
else
  echo "soroban-cli $SOROBAN_VERSION already installed."
fi

echo "Done. Current version:"
soroban --version