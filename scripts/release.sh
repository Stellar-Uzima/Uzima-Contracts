#!/bin/bash
# Release automation script for Uzima-Contracts
# Usage: ./scripts/release.sh VERSION [RELEASE_TYPE]

set -euo pipefail

# Configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
VERSION=${1:-}
RELEASE_TYPE=${2:-minor}
DRY_RUN=${DRY_RUN:-false}

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

# Validation functions
validate_version() {
    local version="$1"
    if [[ ! "$version" =~ ^[0-9]+\.[0-9]+\.[0-9]+(-[a-zA-Z0-9]+)?$ ]]; then
        log_error "Invalid version format: $version"
        log_error "Expected format: X.Y.Z or X.Y.Z-PRERELEASE"
        exit 1
    fi
}

validate_dependencies() {
    log_info "Validating dependencies..."
    
    # Check required commands
    local required_commands=("git" "cargo" "soroban" "gh")
    for cmd in "${required_commands[@]}"; do
        if ! command -v "$cmd" &> /dev/null; then
            log_error "Required command not found: $cmd"
            exit 1
        fi
    done
    
    # Check if we're in a git repository
    if ! git rev-parse --git-dir &> /dev/null; then
        log_error "Not in a git repository"
        exit 1
    fi
    
    # Check if working directory is clean
    if [[ -n $(git status --porcelain) ]]; then
        log_error "Working directory is not clean"
        git status --porcelain
        exit 1
    fi
    
    log_success "Dependencies validated"
}

validate_version_not_exists() {
    local version="$1"
    local tag="v$version"
    
    if git rev-parse "$tag" &> /dev/null; then
        log_error "Tag $tag already exists"
        exit 1
    fi
    
    log_success "Version $version is available"
}

# Version bump functions
bump_version() {
    local version="$1"
    log_info "Bumping version to $version..."
    
    # Update workspace version
    sed -i.bak "s/^version = .*/version = \"$version\"/" "$PROJECT_ROOT/Cargo.toml"
    rm "$PROJECT_ROOT/Cargo.toml.bak"
    
    # Update contract versions
    find "$PROJECT_ROOT/contracts" -name "Cargo.toml" -type f | while read -r cargo_toml; do
        if grep -q "^version = " "$cargo_toml"; then
            sed -i.bak "s/^version = .*/version = \"$version\"/" "$cargo_toml"
            rm "$cargo_toml.bak"
        fi
    done
    
    log_success "Version bumped to $version"
}

# Changelog functions
update_changelog() {
    local version="$1"
    local release_date=$(date +%Y-%m-%d)
    
    log_info "Updating changelog for version $version..."
    
    local changelog_file="$PROJECT_ROOT/CHANGELOG.md"
    
    # Check if changelog exists
    if [[ ! -f "$changelog_file" ]]; then
        log_warning "CHANGELOG.md not found, creating new one"
        create_initial_changelog "$changelog_file"
    fi
    
    # Move Unreleased section to new version
    if grep -q "## \[Unreleased\]" "$changelog_file"; then
        # Replace Unreleased with new version
        sed -i.bak "s/## \[Unreleased\]/## [$version] - $release_date/" "$changelog_file"
        rm "$changelog_file.bak"
        
        # Add new Unreleased section at the top
        sed -i.bak "/^## \[$version\] - $release_date/i\\
## [Unreleased]\\
\\
" "$changelog_file"
        rm "$changelog_file.bak"
    else
        # Add new version section
        sed -i.bak "/^# Changelog/a\\
\\
## [$version] - $release_date\\
\\
### Added\\
- Release version $version\\
\\
### Fixed\\
- Bug fixes and improvements\\
\\
" "$changelog_file"
        rm "$changelog_file.bak"
    fi
    
    log_success "Changelog updated for version $version"
}

create_initial_changelog() {
    local changelog_file="$1"
    
    cat > "$changelog_file" << 'EOF'
# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]
EOF
}

# Build and test functions
build_and_test() {
    log_info "Building and testing..."
    
    cd "$PROJECT_ROOT"
    
    # Clean build
    make clean
    
    # Build optimized contracts
    make build-opt
    
    # Run tests
    make test
    
    # Run integration tests
    make test-integration
    
    # Check code quality
    make check
    
    # Check WASM sizes
    make check-wasm-size
    
    log_success "Build and tests completed"
}

# Git functions
commit_changes() {
    local version="$1"
    
    log_info "Committing version changes..."
    
    cd "$PROJECT_ROOT"
    
    # Add version changes
    git add Cargo.toml
    git add "contracts/*/Cargo.toml"
    git add CHANGELOG.md
    
    # Commit
    git commit -m "chore: bump version to $version"
    
    log_success "Changes committed"
}

create_tag() {
    local version="$1"
    local tag="v$version"
    
    log_info "Creating tag $tag..."
    
    cd "$PROJECT_ROOT"
    
    # Create annotated tag
    git tag -a "$tag" -m "Release $tag"
    
    log_success "Tag $tag created"
}

push_changes() {
    local version="$1"
    local tag="v$version"
    
    if [[ "$DRY_RUN" == "true" ]]; then
        log_info "DRY RUN: Would push changes and tag"
        return
    fi
    
    log_info "Pushing changes and tag..."
    
    cd "$PROJECT_ROOT"
    
    # Push current branch
    git push origin
    
    # Push tag
    git push origin "$tag"
    
    log_success "Changes and tag pushed"
}

# Deployment functions
deploy_contracts() {
    local version="$1"
    
    log_info "Deploying contracts for version $version..."
    
    cd "$PROJECT_ROOT"
    
    # Deploy to testnet
    if [[ "$DRY_RUN" == "true" ]]; then
        log_info "DRY RUN: Would deploy contracts to testnet"
    else
        ./scripts/deploy_environment.sh testnet
    fi
    
    log_success "Contracts deployed"
}

# GitHub release functions
create_github_release() {
    local version="$1"
    local tag="v$version"
    
    log_info "Creating GitHub release for $tag..."
    
    cd "$PROJECT_ROOT"
    
    if [[ "$DRY_RUN" == "true" ]]; then
        log_info "DRY RUN: Would create GitHub release"
        return
    fi
    
    # Extract changelog entry
    local changelog_entry=$(sed -n "/## \[$version\]/,/^## /p" CHANGELOG.md | sed '$d')
    
    # Create GitHub release
    echo "$changelog_entry" | gh release create "$tag" \
        --title "Release $tag" \
        --notes-file "-" \
        --target main
    
    log_success "GitHub release created"
}

# Notification functions
send_notifications() {
    local version="$1"
    
    log_info "Sending notifications for version $version..."
    
    if [[ "$DRY_RUN" == "true" ]]; then
        log_info "DRY RUN: Would send notifications"
        return
    fi
    
    # Send Slack notification (if configured)
    if command -v slack-cli &> /dev/null && [[ -n "${SLACK_WEBHOOK:-}" ]]; then
        slack-cli send "🚀 Uzima-Contracts v$version released! Details: https://github.com/Stellar-Uzima/Uzima-Contracts/releases/tag/v$version" || true
    fi
    
    # Send email notification (if configured)
    if [[ -f "$PROJECT_ROOT/scripts/notify_email.sh" ]]; then
        "$PROJECT_ROOT/scripts/notify_email.sh" --version "$version" || true
    fi
    
    log_success "Notifications sent"
}

# Cleanup functions
cleanup() {
    log_info "Cleaning up..."
    
    # Remove any temporary files
    find "$PROJECT_ROOT" -name "*.bak" -delete 2>/dev/null || true
    
    log_success "Cleanup completed"
}

# Main release function
perform_release() {
    local version="$1"
    
    log_info "Starting release process for v$version..."
    
    # Validation
    validate_version "$version"
    validate_dependencies
    validate_version_not_exists "$version"
    
    # Version bump
    bump_version "$version"
    
    # Changelog update
    update_changelog "$version"
    
    # Build and test
    build_and_test
    
    # Git operations
    commit_changes "$version"
    create_tag "$version"
    push_changes "$version"
    
    # Deployment
    deploy_contracts "$version"
    
    # GitHub release
    create_github_release "$version"
    
    # Notifications
    send_notifications "$version"
    
    # Cleanup
    cleanup
    
    log_success "Release v$version completed successfully! 🚀"
}

# Help function
show_help() {
    cat << EOF
Release automation script for Uzima-Contracts

Usage:
    $0 VERSION [RELEASE_TYPE] [OPTIONS]

Arguments:
    VERSION        Version to release (e.g., 1.2.0, 1.2.0-alpha.1)
    RELEASE_TYPE   Release type: patch, minor, major (default: minor)

Options:
    --dry-run      Perform a dry run without making actual changes
    --help         Show this help message

Examples:
    $0 1.2.0 minor
    $0 1.1.1 patch
    $0 2.0.0 major
    $0 1.3.0-alpha.1 minor --dry-run

Environment Variables:
    DRY_RUN        Set to 'true' for dry run mode
    SLACK_WEBHOOK  Slack webhook URL for notifications

Release Types:
    patch      - Bug fixes and security updates
    minor      - New features and enhancements
    major      - Breaking changes and significant updates

The script will:
1. Validate the version format and dependencies
2. Bump version in Cargo.toml files
3. Update CHANGELOG.md
4. Build and test the project
5. Commit changes and create git tag
6. Push changes to remote
7. Deploy contracts to testnet
8. Create GitHub release
9. Send notifications

Requirements:
- Clean git working directory
- All tests passing
- Proper git configuration
- GitHub CLI installed and authenticated
- Soroban CLI installed

EOF
}

# Main execution
main() {
    # Check for help flag
    if [[ "${1:-}" == "--help" ]] || [[ "${1:-}" == "-h" ]]; then
        show_help
        exit 0
    fi
    
    # Check if version is provided
    if [[ -z "${VERSION:-}" ]]; then
        log_error "Version is required"
        show_help
        exit 1
    fi
    
    # Check for dry run flag
    if [[ "${3:-}" == "--dry-run" ]] || [[ "${DRY_RUN:-}" == "true" ]]; then
        export DRY_RUN="true"
        log_info "Running in dry-run mode"
    fi
    
    # Perform release
    perform_release "$VERSION"
}

# Trap for cleanup
trap cleanup EXIT

# Run main function
main "$@"
