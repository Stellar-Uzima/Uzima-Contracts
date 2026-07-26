#!/bin/bash
set -e

echo "Generating standard Rust documentation..."
cargo doc --workspace --no-deps

echo "Tracking API Surface..."
mkdir -p docs/api

CHANGES_DETECTED=0
BREAKING_CHANGES=0

# Ensure we have compiled wasms
if ! ls target/wasm32-unknown-unknown/release/*.wasm 1> /dev/null 2>&1; then
    echo "No compiled WebAssembly contracts found. Ensure 'cargo build --target wasm32-unknown-unknown --release' was run."
    exit 1
fi

for wasm in target/wasm32-unknown-unknown/release/*.wasm; do
  CONTRACT_NAME=$(basename "$wasm" .wasm)
  SPEC_FILE="docs/api/${CONTRACT_NAME}_spec.txt"
  TEMP_SPEC="${SPEC_FILE}.tmp"
  
  echo "Inspecting API surface for $CONTRACT_NAME..."
  soroban contract inspect --wasm "$wasm" > "$TEMP_SPEC"
  
  if [ -f "$SPEC_FILE" ]; then
    if ! cmp -s "$SPEC_FILE" "$TEMP_SPEC"; then
      echo "Changes detected in $CONTRACT_NAME API surface."
      CHANGES_DETECTED=1
      
      # Detect Breaking Changes: Look for removed lines (e.g., removed functions or changed signatures)
      REMOVED_LINES=$(diff -U0 "$SPEC_FILE" "$TEMP_SPEC" | grep '^-' | grep -v '^---' || true)
      if [ -n "$REMOVED_LINES" ]; then
        echo "Potential BREAKING CHANGE detected in $CONTRACT_NAME!"
        BREAKING_CHANGES=1
      fi
    fi
  else
    echo "New contract detected: $CONTRACT_NAME"
    CHANGES_DETECTED=1
  fi
  
  mv "$TEMP_SPEC" "$SPEC_FILE"
done

if [ $CHANGES_DETECTED -eq 1 ]; then
  echo "Updating Version History..."
  DATE=$(date +'%Y-%m-%d %H:%M:%S')
  TEMP_HISTORY=$(mktemp)
  
  echo -e "## [Automated API Update] - $DATE\n" > "$TEMP_HISTORY"
  if [ $BREAKING_CHANGES -eq 1 ]; then
    echo -e "### ⚠️ Breaking Changes Detected\nAPI signatures were modified or removed. Please review the updated specs.\n" >> "$TEMP_HISTORY"
  else
    echo -e "### Minor API Updates\nContract API surfaces were added or extended.\n" >> "$TEMP_HISTORY"
  fi
  
  if [ -f docs/VERSION_HISTORY.md ]; then
    cat docs/VERSION_HISTORY.md >> "$TEMP_HISTORY"
  fi
  mv "$TEMP_HISTORY" docs/VERSION_HISTORY.md
  
  echo "Documentation automation complete."
else
  echo "No API surface changes detected. Docs are up to date."
fi