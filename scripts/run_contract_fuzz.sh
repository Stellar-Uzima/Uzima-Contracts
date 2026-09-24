#!/bin/bash

set -euo pipefail

PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

cd "${PROJECT_ROOT}"

# Failure persistence is intentionally left ENABLED (proptest's default) so
# panic witnesses are written to each fuzz target's proptest-regressions/
# directory and can be uploaded as CI artifacts for investigation. Set
# PROPTEST_DISABLE_FAILURE_PERSISTENCE=1 in the environment to opt out.
PROPTEST_CASES="${PROPTEST_CASES:-40}" \
cargo test -p contract_behavior_fuzzing --test sut_token_fuzz --test token_sale_fuzz --test identity_registry_fuzz
