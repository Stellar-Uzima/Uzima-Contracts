#!/bin/bash
# sign_artifacts.sh - Sign WASM contract artifacts for release verification
# Usage: ./scripts/sign_artifacts.sh [OPTIONS]
#
# Signs individual WASM artifacts and creates per-artifact signatures
# alongside SHA256 checksums. Used by CI and release pipelines.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
SIGNING_DIR="$PROJECT_ROOT/deployments/signing"
BUILD_DIR="$PROJECT_ROOT/target/wasm32-unknown-unknown/release"

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

log_info() { echo -e "${BLUE}[INFO]${NC} $1"; }
log_success() { echo -e "${GREEN}[SUCCESS]${NC} $1"; }
log_warning() { echo -e "${YELLOW}[WARNING]${NC} $1"; }
log_error() { echo -e "${RED}[ERROR]${NC} $1"; }

# Defaults
SIGNING_KEY=""
VERSION=""
NETWORK="testnet"
DRY_RUN=false
FORCE=false

show_help() {
    cat <<EOF
Sign WASM contract artifacts for release verification.

Usage:
    $0 [OPTIONS]

Options:
    --key <path>         Path to signing key (GPG or ed25519 PEM). If omitted, generates ephemeral key.
    --version <version>  Release version tag (e.g., 1.0.0). Required.
    --network <network>  Target network (default: testnet)
    --dry-run            Show what would be signed without writing
    --force              Overwrite existing signatures
    --help               Show this help message

Environment Variables:
    SIGNING_KEY          Alternative to --key flag
    RELEASE_VERSION      Alternative to --version flag

Examples:
    $0 --version 1.0.0
    $0 --version 1.0.0 --key /path/to/release-key.pem --network mainnet
    $0 --version 1.0.0 --dry-run
EOF
}

parse_args() {
    while [[ $# -gt 0 ]]; do
        case $1 in
            --key) SIGNING_KEY="$2"; shift 2 ;;
            --version) VERSION="$2"; shift 2 ;;
            --network) NETWORK="$2"; shift 2 ;;
            --dry-run) DRY_RUN=true; shift ;;
            --force) FORCE=true; shift ;;
            --help|-h) show_help; exit 0 ;;
            *) log_error "Unknown option: $1"; show_help; exit 1 ;;
        esac
    done

    VERSION="${VERSION:-${RELEASE_VERSION:-}}"
    SIGNING_KEY="${SIGNING_KEY:-${SIGNING_KEY_ENV:-}}"

    if [[ -z "$VERSION" ]]; then
        log_error "Version is required. Use --version or set RELEASE_VERSION."
        exit 1
    fi

    if [[ ! "$VERSION" =~ ^[0-9]+\.[0-9]+\.[0-9]+(-[a-zA-Z0-9.]+)?$ ]]; then
        log_error "Invalid version format: $VERSION (expected X.Y.Z)"
        exit 1
    fi
}

ensure_signing_dir() {
    mkdir -p "$SIGNING_DIR"
}

generate_ephemeral_key() {
    local key_file="$SIGNING_DIR/release-signing-key.pem"
    if [[ -f "$key_file" ]]; then
        log_info "Using existing signing key: $key_file"
        return
    fi

    log_info "Generating ephemeral ed25519 signing key..."
    if command -v openssl &>/dev/null; then
        openssl genpkey -algorithm Ed25519 -out "$key_file" 2>/dev/null
        log_success "Signing key generated: $key_file"
        log_warning "Store this key securely if you need to verify signatures later."
    else
        log_error "openssl not found. Install openssl or provide a signing key with --key."
        exit 1
    fi
}

get_signing_key() {
    if [[ -n "$SIGNING_KEY" ]]; then
        if [[ ! -f "$SIGNING_KEY" ]]; then
            log_error "Signing key not found: $SIGNING_KEY"
            exit 1
        fi
        echo "$SIGNING_KEY"
    else
        generate_ephemeral_key
        echo "$SIGNING_DIR/release-signing-key.pem"
    fi
}

find_wasm_artifacts() {
    local artifacts=()
    if [[ -d "$BUILD_DIR" ]]; then
        while IFS= read -r -d '' wasm_file; do
            artifacts+=("$wasm_file")
        done < <(find "$BUILD_DIR" -name "*.wasm" -type f -print0 | sort -z)
    fi

    if [[ -d "$PROJECT_ROOT/dist" ]]; then
        while IFS= read -r -d '' wasm_file; do
            artifacts+=("$wasm_file")
        done < <(find "$PROJECT_ROOT/dist" -name "*.wasm" -type f -print0 | sort -z)
    fi

    echo "${artifacts[@]}"
}

sign_artifact() {
    local wasm_file="$1"
    local key_file="$2"
    local artifact_name
    artifact_name=$(basename "$wasm_file" .wasm)
    local output_dir="$SIGNING_DIR/artifacts/$artifact_name"
    local checksum_file="$output_dir/SHA256SUM"
    local signature_file="$output_dir/SHA256SUM.sig"
    local metadata_file="$output_dir/metadata.json"

    mkdir -p "$output_dir"

    if [[ -f "$signature_file" && "$FORCE" != "true" ]]; then
        log_warning "Signature already exists for $artifact_name (use --force to overwrite)"
        return 0
    fi

    local checksum
    checksum=$(sha256sum "$wasm_file" | awk '{print $1}')
    local file_size
    file_size=$(stat -f%z "$wasm_file" 2>/dev/null || stat --printf="%s" "$wasm_file" 2>/dev/null || wc -c < "$wasm_file" | tr -d ' ')

    if [[ "$DRY_RUN" == "true" ]]; then
        log_info "[DRY RUN] Would sign $artifact_name ($checksum)"
        return 0
    fi

    echo "$checksum  $(basename "$wasm_file")" > "$checksum_file"

    if command -v openssl &>/dev/null; then
        openssl dgst -sha256 -sign "$key_file" -out "$signature_file" "$wasm_file" 2>/dev/null
    elif command -v gpg &>/dev/null; then
        gpg --batch --yes --armor --output "$signature_file" --sign "$checksum_file" 2>/dev/null
    else
        log_error "No signing tool available (openssl or gpg)"
        exit 1
    fi

    cat > "$metadata_file" <<METAEOF
{
    "artifact": "$(basename "$wasm_file")",
    "checksum_sha256": "$checksum",
    "file_size_bytes": $file_size,
    "signed_at": "$(date -u +%Y-%m-%dT%H:%M:%SZ)",
    "version": "$VERSION",
    "network": "$NETWORK",
    "toolchain": "$(rustc --version 2>/dev/null || echo 'unknown')",
    "git_commit": "$(git -C "$PROJECT_ROOT" rev-parse --short HEAD 2>/dev/null || echo 'unknown')",
    "signer_key": "$(basename "$key_file")"
}
METAEOF

    log_success "Signed: $artifact_name ($checksum)"
}

create_signing_manifest() {
    local manifest_file="$SIGNING_DIR/manifest.json"
    local key_file="$1"

    if [[ "$DRY_RUN" == "true" ]]; then
        log_info "[DRY RUN] Would create signing manifest"
        return 0
    fi

    cat > "$manifest_file" <<MFEOF
{
    "schema_version": "1.0.0",
    "signing_version": "$VERSION",
    "signing_network": "$NETWORK",
    "signed_at": "$(date -u +%Y-%m-%dT%H:%M:%SZ)",
    "signing_key": "$(basename "$key_file")",
    "repository": "https://github.com/Stellar-Uzima/Uzima-Contracts",
    "artifacts_dir": "deployments/signing/artifacts/",
    "verification_script": "./scripts/verify_artifacts.sh",
    "git_commit": "$(git -C "$PROJECT_ROOT" rev-parse HEAD 2>/dev/null || echo 'unknown')",
    "toolchain": "$(rustc --version 2>/dev/null || echo 'unknown')"
}
MFEOF

    log_success "Signing manifest created: $manifest_file"
}

main() {
    parse_args "$@"

    echo "=========================================="
    echo "  Uzima WASM Artifact Signing"
    echo "  Version: v$VERSION"
    echo "  Network: $NETWORK"
    echo "=========================================="
    echo ""

    ensure_signing_dir
    local key_file
    key_file=$(get_signing_key)

    local artifacts
    artifacts=$(find_wasm_artifacts)

    if [[ -z "$artifacts" ]]; then
        log_warning "No WASM artifacts found in $BUILD_DIR or $PROJECT_ROOT/dist"
        log_info "Run 'cargo build --target wasm32-unknown-unknown --release' first"
        exit 0
    fi

    local count=0
    for artifact in $artifacts; do
        if [[ -n "$artifact" ]]; then
            sign_artifact "$artifact" "$key_file"
            count=$((count + 1))
        fi
    done

    create_signing_manifest "$key_file"

    echo ""
    log_success "Signed $count artifact(s) for v$VERSION"
    echo "=========================================="
}

trap 'log_error "Script failed at line $LINENO"' ERR

main "$@"
