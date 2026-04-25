#!/bin/bash

# WASM Size Monitoring Script for Stellar Contracts
# Monitors contract WASM sizes against Stellar limits and provides optimization recommendations

set -euo pipefail

# Configuration
MAX_CONTRACT_SIZE=65536      # 64KB Stellar limit
WARNING_THRESHOLD=0.8        # 80% warning threshold
CRITICAL_THRESHOLD=0.95     # 95% critical threshold
TREND_DATA_FILE=".wasm_size_trends.json"
OPTIMIZATION_TIPS_FILE=".wasm_optimization_tips.json"

# Colors for output
RED='\033[0;31m'
YELLOW='\033[1;33m'
GREEN='\033[0;32m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Logging function
log() {
    echo -e "$1"
}

# Initialize trend data file
init_trend_data() {
    if [[ ! -f "$TREND_DATA_FILE" ]]; then
        echo '{"contracts": {}, "last_updated": "'$(date -Iseconds)'"}' > "$TREND_DATA_FILE"
    fi
}

# Initialize optimization tips file
init_optimization_tips() {
    if [[ ! -f "$OPTIMIZATION_TIPS_FILE" ]]; then
        cat > "$OPTIMIZATION_TIPS_FILE" << 'EOF'
{
  "tips": [
    {
      "size_range": "50-60KB",
      "recommendations": [
        "Remove unused dependencies and features",
        "Use cargo-bloat to identify large functions",
        "Consider splitting contract into multiple contracts",
        "Optimize string operations and reduce allocations"
      ]
    },
    {
      "size_range": "40-50KB", 
      "recommendations": [
        "Review error message sizes",
        "Use more efficient data structures",
        "Remove debug code and assertions",
        "Optimize serialization formats"
      ]
    },
    {
      "size_range": "30-40KB",
      "recommendations": [
        "Use feature flags for optional functionality",
        "Review and optimize imports",
        "Consider using no_std when possible",
        "Optimize enum representations"
      ]
    }
  ]
}
EOF
    fi
}

# Get current timestamp
get_timestamp() {
    date -Iseconds
}

# Convert bytes to human readable format
bytes_to_human() {
    local bytes=$1
    if [[ $bytes -ge 65536 ]]; then
        echo "$(($bytes / 1024))KB"
    elif [[ $bytes -ge 1024 ]]; then
        echo "$(($bytes / 1024))KB"
    else
        echo "${bytes}B"
    fi
}

# Calculate percentage
calculate_percentage() {
    local size=$1
    local max=$2
    echo "scale=1; $size * 100 / $max" | bc -l
}

# Get optimization recommendations based on size
get_optimization_recommendations() {
    local size=$1
    local size_kb=$((size / 1024))
    
    if [[ $size_kb -ge 50 ]]; then
        echo "50-60KB"
    elif [[ $size_kb -ge 40 ]]; then
        echo "40-50KB"
    elif [[ $size_kb -ge 30 ]]; then
        echo "30-40KB"
    else
        echo ""
    fi
}

# Update trend data
update_trend_data() {
    local contract_name=$1
    local size=$2
    local timestamp=$(get_timestamp)
    
    # Read current data
    local temp_file=$(mktemp)
    jq --arg contract "$contract_name" --arg size "$size" --arg timestamp "$timestamp" '
        .contracts[$contract] = (.contracts[$contract] // {} | 
            .size_history += [{"size": ($size | tonumber), "timestamp": $timestamp}] |
            .current_size = ($size | tonumber) |
            .last_updated = $timestamp
        ) |
        .last_updated = $timestamp
    ' "$TREND_DATA_FILE" > "$temp_file"
    mv "$temp_file" "$TREND_DATA_FILE"
}

# Analyze size trend
analyze_trend() {
    local contract_name=$1
    local current_size=$2
    
    # Get last 5 measurements
    local trend_data=$(jq -r --arg contract "$contract_name" '
        .contracts[$contract].size_history[-5:] // []
    ' "$TREND_DATA_FILE")
    
    if [[ "$trend_data" != "[]" ]]; then
        local count=$(echo "$trend_data" | jq length)
        if [[ $count -gt 1 ]]; then
            local first_size=$(echo "$trend_data" | jq -r '.[0].size')
            local last_size=$(echo "$trend_data" | jq -r '.[-1].size')
            local change=$((last_size - first_size))
            local change_percent=$(echo "scale=1; $change * 100 / $first_size" | bc -l)
            
            if [[ $change -gt 0 ]]; then
                log "${YELLOW}  Trend: +${change_percent}% over last $count builds${NC}"
            elif [[ $change -lt 0 ]]; then
                log "${GREEN}  Trend: ${change_percent}% over last $count builds${NC}"
            else
                log "${BLUE}  Trend: Stable over last $count builds${NC}"
            fi
        fi
    fi
}

# Check single contract
check_contract() {
    local wasm_file=$1
    local contract_name=$2
    
    if [[ ! -f "$wasm_file" ]]; then
        log "${RED}  Error: WASM file not found: $wasm_file${NC}"
        return 1
    fi
    
    local size=$(wc -c < "$wasm_file")
    local percentage=$(calculate_percentage "$size" "$MAX_CONTRACT_SIZE")
    local size_human=$(bytes_to_human "$size")
    
    # Update trend data
    update_trend_data "$contract_name" "$size"
    
    # Display results
    printf "%-25s %8s %6s%% " "$contract_name" "$size_human" "$percentage"
    
    # Status indicators
    if [[ $size -gt $((MAX_CONTRACT_SIZE * CRITICAL_THRESHOLD / 100)) ]]; then
        log "${RED}CRITICAL${NC}"
        log "${RED}  Risk: Contract will likely fail deployment!${NC}"
    elif [[ $size -gt $((MAX_CONTRACT_SIZE * WARNING_THRESHOLD / 100)) ]]; then
        log "${YELLOW}WARNING${NC}"
        log "${YELLOW}  Risk: Approaching deployment limit${NC}"
    else
        log "${GREEN}OK${NC}"
    fi
    
    # Show trend
    analyze_trend "$contract_name" "$size"
    
    # Show optimization recommendations if needed
    local recommendation_range=$(get_optimization_recommendations "$size")
    if [[ -n "$recommendation_range" ]]; then
        local tips=$(jq -r --arg range "$recommendation_range" '
            .tips[] | select(.size_range == $range) | .recommendations[]
        ' "$OPTIMIZATION_TIPS_FILE")
        log "${BLUE}  Optimization recommendations:${NC}"
        while IFS= read -r tip; do
            log "${BLUE}    - $tip${NC}"
        done <<< "$tips"
    fi
}

# Generate summary report
generate_summary() {
    log "\n${BLUE}=== WASM Size Summary ===${NC}"
    
    local total_contracts=0
    local total_size=0
    local warning_count=0
    local critical_count=0
    
    for wasm_file in dist/*.wasm; do
        if [[ -f "$wasm_file" ]]; then
            local size=$(wc -c < "$wasm_file")
            local percentage=$(calculate_percentage "$size" "$MAX_CONTRACT_SIZE")
            total_contracts=$((total_contracts + 1))
            total_size=$((total_size + size))
            
            if [[ $size -gt $((MAX_CONTRACT_SIZE * CRITICAL_THRESHOLD / 100)) ]]; then
                critical_count=$((critical_count + 1))
            elif [[ $size -gt $((MAX_CONTRACT_SIZE * WARNING_THRESHOLD / 100)) ]]; then
                warning_count=$((warning_count + 1))
            fi
        fi
    done
    
    log "Total contracts: $total_contracts"
    log "Total size: $(bytes_to_human $total_size)"
    log "Average size: $(bytes_to_human $((total_size / total_contracts)))"
    
    if [[ $critical_count -gt 0 ]]; then
        log "${RED}Critical contracts: $critical_count${NC}"
    fi
    if [[ $warning_count -gt 0 ]]; then
        log "${YELLOW}Warning contracts: $warning_count${NC}"
    fi
    if [[ $critical_count -eq 0 && $warning_count -eq 0 ]]; then
        log "${GREEN}All contracts within safe limits${NC}"
    fi
}

# Export trend data as JSON
export_trend_data() {
    log "\n${BLUE}=== Trend Data Export ===${NC}"
    jq '.' "$TREND_DATA_FILE"
}

# Check dependencies
check_dependencies() {
    local missing_deps=()
    
    if ! command -v jq >/dev/null 2>&1; then
        missing_deps+=("jq")
    fi
    
    if ! command -v bc >/dev/null 2>&1; then
        missing_deps+=("bc")
    fi
    
    if [[ ${#missing_deps[@]} -gt 0 ]]; then
        log "${RED}Error: Missing required dependencies: ${missing_deps[*]}${NC}"
        log "${YELLOW}Install with: sudo apt-get install jq bc (Ubuntu/Debian)${NC}"
        log "${YELLOW}Or with: brew install jq bc (macOS)${NC}"
        exit 1
    fi
}

# Main function
main() {
    log "${BLUE}=== WASM Size Monitor ===${NC}"
    log "Stellar contract size limit: $(bytes_to_human $MAX_CONTRACT_SIZE)"
    log "Warning threshold: $((WARNING_THRESHOLD * 100))%"
    log "Critical threshold: $((CRITICAL_THRESHOLD * 100))%"
    
    # Check dependencies first
    check_dependencies
    
    # Initialize data files
    init_trend_data
    init_optimization_tips
    
    # Check if dist directory exists
    if [[ ! -d "dist" ]]; then
        log "${YELLOW}Warning: dist/ directory not found. Creating it...${NC}"
        mkdir -p dist
        log "${YELLOW}No WASM files to monitor. Run 'make dist' first.${NC}"
        exit 0
    fi
    
    # Check for WASM files
    local wasm_files=(dist/*.wasm)
    if [[ ${#wasm_files[@]} -eq 0 || ! -f "${wasm_files[0]}" ]]; then
        log "${YELLOW}Warning: No WASM files found in dist/. Run 'make dist' first.${NC}"
        exit 0
    fi
    
    log "\n${BLUE}=== Contract Size Analysis ===${NC}"
    printf "%-25s %8s %6s %s\n" "Contract" "Size" "Usage" "Status"
    printf "%-25s %8s %6s %s\n" "--------" "----" "-----" "------"
    
    # Check each contract
    for wasm_file in dist/*.wasm; do
        if [[ -f "$wasm_file" ]]; then
            local contract_name=$(basename "$wasm_file" .wasm)
            check_contract "$wasm_file" "$contract_name"
        fi
    done
    
    # Generate summary
    generate_summary
    
    # Export trend data if requested
    if [[ "${1:-}" == "--export-trends" ]]; then
        export_trend_data
    fi
    
    log "\n${BLUE}=== Recommendations ===${NC}"
    log "1. Run 'cargo install cargo-bloat' to analyze large functions"
    log "2. Use 'cargo build --target wasm32-unknown-unknown --release' for optimized builds"
    log "3. Consider splitting large contracts into smaller ones"
    log "4. Review dependencies and remove unused ones"
    
    # Exit with error code if critical issues found
    local critical_count=0
    for wasm_file in dist/*.wasm; do
        if [[ -f "$wasm_file" ]]; then
            local size=$(wc -c < "$wasm_file")
            if [[ $size -gt $((MAX_CONTRACT_SIZE * CRITICAL_THRESHOLD / 100)) ]]; then
                critical_count=$((critical_count + 1))
            fi
        fi
    done
    
    if [[ $critical_count -gt 0 ]]; then
        log "\n${RED}Critical: $critical_count contracts exceed safe limits${NC}"
        exit 1
    else
        log "\n${GREEN}All contracts pass size checks${NC}"
        exit 0
    fi
}

# Help function
show_help() {
    cat << EOF
Usage: $0 [OPTIONS]

WASM Size Monitoring Script for Stellar Contracts

Options:
  --export-trends    Export trend data as JSON
  --help            Show this help message

Examples:
  $0                    # Check all contracts
  $0 --export-trends    # Check contracts and export trend data

Requirements:
  - jq (JSON processor)
  - bc (calculator)
  - dist/ directory with WASM files

Stellar Limits:
  - Max contract size: 64KB
  - Max transaction size: 64KB
  - Warning at 80% (51.2KB)
  - Critical at 95% (61.4KB)
EOF
}

# Parse arguments
case "${1:-}" in
    --help|-h)
        show_help
        exit 0
        ;;
    *)
        main "$@"
        ;;
esac
