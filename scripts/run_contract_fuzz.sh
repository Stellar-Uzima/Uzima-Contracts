#!/bin/bash

set -euo pipefail

PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

cd "${PROJECT_ROOT}"

PROPTEST_CASES="${PROPTEST_CASES:-40}" \
PROPTEST_DISABLE_FAILURE_PERSISTENCE=1 \
cargo test -p contract_behavior_fuzzing --test sut_token_fuzz --test token_sale_fuzz --test identity_registry_fuzz
