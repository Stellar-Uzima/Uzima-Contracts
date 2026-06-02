#!/usr/bin/env python3
"""
Contract Event Analysis Tool

Analyzes Soroban contracts to identify:
- State-changing functions
- Event emission patterns
- Missing event definitions
- Event naming conventions
"""

import os
import re
import json
from pathlib import Path
from typing import Dict, List, Set, Tuple
from dataclasses import dataclass, asdict


@dataclass
class FunctionAnalysis:
    """Analysis of a contract function"""
    name: str
    is_state_changing: bool
    has_event_emit: bool
    event_topics: List[str]
    storage_ops: List[str]  # set, update, remove operations


def analyze_contract(contract_path: Path) -> Dict:
    """Analyze a contract for state-changing functions and events"""
    lib_rs = contract_path / "src" / "lib.rs"
    events_rs = contract_path / "src" / "events.rs"
    
    result = {
        "contract": contract_path.name,
        "has_events_rs": events_rs.exists(),
        "state_changing_functions": [],
        "event_emissions": [],
        "missing_events": [],
    }
    
    if not lib_rs.exists():
        return result
    
    try:
        with open(lib_rs, 'r', encoding='utf-8') as f:
            content = f.read()
    except:
        return result
    
    # Find all public functions that mutate state
    # Pattern: pub fn XXX(...) with storage operations or named like init, set, update, create, delete, etc.
    
    # State-changing function names
    state_changing_patterns = [
        r'pub\s+fn\s+initialize',
        r'pub\s+fn\s+init',
        r'pub\s+fn\s+set_',
        r'pub\s+fn\s+create_',
        r'pub\s+fn\s+update_',
        r'pub\s+fn\s+delete_',
        r'pub\s+fn\s+remove_',
        r'pub\s+fn\s+grant_',
        r'pub\s+fn\s+revoke_',
        r'pub\s+fn\s+transfer_',
        r'pub\s+fn\s+submit_',
        r'pub\s+fn\s+execute_',
        r'pub\s+fn\s+finalize_',
        r'pub\s+fn\s+confirm_',
        r'pub\s+fn\s+approve_',
        r'pub\s+fn\s+reject_',
    ]
    
    state_changing_funcs = set()
    for pattern in state_changing_patterns:
        matches = re.finditer(pattern, content)
        for match in matches:
            func_name = re.search(r'fn\s+(\w+)', match.group()).group(1)
            state_changing_funcs.add(func_name)
    
    # Check for storage operations
    storage_ops_pattern = r'storage\(\)\.(?:instance|persistent|temporary)\(\)\.(?:set|update|remove|delete)'
    storage_ops = len(re.findall(storage_ops_pattern, content)) > 0
    
    # Check for event emissions
    event_emit_pattern = r'env\.events\(\)\.publish\s*\(\s*\((.*?)\)\s*,'
    event_emissions = re.findall(event_emit_pattern, content)
    
    result["state_changing_functions"] = sorted(list(state_changing_funcs))
    result["event_emissions"] = event_emissions[:5]  # First 5 examples
    result["has_storage_ops"] = storage_ops
    result["needs_events_rs"] = not events_rs.exists() and (state_changing_funcs or storage_ops)
    
    return result


def analyze_all_contracts(contracts_dir: Path) -> List[Dict]:
    """Analyze all contracts in directory"""
    results = []
    
    for contract_dir in sorted(contracts_dir.iterdir()):
        if not contract_dir.is_dir():
            continue
        
        analysis = analyze_contract(contract_dir)
        results.append(analysis)
    
    return results


def generate_report(analyses: List[Dict]) -> Dict:
    """Generate summary report"""
    total = len(analyses)
    with_events = sum(1 for a in analyses if a["has_events_rs"])
    needs_events = sum(1 for a in analyses if a["needs_events_rs"])
    
    missing_event_contracts = [a for a in analyses if a["needs_events_rs"]]
    
    return {
        "summary": {
            "total_contracts": total,
            "with_events_rs": with_events,
            "without_events_rs": total - with_events,
            "needs_events_rs": needs_events,
        },
        "contracts_needing_events": [
            {
                "name": a["contract"],
                "state_changing_functions": len(a["state_changing_functions"]),
                "functions": a["state_changing_functions"][:10],  # First 10
                "examples": a["event_emissions"][:2] if a["event_emissions"] else [],
            }
            for a in missing_event_contracts
        ],
    }


if __name__ == "__main__":
    contracts_dir = Path("contracts")
    
    print("Analyzing contracts...")
    analyses = analyze_all_contracts(contracts_dir)
    
    report = generate_report(analyses)
    
    print(json.dumps(report, indent=2))
    
    # Save report
    with open("event_analysis_report.json", "w") as f:
        json.dump(report, f, indent=2)
    
    print("\nReport saved to event_analysis_report.json")
