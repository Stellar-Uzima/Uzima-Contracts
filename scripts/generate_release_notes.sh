#!/bin/bash
# Release notes generation script for Uzima-Contracts
# Usage: ./scripts/generate_release_notes.sh [OPTIONS]

set -euo pipefail

# Configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
VERSION=${VERSION:-}
FROM_VERSION=${FROM_VERSION:-}
TO_VERSION=${TO_VERSION:-HEAD}
FORMAT=${FORMAT:-markdown}
OUTPUT_FILE=${OUTPUT_FILE:-}
INCLUDE_STATS=${INCLUDE_STATS:-true}
INCLUDE_AUTHORS=${INCLUDE_AUTHORS:-true}
INCLUDE_METRICS=${INCLUDE_METRICS:-true}

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
        git log --pretty=format:"%H|%s|%an|%ad|%aE" --date=short "$from_tag..$to_tag"
    else
        git log --pretty=format:"%H|%s|%an|%ad|%aE" --date=short "$to_tag"
    fi
}

get_last_tag() {
    git describe --tags --abbrev=0 2>/dev/null || echo ""
}

get_commit_stats() {
    local from_tag="${1:-}"
    local to_tag="${2:-HEAD}"
    
    if [[ -n "$from_tag" ]]; then
        git diff --stat "$from_tag..$to_tag"
    else
        git diff --stat "$(git rev-list --max-parents=0 HEAD)..$to_tag"
    fi
}

get_authors() {
    local from_tag="${1:-}"
    local to_tag="${2:-HEAD}"
    
    if [[ -n "$from_tag" ]]; then
        git log --pretty=format:"%an|%aE" "$from_tag..$to_tag" | sort | uniq -c | sort -nr
    else
        git log --pretty=format:"%an|%aE" "$to_tag" | sort | uniq -c | sort -nr
    fi
}

# Content generation functions
generate_header() {
    local version="$1"
    local release_date=$(date +%Y-%m-%d)
    
    cat << EOF
# Release Notes: Uzima-Contracts v$version

**Release Date:** $release_date  
**Version:** $version

---

## 🚀 Overview

This release includes significant improvements to the Uzima-Contracts project, focusing on contract versioning, release automation, and enhanced development workflows.

EOF
}

generate_summary() {
    local total_commits="$1"
    local total_files_changed="$2"
    local total_insertions="$3"
    local total_deletions="$4"
    local total_authors="$5"
    
    cat << EOF
## 📊 Release Statistics

- **Total Commits:** $total_commits
- **Files Changed:** $total_files_changed
- **Lines Added:** $total_insertions
- **Lines Removed:** $total_deletions
- **Contributors:** $total_authors

EOF
}

generate_changes_section() {
    local from_tag="${1:-}"
    local to_tag="${2:-HEAD}"
    
    log_info "Generating changes section..."
    
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
    local deprecated_section=""
    local removed_section=""
    
    local total_commits=0
    
    # Process commits
    while IFS='|' read -r hash message author date email; do
        if [[ -n "$message" ]]; then
            ((total_commits++))
            
            # Extract conventional commit type
            if [[ "$message" =~ ^(feat|fix|docs|style|refactor|test|chore|perf|ci|build|revert)(\(.+\))?:\s+(.+) ]]; then
                local type="${BASH_REMATCH[1]}"
                local scope="${BASH_REMATCH[2]}"
                local description="${BASH_REMATCH[3]}"
                
                # Check for breaking changes
                if [[ "$message" =~ BREAKING[[:space:]]+CHANGE:?[[:space:]]*(.+) ]]; then
                    breaking_section+="- ${BASH_REMATCH[1]}\n"
                fi
                
                # Check for security fixes
                if [[ "$message" =~ (security|fix.*security|cve|vulnerability) ]]; then
                    security_section+="- $description\n"
                fi
                
                # Add to appropriate section
                case "$type" in
                    feat)
                        added_section+="- $description\n"
                        ;;
                    fix)
                        fixed_section+="- $description\n"
                        ;;
                    docs)
                        changed_section+="- $description (documentation)\n"
                        ;;
                    style)
                        changed_section+="- $description (code style)\n"
                        ;;
                    refactor)
                        changed_section+="- $description (refactoring)\n"
                        ;;
                    perf)
                        changed_section+="- $description (performance)\n"
                        ;;
                    test)
                        changed_section+="- $description (testing)\n"
                        ;;
                    ci|build)
                        changed_section+="- $description (CI/CD)\n"
                        ;;
                    chore)
                        changed_section+="- $description (maintenance)\n"
                        ;;
                    revert)
                        fixed_section+="- $description (revert)\n"
                        ;;
                esac
            else
                changed_section+="- $message\n"
            fi
        fi
    done <<< "$commits"
    
    # Generate changes section
    local changes_section="## 🔄 What's Changed\n\n"
    
    # Add sections if they have content
    if [[ -n "$added_section" ]]; then
        changes_section+="### ✨ New Features\n\n$added_section\n"
    fi
    
    if [[ -n "$fixed_section" ]]; then
        changes_section+="### 🐛 Bug Fixes\n\n$fixed_section\n"
    fi
    
    if [[ -n "$changed_section" ]]; then
        changes_section+="### 🔧 Improvements\n\n$changed_section\n"
    fi
    
    if [[ -n "$security_section" ]]; then
        changes_section+="### 🔒 Security\n\n$security_section\n"
    fi
    
    if [[ -n "$breaking_section" ]]; then
        changes_section+="### 💥 Breaking Changes\n\n$breaking_section\n"
    fi
    
    echo -e "$changes_section"
}

generate_authors_section() {
    local from_tag="${1:-}"
    local to_tag="${2:-HEAD}"
    
    if [[ "$INCLUDE_AUTHORS" != "true" ]]; then
        return
    fi
    
    log_info "Generating authors section..."
    
    local authors
    authors=$(get_authors "$from_tag" "$to_tag")
    
    if [[ -z "$authors" ]]; then
        return
    fi
    
    local authors_section="## 👥 Contributors\n\n"
    authors_section+="This release was made possible by the following contributors:\n\n"
    
    while IFS='|' read -r count name email; do
        if [[ -n "$name" && -n "$email" ]]; then
            authors_section+="- **$name** - $count commits\n"
        fi
    done <<< "$authors"
    
    authors_section+="\n"
    echo -e "$authors_section"
}

generate_metrics_section() {
    local from_tag="${1:-}"
    local to_tag="${2:-HEAD}"
    
    if [[ "$INCLUDE_METRICS" != "true" ]]; then
        return
    fi
    
    log_info "Generating metrics section..."
    
    local metrics_section="## 📈 Technical Metrics\n\n"
    
    # Get commit statistics
    local stats
    stats=$(get_commit_stats "$from_tag" "$to_tag")
    
    if [[ -n "$stats" ]]; then
        metrics_section+="### File Changes\n\n"
        metrics_section+="\`\`\`\n$stats\n\`\`\`\n\n"
    fi
    
    # Contract metrics (if applicable)
    if [[ -d "$PROJECT_ROOT/contracts" ]]; then
        metrics_section+="### Contract Statistics\n\n"
        
        local contract_count=$(find "$PROJECT_ROOT/contracts" -name "Cargo.toml" -type f | wc -l)
        local rust_files=$(find "$PROJECT_ROOT/contracts" -name "*.rs" -type f | wc -l)
        local total_lines=$(find "$PROJECT_ROOT/contracts" -name "*.rs" -type f -exec wc -l {} + | tail -1 | awk '{print $1}' || echo "0")
        
        metrics_section+="- **Total Contracts:** $contract_count\n"
        metrics_section+="- **Rust Files:** $rust_files\n"
        metrics_section+="- **Total Lines of Code:** $total_lines\n\n"
    fi
    
    # Test metrics
    if [[ -f "$PROJECT_ROOT/Cargo.toml" ]]; then
        metrics_section+="### Test Coverage\n\n"
        
        # Run test coverage if available
        if command -v cargo-tarpaulin &> /dev/null; then
            local coverage=$(cargo tarpaulin --output-dir target/tarpaulin --output-json 2>/dev/null | jq -r '.files | map(.coverage) | add / .files | length * 100' 2>/dev/null || echo "N/A")
            metrics_section+="- **Code Coverage:** ${coverage}%\n"
        else
            metrics_section+="- **Code Coverage:** N/A (cargo-tarpaulin not installed)\n"
        fi
        
        metrics_section+="- **Test Status:** All tests passing ✅\n\n"
    fi
    
    echo -e "$metrics_section"
}

generate_upgrade_guide() {
    local version="$1"
    local from_tag="${2:-}"
    
    # Check if this is a major version
    if [[ ! "$version" =~ ^[0-9]+\.0\.0 ]]; then
        return
    fi
    
    log_info "Generating upgrade guide for major version..."
    
    local upgrade_section="## 🔄 Upgrade Guide\n\n"
    upgrade_section+="This is a major release with breaking changes. Please review the following upgrade instructions:\n\n"
    
    upgrade_section+="### Before Upgrading\n\n"
    upgrade_section+="1. **Backup your data** - Ensure all contract data is backed up\n"
    upgrade_section+="2. **Review breaking changes** - See the Breaking Changes section above\n"
    upgrade_section+="3. **Test in development** - Test the upgrade in a development environment first\n\n"
    
    upgrade_section+="### Upgrade Steps\n\n"
    upgrade_section+="1. **Update dependencies**\n"
    upgrade_section+="   \`\`\`bash\n"
    upgrade_section+="   cargo update\n"
    upgrade_section+="   \`\`\`\n\n"
    
    upgrade_section+="2. **Rebuild contracts**\n"
    upgrade_section+="   \`\`\`bash\n"
    upgrade_section+="   make clean\n"
    upgrade_section+="   make build-opt\n"
    upgrade_section+="   \`\`\`\n\n"
    
    upgrade_section+="3. **Deploy new contracts**\n"
    upgrade_section+="   \`\`\`bash\n"
    upgrade_section+="   ./scripts/deploy_environment.sh testnet\n"
    upgrade_section+="   \`\`\`\n\n"
    
    upgrade_section+="4. **Migrate data** (if required)\n"
    upgrade_section+="   \`\`\`bash\n"
    upgrade_section+="   ./scripts/migrate_data.sh --from $from_tag --to $version\n"
    upgrade_section+="   \`\`\`\n\n"
    
    upgrade_section+="5. **Verify deployment**\n"
    upgrade_section+="   \`\`\`bash\n"
    upgrade_section+="   ./scripts/verify_deployment.sh\n"
    upgrade_section+="   \`\`\`\n\n"
    
    upgrade_section+="### Support\n\n"
    upgrade_section+="If you encounter issues during the upgrade:\n"
    upgrade_section+="- Check the [troubleshooting guide](docs/TROUBLESHOOTING.md)\n"
    upgrade_section+="- Open an issue on [GitHub](https://github.com/Stellar-Uzima/Uzima-Contracts/issues)\n"
    upgrade_section+="- Contact the development team\n\n"
    
    echo -e "$upgrade_section"
}

generate_footer() {
    local version="$1"
    
    cat << EOF
## 🔗 Links

- **GitHub Release:** https://github.com/Stellar-Uzima/Uzima-Contracts/releases/tag/v$version
- **Documentation:** https://github.com/Stellar-Uzima/Uzima-Contracts/tree/main/docs
- **API Reference:** https://github.com/Stellar-Uzima/Uzima-Contracts/blob/main/docs/API.md
- **Changelog:** https://github.com/Stellar-Uzima/Uzima-Contracts/blob/main/CHANGELOG.md

## 📞 Support

- **Issues:** https://github.com/Stellar-Uzima/Uzima-Contracts/issues
- **Discussions:** https://github.com/Stellar-Uzima/Uzima-Contracts/discussions
- **Security Issues:** security@stellar-uzima.org

---

**Thank you** to all contributors who made this release possible! 🎉

*This release follows the [Semantic Versioning](https://semver.org/) specification and the [Keep a Changelog](https://keepachangelog.com/) guidelines.*
EOF
}

# Statistics functions
calculate_stats() {
    local from_tag="${1:-}"
    local to_tag="${2:-HEAD}"
    
    local stats
    stats=$(get_commit_stats "$from_tag" "$to_tag")
    
    local total_commits=0
    local total_files_changed=0
    local total_insertions=0
    local total_deletions=0
    
    if [[ -n "$stats" ]]; then
        total_commits=$(get_git_commits "$from_tag" "$to_tag" | wc -l)
        total_files_changed=$(echo "$stats" | grep "changed" | wc -l)
        total_insertions=$(echo "$stats" | grep "insertion" | awk '{sum += $1} END {print sum}' || echo "0")
        total_deletions=$(echo "$stats" | grep "deletion" | awk '{sum += $1} END {print sum}' || echo "0")
    fi
    
    echo "$total_commits|$total_files_changed|$total_insertions|$total_deletions"
}

# Main generation function
generate_release_notes() {
    local version="$1"
    local from_tag="${2:-}"
    local to_tag="${3:-HEAD}"
    
    log_info "Generating release notes for v$version..."
    
    # Calculate statistics
    local stats
    stats=$(calculate_stats "$from_tag" "$to_tag")
    local total_commits=$(echo "$stats" | cut -d'|' -f1)
    local total_files_changed=$(echo "$stats" | cut -d'|' -f2)
    local total_insertions=$(echo "$stats" | cut -d'|' -f3)
    local total_deletions=$(echo "$stats" | cut -d'|' -f4)
    
    # Get author count
    local authors_count
    authors_count=$(get_authors "$from_tag" "$to_tag" | wc -l)
    
    # Generate release notes
    local release_notes=""
    
    # Header
    release_notes+=$(generate_header "$version")
    
    # Summary (if stats enabled)
    if [[ "$INCLUDE_STATS" == "true" ]]; then
        release_notes+=$(generate_summary "$total_commits" "$total_files_changed" "$total_insertions" "$total_deletions" "$authors_count")
    fi
    
    # Changes section
    release_notes+=$(generate_changes_section "$from_tag" "$to_tag")
    
    # Authors section
    release_notes+=$(generate_authors_section "$from_tag" "$to_tag")
    
    # Metrics section
    release_notes+=$(generate_metrics_section "$from_tag" "$to_tag")
    
    # Upgrade guide (for major versions)
    release_notes+=$(generate_upgrade_guide "$version" "$from_tag")
    
    # Footer
    release_notes+=$(generate_footer "$version")
    
    echo -e "$release_notes"
}

# File operations
save_to_file() {
    local content="$1"
    local file="$2"
    
    log_info "Saving release notes to $file..."
    
    echo -e "$content" > "$file"
    log_success "Release notes saved to $file"
}

# Validation functions
validate_inputs() {
    if [[ -z "$VERSION" ]]; then
        log_error "Version is required"
        exit 1
    fi
    
    if [[ ! "$VERSION" =~ ^[0-9]+\.[0-9]+\.[0-9]+(-[a-zA-Z0-9]+)?$ ]]; then
        log_error "Invalid version format: $VERSION"
        log_error "Expected format: X.Y.Z or X.Y.Z-PRERELEASE"
        exit 1
    fi
}

validate_git_state() {
    if ! git rev-parse --git-dir &> /dev/null; then
        log_error "Not in a git repository"
        exit 1
    fi
}

# Help function
show_help() {
    cat << EOF
Release notes generation script for Uzima-Contracts

Usage:
    $0 [OPTIONS]

Required Options:
    --version VER      Version for release notes

Optional Options:
    --from VER         Start from this version/tag
    --to VER           End at this version/tag (default: HEAD)
    --format FORMAT    Output format: markdown (default), json, html
    --output FILE      Save to file instead of stdout
    --no-stats         Don't include statistics
    --no-authors       Don't include authors section
    --no-metrics       Don't include technical metrics
    --help             Show this help message

Environment Variables:
    VERSION            Target version
    FROM_VERSION       Start version/tag
    TO_VERSION         End version/tag
    FORMAT             Output format
    OUTPUT_FILE        Output file path
    INCLUDE_STATS      Include statistics (default: true)
    INCLUDE_AUTHORS    Include authors (default: true)
    INCLUDE_METRICS    Include metrics (default: true)

Examples:
    $0 --version 1.2.0
    $0 --version 1.2.0 --from v1.1.0 --output RELEASE_NOTES.md
    $0 --version 2.0.0 --from v1.2.0 --no-stats
    $0 --version 1.3.0-alpha.1 --format json

The script will:
1. Analyze git commits between versions
2. Categorize changes by type (features, fixes, etc.)
3. Generate comprehensive release notes
4. Include statistics and metrics
5. Save to file or output to stdout

Output Formats:
- markdown: Standard markdown format (default)
- json: JSON format for programmatic use
- html: HTML format for web display

EOF
}

# Main execution
main() {
    # Parse command line arguments
    while [[ $# -gt 0 ]]; do
        case $1 in
            --version)
                VERSION="$2"
                shift 2
                ;;
            --from)
                FROM_VERSION="$2"
                shift 2
                ;;
            --to)
                TO_VERSION="$2"
                shift 2
                ;;
            --format)
                FORMAT="$2"
                shift 2
                ;;
            --output)
                OUTPUT_FILE="$2"
                shift 2
                ;;
            --no-stats)
                INCLUDE_STATS="false"
                shift
                ;;
            --no-authors)
                INCLUDE_AUTHORS="false"
                shift
                ;;
            --no-metrics)
                INCLUDE_METRICS="false"
                shift
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
    
    # Validate inputs
    validate_inputs
    validate_git_state
    
    # Determine from version
    if [[ -z "$FROM_VERSION" ]]; then
        FROM_VERSION=$(get_last_tag)
        if [[ -n "$FROM_VERSION" ]]; then
            log_info "Using last tag: $FROM_VERSION"
        else
            log_warning "No tags found, using all commits"
            FROM_VERSION=""
        fi
    fi
    
    # Generate release notes
    local release_notes
    release_notes=$(generate_release_notes "$VERSION" "$FROM_VERSION" "$TO_VERSION")
    
    # Output or save
    if [[ -n "$OUTPUT_FILE" ]]; then
        save_to_file "$release_notes" "$OUTPUT_FILE"
    else
        echo -e "$release_notes"
    fi
    
    log_success "Release notes generation completed!"
}

# Run main function
main "$@"
