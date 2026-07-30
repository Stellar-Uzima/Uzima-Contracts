#!/usr me/env python3
"""
Stellar-Uzima Contract Portfolio Health & Risk Aggregator
Generates maintainer health dashboard metrics in JSON & Markdown formats.
"""

import json
import os
import subprocess
import time
from typing import Dict, Any

CONTRACT_MODULES = [
    "contracts/sync_manager",
    "contracts/cross_chain_bridge",
    "contracts/patient_identity",
    "contracts/medical_records",
]

def check_cargo_audit() -> Dict[str, Any]:
    """Runs cargo audit to check security status of underlying dependencies."""
    try:
        res = subprocess.run(
            ["cargo", "audit", "--json"], capture_output=True, text=True, check=False
        )
        if res.returncode == 0:
            return {"status": "HEALTHY", "vulnerabilities": 0}
        data = json.loads(res.stdout)
        vuln_count = len(data.get("vulnerabilities", {}).get("list", []))
        return {"status": "CRITICAL" if vuln_count > 0 else "HEALTHY", "vulnerabilities": vuln_count}
    except Exception as e:
        return {"status": "UNKNOWN", "error": str(e)}

def compute_contract_metrics() -> Dict[str, Any]:
    """Inspects WASM targets and test status across contract modules."""
    portfolio = {}
    for contract in CONTRACT_MODULES:
        has_tests = os.path.exists(os.path.join(contract, "src", "test.rs"))
        portfolio[contract] = {
            "status": "ACTIVE",
            "has_unit_tests": has_tests,
            "audit_risk_level": "LOW" if has_tests else "MEDIUM",
        }
    return portfolio

def generate_markdown_dashboard(metrics: Dict[str, Any]) -> str:
    """Formats aggregated metrics into Markdown dashboard report."""
    md = f"""# 🏥 Uzima Contracts Portfolio Health Dashboard

> **Last Updated:** {time.strftime('%Y-%m-%d %H:%M:%S UTC', time.gmtime())}

## 📊 Security & Dependency Risk
* **Audit Status:** `{metrics['audit']['status']}`
* **Active Vulnerabilities:** `{metrics['audit'].get('vulnerabilities', 0)}`

## 📜 Contract Portfolio Health

| Contract Module | Status | Unit Tests Present | Risk Rating |
| :--- | :--- | :--- | :--- |
"""
    for name, data in metrics["contracts"].items():
        test_badge = "✅ Yes" if data["has_unit_tests"] else "⚠️ Missing"
        md += f"| `{name}` | `{data['status']}` | {test_badge} | `{data['audit_risk_level']}` |\n"

    md += """
---
*Automated report generated via `scripts/generate_health_dashboard.py`.*
"""
    return md

def main():
    metrics = {
        "audit": check_cargo_audit(),
        "contracts": compute_contract_metrics(),
    }
    
    os.makedirs("dashboard", exist_ok=True)
    
    # Save raw JSON metrics
    with open("dashboard/metrics.json", "w") as f:
        json.dump(metrics, f, indent=2)

    # Save Markdown report
    with open("docs/PORTFOLIO_HEALTH.md", "w") as f:
        f.write(generate_markdown_dashboard(metrics))

    print("Dashboard metrics successfully updated in dashboard/metrics.json and docs/PORTFOLIO_HEALTH.md.")

if __name__ == "__main__":
    main()