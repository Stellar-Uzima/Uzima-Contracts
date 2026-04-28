# Contract Invariant Documentation

## Overview

This document establishes standards for identifying, documenting, and testing critical invariants and assumptions for smart contracts in the Uzima-Contracts project. Invariants are properties that must always hold true for the contract to function correctly and securely.

## Invariant Classification

### Critical Invariants
Properties that, if violated, could lead to catastrophic failure, loss of funds, or security breaches.

### Important Invariants
Properties that, if violated, could lead to incorrect behavior, financial loss, or degraded functionality.

### Informational Invariants
Properties that, if violated, could lead to minor issues or unexpected behavior but don't pose immediate risks.

## Invariant Documentation Template

### Contract Header Template

```markdown
# ContractName Invariant Documentation

## Contract Overview
**Contract**: ContractName  
**Version**: 1.0.0  
**Author**: Development Team  
**Last Updated**: 2024-01-15  
**Review Date**: 2024-04-15  

## Purpose
Brief description of the contract's primary function and role in the system.

## Key Assumptions
List of fundamental assumptions about the contract's operating environment and dependencies.
```

### Invariant Entry Template

```markdown
### INV-001: Total Supply Conservation

**Category**: Critical  
**Scope**: State Management  
**Priority**: P0  

#### Description
The total supply of tokens must always equal the sum of all individual token balances across all addresses.

#### Formal Specification
```
∀ balances: mapping(address => uint256)
totalSupply == Σ balances[address] for all addresses
```

#### Pre-conditions
- Contract is properly initialized
- No overflow/underflow in arithmetic operations

#### Post-conditions
- After any transfer, totalSupply remains unchanged
- After minting, totalSupply increases by minted amount
- After burning, totalSupply decreases by burned amount

#### Potential Violations
1. **Integer Overflow**: Arithmetic operations exceed uint256 limits
2. **State Corruption**: Direct manipulation of storage variables
3. **Reentrancy**: State changes during external calls

#### Detection Methods
- **Event Monitoring**: Track Transfer events and validate balance consistency
- **State Verification**: Periodic balance sum verification
- **Unit Tests**: Comprehensive test coverage for all state-changing functions

#### Testing Strategy
```solidity
// Property test example
function test_totalSupplyInvariant() public {
    uint256 initialSupply = token.totalSupply();
    
    // Perform random operations
    for (uint i = 0; i < 100; i++) {
        uint256 amount = _randomAmount();
        address from = _randomAddress();
        address to = _randomAddress();
        
        try token.transferFrom(from, to, amount) {
            // Transfer succeeded
        } catch {
            // Transfer failed
        }
    }
    
    // Verify invariant holds
    uint256 calculatedSupply = _calculateTotalSupply();
    assert(calculatedSupply == token.totalSupply());
}
```

#### Mitigation Measures
- Use SafeMath or Solidity 0.8+ built-in overflow protection
- Implement reentrancy guards
- Add comprehensive event logging
- Regular balance reconciliation checks

#### Related Issues
- #123: Reentrancy vulnerability in transfer function
- #456: Overflow protection implementation
```

## Contract-Specific Invariants

### ERC20 Token Invariants

```markdown
### INV-201: Non-Negative Balances

**Category**: Critical  
**Scope**: Balance Management  
**Priority**: P0  

#### Description
No address should ever have a negative token balance.

#### Formal Specification
```
∀ address: address
balances[address] >= 0
```

#### Testing Strategy
```solidity
function test_nonNegativeBalances() public {
    // Test all transfer scenarios
    // Test minting scenarios
    // Test burning scenarios
    // Test edge cases (zero transfers, max uint256)
}
```

### INV-202: Approval Consistency

**Category**: Important  
**Scope**: Allowance Management  
**Priority**: P1  

#### Description
Allowance amounts should never exceed the owner's current balance.

#### Formal Specification
```
∀ owner, spender: address
allowance[owner][spender] <= balances[owner]
```

#### Testing Strategy
```solidity
function test_approvalConsistency() public {
    // Test approval scenarios
    // Test transferFrom scenarios
    // Test approval modification scenarios
}
```
```

### Governance Contract Invariants

```markdown
### INV-301: Voting Power Accuracy

**Category**: Critical  
**Scope**: Voting System  
**Priority**: P0  

#### Description
The sum of all voting powers must equal the total supply of governance tokens.

#### Formal Specification
```
∀ addresses: address[]
Σ votingPower[address] == totalSupply
```

#### Testing Strategy
```solidity
function test_votingPowerAccuracy() public {
    // Test voting power delegation
    // Test token transfers affecting voting power
    // Test voting power calculations
}
```

### INV-302: Proposal Uniqueness

**Category**: Important  
**Scope**: Proposal Management  
**Priority**: P1  

#### Description
Each proposal ID must be unique and monotonically increasing.

#### Formal Specification
```
∀ proposalId1, proposalId2: uint256
proposalId1 != proposalId2 => proposals[proposalId1] != proposals[proposalId2]
```

#### Testing Strategy
```solidity
function test_proposalUniqueness() public {
    // Test proposal creation
    // Test proposal ID generation
    // Test proposal storage
}
```
```

### Staking Contract Invariants

```markdown
### INV-401: Total Staked Amount

**Category**: Critical  
**Scope**: Staking Management  
**Priority**: P0  

#### Description
The total staked amount must equal the sum of all individual stake amounts.

#### Formal Specification
```
∀ stakers: address[]
totalStaked == Σ stakes[staker]
```

#### Testing Strategy
```solidity
function test_totalStakedInvariant() public {
    // Test staking operations
    // Test unstaking operations
    // Test reward calculations
}
```

### INV-402: Reward Distribution Correctness

**Category**: Important  
**Scope**: Reward System  
**Priority**: P1  

#### Description
Total rewards distributed must not exceed the allocated reward pool.

#### Formal Specification
```
totalRewardsDistributed <= rewardPool
```

#### Testing Strategy
```solidity
function test_rewardDistribution() public {
    // Test reward calculation
    // Test reward claiming
    // Test reward pool management
}
```
```

## State Assumptions Documentation

### External Dependencies

```markdown
### EXT-001: Oracle Price Validity

**Dependency**: PriceOracle Contract  
**Assumption**: Oracle always returns valid, positive prices  

#### Expected Behavior
- Prices are always > 0
- Prices are updated regularly
- Prices reflect market conditions

#### Failure Modes
- Oracle returns 0 price
- Oracle becomes stale
- Oracle returns negative values

#### Mitigation
- Price validation checks
- Fallback price mechanisms
- Oracle health monitoring
```

### Network Assumptions

```markdown
### NET-001: Block Time Consistency

**Assumption**: Block times remain within expected range  

#### Expected Range
- Minimum block time: 12 seconds
- Maximum block time: 30 seconds
- Average block time: ~15 seconds

#### Impact of Violations
- Time-based calculations may be incorrect
- Vesting schedules may be affected
- Deadline enforcement may fail

#### Mitigation
- Block time validation
- Grace periods for time-sensitive operations
- Manual override capabilities
```

## Pre/Post Condition Specifications

### Function Specification Template

```markdown
#### Function: transfer(address to, uint256 amount)

**Pre-conditions**:
1. `msg.sender != address(0)`
2. `to != address(0)`
3. `amount > 0`
4. `balances[msg.sender] >= amount`
5. `balances[msg.sender] - amount >= 0` (no underflow)
6. `balances[to] + amount >= 0` (no overflow)

**Post-conditions**:
1. `balances[msg.sender] = old_balances[msg.sender] - amount`
2. `balances[to] = old_balances[to] + amount`
3. `totalSupply` remains unchanged
4. `Transfer(msg.sender, to, amount)` event emitted

**Invariant Preservation**:
- Total supply conservation (INV-201)
- Non-negative balances (INV-202)
- Approval consistency (if applicable)

**Error Conditions**:
- Revert if `to == address(0)`
- Revert if `balances[msg.sender] < amount`
- Revert on arithmetic overflow/underflow
```

## Example Violations and Analysis

### Historical Violation Examples

```markdown
### Case Study: Integer Overflow in Token Transfer

**Contract**: LegacyToken  
**Date**: 2023-06-15  
**Severity**: Critical  

#### Description
A token transfer function experienced integer overflow when transferring large amounts, allowing users to receive more tokens than intended.

#### Violated Invariants
- INV-201: Total Supply Conservation
- INV-202: Non-Negative Balances

#### Root Cause
- Missing overflow protection in arithmetic operations
- Insufficient input validation

#### Resolution
- Implemented SafeMath library
- Added input validation
- Enhanced test coverage

#### Lessons Learned
- Always use overflow protection
- Test edge cases thoroughly
- Implement comprehensive input validation
```

## Testing Strategy for Invariants

### Property-Based Testing

```solidity
// test/invariants/TokenInvariants.sol
import "forge-std/Test.sol";

contract TokenInvariantTests is Test {
    Token token;
    
    function setUp() public {
        token = new Token();
        token.mint(address(this), 1000000);
    }
    
    function invariant_totalSupplyConservation() public {
        uint256 totalSupply = token.totalSupply();
        uint256 calculatedSupply = _calculateTotalSupply();
        assertEq(totalSupply, calculatedSupply, "Total supply invariant violated");
    }
    
    function invariant_nonNegativeBalances() public {
        address[] memory users = _getAllUsers();
        for (uint i = 0; i < users.length; i++) {
            uint256 balance = token.balanceOf(users[i]);
            assertTrue(balance >= 0, "Negative balance detected");
        }
    }
    
    function _calculateTotalSupply() internal view returns (uint256) {
        // Implementation to sum all balances
    }
    
    function _getAllUsers() internal view returns (address[] memory) {
        // Implementation to get all addresses with balances
    }
}
```

### Fuzz Testing Framework

```solidity
// test/fuzz/TokenFuzz.sol
import "forge-std/Test.sol";

contract TokenFuzzTests is Test {
    Token token;
    address[] users;
    
    function setUp() public {
        token = new Token();
        // Initialize users
        for (uint i = 0; i < 10; i++) {
            users.push(address(uint160(0x10000 + i)));
            token.mint(users[i], 1000);
        }
    }
    
    function fuzz_transfer(uint256 fromIndex, uint256 toIndex, uint256 amount) public {
        fromIndex = bound(fromIndex, 0, users.length - 1);
        toIndex = bound(toIndex, 0, users.length - 1);
        amount = bound(amount, 0, 1000);
        
        address from = users[fromIndex];
        address to = users[toIndex];
        
        vm.prank(from);
        try token.transfer(to, amount) {
            // Verify invariants after successful transfer
            assertTrue(_verifyInvariants());
        } catch {
            // Invariants should still hold even on failed transfers
            assertTrue(_verifyInvariants());
        }
    }
    
    function _verifyInvariants() internal view returns (bool) {
        // Check all invariants
        return _checkTotalSupplyInvariant() && 
               _checkNonNegativeBalances() && 
               _checkApprovalConsistency();
    }
}
```

### State Machine Testing

```solidity
// test/statemachine/TokenStateMachine.sol
import "forge-std/Test.sol";
import "solmate/test/utils/machine/StateMachine.sol";

contract TokenStateMachine is Test, StateMachine {
    Token token;
    address[] users;
    
    struct State {
        mapping(address => uint256) balances;
        uint256 totalSupply;
    }
    
    function setUp() public {
        token = new Token();
        // Initialize state
    }
    
    function transition(State memory state, uint256 action) public {
        if (action == 0) {
            // Transfer action
            _executeTransfer(state);
        } else if (action == 1) {
            // Approve action
            _executeApprove(state);
        } else if (action == 2) {
            // TransferFrom action
            _executeTransferFrom(state);
        }
        
        // Verify invariants after each transition
        assertTrue(_verifyInvariants(state));
    }
    
    function _verifyInvariants(State memory state) internal view returns (bool) {
        // Comprehensive invariant checking
    }
}
```

## Monitoring and Alerting

### Runtime Invariant Checking

```solidity
contract InvariantMonitor {
    event InvariantViolation(string invariant, address indexed contract, bytes data);
    
    function checkTotalSupplyInvariant(address tokenAddress) external {
        Token token = Token(tokenAddress);
        uint256 totalSupply = token.totalSupply();
        uint256 calculatedSupply = _calculateTotalSupply(token);
        
        if (totalSupply != calculatedSupply) {
            emit InvariantViolation("TotalSupply", tokenAddress, abi.encode(totalSupply, calculatedSupply));
        }
    }
    
    function checkBalanceInvariant(address tokenAddress, address user) external {
        Token token = Token(tokenAddress);
        uint256 balance = token.balanceOf(user);
        
        if (balance < 0) {
            emit InvariantViolation("NegativeBalance", tokenAddress, abi.encode(user, balance));
        }
    }
}
```

### Off-Chain Monitoring

```javascript
// monitoring/invariant-monitor.js
class InvariantMonitor {
    constructor(contractAddress, provider) {
        this.contract = new Contract(contractAddress, ABI, provider);
    }
    
    async checkInvariants() {
        const invariants = await Promise.all([
            this.checkTotalSupplyInvariant(),
            this.checkBalanceInvariants(),
            this.checkApprovalInvariants()
        ]);
        
        const violations = invariants.filter(inv => !inv.passed);
        
        if (violations.length > 0) {
            await this.alertViolations(violations);
        }
    }
    
    async checkTotalSupplyInvariant() {
        const totalSupply = await this.contract.totalSupply();
        const calculatedSupply = await this.calculateTotalSupply();
        
        return {
            name: 'TotalSupply',
            passed: totalSupply.eq(calculatedSupply),
            expected: totalSupply.toString(),
            actual: calculatedSupply.toString()
        };
    }
    
    async alertViolations(violations) {
        // Send alerts to monitoring systems
        // Create GitHub issues
        // Notify development team
    }
}
```

## Review Process

### Invariant Review Checklist

#### Documentation Review
- [ ] All critical invariants are documented
- [ ] Formal specifications are provided
- [ ] Pre/post conditions are clearly defined
- [ ] Potential violations are identified
- [ ] Testing strategies are comprehensive

#### Implementation Review
- [ ] Invariants are properly implemented
- [ ] Error handling is robust
- [ ] Edge cases are covered
- [ ] Performance impact is acceptable
- [ ] Gas costs are optimized

#### Testing Review
- [ ] Unit tests cover all invariants
- [ ] Property tests are implemented
- [ ] Fuzz tests are comprehensive
- [ ] State machine tests are included
- [ ] Coverage requirements are met

### Approval Requirements

- **Critical Invariants**: Security team + tech lead approval
- **Important Invariants**: Tech lead approval
- **Informational Invariants**: Team lead approval

## Documentation Standards

### File Organization

```
docs/invariants/
├── contracts/
│   ├── Token.md
│   ├── Governance.md
│   ├── Staking.md
│   └── Treasury.md
├── templates/
│   ├── invariant-template.md
│   ├── function-specification.md
│   └── test-strategy.md
├── examples/
│   ├── violations/
│   └── best-practices/
└── review-process.md
```

### Version Control

- All invariant documentation must be versioned
- Changes to invariants require formal review
- Historical violations must be documented
- Lessons learned should be shared

## Tools and Resources

### Recommended Tools

- **Solidity**: For implementing invariants
- **Foundry**: For property-based testing
- **Echidna**: For fuzz testing
- **Mythril**: For static analysis
- **Slither**: For security analysis

### External Resources

- [Solidity by Example](https://solidity-by-example.org/)
- [Foundry Book](https://book.getfoundry.sh/)
- [ConsenSys Smart Contract Best Practices](https://consensys.github.io/smart-contract-best-practices/)

---

This invariant documentation framework should be reviewed and updated regularly to ensure comprehensive coverage of all contract invariants and maintain the security and reliability of the Uzima-Contracts ecosystem.
