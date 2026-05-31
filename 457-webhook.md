# Code Complexity Metrics and Limits

## Overview

This document establishes automated code complexity analysis standards for the Uzima-Contracts project. These metrics help maintain code quality, readability, and security by preventing overly complex functions and contracts.

## Metrics to Track

### 1. Cyclomatic Complexity

**Definition**: Measures the number of linearly independent paths through a function's source code.

**Calculation**:
```
CC = E - N + 2P
Where:
E = Number of edges in control flow graph
N = Number of nodes in control flow graph
P = Number of connected components
```

**Practical Calculation**:
```
CC = 1 + (number of decision points)
Decision points include: if, for, while, do-while, case, catch, &&, ||
```

### 2. Cognitive Complexity

**Definition**: Measures how difficult it is to understand the control flow of a function.

**Key Factors**:
- Nesting depth
- Control flow breaks (break, continue, goto)
- Recursion
- Inheritance and polymorphism

### 3. Function Length

**Definition**: Number of lines of code in a function, excluding comments and blank lines.

### 4. Nesting Depth

**Definition**: Maximum level of nested control structures within a function.

### 5. Parameter Count

**Definition**: Number of parameters a function accepts.

## Complexity Thresholds

### Contract Tier Classification

| Contract Tier | Criticality | Review Requirements |
|---------------|-------------|-------------------|
| Core | Critical | Security review required |
| Utility | Important | Tech lead review required |
| Experimental | Development | Team lead review required |

### Threshold Limits by Tier

#### Core Contracts (Critical)

| Metric | Warning Threshold | Failure Threshold | Rationale |
|--------|------------------|-------------------|-----------|
| Cyclomatic Complexity | 8 | 12 | High security impact |
| Cognitive Complexity | 10 | 15 | Complex logic risks |
| Function Length | 50 lines | 75 lines | Maintainability |
| Nesting Depth | 3 | 4 | Readability impact |
| Parameter Count | 6 | 8 | Interface complexity |

#### Utility Contracts (Important)

| Metric | Warning Threshold | Failure Threshold | Rationale |
|--------|------------------|-------------------|-----------|
| Cyclomatic Complexity | 10 | 15 | Moderate impact |
| Cognitive Complexity | 12 | 18 | Standard complexity |
| Function Length | 75 lines | 100 lines | Reasonable size |
| Nesting Depth | 4 | 5 | Acceptable nesting |
| Parameter Count | 8 | 10 | Moderate interface |

#### Experimental Contracts (Development)

| Metric | Warning Threshold | Failure Threshold | Rationale |
|--------|------------------|-------------------|-----------|
| Cyclomatic Complexity | 12 | 20 | Development flexibility |
| Cognitive Complexity | 15 | 25 | Experimental nature |
| Function Length | 100 lines | 150 lines | Prototyping allowed |
| Nesting Depth | 5 | 6 | Higher tolerance |
| Parameter Count | 10 | 12 | Flexible interfaces |

## Implementation

### CI/CD Integration

#### GitHub Actions Configuration

```yaml
# .github/workflows/complexity-check.yml
name: Code Complexity Analysis

on:
  pull_request:
    branches: [ main, develop ]
  push:
    branches: [ main ]

jobs:
  complexity-analysis:
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
    
    - name: Run complexity analysis
      run: npm run complexity:check
    
    - name: Generate complexity report
      run: npm run complexity:report
    
    - name: Upload complexity report
      uses: actions/upload-artifact@v3
      with:
        name: complexity-report
        path: reports/complexity/
    
    - name: Comment PR with results
      uses: actions/github-script@v6
      with:
        script: |
          const fs = require('fs');
          const report = JSON.parse(fs.readFileSync('reports/complexity/summary.json', 'utf8'));
          
          const comment = `
          ## Code Complexity Analysis Results
          
          ${report.violations.length > 0 ? '⚠️ **Violations Found**' : '✅ **All Checks Passed**'}
          
          **Summary:**
          - Functions analyzed: ${report.functionsAnalyzed}
          - Warnings: ${report.warnings}
          - Failures: ${report.failures}
          
          ${report.violations.length > 0 ? '### Violations:\n' + report.violations.map(v => `- **${v.function}** (${v.contract}): ${v.metric} = ${v.value} (limit: ${v.limit})`).join('\n') : ''}
          `;
          
          github.rest.issues.createComment({
            issue_number: context.issue.number,
            owner: context.repo.owner,
            repo: context.repo.repo,
            body: comment
          });
```

### Complexity Analysis Tools

#### Slither Configuration

```python
# slither.config.py
from slither.analyses.data_dependency.data_dependency import DataDependency
from slither.detectors.abstract_detector import AbstractDetector, DetectorClassification

class ComplexityDetector(AbstractDetector):
    ARGUMENT = 'complexity'
    HELP = 'High complexity functions'
    IMPACT = DetectorClassification.MEDIUM
    CONFIDENCE = DetectorClassification.HIGH

    WIKI = 'https://github.com/crytic/slither/wiki/Detector-Documentation#high-complexity'

    def _detect(self):
        results = []
        for contract in self.compilation_unit.contracts:
            for function in contract.functions:
                if function.is_constructor or function.is_fallback:
                    continue
                
                complexity = self._calculate_cyclomatic_complexity(function)
                cognitive = self._calculate_cognitive_complexity(function)
                
                if complexity > self.complexity_threshold or cognitive > self.cognitive_threshold:
                    results.append({
                        'contract': contract.name,
                        'function': function.name,
                        'cyclomatic': complexity,
                        'cognitive': cognitive,
                        'lines': len(function.source_mapping.lines),
                        'nesting': self._calculate_nesting_depth(function),
                        'parameters': len(function.parameters)
                    })
        
        return results
```

#### Custom Complexity Analyzer

```javascript
// scripts/complexity-analyzer.js
const fs = require('fs');
const path = require('path');
const { parse } = require('@solidity-parser/parser');

class ComplexityAnalyzer {
    constructor(thresholds) {
        this.thresholds = thresholds;
        this.results = [];
    }
    
    analyzeContract(filePath) {
        const source = fs.readFileSync(filePath, 'utf8');
        const ast = parse(source);
        
        return this.traverseAST(ast, filePath);
    }
    
    traverseAST(node, filePath, contract = null) {
        if (node.type === 'ContractDefinition') {
            contract = node.name;
            return this.analyzeContract(node, filePath);
        }
        
        if (node.type === 'FunctionDefinition') {
            const complexity = this.calculateCyclomaticComplexity(node);
            const cognitive = this.calculateCognitiveComplexity(node);
            const lines = this.countLines(node);
            const nesting = this.calculateNestingDepth(node);
            const parameters = node.parameters ? node.parameters.length : 0;
            
            const result = {
                contract,
                function: node.name,
                file: filePath,
                cyclomatic: complexity,
                cognitive: cognitive,
                lines,
                nesting,
                parameters
            };
            
            this.checkThresholds(result);
            this.results.push(result);
        }
        
        // Recursively traverse child nodes
        if (node.children) {
            for (const child of node.children) {
                this.traverseAST(child, filePath, contract);
            }
        }
    }
    
    calculateCyclomaticComplexity(node) {
        let complexity = 1; // Base complexity
        
        // Count decision points
        this.traverseNode(node, (child) => {
            if (['IfStatement', 'ForStatement', 'WhileStatement', 'DoWhileStatement'].includes(child.type)) {
                complexity++;
            }
            
            if (child.type === 'BinaryOperation' && ['&&', '||'].includes(child.operator)) {
                complexity++;
            }
            
            if (child.type === 'Conditional') {
                complexity++;
            }
        });
        
        return complexity;
    }
    
    calculateCognitiveComplexity(node) {
        let complexity = 0;
        let nestingLevel = 0;
        
        this.traverseNode(node, (child) => {
            if (['IfStatement', 'ForStatement', 'WhileStatement', 'DoWhileStatement'].includes(child.type)) {
                complexity += 1 + nestingLevel;
                nestingLevel++;
            }
            
            if (child.type === 'BreakStatement' || child.type === 'ContinueStatement') {
                complexity += 1;
            }
        });
        
        return complexity;
    }
    
    calculateNestingDepth(node) {
        let maxDepth = 0;
        let currentDepth = 0;
        
        this.traverseNode(node, (child) => {
            if (['IfStatement', 'ForStatement', 'WhileStatement', 'DoWhileStatement'].includes(child.type)) {
                currentDepth++;
                maxDepth = Math.max(maxDepth, currentDepth);
            }
        });
        
        return maxDepth;
    }
    
    countLines(node) {
        if (!node.loc) return 0;
        return node.loc.end.line - node.loc.start.line + 1;
    }
    
    checkThresholds(result) {
        const tier = this.getContractTier(result.contract);
        const thresholds = this.thresholds[tier];
        
        const violations = [];
        
        if (result.cyclomatic > thresholds.cyclomatic.failure) {
            violations.push({
                metric: 'Cyclomatic Complexity',
                value: result.cyclomatic,
                limit: thresholds.cyclomatic.failure,
                severity: 'error'
            });
        } else if (result.cyclomatic > thresholds.cyclomatic.warning) {
            violations.push({
                metric: 'Cyclomatic Complexity',
                value: result.cyclomatic,
                limit: thresholds.cyclomatic.warning,
                severity: 'warning'
            });
        }
        
        // Similar checks for other metrics...
        
        if (violations.length > 0) {
            result.violations = violations;
        }
    }
    
    getContractTier(contract) {
        // Determine contract tier based on naming convention or configuration
        if (contract.includes('Core') || contract.includes('Vault')) {
            return 'core';
        } else if (contract.includes('Util') || contract.includes('Helper')) {
            return 'utility';
        } else {
            return 'experimental';
        }
    }
    
    generateReport() {
        const summary = {
            functionsAnalyzed: this.results.length,
            warnings: 0,
            failures: 0,
            violations: []
        };
        
        for (const result of this.results) {
            if (result.violations) {
                for (const violation of result.violations) {
                    if (violation.severity === 'error') {
                        summary.failures++;
                    } else {
                        summary.warnings++;
                    }
                    
                    summary.violations.push({
                        contract: result.contract,
                        function: result.function,
                        metric: violation.metric,
                        value: violation.value,
                        limit: violation.limit
                    });
                }
            }
        }
        
        return summary;
    }
}

// Usage example
const thresholds = {
    core: {
        cyclomatic: { warning: 8, failure: 12 },
        cognitive: { warning: 10, failure: 15 },
        lines: { warning: 50, failure: 75 },
        nesting: { warning: 3, failure: 4 },
        parameters: { warning: 6, failure: 8 }
    },
    utility: {
        cyclomatic: { warning: 10, failure: 15 },
        cognitive: { warning: 12, failure: 18 },
        lines: { warning: 75, failure: 100 },
        nesting: { warning: 4, failure: 5 },
        parameters: { warning: 8, failure: 10 }
    },
    experimental: {
        cyclomatic: { warning: 12, failure: 20 },
        cognitive: { warning: 15, failure: 25 },
        lines: { warning: 100, failure: 150 },
        nesting: { warning: 5, failure: 6 },
        parameters: { warning: 10, failure: 12 }
    }
};

const analyzer = new ComplexityAnalyzer(thresholds);
const contracts = fs.readdirSync('contracts').filter(f => f.endsWith('.sol'));

for (const contract of contracts) {
    analyzer.analyzeContract(`contracts/${contract}`);
}

const report = analyzer.generateReport();
fs.writeFileSync('reports/complexity/summary.json', JSON.stringify(report, null, 2));
```

## Baseline Metrics

### Current Project Baseline

```json
{
  "baseline": {
    "established": "2024-01-15",
    "contracts": {
      "Token": {
        "cyclomatic": { "avg": 4.2, "max": 8, "min": 1 },
        "cognitive": { "avg": 5.1, "max": 10, "min": 1 },
        "lines": { "avg": 25, "max": 50, "min": 5 },
        "nesting": { "avg": 1.5, "max": 3, "min": 0 },
        "parameters": { "avg": 2.3, "max": 5, "min": 0 }
      },
      "Governance": {
        "cyclomatic": { "avg": 6.8, "max": 12, "min": 2 },
        "cognitive": { "avg": 8.2, "max": 15, "min": 2 },
        "lines": { "avg": 45, "max": 75, "min": 10 },
        "nesting": { "avg": 2.3, "max": 4, "min": 1 },
        "parameters": { "avg": 3.7, "max": 7, "min": 1 }
      }
    }
  }
}
```

### Trend Analysis

```javascript
// scripts/trend-analyzer.js
class TrendAnalyzer {
    constructor() {
        this.historicalData = [];
    }
    
    recordSnapshot(results, timestamp = new Date()) {
        const snapshot = {
            timestamp,
            summary: this.calculateSummary(results),
            details: results
        };
        
        this.historicalData.push(snapshot);
        this.saveHistoricalData();
    }
    
    calculateSummary(results) {
        const summary = {
            totalFunctions: results.length,
            avgCyclomatic: 0,
            avgCognitive: 0,
            avgLines: 0,
            avgNesting: 0,
            avgParameters: 0,
            violations: 0
        };
        
        for (const result of results) {
            summary.avgCyclomatic += result.cyclomatic;
            summary.avgCognitive += result.cognitive;
            summary.avgLines += result.lines;
            summary.avgNesting += result.nesting;
            summary.avgParameters += result.parameters;
            
            if (result.violations) {
                summary.violations += result.violations.length;
            }
        }
        
        // Calculate averages
        summary.avgCyclomatic /= results.length;
        summary.avgCognitive /= results.length;
        summary.avgLines /= results.length;
        summary.avgNesting /= results.length;
        summary.avgParameters /= results.length;
        
        return summary;
    }
    
    generateTrendReport(days = 30) {
        const cutoffDate = new Date();
        cutoffDate.setDate(cutoffDate.getDate() - days);
        
        const recentData = this.historicalData.filter(
            snapshot => new Date(snapshot.timestamp) >= cutoffDate
        );
        
        if (recentData.length < 2) {
            return { error: 'Insufficient data for trend analysis' };
        }
        
        const oldest = recentData[0].summary;
        const newest = recentData[recentData.length - 1].summary;
        
        return {
            period: `${days} days`,
            trends: {
                cyclomatic: this.calculateTrend(oldest.avgCyclomatic, newest.avgCyclomatic),
                cognitive: this.calculateTrend(oldest.avgCognitive, newest.avgCognitive),
                lines: this.calculateTrend(oldest.avgLines, newest.avgLines),
                nesting: this.calculateTrend(oldest.avgNesting, newest.avgNesting),
                parameters: this.calculateTrend(oldest.avgParameters, newest.avgParameters),
                violations: this.calculateTrend(oldest.violations, newest.violations)
            }
        };
    }
    
    calculateTrend(oldValue, newValue) {
        const change = newValue - oldValue;
        const percentChange = oldValue > 0 ? (change / oldValue) * 100 : 0;
        
        return {
            change,
            percentChange,
            direction: percentChange > 5 ? 'increasing' : percentChange < -5 ? 'decreasing' : 'stable'
        };
    }
}
```

## Documentation Requirements

### Function Complexity Documentation

```solidity
/**
 * @title Complex Function Example
 * @dev This function demonstrates complexity documentation requirements
 * 
 * Complexity Metrics:
 * - Cyclomatic: 8 (WARNING: Approaching limit of 12)
 * - Cognitive: 10 (WARNING: Approaching limit of 15)
 * - Lines: 45 (OK)
 * - Nesting: 3 (WARNING: Approaching limit of 4)
 * - Parameters: 4 (OK)
 * 
 * Complexity Rationale:
 * This function requires multiple validation steps and conditional logic
 * to ensure secure token transfers. The complexity is justified by:
 * 1. Multiple input validations
 * 2. Access control checks
 * 3. State transition validation
 * 4. Event emission requirements
 * 
 * Refactoring Considerations:
 * - Consider extracting validation logic
 * - Potential for helper functions
 * - State machine pattern could reduce nesting
 */
function complexTransfer(
    address to,
    uint256 amount,
    bytes calldata data,
    bool validateRecipient
) external nonReentrant whenNotPaused returns (bool) {
    // Input validation
    require(to != address(0), "Invalid recipient");
    require(amount > 0, "Amount must be positive");
    require(amount <= balanceOf(msg.sender), "Insufficient balance");
    
    // Access control
    if (validateRecipient) {
        require(isValidRecipient(to), "Recipient not whitelisted");
    }
    
    // State validation
    if (hasRestrictions(msg.sender)) {
        require(amount <= transferLimit(msg.sender), "Exceeds transfer limit");
    }
    
    // Execute transfer
    _transfer(msg.sender, to, amount);
    
    // Additional processing
    if (data.length > 0) {
        _processTransferData(data);
    }
    
    emit Transfer(msg.sender, to, amount);
    return true;
}
```

### Contract Complexity Summary

```solidity
/**
 * @title Token Contract
 * @dev ERC20 token with additional features
 * 
 * Contract Complexity Summary:
 * - Total Functions: 15
 * - Average Cyclomatic Complexity: 4.2
 * - Maximum Cyclomatic Complexity: 8 (transfer function)
 * - Average Cognitive Complexity: 5.1
 * - Maximum Cognitive Complexity: 10 (transfer function)
 * - Average Function Length: 25 lines
 * - Maximum Function Length: 50 lines
 * - Average Nesting Depth: 1.5
 * - Maximum Nesting Depth: 3
 * - Average Parameter Count: 2.3
 * - Maximum Parameter Count: 5
 * 
 * Complexity Assessment: ACCEPTABLE
 * All functions within established thresholds for Core contracts.
 * 
 * Identified Areas for Improvement:
 * - transfer function: Consider extracting validation logic
 * - _batchTransfer function: High complexity, needs refactoring
 */
contract Token is ERC20, Ownable {
    // Contract implementation
}
```

## Refactoring Guidelines

### Complexity Reduction Strategies

#### 1. Extract Function Method

```solidity
// Before: High complexity
function processPayment(address recipient, uint256 amount, bytes calldata data) external {
    require(recipient != address(0), "Invalid recipient");
    require(amount > 0, "Invalid amount");
    require(amount <= balanceOf(msg.sender), "Insufficient balance");
    
    if (hasRestrictions(msg.sender)) {
        require(amount <= transferLimit(msg.sender), "Exceeds limit");
        require(isWhitelisted(recipient), "Recipient not whitelisted");
    }
    
    if (data.length > 0) {
        require(validateData(data), "Invalid data");
        _processData(data);
    }
    
    _transfer(msg.sender, recipient, amount);
    emit PaymentProcessed(msg.sender, recipient, amount);
}

// After: Reduced complexity
function processPayment(address recipient, uint256 amount, bytes calldata data) external {
    _validatePayment(recipient, amount);
    _processOptionalData(data);
    _executePayment(recipient, amount);
}

function _validatePayment(address recipient, uint256 amount) internal view {
    require(recipient != address(0), "Invalid recipient");
    require(amount > 0, "Invalid amount");
    require(amount <= balanceOf(msg.sender), "Insufficient balance");
    
    if (hasRestrictions(msg.sender)) {
        _validateRestrictedPayment(recipient, amount);
    }
}

function _validateRestrictedPayment(address recipient, uint256 amount) internal view {
    require(amount <= transferLimit(msg.sender), "Exceeds limit");
    require(isWhitelisted(recipient), "Recipient not whitelisted");
}

function _processOptionalData(bytes calldata data) internal {
    if (data.length > 0) {
        require(validateData(data), "Invalid data");
        _processData(data);
    }
}

function _executePayment(address recipient, uint256 amount) internal {
    _transfer(msg.sender, recipient, amount);
    emit PaymentProcessed(msg.sender, recipient, amount);
}
```

#### 2. Strategy Pattern for Complex Logic

```solidity
// Before: Complex conditional logic
function calculateReward(address user, uint256 stakedAmount, uint256 duration) external view returns (uint256) {
    uint256 baseReward = stakedAmount * duration / REWARD_RATE;
    
    if (isVipUser(user)) {
        baseReward = baseReward * 150 / 100;
    } else if (isEarlyUser(user)) {
        baseReward = baseReward * 120 / 100;
    }
    
    if (duration > 365 days) {
        baseReward = baseReward * 110 / 100;
    }
    
    if (stakedAmount > 1000 ether) {
        baseReward = baseReward * 105 / 100;
    }
    
    return baseReward;
}

// After: Strategy pattern
interface IRewardStrategy {
    function calculateBonus(address user, uint256 stakedAmount, uint256 duration) external view returns (uint256);
}

contract RewardCalculator {
    mapping(address => IRewardStrategy) public strategies;
    
    function calculateReward(address user, uint256 stakedAmount, uint256 duration) external view returns (uint256) {
        uint256 baseReward = stakedAmount * duration / REWARD_RATE;
        uint256 totalBonus = 0;
        
        for (uint i = 0; i < strategyCount; i++) {
            address strategyAddress = strategyAddresses[i];
            totalBonus += strategies[strategyAddress].calculateBonus(user, stakedAmount, duration);
        }
        
        return baseReward + totalBonus;
    }
}
```

## Monitoring and Alerting

### Complexity Dashboard

```javascript
// monitoring/complexity-dashboard.js
class ComplexityDashboard {
    constructor() {
        this.metrics = new Map();
        this.alerts = [];
    }
    
    updateMetrics(contractName, functionMetrics) {
        this.metrics.set(contractName, functionMetrics);
        this.checkAlerts(contractName, functionMetrics);
    }
    
    checkAlerts(contractName, metrics) {
        for (const [functionName, metric] of Object.entries(metrics)) {
            if (metric.cyclomatic > 10) {
                this.createAlert('HIGH_CYCLOMATIC', contractName, functionName, metric.cyclomatic);
            }
            
            if (metric.cognitive > 12) {
                this.createAlert('HIGH_COGNITIVE', contractName, functionName, metric.cognitive);
            }
            
            if (metric.lines > 60) {
                this.createAlert('LONG_FUNCTION', contractName, functionName, metric.lines);
            }
        }
    }
    
    createAlert(type, contract, function, value) {
        const alert = {
            id: Date.now(),
            type,
            contract,
            function,
            value,
            timestamp: new Date(),
            status: 'active'
        };
        
        this.alerts.push(alert);
        this.notifyTeam(alert);
    }
    
    generateReport() {
        const report = {
            timestamp: new Date(),
            summary: this.generateSummary(),
            alerts: this.alerts.filter(a => a.status === 'active'),
            trends: this.calculateTrends()
        };
        
        return report;
    }
}
```

## Review Process

### Complexity Review Checklist

#### Pre-merge Review
- [ ] All functions within complexity thresholds
- [ ] High complexity functions are documented
- [ ] Refactoring plan exists for violations
- [ ] Baseline metrics are updated
- [ ] Trend analysis shows improvement

#### Periodic Review
- [ ] Monthly complexity trends analyzed
- [ ] Baseline thresholds evaluated
- [ ] Refactoring backlog reviewed
- [ ] Team training needs identified
- [ ] Tool effectiveness assessed

### Exception Process

#### Temporary Waivers

```markdown
### Complexity Waiver Request

**Contract**: ComplexContract  
**Function**: complexLogic  
**Requested Threshold**: Cyclomatic 15 (limit: 12)  
**Duration**: 3 months  
**Reasoning**: 
- Function handles critical security validation
- Refactoring would require extensive testing
- Temporary measure while migration plan is developed

**Mitigation Plan**:
1. Extract validation logic in next sprint
2. Implement comprehensive unit tests
3. Schedule security review for refactored version

**Approval Required**: Tech Lead + Security Team
```

## Best Practices

### Development Guidelines

1. **Design for Simplicity**
   - Prefer simple, clear code over clever solutions
   - Use established design patterns
   - Consider readability and maintainability

2. **Continuous Monitoring**
   - Run complexity analysis on every commit
   - Track trends over time
   - Address violations early

3. **Regular Refactoring**
   - Schedule regular refactoring sessions
   - Address technical debt proactively
   - Use complexity metrics to prioritize

4. **Team Training**
   - Educate team on complexity concepts
   - Share refactoring techniques
   - Establish coding standards

### Tool Recommendations

- **Slither**: Static analysis with complexity detection
- **Solidity Metrics**: Custom complexity analysis
- **SonarQube**: Code quality and complexity tracking
- **CodeClimate**: Automated complexity monitoring

## Resources

### External References

- [Cyclomatic Complexity](https://en.wikipedia.org/wiki/Cyclomatic_complexity)
- [Cognitive Complexity](https://.sonarsource.github.io/cognitive-complexity/)
- [Solidity Style Guide](https://docs.soliditylang.org/en/latest/style-guide.html)
- [Refactoring Guru](https://refactoring.guru/)

### Internal Resources

- Code Review Guidelines
- Refactoring Playbook
- Architecture Decision Records
- Team Coding Standards

---

This complexity metrics framework should be reviewed quarterly and updated based on project experience, team feedback, and evolving best practices in smart contract development.
