#!/bin/bash

# rollback_deployment.sh - Rollback a contract deployment to a previous version
# Usage: ./scripts/rollback_deployment.sh <contract_name> <network> [backup_file]

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

print_status() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

print_step() {
    echo -e "${BLUE}[STEP]${NC} $1"
}

# Check arguments
if [ $# -lt 2 ]; then
    print_error "Usage: $0 <contract_name> <network> [backup_file]"
    print_error "Example: $0 medical_records testnet"
    print_error "Example: $0 medical_records testnet deployments/testnet_medical_records_backup_20240101_120000.json"
    exit 1
fi

CONTRACT_NAME="$1"
NETWORK="$2"
BACKUP_FILE="${3:-}"

DEPLOYMENTS_DIR="deployments"
DEPLOYMENT_FILE="$DEPLOYMENTS_DIR/${NETWORK}_${CONTRACT_NAME}.json"

# Function to list available backups
list_backups() {
    print_step "Available backups for $CONTRACT_NAME on $NETWORK:"
    local backups=($(ls -t "$DEPLOYMENTS_DIR/${NETWORK}_${CONTRACT_NAME}_backup_"*.json 2>/dev/null))
    
    if [ ${#backups[@]} -eq 0 ]; then
        print_warning "No backups found"
        return 1
    fi
    
    local i=1
    for backup in "${backups[@]}"; do
        local timestamp=$(jq -r '.backed_up_at' "$backup" 2>/dev/null || echo "unknown")
        local contract_id=$(jq -r '.contract_id' "$backup" 2>/dev/null || echo "unknown")
        echo "  $i. $backup"
        echo "     Contract ID: $contract_id"
        echo "     Backed up at: $timestamp"
        i=$((i + 1))
    done
}

# Function to select backup interactively
select_backup() {
    local backups=($(ls -t "$DEPLOYMENTS_DIR/${NETWORK}_${CONTRACT_NAME}_backup_"*.json 2>/dev/null))
    
    if [ ${#backups[@]} -eq 0 ]; then
        print_error "No backups available"
        return 1
    fi
    
    if [ ${#backups[@]} -eq 1 ]; then
        echo "${backups[0]}"
        return 0
    fi
    
    list_backups
    echo ""
    read -p "Select backup number (1-${#backups[@]}): " selection
    
    if [ "$selection" -ge 1 ] && [ "$selection" -le ${#backups[@]} ]; then
        echo "${backups[$((selection - 1))]}"
    else
        print_error "Invalid selection"
        return 1
    fi
}

# Main rollback function
main() {
    print_status "Rolling back $CONTRACT_NAME on $NETWORK network"
    
    # Get backup file
    if [ -z "$BACKUP_FILE" ]; then
        print_step "No backup file specified, selecting from available backups..."
        BACKUP_FILE=$(select_backup)
        if [ -z "$BACKUP_FILE" ]; then
            exit 1
        fi
    fi
    
    if [ ! -f "$BACKUP_FILE" ]; then
        print_error "Backup file not found: $BACKUP_FILE"
        exit 1
    fi
    
    # Validate backup file
    local contract_id=$(jq -r '.contract_id' "$BACKUP_FILE" 2>/dev/null)
    if [ -z "$contract_id" ] || [ "$contract_id" = "null" ]; then
        print_error "Invalid backup file format"
        exit 1
    fi
    
    local backed_up_at=$(jq -r '.backed_up_at' "$BACKUP_FILE" 2>/dev/null)
    
    print_step "Backup details:"
    echo "  Contract ID: $contract_id"
    echo "  Backed up at: $backed_up_at"
    echo "  Backup file: $BACKUP_FILE"
    
    # Confirm rollback
    print_warning "⚠️  This will restore the deployment to the backup version."
    read -p "Continue with rollback? (yes/no): " confirm
    
    if [ "$confirm" != "yes" ]; then
        print_status "Rollback cancelled"
        exit 0
    fi
    
    # Verify the backup contract still exists
    print_step "Verifying backup contract exists on network..."
    if ! soroban contract invoke \
        --id "$contract_id" \
        --network "$NETWORK" \
        -- --help &> /dev/null; then
        print_warning "Could not verify contract (this may be expected)"
    else
        print_status "Contract verified"
    fi
    
    # Restore deployment file
    print_step "Restoring deployment file..."
    jq --arg id "$contract_id" \
       --arg name "$CONTRACT_NAME" \
       --arg network "$NETWORK" \
       --arg timestamp "$(date -u +%Y-%m-%dT%H:%M:%SZ)" \
       --arg backup_file "$BACKUP_FILE" \
       '{
         contract_name: $name,
         contract_id: $id,
         network: $network,
         deployed_at: $timestamp,
         rolled_back: true,
         rollback_from: $backup_file,
         original_backup_timestamp: .backed_up_at
       }' "$BACKUP_FILE" > "$DEPLOYMENT_FILE"
    
    print_status "Rollback complete!"
    print_status "Deployment file updated: $DEPLOYMENT_FILE"
    print_status "Contract ID: $contract_id"
    
    # Create rollback log entry
    local rollback_log="$DEPLOYMENTS_DIR/rollback_log.json"
    if [ ! -f "$rollback_log" ]; then
        echo "[]" > "$rollback_log"
    fi
    
    jq --arg name "$CONTRACT_NAME" \
       --arg network "$NETWORK" \
       --arg id "$contract_id" \
       --arg backup "$BACKUP_FILE" \
       --arg timestamp "$(date -u +%Y-%m-%dT%H:%M:%SZ)" \
       '. += [{
         contract_name: $name,
         network: $network,
         contract_id: $id,
         backup_file: $backup,
         rolled_back_at: $timestamp
       }]' "$rollback_log" > "${rollback_log}.tmp" && mv "${rollback_log}.tmp" "$rollback_log"
    
    print_status "Rollback logged to: $rollback_log"
}

# Run rollback
main "$@"

