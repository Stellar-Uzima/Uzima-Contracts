#!/usr/bin/env bash
#
# generate_contract.sh — Generate a new Soroban contract scaffold with
# category-specific boilerplate for the Uzima healthcare platform.
#
# Usage:
#   ./scripts/generate_contract.sh <contract_name> <category>
#
# Categories:
#   medical_records  — Health record storage, retrieval, and lifecycle
#   payments         — Escrow, settlement, and payment routing
#   auth             — Authentication, MFA, and access control
#   governance       — Proposals, voting, and upgrade management
#   cross_chain      — Cross-chain bridge and identity sync
#
# Example:
#   ./scripts/generate_contract.sh lab_results medical_records
#
# This will create:
#   contracts/<contract_name>/
#     Cargo.toml
#     src/lib.rs
#     src/test.rs

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
CONTRACTS_DIR="$PROJECT_ROOT/contracts"

if [[ $# -ne 2 ]]; then
  echo "Usage: $0 <contract_name> <category>"
  echo ""
  echo "Categories: medical_records, payments, auth, governance, cross_chain"
  echo "Example:    $0 lab_results medical_records"
  exit 1
fi

CONTRACT_NAME="$1"
CATEGORY="$2"

if [[ ! "$CONTRACT_NAME" =~ ^[a-z][a-z0-9_]*$ ]]; then
  echo "Error: Contract name must be snake_case."
  exit 1
fi

VALID_CATEGORIES="medical_records payments auth governance cross_chain"
if ! echo "$VALID_CATEGORIES" | grep -qw "$CATEGORY"; then
  echo "Error: Invalid category '$CATEGORY'."
  exit 1
fi

TARGET_DIR="$CONTRACTS_DIR/$CONTRACT_NAME"
if [[ -d "$TARGET_DIR" ]]; then
  echo "Error: Directory already exists: $TARGET_DIR"
  exit 1
fi

PASCAL_NAME=$(echo "$CONTRACT_NAME" | awk -F_ '{for(i=1;i<=NF;i++) $i=toupper(substr($i,1,1)) substr($i,2)}1' OFS="")
KEBAB_NAME=$(echo "$CONTRACT_NAME" | tr '_' '-')

echo "Generating contract: $CONTRACT_NAME ($CATEGORY)"
echo "  PascalCase: $PASCAL_NAME"
echo "  Kebab-case: $KEBAB_NAME"

mkdir -p "$TARGET_DIR/src"

# ── Generate Cargo.toml ─────────────────────────────────────────────────────

cat > "$TARGET_DIR/Cargo.toml" << EOF
[package]
name = "$KEBAB_NAME"
version.workspace = true
edition.workspace = true
authors.workspace = true
license.workspace = true

[lib]
crate-type = ["cdylib"]

[dependencies]
soroban-sdk = { workspace = true }

[dev-dependencies]
soroban-sdk = { workspace = true, features = ["testutils"] }

[features]
default = []
testutils = ["soroban-sdk/testutils"]
EOF

# ── Generate src/lib.rs using awk for reliable substitution ──────────────────

awk -v pascal="$PASCAL_NAME" -v catdesc="$CATEGORY" '
/contract_desc_placeholder/ { print "//! " catdesc; next }
/contract_name_placeholder/ { gsub(/contract_name_placeholder/, pascal); print; next }
{ print }
' > /dev/null  # Just validate awk works

# Build the lib.rs using cat with proper escaping
cat > "$TARGET_DIR/src/lib.rs" << 'RUSTEOF'
#![no_std]

use soroban_sdk::{contract, contractimpl, symbol_short, Address, Env, String};
use soroban_sdk::contracterror;
use soroban_sdk::contracttype;

// ---------------------------------------------------------------------------
// Errors
// ---------------------------------------------------------------------------

#[contracterror]
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
#[repr(u32)]
pub enum Error {
    NotInitialized = 0,
    AlreadyInitialized = 1,
    Unauthorized = 2,
    InputTooLong = 3,
RUSTEOF

case "$CATEGORY" in
  medical_records)
    cat >> "$TARGET_DIR/src/lib.rs" << 'EOF'
    RecordNotFound = 10,
    RecordAlreadyExists = 11,
    RecordNotOwned = 12,
EOF
    CATEGORY_STORAGE='    Record { record_id: u64 },
    RecordOwner { record_id: u64 },
    RecordCount,'
    ;;
  payments)
    cat >> "$TARGET_DIR/src/lib.rs" << 'EOF'
    PaymentNotFound = 10,
    InsufficientFunds = 11,
    PaymentAlreadySettled = 12,
    EscrowNotFunded = 13,
EOF
    CATEGORY_STORAGE='    Payment { payment_id: u64 },
    Escrow { escrow_id: u64 },
    PaymentCount,'
    ;;
  auth)
    cat >> "$TARGET_DIR/src/lib.rs" << 'EOF'
    InvalidCredentials = 10,
    SessionExpired = 11,
    MfaChallengeExpired = 12,
    InsufficientPermissions = 13,
EOF
    CATEGORY_STORAGE='    Role { address: Address },
    Session { session_id: u64 },
    MfaChallenge { address: Address },'
    ;;
  governance)
    cat >> "$TARGET_DIR/src/lib.rs" << 'EOF'
    ProposalNotFound = 10,
    ProposalAlreadyActive = 11,
    VotingClosed = 12,
    QuorumNotReached = 13,
EOF
    CATEGORY_STORAGE='    Proposal { proposal_id: u64 },
    Vote { proposal_id: u64, voter: Address },
    ProposalCount,'
    ;;
  cross_chain)
    cat >> "$TARGET_DIR/src/lib.rs" << 'EOF'
    BridgeRequestNotFound = 10,
    ChainNotRegistered = 11,
    SyncInProgress = 12,
    BridgeRequestExpired = 13,
EOF
    CATEGORY_STORAGE='    BridgeRequest { request_id: u64 },
    ChainMapping { chain_id: u64 },
    SyncStatus { chain_id: u64 },'
    ;;
esac

cat >> "$TARGET_DIR/src/lib.rs" << 'EOF'
}

impl core::fmt::Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        match self {
            Error::NotInitialized => write!(f, "not initialized"),
            Error::AlreadyInitialized => write!(f, "already initialized"),
            Error::Unauthorized => write!(f, "unauthorized"),
            Error::InputTooLong => write!(f, "input too long"),
            _ => write!(f, "contract error"),
        }
    }
}

// ---------------------------------------------------------------------------
// Storage keys
// ---------------------------------------------------------------------------

#[contracttype]
pub enum DataKey {
    Admin,
EOF

# Insert category-specific storage variants
echo "$CATEGORY_STORAGE" >> "$TARGET_DIR/src/lib.rs"

cat >> "$TARGET_DIR/src/lib.rs" << 'EOF'
}

// ---------------------------------------------------------------------------
// Events
// ---------------------------------------------------------------------------

fn emit_initialized(env: &Env, admin: &Address) {
    env.events()
        .publish((symbol_short!("init"),), (admin.clone(),));
}

fn emit_admin_transferred(env: &Env, old_admin: &Address, new_admin: &Address) {
    env.events().publish(
        (symbol_short!("adm_xfer"),),
        (old_admin.clone(), new_admin.clone()),
    );
}

EOF

# Add category-specific event helpers
case "$CATEGORY" in
  medical_records)
    cat >> "$TARGET_DIR/src/lib.rs" << 'EOF'
fn emit_record_created(env: &Env, caller: &Address, record_id: u64) {
    env.events()
        .publish((symbol_short!("rec_crt"),), (caller.clone(), record_id));
}

fn emit_record_deleted(env: &Env, caller: &Address, record_id: u64) {
    env.events()
        .publish((symbol_short!("rec_del"),), (caller.clone(), record_id));
}
EOF
    ;;
  payments)
    cat >> "$TARGET_DIR/src/lib.rs" << 'EOF'
fn emit_payment_created(env: &Env, caller: &Address, payment_id: u64) {
    env.events()
        .publish((symbol_short!("pay_crt"),), (caller.clone(), payment_id));
}

fn emit_payment_settled(env: &Env, payment_id: u64) {
    env.events()
        .publish((symbol_short!("pay_set"),), (payment_id,));
}
EOF
    ;;
  auth)
    cat >> "$TARGET_DIR/src/lib.rs" << 'EOF'
fn emit_auth_granted(env: &Env, caller: &Address) {
    env.events()
        .publish((symbol_short!("auth_grt"),), (caller.clone(),));
}

fn emit_auth_revoked(env: &Env, caller: &Address) {
    env.events()
        .publish((symbol_short!("auth_rvk"),), (caller.clone(),));
}
EOF
    ;;
  governance)
    cat >> "$TARGET_DIR/src/lib.rs" << 'EOF'
fn emit_proposal_created(env: &Env, proposer: &Address, proposal_id: u64) {
    env.events()
        .publish((symbol_short!("prop_crt"),), (proposer.clone(), proposal_id));
}

fn emit_vote_cast(env: &Env, voter: &Address, proposal_id: u64) {
    env.events()
        .publish((symbol_short!("vote_cst"),), (voter.clone(), proposal_id));
}
EOF
    ;;
  cross_chain)
    cat >> "$TARGET_DIR/src/lib.rs" << 'EOF'
fn emit_bridge_request(env: &Env, caller: &Address, request_id: u64) {
    env.events()
        .publish((symbol_short!("brg_req"),), (caller.clone(), request_id));
}

fn emit_sync_completed(env: &Env, chain_id: u64) {
    env.events()
        .publish((symbol_short!("sync_cmp"),), (chain_id,));
}
EOF
    ;;
esac

# Contract struct and impl
cat >> "$TARGET_DIR/src/lib.rs" << CONTRACTEOF

// ---------------------------------------------------------------------------
// Contract
// ---------------------------------------------------------------------------

#[contract]
pub struct $PASCAL_NAME;

#[contractimpl]
impl $PASCAL_NAME {
    pub fn initialize(env: Env, admin: Address) -> Result<(), Error> {
        if env.storage().instance().has(&DataKey::Admin) {
            return Err(Error::AlreadyInitialized);
        }
        env.storage().instance().set(&DataKey::Admin, &admin);
        emit_initialized(&env, &admin);
        Ok(())
    }

    pub fn transfer_admin(env: Env, new_admin: Address) -> Result<(), Error> {
        let admin = Self::get_admin(&env)?;
        admin.require_auth();

        env.storage().instance().set(&DataKey::Admin, &new_admin);
        emit_admin_transferred(&env, &admin, &new_admin);
        Ok(())
    }

    pub fn get_admin(env: &Env) -> Result<Address, Error> {
        env.storage()
            .instance()
            .get(&DataKey::Admin)
            .ok_or(Error::NotInitialized)
    }
}
CONTRACTEOF

# ── Generate src/test.rs ────────────────────────────────────────────────────

cat > "$TARGET_DIR/src/test.rs" << TESTEOF
use super::*;
use soroban_sdk::{testutils::Address as _, Address, Env};

fn setup() -> (Env, Address, ${PASCAL_NAME}Client<'static>) {
    let env = Env::default();
    env.mock_all_auths();
    let admin = Address::generate(&env);
    let contract_id = env.register_contract(None, ${PASCAL_NAME});
    let client = ${PASCAL_NAME}Client::new(&env, &contract_id);
    client.initialize(&admin);
    (env, admin, client)
}

#[test]
fn test_initialize() {
    let (_, _, client) = setup();
    let admin2 = Address::generate(&client.env);
    assert_eq!(
        client.try_initialize(&admin2),
        Err(Ok(Error::AlreadyInitialized))
    );
}

#[test]
fn test_get_admin() {
    let (env, admin, client) = setup();
    assert_eq!(client.get_admin(), admin);
}

#[test]
fn test_transfer_admin() {
    let (env, admin, client) = setup();
    let new_admin = Address::generate(&env);
    assert!(client.try_transfer_admin(&new_admin).is_ok());
    assert_eq!(client.get_admin(), new_admin);
}
TESTEOF

echo ""
echo "Generated contract at: $TARGET_DIR"
echo "Files: Cargo.toml, src/lib.rs, src/test.rs"
echo "Next: cargo test --package $KEBAB_NAME"
