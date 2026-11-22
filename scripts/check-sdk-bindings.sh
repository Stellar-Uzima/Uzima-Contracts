#!/usr/bin/env bash
# Local/CI helper: verify committed SDK bindings match generator output, cover
# every registered public interface in all four languages, and reject newly
# added public contract functions that have no registry entry/binding.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

node scripts/generate-sdk-types.mjs --check "$@"
node scripts/check-sdk-bindings.mjs
