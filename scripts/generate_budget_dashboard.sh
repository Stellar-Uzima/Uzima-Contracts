#!/bin/bash
# generate_budget_dashboard.sh - Generate and open resource budget dashboard

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
DASHBOARD_OUTPUT="$PROJECT_ROOT/dashboard/resource-budgets.html"

RED='\033[0;31m'
GREEN='\033[0;32m'
BLUE='\033[0;34m'
NC='\033[0m'

log_info() { echo -e "${BLUE}[INFO]${NC} $1"; }
log_success() { echo -e "${GREEN}[SUCCESS]${NC} $1"; }
log_error() { echo -e "${RED}[ERROR]${NC} $1"; }

OPEN_BROWSER=false
CHECK_ALERTS_ONLY=false

show_help() {
    cat <<EOF
Generate resource budget dashboard for Uzima contracts.

Usage: $0 [OPTIONS]

Options:
    --open          Open dashboard in browser after generation
    --check-alerts  Check alerting configuration only
    --help          Show this help message
EOF
}

while [[ $# -gt 0 ]]; do
    case $1 in
        --open) OPEN_BROWSER=true; shift ;;
        --check-alerts) CHECK_ALERTS_ONLY=true; shift ;;
        --help|-h) show_help; exit 0 ;;
        *) shift ;;
    esac
done

if [[ "$CHECK_ALERTS_ONLY" == "true" ]]; then
    local alerting_file="$PROJECT_ROOT/resource-budgets/alerting.json"
    if [[ -f "$alerting_file" ]]; then
        if command -v python3 &>/dev/null; then
            if python3 -m json.tool "$alerting_file" >/dev/null 2>&1; then
                log_success "Alerting configuration is valid"
            else
                log_error "Alerting configuration is not valid JSON"
            fi
        fi
    fi
    exit 0
fi

log_info "Generating resource budget dashboard..."
if ! command -v node &>/dev/null; then
    log_error "Node.js is required. Install node to generate dashboard."
    exit 1
fi

node "$SCRIPT_DIR/generate_budget_dashboard.mjs" --output "$DASHBOARD_OUTPUT"

if [[ -f "$DASHBOARD_OUTPUT" ]]; then
    log_success "Dashboard generated: $DASHBOARD_OUTPUT"
else
    log_error "Dashboard generation failed"
    exit 1
fi

if [[ "$OPEN_BROWSER" == "true" ]]; then
    if command -v start &>/dev/null; then
        start "$DASHBOARD_OUTPUT"
    elif command -v open &>/dev/null; then
        open "$DASHBOARD_OUTPUT"
    elif command -v xdg-open &>/dev/null; then
        xdg-open "$DASHBOARD_OUTPUT"
    else
        log_info "Open $DASHBOARD_OUTPUT in your browser"
    fi
fi
