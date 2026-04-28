# Comprehensive Testing Guidelines for Smart Contracts

## Overview

This document outlines best practices and standards for testing smart contracts across the Uzima-Contracts project. Proper testing ensures contract security, reliability, and maintainability.

## Testing Strategy Overview

### Testing Pyramid

```
    E2E Tests (5%)
   ─────────────────
  Integration Tests (25%)
 ─────────────────────────
Unit Tests (70%)
```

### Test Categories

1. **Unit Tests**: Individual function and state testing
2. **Integration Tests**: Contract interaction testing
3. **End-to-End Tests**: Complete workflow testing
4. **Property Tests**: Invariant and property verification
5. **Fuzz Tests**: Random input testing
6. **Gas Tests**: Performance and optimization validation

## Unit Test Guidelines

### Test Structure

```javascript
// test/Token.test.js
describe('Token Contract', function () {
  let token;
  let owner;
  let addr1;
  let addr2;

  beforeEach(async function () {
    [owner, addr1, addr2] = await ethers.getSigners();
    const Token = await ethers.getContractFactory('Token');
    token = await Token.deploy();
    await token.deployed();
  });

  describe('Deployment', function () {
    it('Should set the right owner', async function () {
      expect(await token.owner()).to.equal(owner.address);
    });

    it('Should assign the total supply to the owner', async function () {
      const ownerBalance = await token.balanceOf(owner.address);
      expect(await token.totalSupply()).to.equal(ownerBalance);
    });
  });

  describe('Transactions', function () {
    it('Should transfer tokens between accounts', async function () {
      await token.transfer(addr1.address, 50);
      const addr1Balance = await token.balanceOf(addr1.address);
      expect(addr1Balance).to.equal(50);
    });

    it('Should fail if sender doesn\'t have enough tokens', async function () {
      const initialOwnerBalance = await token.balanceOf(owner.address);
      await expect(
        token.connect(addr1).transfer(owner.address, 1)
      ).to.be.revertedWith('ERC20: transfer amount exceeds balance');
      
      expect(await token.balanceOf(owner.address)).to.equal(
        initialOwnerBalance
      );
    });
  });
});
```

### Unit Test Best Practices

1. **Test Naming Convention**
   ```javascript
   // Good: Descriptive and clear
   it('should revert when transferring more than balance');
   
   // Bad: Vague
   it('test transfer');
   ```

2. **Arrange-Act-Assert Pattern**
   ```javascript
   it('should update balances correctly', async function () {
     // Arrange
     const transferAmount = 100;
     const initialBalance = await token.balanceOf(addr1.address);
     
     // Act
     await token.transfer(addr1.address, transferAmount);
     
     // Assert
     const finalBalance = await token.balanceOf(addr1.address);
     expect(finalBalance).to.equal(initialBalance.add(transferAmount));
   });
   ```

3. **Test Edge Cases**
   ```javascript
   describe('Edge Cases', function () {
     it('should handle zero transfers');
     it('should handle maximum uint256 values');
     it('should handle contract addresses');
     it('should handle dead addresses');
   });
   ```

## Integration Test Patterns

### Contract Interaction Testing

```javascript
describe('Token-Governance Integration', function () {
  let token, governance;
  
  beforeEach(async function () {
    // Deploy both contracts
    const Token = await ethers.getContractFactory('Token');
    token = await Token.deploy();
    
    const Governance = await ethers.getContractFactory('Governance');
    governance = await Governance.deploy(token.address);
    
    // Set up relationship
    await token.transferOwnership(governance.address);
  });

  it('should allow governance to mint tokens', async function () {
    const proposalId = await governance.createProposal(
      'mint',
      [addr1.address, 1000]
    );
    
    await governance.vote(proposalId, true);
    await governance.execute(proposalId);
    
    expect(await token.balanceOf(addr1.address)).to.equal(1000);
  });
});
```

### Cross-Contract Communication Testing

```javascript
describe('Cross-Contract Events', function () {
  it('should emit events across contracts', async function () {
    await expect(token.approve(spender.address, 100))
      .to.emit(token, 'Approval')
      .withArgs(owner.address, spender.address, 100);
      
    await expect(token.connect(spender).transferFrom(owner.address, addr1.address, 50))
      .to.emit(token, 'Transfer')
      .withArgs(owner.address, addr1.address, 50);
  });
});
```

## Fixtures and Test Data Management

### Fixture Structure

```javascript
// test/fixtures.js
const { ethers } = require('hardhat');

async function tokenFixture() {
  const [owner, addr1, addr2, addr3] = await ethers.getSigners();
  
  const Token = await ethers.getContractFactory('Token');
  const token = await Token.deploy();
  
  return { token, owner, addr1, addr2, addr3 };
}

async function governanceFixture() {
  const { token, owner, addr1, addr2, addr3 } = await tokenFixture();
  
  const Governance = await ethers.getContractFactory('Governance');
  const governance = await Governance.deploy(token.address);
  
  return { token, governance, owner, addr1, addr2, addr3 };
}

module.exports = {
  tokenFixture,
  governanceFixture,
};
```

### Test Data Management

```javascript
// test/test-data.js
const TEST_DATA = {
  users: {
    owner: '0x1234...',
    user1: '0x5678...',
    user2: '0x9abc...'
  },
  amounts: {
    small: ethers.utils.parseEther('0.1'),
    medium: ethers.utils.parseEther('1'),
    large: ethers.utils.parseEther('100')
  },
  invalidAddresses: [
    '0x0000000000000000000000000000000000000000',
    '0xffffffffffffffffffffffffffffffffffffffff'
  ]
};

module.exports = TEST_DATA;
```

## Mocking Strategies

### External Contract Mocking

```javascript
// test/mocks/PriceOracleMock.sol
contract PriceOracleMock {
    mapping(address => uint256) private prices;
    
    function setPrice(address token, uint256 price) external {
        prices[token] = price;
    }
    
    function getPrice(address token) external view returns (uint256) {
        return prices[token];
    }
}

// test/TokenSale.test.js
describe('TokenSale with Mock Oracle', function () {
  let tokenSale, mockOracle;
  
  beforeEach(async function () {
    const MockOracle = await ethers.getContractFactory('PriceOracleMock');
    mockOracle = await MockOracle.deploy();
    
    const TokenSale = await ethers.getContractFactory('TokenSale');
    tokenSale = await TokenSale.deploy(mockOracle.address);
    
    // Set mock price
    await mockOracle.setPrice(token.address, ethers.utils.parseEther('0.001'));
  });
});
```

### Time Manipulation

```javascript
describe('Time-based Functions', function () {
  it('should respect vesting periods', async function () {
    const vestingPeriod = 365 * 24 * 60 * 60; // 1 year
    
    // Initial state
    expect(await vesting.getVestedAmount(addr1.address)).to.equal(0);
    
    // Fast forward time
    await ethers.provider.send('evm_increaseTime', [vestingPeriod]);
    await ethers.provider.send('evm_mine');
    
    // After vesting period
    expect(await vesting.getVestedAmount(addr1.address)).to.equal(totalAmount);
  });
});
```

## Test Naming Conventions

### File Naming

```
test/
├── contracts/
│   ├── Token.test.js          // Main contract tests
│   ├── Token.unit.test.js     // Unit-specific tests
│   ├── Token.integration.test.js // Integration tests
│   └── Token.fuzz.test.js     // Fuzz tests
├── fixtures/
│   ├── Token.fixture.js       // Token fixtures
│   └── common.fixture.js     // Common fixtures
└── utils/
    ├── helpers.js             // Test utilities
    └── constants.js           // Test constants
```

### Test Description Standards

```javascript
// Good naming examples
describe('Token Contract', function () {
  describe('transfer function', function () {
    it('should transfer tokens when sender has sufficient balance');
    it('should revert when transferring to zero address');
    it('should revert when amount exceeds balance');
    it('should emit Transfer event on successful transfer');
    it('should update balances correctly after transfer');
  });
});

// Bad naming examples
describe('Token', function () {
  it('test transfer');
  it('transfer test');
  it('should work');
});
```

## Coverage Requirements by Contract Tier

### Tier Classification

1. **Core Contracts** (Critical)
   - Line coverage: 95%
   - Branch coverage: 90%
   - Function coverage: 100%
   - Statement coverage: 95%

2. **Utility Contracts** (Important)
   - Line coverage: 90%
   - Branch coverage: 85%
   - Function coverage: 95%
   - Statement coverage: 90%

3. **Experimental Contracts** (Development)
   - Line coverage: 80%
   - Branch coverage: 75%
   - Function coverage: 85%
   - Statement coverage: 80%

### Coverage Configuration

```javascript
// hardhat.config.js
module.exports = {
  solidity: "0.8.19",
  mocha: {
    timeout: 40000
  },
  coverage: {
    providerOptions: {
      allowUnlimitedContractSize: true
    }
  },
  solidity: {
    coverage: {
      optimizer: {
        enabled: false,
        runs: 200
      }
    }
  }
};
```

### Coverage Scripts

```json
{
  "scripts": {
    "test": "hardhat test",
    "test:coverage": "hardhat coverage",
    "test:unit": "hardhat test test/**/*.unit.test.js",
    "test:integration": "hardhat test test/**/*.integration.test.js",
    "test:fuzz": "hardhat test test/**/*.fuzz.test.js",
    "coverage:report": "hardhat coverage --reporter lcov && genhtml coverage/lcov.info -o coverage/html"
  }
}
```

## Property Testing and Invariants

### Property Testing Example

```javascript
const { expect } = require('chai');
const { ethers } = require('hardhat');

describe('Token Property Tests', function () {
  it('should maintain total supply invariant', async function () {
    const { token, owner, addr1, addr2 } = await loadFixture(tokenFixture);
    
    // Property: Total supply always equals sum of all balances
    for (let i = 0; i < 100; i++) {
      const amount = ethers.BigNumber.from(Math.floor(Math.random() * 1000));
      const from = i % 2 === 0 ? owner : addr1;
      const to = i % 2 === 0 ? addr1 : addr2;
      
      try {
        await token.connect(from).transfer(to.address, amount);
      } catch (e) {
        // Transfer failed, continue
      }
      
      const totalSupply = await token.totalSupply();
      const ownerBalance = await token.balanceOf(owner.address);
      const addr1Balance = await token.balanceOf(addr1.address);
      const addr2Balance = await token.balanceOf(addr2.address);
      
      expect(totalSupply).to.equal(ownerBalance.add(addr1Balance).add(addr2Balance));
    }
  });
});
```

### Invariant Testing

```javascript
describe('Token Invariants', function () {
  it('should never allow negative balances', async function () {
    const { token, owner, addr1 } = await loadFixture(tokenFixture);
    
    for (let i = 0; i < 50; i++) {
      const amount = ethers.BigNumber.from(Math.floor(Math.random() * 1000));
      
      try {
        await token.transfer(addr1.address, amount);
      } catch (e) {
        // Expected for insufficient balance
      }
      
      const ownerBalance = await token.balanceOf(owner.address);
      const addr1Balance = await token.balanceOf(addr1.address);
      
      expect(ownerBalance.gte(0)).to.be.true;
      expect(addr1Balance.gte(0)).to.be.true;
    }
  });
});
```

## Fuzz Testing

### Fuzz Testing with Echidna

```solidity
// test/fuzz/TokenFuzz.sol
import "echidna.sol";

contract TokenFuzzTest {
    Token token;
    
    constructor() {
        token = new Token();
        token.mint(address(this), 1000000);
    }
    
    function test_transfer(uint256 amount) public {
        amount = bound(amount, 0, 1000);
        
        uint256 initialBalance = token.balanceOf(address(this));
        
        if (amount <= initialBalance) {
            token.transfer(address(0x1), amount);
            assert(token.balanceOf(address(0x1)) == amount);
        }
    }
    
    function test_no_overflow(uint256 amount) public {
        amount = bound(amount, 0, type(uint256).max);
        
        uint256 initialBalance = token.balanceOf(address(this));
        
        if (initialBalance <= type(uint256).max - amount) {
            token.transfer(address(0x1), amount);
            assert(token.balanceOf(address(this)) == initialBalance - amount);
        }
    }
}
```

## Gas Testing

### Gas Usage Testing

```javascript
describe('Gas Usage Tests', function () {
  it('should not exceed gas limits for transfer', async function () {
    const { token, owner, addr1 } = await loadFixture(tokenFixture);
    
    const tx = await token.transfer(addr1.address, 100);
    const receipt = await tx.wait();
    
    expect(receipt.gasUsed.toNumber()).to.be.lessThan(50000);
  });
  
  it('should optimize gas for batch operations', async function () {
    const { token, owner, addr1 } = await loadFixture(tokenFixture);
    
    const transfers = [];
    for (let i = 0; i < 10; i++) {
      transfers.push(token.transfer(addr1.address, 100));
    }
    
    const receipts = await Promise.all(transfers.map(tx => tx.then(t => t.wait())));
    const totalGas = receipts.reduce((sum, receipt) => sum + receipt.gasUsed.toNumber(), 0);
    const avgGas = totalGas / receipts.length;
    
    expect(avgGas).to.be.lessThan(60000); // Allow some overhead
  });
});
```

## Test Utilities and Helpers

### Common Test Utilities

```javascript
// test/utils/helpers.js
const { ethers } = require('hardhat');

async function getTimestamp() {
  const block = await ethers.provider.getBlock('latest');
  return block.timestamp;
}

async function increaseTime(seconds) {
  await ethers.provider.send('evm_increaseTime', [seconds]);
  await ethers.provider.send('evm_mine');
}

async function setNextBlockTimestamp(timestamp) {
  await ethers.provider.send('evm_setNextBlockTimestamp', [timestamp]);
}

async function expectRevert(contract, method, args, expectedError) {
  await expect(contract[method](...args)).to.be.revertedWith(expectedError);
}

async function getEvent(tx, eventName) {
  const receipt = await tx.wait();
  return receipt.events.find(event => event.event === eventName);
}

module.exports = {
  getTimestamp,
  increaseTime,
  setNextBlockTimestamp,
  expectRevert,
  getEvent
};
```

### Custom Matchers

```javascript
// test/utils/matchers.js
const { expect } = require('chai');
const { ethers } = require('hardhat');

expect.extend({
  toHaveEmittedEvent(receipt, eventName, expectedArgs = {}) {
    const event = receipt.events.find(e => e.event === eventName);
    
    if (!event) {
      throw new Error(`Event ${eventName} not found`);
    }
    
    for (const [key, value] of Object.entries(expectedArgs)) {
      if (!event.args[key].eq(value)) {
        throw new Error(`Event argument ${key} does not match expected value`);
      }
    }
    
    return {
      message: () => `Event ${eventName} emitted with correct arguments`,
      pass: true
    };
  }
});
```

## CI/CD Integration

### GitHub Actions Configuration

```yaml
# .github/workflows/test.yml
name: Smart Contract Tests

on:
  push:
    branches: [ main, develop ]
  pull_request:
    branches: [ main ]

jobs:
  test:
    runs-on: ubuntu-latest
    
    steps:
    - uses: actions/checkout@v3
    
    - name: Setup Node.js
      uses: actions/setup-node@v3
      with:
        node-version: '18'
        cache: 'npm'
    
    - name: Install dependencies
      run: npm ci
    
    - name: Run unit tests
      run: npm run test:unit
    
    - name: Run integration tests
      run: npm run test:integration
    
    - name: Run fuzz tests
      run: npm run test:fuzz
    
    - name: Generate coverage report
      run: npm run test:coverage
    
    - name: Upload coverage to Codecov
      uses: codecov/codecov-action@v3
      with:
        file: ./coverage/lcov.info
```

## Test Documentation Standards

### Test Documentation Template

```javascript
/**
 * @title Token Contract Tests
 * @dev Comprehensive test suite for ERC20 Token implementation
 * 
 * Test Categories:
 * - Unit Tests: Individual function testing
 * - Integration Tests: Contract interaction testing
 * - Property Tests: Invariant verification
 * - Gas Tests: Performance validation
 * 
 * Coverage Requirements:
 * - Line Coverage: 95%
 * - Branch Coverage: 90%
 * - Function Coverage: 100%
 * 
 * Test Environment:
 * - Hardhat Test Network
 * - ethers.js v5
 * - Mocha/Chai assertion library
 */
describe('Token Contract', function () {
  // Test implementation
});
```

## Best Practices Summary

### Do's
- Write descriptive test names
- Test both happy paths and edge cases
- Use fixtures for reusable test setup
- Mock external dependencies
- Test gas usage for critical functions
- Maintain high coverage requirements
- Document test purposes and scenarios

### Don'ts
- Skip testing error conditions
- Use hardcoded addresses in tests
- Ignore gas optimization
- Test implementation details instead of behavior
- Skip integration testing
- Ignore test flakiness
- Write tests without assertions

## Review Process

### Test Review Checklist

- [ ] Test names are descriptive and follow conventions
- [ ] Tests cover all critical paths and edge cases
- [ ] Error conditions are properly tested
- [ ] Gas usage is within acceptable limits
- [ ] Coverage requirements are met
- [ ] Fixtures are used appropriately
- [ ] External dependencies are mocked
- [ ] Tests are deterministic and not flaky
- [ ] Documentation is complete and accurate

### Approval Requirements

- **Core Contracts**: Security team + tech lead approval
- **Utility Contracts**: Tech lead approval
- **Experimental Contracts**: Team lead approval

---

This testing guidelines document should be reviewed quarterly and updated to incorporate new testing techniques, tools, and lessons learned from testing activities.
