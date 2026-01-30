#!/bin/bash

# =============================================================================
# DID Management Script
# =============================================================================
# This script provides commands for managing DIDs and verifiable credentials
# on the Uzima identity registry.
#
# Usage: ./scripts/did_management.sh <command> [options]
#
# Commands:
#   create-did        Create a new DID for an address
#   resolve-did       Resolve a DID document
#   add-key           Add a verification method
#   rotate-key        Rotate an existing key
#   add-service       Add a service endpoint
#   issue-credential  Issue a verifiable credential
#   verify-credential Verify a credential's status
#   revoke-credential Revoke a credential
#   add-guardian      Add a recovery guardian
#   initiate-recovery Initiate identity recovery
#   add-verifier      Add a credential issuer (verifier)
#
# Example:
#   ./scripts/did_management.sh create-did --subject GA... --public-key 0x...
#   ./scripts/did_management.sh issue-credential --subject GA... --type MedicalLicense
# =============================================================================

set -e

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
# shellcheck disable=SC2034  # YELLOW is used in print functions
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m'

# Default values
NETWORK=${NETWORK:-testnet}
IDENTITY=${IDENTITY:-default}
CONTRACT_ID=${DID_CONTRACT_ID:-}

# Load deployment info if available
DEPLOY_INFO_FILE="deployments/${NETWORK}_identity_registry.json"
if [[ -f "$DEPLOY_INFO_FILE" && -z "$CONTRACT_ID" ]]; then
    CONTRACT_ID=$(jq -r '.contract_id' "$DEPLOY_INFO_FILE")
fi

print_header() {
    echo ""
    echo -e "${CYAN}=============================================="
    echo "  $1"
    echo -e "==============================================${NC}"
    echo ""
}

print_step() {
    echo -e "${BLUE}==>${NC} $1"
}

print_success() {
    echo -e "${GREEN}✓${NC} $1"
}

print_error() {
    echo -e "${RED}✗${NC} $1"
}

show_help() {
    echo "DID Management Script for Uzima Healthcare Platform"
    echo ""
    echo "Usage: $0 <command> [options]"
    echo ""
    echo "Commands:"
    echo "  create-did        Create a new DID document"
    echo "  resolve-did       Resolve a DID by address"
    echo "  update-did        Update DID services and also-known-as"
    echo "  deactivate-did    Deactivate a DID"
    echo "  add-key           Add a new verification method"
    echo "  rotate-key        Rotate an existing key"
    echo "  revoke-key        Revoke a verification method"
    echo "  add-service       Add a service endpoint"
    echo "  remove-service    Remove a service endpoint"
    echo "  issue-credential  Issue a verifiable credential"
    echo "  verify-credential Verify a credential status"
    echo "  revoke-credential Revoke a credential"
    echo "  get-credentials   Get all credentials for a subject"
    echo "  add-guardian      Add a recovery guardian"
    echo "  remove-guardian   Remove a recovery guardian"
    echo "  set-threshold     Set recovery threshold"
    echo "  initiate-recovery Initiate identity recovery"
    echo "  approve-recovery  Approve a recovery request"
    echo "  execute-recovery  Execute an approved recovery"
    echo "  cancel-recovery   Cancel an active recovery"
    echo "  add-verifier      Add a credential issuer"
    echo "  remove-verifier   Remove a credential issuer"
    echo "  help              Show this help message"
    echo ""
    echo "Environment Variables:"
    echo "  NETWORK          Network to use (default: testnet)"
    echo "  IDENTITY         Stellar identity to use (default: default)"
    echo "  DID_CONTRACT_ID  Contract ID (auto-detected from deployments/)"
    echo ""
    echo "Examples:"
    echo "  $0 create-did --subject GA... --public-key 0101..."
    echo "  $0 issue-credential --subject GA... --type MedicalLicense --uri ipfs://..."
    echo "  $0 verify-credential --id 0x..."
    echo ""
}

check_contract() {
    if [[ -z "$CONTRACT_ID" ]]; then
        print_error "Contract ID not set. Set DID_CONTRACT_ID or deploy the contract first."
        exit 1
    fi
}

# Create a new DID
cmd_create_did() {
    check_contract
    print_header "Create DID Document"

    SUBJECT=""
    PUBLIC_KEY=""

    while [[ $# -gt 0 ]]; do
        case $1 in
            --subject)
                SUBJECT="$2"
                shift 2
                ;;
            --public-key)
                PUBLIC_KEY="$2"
                shift 2
                ;;
            *)
                shift
                ;;
        esac
    done

    if [[ -z "$SUBJECT" || -z "$PUBLIC_KEY" ]]; then
        print_error "Usage: create-did --subject <address> --public-key <hex>"
        exit 1
    fi

    print_step "Creating DID for: $SUBJECT"

    stellar contract invoke \
        --id $CONTRACT_ID \
        --source $IDENTITY \
        --network $NETWORK \
        -- \
        create_did \
        --subject $SUBJECT \
        --primary_public_key $PUBLIC_KEY \
        --services "[]"

    print_success "DID created successfully"
}

# Resolve a DID
cmd_resolve_did() {
    check_contract
    print_header "Resolve DID Document"

    SUBJECT=""

    while [[ $# -gt 0 ]]; do
        case $1 in
            --subject)
                SUBJECT="$2"
                shift 2
                ;;
            *)
                shift
                ;;
        esac
    done

    if [[ -z "$SUBJECT" ]]; then
        print_error "Usage: resolve-did --subject <address>"
        exit 1
    fi

    print_step "Resolving DID for: $SUBJECT"

    stellar contract invoke \
        --id $CONTRACT_ID \
        --source $IDENTITY \
        --network $NETWORK \
        -- \
        resolve_did \
        --subject $SUBJECT
}

# Issue a credential
cmd_issue_credential() {
    check_contract
    print_header "Issue Verifiable Credential"

    SUBJECT=""
    CRED_TYPE="MedicalLicense"
    CRED_HASH=""
    CRED_URI=""
    EXPIRATION="0"

    while [[ $# -gt 0 ]]; do
        case $1 in
            --subject)
                SUBJECT="$2"
                shift 2
                ;;
            --type)
                CRED_TYPE="$2"
                shift 2
                ;;
            --hash)
                CRED_HASH="$2"
                shift 2
                ;;
            --uri)
                CRED_URI="$2"
                shift 2
                ;;
            --expiration)
                EXPIRATION="$2"
                shift 2
                ;;
            *)
                shift
                ;;
        esac
    done

    if [[ -z "$SUBJECT" || -z "$CRED_HASH" || -z "$CRED_URI" ]]; then
        print_error "Usage: issue-credential --subject <address> --type <type> --hash <hex> --uri <uri>"
        echo "Credential types: MedicalLicense, SpecialistCertification, HospitalAffiliation,"
        echo "                  ResearchAuthorization, PatientConsent, EmergencyAccess, DataAccessPermission"
        exit 1
    fi

    ISSUER=$(stellar keys address $IDENTITY)
    print_step "Issuing $CRED_TYPE credential"
    print_step "  Subject: $SUBJECT"
    print_step "  Issuer: $ISSUER"
    print_step "  URI: $CRED_URI"

    stellar contract invoke \
        --id $CONTRACT_ID \
        --source $IDENTITY \
        --network $NETWORK \
        -- \
        issue_credential \
        --issuer $ISSUER \
        --subject $SUBJECT \
        --credential_type $CRED_TYPE \
        --credential_hash $CRED_HASH \
        --credential_uri "$CRED_URI" \
        --expiration_date $EXPIRATION

    print_success "Credential issued successfully"
}

# Verify a credential
cmd_verify_credential() {
    check_contract
    print_header "Verify Credential"

    CRED_ID=""

    while [[ $# -gt 0 ]]; do
        case $1 in
            --id)
                CRED_ID="$2"
                shift 2
                ;;
            *)
                shift
                ;;
        esac
    done

    if [[ -z "$CRED_ID" ]]; then
        print_error "Usage: verify-credential --id <credential_id>"
        exit 1
    fi

    print_step "Verifying credential: $CRED_ID"

    stellar contract invoke \
        --id $CONTRACT_ID \
        --source $IDENTITY \
        --network $NETWORK \
        -- \
        verify_credential \
        --credential_id $CRED_ID
}

# Add a verifier
cmd_add_verifier() {
    check_contract
    print_header "Add Verifier (Credential Issuer)"

    VERIFIER=""

    while [[ $# -gt 0 ]]; do
        case $1 in
            --address)
                VERIFIER="$2"
                shift 2
                ;;
            *)
                shift
                ;;
        esac
    done

    if [[ -z "$VERIFIER" ]]; then
        print_error "Usage: add-verifier --address <verifier_address>"
        exit 1
    fi

    print_step "Adding verifier: $VERIFIER"

    stellar contract invoke \
        --id $CONTRACT_ID \
        --source $IDENTITY \
        --network $NETWORK \
        -- \
        add_verifier \
        --verifier $VERIFIER

    print_success "Verifier added successfully"
}

# Add a recovery guardian
cmd_add_guardian() {
    check_contract
    print_header "Add Recovery Guardian"

    SUBJECT=""
    GUARDIAN=""
    WEIGHT="1"

    while [[ $# -gt 0 ]]; do
        case $1 in
            --subject)
                SUBJECT="$2"
                shift 2
                ;;
            --guardian)
                GUARDIAN="$2"
                shift 2
                ;;
            --weight)
                WEIGHT="$2"
                shift 2
                ;;
            *)
                shift
                ;;
        esac
    done

    if [[ -z "$SUBJECT" || -z "$GUARDIAN" ]]; then
        print_error "Usage: add-guardian --subject <address> --guardian <address> [--weight <n>]"
        exit 1
    fi

    print_step "Adding guardian $GUARDIAN for subject $SUBJECT"

    stellar contract invoke \
        --id $CONTRACT_ID \
        --source $IDENTITY \
        --network $NETWORK \
        -- \
        add_recovery_guardian \
        --subject $SUBJECT \
        --guardian $GUARDIAN \
        --weight $WEIGHT

    print_success "Guardian added successfully"
}

# Rotate a key
cmd_rotate_key() {
    check_contract
    print_header "Rotate Verification Key"

    SUBJECT=""
    METHOD_ID=""
    NEW_KEY=""

    while [[ $# -gt 0 ]]; do
        case $1 in
            --subject)
                SUBJECT="$2"
                shift 2
                ;;
            --method-id)
                METHOD_ID="$2"
                shift 2
                ;;
            --new-key)
                NEW_KEY="$2"
                shift 2
                ;;
            *)
                shift
                ;;
        esac
    done

    if [[ -z "$SUBJECT" || -z "$METHOD_ID" || -z "$NEW_KEY" ]]; then
        print_error "Usage: rotate-key --subject <address> --method-id <id> --new-key <hex>"
        exit 1
    fi

    print_step "Rotating key $METHOD_ID for subject $SUBJECT"

    stellar contract invoke \
        --id $CONTRACT_ID \
        --source $IDENTITY \
        --network $NETWORK \
        -- \
        rotate_key \
        --subject $SUBJECT \
        --method_id "$METHOD_ID" \
        --new_public_key $NEW_KEY

    print_success "Key rotated successfully"
}

# Main command dispatcher
COMMAND=${1:-help}
shift || true

case $COMMAND in
    create-did)
        cmd_create_did "$@"
        ;;
    resolve-did)
        cmd_resolve_did "$@"
        ;;
    issue-credential)
        cmd_issue_credential "$@"
        ;;
    verify-credential)
        cmd_verify_credential "$@"
        ;;
    add-verifier)
        cmd_add_verifier "$@"
        ;;
    add-guardian)
        cmd_add_guardian "$@"
        ;;
    rotate-key)
        cmd_rotate_key "$@"
        ;;
    help|--help|-h)
        show_help
        ;;
    *)
        print_error "Unknown command: $COMMAND"
        echo "Use '$0 help' for usage information"
        exit 1
        ;;
esac
