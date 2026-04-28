# Contract Optimization Recommendations Engine

This tool analyzes Soroban smart contracts written in Rust and provides optimization recommendations to improve gas efficiency, storage usage, and overall performance.

## Features

- **Gas Optimization**: Identifies expensive operations and suggests more efficient alternatives
- **Storage Efficiency**: Recommends improvements for data storage patterns
- **Algorithm Optimization**: Suggests better algorithms for complex operations
- **Batching Opportunities**: Identifies where operations can be batched to reduce costs
- **Parallelization Possibilities**: Suggests opportunities for parallel execution where applicable

## Usage

### Analyze Contracts

```bash
cargo run --package contract_optimizer -- analyze
```

This will scan all contracts in the `contracts/` directory and output recommendations in text format.

For JSON output:

```bash
cargo run --package contract_optimizer -- analyze --format json
```

### Generate Report

```bash
cargo run --package contract_optimizer -- report --input optimization_results.json --output report.md
```

### View Metrics

```bash
cargo run --package contract_optimizer -- metrics
```

This shows the accuracy of recommendations (how many were applied).

### PR Integration

The engine automatically runs on pull requests that modify contracts and posts recommendations as comments.

## Categories of Recommendations

### Gas Optimization
- Minimize computational complexity
- Avoid unnecessary allocations
- Use efficient data structures

### Storage Efficiency
- Batch storage operations
- Use appropriate data types
- Minimize storage reads/writes

### Algorithm Optimization
- Use more efficient algorithms
- Reduce loop iterations
- Optimize data access patterns

### Batching Opportunities
- Group multiple operations
- Use bulk operations where possible

### Parallelization Possibilities
- Identify independent operations
- Suggest async processing

## Accuracy Tracking

The engine tracks which recommendations are applied to measure accuracy and improve suggestions over time.

To mark a recommendation as applied, update the metrics manually or integrate with your development workflow.

## Integration

The engine is integrated into the CI/CD pipeline:

- Runs on every pull request that modifies contracts
- Posts recommendations as PR comments
- Tracks metrics for continuous improvement