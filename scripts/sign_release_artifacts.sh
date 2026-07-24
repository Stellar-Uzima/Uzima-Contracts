#!/bin/bash
# Sign release artifacts for Uzima-Contracts
# Usage: ./scripts/sign_release_artifacts.sh VERSION SIGNING_KEY_PATH

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
VERSION=${1:-}
SIGNING_KEY=${2:-}
DRY_RUN=${DRY_RUN:-false}

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

log_info() { echo -e "${BLUE}[INFO]${NC} $1"; }
log_success() { echo -e "${GREEN}[SUCCESS]${NC} $1"; }
log_warning() { echo -e "${YELLOW}[WARNING]${NC} $1"; }
log_error() { echo -e "${RED}[ERROR]${NC} $1"; }

validate_inputs() {
    if [[ -z "$VERSION" ]]; then
        log_error "Version is required"
        echo "Usage: $0 VERSION [SIGNING_KEY_PATH]"
        exit 1
    fi

    if [[ ! "$VERSION" =~ ^[0-9]+\.[0-9]+\.[0-9]+(-[a-zA-Z0-9.]+)?$ ]]; then
        log_error "Invalid version format: $VERSION"
        exit 1
    fi

    if [[ -n "$SIGNING_KEY" && ! -f "$SIGNING_KEY" ]]; then
        log_error "Signing key not found: $SIGNING_KEY"
        exit 1
    fi
}

generate_checksums() {
    local version="$1"
    local release_dir="$PROJECT_ROOT/artifacts/release-v$version"

    log_info "Generating checksums for release v$version..."

    if [[ ! -d "$release_dir" ]]; then
        mkdir -p "$release_dir"
    fi

    local checksums_file="$release_dir/SHA256SUMS.txt"

    {
        echo "# Uzima-Contracts Release v$version Checksums"
        echo "# Generated: $(date -u +%Y-%m-%dT%H:%M:%SZ)"
        echo "#"
        find "$PROJECT_ROOT/dist" -name "*.wasm" -type f 2>/dev/null | sort | while read -r wasm_file; do
            sha256sum "$wasm_file" | sed "s|$PROJECT_ROOT/dist/||"
        done
        find "$PROJECT_ROOT/artifacts" -name "*.tar.gz" -type f 2>/dev/null | sort | while read -r archive; do
            sha256sum "$archive" | sed "s|$PROJECT_ROOT/artifacts/||"
        done
    } > "$checksums_file"

    log_success "Checksums generated: $checksums_file"
    cat "$checksums_file"
}

sign_checksums() {
    local version="$1"
    local key_path="$2"
    local release_dir="$PROJECT_ROOT/artifacts/release-v$version"
    local checksums_file="$release_dir/SHA256SUMS.txt"
    local signature_file="$release_dir/SHA256SUMS.txt.sig"

    if [[ -z "$key_path" ]]; then
        log_warning "No signing key provided, generating self-signed certificate"
        generate_self_signed_key
        key_path="$PROJECT_ROOT/.release-signing-key"
    fi

    log_info "Signing release artifacts..."

    if [[ "$DRY_RUN" == "true" ]]; then
        log_info "DRY RUN: Would sign checksums with key $key_path"
        return
    fi

    if command -v gpg &>/dev/null; then
        gpg --batch --yes --armor --local-user "$key_path" \
            --output "$signature_file" --sign "$checksums_file"
    elif command -v openssl &>/dev/null; then
        openssl dgst -sha256 -sign "$key_path" -out "$signature_file" "$checksums_file"
    else
        log_error "Neither gpg nor openssl found for signing"
        exit 1
    fi

    log_success "Signature created: $signature_file"
}

generate_self_signed_key() {
    local key_dir="$PROJECT_ROOT/.release-signing"
    mkdir -p "$key_dir"

    if [[ ! -f "$key_dir/release-key.asc" ]]; then
        log_info "Generating GPG signing key..."
        gpg --batch --gen-key <<EOF
%no-protection
Key-Type: RSA
Key-Length: 4096
Subkey-Type: RSA
Subkey-Length: 4096
Name-Real: Uzima Release Signing
Name-Email: releases@stellar-uzima.org
Expire-Date: 0
%commit
EOF
        gpg --export --armor "releases@stellar-uzima.org" > "$key_dir/release-key.asc"
        log_success "Signing key generated"
    fi
}

create_release_manifest() {
    local version="$1"
    local release_dir="$PROJECT_ROOT/artifacts/release-v$version"
    local manifest_file="$release_dir/manifest.json"

    log_info "Creating release manifest..."

    local git_commit
    git_commit=$(git -C "$PROJECT_ROOT" rev-parse HEAD 2>/dev/null || echo "unknown")

    local git_tag
    git_tag=$(git -C "$PROJECT_ROOT" describe --tags --exact-match 2>/dev/null || echo "v$version")

    local checksums_sha
    checksums_sha=$(sha256sum "$release_dir/SHA256SUMS.txt" 2>/dev/null | cut -d' ' -f1 || echo "pending")

    cat > "$manifest_file" <<EOF
{
    "schema_version": "1.0.0",
    "version": "$version",
    "tag": "$git_tag",
    "release_date": "$(date -u +%Y-%m-%dT%H:%M:%SZ)",
    "repository": "https://github.com/Stellar-Uzima/Uzima-Contracts",
    "artifacts": {
        "checksums": "SHA256SUMS.txt",
        "checksums_signature": "SHA256SUMS.txt.sig",
        "wasm_directory": "../dist/",
        "source_archive": "uzima-contracts-v$version.tar.gz"
    },
    "integrity": {
        "checksums_sha256": "$checksums_sha"
    },
    "build_environment": {
        "toolchain": "rustc $(rustc --version 2>/dev/null | awk '{print $2}' || echo 'unknown')",
        "target": "wasm32-unknown-unknown",
        "commit": "$git_commit"
    },
    "signing": {
        "algorithm": "ed25519",
        "key_id": "release-signing-key",
        "signed_at": "$(date -u +%Y-%m-%dT%H:%M:%SZ)"
    },
    "verification": {
        "instructions": "Run ./scripts/verify_release_artifacts.sh v$version to verify integrity"
    }
}
EOF

    log_success "Release manifest created: $manifest_file"
}

record_deployment_hash() {
    local version="$1"
    local network="${2:-testnet}"
    local deploy_dir="$PROJECT_ROOT/deployments/$network/v$version"

    mkdir -p "$deploy_dir"

    {
        echo "# Uzima-Contracts deployment WASM hashes"
        echo "# network:   $network"
        echo "# release:   v$version"
        echo "# generated: $(date -u +%Y-%m-%dT%H:%M:%SZ)"
        echo "# toolchain: $(rustc --version 2>/dev/null || echo 'unknown')"
        echo "#"
        find "$PROJECT_ROOT/dist" -name "*.wasm" -type f 2>/dev/null | sort | while read -r wasm_file; do
            sha256sum "$wasm_file" | sed "s|$PROJECT_ROOT/dist/||"
        done
    } > "$deploy_dir/hashes.txt"

    log_success "Deployment hash recorded: $deploy_dir/hashes.txt"
}

perform_signing() {
    local version="$1"

    log_info "Starting release artifact signing for v$version..."

    validate_inputs
    generate_checksums "$version"
    sign_checksums "$version" "$SIGNING_KEY"
    create_release_manifest "$version"
    record_deployment_hash "$version"

    log_success "Release artifact signing completed for v$version"
}

show_help() {
    cat <<EOF
Sign release artifacts for Uzima-Contracts

Usage:
    $0 VERSION [SIGNING_KEY_PATH] [OPTIONS]

Arguments:
    VERSION              Version to sign (e.g., 1.2.0)
    SIGNING_KEY_PATH     Path to GPG private key (optional, generates self-signed if omitted)

Options:
    --dry-run            Perform a dry run without making changes
    --help               Show this help message

Environment Variables:
    DRY_RUN              Set to 'true' for dry run mode

Examples:
    $0 1.2.0
    $0 1.2.0 /path/to/signing-key.asc
    $0 1.2.0 --dry-run

The script will:
1. Generate SHA256 checksums for all WASM artifacts
2. Sign the checksums file with the provided or generated key
3. Create a release manifest with metadata
4. Record deployment hashes for the target network
EOF
}

main() {
    if [[ "${1:-}" == "--help" ]] || [[ "${1:-}" == "-h" ]]; then
        show_help
        exit 0
    fi

    if [[ "${2:-}" == "--dry-run" ]] || [[ "${3:-}" == "--dry-run" ]]; then
        export DRY_RUN="true"
    fi

    perform_signing "$VERSION"
}

trap 'log_error "Script failed at line $LINENO"' ERR

main "$@"
