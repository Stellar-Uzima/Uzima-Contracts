#!/bin/bash
# Release validation script for Uzima-Contracts
# Usage: ./scripts/validate_release.sh VERSION [OPTIONS]

set -euo pipefail

# Configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
VERSION=${1:-}
SKIP_TESTS=${SKIP_TESTS:-false}
SKIP_SECURITY=${SKIP_SECURITY:-false}
STRICT=${STRICT:-false}

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Validation results
VALIDATION_ERRORS=0
VALIDATION_WARNINGS=0

# Logging functions
log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

log_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
    ((VALIDATION_WARNINGS++))
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
    ((VALIDATION_ERRORS++))
}

# Validation functions
validate_version_format() {
    local version="$1"
    log_info "Validating version format: $version"
    
    if [[ ! "$version" =~ ^[0-9]+\.[0-9]+\.[0-9]+(-[a-zA-Z0-9]+)?$ ]]; then
        log_error "Invalid version format: $version"
        log_error "Expected format: X.Y.Z or X.Y.Z-PRERELEASE"
        return 1
    fi
    
    log_success "Version format is valid"
}

validate_git_state() {
    log_info "Validating git state..."
    
    # Check if we're in a git repository
    if ! git rev-parse --git-dir &> /dev/null; then
        log_error "Not in a git repository"
        return 1
    fi
    
    # Check if working directory is clean
    if [[ -n $(git status --porcelain) ]]; then
        log_error "Working directory is not clean"
        git status --porcelain
        return 1
    fi
    
    # Check if tag already exists
    local tag="v$VERSION"
    if git rev-parse "$tag" &> /dev/null; then
        log_error "Tag $tag already exists"
        return 1
    fi
    
    log_success "Git state is valid"
}

validate_dependencies() {
    log_info "Validating dependencies..."
    
    # Check required commands
    local required_commands=("git" "cargo" "soroban")
    for cmd in "${required_commands[@]}"; do
        if ! command -v "$cmd" &> /dev/null; then
            log_error "Required command not found: $cmd"
            return 1
        fi
    done
    
    # Check Rust version
    local rust_version=$(rustc --version | cut -d' ' -f2)
    log_info "Rust version: $rust_version"
    
    # Check Soroban version
    local soroban_version=$(soroban --version | cut -d' ' -f2)
    log_info "Soroban version: $soroban_version"
    
    log_success "Dependencies are valid"
}

validate_changelog() {
    log_info "Validating changelog..."
    
    local changelog_file="$PROJECT_ROOT/CHANGELOG.md"
    
    if [[ ! -f "$changelog_file" ]]; then
        log_error "CHANGELOG.md not found"
        return 1
    fi
    
    # Check if version entry exists
    if ! grep -q "## \[$VERSION\]" "$changelog_file"; then
        log_error "Changelog entry for version $VERSION not found"
        return 1
    fi
    
    # Check changelog format
    local version_entry=$(sed -n "/## \[$VERSION\]/,/^## /p" "$changelog_file" | sed '$d')
    
    # Check for required sections
    local has_content=false
    if echo "$version_entry" | grep -q "### Added"; then
        has_content=true
    fi
    if echo "$version_entry" | grep -q "### Fixed"; then
        has_content=true
    fi
    if echo "$version_entry" | grep -q "### Changed"; then
        has_content=true
    fi
    
    if [[ "$has_content" == "false" ]]; then
        log_warning "Changelog entry appears to be empty"
    fi
    
    log_success "Changelog is valid"
}

validate_code_quality() {
    log_info "Validating code quality..."
    
    cd "$PROJECT_ROOT"
    
    # Format check
    if ! cargo fmt --all -- --check; then
        log_error "Code formatting check failed"
        return 1
    fi
    
    # Clippy check
    if ! cargo clippy --all-targets --all-features -- -D warnings; then
        log_error "Clippy check failed"
        return 1
    fi
    
    log_success "Code quality is valid"
}

validate_tests() {
    if [[ "$SKIP_TESTS" == "true" ]]; then
        log_warning "Skipping tests (SKIP_TESTS=true)"
        return 0
    fi
    
    log_info "Running tests..."
    
    cd "$PROJECT_ROOT"
    
    # Unit tests
    if ! cargo test --lib; then
        log_error "Unit tests failed"
        return 1
    fi
    
    # Integration tests
    if ! cargo test --test integration; then
        log_error "Integration tests failed"
        return 1
    fi
    
    log_success "All tests passed"
}

validate_build() {
    log_info "Validating build..."
    
    cd "$PROJECT_ROOT"
    
    # Clean build
    make clean
    
    # Build optimized contracts
    if ! make build-opt; then
        log_error "Build failed"
        return 1
    fi
    
    # Check WASM sizes
    if ! make check-wasm-size; then
        log_warning "WASM size check failed"
        if [[ "$STRICT" == "true" ]]; then
            return 1
        fi
    fi
    
    log_success "Build is valid"
}

validate_security() {
    if [[ "$SKIP_SECURITY" == "true" ]]; then
        log_warning "Skipping security checks (SKIP_SECURITY=true)"
        return 0
    fi
    
    log_info "Running security validation..."
    
    cd "$PROJECT_ROOT"
    
    # Security audit
    if command -v cargo-audit &> /dev/null; then
        if ! cargo audit; then
            log_error "Security audit failed"
            return 1
        fi
    else
        log_warning "cargo-audit not installed, skipping security audit"
    fi
    
    # Security-focused clippy
    if ! cargo clippy --all-targets --all-features -- -W clippy::indexing_slicing -W clippy::panic -W clippy::unwrap_used; then
        log_error "Security-focused clippy failed"
        return 1
    fi
    
    log_success "Security validation passed"
}

validate_versions() {
    log_info "Validating version consistency..."
    
    cd "$PROJECT_ROOT"
    
    # Check workspace version
    local workspace_version=$(grep '^version = ' Cargo.toml | head -1 | cut -d'"' -f2)
    if [[ "$workspace_version" != "$VERSION" ]]; then
        log_error "Workspace version mismatch: expected $VERSION, found $workspace_version"
        return 1
    fi
    
    # Check contract versions
    for cargo_toml in contracts/*/Cargo.toml; do
        if [[ -f "$cargo_toml" ]]; then
            local contract_name=$(basename "$(dirname "$cargo_toml")")
            local contract_version=$(grep '^version = ' "$cargo_toml" | cut -d'"' -f2)
            
            if [[ "$contract_version" != "$VERSION" ]]; then
                log_error "Contract $contract_name version mismatch: expected $VERSION, found $contract_version"
                return 1
            fi
        fi
    done
    
    log_success "Version consistency validated"
}

validate_documentation() {
    log_info "Validating documentation..."
    
    # Check README exists
    if [[ ! -f "$PROJECT_ROOT/README.md" ]]; then
        log_error "README.md not found"
        return 1
    fi
    
    # Check versioning documentation
    local versioning_docs=(
        "docs/VERSIONING_STRATEGY.md"
        "docs/RELEASE_PROCESS.md"
        "docs/CHANGELOG_FORMAT.md"
    )
    
    for doc in "${versioning_docs[@]}"; do
        if [[ ! -f "$PROJECT_ROOT/$doc" ]]; then
            log_error "Documentation file not found: $doc"
            return 1
        fi
    done
    
    log_success "Documentation is valid"
}

validate_artifacts() {
    log_info "Validating build artifacts..."
    
    cd "$PROJECT_ROOT"
    
    # Create dist directory
    make dist
    
    # Check if WASM files exist
    local wasm_files=dist/*.wasm
    local file_count=$(ls $wasm_files 2>/dev/null | wc -l)
    
    if [[ $file_count -eq 0 ]]; then
        log_error "No WASM files found in dist/"
        return 1
    fi
    
    # Validate each WASM file
    for wasm_file in $wasm_files; do
        if [[ -f "$wasm_file" ]]; then
            local file_size=$(stat -f%z "$wasm_file" 2>/dev/null || stat -c%s "$wasm_file" 2>/dev/null)
            
            if [[ $file_size -eq 0 ]]; then
                log_error "WASM file is empty: $wasm_file"
                return 1
            fi
            
            if [[ $file_size -gt 65536 ]]; then
                log_warning "WASM file exceeds size limit: $wasm_file ($file_size bytes)"
                if [[ "$STRICT" == "true" ]]; then
                    return 1
                fi
            fi
        fi
    done
    
    log_success "Build artifacts are valid"
}

validate_migration_requirements() {
    local version="$1"
    log_info "Validating migration requirements..."
    
    # Check if this is a major version
    if [[ "$version" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
        local major=$(echo "$version" | cut -d. -f1)
        local minor=$(echo "$version" | cut -d. -f2)
        
        if [[ "$minor" == "0" && "$major" > "1" ]]; then
            # This is a major release, check for migration guide
            local migration_guide="$PROJECT_ROOT/docs/migration/v$version.md"
            if [[ ! -f "$migration_guide" ]]; then
                log_warning "Migration guide not found: $migration_guide"
                if [[ "$STRICT" == "true" ]]; then
                    return 1
                fi
            fi
        fi
    fi
    
    log_success "Migration requirements validated"
}

# Main validation function
perform_validation() {
    local version="$1"
    
    log_info "Starting comprehensive release validation for v$version..."
    echo
    
    # Core validations
    validate_version_format "$version"
    validate_git_state
    validate_dependencies
    validate_changelog
    validate_versions
    validate_documentation
    validate_code_quality
    
    # Build and tests
    validate_build
    validate_tests
    
    # Security and artifacts
    validate_security
    validate_artifacts
    
    # Migration requirements
    validate_migration_requirements "$version"
    
    echo
    log_info "Validation completed"
    echo
    
    # Summary
    if [[ $VALIDATION_ERRORS -gt 0 ]]; then
        log_error "Validation failed with $VALIDATION_ERRORS error(s)"
        return 1
    fi
    
    if [[ $VALIDATION_WARNINGS -gt 0 ]]; then
        log_warning "Validation completed with $VALIDATION_WARNINGS warning(s)"
    else
        log_success "All validations passed successfully! 🎉"
    fi
    
    return 0
}

# Help function
show_help() {
    cat << EOF
Release validation script for Uzima-Contracts

Usage:
    $0 VERSION [OPTIONS]

Arguments:
    VERSION        Version to validate (e.g., 1.2.0, 1.2.0-alpha.1)

Options:
    --skip-tests      Skip running tests
    --skip-security   Skip security checks
    --strict          Treat warnings as errors
    --help            Show this help message

Environment Variables:
    SKIP_TESTS        Set to 'true' to skip tests
    SKIP_SECURITY     Set to 'true' to skip security checks
    STRICT            Set to 'true' for strict validation

Examples:
    $0 1.2.0
    $0 1.2.0-alpha.1 --skip-tests
    $0 2.0.0 --strict

The script validates:
- Version format and git state
- Changelog existence and format
- Code quality (formatting, clippy)
- Build process and WASM artifacts
- Test suite execution
- Security audit and checks
- Version consistency across workspace
- Documentation completeness
- Migration requirements for major releases

Exit codes:
- 0: Success
- 1: Validation failed

EOF
}

# Main execution
main() {
    # Parse command line arguments
    while [[ $# -gt 0 ]]; do
        case $1 in
            --skip-tests)
                SKIP_TESTS="true"
                shift
                ;;
            --skip-security)
                SKIP_SECURITY="true"
                shift
                ;;
            --strict)
                STRICT="true"
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
    
    # Perform validation
    perform_validation "$VERSION"
}

# Run main function
main "$@"
