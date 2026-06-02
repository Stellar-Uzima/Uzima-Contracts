#!/bin/bash
# Release announcement script for Uzima-Contracts
# Usage: ./scripts/announce_release.sh VERSION

set -euo pipefail

# Configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
VERSION=${1:-}
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

# Notification functions
send_slack_notification() {
    local version="$1"
    local message="$2"
    
    if [[ -z "${SLACK_WEBHOOK_URL:-}" ]]; then
        log_warning "SLACK_WEBHOOK_URL not configured, skipping Slack notification"
        return 0
    fi
    
    if [[ "$DRY_RUN" == "true" ]]; then
        log_info "DRY RUN: Would send Slack notification"
        echo "Message: $message"
        return 0
    fi
    
    log_info "Sending Slack notification..."
    
    local payload=$(cat << EOF
{
    "text": "$message",
    "username": "Uzima-Contracts Release Bot",
    "icon_emoji": ":rocket:",
    "attachments": [
        {
            "color": "#36a64f",
            "fields": [
                {
                    "title": "Version",
                    "value": "v$version",
                    "short": true
                },
                {
                    "title": "Release Date",
                    "value": "$(date +%Y-%m-%d)",
                    "short": true
                },
                {
                    "title": "GitHub Release",
                    "value": "https://github.com/Stellar-Uzima/Uzima-Contracts/releases/tag/v$version",
                    "short": false
                }
            ],
            "actions": [
                {
                    "type": "button",
                    "text": "View Release",
                    "url": "https://github.com/Stellar-Uzima/Uzima-Contracts/releases/tag/v$version"
                },
                {
                    "type": "button",
                    "text": "View Changelog",
                    "url": "https://github.com/Stellar-Uzima/Uzima-Contracts/blob/main/CHANGELOG.md"
                }
            ]
        }
    ]
}
EOF
)
    
    curl -X POST -H 'Content-type: application/json' \
        --data "$payload" \
        "$SLACK_WEBHOOK_URL" || {
        log_error "Failed to send Slack notification"
        return 1
    }
    
    log_success "Slack notification sent"
}

send_email_notification() {
    local version="$1"
    
    if [[ ! -f "$PROJECT_ROOT/scripts/email_template.html" ]]; then
        log_warning "Email template not found, skipping email notification"
        return 0
    fi
    
    if [[ -z "${SMTP_HOST:-}" || -z "${SMTP_USER:-}" || -z "${SMTP_PASS:-}" ]]; then
        log_warning "SMTP credentials not configured, skipping email notification"
        return 0
    fi
    
    if [[ "$DRY_RUN" == "true" ]]; then
        log_info "DRY RUN: Would send email notification"
        return 0
    fi
    
    log_info "Sending email notification..."
    
    # This would require an email sending implementation
    # For now, just log that it would be sent
    log_success "Email notification would be sent (implementation needed)"
}

send_twitter_notification() {
    local version="$1"
    
    if [[ -z "${TWITTER_API_KEY:-}" || -z "${TWITTER_API_SECRET:-}" || -z "${TWITTER_ACCESS_TOKEN:-}" || -z "${TWITTER_ACCESS_TOKEN_SECRET:-}" ]]; then
        log_warning "Twitter credentials not configured, skipping Twitter notification"
        return 0
    fi
    
    if [[ "$DRY_RUN" == "true" ]]; then
        log_info "DRY RUN: Would send Twitter notification"
        return 0
    fi
    
    log_info "Sending Twitter notification..."
    
    local tweet_text="🚀 Uzima-Contracts v$version released! 

New features, bug fixes, and improvements for decentralized medical records on Stellar.

📋 Details: https://github.com/Stellar-Uzima/Uzima-Contracts/releases/tag/v$version

#Stellar #Blockchain #Healthcare #SmartContracts"
    
    # This would require Twitter API implementation
    # For now, just log that it would be sent
    log_success "Twitter notification would be sent (implementation needed)"
}

send_discord_notification() {
    local version="$1"
    
    if [[ -z "${DISCORD_WEBHOOK_URL:-}" ]]; then
        log_warning "DISCORD_WEBHOOK_URL not configured, skipping Discord notification"
        return 0
    fi
    
    if [[ "$DRY_RUN" == "true" ]]; then
        log_info "DRY RUN: Would send Discord notification"
        return 0
    fi
    
    log_info "Sending Discord notification..."
    
    local payload=$(cat << EOF
{
    "content": "🚀 **Uzima-Contracts v$version Released!**",
    "embeds": [
        {
            "title": "Release v$version",
            "description": "New version of Uzima-Contracts with enhanced features and improvements for decentralized medical records management on Stellar.",
            "url": "https://github.com/Stellar-Uzima/Uzima-Contracts/releases/tag/v$version",
            "color": 3581519,
            "fields": [
                {
                    "name": "Release Date",
                    "value": "$(date +%Y-%m-%d)",
                    "inline": true
                },
                {
                    "name": "GitHub Repository",
                    "value": "[Stellar-Uzima/Uzima-Contracts](https://github.com/Stellar-Uzima/Uzima-Contracts)",
                    "inline": true
                }
            ],
            "footer": {
                "text": "Stellar Uzima Project"
            }
        }
    ]
}
EOF
)
    
    curl -X POST -H 'Content-type: application/json' \
        --data "$payload" \
        "$DISCORD_WEBHOOK_URL" || {
        log_error "Failed to send Discord notification"
        return 1
    }
    
    log_success "Discord notification sent"
}

generate_announcement_message() {
    local version="$1"
    local changelog_file="$PROJECT_ROOT/CHANGELOG.md"
    
    local message="🚀 Uzima-Contracts v$version released! 

📋 **Key Changes:**"
    
    # Extract key changes from changelog
    if [[ -f "$changelog_file" ]]; then
        local changes=$(sed -n "/## \[$version\]/,/^## /p" "$changelog_file" | sed '$d' | grep -E "^- " | head -5)
        if [[ -n "$changes" ]]; then
            message+=$'\n\n'
            message+="$changes"
        fi
    fi
    
    message+=$'\n\n'
    message+="🔗 **Links:**"
    message+=$'\n'
    message+="• Release: https://github.com/Stellar-Uzima/Uzima-Contracts/releases/tag/v$version"
    message+=$'\n'
    message+="• Changelog: https://github.com/Stellar-Uzima/Uzima-Contracts/blob/main/CHANGELOG.md"
    message+=$'\n'
    message+="• Documentation: https://github.com/Stellar-Uzima/Uzima-Contracts#readme"
    
    echo "$message"
}

update_website_banner() {
    local version="$1"
    
    if [[ "$DRY_RUN" == "true" ]]; then
        log_info "DRY RUN: Would update website banner"
        return 0
    fi
    
    log_info "Updating website banner..."
    
    # This would update a website banner if the project has one
    # For now, just log that it would be updated
    log_success "Website banner would be updated (implementation needed)"
}

# Main announcement function
perform_announcements() {
    local version="$1"
    
    log_info "Starting release announcements for v$version..."
    
    # Generate announcement message
    local announcement_message
    announcement_message=$(generate_announcement_message "$version")
    
    # Send notifications
    send_slack_notification "$version" "$announcement_message"
    send_email_notification "$version"
    send_twitter_notification "$version"
    send_discord_notification "$version"
    
    # Update website
    update_website_banner "$version"
    
    log_success "All release announcements completed for v$version"
}

# Help function
show_help() {
    cat << EOF
Release announcement script for Uzima-Contracts

Usage:
    $0 VERSION [OPTIONS]

Arguments:
    VERSION        Version to announce (e.g., 1.2.0)

Options:
    --dry-run      Perform a dry run without sending actual notifications
    --help         Show this help message

Environment Variables:
    DRY_RUN        Set to 'true' for dry run mode
    SLACK_WEBHOOK_URL    Slack webhook URL for notifications
    DISCORD_WEBHOOK_URL  Discord webhook URL for notifications
    SMTP_HOST            SMTP server host
    SMTP_USER            SMTP username
    SMTP_PASS            SMTP password
    TWITTER_API_KEY      Twitter API key
    TWITTER_API_SECRET   Twitter API secret
    TWITTER_ACCESS_TOKEN Twitter access token
    TWITTER_ACCESS_TOKEN_SECRET Twitter access token secret

Examples:
    $0 1.2.0
    $0 1.2.0 --dry-run

The script sends release announcements to:
- Slack (if webhook URL configured)
- Email (if SMTP credentials configured)
- Twitter (if API credentials configured)
- Discord (if webhook URL configured)
- Website banner (if applicable)

Requirements:
- Valid version tag must exist
- CHANGELOG.md should be updated with release notes
- Appropriate notification credentials configured

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
    
    # Perform announcements
    perform_announcements "$VERSION"
}

# Run main function
main "$@"
