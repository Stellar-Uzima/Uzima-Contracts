#!/usr/bin/env bash
# ============================================================================
# validate_multi_tenant_config.sh
# Validates the multi-tenant configuration file against the schema and checks
# that all referenced contracts exist in the workspace.
#
# Usage:
#   ./scripts/validate_multi_tenant_config.sh [config_file]
# ============================================================================
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

info()  { echo -e "${GREEN}[OK]${NC}    $1"; }
warn()  { echo -e "${YELLOW}[WARN]${NC}  $1"; }
fail()  { echo -e "${RED}[FAIL]${NC}  $1"; }

CONFIG="${1:-$ROOT_DIR/config/multi_tenant.json}"
ERRORS=0

echo ""
echo "Multi-tenant configuration validator"
echo "─────────────────────────────────────"

# ── 1. JSON validity ──────────────────────────────────────────────────────────
if ! python3 -m json.tool "$CONFIG" >/dev/null 2>&1; then
  fail "$CONFIG is not valid JSON"
  exit 1
fi
info "JSON syntax valid"

# ── 2. Required fields ────────────────────────────────────────────────────────
for field in version tenants; do
  if python3 -c "import json; d=json.load(open('$CONFIG')); assert '$field' in d" 2>/dev/null; then
    info "Required field '$field' present"
  else
    fail "Missing required field: $field"
    ERRORS=$((ERRORS + 1))
  fi
done

# ── 3. Tenant inheritance ─────────────────────────────────────────────────────
python3 -c "
import json, sys

with open('$CONFIG') as f:
    config = json.load(f)

tenants = config.get('tenants', {})
errors = 0

for name, tenant in tenants.items():
    if name == '_default':
        continue
    parent = tenant.get('extends')
    if parent and parent not in tenants:
        print(f'FAIL: Tenant \"{name}\" extends unknown parent \"{parent}\"')
        errors += 1
    elif parent:
        print(f'OK:   Tenant \"{name}\" extends \"{parent}\"')

sys.exit(errors)
" 2>/dev/null || ERRORS=$((ERRORS + 1))

# ── 4. Contract existence check ───────────────────────────────────────────────
python3 -c "
import json, os

with open('$CONFIG') as f:
    config = json.load(f)

contracts_dir = '$ROOT_DIR/contracts'
errors = 0

for name, tenant in config.get('tenants', {}).items():
    enabled = tenant.get('contracts', {}).get('enabled', [])
    for contract in enabled:
        manifest = os.path.join(contracts_dir, contract, 'Cargo.toml')
        if not os.path.exists(manifest):
            print(f'WARN: Tenant \"{name}\" references contract \"{contract}\" but Cargo.toml not found')
        else:
            print(f'OK:   Contract \"{contract}\" exists (tenant: {name})')

if errors > 0:
    exit(1)
" 2>/dev/null || true

# ── Summary ───────────────────────────────────────────────────────────────────
echo ""
if [[ $ERRORS -gt 0 ]]; then
  fail "Validation completed with $ERRORS error(s)"
  exit 1
else
  info "Multi-tenant configuration is valid"
  exit 0
fi
