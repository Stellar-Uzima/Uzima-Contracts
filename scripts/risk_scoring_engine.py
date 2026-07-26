#!/usr/bin/env python3
"""
Contract Risk Scoring Model Engine

Evaluates Soroban smart contracts across three dimensions:
1. Complexity Score (SLOC, control flow statements, data types)
2. History Score (Git commit history, past security patches, modification frequency)
3. Blast Radius (Cross-contract calls, authorization depth, TVL/state dependencies)
"""

import json
import math
import os
import re
import subprocess
from pathlib import Path
from typing import Dict, Any


class ContractRiskAnalyzer:
    def __init__(self, contract_path: Path):
        self.contract_path = contract_path
        self.code = self._load_code()

    def _load_code(self) -> str:
        code_str = ""
        for rs_file in self.contract_path.glob("**/*.rs"):
            with open(rs_file, "r", encoding="utf-8") as f:
                code_str += f.read() + "\n"
        return code_str

    def compute_complexity_score(self) -> float:
        """Calculates complexity based on SLOC and branching logic."""
        lines = [line.strip() for line in self.code.splitlines() if line.strip() and not line.startswith("//")]
        sloc = len(lines)
        
        # Count branching operators (if, match, loop, while, for)
        branching_count = len(re.findall(r"\b(if|match|loop|while|for)\b", self.code))
        
        # Normalized score on scale 0 - 100
        complexity = (sloc * 0.1) + (branching_count * 2.5)
        return min(100.0, round(complexity, 2))

    def compute_history_score(self) -> float:
        """Evaluates revision churn and past bug fix commitments via Git."""
        try:
            cmd = ["git", "log", "--oneline", "--", str(self.contract_path)]
            git_log = subprocess.check_output(cmd, text=True)
            commits = git_log.strip().splitlines()
            commit_count = len(commits)
            
            # Count fix/security commits
            fix_commits = sum(1 for c in commits if re.search(r"\b(fix|bug|sec|patch)\b", c, re.IGNORECASE))
            
            history_score = (commit_count * 1.5) + (fix_commits * 5.0)
            return min(100.0, round(history_score, 2))
        except Exception:
            return 10.0  # Default baseline for non-git environments

    def compute_blast_radius(self) -> float:
        """Measures cross-contract invocation footprint and admin permissions."""
        auth_checks = len(re.findall(r"\brequire_auth\b", self.code))
        cross_calls = len(re.findall(r"Client::new\(", self.code))
        storage_keys = len(re.findall(r"env\.storage\(\)", self.code))

        blast_radius = (auth_checks * 15.0) + (cross_calls * 20.0) + (storage_keys * 2.0)
        return min(100.0, round(blast_radius, 2))

    def evaluate_composite_risk(self) -> Dict[str, Any]:
        complexity = self.compute_complexity_score()
        history = self.compute_history_score()
        blast_radius = self.compute_blast_radius()

        # Weighted calculation: 30% Complexity, 20% History, 50% Blast Radius
        composite_score = (complexity * 0.30) + (history * 0.20) + (blast_radius * 0.50)
        
        if composite_score >= 70.0:
            risk_tier = "CRITICAL"
        elif composite_score >= 40.0:
            risk_tier = "HIGH"
        elif composite_score >= 20.0:
            risk_tier = "MEDIUM"
        else:
            risk_tier = "LOW"

        return {
            "contract": self.contract_path.name,
            "composite_score": round(composite_score, 2),
            "risk_tier": risk_tier,
            "metrics": {
                "complexity": complexity,
                "history": history,
                "blast_radius": blast_radius,
            },
        }


def analyze_all_contracts(contracts_dir: str = "contracts") -> str:
    results = []
    base_path = Path(contracts_dir)
    
    if not base_path.exists():
        return json.dumps({"error": f"Path {contracts_dir} does not exist"}, indent=2)

    for item in base_path.iterdir():
        if item.is_dir() and (item / "Cargo.toml").exists():
            analyzer = ContractRiskAnalyzer(item)
            results.append(analyzer.evaluate_composite_risk())

    return json.dumps(results, indent=2)


if __name__ == "__main__":
    print(analyze_all_contracts())