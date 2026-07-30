#!/usr/bin/env bash
# Generate typed bindings for mobile SDK and Python SDK from the contract registry.
#
# Usage:
#   ./scripts/generate_bindings.sh              # generate all bindings
#   ./scripts/generate_bindings.sh --python     # generate Python bindings only
#   ./scripts/generate_bindings.sh --typescript  # generate TypeScript bindings only
#   ./scripts/generate_bindings.sh --check       # verify bindings are up to date
#
# This script reads schemas/interface-registry/registry.json and generates
# type-safe bindings for each target SDK language.

set -euo pipefail

ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
REGISTRY_FILE="$ROOT_DIR/schemas/interface-registry/registry.json"
PYTHON_OUTPUT="$ROOT_DIR/mobile-sdk/python/uzima_sdk/contract_bindings.py"
TYPESCRIPT_OUTPUT="$ROOT_DIR/mobile-sdk/core/src/generated/contract-bindings.ts"

# Parse arguments
GENERATE_ALL=true
GENERATE_PYTHON=false
GENERATE_TYPESCRIPT=false
CHECK_MODE=false

for arg in "$@"; do
    case $arg in
        --python) GENERATE_ALL=false; GENERATE_PYTHON=true ;;
        --typescript) GENERATE_ALL=false; GENERATE_TYPESCRIPT=true ;;
        --check) CHECK_MODE=true ;;
        --help|-h)
            echo "Usage: $0 [--python] [--typescript] [--check]"
            exit 0
            ;;
        *)
            echo "Unknown argument: $arg"
            exit 1
            ;;
    esac
done

if $GENERATE_ALL; then
    GENERATE_PYTHON=true
    GENERATE_TYPESCRIPT=true
fi

# Validate registry exists
if [[ ! -f "$REGISTRY_FILE" ]]; then
    echo "ERROR: Registry file not found: $REGISTRY_FILE"
    echo "Run 'node scripts/abi-compat.mjs' first to generate the registry."
    exit 1
fi

echo "Contract Interface Registry Bindings Generator"
echo "=============================================="
echo "Registry: $REGISTRY_FILE"
echo ""

# ---------------------------------------------------------------------------
# Python bindings generator
# ---------------------------------------------------------------------------
generate_python() {
    echo "Generating Python bindings..."

    cat > "$PYTHON_OUTPUT" << 'PYTHON_HEADER'
"""
Auto-generated contract bindings for the Uzima Python SDK.

DO NOT EDIT MANUALLY — run `./scripts/generate_bindings.sh` instead.
Generator version: 1.0.0
"""

from __future__ import annotations

from dataclasses import dataclass
from enum import Enum
from typing import Dict, List, Optional
from typing import Literal

# ==================== Contract Enums ====================

PYTHON_HEADER

    # Use node to parse JSON and generate Python code
    node -e "
const fs = require('fs');
const registry = JSON.parse(fs.readFileSync('$REGISTRY_FILE', 'utf8'));

// Generate enums from shared_types
const shared = registry.contracts.find(c => c.name === 'shared_types');
if (shared && shared.enums) {
    for (const en of shared.enums) {
        console.log('class ' + en.name + '(str, Enum):');
        console.log('    \"\"\"' + (en.description || '') + '\"\"\"');
        for (const v of en.values) {
            console.log('    ' + v.member + ' = \"' + v.value + '\"  # ' + (v.description || ''));
        }
        console.log('');
    }
}

console.log('# ==================== Contract Dataclasses ====================');
console.log('');

// Generate dataclasses from interfaces
for (const contract of registry.contracts) {
    if (contract.name === 'shared_types') continue;
    if (!contract.interfaces) continue;

    for (const [name, iface] of Object.entries(contract.interfaces)) {
        if (!iface.fields && !iface.args) continue;

        const fields = iface.fields || iface.args || [];
        if (fields.length === 0) continue;

        const className = name.charAt(0).toUpperCase() + name.slice(1).replace(/_([a-z])/g, (_, c) => c.toUpperCase());
        console.log('@dataclass');
        console.log('class ' + className + ':');
        console.log('    \"\"\"' + (iface.description || name) + '\"\"\"');

        for (const f of fields) {
            const pyType = mapType(f.type || 'String');
            const opt = f.optional ? ' = None' : '';
            const snakeName = f.name.replace(/([A-Z])/g, '_$1').toLowerCase();
            console.log('    ' + snakeName + ': ' + pyType + opt);
        }
        console.log('');
    }
}

function mapType(t) {
    if (t === 'String' || t === 'string') return 'str';
    if (t === 'u32' || t === 'u64' || t === 'number') return 'int';
    if (t === 'bool') return 'bool';
    if (t.endsWith('[]')) return 'List[' + mapType(t.slice(0, -2)) + ']';
    if (t.startsWith('Option<')) return 'Optional[' + mapType(t.slice(7, -1)) + ']';
    return t;
}
" >> "$PYTHON_OUTPUT"

    echo "  Written: $PYTHON_OUTPUT"
}

# ---------------------------------------------------------------------------
# TypeScript bindings generator
# ---------------------------------------------------------------------------
generate_typescript() {
    echo "Generating TypeScript bindings..."

    mkdir -p "$(dirname "$TYPESCRIPT_OUTPUT")"

    cat > "$TYPESCRIPT_OUTPUT" << 'TS_HEADER'
/**
 * Auto-generated contract bindings for the Uzima TypeScript SDK.
 *
 * DO NOT EDIT MANUALLY — run `./scripts/generate_bindings.sh` instead.
 * Generator version: 1.0.0
 */

TS_HEADER

    node -e "
const fs = require('fs');
const registry = JSON.parse(fs.readFileSync('$REGISTRY_FILE', 'utf8'));

// Generate enums
const shared = registry.contracts.find(c => c.name === 'shared_types');
if (shared && shared.enums) {
    for (const en of shared.enums) {
        console.log('export enum ' + en.name + ' {');
        for (const v of en.values) {
            console.log('  ' + v.member + ' = \"' + v.value + '\",');
        }
        console.log('}');
        console.log('');
    }
}

console.log('// ==================== Contract Interfaces ====================');
console.log('');

// Generate interfaces
for (const contract of registry.contracts) {
    if (contract.name === 'shared_types') continue;
    if (!contract.interfaces) continue;

    for (const [name, iface] of Object.entries(contract.interfaces)) {
        const fields = iface.fields || iface.args || [];
        if (fields.length === 0) continue;

        const iName = name.charAt(0).toUpperCase() + name.slice(1).replace(/_([a-z])/g, (_, c) => c.toUpperCase());
        console.log('export interface ' + iName + ' {');
        for (const f of fields) {
            const tsType = mapTsType(f.type || 'string');
            const opt = f.optional ? '?' : '';
            console.log('  ' + f.name + opt + ': ' + tsType + ';');
        }
        console.log('}');
        console.log('');
    }
}

function mapTsType(t) {
    if (t === 'String' || t === 'string') return 'string';
    if (t === 'u32' || t === 'u64' || t === 'number') return 'number';
    if (t === 'bool') return 'boolean';
    if (t.endsWith('[]')) return mapTsType(t.slice(0, -2)) + '[]';
    if (t.startsWith('Option<')) return mapTsType(t.slice(7, -1)) + ' | null';
    if (t === 'void') return 'void';
    return t;
}
" >> "$TYPESCRIPT_OUTPUT"

    echo "  Written: $TYPESCRIPT_OUTPUT"
}

# ---------------------------------------------------------------------------
# Check mode
# ---------------------------------------------------------------------------
if $CHECK_MODE; then
    echo "Checking if bindings are up to date..."
    # Generate to temp and compare
    TMP_PY=$(mktemp)
    TMP_TS=$(mktemp)

    if $GENERATE_PYTHON; then
        generate_python > "$TMP_PY" 2>&1
        if ! diff -q "$PYTHON_OUTPUT" "$TMP_PY" > /dev/null 2>&1; then
            echo "FAIL: Python bindings are out of date. Run ./scripts/generate_bindings.sh"
            exit 1
        fi
        echo "  Python bindings: OK"
    fi

    if $GENERATE_TYPESCRIPT; then
        generate_typescript > "$TMP_TS" 2>&1
        if ! diff -q "$TYPESCRIPT_OUTPUT" "$TMP_TS" > /dev/null 2>&1; then
            echo "FAIL: TypeScript bindings are out of date. Run ./scripts/generate_bindings.sh"
            exit 1
        fi
        echo "  TypeScript bindings: OK"
    fi

    rm -f "$TMP_PY" "$TMP_TS"
    echo "All bindings are up to date."
    exit 0
fi

# Generate bindings
if $GENERATE_PYTHON; then
    generate_python
fi

if $GENERSCRIPT; then
    generate_typescript
fi

echo ""
echo "Bindings generation complete."
echo "Run 'git diff' to review changes."