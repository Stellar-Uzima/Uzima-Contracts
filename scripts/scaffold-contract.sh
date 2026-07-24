#!/usr/bin/env bash
#
# scaffold-contract.sh — Create a new Soroban contract from the project template.
#
# Usage:
#   ./scripts/scaffold-contract.sh <contract_name>
#
# Example:
#   ./scripts/scaffold-contract.sh appointment_scheduling
#
# This will create:
#   contracts/appointment_scheduling/
#     src/lib.rs, errors.rs, events.rs, types.rs, test.rs
#     Cargo.toml
#
# After scaffolding:
#   1. Run `cargo test --package <contract_name>` to verify it compiles and tests pass.
#   2. Add the contract to the workspace Cargo.toml if it was excluded.
#   3. Run `node scripts/abi-compat.mjs` to update the interface registry baseline.
#

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
TEMPLATE_DIR="$PROJECT_ROOT/contracts/contract_template"

# ── Validate input ───────────────────────────────────────────────────────────

if [[ $# -ne 1 ]]; then
  echo "Usage: $0 <contract_name>"
  echo ""
  echo "Examples:"
  echo "  $0 appointment_scheduling"
  echo "  $0 lab_results"
  exit 1
fi

CONTRACT_NAME="$1"

# Validate name format (snake_case, lowercase, underscores only)
if [[ ! "$CONTRACT_NAME" =~ ^[a-z][a-z0-9_]*$ ]]; then
  echo "Error: Contract name must be snake_case (lowercase letters, digits, underscores)."
  echo "  Got: $CONTRACT_NAME"
  exit 1
fi

# Check template exists
if [[ ! -d "$TEMPLATE_DIR/src" ]]; then
  echo "Error: Template directory not found at $TEMPLATE_DIR"
  exit 1
fi

# Check target doesn't already exist
TARGET_DIR="$PROJECT_ROOT/contracts/$CONTRACT_NAME"
if [[ -d "$TARGET_DIR" ]]; then
  echo "Error: Directory already exists: $TARGET_DIR"
  exit 1
fi

# ── Derive names ─────────────────────────────────────────────────────────────

# PascalCase from snake_case: "appointment_scheduling" -> "AppointmentScheduling"
PASCAL_NAME=$(echo "$CONTRACT_NAME" | sed -r 's/(^|_)([a-z])/\U\2/g')

# Kebab-case for Cargo package name: "appointment_scheduling" -> "appointment-scheduling"
KEBAB_NAME=$(echo "$CONTRACT_NAME" | tr '_' '-')

echo "Scaffolding contract: $CONTRACT_NAME"
echo "  PascalCase: $PASCAL_NAME"
echo "  Kebab-case: $KEBAB_NAME"
echo ""

# ── Copy template ────────────────────────────────────────────────────────────

cp -r "$TEMPLATE_DIR" "$TARGET_DIR"

# ── Rename types in files ────────────────────────────────────────────────────

# Replace "contract-template" with the kebab name in Cargo.toml
sed -i "s/contract-template/$KEBAB_NAME/g" "$TARGET_DIR/Cargo.toml"

# Replace "ContractTemplate" with the Pascal name in Rust files
find "$TARGET_DIR/src" -name '*.rs' -exec sed -i "s/ContractTemplate/$PASCAL_NAME/g" {} +

# Replace the crate-level doc comment with the new contract name
sed -i "s/Contract Template/$PASCAL_NAME/g" "$TARGET_DIR/src/lib.rs"
sed -i "s/ContractTemplate/$PASCAL_NAME/g" "$TARGET_DIR/src/lib.rs"

# ── Create README ────────────────────────────────────────────────────────────

cat > "$TARGET_DIR/README.md" << EOF
# $PASCAL_NAME

Soroban smart contract for the Uzima healthcare platform.

## Overview

TODO: Describe what this contract does.

## Initialization

\`\`\`rust,ignore
client.initialize(&admin);
\`\`\`

## Functions

| Function | Description |
|---|---|
| \`initialize\` | Initialize the contract with an admin address |
| \`transfer_admin\` | Transfer admin rights to a new address |
| \`update_data\` | Update the contract's stored data |
| \`get_admin\` | Return the current admin address |
| \`get_data\` | Return the stored data |

## Testing

\`\`\`bash
cargo test --package $KEBAB_NAME
\`\`\`

## Files

- \`src/lib.rs\` - Main contract implementation
- \`src/errors.rs\` - Error definitions
- \`src/events.rs\` - Event emission helpers
- \`src/types.rs\` - Type definitions
- \`src/test.rs\` - Unit tests
EOF

# ── Summary ──────────────────────────────────────────────────────────────────

echo "Created contract at: $TARGET_DIR"
echo ""
echo "Next steps:"
echo "  1. Edit $TARGET_DIR/src/lib.rs to implement your contract logic."
echo "  2. Run: cargo test --package $KEBAB_NAME"
echo "  3. Run: cargo clippy --package $KEBAB_NAME -- -D warnings"
echo "  4. Run: node scripts/abi-compat.mjs  (to update interface baseline)"
echo "  5. Add entry to schemas/interface-registry/registry.json"
echo ""
