#!/usr/bin/env python3
"""
Contract Risk Scoring Model Engine

Evaluates Soroban smart contracts across three dimensions:
1. Complexity Score (SLOC, control flow statements, data types)
2. History Score (Git commit history, past security patches, modification frequency)
3. Blast Radius (Cross-contract calls, authorization depth, TVL/state dependencies)
"""

import argparse
import json
import math
import os
import re
import subprocess
import sys
from pathlib import Path
from typing import Dict, Any

# Single source of truth for the scoring model. These were previously hardcoded
# inline in evaluate_composite_risk() and separately transcribed into
# dashboard/risk_matrix.json, so the file and the engine could disagree.
SCORING_WEIGHTS: Dict[str, float] = {
    "complexity": 0.30,
    "history": 0.20,
    "blast_radius": 0.50,
}

# Lower bound of each tier, in descending order.
RISK_THRESHOLDS: Dict[str, float] = {
    "CRITICAL": 70.0,
    "HIGH": 40.0,
    "MEDIUM": 20.0,
    "LOW": 0.0,
}

RISK_MATRIX_VERSION = "1.0.0"


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
        composite_score = (
            (complexity * SCORING_WEIGHTS["complexity"])
            + (history * SCORING_WEIGHTS["history"])
            + (blast_radius * SCORING_WEIGHTS["blast_radius"])
        )

        if composite_score >= RISK_THRESHOLDS["CRITICAL"]:
            risk_tier = "CRITICAL"
        elif composite_score >= RISK_THRESHOLDS["HIGH"]:
            risk_tier = "HIGH"
        elif composite_score >= RISK_THRESHOLDS["MEDIUM"]:
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


def collect_contract_risks(contracts_dir: str = "contracts"):
    """Score every contract directory. Sorted so output is byte-stable."""
    base_path = Path(contracts_dir)

    if not base_path.exists():
        raise FileNotFoundError(f"Path {contracts_dir} does not exist")

    results = []
    for item in sorted(base_path.iterdir()):
        if item.is_dir() and (item / "Cargo.toml").exists():
            analyzer = ContractRiskAnalyzer(item)
            results.append(analyzer.evaluate_composite_risk())

    return results


def build_risk_matrix(contracts_dir: str = "contracts") -> Dict[str, Any]:
    """The dashboard/risk_matrix.json document."""
    return {
        "version": RISK_MATRIX_VERSION,
        "scoring_weights": dict(SCORING_WEIGHTS),
        "risk_thresholds": dict(RISK_THRESHOLDS),
        "contracts": collect_contract_risks(contracts_dir),
    }


def analyze_all_contracts(contracts_dir: str = "contracts") -> str:
    """Backwards-compatible entrypoint: a bare JSON list of scores."""
    try:
        return json.dumps(collect_contract_risks(contracts_dir), indent=2)
    except FileNotFoundError as exc:
        return json.dumps({"error": str(exc)}, indent=2)


def main(argv=None) -> int:
    parser = argparse.ArgumentParser(
        description="Score contracts for dashboard/risk_matrix.json."
    )
    parser.add_argument(
        "--contracts-dir",
        default="contracts",
        help="Directory holding one subdirectory per contract (default: contracts)",
    )
    parser.add_argument(
        "--out",
        default=None,
        help="Write the full risk-matrix document here. Without it, prints a "
        "bare JSON list as before.",
    )
    args = parser.parse_args(argv)

    try:
        if args.out:
            matrix = build_risk_matrix(args.contracts_dir)
            destination = Path(args.out)
            destination.parent.mkdir(parents=True, exist_ok=True)
            destination.write_text(
                json.dumps(matrix, indent=2) + "\n", encoding="utf-8"
            )
            print(
                f"risk matrix written to {destination} "
                f"({len(matrix['contracts'])} contracts)"
            )
        else:
            print(analyze_all_contracts(args.contracts_dir))
    except FileNotFoundError as exc:
        print(f"error: {exc}", file=sys.stderr)
        return 1

    return 0


if __name__ == "__main__":
    sys.exit(main())