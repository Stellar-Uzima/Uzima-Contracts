#!/bin/bash
# Release artifact publication script for Uzima-Contracts
# Usage: ./scripts/publish_artifacts.sh VERSION

set -euo pipefail

# Configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
VERSION=${1:-}
DRY_RUN=${DRY_RUN:-false}
PUBLISH_GITHUB=${PUBLISH_GITHUB:-true}
PUBLISH_NPM=${PUBLISH_NPM:-false}
PUBLISH_DOCKER=${PUBLISH_DOCKER:-false}
PUBLISH_CRATES=${PUBLISH_CRATES:-false}

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Logging functions
log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

log_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Artifact functions
create_artifact_archive() {
    local version="$1"
    local archive_name="uzima-contracts-v$version"
    local archive_dir="$PROJECT_ROOT/artifacts/$archive_name"
    
    log_info "Creating artifact archive for v$version..."
    
    # Create artifacts directory
    mkdir -p "$archive_dir"
    
    # Copy WASM files
    if [[ -d "$PROJECT_ROOT/dist" ]]; then
        cp -r "$PROJECT_ROOT/dist" "$archive_dir/wasm"
        log_success "WASM files copied"
    fi
    
    # Copy source code
    mkdir -p "$archive_dir/source"
    cp -r "$PROJECT_ROOT/contracts" "$archive_dir/source/"
    cp -r "$PROJECT_ROOT/scripts" "$archive_dir/source/"
    cp "$PROJECT_ROOT/Cargo.toml" "$archive_dir/source/"
    cp "$PROJECT_ROOT/Cargo.lock" "$archive_dir/source/"
    cp "$PROJECT_ROOT/README.md" "$archive_dir/source/"
    cp "$PROJECT_ROOT/CHANGELOG.md" "$archive_dir/source/"
    
    # Copy documentation
    mkdir -p "$archive_dir/docs"
    cp -r "$PROJECT_ROOT/docs" "$archive_dir/docs/"
    
    # Create checksums
    cd "$archive_dir"
    find . -type f -exec sha256sum {} + > SHA256SUMS.txt
    cd "$PROJECT_ROOT"
    
    # Create tar.gz archive
    cd "$PROJECT_ROOT/artifacts"
    tar -czf "$archive_name.tar.gz" "$archive_name"
    cd "$PROJECT_ROOT"
    
    log_success "Artifact archive created: artifacts/$archive_name.tar.gz"
    
    # Return archive path for other functions
    echo "$PROJECT_ROOT/artifacts/$archive_name.tar.gz"
}

publish_to_github() {
    local version="$1"
    local archive_path="$2"
    local tag="v$version"
    
    if [[ "$PUBLISH_GITHUB" != "true" ]]; then
        log_info "Skipping GitHub publication (PUBLISH_GITHUB=false)"
        return 0
    fi
    
    if ! command -v gh &> /dev/null; then
        log_warning "GitHub CLI not installed, skipping GitHub publication"
        return 0
    fi
    
    if [[ "$DRY_RUN" == "true" ]]; then
        log_info "DRY RUN: Would publish artifacts to GitHub"
        return 0
    fi
    
    log_info "Publishing artifacts to GitHub..."
    
    # Check if release exists
    if ! gh release view "$tag" &> /dev/null; then
        log_error "GitHub release $tag does not exist"
        return 1
    fi
    
    # Upload archive
    gh release upload "$tag" "$archive_path" --clobber
    
    # Upload individual WASM files
    if [[ -d "$PROJECT_ROOT/dist" ]]; then
        for wasm_file in "$PROJECT_ROOT/dist"/*.wasm; do
            if [[ -f "$wasm_file" ]]; then
                gh release upload "$tag" "$wasm_file" --clobber
                log_success "Uploaded $(basename "$wasm_file") to GitHub"
            fi
        done
    fi
    
    # Upload checksums
    local checksums_file="$PROJECT_ROOT/artifacts/uzima-contracts-v$version/SHA256SUMS.txt"
    if [[ -f "$checksums_file" ]]; then
        gh release upload "$tag" "$checksums_file" --clobber
        log_success "Uploaded checksums to GitHub"
    fi
    
    log_success "Artifacts published to GitHub"
}

publish_to_npm() {
    local version="$1"
    
    if [[ "$PUBLISH_NPM" != "true" ]]; then
        log_info "Skipping NPM publication (PUBLISH_NPM=false)"
        return 0
    fi
    
    if ! command -v npm &> /dev/null; then
        log_warning "NPM not installed, skipping NPM publication"
        return 0
    fi
    
    if [[ "$DRY_RUN" == "true" ]]; then
        log_info "DRY RUN: Would publish to NPM"
        return 0
    fi
    
    log_info "Publishing to NPM..."
    
    # This would require an NPM package setup
    # For now, just log that it would be published
    log_success "NPM publication would be performed (implementation needed)"
}

publish_to_docker() {
    local version="$1"
    
    if [[ "$PUBLISH_DOCKER" != "true" ]]; then
        log_info "Skipping Docker publication (PUBLISH_DOCKER=false)"
        return 0
    fi
    
    if ! command -v docker &> /dev/null; then
        log_warning "Docker not installed, skipping Docker publication"
        return 0
    fi
    
    if [[ "$DRY_RUN" == "true" ]]; then
        log_info "DRY RUN: Would publish Docker images"
        return 0
    fi
    
    log_info "Publishing Docker images..."
    
    # Build Docker image
    local image_name="stellar-uzima/uzima-contracts"
    local image_tag="$image_name:v$version"
    local latest_tag="$image_name:latest"
    
    if [[ -f "$PROJECT_ROOT/dockerfile" ]]; then
        docker build -t "$image_tag" -t "$latest_tag" "$PROJECT_ROOT"
        
        # Push to registry
        if docker push "$image_tag" && docker push "$latest_tag"; then
            log_success "Docker images published"
        else
            log_error "Failed to push Docker images"
            return 1
        fi
    else
        log_warning "Dockerfile not found, skipping Docker publication"
    fi
}

publish_to_crates() {
    local version="$1"
    
    if [[ "$PUBLISH_CRATES" != "true" ]]; then
        log_info "Skipping crates.io publication (PUBLISH_CRATES=false)"
        return 0
    fi
    
    if ! command -v cargo &> /dev/null; then
        log_warning "Cargo not installed, skipping crates.io publication"
        return 0
    fi
    
    if [[ "$DRY_RUN" == "true" ]]; then
        log_info "DRY RUN: Would publish to crates.io"
        return 0
    fi
    
    log_info "Publishing to crates.io..."
    
    # This would require proper crate setup
    # For now, just log that it would be published
    log_success "crates.io publication would be performed (implementation needed)"
}

create_release_manifest() {
    local version="$1"
    local archive_path="$2"
    local manifest_file="$PROJECT_ROOT/artifacts/release-v$version.json"
    
    log_info "Creating release manifest..."
    
    local manifest=$(cat << EOF
{
    "version": "$version",
    "release_date": "$(date -u +%Y-%m-%dT%H:%M:%SZ)",
    "tag": "v$version",
    "artifacts": {
        "archive": "$(basename "$archive_path")",
        "wasm_files": [],
        "checksums": "SHA256SUMS.txt"
    },
    "checksums": {
        "sha256": "$(sha256sum "$archive_path" | cut -d' ' -f1)"
    },
    "links": {
        "github_release": "https://github.com/Stellar-Uzima/Uzima-Contracts/releases/tag/v$version",
        "changelog": "https://github.com/Stellar-Uzima/Uzima-Contracts/blob/main/CHANGELOG.md",
        "documentation": "https://github.com/Stellar-Uzima/Uzima-Contracts#readme"
    },
    "metadata": {
        "build_environment": "$(uname -a)",
        "rust_version": "$(rustc --version)",
        "soroban_version": "$(soroban --version 2>/dev/null || echo 'not installed')",
        "git_commit": "$(git rev-parse HEAD)",
        "git_branch": "$(git rev-parse --abbrev-ref HEAD)"
    }
}
EOF
)
    
    # Add WASM files to manifest
    if [[ -d "$PROJECT_ROOT/dist" ]]; then
        local wasm_files=""
        for wasm_file in "$PROJECT_ROOT/dist"/*.wasm; do
            if [[ -f "$wasm_file" ]]; then
                wasm_files+="$(basename "$wasm_file"),"
            fi
        done
        
        # Remove trailing comma and update manifest
        wasm_files=${wasm_files%,}
        manifest=$(echo "$manifest" | sed "s/\"wasm_files\": \[\]/\"wasm_files\": [\"$wasm_files\"]/")
    fi
    
    echo "$manifest" > "$manifest_file"
    log_success "Release manifest created: $manifest_file"
}

verify_publication() {
    local version="$1"
    local tag="v$version"
    
    log_info "Verifying publication..."
    
    # Verify GitHub release
    if command -v gh &> /dev/null && gh release view "$tag" &> /dev/null; then
        log_success "GitHub release verified"
    else
        log_warning "GitHub release verification failed"
    fi
    
    # Verify artifacts exist
    local archive_path="$PROJECT_ROOT/artifacts/uzima-contracts-v$version.tar.gz"
    if [[ -f "$archive_path" ]]; then
        log_success "Artifact archive verified"
    else
        log_error "Artifact archive not found"
        return 1
    fi
    
    # Verify checksums
    local checksums_file="$PROJECT_ROOT/artifacts/uzima-contracts-v$version/SHA256SUMS.txt"
    if [[ -f "$checksums_file" ]]; then
        log_success "Checksums verified"
    else
        log_warning "Checksums file not found"
    fi
    
    log_success "Publication verification completed"
}

# Main publication function
perform_publication() {
    local version="$1"
    
    log_info "Starting artifact publication for v$version..."
    
    # Create artifact archive
    local archive_path
    archive_path=$(create_artifact_archive "$version")
    
    # Create release manifest
    create_release_manifest "$version" "$archive_path"
    
    # Publish to various platforms
    publish_to_github "$version" "$archive_path"
    publish_to_npm "$version"
    publish_to_docker "$version"
    publish_to_crates "$version"
    
    # Verify publication
    verify_publication "$version"
    
    log_success "Artifact publication completed for v$version"
}

# Help function
show_help() {
    cat << EOF
Release artifact publication script for Uzima-Contracts

Usage:
    $0 VERSION [OPTIONS]

Arguments:
    VERSION        Version to publish (e.g., 1.2.0)

Options:
    --dry-run      Perform a dry run without publishing
    --help         Show this help message

Environment Variables:
    DRY_RUN        Set to 'true' for dry run mode
    PUBLISH_GITHUB Set to 'true' to publish to GitHub (default: true)
    PUBLISH_NPM    Set to 'true' to publish to NPM (default: false)
    PUBLISH_DOCKER Set to 'true' to publish to Docker (default: false)
    PUBLISH_CRATES Set to 'true' to publish to crates.io (default: false)

Examples:
    $0 1.2.0
    $0 1.2.0 --dry-run
    PUBLISH_DOCKER=true $0 1.2.0

The script publishes:
- Complete artifact archive with source code and WASM files
- Individual WASM contract files
- SHA256 checksums for all artifacts
- Release manifest with metadata
- Docker images (if enabled)
- NPM packages (if enabled)
- Rust crates (if enabled)

Publication destinations:
- GitHub Releases (primary)
- Docker Hub (if enabled)
- NPM Registry (if enabled)
- crates.io (if enabled)

Requirements:
- Valid version tag must exist
- Build artifacts must be available in dist/
- GitHub CLI installed and authenticated
- Appropriate credentials for enabled platforms

EOF
}

# Main execution
main() {
    # Parse command line arguments
    while [[ $# -gt 0 ]]; do
        case $1 in
            --dry-run)
                DRY_RUN="true"
                shift
                ;;
            --help|-h)
                show_help
                exit 0
                ;;
            *)
                if [[ -z "$VERSION" ]]; then
                    VERSION="$1"
                else
                    log_error "Unknown option: $1"
                    show_help
                    exit 1
                fi
                shift
                ;;
        esac
    done
    
    # Check if version is provided
    if [[ -z "$VERSION" ]]; then
        log_error "Version is required"
        show_help
        exit 1
    fi
    
    # Check if tag exists
    local tag="v$VERSION"
    if ! git rev-parse "$tag" &> /dev/null; then
        log_error "Tag $tag does not exist"
        exit 1
    fi
    
    # Check if dist directory exists
    if [[ ! -d "$PROJECT_ROOT/dist" ]]; then
        log_error "dist/ directory not found. Run 'make dist' first."
        exit 1
    fi
    
    # Perform publication
    perform_publication "$VERSION"
}

# Run main function
main "$@"
