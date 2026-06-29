#!/bin/bash
# Changelog generation script for Uzima-Contracts
# Usage: ./scripts/generate_changelog.sh [OPTIONS]

set -euo pipefail

# Configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
FROM_VERSION=${FROM_VERSION:-}
TO_VERSION=${TO_VERSION:-}
VERSION=${VERSION:-}
INTERACTIVE=${INTERACTIVE:-false}
FORMAT=${FORMAT:-markdown}

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

# Git functions
get_git_commits() {
    local from_tag="${1:-}"
    local to_tag="${2:-HEAD}"
    
    if [[ -n "$from_tag" ]]; then
        git log --pretty=format:"%H|%s|%an|%ad" --date=short "$from_tag..$to_tag"
    else
        git log --pretty=format:"%H|%s|%an|%ad" --date=short "$to_tag"
    fi
}

get_last_tag() {
    git describe --tags --abbrev=0 2>/dev/null || echo ""
}

# Commit parsing functions
parse_commit_message() {
    local commit_message="$1"
    
    # Extract conventional commit type
    if [[ "$commit_message" =~ ^(feat|fix|docs|style|refactor|test|chore|perf|ci|build|revert)(\(.+\))?:\s+(.+) ]]; then
        local type="${BASH_REMATCH[1]}"
        local scope="${BASH_REMATCH[2]}"
        local description="${BASH_REMATCH[3]}"
        
        case "$type" in
            feat)
                echo "Added|$description"
                ;;
            fix)
                echo "Fixed|$description"
                ;;
            docs)
                echo "Changed|$description (documentation)"
                ;;
            style)
                echo "Changed|$description (code style)"
                ;;
            refactor)
                echo "Changed|$description (refactoring)"
                ;;
            perf)
                echo "Changed|$description (performance)"
                ;;
            test)
                echo "Changed|$description (testing)"
                ;;
            ci|build)
                echo "Changed|$description (CI/CD)"
                ;;
            chore)
                echo "Changed|$description (maintenance)"
                ;;
            revert)
                echo "Fixed|$description (revert)"
                ;;
            *)
                echo "Changed|$description"
                ;;
        esac
    else
        echo "Changed|$commit_message"
    fi
}

# Changelog generation functions
generate_changelog_from_commits() {
    local from_tag="${1:-}"
    local to_tag="${2:-HEAD}"
    
    log_info "Generating changelog from git history..."
    
    # Get commits
    local commits
    commits=$(get_git_commits "$from_tag" "$to_tag")
    
    if [[ -z "$commits" ]]; then
        log_warning "No commits found in the specified range"
        return
    fi
    
    # Initialize sections
    local added_section=""
    local fixed_section=""
    local changed_section=""
    local security_section=""
    local breaking_section=""
    
    # Process commits
    while IFS='|' read -r hash message author date; do
        if [[ -n "$message" ]]; then
            local parsed
            parsed=$(parse_commit_message "$message")
            
            local section=$(echo "$parsed" | cut -d'|' -f1)
            local description=$(echo "$parsed" | cut -d'|' -f2)
            
            # Check for breaking changes
            if [[ "$message" =~ BREAKING[[:space:]]+CHANGE:?[[:space:]]*(.+) ]]; then
                breaking_section+="- ${BASH_REMATCH[1]} ($hash)\n"
            fi
            
            # Check for security fixes
            if [[ "$message" =~ (security|fix.*security|cve|vulnerability) ]]; then
                security_section+="- $description ($hash)\n"
            fi
            
            # Add to appropriate section
            case "$section" in
                "Added")
                    added_section+="- $description ($hash)\n"
                    ;;
                "Fixed")
                    fixed_section+="- $description ($hash)\n"
                    ;;
                "Changed")
                    changed_section+="- $description ($hash)\n"
                    ;;
            esac
        fi
    done <<< "$commits"
    
    # Generate changelog entry
    local changelog_entry=""
    local release_date=$(date +%Y-%m-%d)
    
    if [[ -n "$VERSION" ]]; then
        changelog_entry+="## [$VERSION] - $release_date\n\n"
    else
        changelog_entry+="## [Unreleased]\n\n"
    fi
    
    # Add sections if they have content
    if [[ -n "$added_section" ]]; then
        changelog_entry+="### Added\n$added_section\n"
    fi
    
    if [[ -n "$fixed_section" ]]; then
        changelog_entry+="### Fixed\n$fixed_section\n"
    fi
    
    if [[ -n "$changed_section" ]]; then
        changelog_entry+="### Changed\n$changed_section\n"
    fi
    
    if [[ -n "$security_section" ]]; then
        changelog_entry+="### Security\n$security_section\n"
    fi
    
    if [[ -n "$breaking_section" ]]; then
        changelog_entry+="### Breaking Changes\n$breaking_section\n"
    fi
    
    echo -e "$changelog_entry"
}

# Interactive changelog generation
interactive_changelog() {
    log_info "Interactive changelog generation..."
    
    # Get available tags
    local tags
    tags=$(git tag --sort=-version:refname | head -10)
    
    echo "Available tags:"
    echo "$tags"
    echo
    
    # Ask for from version
    read -p "Enter from tag (leave empty for all commits): " from_tag
    
    # Ask for to version
    local default_to="HEAD"
    read -p "Enter to tag (default: $default_to): " to_tag
    to_tag=${to_tag:-$default_to}
    
    # Generate changelog
    local changelog
    changelog=$(generate_changelog_from_commits "$from_tag" "$to_tag")
    
    echo
    echo "Generated changelog:"
    echo "===================="
    echo -e "$changelog"
    echo
    
    # Ask to save
    read -p "Save to CHANGELOG.md? (y/N): " save_changelog
    if [[ "$save_changelog" =~ ^[Yy]$ ]]; then
        save_changelog_to_file "$changelog"
    fi
}

# File operations
save_changelog_to_file() {
    local changelog_entry="$1"
    local changelog_file="$PROJECT_ROOT/CHANGELOG.md"
    
    log_info "Saving changelog to $changelog_file..."
    
    # Create changelog if it doesn't exist
    if [[ ! -f "$changelog_file" ]]; then
        create_initial_changelog "$changelog_file"
    fi
    
    # Backup original file
    cp "$changelog_file" "$changelog_file.bak"
    
    if [[ -n "$VERSION" ]]; then
        # Replace Unreleased section with versioned entry
        if grep -q "## \[Unreleased\]" "$changelog_file"; then
            # Insert version entry after Unreleased
            sed -i.bak "/## \[Unreleased\]/r /dev/stdin" "$changelog_file" <<< "$changelog_entry"
        else
            # Insert at the beginning after header
            sed -i.bak "2i\\
\\
$changelog_entry" "$changelog_file"
        fi
    else
        # Update Unreleased section
        if grep -q "## \[Unreleased\]" "$changelog_file"; then
            # Replace existing Unreleased section
            sed -i.bak "/## \[Unreleased\]/,/^## /c\\
## [Unreleased]\\
\\
$(echo -e "$changelog_entry" | sed '1d;$d')" "$changelog_file"
        else
            # Add Unreleased section
            sed -i.bak "2i\\
\\
## [Unreleased]\\
\\
$(echo -e "$changelog_entry" | sed '1d;$d')" "$changelog_file"
        fi
    fi
    
    # Clean up backup
    rm "$changelog_file.bak" 2>/dev/null || true
    
    log_success "Changelog saved to $changelog_file"
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
    
    log_info "Created initial changelog"
}

# Validation functions
validate_git_state() {
    log_info "Validating git state..."
    
    # Check if we're in a git repository
    if ! git rev-parse --git-dir &> /dev/null; then
        log_error "Not in a git repository"
        exit 1
    fi
    
    log_success "Git state validated"
}

# Help function
show_help() {
    cat << EOF
Changelog generation script for Uzima-Contracts

Usage:
    $0 [OPTIONS]

Options:
    --from VERSION    Start from this version/tag
    --to VERSION      End at this version/tag (default: HEAD)
    --version VER     Create changelog for specific version
    --interactive     Interactive mode
    --format FORMAT   Output format: markdown (default), json
    --help            Show this help message

Environment Variables:
    FROM_VERSION      Start version/tag
    TO_VERSION        End version/tag
    VERSION           Target version
    INTERACTIVE       Set to 'true' for interactive mode
    FORMAT            Output format

Examples:
    $0 --from v1.1.0 --to v1.2.0
    $0 --version 1.2.0
    $0 --interactive
    $0 --from v1.1.0

The script will:
1. Parse git commit messages using conventional commits
2. Categorize changes into sections (Added, Fixed, Changed, etc.)
3. Generate changelog entry in proper format
4. Save to CHANGELOG.md if requested

Conventional Commit Types:
- feat:     New features
- fix:      Bug fixes
- docs:     Documentation changes
- style:    Code style changes
- refactor: Code refactoring
- test:     Test changes
- chore:    Maintenance tasks
- perf:     Performance improvements
- ci:       CI/CD changes
- build:    Build system changes
- revert:   Revert changes

Breaking Changes:
Add "BREAKING CHANGE:" to commit message to mark breaking changes.

Security Issues:
Commits containing 'security', 'cve', or 'vulnerability' are marked as security fixes.

EOF
}

# Main execution
main() {
    # Parse command line arguments
    while [[ $# -gt 0 ]]; do
        case $1 in
            --from)
                FROM_VERSION="$2"
                shift 2
                ;;
            --to)
                TO_VERSION="$2"
                shift 2
                ;;
            --version)
                VERSION="$2"
                shift 2
                ;;
            --interactive)
                INTERACTIVE="true"
                shift
                ;;
            --format)
                FORMAT="$2"
                shift 2
                ;;
            --help|-h)
                show_help
                exit 0
                ;;
            *)
                log_error "Unknown option: $1"
                show_help
                exit 1
                ;;
        esac
    done
    
    # Validate git state
    validate_git_state
    
    # Interactive mode
    if [[ "$INTERACTIVE" == "true" ]]; then
        interactive_changelog
        exit 0
    fi
    
    # Auto mode
    log_info "Generating changelog..."
    
    # Determine from/to versions
    if [[ -z "$FROM_VERSION" ]]; then
        FROM_VERSION=$(get_last_tag)
        if [[ -n "$FROM_VERSION" ]]; then
            log_info "Using last tag: $FROM_VERSION"
        else
            log_warning "No tags found, using all commits"
            FROM_VERSION=""
        fi
    fi
    
    if [[ -z "$TO_VERSION" ]]; then
        TO_VERSION="HEAD"
    fi
    
    # Generate changelog
    local changelog
    changelog=$(generate_changelog_from_commits "$FROM_VERSION" "$TO_VERSION")
    
    # Output changelog
    echo -e "$changelog"
    
    # Save to file if version specified
    if [[ -n "$VERSION" ]]; then
        save_changelog_to_file "$changelog"
    fi
    
    log_success "Changelog generation completed!"
}

# Run main function
main "$@"
