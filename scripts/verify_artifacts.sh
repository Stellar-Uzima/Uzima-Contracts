#!/bin/bash
# verify_artifacts.sh - Verify WASM artifact signatures and checksums
# Usage: ./scripts/verify_artifacts.sh [OPTIONS]
#
# Verifies that built WASM artifacts match their recorded checksums
# and have valid signatures from the release signing process.

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

ERRORS=0
WARNINGS=0

log_info() { echo -e "${BLUE}[INFO]${NC} $1"; }
log_success() { echo -e "${GREEN}[PASS]${NC} $1"; }
log_warning() { echo -e "${YELLOW}[WARN]${NC} $1"; WARNINGS=$((WARNINGS + 1)); }
log_error() { echo -e "${RED}[FAIL]${NC} $1"; ERRORS=$((ERRORS + 1)); }

VERSION=""
NETWORK="testnet"
VERBOSE=false

show_help() {
    cat <<EOF
Verify WASM artifact signatures and checksums.

Usage:
    $0 [OPTIONS]

Options:
    --version <version>  Version to verify (reads from deployments/signing/manifest.json)
    --network <network>  Network to verify against (default: testnet)
    --verbose            Show detailed verification output
    --help               Show this help message

Examples:
    $0
    $0 --version 1.0.0
    $0 --version 1.0.0 --network mainnet --verbose
EOF
}

parse_args() {
    while [[ $# -gt 0 ]]; do
        case $1 in
            --version) VERSION="$2"; shift 2 ;;
            --network) NETWORK="$2"; shift 2 ;;
            --verbose) VERBOSE=true; shift ;;
            --help|-h) show_help; exit 0 ;;
            *) log_error "Unknown option: $1"; exit 1 ;;
        esac
    done
}

verify_checksums() {
    log_info "Verifying WASM checksums..."

    if [[ ! -d "$SIGNING_DIR/artifacts" ]]; then
        log_error "Signing artifacts directory not found: $SIGNING_DIR/artifacts"
        return 1
    fi

    local verified=0
    local failed=0

    for artifact_dir in "$SIGNING_DIR/artifacts"/*/; do
        [[ -d "$artifact_dir" ]] || continue
        local artifact_name
        artifact_name=$(basename "$artifact_dir")
        local checksum_file="$artifact_dir/SHA256SUM"
        local wasm_file="$BUILD_DIR/${artifact_name}.wasm"

        if [[ ! -f "$checksum_file" ]]; then
            log_warning "No checksum found for $artifact_name"
            continue
        fi

        if [[ ! -f "$wasm_file" ]]; then
            log_error "WASM artifact not found: $wasm_file"
            failed=$((failed + 1))
            continue
        fi

        local expected_checksum
        expected_checksum=$(awk '{print $1}' "$checksum_file")
        local actual_checksum
        actual_checksum=$(sha256sum "$wasm_file" | awk '{print $1}')

        if [[ "$expected_checksum" == "$actual_checksum" ]]; then
            log_success "Checksum verified: $artifact_name"
            verified=$((verified + 1))
            if [[ "$VERBOSE" == "true" ]]; then
                echo "  Hash: $actual_checksum"
            fi
        else
            log_error "Checksum mismatch: $artifact_name"
            echo "  Expected: $expected_checksum"
            echo "  Actual:   $actual_checksum"
            failed=$((failed + 1))
        fi
    done

    echo ""
    log_info "Checksum results: $verified passed, $failed failed"
}

verify_signatures() {
    log_info "Verifying artifact signatures..."

    if [[ ! -d "$SIGNING_DIR/artifacts" ]]; then
        log_warning "No signing artifacts directory found"
        return 0
    fi

    local verified=0
    local failed=0

    for artifact_dir in "$SIGNING_DIR/artifacts"/*/; do
        [[ -d "$artifact_dir" ]] || continue
        local artifact_name
        artifact_name=$(basename "$artifact_dir")
        local signature_file="$artifact_dir/SHA256SUM.sig"
        local checksum_file="$artifact_dir/SHA256SUM"

        if [[ ! -f "$signature_file" ]]; then
            log_warning "No signature found for $artifact_name"
            continue
        fi

        if [[ ! -f "$checksum_file" ]]; then
            log_warning "No checksum file to verify signature against for $artifact_name"
            continue
        fi

        local key_file="$SIGNING_DIR/release-signing-key.pem"
        if [[ ! -f "$key_file" ]]; then
            log_warning "Signing key not found, skipping signature verification for $artifact_name"
            continue
        fi

        if command -v openssl &>/dev/null; then
            if openssl dgst -sha256 -verify <(openssl pkey -in "$key_file" -pubout 2>/dev/null) -signature "$signature_file" "$checksum_file" 2>/dev/null; then
                log_success "Signature verified: $artifact_name"
                verified=$((verified + 1))
            else
                log_error "Signature verification failed: $artifact_name"
                failed=$((failed + 1))
            fi
        fi
    done

    echo ""
    log_info "Signature results: $verified passed, $failed failed"
}

verify_manifest() {
    log_info "Verifying signing manifest..."

    local manifest_file="$SIGNING_DIR/manifest.json"
    if [[ ! -f "$manifest_file" ]]; then
        log_error "Signing manifest not found: $manifest_file"
        return 1
    fi

    if command -v python3 &>/dev/null; then
        if python3 -m json.tool "$manifest_file" >/dev/null 2>&1; then
            log_success "Manifest is valid JSON"
        else
            log_error "Manifest is not valid JSON"
        fi
    fi

    if [[ -n "$VERSION" ]]; then
        local manifest_version
        manifest_version=$(grep '"signing_version"' "$manifest_file" | awk -F'"' '{print $4}')
        if [[ "$manifest_version" == "$VERSION" ]]; then
            log_success "Manifest version matches: v$VERSION"
        else
            log_error "Manifest version mismatch: expected v$VERSION, got v$manifest_version"
        fi
    fi

    local manifest_network
    manifest_network=$(grep '"signing_network"' "$manifest_file" | awk -F'"' '{print $4}')
    if [[ "$manifest_network" == "$NETWORK" ]]; then
        log_success "Manifest network matches: $NETWORK"
    else
        log_error "Manifest network mismatch: expected $NETWORK, got $manifest_network"
    fi
}

verify_metadata_consistency() {
    log_info "Verifying metadata consistency..."

    if [[ ! -d "$SIGNING_DIR/artifacts" ]]; then
        return 0
    fi

    local issues=0
    for artifact_dir in "$SIGNING_DIR/artifacts"/*/; do
        [[ -d "$artifact_dir" ]] || continue
        local artifact_name
        artifact_name=$(basename "$artifact_dir")
        local metadata_file="$artifact_dir/metadata.json"

        if [[ ! -f "$metadata_file" ]]; then
            log_warning "No metadata for $artifact_name"
            issues=$((issues + 1))
            continue
        fi

        if [[ -n "$VERSION" ]]; then
            local meta_version
            meta_version=$(grep '"version"' "$metadata_file" | awk -F'"' '{print $4}')
            if [[ "$meta_version" != "$VERSION" ]]; then
                log_error "Version mismatch in $artifact_name metadata: expected v$VERSION, got v$meta_version"
                issues=$((issues + 1))
            fi
        fi

        if [[ -n "$NETWORK" ]]; then
            local meta_network
            meta_network=$(grep '"network"' "$metadata_file" | awk -F'"' '{print $4}')
            if [[ "$meta_network" != "$NETWORK" ]]; then
                log_error "Network mismatch in $artifact_name metadata: expected $NETWORK, got $meta_network"
                issues=$((issues + 1))
            fi
        fi
    done

    if [[ $issues -eq 0 ]]; then
        log_success "All metadata is consistent"
    fi
}

main() {
    parse_args "$@"

    echo "=========================================="
    echo "  Uzima Artifact Verification"
    [[ -n "$VERSION" ]] && echo "  Version: v$VERSION"
    echo "  Network: $NETWORK"
    echo "=========================================="
    echo ""

    verify_manifest
    echo ""
    verify_checksums
    echo ""
    verify_signatures
    echo ""
    verify_metadata_consistency

    echo ""
    echo "=========================================="
    if [[ $ERRORS -eq 0 ]]; then
        log_success "All verifications passed! ($WARNINGS warnings)"
    else
        log_error "$ERRORS verification(s) failed ($WARNINGS warnings)"
        exit 1
    fi
    echo "=========================================="
}

trap 'log_error "Script failed at line $LINENO"' ERR

main "$@"
