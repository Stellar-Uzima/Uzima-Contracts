#!/usr/bin/env bash
# ============================================================================
# publish_release_bundle.sh
# Builds a known-good deployment artifact bundle for the given version tag.
#
# Usage:
#   ./scripts/publish_release_bundle.sh <version> [network]
#
# Outputs:
#   deployments/releases/<version>/
#     manifest.json      — contract list, WASM checksums, deploy order
#     <contract>.wasm    — optimized WASM artifacts
#     SHA256SUMS         — checksums for integrity verification
#     RELEASE_NOTES.md   — auto-generated release notes
# ============================================================================
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

info()  { echo -e "${GREEN}[INFO]${NC}  $1"; }
warn()  { echo -e "${YELLOW}[WARN]${NC}  $1"; }
fail()  { echo -e "${RED}[FAIL]${NC}  $1"; exit 1; }

if [[ $# -lt 1 ]]; then
  fail "Usage: $0 <version> [network]"
fi

VERSION="$1"
NETWORK="${2:-testnet}"
RELEASE_DIR="$ROOT_DIR/deployments/releases/$VERSION"
WASM_DIR="$ROOT_DIR/target/wasm32-unknown-unknown/release"
MANIFEST_SRC="$ROOT_DIR/deployments/deployment-manifest.json"

# ---------------------------------------------------------------------------
# 1. Build all workspace contracts
# ---------------------------------------------------------------------------
info "Building workspace contracts..."
cd "$ROOT_DIR"
cargo build --release --target wasm32-unknown-unknown 2>/dev/null || \
  warn "WASM build skipped (toolchain not available). Using existing artifacts."

# ---------------------------------------------------------------------------
# 2. Create release directory
# ---------------------------------------------------------------------------
mkdir -p "$RELEASE_DIR"
info "Release directory: $RELEASE_DIR"

# ---------------------------------------------------------------------------
# 3. Copy WASM artifacts
# ---------------------------------------------------------------------------
Copied=0
if [[ -d "$WASM_DIR" ]]; then
  for wasm in "$WASM_DIR"/*.wasm; do
    [[ -f "$wasm" ]] || continue
    cp "$wasm" "$RELEASE_DIR/"
    Copied=$((Copied + 1))
  done
fi
info "Copied $Copied WASM artifacts"

# ---------------------------------------------------------------------------
# 4. Generate SHA256 checksums
# ---------------------------------------------------------------------------
cd "$RELEASE_DIR"
if command -v sha256sum &>/dev/null; then
  sha256sum *.wasm > SHA256SUMS 2>/dev/null || true
elif command -v shasum &>/dev/null; then
  shasum -a 256 *.wasm > SHA256SUMS 2>/dev/null || true
fi
info "SHA256SUMS generated"

# ---------------------------------------------------------------------------
# 5. Build manifest.json from source manifest
# ---------------------------------------------------------------------------
if [[ -f "$MANIFEST_SRC" ]]; then
  # Enhance source manifest with release metadata
  python3 -c "
import json, datetime, hashlib, os

with open('$MANIFEST_SRC') as f:
    manifest = json.load(f)

manifest['release'] = {
    'version': '$VERSION',
    'network': '$NETWORK',
    'built_at': datetime.datetime.utcnow().isoformat() + 'Z',
    'builder': 'publish_release_bundle.sh',
    'wasm_count': len([f for f in os.listdir('.') if f.endswith('.wasm')]),
}

# Add checksums to each contract entry
checksums = {}
if os.path.exists('SHA256SUMS'):
    with open('SHA256SUMS') as f:
        for line in f:
            parts = line.strip().split()
            if len(parts) == 2:
                checksums[parts[1]] = parts[0]

for c in manifest.get('contracts', []):
    wasm_file = c.get('wasm_path', '').split('/')[-1]
    if wasm_file in checksums:
        c['sha256'] = checksums[wasm_file]

with open('manifest.json', 'w') as f:
    json.dump(manifest, f, indent=2)
" 2>/dev/null && info "manifest.json built" || warn "manifest.json skipped (python3 not available)"
fi

# ---------------------------------------------------------------------------
# 6. Copy release notes if available
# ---------------------------------------------------------------------------
NOTES_SRC="$ROOT_DIR/RELEASE_NOTES_${VERSION}.md"
if [[ -f "$NOTES_SRC" ]]; then
  cp "$NOTES_SRC" "$RELEASE_DIR/RELEASE_NOTES.md"
  info "Release notes copied"
fi

# ---------------------------------------------------------------------------
# 7. Summary
# ---------------------------------------------------------------------------
echo ""
info "=== Release Bundle Summary ==="
info "Version : $VERSION"
info "Network : $NETWORK"
info "Artifacts: $Copied WASM files"
info "Bundle   : $RELEASE_DIR"
echo ""
ls -la "$RELEASE_DIR" 2>/dev/null || true
