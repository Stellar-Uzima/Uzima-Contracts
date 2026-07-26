#!/bin/bash
# Release health check script for Uzima-Contracts
# Usage: ./scripts/check_release_health.sh VERSION

set -euo pipefail

# Configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
VERSION=${1:-}
NETWORK=${NETWORK:-testnet}
HEALTH_CHECK_TIMEOUT=${HEALTH_CHECK_TIMEOUT:-300}
ALERT_ON_FAILURE=${ALERT_ON_FAILURE:-false}

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Health check results
HEALTH_ERRORS=0
HEALTH_WARNINGS=0

# Logging functions
log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

log_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
    ((HEALTH_WARNINGS++))
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
    ((HEALTH_ERRORS++))
}

# Health check functions
check_github_release() {
    local version="$1"
    local tag="v$version"
    
    log_info "Checking GitHub release health..."
    
    if ! command -v gh &> /dev/null; then
        log_warning "GitHub CLI not installed, skipping GitHub release check"
        return 0
    fi
    
    # Check if release exists
    if ! gh release view "$tag" &> /dev/null; then
        log_error "GitHub release $tag does not exist"
        return 1
    fi
    
    # Check if release has assets
    local asset_count
    asset_count=$(gh release view "$tag" --json assets --jq '.assets | length')
    
    if [[ $asset_count -eq 0 ]]; then
        log_warning "GitHub release has no assets"
    else
        log_success "GitHub release has $asset_count assets"
    fi
    
    # Check if release is published (not draft)
    local release_state
    release_state=$(gh release view "$tag" --json state --jq '.state')
    
    if [[ "$release_state" == "published" ]]; then
        log_success "GitHub release is published"
    else
        log_warning "GitHub release state: $release_state"
    fi
    
    log_success "GitHub release health check completed"
}

check_artifact_integrity() {
    local version="$1"
    
    log_info "Checking artifact integrity..."
    
    local artifacts_dir="$PROJECT_ROOT/artifacts/uzima-contracts-v$version"
    
    if [[ ! -d "$artifacts_dir" ]]; then
        log_error "Artifacts directory not found: $artifacts_dir"
        return 1
    fi
    
    # Check checksums file
    local checksums_file="$artifacts_dir/SHA256SUMS.txt"
    if [[ ! -f "$checksums_file" ]]; then
        log_error "Checksums file not found"
        return 1
    fi
    
    # Verify checksums
    cd "$artifacts_dir"
    if sha256sum -c "SHA256SUMS.txt" &> /dev/null; then
        log_success "All artifact checksums verified"
    else
        log_error "Artifact checksum verification failed"
        cd "$PROJECT_ROOT"
        return 1
    fi
    cd "$PROJECT_ROOT"
    
    # Check WASM files
    local wasm_files=("$artifacts_dir"/wasm/*.wasm)
    local wasm_count=0
    
    for wasm_file in "${wasm_files[@]}"; do
        if [[ -f "$wasm_file" ]]; then
            ((wasm_count++))
            
            # Check file size
            local file_size
            file_size=$(stat -f%z "$wasm_file" 2>/dev/null || stat -c%s "$wasm_file" 2>/dev/null)
            
            if [[ $file_size -eq 0 ]]; then
                log_error "WASM file is empty: $(basename "$wasm_file")"
                return 1
            fi
            
            if [[ $file_size -gt 65536 ]]; then
                log_warning "WASM file exceeds size limit: $(basename "$wasm_file") ($file_size bytes)"
            fi
        fi
    done
    
    if [[ $wasm_count -gt 0 ]]; then
        log_success "$wasm_count WASM files verified"
    else
        log_warning "No WASM files found in artifacts"
    fi
    
    log_success "Artifact integrity check completed"
}

check_contract_deployment() {
    local version="$1"
    local network="$2"
    
    log_info "Checking contract deployment health on $network..."
    
    if ! command -v soroban &> /dev/null; then
        log_warning "Soroban CLI not installed, skipping deployment check"
        return 0
    fi
    
    # Check deployment status file
    local deployment_file="$PROJECT_ROOT/deployments/${network}_deployment.json"
    if [[ ! -f "$deployment_file" ]]; then
        log_warning "Deployment file not found: $deployment_file"
        return 0
    fi
    
    # Parse deployment contracts
    local contracts
    contracts=$(jq -r '.contracts[]? | select(.version == "'$version'") | .contract_id' "$deployment_file" 2>/dev/null || echo "")
    
    if [[ -z "$contracts" ]]; then
        log_warning "No contracts found for version $version on $network"
        return 0
    fi
    
    local healthy_contracts=0
    local total_contracts=0
    
    while IFS= read -r contract_id; do
        if [[ -n "$contract_id" ]]; then
            ((total_contracts++))
            
            # Check if contract exists
            if soroban contract inspect --id "$contract_id" --network "$network" &> /dev/null; then
                # Check contract version
                local contract_version
                contract_version=$(soroban contract invoke --id "$contract_id" --network "$network" \
                    --wasm /dev/null --function get_version 2>/dev/null | jq -r '.result' 2>/dev/null || echo "")
                
                if [[ "$contract_version" == "$version" ]]; then
                    log_success "Contract $contract_id is healthy (version: $contract_version)"
                    ((healthy_contracts++))
                else
                    log_warning "Contract $contract_id version mismatch: expected $version, got $contract_version"
                fi
            else
                log_error "Contract $contract_id not found on $network"
            fi
        fi
    done <<< "$contracts"
    
    if [[ $healthy_contracts -eq $total_contracts ]]; then
        log_success "All $total_contracts contracts are healthy on $network"
    else
        log_warning "$healthy_contracts/$total_contracts contracts are healthy on $network"
    fi
    
    log_success "Contract deployment health check completed"
}

check_network_connectivity() {
    local network="$1"
    
    log_info "Checking network connectivity for $network..."
    
    if ! command -v soroban &> /dev/null; then
        log_warning "Soroban CLI not installed, skipping network connectivity check"
        return 0
    fi
    
    # Get network RPC URL
    local rpc_url
    case "$network" in
        "testnet")
            rpc_url="https://soroban-testnet.stellar.org:443"
            ;;
        "futurenet")
            rpc_url="https://rpc-futurenet.stellar.org:443"
            ;;
        "mainnet")
            rpc_url="https://mainnet.stellar.org:443"
            ;;
        "local")
            rpc_url="http://localhost:8000/soroban/rpc"
            ;;
        *)
            log_warning "Unknown network: $network"
            return 0
            ;;
    esac
    
    # Test connectivity
    if curl -s --max-time 10 "$rpc_url" | grep -q "jsonrpc\|result" 2>/dev/null; then
        log_success "Network connectivity to $network is healthy"
    else
        log_error "Network connectivity to $network failed"
        return 1
    fi
}

check_documentation_availability() {
    local version="$1"
    
    log_info "Checking documentation availability..."
    
    # Check if changelog has version entry
    local changelog_file="$PROJECT_ROOT/CHANGELOG.md"
    if [[ -f "$changelog_file" ]]; then
        if grep -q "## \[$version\]" "$changelog_file"; then
            log_success "Changelog entry found for v$version"
        else
            log_warning "Changelog entry not found for v$version"
        fi
    else
        log_warning "CHANGELOG.md not found"
    fi
    
    # Check if versioning documentation exists
    local versioning_docs=(
        "docs/VERSIONING_STRATEGY.md"
        "docs/RELEASE_PROCESS.md"
        "docs/CHANGELOG_FORMAT.md"
    )
    
    for doc in "${versioning_docs[@]}"; do
        if [[ -f "$PROJECT_ROOT/$doc" ]]; then
            log_success "Documentation available: $doc"
        else
            log_warning "Documentation missing: $doc"
        fi
    done
    
    log_success "Documentation availability check completed"
}

check_dependency_versions() {
    log_info "Checking dependency versions..."
    
    cd "$PROJECT_ROOT"
    
    # Check Rust version
    if command -v rustc &> /dev/null; then
        local rust_version
        rust_version=$(rustc --version | cut -d' ' -f2)
        log_info "Current Rust version: $rust_version"
        
        # Check if it matches expected version (from rust-toolchain.toml if exists)
        if [[ -f "$PROJECT_ROOT/rust-toolchain.toml" ]]; then
            local expected_rust_version
            expected_rust_version=$(grep -o 'channel = "[^"]*"' "$PROJECT_ROOT/rust-toolchain.toml" | cut -d'"' -f2)
            if [[ "$rust_version" == "$expected_rust_version"* ]]; then
                log_success "Rust version matches expected: $expected_rust_version"
            else
                log_warning "Rust version mismatch: expected $expected_rust_version, got $rust_version"
            fi
        fi
    fi
    
    # Check Soroban version
    if command -v soroban &> /dev/null; then
        local soroban_version
        soroban_version=$(soroban --version | cut -d' ' -f2)
        log_info "Current Soroban version: $soroban_version"
        log_success "Soroban CLI available"
    else
        log_warning "Soroban CLI not available"
    fi
    
    log_success "Dependency versions check completed"
}

check_performance_metrics() {
    local version="$1"
    
    log_info "Checking performance metrics..."
    
    # Check WASM file sizes
    local dist_dir="$PROJECT_ROOT/dist"
    if [[ -d "$dist_dir" ]]; then
        local total_size=0
        local file_count=0
        
        for wasm_file in "$dist_dir"/*.wasm; do
            if [[ -f "$wasm_file" ]]; then
                local file_size
                file_size=$(stat -f%z "$wasm_file" 2>/dev/null || stat -c%s "$wasm_file" 2>/dev/null)
                ((total_size += file_size))
                ((file_count++))
                
                # Check individual file size
                if [[ $file_size -gt 65536 ]]; then
                    log_warning "Large WASM file: $(basename "$wasm_file") ($file_size bytes)"
                fi
            fi
        done
        
        if [[ $file_count -gt 0 ]]; then
            local avg_size=$((total_size / file_count))
            log_info "Average WASM size: $avg_size bytes across $file_count files"
            log_success "Performance metrics collected"
        else
            log_warning "No WASM files found for performance analysis"
        fi
    else
        log_warning "dist/ directory not found, skipping performance metrics"
    fi
}

generate_health_report() {
    local version="$1"
    local report_file="$PROJECT_ROOT/artifacts/health-report-v$version.json"
    
    log_info "Generating health report..."
    
    local report=$(cat << EOF
{
    "version": "$version",
    "check_timestamp": "$(date -u +%Y-%m-%dT%H:%M:%SZ)",
    "health_status": "$([ $HEALTH_ERRORS -eq 0 ] && echo "healthy" || echo "unhealthy")",
    "errors": $HEALTH_ERRORS,
    "warnings": $HEALTH_WARNINGS,
    "checks_performed": [
        "github_release",
        "artifact_integrity", 
        "contract_deployment",
        "network_connectivity",
        "documentation_availability",
        "dependency_versions",
        "performance_metrics"
    ],
    "summary": {
        "overall_health": "$([ $HEALTH_ERRORS -eq 0 ] && echo "PASS" || echo "FAIL")",
        "recommendations": []
    }
}
EOF
)
    
    # Add recommendations based on results
    if [[ $HEALTH_ERRORS -gt 0 ]]; then
        report=$(echo "$report" | sed 's/"recommendations": \[\]/"recommendations": ["Address critical health errors before proceeding"]/')
    elif [[ $HEALTH_WARNINGS -gt 0 ]]; then
        report=$(echo "$report" | sed 's/"recommendations": \[\]/"recommendations": ["Review and address health warnings"]/')
    fi
    
    echo "$report" > "$report_file"
    log_success "Health report generated: $report_file"
}

send_health_alert() {
    local version="$1"
    local status="$2"
    
    if [[ "$ALERT_ON_FAILURE" != "true" && "$status" != "FAIL" ]]; then
        return 0
    fi
    
    log_info "Sending health alert..."
    
    local message="🏥 Release Health Alert - Uzima-Contracts v$version

Status: $status
Errors: $HEALTH_ERRORS
Warnings: $HEALTH_WARNINGS

Check the full health report for details."
    
    # Send to Slack if configured
    if [[ -n "${SLACK_WEBHOOK_URL:-}" ]]; then
        curl -X POST -H 'Content-type: application/json' \
            --data "{\"text\":\"$message\"}" \
            "$SLACK_WEBHOOK_URL" || true
    fi
    
    log_success "Health alert sent"
}

# Main health check function
perform_health_check() {
    local version="$1"
    local network="$2"
    
    log_info "Starting comprehensive health check for v$version..."
    echo
    
    # Perform all health checks
    check_github_release "$version"
    check_artifact_integrity "$version"
    check_network_connectivity "$network"
    check_contract_deployment "$version" "$network"
    check_documentation_availability "$version"
    check_dependency_versions
    check_performance_metrics "$version"
    
    echo
    log_info "Health check completed"
    echo
    
    # Generate report
    generate_health_report "$version"
    
    # Determine overall health
    local overall_status
    if [[ $HEALTH_ERRORS -eq 0 ]]; then
        overall_status="PASS"
        log_success "Release health check PASSED 🎉"
    else
        overall_status="FAIL"
        log_error "Release health check FAILED with $HEALTH_ERRORS error(s)"
    fi
    
    # Send alert if needed
    send_health_alert "$version" "$overall_status"
    
    return $([[ $HEALTH_ERRORS -eq 0 ]] && echo 0 || echo 1)
}

# Help function
show_help() {
    cat << EOF
Release health check script for Uzima-Contracts

Usage:
    $0 VERSION [OPTIONS]

Arguments:
    VERSION        Version to check (e.g., 1.2.0)

Options:
    --network NETWORK     Network to check contracts on (default: testnet)
    --timeout SECONDS    Health check timeout (default: 300)
    --alert-on-failure   Send alerts on health check failures
    --help               Show this help message

Environment Variables:
    NETWORK              Network to check (testnet, futurenet, mainnet, local)
    HEALTH_CHECK_TIMEOUT Timeout for health checks in seconds
    ALERT_ON_FAILURE     Set to 'true' to send alerts on failures
    SLACK_WEBHOOK_URL    Slack webhook URL for alerts

Examples:
    $0 1.2.0
    $0 1.2.0 --network mainnet
    $0 1.2.0 --alert-on-failure

The script checks:
- GitHub release availability and assets
- Artifact integrity and checksums
- Contract deployment health on specified network
- Network connectivity
- Documentation availability
- Dependency versions
- Performance metrics (WASM sizes)

Exit codes:
- 0: Health check passed
- 1: Health check failed

EOF
}

# Main execution
main() {
    # Parse command line arguments
    while [[ $# -gt 0 ]]; do
        case $1 in
            --network)
                NETWORK="$2"
                shift 2
                ;;
            --timeout)
                HEALTH_CHECK_TIMEOUT="$2"
                shift 2
                ;;
            --alert-on-failure)
                ALERT_ON_FAILURE="true"
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
    
    # Perform health check
    perform_health_check "$VERSION" "$NETWORK"
}

# Run main function
main "$@"
