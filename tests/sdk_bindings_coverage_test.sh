#!/usr/bin/env bash
# Regression tests for the SDK binding coverage gate (#1971).
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

TMP_DIR="$(mktemp -d)"
CONTRACT_FILE="contracts/contract_monitoring/src/lib.rs"
cp "$CONTRACT_FILE" "$TMP_DIR/lib.rs.bak"
trap 'cp "$TMP_DIR/lib.rs.bak" "$CONTRACT_FILE" 2>/dev/null || true; rm -rf "$TMP_DIR"' EXIT

fail() {
  echo "FAIL: $1" >&2
  exit 1
}

echo "== current bindings pass =="
SDK_BINDINGS_BASE=HEAD npm run sdk:check --silent >"$TMP_DIR/pass.log" 2>&1 \
  || { cat "$TMP_DIR/pass.log" >&2; fail "clean tree should pass"; }
grep -q "All 4 registered contracts have bindings in typescript, python, kotlin, swift." "$TMP_DIR/pass.log" \
  || fail "expected four-language coverage summary"

echo "== new contract with a public interface without bindings fails =="
cp schemas/interface-registry/registry.json "$TMP_DIR/new-contract.json"
node - "$TMP_DIR/new-contract.json" <<'NODE'
const fs = require('fs');
const file = process.argv[2];
const registry = JSON.parse(fs.readFileSync(file, 'utf8'));
registry.contracts.brand_new_contract = {
  version: '1.0.0',
  description: 'New contract with no SDK bindings',
  interfaces: {
    do_something_new: {
      description: 'New public endpoint',
      args: [],
      returns: 'void',
      state_mutation: false,
    },
  },
};
fs.writeFileSync(file, JSON.stringify(registry, null, 2));
NODE

if node scripts/generate-sdk-types.mjs --check \
  --registry "$TMP_DIR/new-contract.json" \
  --report "$TMP_DIR/new-contract.txt" >"$TMP_DIR/new-contract.log" 2>&1; then
  cat "$TMP_DIR/new-contract.log" >&2
  fail "new public interface without bindings should fail"
fi
grep -q "brand_new_contract: no typescript client binding" "$TMP_DIR/new-contract.log" \
  || fail "missing TypeScript binding not reported"
grep -q "brand_new_contract: no python client binding" "$TMP_DIR/new-contract.log" \
  || fail "missing Python binding not reported"
grep -q "brand_new_contract: no kotlin client binding" "$TMP_DIR/new-contract.log" \
  || fail "missing Kotlin binding not reported"
grep -q "brand_new_contract: no swift client binding" "$TMP_DIR/new-contract.log" \
  || fail "missing Swift binding not reported"
grep -q "SDK_BINDINGS_MISSING=[1-9]" "$TMP_DIR/new-contract.txt" \
  || fail "drift report should record missing bindings"

echo "== new interface on an existing contract without bindings fails =="
cp schemas/interface-registry/registry.json "$TMP_DIR/new-interface.json"
node - "$TMP_DIR/new-interface.json" <<'NODE'
const fs = require('fs');
const file = process.argv[2];
const registry = JSON.parse(fs.readFileSync(file, 'utf8'));
registry.contracts.medical_records.interfaces.brand_new_endpoint = {
  description: 'New medical records endpoint',
  args: [],
  returns: 'void',
  state_mutation: false,
};
fs.writeFileSync(file, JSON.stringify(registry, null, 2));
NODE

if node scripts/generate-sdk-types.mjs --check \
  --registry "$TMP_DIR/new-interface.json" >"$TMP_DIR/new-interface.log" 2>&1; then
  cat "$TMP_DIR/new-interface.log" >&2
  fail "new interface without regenerated bindings should fail"
fi
grep -q "medical_records.brand_new_endpoint: no typescript binding" "$TMP_DIR/new-interface.log" \
  || fail "missing interface binding not reported"

echo "== newly added public contract function fails =="
python3 - "$CONTRACT_FILE" <<'PY'
import sys

path = sys.argv[1]
with open(path, encoding='utf-8') as handle:
    content = handle.read()
content = content.replace(
    'impl ContractMonitoring {\n',
    'impl ContractMonitoring {\n    pub fn brand_new_endpoint(_env: Env) {}\n',
    1,
)
with open(path, 'w', encoding='utf-8') as handle:
    handle.write(content)
PY

if SDK_BINDINGS_BASE=HEAD node scripts/check-sdk-bindings.mjs >"$TMP_DIR/new-function.log" 2>&1; then
  cat "$TMP_DIR/new-function.log" >&2
  fail "new public contract function without registry entry should fail"
fi
grep -q "contract_monitoring.brand_new_endpoint: new public interface has no SDK binding" "$TMP_DIR/new-function.log" \
  || fail "new public contract function not reported"
cp "$TMP_DIR/lib.rs.bak" "$CONTRACT_FILE"

echo "sdk binding coverage gate: ok"
