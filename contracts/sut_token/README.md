# SUT Token - Stellar Utility Token

A comprehensive ERC-20-compatible utility token contract built for the Stellar ecosystem using Soroban smart contracts.

## Overview

The SUT (Stellar Utility Token) is designed for payments, staking, and access control across the Stellar ecosystem. It implements ERC-20-like functionality adapted for Soroban with additional features like supply caps, role-based minting, and snapshot capabilities.

## Features

### Core ERC-20 Functionality
- **Standard Interface**: Implements `name`, `symbol`, `decimals`, `totalSupply`, `balanceOf`, `allowance`
- **Transfers**: `transfer` and `transferFrom` with proper authorization
- **Approvals**: `approve` function for delegation

### Enhanced Features
- **Mintable & Burnable**: Controlled minting and burning by authorized minters
- **Supply Cap**: Configurable maximum supply that cannot be exceeded
- **Role-based Access Control**: Admin and minter roles with proper authorization
- **Snapshot Capability**: Create snapshots for voting and reward distribution
- **Comprehensive Events**: All operations emit appropriate events

### Security Features
- **Authorization**: All sensitive operations require proper authentication
- **Input Validation**: Comprehensive validation of amounts and addresses
- **Overflow Protection**: Safe arithmetic operations
- **Role Management**: Granular control over minting permissions

## Contract Interface

### Initialization
```rust
pub fn initialize(
    env: Env,
    admin: Address,
    name: String,
    symbol: String,
    decimals: u32,
    supply_cap: i128,
) -> Result<(), Error>
```

### Core Functions
```rust
// ERC-20 Standard
pub fn name(env: Env) -> Result<String, Error>
pub fn symbol(env: Env) -> Result<String, Error>
pub fn decimals(env: Env) -> Result<u32, Error>
pub fn total_supply(env: Env) -> Result<i128, Error>
pub fn balance_of(env: Env, account: Address) -> i128
pub fn allowance(env: Env, owner: Address, spender: Address) -> i128

// Transfer Functions
pub fn transfer(env: Env, from: Address, to: Address, amount: i128) -> Result<(), Error>
pub fn transfer_from(env: Env, spender: Address, from: Address, to: Address, amount: i128) -> Result<(), Error>
pub fn approve(env: Env, owner: Address, spender: Address, amount: i128) -> Result<(), Error>

// Minting & Burning
pub fn mint(env: Env, minter: Address, to: Address, amount: i128) -> Result<(), Error>
pub fn burn(env: Env, minter: Address, from: Address, amount: i128) -> Result<(), Error>

// Role Management
pub fn add_minter(env: Env, minter: Address) -> Result<(), Error>
pub fn remove_minter(env: Env, minter: Address) -> Result<(), Error>
pub fn is_minter(env: Env, address: Address) -> bool

// Snapshot Functions
pub fn snapshot(env: Env) -> Result<u32, Error>
pub fn balance_of_at(env: Env, account: Address, snapshot_id: u32) -> Result<i128, Error>
pub fn total_supply_at(env: Env, snapshot_id: u32) -> Result<i128, Error>
```

## Error Types

```rust
pub enum Error {
    AlreadyInitialized = 1,
    NotInitialized = 2,
    Unauthorized = 3,
    InsufficientBalance = 4,
    InsufficientAllowance = 5,
    ExceedsSupplyCap = 6,
    InvalidAmount = 7,
    InvalidAddress = 8,
    SnapshotNotFound = 9,
}
```

## Events

The contract emits the following events:
- **Transfer**: When tokens are transferred
- **Approval**: When allowances are set
- **Mint**: When new tokens are minted
- **Burn**: When tokens are burned
- **Snapshot**: When a snapshot is created

## Usage Examples

### Deployment and Initialization
```rust
// Deploy the contract
let contract_id = env.register_contract(None, SutToken);
let client = SutTokenClient::new(&env, &contract_id);

// Initialize with parameters
let admin = Address::generate(&env);
let name = String::from_str(&env, "Stellar Utility Token");
let symbol = String::from_str(&env, "SUT");
let decimals = 18u32;
let supply_cap = 1_000_000_000i128 * 10i128.pow(decimals); // 1 billion tokens

client.initialize(&admin, &name, &symbol, &decimals, &supply_cap);
```

### Minting Tokens
```rust
// Admin mints tokens to a user
let user = Address::generate(&env);
let amount = 1000i128 * 10i128.pow(18); // 1000 tokens with 18 decimals
client.mint(&admin, &user, &amount);
```

### Creating Snapshots
```rust
// Create a snapshot for governance voting
let snapshot_id = client.snapshot();
let user_balance_at_snapshot = client.balance_of_at(&user, &snapshot_id);
```

## Testing

The contract includes comprehensive unit tests covering:
- ✅ Initialization and metadata
- ✅ Minting with supply cap enforcement
- ✅ Burning with balance validation
- ✅ Transfers and allowances
- ✅ Role-based access control
- ✅ Snapshot functionality
- ✅ Edge cases and error conditions
- ✅ Input validation

Run tests with:
```bash
cargo test
```

## Security Considerations

1. **Authorization**: All minting, burning, and administrative functions require proper authorization
2. **Supply Cap**: Total supply cannot exceed the configured cap
3. **Input Validation**: All amounts and addresses are validated
4. **Role Management**: Only admin can add/remove minters
5. **Overflow Protection**: Safe arithmetic prevents overflow attacks

## Gas Optimization

The contract is optimized for common operations:
- Efficient storage patterns using persistent and instance storage
- Minimal storage operations for balance updates
- Event emission for off-chain indexing

## Upgrade Considerations

The contract is designed to be immutable once deployed. For upgradeable versions, consider:
- Implementing proxy patterns
- Using storage versioning
- Planning migration strategies

## Chain Agnostic Design

The contract avoids chain-specific features and can be adapted for other blockchain platforms by:
- Using generic address types
- Avoiding platform-specific oracles
- Implementing standard interfaces

## License

MIT License - See LICENSE file for details.

## Contributing

1. Fork the repository
2. Create a feature branch
3. Add comprehensive tests
4. Submit a pull request

## Audit Status

⚠️ **This contract has not been audited**. Please conduct a thorough security audit before using in production.
