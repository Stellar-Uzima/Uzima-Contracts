# Token Sale Contract

A comprehensive token sale contract for Soroban/Stellar that supports multi-phase sales, vesting schedules, and refund mechanisms.

## Features

### Core Sale Features
- ✅ Multi-phase sales with configurable timing and pricing
- ✅ Per-address and global caps per phase
- ✅ Support for multiple ERC-20 payment tokens (USDC, USDT, etc.)
- ✅ Soft cap and hard cap enforcement
- ✅ Pausable sales for emergency situations
- ✅ Refund mechanism if soft cap not met

### Vesting Features
- ✅ Cliff period before any tokens vest
- ✅ Linear vesting after cliff period
- ✅ Batch vesting schedule creation for team members
- ✅ Emergency vesting schedule updates (with safeguards)

### Security Features
- ✅ Owner-only administrative functions
- ✅ Emergency withdrawal to treasury
- ✅ Pull-over-push payment pattern
- ✅ Comprehensive event logging

## Contract Architecture

### Main Contracts
1. **TokenSaleContract** - Handles the token sale phases and contributions
2. **VestingContract** - Manages token vesting schedules with cliff and linear release
3. **SutToken** - The utility token being sold (SUT)

### Key Data Structures

```rust
pub struct SalePhase {
    pub start_time: u64,
    pub end_time: u64,
    pub price_per_token: u128,
    pub max_tokens: u128,
    pub sold_tokens: u128,
    pub per_address_cap: u128,
    pub is_active: bool,
}

pub struct VestingSchedule {
    pub cliff_duration: u64,
    pub vesting_duration: u64,
    pub start_time: u64,
    pub total_amount: u128,
    pub released_amount: u128,
}
```

## Usage Examples

### 1. Initialize Token Sale

```rust
// Deploy and initialize the token sale contract
client.initialize(
    &owner,
    &sut_token_address,
    &treasury_address,
    &1_000_000_000_000, // 1M USDC soft cap (6 decimals)
    &10_000_000_000_000 // 10M USDC hard cap
);
```

### 2. Add Sale Phases

```rust
// Phase 1: Early bird (30 days)
client.add_sale_phase(
    &start_time,
    &(start_time + 30 * 24 * 60 * 60), // 30 days
    &800_000,    // 0.8 USDC per SUT token
    &2_000_000_000_000, // 2M SUT tokens available
    &50_000_000_000     // 50K USDC per address cap
);

// Phase 2: Public sale (60 days)
client.add_sale_phase(
    &(start_time + 30 * 24 * 60 * 60),
    &(start_time + 90 * 24 * 60 * 60), // 60 more days
    &1_000_000,  // 1.0 USDC per SUT token
    &5_000_000_000_000, // 5M SUT tokens available
    &100_000_000_000    // 100K USDC per address cap
);
```

### 3. Contribute to Sale

```rust
// User contributes 1000 USDC to phase 0
client.contribute(
    &contributor_address,
    &0, // phase_id
    &usdc_token_address,
    &1_000_000_000 // 1000 USDC (6 decimals)
);
```

### 4. Create Team Vesting

```rust
// Create vesting for team member: 6 month cliff, 2 year total vesting
vesting_client.create_vesting_schedule(
    &team_member_address,
    &(6 * 30 * 24 * 60 * 60),  // 6 month cliff
    &(24 * 30 * 24 * 60 * 60), // 24 month total vesting
    &1_000_000_000_000          // 1M SUT tokens
);
```

### 5. Claim Tokens

```rust
// After sale is finalized and successful
client.claim_tokens(&contributor_address);

// Release vested tokens
vesting_client.release_tokens(&team_member_address);
```

## Testing

Run the comprehensive test suite:

```bash
cargo test
```

Key test scenarios covered:
- ✅ Multi-phase sale flow
- ✅ Over-subscription handling
- ✅ Refund mechanism when soft cap not met
- ✅ Vesting cliff and linear release
- ✅ Emergency functions
- ✅ Edge cases and error conditions

## Deployment

1. **Build contracts:**
   ```bash
   soroban contract build
   ```

2. **Deploy to testnet:**
   ```bash
   ./scripts/deploy_token_sale.sh
   ```

3. **Configure for mainnet:**
   - Update treasury address to multisig
   - Set appropriate caps and pricing
   - Configure supported payment tokens

## Security Considerations

### Access Control
- All administrative functions require owner authentication
- Emergency functions can only send funds to treasury
- Vesting schedules can only be created by contract owner

### Economic Security
- Per-address caps prevent whale dominance
- Soft cap ensures minimum viable raise
- Refund mechanism protects contributors if targets not met
- Vesting prevents immediate token dumps

### Technical Security
- Pull-over-push pattern for token transfers
- Comprehensive input validation
- Safe arithmetic operations
- Event logging for transparency

## Integration Guide

### Frontend Integration

```typescript
// Initialize sale contract client
const saleContract = new Contract({
  contractId: SALE_CONTRACT_ID,
  networkPassphrase: Networks.TESTNET,
  rpcUrl: 'https://soroban-testnet.stellar.org'
});

// Get current active phase
const currentPhase = await saleContract.call('get_current_phase');

// Contribute to sale
await saleContract.call('contribute', {
  contributor: userAddress,
  phase_id: currentPhase,
  token: USDC_ADDRESS,
  amount: contributionAmount
});
```

### Backend Integration

```javascript
// Monitor sale events
const events = await server.getEvents({
  startLedger: lastProcessedLedger,
  filters: [{
    type: 'contract',
    contractIds: [SALE_CONTRACT_ID]
  }]
});

// Process contributions
events.forEach(event => {
  if (event.topic.includes('contribution')) {
    // Update database, send notifications, etc.
  }
});
```

## Gas Optimization

The contracts are optimized for gas efficiency:
- Batch operations for multiple vesting schedules
- Efficient storage patterns using Soroban's storage types
- Minimal external calls
- Optimized data structures

## Roadmap

Future enhancements:
- [ ] Dutch auction pricing mechanism
- [ ] Whitelist/KYC integration
- [ ] Cross-chain bridge support
- [ ] Governance token integration
- [ ] Advanced vesting curves (exponential, step-wise)

## License

MIT License - see LICENSE file for details.