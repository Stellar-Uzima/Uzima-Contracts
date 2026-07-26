#!/usr/bin/env bash
# Local CI helper: verify committed SDK bindings match generator output.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

node scripts/generate-sdk-types.mjs --check
