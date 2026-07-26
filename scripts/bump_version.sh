#!/bin/bash
# Version bump script for Uzima-Contracts
# Usage: ./scripts/bump_version.sh VERSION

set -euo pipefail

# Configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
VERSION=${1:-}

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

validate_workspace() {
    log_info "Validating workspace..."
    
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
    
    log_success "Workspace validated"
}

# Version bump functions
bump_workspace_version() {
    local version="$1"
    local cargo_toml="$PROJECT_ROOT/Cargo.toml"
    
    log_info "Bumping workspace version to $version..."
    
    if [[ ! -f "$cargo_toml" ]]; then
        log_error "Cargo.toml not found at $cargo_toml"
        exit 1
    fi
    
    # Backup original file
    cp "$cargo_toml" "$cargo_toml.bak"
    
    # Update workspace version
    sed -i.bak "s/^version = .*/version = \"$version\"/" "$cargo_toml"
    
    # Verify the change
    if grep -q "^version = \"$version\"" "$cargo_toml"; then
        log_success "Workspace version updated to $version"
        rm "$cargo_toml.bak"
    else
        log_error "Failed to update workspace version"
        mv "$cargo_toml.bak" "$cargo_toml"
        exit 1
    fi
}

bump_contract_versions() {
    local version="$1"
    local contracts_dir="$PROJECT_ROOT/contracts"
    
    log_info "Bumping contract versions to $version..."
    
    if [[ ! -d "$contracts_dir" ]]; then
        log_warning "Contracts directory not found at $contracts_dir"
        return
    fi
    
    local updated_count=0
    
    # Find all Cargo.toml files in contracts directory
    while IFS= read -r -d '' cargo_toml; do
        if grep -q "^version = " "$cargo_toml"; then
            # Backup original file
            cp "$cargo_toml" "$cargo_toml.bak"
            
            # Update version
            sed -i.bak "s/^version = .*/version = \"$version\"/" "$cargo_toml"
            
            # Verify the change
            if grep -q "^version = \"$version\"" "$cargo_toml"; then
                local contract_name=$(basename "$(dirname "$cargo_toml")")
                log_info "Updated $contract_name version to $version"
                rm "$cargo_toml.bak"
                ((updated_count++))
            else
                log_error "Failed to update version in $cargo_toml"
                mv "$cargo_toml.bak" "$cargo_toml"
                exit 1
            fi
        fi
    done < <(find "$contracts_dir" -name "Cargo.toml" -type f -print0)
    
    log_success "Updated $updated_count contract versions"
}

update_version_constants() {
    local version="$1"
    
    log_info "Updating version constants in source code..."
    
    # Update version constants in Rust source files
    find "$PROJECT_ROOT/contracts" -name "*.rs" -type f | while read -rs rust_file; do
        if grep -q "const CONTRACT_VERSION" "$rust_file"; then
            # Backup original file
            cp "$rust_file" "$rust_file.bak"
            
            # Update version constant
            sed -i.bak "s/const CONTRACT_VERSION: &str = .*/const CONTRACT_VERSION: &str = \"$version\";/" "$rust_file"
            
            # Verify the change
            if grep -q "const CONTRACT_VERSION: &str = \"$version\"" "$rust_file"; then
                local contract_name=$(basename "$(dirname "$rust_file")")
                log_info "Updated version constant in $contract_name"
                rm "$rust_file.bak"
            else
                log_error "Failed to update version constant in $rust_file"
                mv "$rust_file.bak" "$rust_file"
                exit 1
            fi
        fi
    done
    
    log_success "Version constants updated"
}

update_documentation() {
    local version="$1"
    
    log_info "Updating documentation references..."
    
    # Update README.md
    local readme="$PROJECT_ROOT/README.md"
    if [[ -f "$readme" ]]; then
        # Update version references in README
        sed -i.bak "s/v[0-9]\+\.[0-9]\+\.[0-9]\+/v$version/g" "$readme"
        rm "$readme.bak" 2>/dev/null || true
        log_info "Updated README.md version references"
    fi
    
    # Update other documentation files
    find "$PROJECT_ROOT/docs" -name "*.md" -type f | while read -rs doc_file; do
        # Update version references
        sed -i.bak "s/v[0-9]\+\.[0-9]\+\.[0-9]\+/v$version/g" "$doc_file"
        rm "$doc_file.bak" 2>/dev/null || true
    done
    
    log_success "Documentation updated"
}

# Validation functions
verify_version_consistency() {
    local version="$1"
    
    log_info "Verifying version consistency..."
    
    local inconsistencies=0
    
    # Check workspace version
    if ! grep -q "^version = \"$version\"" "$PROJECT_ROOT/Cargo.toml"; then
        log_error "Workspace version mismatch in Cargo.toml"
        ((inconsistencies++))
    fi
    
    # Check contract versions
    find "$PROJECT_ROOT/contracts" -name "Cargo.toml" -type f | while read -rs cargo_toml; do
        if grep -q "^version = " "$cargo_toml"; then
            if ! grep -q "^version = \"$version\"" "$cargo_toml"; then
                local contract_name=$(basename "$(dirname "$cargo_toml")")
                log_error "Version mismatch in $contract_name/Cargo.toml"
                ((inconsistencies++))
            fi
        fi
    done
    
    if [[ $inconsistencies -gt 0 ]]; then
        log_error "Found $inconsistencies version inconsistencies"
        exit 1
    fi
    
    log_success "Version consistency verified"
}

# Git functions
commit_version_changes() {
    local version="$1"
    
    log_info "Committing version changes..."
    
    cd "$PROJECT_ROOT"
    
    # Add all version-related changes
    git add Cargo.toml
    git add "contracts/*/Cargo.toml"
    git add "contracts/**/*.rs"
    git add "README.md"
    git add "docs/**/*.md"
    
    # Check if there are changes to commit
    if git diff --cached --quiet; then
        log_warning "No changes to commit"
        return
    fi
    
    # Commit changes
    git commit -m "chore: bump version to $version"
    
    log_success "Version changes committed"
}

# Help function
show_help() {
    cat << EOF
Version bump script for Uzima-Contracts

Usage:
    $0 VERSION

Arguments:
    VERSION        Version to bump to (e.g., 1.2.0, 1.2.0-alpha.1)

Examples:
    $0 1.2.0
    $0 1.1.1
    $0 2.0.0
    $0 1.3.0-alpha.1

The script will:
1. Validate the version format
2. Check workspace state
3. Update workspace version in Cargo.toml
4. Update contract versions in contracts/*/Cargo.toml
5. Update version constants in source code
6. Update documentation references
7. Verify version consistency
8. Commit changes to git

Requirements:
- Clean git working directory
- Valid semantic version format
- Proper project structure

Version Format:
- X.Y.Z where X=MAJOR, Y=MINOR, Z=PATCH
- Optional pre-release suffix: X.Y.Z-alpha.1, X.Y.Z-beta.2, X.Y.Z-rc.1

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
    
    log_info "Starting version bump to $VERSION..."
    
    # Validation
    validate_version "$VERSION"
    validate_workspace
    
    # Version bump operations
    bump_workspace_version "$VERSION"
    bump_contract_versions "$VERSION"
    update_version_constants "$VERSION"
    update_documentation "$VERSION"
    
    # Verification
    verify_version_consistency "$VERSION"
    
    # Git operations
    commit_version_changes "$VERSION"
    
    log_success "Version bump to $VERSION completed successfully! 🚀"
}

# Run main function
main "$@"
