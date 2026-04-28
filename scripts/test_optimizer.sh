#!/bin/bash

# Test script for Contract Optimization Recommendations Engine

set -e

echo "🧪 Testing Contract Optimization Engine"

# Build the optimizer
echo "Building optimizer..."
cargo build --package contract_optimizer

# Run analysis
echo "Running analysis..."
cargo run --package contract_optimizer -- analyze --format json > test_optimization_results.json

# Check if results were generated
if [ -f "test_optimization_results.json" ]; then
    echo "✅ Analysis completed successfully"
    echo "Results saved to test_optimization_results.json"

    # Show summary
    recommendations=$(jq '. | length' test_optimization_results.json)
    echo "Found recommendations for $recommendations contracts"
else
    echo "❌ Analysis failed"
    exit 1
fi

# Generate report
echo "Generating report..."
cargo run --package contract_optimizer -- report --input test_optimization_results.json --output test_report.md

if [ -f "test_report.md" ]; then
    echo "✅ Report generated successfully"
    echo "Report saved to test_report.md"
else
    echo "❌ Report generation failed"
    exit 1
fi

# Show metrics
echo "Checking metrics..."
cargo run --package contract_optimizer -- metrics

echo "🎉 All tests passed!"
echo ""
echo "Test files created:"
echo "  - test_optimization_results.json"
echo "  - test_report.md"
echo ""
echo "Clean up test files with: rm test_optimization_results.json test_report.md"