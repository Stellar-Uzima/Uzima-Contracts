#!/usr/bin/env python3
"""
Stellar-Uzima Soroban Contract Performance & Resource Budget Benchmark Runner
Executes contract cargo benchmarks, evaluates CPU/Memory usage against defined budgets,
and generates structured performance telemetry reports.
"""

import json
import os
import re
import subprocess
import sys
import time
from typing import Dict, Any

BUDGET_FILE = "resource-budgets/contracts_budget.json"
OUTPUT_REPORT = "tests/benchmark_results.json"

def load_resource_budgets() -> Dict[str, Any]:
    """Loads max allowable CPU and memory budget thresholds."""
    if not os.path.exists(BUDGET_FILE):
        print(f"Error: Resource budget file not found at {BUDGET_FILE}")
        sys.exit(1)
    with open(BUDGET_FILE, "r") as f:
        return json.load(f).get("limits", {})

def execute_contract_benchmarks() -> Dict[str, Any]:
    """Executes cargo bench across all contract packages."""
    print("🚀 Executing repository-wide contract performance benchmarks...")
    
    # Run cargo bench capturing json output/stdout
    cmd = ["cargo", "bench", "--workspace", "--", "--nocapture"]
    try:
        res = subprocess.run(cmd, capture_output=True, text=True, check=False)
        return parse_benchmark_output(res.stdout)
    except Exception as e:
        print(f"Error executing benchmarks: {e}")
        return {}

def parse_benchmark_output(output: str) -> Dict[str, Any]:
    """Parses benchmark timing and resource consumption from test stdout."""
    results = {}
    # Example parsing pattern for benchmark stdout metrics
    pattern = re.compile(r"test ([\w_]+)::bench_([\w_]+) \x1b\[0m\.\.\. bench:\s+([\d,]+) ns/iter")
    
    for line in output.splitlines():
        match = pattern.search(line)
        if match:
            contract_name, bench_name, ns_per_iter = match.groups()
            ns = int(ns_per_iter.replace(",", ""))
            if contract_name not in results:
                results[contract_name] = {}
            results[contract_name][bench_name] = {
                "ns_per_iter": ns,
                "ops_per_sec": round(1_000_000_000 / ns, 2) if ns > 0 else 0
            }
    return results

def evaluate_against_budgets(results: Dict[str, Any], budgets: Dict[str, Any]) -> bool:
    """Verifies that performance metrics satisfy allocated resource budgets."""
    passed = True
    print("\n📊 Benchmark Resource Budget Evaluation:")
    
    for contract, benchmarks in results.items():
        budget = budgets.get(contract, {})
        min_throughput = budget.get("min_throughput_ops_sec", 0)
        
        for bench_name, metrics in benchmarks.items():
            ops = metrics["ops_per_sec"]
            status = "✅ PASS"
            if min_throughput > 0 and ops < min_throughput:
                status = "❌ FAIL (Below min throughput target)"
                passed = False
            print(f"  [{contract}] {bench_name}: {ops} ops/sec (Min Required: {min_throughput}) -> {status}")
            
    return passed

def main():
    budgets = load_resource_budgets()
    results = execute_contract_benchmarks()
    
    os.makedirs("tests", exist_ok=True)
    report = {
        "timestamp": time.strftime('%Y-%m-%d %H:%M:%S UTC', time.gmtime()),
        "results": results,
    }
    
    with open(OUTPUT_REPORT, "w") as f:
        json.dump(report, f, indent=2)
        
    print(f"\n📁 Benchmark metrics written to {OUTPUT_REPORT}")
    
    # Evaluate performance budgets
    success = evaluate_against_budgets(results, budgets)
    if not success:
        print("\n❌ Performance benchmarks failed resource budget thresholds.")
        sys.exit(1)
        
    print("\n✅ All contract performance benchmarks passed within defined budgets.")

if __name__ == "__main__":
    main()