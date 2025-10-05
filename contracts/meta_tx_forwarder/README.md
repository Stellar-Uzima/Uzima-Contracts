# Meta-Transaction Forwarder (ERC-2771 Compatible)

## Overview

The Meta-Transaction Forwarder enables gasless transactions for the Uzima healthcare platform, allowing patients and doctors to interact with smart contracts without needing to hold native tokens for gas fees. This significantly improves user experience for non-crypto-savvy users.

## Features

- **ERC-2771 Compatibility**: Follows the ERC-2771 standard for meta-transaction forwarding
- **Signature Verification**: Validates user signatures to ensure authorization
- **Nonce-Based Replay Protection**: Prevents replay attacks using per-user nonces
- **Batch Transaction Support**: Execute multiple transactions in a single call
- **Relayer Management**: Register and manage trusted relayers
- **Fee Configuration**: Configurable relay fees per relayer

## Architecture

### Components

1. **MetaTxForwarder Contract**: Main forwarder contract that validates and executes meta-transactions
2. **ERC2771Context Module**: Utility module for target contracts to extract original sender
3. **Relayer Infrastructure**: Off-chain relayers that submit transactions on behalf of users

### Flow

```
User → Sign Transaction → Relayer → Forwarder Contract → Target Contract
                                          ↓
                                   Verify Signature
                                   Check Nonce
                                   Execute Call
```

## Data Structures

### ForwardRequest

```rust
pub struct ForwardRequest {
    pub from: Address,        // Original sender
    pub to: Address,          // Target contract
    pub value: i128,          // Value to transfer
    pub gas: u32,             // Gas limit
    pub nonce: u64,           // Nonce for replay protection
    pub deadline: u64,        // Expiration timestamp
    pub data: Bytes,          // Encoded function call data
}
```

### RelayerConfig

```rust
pub struct RelayerConfig {
    pub address: Address,
    pub is_active: bool,
    pub fee_percentage: u32,  // Fee in basis points (e.g., 100 = 1%)
}
```

## Core Functions

### Initialization

```rust
pub fn initialize(
    env: Env,
    owner: Address,
    fee_collector: Address,
    min_relayer_stake: i128,
) -> Result<(), Error>
```

Initialize the forwarder contract with owner and configuration.

### Execute Meta-Transaction

```rust
pub fn execute(
    env: Env,
    relayer: Address,
    request: ForwardRequest,
    signature: BytesN<64>,
) -> Result<Bytes, Error>
```

Execute a single meta-transaction on behalf of a user.

**Validation Steps:**
1. Verify relayer is authorized
2. Check request hasn't expired
3. Verify and increment nonce
4. Validate signature
5. Forward call to target contract

### Execute Batch

```rust
pub fn execute_batch(
    env: Env,
    relayer: Address,
    requests: Vec<ForwardRequest>,
    signatures: Vec<BytesN<64>>,
) -> Result<Vec<Bytes>, Error>
```

Execute multiple meta-transactions in a single call.

### Relayer Management

```rust
// Register a new relayer
pub fn register_relayer(
    env: Env,
    owner: Address,
    relayer: Address,
    fee_percentage: u32,
) -> Result<(), Error>

// Deactivate a relayer
pub fn deactivate_relayer(
    env: Env,
    owner: Address,
    relayer: Address,
) -> Result<(), Error>
```

### View Functions

```rust
// Get current nonce for a user
pub fn get_nonce(env: Env, user: Address) -> u64

// Check if address is an active relayer
pub fn is_relayer(env: Env, relayer: Address) -> bool

// Get relayer configuration
pub fn get_relayer_config(env: Env, relayer: Address) -> Option<RelayerConfig>

// Get trusted forwarder address
pub fn get_trusted_forwarder(env: Env) -> Address
```

## Integration with Target Contracts

Target contracts must implement ERC-2771 context awareness to correctly identify the original sender.

### Step 1: Import ERC2771Context

```rust
use meta_tx_forwarder::erc2771_context::{ERC2771Context, ERC2771ContextImpl};
```

### Step 2: Set Trusted Forwarder During Initialization

```rust
pub fn initialize(env: Env, admin: Address, forwarder: Address) {
    admin.require_auth();
    
    // Set trusted forwarder
    ERC2771ContextImpl::set_trusted_forwarder(&env, forwarder);
    
    // ... rest of initialization
}
```

### Step 3: Use msg_sender() Instead of env.invoker()

```rust
impl ERC2771Context for YourContract {
    fn get_trusted_forwarder(env: &Env) -> Option<Address> {
        ERC2771ContextImpl::get_trusted_forwarder(env)
    }
}

pub fn your_function(env: Env) {
    // Get the original sender (works for both direct and forwarded calls)
    let sender = Self::msg_sender(&env);
    
    // Use sender for authorization checks
    // ...
}
```

## Security Considerations

### Signature Verification

The forwarder uses Ed25519 signature verification to ensure that only the authorized user can execute transactions on their behalf.

### Nonce Management

Each user has a sequential nonce that must be used in order. This prevents:
- Replay attacks
- Out-of-order execution
- Double-spending

### Deadline Enforcement

Each request includes a deadline timestamp. Expired requests are rejected to prevent:
- Stale transactions from being executed
- Front-running attacks

### Trusted Forwarder Pattern

Target contracts should:
- Only trust a single forwarder address per environment
- Validate the forwarder address during initialization
- Never allow the forwarder address to be changed after initialization

### Relayer Authorization

Only registered and active relayers can submit transactions. This prevents:
- Unauthorized relayers from submitting transactions
- Spam attacks
- Malicious relayers

## Relayer Economics

### Fee Model

Relayers charge fees for submitting transactions on behalf of users. The fee model includes:

1. **Percentage-Based Fees**: Configurable per relayer (in basis points)
2. **Fee Collection**: Fees are collected by a designated fee collector address
3. **Minimum Stake**: Relayers must stake a minimum amount to be registered

### Fee Calculation Example

```
Transaction Value: 1000 tokens
Relayer Fee: 100 basis points (1%)
Fee Amount: 1000 * 0.01 = 10 tokens
User Receives: 990 tokens
```

### Relayer Incentives

- **Transaction Fees**: Earn fees for each transaction submitted
- **Reputation**: Build reputation by providing reliable service
- **Stake Returns**: Earn returns on staked tokens

## Deployment Guide

### Step 1: Deploy Forwarder Contract

```bash
# Build the contract
cargo build --target wasm32-unknown-unknown --release

# Deploy to network
soroban contract deploy \
  --wasm target/wasm32-unknown-unknown/release/meta_tx_forwarder.wasm \
  --network testnet
```

### Step 2: Initialize Forwarder

```bash
soroban contract invoke \
  --id <FORWARDER_CONTRACT_ID> \
  --network testnet \
  -- initialize \
  --owner <OWNER_ADDRESS> \
  --fee_collector <FEE_COLLECTOR_ADDRESS> \
  --min_relayer_stake 1000000000
```

### Step 3: Register Relayers

```bash
soroban contract invoke \
  --id <FORWARDER_CONTRACT_ID> \
  --network testnet \
  -- register_relayer \
  --owner <OWNER_ADDRESS> \
  --relayer <RELAYER_ADDRESS> \
  --fee_percentage 100
```

### Step 4: Update Target Contracts

Update your target contracts (medical_records, identity_registry, etc.) to:
1. Accept the forwarder address during initialization
2. Use ERC2771Context for sender extraction
3. Replace `env.invoker()` with `Self::msg_sender(&env)`

### Step 5: Deploy Relayer Infrastructure

Set up off-chain relayers to:
1. Listen for user transaction requests
2. Validate and sign requests
3. Submit to forwarder contract
4. Monitor transaction status

## Testing

### Unit Tests

```bash
cargo test
```

### Integration Tests

Test the complete flow:
1. User signs a transaction
2. Relayer submits to forwarder
3. Forwarder validates and executes
4. Target contract receives correct sender

### Test Scenarios

- ✅ Valid meta-transaction execution
- ✅ Invalid signature rejection
- ✅ Expired request rejection
- ✅ Invalid nonce rejection
- ✅ Unauthorized relayer rejection
- ✅ Batch transaction execution
- ✅ Relayer registration and deactivation
- ✅ Fee calculation and collection

## Usage Examples

### Example 1: Patient Adding Medical Record (Gasless)

```rust
// 1. Patient creates and signs request off-chain
let request = ForwardRequest {
    from: patient_address,
    to: medical_records_contract,
    value: 0,
    gas: 100000,
    nonce: get_current_nonce(patient_address),
    deadline: current_time + 3600,
    data: encode_add_record_call(diagnosis, treatment, ...),
};

let signature = sign_request(patient_private_key, request);

// 2. Send to relayer (off-chain)
send_to_relayer(request, signature);

// 3. Relayer submits to forwarder
forwarder.execute(relayer_address, request, signature);

// 4. Forwarder validates and forwards to medical_records contract
// 5. Medical records contract extracts original sender (patient)
// 6. Record is added with patient as the original sender
```

### Example 2: Doctor Viewing Records (Gasless)

```rust
// Similar flow for doctor operations
let request = ForwardRequest {
    from: doctor_address,
    to: medical_records_contract,
    value: 0,
    gas: 50000,
    nonce: get_current_nonce(doctor_address),
    deadline: current_time + 3600,
    data: encode_read_record_call(patient_id, record_id),
};

// Sign, relay, and execute
```

## Monitoring and Analytics

### Events

The forwarder emits events for:
- Initialization: `("init", owner, fee_collector, min_stake)`
- Transaction Forwarding: `("fwd", relayer, from, to, nonce)`
- Relayer Registration: `("reg_relay", relayer, fee_percentage)`
- Relayer Deactivation: `("deact_rel", relayer)`

### Metrics to Track

- Total transactions forwarded
- Success/failure rates
- Average gas costs
- Relayer performance
- User adoption rates
- Fee revenue

## Troubleshooting

### Common Issues

1. **Invalid Signature Error**
   - Ensure signature is generated correctly
   - Verify request encoding matches expected format
   - Check that the correct private key is used

2. **Invalid Nonce Error**
   - Get current nonce before creating request
   - Ensure nonces are used sequentially
   - Don't reuse nonces

3. **Request Expired Error**
   - Set appropriate deadline (not too short)
   - Ensure relayer submits promptly
   - Check system time synchronization

4. **Unauthorized Relayer Error**
   - Verify relayer is registered
   - Check relayer is active
   - Ensure relayer address is correct

## Future Enhancements

- [ ] Support for EIP-712 typed data signing
- [ ] Gas estimation and optimization
- [ ] Dynamic fee adjustment based on network conditions
- [ ] Multi-signature support for high-value transactions
- [ ] Relayer reputation system
- [ ] Automatic nonce management
- [ ] Transaction batching optimization
- [ ] Cross-contract call support

## References

- [ERC-2771: Secure Protocol for Native Meta Transactions](https://eips.ethereum.org/EIPS/eip-2771)
- [OpenZeppelin MinimalForwarder](https://docs.openzeppelin.com/contracts/4.x/api/metatx)
- [Soroban Documentation](https://soroban.stellar.org/docs)

## License

MIT © 2025 Stellar Uzima Contributors
