#!/bin/bash
# Verify release artifacts for Uzima-Contracts
# Usage: ./scripts/verify_release_artifacts.sh VERSION

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
VERSION=${1:-}
NETWORK=${2:-testnet}

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

ERRORS=0

log_info() { echo -e "${BLUE}[INFO]${NC} $1"; }
log_success() { echo -e "${GREEN}[PASS]${NC} $1"; }
log_warning() { echo -e "${YELLOW}[WARN]${NC} $1"; }
log_error() { echo -e "${RED}[FAIL]${NC} $1"; ERRORS=$((ERRORS + 1)); }

validate_inputs() {
    if [[ -z "$VERSION" ]]; then
        log_error "Version is required"
        echo "Usage: $0 VERSION [NETWORK]"
        exit 1
    fi
}

verify_checksums() {
    local version="$1"
    local release_dir="$PROJECT_ROOT/artifacts/release-v$version"
    local checksums_file="$release_dir/SHA256SUMS.txt"

    log_info "Verifying checksums for v$version..."

    if [[ ! -f "$checksums_file" ]]; then
        log_error "Checksums file not found: $checksums_file"
        return
    fi

    local wasm_count=0
    local valid_count=0

    while IFS= read -r line; do
        if [[ "$line" =~ ^# ]] || [[ -z "$line" ]]; then
            continue
        fi

        local expected_hash
        expected_hash=$(echo "$line" | awk '{print $1}')
        local file_path
        file_path=$(echo "$line" | awk '{print $2}')

        local full_path="$PROJECT_ROOT/dist/$file_path"
        if [[ ! -f "$full_path" ]]; then
            log_warning "Artifact not found: $file_path"
            continue
        fi

        local actual_hash
        actual_hash=$(sha256sum "$full_path" | awk '{print $1}')
        wasm_count=$((wasm_count + 1))

        if [[ "$expected_hash" == "$actual_hash" ]]; then
            valid_count=$((valid_count + 1))
        else
            log_error "Checksum mismatch for $file_path"
            log_error "  Expected: $expected_hash"
            log_error "  Actual:   $actual_hash"
        fi
    done < "$checksums_file"

    if [[ $wasm_count -eq 0 ]]; then
        log_warning "No WASM artifacts found to verify"
    elif [[ $valid_count -eq $wasm_count ]]; then
        log_success "All $wasm_count WASM checksums verified"
    fi
}

verify_signature() {
    local version="$1"
    local release_dir="$PROJECT_ROOT/artifacts/release-v$version"
    local checksums_file="$release_dir/SHA256SUMS.txt"
    local signature_file="$release_dir/SHA256SUMS.txt.sig"

    log_info "Verifying signature..."

    if [[ ! -f "$signature_file" ]]; then
        log_warning "Signature file not found: $signature_file"
        return
    fi

    if command -v gpg &>/dev/null; then
        if gpg --verify "$signature_file" "$checksums_file" 2>/dev/null; then
            log_success "GPG signature verified"
        else
            log_warning "GPG signature verification failed (key may not be available locally)"
        fi
    elif command -v openssl &>/dev/null; then
        log_info "OpenSSL signature present (verification requires public key)"
    fi
}

verify_deployment_hash() {
    local version="$1"
    local network="$2"
    local deploy_dir="$PROJECT_ROOT/deployments/$network/v$version"
    local hashes_file="$deploy_dir/hashes.txt"

    log_info "Verifying deployment hash for $network/v$version..."

    if [[ ! -f "$hashes_file" ]]; then
        log_warning "Deployment hash not found: $hashes_file"
        return
    fi

    local mismatches=0

    while IFS= read -r line; do
        if [[ "$line" =~ ^# ]] || [[ -z "$line" ]]; then
            continue
        fi

        local expected_hash
        expected_hash=$(echo "$line" | awk '{print $1}')
        local file_path
        file_path=$(echo "$line" | awk '{print $2}')

        local full_path="$PROJECT_ROOT/dist/$file_path"
        if [[ ! -f "$full_path" ]]; then
            log_warning "Artifact not found for comparison: $file_path"
            continue
        fi

        local actual_hash
        actual_hash=$(sha256sum "$full_path" | awk '{print $1}')

        if [[ "$expected_hash" == "$actual_hash" ]]; then
            log_success "Deployment hash match: $file_path"
        else
            log_error "Deployment hash mismatch: $file_path"
            mismatches=$((mismatches + 1))
        fi
    done < "$hashes_file"

    if [[ $mismatches -eq 0 ]] && [[ -f "$hashes_file" ]]; then
        log_success "Deployment hash verification passed"
    fi
}

verify_manifest() {
    local version="$1"
    local release_dir="$PROJECT_ROOT/artifacts/release-v$version"
    local manifest_file="$release_dir/manifest.json"

    log_info "Verifying release manifest..."

    if [[ ! -f "$manifest_file" ]]; then
        log_error "Manifest not found: $manifest_file"
        return
    fi

    if command -v python3 &>/dev/null; then
        if python3 -m json.tool "$manifest_file" >/dev/null 2>&1; then
            log_success "Manifest is valid JSON"
        else
            log_error "Manifest is not valid JSON"
        fi
    fi

    if [[ -f "$release_dir/SHA256SUMS.txt" ]]; then
        local expected_sha
        expected_sha=$(grep '"checksums_sha256"' "$manifest_file" | awk -F'"' '{print $4}')
        local actual_sha
        actual_sha=$(sha256sum "$release_dir/SHA256SUMS.txt" | awk '{print $1}')

        if [[ "$expected_sha" == "$actual_sha" ]]; then
            log_success "Manifest checksum matches SHA256SUMS.txt"
        else
            log_error "Manifest checksum mismatch"
            log_error "  Expected: $expected_sha"
            log_error "  Actual:   $actual_sha"
        fi
    fi
}

verify_build_provenance() {
    local version="$1"
    local release_dir="$PROJECT_ROOT/artifacts/release-v$version"
    local manifest_file="$release_dir/manifest.json"

    log_info "Checking build provenance..."

    if [[ ! -f "$manifest_file" ]]; then
        return
    fi

    local recorded_commit
    recorded_commit=$(grep '"commit"' "$manifest_file" | awk -F'"' '{print $4}')
    local current_commit
    current_commit=$(git -C "$PROJECT_ROOT" rev-parse HEAD 2>/dev/null || echo "unknown")

    if [[ "$recorded_commit" == "$current_commit" ]]; then
        log_success "Build provenance verified: commit matches"
    else
        log_warning "Build provenance: commit mismatch"
        log_warning "  Recorded: $recorded_commit"
        log_warning "  Current:  $current_commit"
    fi
}

perform_verification() {
    local version="$1"
    local network="$2"

    echo "=========================================="
    echo "  Uzima Release Artifact Verification"
    echo "  Version: v$version"
    echo "  Network: $network"
    echo "=========================================="
    echo ""

    validate_inputs
    verify_checksums "$version"
    verify_signature "$version"
    verify_deployment_hash "$version" "$network"
    verify_manifest "$version"
    verify_build_provenance "$version"

    echo ""
    echo "=========================================="
    if [[ $ERRORS -eq 0 ]]; then
        log_success "All verifications passed!"
    else
        log_error "$ERRORS verification(s) failed"
        exit 1
    fi
    echo "=========================================="
}

show_help() {
    cat <<EOF
Verify release artifacts for Uzima-Contracts

Usage:
    $0 VERSION [NETWORK] [OPTIONS]

Arguments:
    VERSION          Version to verify (e.g., 1.2.0)
    NETWORK          Network to verify against (default: testnet)

Options:
    --help           Show this help message

Examples:
    $0 1.2.0
    $0 1.2.0 mainnet
    $0 1.2.0 testnet

The script verifies:
1. WASM artifact checksums match SHA256SUMS.txt
2. Release signature integrity
3. Deployment hash records match current build
4. Release manifest validity
5. Build provenance (git commit alignment)
EOF
}

main() {
    if [[ "${1:-}" == "--help" ]] || [[ "${1:-}" == "-h" ]]; then
        show_help
        exit 0
    fi

    perform_verification "$VERSION" "$NETWORK"
}

trap 'log_error "Script failed at line $LINENO"' ERR

main "$@"
