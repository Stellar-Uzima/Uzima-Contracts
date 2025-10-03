# Treasury Controller - Multi-Sig Compatible

A comprehensive treasury management system with multisig approval workflows and Gnosis Safe compatibility for the Uzima ecosystem.

## Overview

The Treasury Controller provides a secure, auditable way to manage protocol funds with the following key features:

- **Multi-signature approvals** with configurable thresholds
- **Timelock delays** for enhanced security
- **Gnosis Safe compatibility** for seamless integration with existing multisig wallets
- **Emergency halt mechanisms** for crisis management
- **Comprehensive audit trails** for all treasury operations
- **Proposal-based governance** for transparent decision making

## Features

### 🔒 Security Features

- **Configurable Multisig**: Support for M-of-N signature schemes
- **Timelock Protection**: Mandatory delays between approval and execution
- **Emergency Halt**: Immediate suspension of all operations during emergencies
- **Access Control**: Role-based permissions for different operations
- **Withdrawal Limits**: Maximum amounts per transaction for risk management

### 📋 Proposal System

- **Proposal Types**: Withdrawals, configuration changes, emergency actions
- **Approval Workflow**: Transparent voting process with approval tracking
- **Rejection Mechanism**: Democratic rejection of unwanted proposals
- **Expiry Management**: Automatic expiration of stale proposals
- **Metadata Support**: Rich context and documentation for each proposal

### 🔗 Gnosis Safe Integration

- **Compatible Interface**: Direct integration with Gnosis Safe wallets
- **Transaction Translation**: Automatic conversion between formats
- **Signature Verification**: Support for Gnosis Safe signature schemes
- **Owner Management**: Seamless signer synchronization

### 📊 Audit & Compliance

- **Complete Audit Trail**: Every action is recorded and timestamped
- **Withdrawal Records**: Detailed history of all fund movements
- **Event Logging**: Comprehensive event emission for monitoring
- **Purpose Tracking**: Required justification for all treasury operations

## Contract Interface

### Initialization

```rust
pub fn initialize(
    env: Env,
    admin: Address,
    signers: Vec<Address>,
    threshold: u32,
    timelock_duration: u64,
    emergency_threshold: u32,
    max_withdrawal_amount: i128,
) -> Result<(), Symbol>
```

Initialize the treasury controller with multisig configuration.

### Core Operations

#### Proposal Management

```rust
// Create a new proposal
pub fn create_proposal(
    env: Env,
    proposer: Address,
    proposal_type: ProposalType,
    target_address: Address,
    token_contract: Address,
    amount: i128,
    purpose: String,
    metadata: String,
    execution_data: Bytes,
) -> Result<u64, Symbol>

// Approve a proposal
pub fn approve_proposal(env: Env, signer: Address, proposal_id: u64) -> Result<(), Symbol>

// Reject a proposal
pub fn reject_proposal(env: Env, signer: Address, proposal_id: u64) -> Result<(), Symbol>

// Execute an approved proposal
pub fn execute_proposal(env: Env, executor: Address, proposal_id: u64) -> Result<(), Symbol>
```

#### Emergency Controls

```rust
// Emergency halt all operations
pub fn emergency_halt(env: Env, caller: Address) -> Result<(), Symbol>

// Resume normal operations
pub fn resume_operations(env: Env, caller: Address) -> Result<(), Symbol>
```

#### Token Management

```rust
// Add supported token for treasury operations
pub fn add_supported_token(env: Env, token_address: Address) -> Result<(), Symbol>
```

### Query Functions

```rust
// Get treasury configuration
pub fn get_config(env: Env) -> Result<TreasuryConfig, Symbol>

// Get proposal details
pub fn get_proposal(env: Env, proposal_id: u64) -> Result<TreasuryProposal, Symbol>

// Get withdrawal audit record
pub fn get_withdrawal_record(env: Env, proposal_id: u64) -> Result<WithdrawalRecord, Symbol>

// Check if proposal is ready for execution
pub fn is_proposal_executable(env: Env, proposal_id: u64) -> Result<bool, Symbol>
```

### Gnosis Safe Compatibility

```rust
// Create proposal using Gnosis Safe format
pub fn gnosis_create_proposal(
    env: Env,
    to: Address,
    value: i128,
    data: Bytes,
    operation: u8,
    safe_tx_gas: u64,
    base_gas: u64,
    gas_price: u64,
    gas_token: Address,
    refund_receiver: Address,
    nonce: u64,
) -> Result<u64, Symbol>

// Execute proposal with Gnosis Safe signatures
pub fn gnosis_execute_proposal(
    env: Env,
    proposal_id: u64,
    signatures: Bytes,
) -> Result<(), Symbol>

// Get multisig threshold (Gnosis Safe compatible)
pub fn gnosis_get_threshold(env: Env) -> Result<u32, Symbol>

// Get multisig owners (Gnosis Safe compatible)
pub fn gnosis_get_owners(env: Env) -> Result<Vec<Address>, Symbol>
```

## Data Structures

### TreasuryProposal

```rust
pub struct TreasuryProposal {
    pub proposal_id: u64,
    pub proposal_type: ProposalType,
    pub proposer: Address,
    pub target_address: Address,
    pub token_contract: Address,
    pub amount: i128,
    pub purpose: String,
    pub metadata: String,
    pub created_at: u64,
    pub timelock_end: u64,
    pub status: ProposalStatus,
    pub approvals: Vec<Address>,
    pub rejections: Vec<Address>,
    pub execution_data: Bytes,
}
```

### MultisigConfig

```rust
pub struct MultisigConfig {
    pub signers: Vec<Address>,
    pub threshold: u32,
    pub timelock_duration: u64,
    pub emergency_threshold: u32,
}
```

### WithdrawalRecord

```rust
pub struct WithdrawalRecord {
    pub proposal_id: u64,
    pub token_contract: Address,
    pub amount: i128,
    pub recipient: Address,
    pub purpose: String,
    pub executed_at: u64,
    pub executed_by: Address,
    pub transaction_hash: BytesN<32>,
}
```

## Events

The contract emits the following events for monitoring and audit purposes:

- **ProposalCreated**: New proposal submitted
- **Approved**: Proposal approved by signer
- **Rejected**: Proposal rejected by signer
- **Executed**: Proposal successfully executed
- **Withdrawal**: Funds withdrawn from treasury
- **Emergency**: Emergency halt activated
- **Resumed**: Operations resumed after halt

## Usage Examples

### 1. Initialize Treasury Controller

```rust
let admin = Address::generate(&env);
let signers = vec![signer1, signer2, signer3];

client.initialize(
    &admin,
    &signers,
    &2u32,        // 2-of-3 multisig
    &86400u64,    // 24 hour timelock
    &2u32,        // Emergency threshold
    &1_000_000i128, // Max withdrawal: 1M tokens
)?;
```

### 2. Create Withdrawal Proposal

```rust
let proposal_id = client.create_proposal(
    &proposer,
    &ProposalType::Withdrawal,
    &recipient_address,
    &token_contract,
    &500_000i128,
    &String::from_str(&env, "Q1 Development Funding"),
    &String::from_str(&env, "Allocated for smart contract development"),
    &Bytes::new(&env),
)?;
```

### 3. Approve and Execute

```rust
// Signers approve the proposal
client.approve_proposal(&signer1, &proposal_id)?;
client.approve_proposal(&signer2, &proposal_id)?;

// Wait for timelock to pass...

// Execute the approved proposal
client.execute_proposal(&signer1, &proposal_id)?;
```

### 4. Emergency Procedures

```rust
// Emergency halt (admin or signers can trigger)
client.emergency_halt(&admin)?;

// Resume operations (admin only)
client.resume_operations(&admin)?;
```

### 5. Gnosis Safe Integration

```rust
// Create proposal via Gnosis Safe interface
let proposal_id = client.gnosis_create_proposal(
    &target_address,
    &amount,
    &call_data,
    &0u8, // Call operation
    &21000u64,
    &21000u64,
    &1u64,
    &gas_token,
    &refund_receiver,
    &nonce,
)?;

// Execute with Gnosis Safe signatures
client.gnosis_execute_proposal(&proposal_id, &signatures)?;
```

## Security Considerations

### Timelock Protection

- Minimum 1 hour timelock for all operations
- Maximum 1 week timelock to prevent indefinite delays
- Emergency operations may have reduced timelock

### Signature Validation

- All operations require proper authentication
- Multisig threshold enforcement
- Prevention of duplicate approvals

### Emergency Mechanisms

- Immediate halt capability for crisis response
- Admin-only resume functionality
- Emergency threshold for critical operations

### Audit Trail

- Complete record of all proposals and executions
- Immutable transaction history
- Comprehensive event logging

## Treasury Operations Process

### Standard Withdrawal Flow

1. **Proposal Creation**: Authorized signer creates withdrawal proposal
2. **Review Period**: Signers evaluate proposal and metadata
3. **Approval Process**: Required number of signers approve proposal
4. **Timelock Wait**: Mandatory delay before execution
5. **Execution**: Approved proposal executed after timelock
6. **Audit Record**: Withdrawal recorded for compliance

### Emergency Halt Procedure

1. **Trigger**: Admin or emergency threshold of signers activate halt
2. **Immediate Effect**: All treasury operations suspended
3. **Investigation**: Review of emergency situation
4. **Resolution**: Admin resumes operations when safe

### Configuration Changes

1. **Proposal**: Submit configuration change proposal
2. **Extended Review**: Longer timelock for critical changes
3. **Consensus**: Higher approval threshold may be required
4. **Implementation**: Configuration updated after approval

## Integration with Existing Contracts

The Treasury Controller integrates seamlessly with the Uzima ecosystem:

- **SUT Token**: Native token management and transfers
- **Token Sale**: Treasury funding from sale proceeds
- **Medical Records**: Emergency fund recovery mechanisms
- **Identity Registry**: Signer identity verification

## Testing

Run the comprehensive test suite:

```bash
cargo test -p treasury_controller
```

### Test Coverage

- ✅ Initialization and configuration validation
- ✅ Proposal creation and approval workflows
- ✅ Timelock enforcement and execution
- ✅ Emergency halt and resume procedures
- ✅ Gnosis Safe compatibility
- ✅ Unauthorized access prevention
- ✅ Proposal expiry handling
- ✅ Withdrawal limit enforcement
- ✅ Complete audit trail verification

## Deployment

### Prerequisites

1. **Build contracts**:
   ```bash
   soroban contract build
   ```

2. **Configure signers**: Prepare list of authorized multisig signers

3. **Set parameters**: Define threshold, timelock, and limits

### Production Deployment

```bash
# Deploy the contract
TREASURY_ID=$(soroban contract deploy \
    --wasm target/wasm32-unknown-unknown/release/treasury_controller.wasm \
    --source deployer \
    --network mainnet)

# Initialize with production parameters
soroban contract invoke \
    --id $TREASURY_ID \
    --source admin \
    --network mainnet \
    -- \
    initialize \
    --admin $ADMIN_ADDRESS \
    --signers "[$SIGNER1,$SIGNER2,$SIGNER3]" \
    --threshold 2 \
    --timelock_duration 86400 \
    --emergency_threshold 2 \
    --max_withdrawal_amount 10000000000
```

### Gnosis Safe Setup

For organizations already using Gnosis Safe:

1. **Deploy Treasury Controller** with Gnosis Safe owners as signers
2. **Configure identical threshold** as Gnosis Safe setup
3. **Use Gnosis Safe interface** for proposal creation and approval
4. **Maintain dual compatibility** for enhanced security

## Best Practices

### Operational Security

- **Regular Reviews**: Periodic assessment of signer list and thresholds
- **Threshold Management**: Balance security with operational efficiency
- **Emergency Preparedness**: Clear procedures for crisis response
- **Access Control**: Strict management of admin privileges

### Proposal Management

- **Clear Documentation**: Detailed purpose and metadata for proposals
- **Reasonable Limits**: Set appropriate withdrawal limits
- **Review Process**: Adequate time for proposal evaluation
- **Rejection Process**: Democratic rejection of inappropriate proposals

### Audit and Compliance

- **Regular Audits**: Periodic review of treasury operations
- **Record Keeping**: Maintain comprehensive transaction records
- **Transparency**: Public access to proposal and execution data
- **Compliance**: Adherence to relevant regulatory requirements

## Architecture Benefits

### Modularity

- **Standalone Operation**: Independent of specific wallet implementations
- **Flexible Integration**: Compatible with various multisig solutions
- **Upgradeable Design**: Support for future enhancements

### Scalability

- **Efficient Storage**: Optimized data structures for large-scale operations
- **Event-Driven**: Comprehensive logging for monitoring systems
- **Batch Operations**: Support for multiple proposals and approvals

### Reliability

- **Error Handling**: Comprehensive error codes and messages
- **State Consistency**: Atomic operations and rollback safety
- **Recovery Mechanisms**: Emergency procedures for crisis management

This Treasury Controller provides a robust, secure, and flexible foundation for managing protocol funds while maintaining compatibility with existing multisig solutions and providing comprehensive audit capabilities.
